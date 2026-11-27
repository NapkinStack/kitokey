//! `kitokey-send`'s core, as `posting/v1` says: which addresses are refused, what a meeting is sent as,
//! and the one POST. The program (`src/main.rs`) uses [`Policy::strict`] alone; a test injects its own policy through this library to reach a
//! server on this computer, never through the program.

pub mod notion;

use std::fmt;
use std::fs;
use std::io;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::path::Path;
use std::time::Duration;

use serde_json::{Value, json};

use ureq::Agent;
use ureq::unversioned::resolver::{DefaultResolver, ResolvedSocketAddrs, Resolver};
use ureq::unversioned::transport::{DefaultConnector, NextTimeout};
use url::{Host, Url};

// ---------------------------------------------------------------------------------------------------
// Which addresses it refuses

/// Why an address is refused (`address.refused`), said to the person in the program's sentence — which
/// never holds the address.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refusal {
    Unreadable,
    NotHttps,
    Credentials,
    Local,
}

impl fmt::Display for Refusal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match self {
            Refusal::Unreadable => "the address is not a web address",
            Refusal::NotHttps => "the address is not https",
            Refusal::Credentials => "the address holds a user or a password",
            Refusal::Local => "the address is this computer's or a local network's",
        })
    }
}

/// The address as the connection reads it — a name lowercased and as punycode, an IPv4 written in hex or
/// decimal as the address it stands for (WHATWG's parser) —, or why it is refused. A name is judged again
/// on every address it resolves to, when it is sent ([`post`]).
pub fn judged(address: &str) -> Result<Url, Refusal> {
    let url = Url::parse(address).map_err(|_| Refusal::Unreadable)?;
    if url.scheme() != "https" {
        return Err(Refusal::NotHttps);
    }
    if !url.username().is_empty() || url.password().is_some() {
        return Err(Refusal::Credentials);
    }
    let ip = match url.host() {
        None => return Err(Refusal::Unreadable),
        Some(Host::Ipv4(ip)) => Some(IpAddr::V4(ip)),
        Some(Host::Ipv6(ip)) => Some(IpAddr::V6(ip)),
        Some(Host::Domain(_)) => None,
    };
    if ip.is_some_and(local) {
        return Err(Refusal::Local);
    }
    Ok(url)
}

/// Whether an address is this computer's or a local network's: the page's ranges, an IPv6 carrying an
/// IPv4 judged as that IPv4.
pub fn local(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ip) => local_v4(ip),
        IpAddr::V6(ip) => match carried(ip) {
            Some(v4) => local_v4(v4),
            None => {
                let first = ip.segments()[0];
                ip.is_loopback()
                    || ip.is_unspecified()
                    || first & 0xffc0 == 0xfe80
                    || first & 0xfe00 == 0xfc00
                    || ip.is_multicast()
                    || ip.segments()[..3] == [0x64, 0xff9b, 1]
            }
        },
    }
}

fn local_v4(ip: Ipv4Addr) -> bool {
    let [a, b, ..] = ip.octets();
    ip.is_loopback()
        || a == 0
        || ip.is_private()
        || (a == 100 && b & 0xc0 == 64)
        || ip.is_link_local()
        || ip.is_multicast()
        || ip.is_broadcast()
}

/// The IPv4 address an IPv6 one carries: mapped (`::ffff:0:0/96`), or NAT64's well-known prefix
/// (`64:ff9b::/96`).
fn carried(ip: Ipv6Addr) -> Option<Ipv4Addr> {
    let segments = ip.segments();
    if segments[..6] == [0, 0, 0, 0, 0, 0xffff] || segments[..6] == [0x64, 0xff9b, 0, 0, 0, 0] {
        let [.., a, b, c, d] = ip.octets();
        return Some(Ipv4Addr::new(a, b, c, d));
    }
    None
}

// ---------------------------------------------------------------------------------------------------
// The one POST

/// What a send may reach. The program's is [`Policy::strict`]; a test's reaches a server on this computer.
#[derive(Clone, Copy, Debug)]
pub struct Policy {
    /// Whether a resolved address may be connected to.
    pub reachable: fn(IpAddr) -> bool,
    /// `https:` alone.
    pub https_only: bool,
    /// The whole send: connection, request and answer.
    pub bound: Duration,
}

impl Policy {
    /// `posting/v1`'s: no local address, `https:` alone, 30 s.
    pub fn strict() -> Self {
        Policy {
            reachable: |ip| !local(ip),
            https_only: true,
            bound: Duration::from_secs(30),
        }
    }
}

/// A request's body and its type.
#[derive(Clone, Debug, PartialEq)]
pub struct Body {
    pub content_type: String,
    pub bytes: Vec<u8>,
}

/// How a send ended, each a code of `posting/v1` but [`Outcome::Sent`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Sent,
    /// A resolved address was local: nothing sent.
    Refused,
    /// No connection: nothing sent.
    Unreachable,
    /// Sent, and no answer within the bound: maybe arrived.
    Silent,
    /// 401, 403, 404, 410.
    Gone,
    /// 429, and the seconds the destination said to wait, if it said them.
    Busy(Option<u64>),
    /// Any other 3xx or 4xx: a redirect is never followed.
    Declined(u16),
    /// 5xx, or a status no class names.
    Failed(u16),
    /// `notion`: 401, the integration's key refused.
    KeyRefused,
    /// `notion`: 404, the database not shared with the integration, or gone.
    Unshared,
    /// `notion`: 403, a capability missing, or the workspace over its block limit.
    KeyRestricted,
    /// `notion`: the database holds more than one data source.
    Several,
    /// `notion`: a read not answered within the bound — nothing created, unlike the POST's silence.
    Unanswered,
    /// `notion`: the page's request past Notion's bounds, known once the title property is read.
    TooLarge,
}

impl Outcome {
    /// The code of `posting/v1`'s `failed` line, none for [`Outcome::Sent`].
    pub fn code(self) -> Option<&'static str> {
        Some(match self {
            Outcome::Sent => return None,
            Outcome::Refused => "address.refused",
            Outcome::Unreachable => "network.unreachable",
            Outcome::Silent => "destination.silent",
            Outcome::Gone => "destination.gone",
            Outcome::Busy(_) => "destination.busy",
            Outcome::Declined(_) => "destination.refused",
            Outcome::Failed(_) | Outcome::Unanswered => "destination.failed",
            Outcome::KeyRefused => "key.refused",
            Outcome::Unshared => "database.unshared",
            Outcome::KeyRestricted => "key.restricted",
            Outcome::Several => "database.several",
            Outcome::TooLarge => "summary.too_large",
        })
    }
}

/// A name resolved once, every address judged; the connection made to those, never to a second
/// resolution. One local address refuses them all.
#[derive(Debug)]
struct Judging {
    resolver: DefaultResolver,
    reachable: fn(IpAddr) -> bool,
}

/// The resolver's refusal, told apart from the network's by its type.
#[derive(Debug)]
struct LocalAddress;

impl fmt::Display for LocalAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("a resolved address is local")
    }
}

impl std::error::Error for LocalAddress {}

/// The resolver's failure to resolve the name: nothing was sent.
#[derive(Debug)]
struct Unresolved;

impl fmt::Display for Unresolved {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("the host could not be resolved")
    }
}

impl std::error::Error for Unresolved {}

impl Resolver for Judging {
    fn resolve(
        &self,
        uri: &ureq::http::Uri,
        config: &ureq::config::Config,
        timeout: NextTimeout,
    ) -> Result<ResolvedSocketAddrs, ureq::Error> {
        // A name not resolved: nothing left, whatever error the system gave for it.
        let addresses = self
            .resolver
            .resolve(uri, config, timeout)
            .map_err(|_| ureq::Error::Other(Box::new(Unresolved)))?;
        if !admitted(&addresses, self.reachable) {
            return Err(ureq::Error::Other(Box::new(LocalAddress)));
        }
        Ok(addresses)
    }
}

/// Whether a name's resolved addresses may be connected to: every one of them reachable, so that a name
/// answering a public address and a local one is refused whole.
pub fn admitted(addresses: &[SocketAddr], reachable: fn(IpAddr) -> bool) -> bool {
    addresses.iter().all(|address| reachable(address.ip()))
}

/// Sends `body` once to `address`, judged first ([`judged`]): no proxy, no redirect, within the bound.
/// Only the answer's status, and its `Retry-After`, are read; never its body.
///
/// The connection, its TLS handshake included, has a third of the bound: one not made by then ends as a
/// connection timeout, read [`Outcome::Unreachable`] — nothing left —, never as the whole's, which would say
/// « may have arrived » (#1167). Windows retries a refused connection's SYN for about 2 s (curl's maintainer,
/// daniel.haxx.se, 2024-08-14): within the program's third, 10 s.
pub fn post(address: &Url, body: &Body, policy: Policy) -> Outcome {
    let answer = agent(policy)
        .post(address.as_str())
        .header("Content-Type", &body.content_type)
        .send(&body.bytes[..]);
    match answer {
        Ok(answer) => {
            let status = answer.status().as_u16();
            match status {
                200..=299 => Outcome::Sent,
                401 | 403 | 404 | 410 => Outcome::Gone,
                429 => Outcome::Busy(retry_after(&answer)),
                300..=499 => Outcome::Declined(status),
                _ => Outcome::Failed(status),
            }
        }
        Err(error) => unsent(error),
    }
}

/// `Retry-After`, in whole seconds, when the answer says it so.
pub(crate) fn retry_after<B>(answer: &ureq::http::Response<B>) -> Option<u64> {
    answer
        .headers()
        .get("retry-after")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.trim().parse().ok())
}

/// A request's error as an outcome: the judging resolver's refusals first, then [`failed`].
pub(crate) fn unsent(error: ureq::Error) -> Outcome {
    match error {
        ureq::Error::Other(error) if error.is::<LocalAddress>() => Outcome::Refused,
        ureq::Error::Other(error) if error.is::<Unresolved>() => Outcome::Unreachable,
        error => failed(&error),
    }
}

/// The agent every request goes through: no proxy, no redirect, its addresses judged, within the bound, its
/// connection within a third of it.
pub(crate) fn agent(policy: Policy) -> Agent {
    let config = Agent::config_builder()
        .proxy(None)
        .max_redirects(0)
        .http_status_as_error(false)
        .https_only(policy.https_only)
        .timeout_global(Some(policy.bound))
        .timeout_connect(Some(policy.bound / 3))
        .build();
    Agent::with_parts(
        config,
        DefaultConnector::default(),
        Judging {
            resolver: DefaultResolver::default(),
            reachable: policy.reachable,
        },
    )
}

/// Whether a failure came before anything was sent, or after: only a failure to resolve or to connect
/// says that nothing left.
fn failed(error: &ureq::Error) -> Outcome {
    use ureq::Error::*;
    match error {
        HostNotFound | ConnectionFailed | Tls(_) | Rustls(_) => Outcome::Unreachable,
        Timeout(ureq::Timeout::Resolve | ureq::Timeout::Connect) => Outcome::Unreachable,
        Io(error)
            if matches!(
                error.kind(),
                io::ErrorKind::ConnectionRefused
                    | io::ErrorKind::NotFound
                    | io::ErrorKind::AddrNotAvailable
                    | io::ErrorKind::NetworkUnreachable
                    | io::ErrorKind::HostUnreachable
            ) =>
        {
            Outcome::Unreachable
        }
        _ => Outcome::Silent,
    }
}

// ---------------------------------------------------------------------------------------------------
// The meeting

/// What of a meeting leaves: its title, its sections, its actions. Never the audio, never the transcript.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Meeting {
    /// The minutes' `title`, within `minutes`' bounds.
    pub title: Option<String>,
    /// The sections, in their order: each its `## ` title and its body.
    pub sections: Vec<(String, String)>,
    /// The section that holds the actions: v1–v4's second, v5's named by `actions`.
    pub actions: Option<usize>,
    /// The section that holds the key points: v1–v4's third, v5's of id `key_points`.
    pub key_points: Option<usize>,
    /// What `only` kept of the parts chosen, for the webhook's `key_points` and `dates`; none when the
    /// input chose none, and the summary is sent whole, as before.
    pub chosen: Option<(Vec<Part>, Vec<Date>)>,
}

/// A part of a summary a person may choose to send (`posting/v1`'s `parts`, D132); the title is always sent.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Part {
    /// Every section but the actions and the key points.
    Summary,
    Actions,
    KeyPoints,
    Dates,
}

/// A date of `posting/v1`'s `dates`: its passage — the transcript's words — never read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Date {
    pub date: String,
    pub what: String,
    pub who: Option<String>,
}

/// The parts an input chose, and the dates with their heading when it gives them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Chosen {
    pub parts: Vec<Part>,
    pub dates: Option<(String, Vec<Date>)>,
}

impl Chosen {
    /// `parts`, `dates` and `dates_title` read from posting/v1's input: none when `parts` is absent, the
    /// summary then sent whole; an error, the field it names, when one is not as `input.schema.json` says.
    pub fn read(input: &Value) -> Result<Option<Chosen>, &'static str> {
        let line = |value: &Value| {
            value
                .as_str()
                .filter(|text| !text.is_empty() && !text.contains(['\n', '\r']))
                .map(str::to_string)
        };
        let dates = match input.get("dates") {
            None => None,
            Some(Value::Array(items)) if !items.is_empty() => Some(items.iter().map(date).collect::<Option<Vec<_>>>().ok_or("« dates »")?),
            Some(_) => return Err("« dates »"),
        };
        let title = match input.get("dates_title") {
            None => None,
            Some(value) => Some(line(value).ok_or("« dates_title »")?),
        };
        let Some(parts) = input.get("parts") else { return Ok(None) };
        let mut chosen = Vec::new();
        for part in parts.as_array().filter(|parts| !parts.is_empty()).ok_or("« parts »")? {
            let part = match part.as_str() {
                Some("summary") => Part::Summary,
                Some("actions") => Part::Actions,
                Some("key_points") => Part::KeyPoints,
                Some("dates") => Part::Dates,
                _ => return Err("« parts »"),
            };
            if chosen.contains(&part) {
                return Err("« parts »");
            }
            chosen.push(part);
        }
        let dates = match (chosen.contains(&Part::Dates), dates) {
            (true, Some(dates)) => Some((title.ok_or("« dates_title »")?, dates)),
            _ => None,
        };
        Ok(Some(Chosen { parts: chosen, dates }))
    }
}

/// One item of `dates`: `id` and `date` of their shape, `what` on one line, `who` and `passage` too when given.
fn date(item: &Value) -> Option<Date> {
    let text = |name: &str| item.get(name).and_then(Value::as_str);
    let line = |text: &str| !text.is_empty() && !text.contains(['\n', '\r']);
    let id = text("id")?;
    let mut chars = id.chars();
    let id_ok = id.len() <= 64
        && chars.next().is_some_and(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
        && chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-');
    let day = text("date")?;
    let day_ok = day.len() == 10
        && day
            .char_indices()
            .all(|(at, c)| if at == 4 || at == 7 { c == '-' } else { c.is_ascii_digit() });
    let what = text("what").filter(|what| line(what))?;
    let who = match item.get("who") {
        None | Some(Value::Null) => None,
        Some(Value::String(who)) if line(who) => Some(who.clone()),
        Some(_) => return None,
    };
    let passage_ok = match item.get("passage") {
        None | Some(Value::Null) => true,
        Some(Value::String(passage)) => !passage.is_empty(),
        Some(_) => false,
    };
    (item.is_object() && id_ok && day_ok && passage_ok).then(|| Date {
        date: day.into(),
        what: what.into(),
        who,
    })
}

/// Why a meeting is not sent.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Unsent {
    Unreadable,
    Confidential,
    NoSummary,
}

impl Meeting {
    /// Reads a session's folder: `session-audio`'s `confidential` first, then `minutes.md` (v1 to v5). A
    /// confidential meeting is refused: `read_confirmed` with `false`.
    pub fn read(folder: &Path) -> Result<Meeting, Unsent> {
        Meeting::read_confirmed(folder, false)
    }

    /// Reads a session's folder as `read` does, a confidential meeting read too when the person confirmed it
    /// for this send (`posting/v1`'s `confirmed_confidential`, D118; PDR-0007 point 4 amended 2026-11-24).
    pub fn read_confirmed(folder: &Path, confirmed: bool) -> Result<Meeting, Unsent> {
        match fs::metadata(folder.join("confidential")) {
            Ok(_) if !confirmed => return Err(Unsent::Confidential),
            Ok(_) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(_) => return Err(Unsent::Unreadable),
        }
        if !folder.is_dir() {
            return Err(Unsent::Unreadable);
        }
        let minutes = match fs::read_to_string(folder.join("minutes.md")) {
            Ok(text) => text,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Err(Unsent::NoSummary),
            Err(_) => return Err(Unsent::Unreadable),
        };
        let meeting = minutes_read(&minutes);
        if meeting.sections.is_empty() {
            return Err(Unsent::NoSummary);
        }
        Ok(meeting)
    }

    /// The items of the actions' section, each without its bullet.
    pub fn action_items(&self) -> Vec<String> {
        let Some((_, body)) = self.actions.and_then(|at| self.sections.get(at)) else {
            return Vec::new();
        };
        body.lines()
            .filter_map(|line| line.strip_prefix("- ").or_else(|| line.strip_prefix("* ")))
            .map(|item| item.trim().to_string())
            .filter(|item| !item.is_empty())
            .collect()
    }

    /// The meeting with the parts `chosen` alone, its sections in their order, the dates last under their
    /// heading, each « date — what — who »; a part asked that the meeting does not hold is empty.
    pub fn only(&self, chosen: &Chosen) -> Meeting {
        let has = |part| chosen.parts.contains(&part);
        let mut meeting = Meeting {
            title: self.title.clone(),
            ..Meeting::default()
        };
        for (at, section) in self.sections.iter().enumerate() {
            let part = if self.actions == Some(at) {
                Part::Actions
            } else if self.key_points == Some(at) {
                Part::KeyPoints
            } else {
                Part::Summary
            };
            if !has(part) {
                continue;
            }
            match part {
                Part::Actions => meeting.actions = Some(meeting.sections.len()),
                Part::KeyPoints => meeting.key_points = Some(meeting.sections.len()),
                _ => {}
            }
            meeting.sections.push(section.clone());
        }
        let dates = match (&chosen.dates, has(Part::Dates)) {
            (Some((heading, dates)), true) => {
                let lines: Vec<String> = dates
                    .iter()
                    .map(|date| match &date.who {
                        Some(who) => format!("- {} — {} — {who}", date.date, date.what),
                        None => format!("- {} — {}", date.date, date.what),
                    })
                    .collect();
                meeting.sections.push((heading.clone(), lines.join("\n")));
                dates.clone()
            }
            _ => Vec::new(),
        };
        meeting.chosen = Some((chosen.parts.clone(), dates));
        meeting
    }
}

/// `minutes`: a front matter of `key: value` lines, then sections of second level.
fn minutes_read(text: &str) -> Meeting {
    let mut meeting = Meeting::default();
    let mut lines = text.lines();
    let (mut contract, mut ids, mut actions) = (None, None, None);
    if lines.next() == Some("---") {
        for line in lines.by_ref().take_while(|line| *line != "---") {
            let Some((key, value)) = line.split_once(':') else { continue };
            let value = value.trim();
            match key {
                "contract" => contract = Some(value.to_string()),
                "title" => {
                    let title = if value.starts_with('"') {
                        serde_json::from_str::<String>(value).ok()
                    } else {
                        Some(value.to_string())
                    };
                    meeting.title = title
                        .filter(|title| !title.trim().is_empty() && !title.chars().any(char::is_control) && title.chars().count() <= 80);
                }
                "sections" => {
                    ids = serde_json::from_str::<Vec<Value>>(value).ok().map(|sections| {
                        sections
                            .iter()
                            .map(|section| section["id"].as_str().unwrap_or_default().to_string())
                            .collect::<Vec<_>>()
                    })
                }
                "actions" => actions = Some(value.to_string()),
                _ => {}
            }
        }
    } else {
        lines = text.lines();
    }
    for line in lines {
        if let Some(title) = line.strip_prefix("## ") {
            meeting.sections.push((title.trim().to_string(), String::new()));
        } else if let Some((_, body)) = meeting.sections.last_mut() {
            body.push_str(line);
            body.push('\n');
        }
    }
    for (_, body) in &mut meeting.sections {
        *body = body.trim().to_string();
    }
    if contract.as_deref() == Some("minutes/v5") {
        let at = |id: &str| ids.as_ref()?.iter().position(|each| each == id);
        meeting.actions = actions.as_deref().and_then(at);
        meeting.key_points = at("key_points");
    } else {
        meeting.actions = (meeting.sections.len() >= 2).then_some(1);
        meeting.key_points = (meeting.sections.len() >= 3).then_some(2);
    }
    meeting
}

// ---------------------------------------------------------------------------------------------------
// What each kind receives

/// The destination's kind: what the address is, and so what it is sent.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Webhook,
    Discord,
    Slack,
}

/// Discord's `content`, in characters.
pub const DISCORD_CONTENT: usize = 2000;
/// Discord's file, by default, in bytes.
pub const DISCORD_FILE: usize = 20 * 1024 * 1024;
/// Past this, Slack truncates a message's `text`, in characters.
pub const SLACK_TEXT: usize = 40_000;

/// The summary's sections as Markdown, each opening on its `## ` title.
pub fn summary(meeting: &Meeting) -> String {
    let mut out = String::new();
    for (number, (title, body)) in meeting.sections.iter().enumerate() {
        if number > 0 {
            out.push('\n');
        }
        out += &format!("## {title}\n\n");
        if !body.is_empty() {
            out += body;
            out.push('\n');
        }
    }
    out
}

/// What `kind` receives of a meeting named `title`, its file named after the session's folder; none when
/// it does not fit within the destination's bounds (`summary.too_large`).
pub fn body(kind: Kind, meeting: &Meeting, title: &str, folder_name: &str) -> Option<Body> {
    let summary = summary(meeting);
    match kind {
        Kind::Webhook => Some(Body {
            content_type: "application/json".into(),
            bytes: webhook(meeting, title, summary).to_string().into_bytes(),
        }),
        Kind::Discord => {
            let file = format!("# {title}\n\n{summary}");
            if file.len() > DISCORD_FILE {
                return None;
            }
            let content = discord_content(title, &summary);
            let payload = json!({"content": content, "allowed_mentions": {"parse": []}}).to_string();
            Some(multipart(&payload, &format!("{folder_name}.md"), file.as_bytes()))
        }
        Kind::Slack => {
            let text = format!("*{}*\n\n{}", mrkdwn_escaped(title), mrkdwn(&summary));
            if text.chars().count() > SLACK_TEXT {
                return None;
            }
            Some(Body {
                content_type: "application/json".into(),
                bytes: json!({"text": text}).to_string().into_bytes(),
            })
        }
    }
}

/// A generic webhook's body: the title, the summary, the actions; and, chosen, the key points and the dates.
fn webhook(meeting: &Meeting, title: &str, summary: String) -> Value {
    let mut sent = json!({"title": title, "summary": summary, "actions": meeting.action_items()});
    if let Some((parts, dates)) = &meeting.chosen {
        if parts.contains(&Part::KeyPoints) {
            sent["key_points"] = Meeting {
                actions: meeting.key_points,
                ..meeting.clone()
            }
            .action_items()
            .into();
        }
        if parts.contains(&Part::Dates) {
            sent["dates"] = dates
                .iter()
                .map(|date| json!({"date": date.date, "what": date.what, "who": date.who}))
                .collect();
        }
    }
    sent
}

/// The title in bold, then the summary's first lines, cut at a line's end within Discord's bound.
fn discord_content(title: &str, summary: &str) -> String {
    let mut content = format!("**{title}**");
    if content.chars().count() > DISCORD_CONTENT {
        return content.chars().take(DISCORD_CONTENT).collect();
    }
    let mut first = true;
    for line in summary.lines() {
        let added = if first { format!("\n\n{line}") } else { format!("\n{line}") };
        if content.chars().count() + added.chars().count() > DISCORD_CONTENT {
            break;
        }
        content += &added;
        first = false;
    }
    content
}

/// A `multipart/form-data` body of `payload_json` and one file, as Discord's *Execute Webhook* takes it.
fn multipart(payload: &str, file_name: &str, file: &[u8]) -> Body {
    let boundary = boundary(&[payload.as_bytes(), file]);
    let mut bytes = Vec::new();
    bytes.extend_from_slice(
        format!(
            "--{boundary}\r\nContent-Disposition: form-data; name=\"payload_json\"\r\n\
             Content-Type: application/json\r\n\r\n{payload}\r\n\
             --{boundary}\r\nContent-Disposition: form-data; name=\"files[0]\"; filename=\"{file_name}\"\r\n\
             Content-Type: text/markdown; charset=utf-8\r\n\r\n"
        )
        .as_bytes(),
    );
    bytes.extend_from_slice(file);
    bytes.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());
    Body {
        content_type: format!("multipart/form-data; boundary={boundary}"),
        bytes,
    }
}

/// A boundary drawn for each send and absent from every part, as RFC 2046 asks: a meeting's text can
/// never close a part, whatever line ending a server's parser accepts.
fn boundary(parts: &[&[u8]]) -> String {
    use std::hash::{BuildHasher, Hasher};
    loop {
        let drawn = || std::collections::hash_map::RandomState::new().build_hasher().finish();
        let boundary = format!("kitokey-send-{:016x}{:016x}", drawn(), drawn());
        if !parts
            .iter()
            .any(|part| part.windows(boundary.len()).any(|window| window == boundary.as_bytes()))
        {
            return boundary;
        }
    }
}

/// `&`, `<` and `>` escaped, as Slack's mrkdwn asks: a summary's `<!channel>` is text, never a mention.
fn mrkdwn_escaped(text: &str) -> String {
    text.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

/// Markdown as Slack's mrkdwn: headings in bold, `**bold**` as `*bold*`, `[text](url)` as `<url|text>`.
pub fn mrkdwn(markdown: &str) -> String {
    let mut out = Vec::new();
    for line in markdown.lines() {
        let line = mrkdwn_escaped(line);
        let heading = line.trim_start_matches('#');
        let line = if heading.len() < line.len() && heading.starts_with(' ') {
            format!("*{}*", heading.trim())
        } else {
            line
        };
        out.push(links(&line.replace("**", "*")));
    }
    out.join("\n") + "\n"
}

/// `[text](url)` as `<url|text>`, for a url that is `http:` or `https:`; anything else left as written.
fn links(line: &str) -> String {
    let mut out = String::new();
    let mut rest = line;
    while let Some(open) = rest.find('[') {
        let after = &rest[open + 1..];
        let Some(close) = after.find("](") else { break };
        let Some(end) = after[close + 2..].find(')') else { break };
        let (text, url) = (&after[..close], &after[close + 2..close + 2 + end]);
        out += &rest[..open];
        if (url.starts_with("http://") || url.starts_with("https://")) && !url.contains(['|', ' ']) {
            out += &format!("<{url}|{text}>");
        } else {
            out += &rest[open..open + 1 + close + 2 + end + 1];
        }
        rest = &after[close + 2 + end + 1..];
    }
    out + rest
}

//! `kitokey-send`: a meeting's summary sent once to a channel, as `posting/v1` says.
//!
//! The arguments hold no word of the meeting and no address: the session's folder, named by its day and
//! hour, and the destination's kind. The address — a secret: whoever holds it posts to the channel — and
//! the meeting's name come on the input. No sentence, no line and no log holds the address.

use std::io::{self, Read};
use std::path::PathBuf;
use std::process::ExitCode;

use kitokey_send::{Kind, Meeting, Outcome, Policy, Unsent, body, judged, notion, post};
use serde_json::{Value, json};

/// The version this program says (`--version`): the package's, named when it is built (`KITOKEY_VERSION`,
/// D78), Cargo's when unset or empty.
const VERSION: &str = match option_env!("KITOKEY_VERSION") {
    Some(version) if !version.is_empty() => version,
    _ => env!("CARGO_PKG_VERSION"),
};

const USAGE: &str = "usage: kitokey-send <session-folder> --kind <webhook|discord|slack|notion>, or kitokey-send \
     --check [--kind notion]; the address and the meeting's name on the input, and notion's key";

/// Why the program ends in failure: its sentence, and the line it says when a person acts on it.
struct Refused(String, Option<Value>);

fn main() -> ExitCode {
    if std::env::args_os().skip(1).eq(["--version"]) {
        println!("kitokey-send {VERSION}");
        return ExitCode::SUCCESS;
    }
    match run() {
        Ok(line) => {
            println!("{line}");
            ExitCode::SUCCESS
        }
        Err(Refused(why, line)) => {
            eprintln!("kitokey-send: {why}");
            match line {
                Some(line) => {
                    println!("{line}");
                    ExitCode::FAILURE
                }
                None => ExitCode::from(2),
            }
        }
    }
}

/// A cause of the contract: its sentence and its `failed` line.
fn failed(why: impl Into<String>, code: &str) -> Refused {
    Refused(why.into(), Some(json!({"event": "failed", "code": code})))
}

/// Where a send goes: a channel's address, or Notion's API.
#[derive(Clone, Copy, PartialEq)]
enum To {
    Channel(Kind),
    Notion,
}

/// What the arguments ask: `--check`, for an address or for Notion, or a folder and where it goes.
enum Asked {
    Check(Option<To>),
    Send(PathBuf, To),
}

fn asked() -> Result<Asked, Refused> {
    let wrong = |why: &str| Refused(format!("{why}; {USAGE}"), None);
    let mut args = Vec::new();
    for arg in std::env::args_os().skip(1) {
        args.push(arg.into_string().map_err(|_| wrong("an argument is not text"))?);
    }
    if args.is_empty() {
        return Err(Refused(USAGE.into(), None));
    }
    let (mut folder, mut kind, mut check) = (None, None, false);
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--kind" if kind.is_some() => return Err(wrong("--kind given twice")),
            "--kind" => {
                kind = Some(match args.next().as_deref() {
                    Some("webhook") => To::Channel(Kind::Webhook),
                    Some("discord") => To::Channel(Kind::Discord),
                    Some("slack") => To::Channel(Kind::Slack),
                    Some("notion") => To::Notion,
                    _ => return Err(wrong("--kind is webhook, discord, slack or notion")),
                })
            }
            "--check" if check => return Err(wrong("--check given twice")),
            "--check" => check = true,
            option if option.starts_with("--") => return Err(wrong(&format!("{option} is no option here"))),
            _ if folder.is_some() => return Err(wrong("one session's folder only")),
            _ => folder = Some(PathBuf::from(arg)),
        }
    }
    if check {
        return match (folder, kind) {
            (Some(_), _) => Err(wrong("--check takes no session's folder")),
            (None, None | Some(To::Notion)) => Ok(Asked::Check(kind)),
            (None, Some(_)) => Err(wrong("--check takes --kind notion alone")),
        };
    }
    let folder = folder.ok_or_else(|| wrong("no session's folder"))?;
    let kind = kind.ok_or_else(|| wrong("no --kind"))?;
    Ok(Asked::Send(folder, kind))
}

/// The input: the address, the meeting's name and, for Notion, the integration's key, each on one line; and
/// whether a confidential meeting is confirmed for this send, a boolean, `false` when absent (`input.schema.json`).
fn input(notion: bool) -> Result<(String, String, Option<String>, bool), Refused> {
    let wrong = |why: &str| Refused(format!("the input is not posting/v1's: {why}"), None);
    let mut text = String::new();
    io::stdin().read_to_string(&mut text).map_err(|_| wrong("it could not be read"))?;
    let object: Value = serde_json::from_str(&text).map_err(|_| wrong("it is not one JSON object"))?;
    let field = |name: &str| match object.get(name).and_then(Value::as_str) {
        Some(value) if !value.is_empty() && !value.contains(['\n', '\r']) => Ok(value.to_string()),
        _ => Err(wrong(&format!("« {name} » missing, empty or on more than one line"))),
    };
    let key = if notion { Some(field("key")?) } else { None };
    let confirmed = match object.get("confirmed_confidential") {
        None => false,
        Some(Value::Bool(confirmed)) => *confirmed,
        Some(_) => return Err(wrong("« confirmed_confidential » is no boolean")),
    };
    Ok((field("address")?, field("name")?, key, confirmed))
}

fn run() -> Result<Value, Refused> {
    let asked = asked()?;
    let notion = matches!(asked, Asked::Check(Some(To::Notion)) | Asked::Send(_, To::Notion));
    let (address, name, key, confirmed) = input(notion)?;
    let api = || url::Url::parse(notion::API).expect("Notion's API is a URL");
    let database = || notion::database(&address).ok_or_else(|| failed("the link is not https: or names no database", "address.refused"));
    let (folder, to) = match asked {
        Asked::Check(None) => {
            judged(&address).map_err(|refusal| failed(refusal.to_string(), "address.refused"))?;
            return Ok(json!({"event": "checked"}));
        }
        Asked::Check(Some(_)) => {
            let key = key.unwrap_or_default();
            notion::check(&api(), &key, &database()?, Policy::strict()).map_err(ended)?;
            return Ok(json!({"event": "checked"}));
        }
        Asked::Send(folder, to) => (folder, to),
    };
    // The meeting before the address: a confidential meeting is sent only once the person confirmed it, for
    // this send (D118), whatever the address.
    let meeting = Meeting::read_confirmed(&folder, confirmed).map_err(|unsent| match unsent {
        Unsent::Confidential => failed(
            "the meeting is said confidential and this send is not confirmed",
            "meeting.confidential",
        ),
        Unsent::NoSummary => failed("the meeting has no summary to send", "meeting.no_summary"),
        Unsent::Unreadable => failed("the session's folder could not be read", "meeting.unreadable"),
    })?;
    let title = meeting.title.clone().unwrap_or(name);
    let outcome = match to {
        To::Notion => {
            let database = database()?;
            let children = notion::blocks(&meeting).ok_or_else(|| ended(Outcome::TooLarge))?;
            let key = key.unwrap_or_default();
            notion::send(&api(), &key, &database, &title, children, Policy::strict())
        }
        To::Channel(kind) => {
            let url = judged(&address).map_err(|refusal| failed(refusal.to_string(), "address.refused"))?;
            let folder_name = folder.file_name().and_then(|name| name.to_str()).unwrap_or("meeting");
            let body = body(kind, &meeting, &title, folder_name).ok_or_else(|| ended(Outcome::TooLarge))?;
            post(&url, &body, Policy::strict())
        }
    };
    match outcome {
        Outcome::Sent => Ok(json!({"event": "sent"})),
        outcome => Err(ended(outcome)),
    }
}

/// An outcome that ends the program: its sentence, which never holds the address or the key, and its line.
fn ended(outcome: Outcome) -> Refused {
    let why = match outcome {
        Outcome::Refused => "an address the host resolves to is this computer's or a local network's".into(),
        Outcome::Unreachable => "no connection to the destination".into(),
        Outcome::Silent => "sent, and no answer within 30 s: it may have arrived".into(),
        Outcome::Gone => "the destination no longer takes this address".into(),
        Outcome::Busy(_) => "the destination asks to wait before sending again".into(),
        Outcome::Declined(status) => format!("the destination refused it ({status})"),
        Outcome::Failed(status) => format!("the destination failed ({status})"),
        Outcome::KeyRefused => "Notion refused the integration's key".into(),
        Outcome::Unshared => "Notion knows no database at this link shared with the integration".into(),
        Outcome::KeyRestricted => "Notion does not let the integration insert content there".into(),
        Outcome::Several => "the database holds more than one data source".into(),
        Outcome::Unanswered => "Notion did not answer a read within 30 s: nothing was created".into(),
        Outcome::TooLarge => "the summary does not fit within the destination's bounds".into(),
        Outcome::Sent => unreachable!("a send that succeeded ends in success"),
    };
    let mut line = json!({"event": "failed", "code": outcome.code()});
    if let Outcome::Busy(Some(seconds)) = outcome {
        line["retry_after"] = seconds.into();
    }
    Refused(why, Some(line))
}

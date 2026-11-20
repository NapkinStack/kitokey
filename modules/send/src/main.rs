//! `kitokey-send`: a meeting's summary sent once to a channel, as `posting/v1` says.
//!
//! The arguments hold no word of the meeting and no address: the session's folder, named by its day and
//! hour, and the destination's kind. The address — a secret: whoever holds it posts to the channel — and
//! the meeting's name come on the input. No sentence, no line and no log holds the address.

use std::io::{self, Read};
use std::path::PathBuf;
use std::process::ExitCode;

use kitokey_send::{Kind, Meeting, Outcome, Policy, Unsent, body, judged, post};
use serde_json::{Value, json};

/// The version this program says (`--version`): the package's, named when it is built (`KITOKEY_VERSION`,
/// D78), Cargo's when unset or empty.
const VERSION: &str = match option_env!("KITOKEY_VERSION") {
    Some(version) if !version.is_empty() => version,
    _ => env!("CARGO_PKG_VERSION"),
};

const USAGE: &str = "usage: kitokey-send <session-folder> --kind <webhook|discord|slack>, or kitokey-send --check; \
     the address and the meeting's name on the input";

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

/// What the arguments ask: `--check`, or a folder and a kind.
enum Asked {
    Check,
    Send(PathBuf, Kind),
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
    if args == ["--check"] {
        return Ok(Asked::Check);
    }
    let (mut folder, mut kind) = (None, None);
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--kind" if kind.is_some() => return Err(wrong("--kind given twice")),
            "--kind" => {
                kind = Some(match args.next().as_deref() {
                    Some("webhook") => Kind::Webhook,
                    Some("discord") => Kind::Discord,
                    Some("slack") => Kind::Slack,
                    _ => return Err(wrong("--kind is webhook, discord or slack")),
                })
            }
            option if option.starts_with("--") => return Err(wrong(&format!("{option} is no option here"))),
            _ if folder.is_some() => return Err(wrong("one session's folder only")),
            _ => folder = Some(PathBuf::from(arg)),
        }
    }
    let folder = folder.ok_or_else(|| wrong("no session's folder"))?;
    let kind = kind.ok_or_else(|| wrong("no --kind"))?;
    Ok(Asked::Send(folder, kind))
}

/// The input: the address and the meeting's name, each on one line (`input.schema.json`).
fn input() -> Result<(String, String), Refused> {
    let wrong = |why: &str| Refused(format!("the input is not posting/v1's: {why}"), None);
    let mut text = String::new();
    io::stdin().read_to_string(&mut text).map_err(|_| wrong("it could not be read"))?;
    let object: Value = serde_json::from_str(&text).map_err(|_| wrong("it is not one JSON object"))?;
    let field = |name: &str| match object.get(name).and_then(Value::as_str) {
        Some(value) if !value.is_empty() && !value.contains(['\n', '\r']) => Ok(value.to_string()),
        _ => Err(wrong(&format!("« {name} » missing, empty or on more than one line"))),
    };
    Ok((field("address")?, field("name")?))
}

fn run() -> Result<Value, Refused> {
    let asked = asked()?;
    let (address, name) = input()?;
    let (folder, kind) = match asked {
        Asked::Check => {
            judged(&address).map_err(|refusal| failed(refusal.to_string(), "address.refused"))?;
            return Ok(json!({"event": "checked"}));
        }
        Asked::Send(folder, kind) => (folder, kind),
    };
    // The meeting before the address: a confidential meeting is never sent, whatever the address.
    let meeting = Meeting::read(&folder).map_err(|unsent| match unsent {
        Unsent::Confidential => failed("the meeting is said confidential: it is never sent", "meeting.confidential"),
        Unsent::NoSummary => failed("the meeting has no summary to send", "meeting.no_summary"),
        Unsent::Unreadable => failed("the session's folder could not be read", "meeting.unreadable"),
    })?;
    let url = judged(&address).map_err(|refusal| failed(refusal.to_string(), "address.refused"))?;
    let title = meeting.title.clone().unwrap_or(name);
    let folder_name = folder.file_name().and_then(|name| name.to_str()).unwrap_or("meeting");
    let body = body(kind, &meeting, &title, folder_name)
        .ok_or_else(|| failed("the summary does not fit within the destination's bounds", "summary.too_large"))?;
    let outcome = post(&url, &body, Policy::strict());
    let Some(code) = outcome.code() else {
        return Ok(json!({"event": "sent"}));
    };
    let why = match outcome {
        Outcome::Refused => "an address the host resolves to is this computer's or a local network's".into(),
        Outcome::Unreachable => "no connection to the destination".into(),
        Outcome::Silent => "sent, and no answer within 30 s: it may have arrived".into(),
        Outcome::Gone => "the destination no longer takes this address".into(),
        Outcome::Busy(_) => "the destination asks to wait before sending again".into(),
        Outcome::Declined(status) => format!("the destination refused it ({status})"),
        Outcome::Failed(status) => format!("the destination failed ({status})"),
        Outcome::Sent => unreachable!(),
    };
    let mut line = json!({"event": "failed", "code": code});
    if let Outcome::Busy(Some(seconds)) = outcome {
        line["retry_after"] = seconds.into();
    }
    Err(Refused(why, Some(line)))
}

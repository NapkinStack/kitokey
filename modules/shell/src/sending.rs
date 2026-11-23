//! A meeting's summary sent to a destination the user set up (D110, PDR-0007), on their gesture each time:
//! what kitokey-send said (`contracts/posting/v1`), and the record of each send — to which destination, when.
//!
//! The record is the shell's own data, one file beside its settings, each meeting by its folder's name: a
//! meeting renamed keeps its folder (`title.txt`, `session-audio`), so it keeps its sends; no file is added to
//! the session's folder, whose files `session-audio` lists. The record says no address, ever: a destination by
//! its name, as Settings show it.

use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::Path;

use serde::{Deserialize, Serialize};

/// The codes `posting/v1` lists: a code it does not list is read as none (« a later code is added here »).
pub const CODES: [&str; 11] = [
    "address.refused",
    "meeting.unreadable",
    "meeting.confidential",
    "meeting.no_summary",
    "summary.too_large",
    "network.unreachable",
    "destination.silent",
    "destination.gone",
    "destination.busy",
    "destination.refused",
    "destination.failed",
];

/// What kitokey-send said of one send.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "outcome", rename_all = "lowercase")]
pub enum Outcome {
    /// The destination answered `2xx`.
    Sent,
    /// It did not: the code a person acts on, when the program named one `posting/v1` lists, and the seconds
    /// to wait when the destination said them (`destination.busy`).
    Failed { code: Option<String>, retry_after: Option<u64> },
}

impl Outcome {
    /// A meeting said confidential, refused before kitokey-send is started, as it would refuse it.
    pub fn confidential() -> Self {
        Outcome::Failed {
            code: Some("meeting.confidential".to_string()),
            retry_after: None,
        }
    }
}

/// Reads kitokey-send's output: its last line is the one that says (`sent`, or `failed` with its code). No
/// line, a line that is not JSON, or an event `posting/v1` does not name: failed, no code.
pub fn read(output: &str) -> Outcome {
    let last = output.lines().rev().find(|line| !line.trim().is_empty());
    let Some(line) = last.and_then(|line| serde_json::from_str::<serde_json::Value>(line).ok()) else {
        return Outcome::Failed {
            code: None,
            retry_after: None,
        };
    };
    match line["event"].as_str() {
        Some("sent") => Outcome::Sent,
        Some("failed") => {
            let code = line["code"].as_str().filter(|code| CODES.contains(code)).map(str::to_string);
            let retry_after = line["retry_after"].as_u64().filter(|_| code.as_deref() == Some("destination.busy"));
            Outcome::Failed { code, retry_after }
        }
        _ => Outcome::Failed {
            code: None,
            retry_after: None,
        },
    }
}

/// One send: to which destination, by its name, and when, as RFC 3339.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Sent {
    pub destination: String,
    pub at: String,
}

/// Every meeting's sends, by its folder's name.
pub type Record = BTreeMap<String, Vec<Sent>>;

/// The record as the file holds it; none yet, or one that cannot be read, is empty: a record lost loses the
/// list and the first-send notice, never a meeting.
pub fn record(file: &Path) -> Record {
    fs::read_to_string(file)
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_default()
}

/// Adds a send to the record and writes it whole, beside then in place, as the shell writes its files.
pub fn keep(file: &Path, meeting: &str, sent: Sent) -> io::Result<()> {
    let mut all = record(file);
    all.entry(meeting.to_string()).or_default().push(sent);
    if let Some(parent) = file.parent() {
        fs::create_dir_all(parent)?;
    }
    let beside = file.with_extension("json.part");
    fs::write(&beside, serde_json::to_string_pretty(&all).map_err(io::Error::other)?)?;
    fs::rename(&beside, file)
}

/// A meeting's sends, in the order they were made.
pub fn of<'a>(record: &'a Record, meeting: &str) -> &'a [Sent] {
    record.get(meeting).map(Vec::as_slice).unwrap_or_default()
}

/// Whether a destination was ever sent to, from any meeting: before its first, the page says what leaves and
/// where (PDR-0007 §3).
pub fn ever(record: &Record, destination: &str) -> bool {
    record.values().flatten().any(|sent| sent.destination == destination)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sent_is_its_last_line() {
        assert_eq!(read("{\"event\":\"sent\"}\n"), Outcome::Sent);
    }

    #[test]
    fn each_code_posting_lists_is_kept_and_an_unknown_one_is_none() {
        for code in CODES {
            let line = format!("{{\"event\":\"failed\",\"code\":\"{code}\"}}");
            assert_eq!(
                read(&line),
                Outcome::Failed {
                    code: Some(code.into()),
                    retry_after: None
                },
                "{code}"
            );
        }
        assert_eq!(
            read("{\"event\":\"failed\",\"code\":\"later.code\"}"),
            Outcome::Failed {
                code: None,
                retry_after: None
            }
        );
    }

    #[test]
    fn busy_says_when_to_send_again_and_no_other_code_does() {
        let busy = read("{\"event\":\"failed\",\"code\":\"destination.busy\",\"retry_after\":30}");
        assert_eq!(
            busy,
            Outcome::Failed {
                code: Some("destination.busy".into()),
                retry_after: Some(30)
            }
        );
        let gone = read("{\"event\":\"failed\",\"code\":\"destination.gone\",\"retry_after\":30}");
        assert_eq!(
            gone,
            Outcome::Failed {
                code: Some("destination.gone".into()),
                retry_after: None
            }
        );
    }

    #[test]
    fn no_line_or_not_json_or_another_event_is_a_failure_without_a_code() {
        for said in ["", "\n\n", "not json", "{\"event\":\"checked\"}"] {
            assert_eq!(
                read(said),
                Outcome::Failed {
                    code: None,
                    retry_after: None
                },
                "{said:?}"
            );
        }
    }

    #[test]
    fn a_send_is_kept_per_meeting_and_a_destination_known_once_sent_to() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("state").join("sends.json");
        assert!(record(&file).is_empty());
        keep(
            &file,
            "2026-11-20_09-00",
            Sent {
                destination: "#équipe".into(),
                at: "2026-11-23T15:00:00+01:00".into(),
            },
        )
        .unwrap();
        keep(
            &file,
            "2026-11-20_09-00",
            Sent {
                destination: "#équipe".into(),
                at: "2026-11-23T15:05:00+01:00".into(),
            },
        )
        .unwrap();
        let all = record(&file);
        assert_eq!(of(&all, "2026-11-20_09-00").len(), 2);
        assert!(of(&all, "2026-11-21_09-00").is_empty());
        assert!(ever(&all, "#équipe"));
        assert!(!ever(&all, "#clients"));
        assert!(!file.with_extension("json.part").exists());
    }

    #[test]
    fn an_unreadable_record_is_empty_and_never_an_error() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("sends.json");
        fs::write(&file, "not json").unwrap();
        assert!(record(&file).is_empty());
    }
}

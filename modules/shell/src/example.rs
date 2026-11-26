//! The example meeting a newcomer tries (D126): a meeting of about 45 seconds, two invented voices
//! (`ui/example/LICENSE`), bundled with Kitokey for each language the window speaks. Tried, it is
//! placed in the meetings' folder as `session-audio/v3` says a meeting is, dated when it is tried,
//! its title saying it is an example; the user renames or deletes it as any other.
//! `scripts/example.py` makes its files.

use std::{
    fs, io,
    path::{Path, PathBuf},
};

use crate::words::Language;

/// One language's files, by their names in the meeting's folder.
type Files = &'static [(&'static str, &'static [u8])];

macro_rules! bundled {
    ($language:literal) => {
        &[
            (
                "session.json",
                include_bytes!(concat!("../ui/example/", $language, "/session.json")),
            ),
            ("clock.jsonl", include_bytes!(concat!("../ui/example/", $language, "/clock.jsonl"))),
            ("local.flac", include_bytes!(concat!("../ui/example/", $language, "/local.flac"))),
            ("remote.flac", include_bytes!(concat!("../ui/example/", $language, "/remote.flac"))),
            ("title.txt", include_bytes!(concat!("../ui/example/", $language, "/title.txt"))),
        ]
    };
}

/// Each example, by the code of the language it is spoken in.
pub const BUNDLED: [(&str, Files); 2] = [("fr", bundled!("fr")), ("en", bundled!("en"))];

/// Places the example of the window's `language` in the meetings' folder, as a meeting started
/// `now` (an instant with its offset, as `session.json` writes one), and gives its folder. Tried
/// again, it is another meeting.
pub fn place(meetings: &Path, language: Language, now: &str) -> io::Result<PathBuf> {
    let code = match language {
        Language::French => "fr",
        Language::English => "en",
    };
    let (_, files) = BUNDLED
        .iter()
        .find(|(spoken, _)| *spoken == code)
        .expect("an example for each language");
    fs::create_dir_all(meetings)?;
    let folder = (1..)
        .map(|n| {
            meetings.join(if n == 1 {
                format!("example-{code}")
            } else {
                format!("example-{code}-{n}")
            })
        })
        .find(|folder| !folder.exists())
        .expect("a free name");
    // `session.json` last, written beside and renamed: a folder without it is no meeting, so the list
    // never reads one half made — its title not there yet.
    fs::create_dir(&folder)?;
    let mut session = None;
    for (name, bytes) in *files {
        if *name == "session.json" {
            session = Some(dated(bytes, now)?);
        } else {
            fs::write(folder.join(name), bytes)?;
        }
    }
    fs::write(folder.join("language"), format!("{code}\n"))?;
    let beside = folder.join(".session.json.part");
    fs::write(&beside, session.expect("a session.json in each example"))?;
    fs::rename(&beside, folder.join("session.json"))?;
    Ok(folder)
}

/// The example's `session.json`, started `now` and stopped as long after as it lasts.
fn dated(bytes: &[u8], now: &str) -> io::Result<Vec<u8>> {
    let invalid = |why: String| io::Error::new(io::ErrorKind::InvalidData, why);
    let mut session: serde_json::Value = serde_json::from_slice(bytes).map_err(|error| invalid(error.to_string()))?;
    let instant = |field: &str| {
        session[field]
            .as_str()
            .and_then(|text| chrono::DateTime::parse_from_rfc3339(text).ok())
            .ok_or_else(|| invalid(format!("the example's {field} is no instant")))
    };
    let lasts = instant("stopped")? - instant("started")?;
    let started = chrono::DateTime::parse_from_rfc3339(now).map_err(|error| invalid(error.to_string()))?;
    session["started"] = started.to_rfc3339_opts(chrono::SecondsFormat::Secs, false).into();
    session["stopped"] = (started + lasts).to_rfc3339_opts(chrono::SecondsFormat::Secs, false).into();
    serde_json::to_vec_pretty(&session).map_err(|error| invalid(error.to_string()))
}

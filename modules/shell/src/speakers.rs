//! The names a person gives a meeting's speakers (#1205): `speakers.json` of `contracts/session-audio/v3`, written by
//! the shell on the user's word and read by it and by `summary`. Without a producer that tells voices apart, the shell
//! names the two sides alone — `me`, the local track, where it says « Vous », and `others`, the remote one, where it
//! says « Les autres » —; `voices` it never writes, and keeps as it found them.

use std::{fs, io, path::Path};

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// The file's name, in the meeting's folder.
pub const FILE: &str = "speakers.json";

/// A name's bound, in characters (the contract's schema).
pub const LONGEST: usize = 60;

/// The two sides' names; none, the side keeps the shell's word for it.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Speakers {
    pub me: Option<String>,
    pub others: Option<String>,
}

impl Speakers {
    /// Each side by its name, else by the words given for it — what `kitokey-export` is handed as
    /// `exporting/v1`'s `me` and `others`.
    pub fn sides<'a>(&'a self, me: &'a str, others: &'a str) -> (&'a str, &'a str) {
        (self.me.as_deref().unwrap_or(me), self.others.as_deref().unwrap_or(others))
    }
}

/// A name as the contract allows it: one line, 1 to 60 characters, not blank.
fn allowed(name: &str) -> bool {
    !name.trim().is_empty() && name.chars().count() <= LONGEST && !name.contains(['\n', '\r'])
}

fn name_in(object: &Map<String, Value>, key: &str) -> Option<String> {
    object
        .get(key)
        .and_then(Value::as_str)
        .filter(|name| allowed(name))
        .map(str::to_string)
}

fn object_of(folder: &Path) -> Option<Map<String, Value>> {
    fs::read(folder.join(FILE))
        .ok()
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())
}

/// The names the meeting holds: each the contract allows, else absent; a file that is no JSON object, none.
pub fn read(folder: &Path) -> Speakers {
    let Some(object) = object_of(folder) else {
        return Speakers::default();
    };
    Speakers {
        me: name_in(&object, "me"),
        others: name_in(&object, "others"),
    }
}

/// Whether the meeting can hold names: a session of `session-audio/v3`, the version that names the file.
pub fn offered(folder: &Path) -> bool {
    fs::read(folder.join("session.json"))
        .ok()
        .and_then(|bytes| serde_json::from_slice::<Value>(&bytes).ok())
        .is_some_and(|session| session["contract"] == "session-audio/v3")
}

/// The two names written, each trimmed — blank, the side's name removed —, whatever else the file holds kept; the
/// names as now kept are returned. Refused before a v3 session (`Unsupported`) and for a name the contract refuses —
/// a line break, over 60 characters (`InvalidInput`): nothing is written then. No name nor voice left: no file.
pub fn write(folder: &Path, wanted: &Speakers) -> io::Result<Speakers> {
    if !offered(folder) {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "names are kept by a session-audio/v3 meeting alone",
        ));
    }
    let trimmed = |name: &Option<String>| name.as_deref().map(str::trim).filter(|name| !name.is_empty()).map(str::to_string);
    let kept = Speakers {
        me: trimmed(&wanted.me),
        others: trimmed(&wanted.others),
    };
    if [&kept.me, &kept.others].into_iter().flatten().any(|name| !allowed(name)) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "a name the contract session-audio/v3 refuses",
        ));
    }
    let mut object = object_of(folder).unwrap_or_default();
    for (key, name) in [("me", &kept.me), ("others", &kept.others)] {
        match name {
            Some(name) => object.insert(key.to_string(), Value::String(name.clone())),
            None => object.remove(key),
        };
    }
    let target = folder.join(FILE);
    if object.is_empty() {
        return match fs::remove_file(&target) {
            Err(error) if error.kind() != io::ErrorKind::NotFound => Err(error),
            _ => Ok(kept),
        };
    }
    let beside = folder.join(".speakers.json.part");
    let text = serde_json::to_vec(&Value::Object(object)).map_err(io::Error::other)?;
    let written = fs::write(&beside, text).and_then(|()| fs::rename(&beside, &target));
    if written.is_err() {
        let _ = fs::remove_file(&beside);
    }
    written.map(|()| kept)
}

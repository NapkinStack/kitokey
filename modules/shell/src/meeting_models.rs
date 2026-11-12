//! A meeting's own models (D91): `models.json` of `contracts/session-audio/v3` (and v1, v2), which
//! the shell writes on the user's word and alone reads. What it names stands in for Settings'
//! choice when the meeting is made and made again; Settings are never changed by it. A Whisper
//! model is found by its name among those this computer lists, never as a path; a model gone, or a
//! service no longer set, gives way to Settings' choice, and that is said — as a microphone chosen
//! and not there gives way to Auto (D74). `confidential` comes first: it is read after this.

use std::{
    fs, io,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::settings::{Engine, Settings};

/// The file's name, in the meeting's folder.
pub const FILE: &str = "models.json";

/// Who hears the meeting, as the file names it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "engine", rename_all = "lowercase")]
pub enum Hearing {
    /// A Whisper model of this computer, by its file's name alone.
    Whisper {
        model: String,
    },
    SpeechAnalyzer,
    /// A service, at its address, with its model's name.
    Service {
        provider: String,
        model: String,
    },
}

/// Who writes the minutes: a provider's address and its model's name.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Writer {
    pub provider: String,
    pub model: String,
}

/// What the meeting chose for itself; a part none is Settings' choice.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Models {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transcription: Option<Hearing>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minutes: Option<Writer>,
}

/// A step whose choice could not be used.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Step {
    /// A Whisper model no longer listed, or a service no longer the one Settings set.
    Transcription,
    /// A provider neither the one Settings set nor on this computer.
    Minutes,
}

/// Settings as this meeting is made with them, and the steps whose own choice is gone.
#[derive(Debug, Clone, PartialEq)]
pub struct Applied {
    pub settings: Settings,
    pub gone: Vec<Step>,
}

/// Text the contract allows: one line, never empty.
fn name(text: &str) -> bool {
    !text.is_empty() && !text.contains(['\n', '\r'])
}

/// A Whisper model's file name as the contract allows it: never a path, nor `.` or `..`.
fn file(text: &str) -> bool {
    name(text) && !text.contains(['/', '\\']) && text != "." && text != ".."
}

fn hearing(value: &Value) -> Option<Hearing> {
    let hearing: Hearing = serde_json::from_value(value.clone()).ok()?;
    let one = Models {
        transcription: Some(hearing),
        minutes: None,
    };
    allowed(&one).then_some(one.transcription).flatten()
}

fn writer(value: &Value) -> Option<Writer> {
    let writer: Writer = serde_json::from_value(value.clone()).ok()?;
    let one = Models {
        transcription: None,
        minutes: Some(writer),
    };
    allowed(&one).then_some(one.minutes).flatten()
}

/// What the meeting chose: each part as the contract allows it, or none — a part refused reads as
/// absent, and a file that is no JSON object as no file.
pub fn read(folder: &Path) -> Models {
    read_file(&folder.join(FILE))
}

fn read_file(file: &Path) -> Models {
    let Some(object) = fs::read(file)
        .ok()
        .and_then(|bytes| serde_json::from_slice::<Map<String, Value>>(&bytes).ok())
    else {
        return Models::default();
    };
    Models {
        transcription: object.get("transcription").and_then(hearing),
        minutes: object.get("minutes").and_then(writer),
    }
}

/// Whether every part is one the contract allows: what is written is what `read` reads back.
pub fn allowed(models: &Models) -> bool {
    let hearing = models.transcription.as_ref().is_none_or(|chosen| match chosen {
        Hearing::Whisper { model } => file(model),
        Hearing::SpeechAnalyzer => true,
        Hearing::Service { provider, model } => name(provider) && name(model),
    });
    hearing
        && models
            .minutes
            .as_ref()
            .is_none_or(|chosen| name(&chosen.provider) && name(&chosen.model))
}

/// The choice written whole — a file beside, renamed over it —; none left: removed. Refused before
/// the folder holds `session.json` (`NotFound`), and for a part the contract refuses — a path for a
/// Whisper model, an empty name, a line break (`InvalidInput`): nothing is written then.
pub fn write(folder: &Path, models: &Models) -> io::Result<()> {
    if !folder.join("session.json").is_file() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "the meeting's folder holds no session.json yet",
        ));
    }
    if !allowed(models) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "a part the contract session-audio/v3 refuses",
        ));
    }
    whole(&folder.join(FILE), &folder.join(".models.json.part"), models)
}

/// `models` written whole at `target` — `beside` first, renamed over it —; none left: removed.
fn whole(target: &Path, beside: &Path, models: &Models) -> io::Result<()> {
    if *models == Models::default() {
        return match fs::remove_file(target) {
            Err(error) if error.kind() != io::ErrorKind::NotFound => Err(error),
            _ => Ok(()),
        };
    }
    let text = serde_json::to_vec(models).map_err(io::Error::other)?;
    let written = fs::write(beside, text).and_then(|()| fs::rename(beside, target));
    if written.is_err() {
        let _ = fs::remove_file(beside);
    }
    written
}

/// The models chosen in the bar at rest for the next recording (D91), as read again when Kitokey
/// starts: each part as the contract allows it, or none — as `read` reads a meeting's.
pub fn read_next(file: &Path) -> Models {
    read_file(file)
}

/// Keeps the models chosen at rest for the next recording — Kitokey started again too, as the
/// context typed before the start is —: written whole, or removed when none is left. Refused for a
/// part the contract refuses (`InvalidInput`): what was kept stays.
pub fn keep_next(file: &Path, models: &Models) -> io::Result<()> {
    if !allowed(models) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "a part the contract session-audio/v3 refuses",
        ));
    }
    if *models != Models::default()
        && let Some(folder) = file.parent()
    {
        fs::create_dir_all(folder)?;
    }
    whole(file, &file.with_extension("part"), models)
}

/// The models kept for the next recording, taken by the meeting begun: written as its own under
/// `write`'s rules, none kept after; none chosen writes nothing. Refused — the folder holds no
/// `session.json` yet —, they stay kept for the next.
pub fn taken(folder: &Path, next: &mut Models) -> io::Result<()> {
    let models = std::mem::take(next);
    if models == Models::default() {
        return Ok(());
    }
    write(folder, &models).inspect_err(|_| *next = models)
}

/// Settings as the meeting is made with them: what it chose in place of Settings' choice, a
/// Whisper model found by its name among `listed` — `settings::whisper_models` —, a service to hear
/// only the one Settings set, a provider to write only the one Settings set or one on this computer,
/// which sends nothing away. What cannot be used is Settings' choice, and named in `gone`: the
/// agreement to send a meeting is never asked for a host Settings do not name.
pub fn applied(settings: &Settings, models: &Models, listed: &[PathBuf]) -> Applied {
    let mut made = settings.clone();
    let mut gone = Vec::new();
    match &models.transcription {
        None => {}
        Some(Hearing::Whisper { model }) => {
            match listed
                .iter()
                .find(|path| path.file_name().is_some_and(|listed| listed == model.as_str()))
            {
                Some(path) => {
                    made.engine = Engine::Whisper;
                    made.model = path.clone();
                }
                None => gone.push(Step::Transcription),
            }
        }
        Some(Hearing::SpeechAnalyzer) => made.engine = Engine::SpeechAnalyzer,
        Some(Hearing::Service { provider, model }) => {
            if settings.transcription_provider.as_deref() == Some(provider.as_str()) {
                made.engine = Engine::Service;
                made.transcription_model = Some(model.clone());
            } else {
                gone.push(Step::Transcription);
            }
        }
    }
    if let Some(Writer { provider, model }) = &models.minutes {
        if settings.minutes_provider.as_deref() == Some(provider.as_str()) || crate::providers::local(provider) {
            made.minutes_provider = Some(provider.clone());
            made.minutes_model = Some(model.clone());
        } else {
            gone.push(Step::Minutes);
        }
    }
    Applied { settings: made, gone }
}

//! How fast this machine makes each step, by model (D92): the seconds a making took for each second
//! of audio, as last measured here. From it and a meeting's length, the page says « environ N min »
//! left; with none measured, the step alone and no number — never a number made up.
//!
//! Kept in the shell's own state, beside its settings: numbers by step and model, nothing of a
//! meeting — no name, no date, no length. A file that does not read is none.

use std::collections::BTreeMap;
use std::io;
use std::path::Path;

/// A step whose speed is kept.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    Transcript,
    Minutes,
}

/// The seconds of making per second of audio, by step and model, as last measured.
#[derive(Debug, Clone, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Speeds {
    #[serde(default)]
    transcript: BTreeMap<String, f64>,
    #[serde(default)]
    minutes: BTreeMap<String, f64>,
}

impl Speeds {
    fn of(&self, step: Step) -> &BTreeMap<String, f64> {
        match step {
            Step::Transcript => &self.transcript,
            Step::Minutes => &self.minutes,
        }
    }

    /// A making ended, written: its speed is the one kept for its step and model. A measure that
    /// says nothing — no audio, a time not a number or below zero — changes nothing.
    pub fn measured(&mut self, step: Step, model: &str, made_in: f64, audio: f64) {
        if !(made_in.is_finite() && made_in >= 0.0 && audio.is_finite() && audio > 0.0) {
            return;
        }
        let speeds = match step {
            Step::Transcript => &mut self.transcript,
            Step::Minutes => &mut self.minutes,
        };
        speeds.insert(model.to_string(), made_in / audio);
    }

    /// The seconds left of a making: each step still to end, at its model's speed for this audio,
    /// less what it has run (`Some`) — never below nothing. None when one of them has no speed
    /// kept: no number for the making, never half of one.
    pub fn left(&self, steps: &[(Step, &str, Option<f64>)], audio: f64) -> Option<f64> {
        steps.iter().try_fold(0.0, |left, (step, model, ran)| {
            let whole = self.of(*step).get(*model)? * audio;
            Some(left + (whole - ran.unwrap_or(0.0)).max(0.0))
        })
    }
}

/// The minutes said for seconds left (GNOME HIG: a bar says what is left past about thirty
/// seconds): none up to thirty seconds; to the minute up to ten, at least one; to five after.
pub fn said(seconds: f64) -> Option<u64> {
    // Not a number, or thirty seconds or less: none.
    if seconds.is_nan() || seconds <= 30.0 {
        return None;
    }
    let minutes = (seconds / 60.0).round().max(1.0);
    let minutes = if minutes > 10.0 { (minutes / 5.0).round() * 5.0 } else { minutes };
    // A count of minutes, rounded and at least one: it fits.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    Some(minutes as u64)
}

/// The speeds kept in `file`; none there, or one that does not read, is none measured.
pub fn read(file: &Path) -> Speeds {
    std::fs::read_to_string(file)
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_default()
}

/// Keeps the speeds in `file`, whole or not at all: written beside it, then renamed over it.
pub fn write(file: &Path, speeds: &Speeds) -> io::Result<()> {
    let text = serde_json::to_string(speeds).map_err(io::Error::other)?;
    let beside = file.with_extension("json.new");
    std::fs::write(&beside, text)?;
    std::fs::rename(&beside, file)
}

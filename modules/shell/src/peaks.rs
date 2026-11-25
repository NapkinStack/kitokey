//! A meeting's waveform per voice (#1201, #1179 B): each track's loudest sample in each of `BINS`
//! stretches of the meeting, each track placed where the session's clock puts it (`meeting::tracks`,
//! read as the player reads it, `mix`). Read once, then kept in the session's `derived/`, which
//! `session-audio/v2` and `v3` leave to their consumers (« Whatever they make from the tracks. Not
//! described here: nothing in it is a source »), and read from there while the tracks and the clock
//! stay as they were. Version 1 names no `derived/`: its peaks are read each time and kept nowhere.
//! Numbers only: nothing a person said can be heard back from them.

use std::{
    fs,
    path::{Path, PathBuf},
    time::UNIX_EPOCH,
};

use serde::{Deserialize, Serialize};

use crate::{meeting, mix::Mix};

/// How many stretches a meeting is read in: more than the widest drawing has bars, which it groups.
pub const BINS: usize = 600;

/// Where the peaks are kept, in the session's folder.
pub const KEPT: &str = "derived/peaks.json";

/// Each voice's peaks over the meeting, 0 to 255 of full scale; a voice whose track does not read,
/// none. `seconds` is the meeting's span the bins divide.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Peaks {
    pub seconds: f64,
    pub local: Option<Vec<u8>>,
    pub remote: Option<Vec<u8>>,
}

/// What is kept: the peaks, and what they were read from — each file's name, size and time.
#[derive(Serialize, Deserialize)]
struct Kept {
    sources: Vec<(String, u64, u128)>,
    #[serde(flatten)]
    peaks: Peaks,
}

/// The meeting's peaks, from `derived/` when they were kept from these very tracks, read from the
/// tracks otherwise — and then kept, for a session of version 2 or 3. None when no track reads.
pub fn of(folder: &Path) -> Option<Peaks> {
    let tracks = meeting::tracks(folder);
    let sources = sources(folder, &tracks);
    let kept = folder.join(KEPT);
    let keeps = derived_allowed(folder);
    if keeps
        && let Some(found) = fs::read_to_string(&kept)
            .ok()
            .and_then(|text| serde_json::from_str::<Kept>(&text).ok())
            .filter(|found| found.sources == sources)
    {
        return Some(found.peaks);
    }
    let peaks = read(&tracks)?;
    if keeps {
        // Kept whole or not at all: written beside, then put in place.
        let text = serde_json::to_string(&Kept {
            sources,
            peaks: peaks.clone(),
        })
        .ok()?;
        let part = kept.with_extension("json.part");
        let written = fs::create_dir_all(folder.join("derived"))
            .and_then(|()| fs::write(&part, text))
            .and_then(|()| fs::rename(&part, &kept));
        if let Err(error) = written {
            tracing::warn!(detail = %error, "shell.peaks_not_kept");
            let _ = fs::remove_file(&part);
        }
    }
    Some(peaks)
}

/// Whether the session's version names a `derived/` for its consumers: 2 and 3.
fn derived_allowed(folder: &Path) -> bool {
    let contract = fs::read_to_string(folder.join("session.json"))
        .ok()
        .and_then(|text| serde_json::from_str::<serde_json::Value>(&text).ok())
        .and_then(|session| session["contract"].as_str().map(str::to_string));
    matches!(contract.as_deref(), Some("session-audio/v2" | "session-audio/v3"))
}

/// Each track's file and the clock, as they are now: a change to any reads the peaks again.
fn sources(folder: &Path, tracks: &[meeting::Track]) -> Vec<(String, u64, u128)> {
    let mut files: Vec<PathBuf> = tracks.iter().map(|track| track.path.clone()).collect();
    files.push(folder.join("clock.jsonl"));
    files
        .iter()
        .filter_map(|path| {
            let meta = fs::metadata(path).ok()?;
            let time = meta.modified().ok()?.duration_since(UNIX_EPOCH).ok()?.as_nanos();
            Some((path.file_name()?.to_string_lossy().into_owned(), meta.len(), time))
        })
        .collect()
}

/// The peaks read from the tracks, each alone as the player places it.
fn read(tracks: &[meeting::Track]) -> Option<Peaks> {
    let mixes: Vec<(&str, Mix)> = tracks
        .iter()
        .filter_map(|track| Some((track.name.as_str(), Mix::placed(&[(track.path.clone(), track.runs.clone())])?)))
        .filter(|(_, mix)| mix.frames() > 0)
        .collect();
    let seconds = mixes
        .iter()
        .map(|(_, mix)| mix.frames() as f64 / f64::from(mix.rate()))
        .fold(0.0, f64::max);
    if mixes.is_empty() || seconds <= 0.0 {
        return None;
    }
    let mut peaks = Peaks {
        seconds,
        local: None,
        remote: None,
    };
    for (name, mix) in &mixes {
        let Ok(bins) = binned(mix, seconds) else {
            continue;
        };
        match *name {
            "local" => peaks.local = Some(bins),
            "remote" => peaks.remote = Some(bins),
            _ => {}
        }
    }
    (peaks.local.is_some() || peaks.remote.is_some()).then_some(peaks)
}

/// The mix's loudest sample in each of `BINS` stretches of `seconds`, read ten seconds at a time.
fn binned(mix: &Mix, seconds: f64) -> std::io::Result<Vec<u8>> {
    const HEADER: u64 = 44;
    let rate = u64::from(mix.rate());
    let per_bin = seconds * mix.rate() as f64 / BINS as f64;
    let mut bins = vec![0u16; BINS];
    let piece = 10 * rate;
    let mut frame = 0u64;
    while frame < mix.frames() {
        let count = piece.min(mix.frames() - frame);
        let bytes = mix.read(HEADER + 2 * frame, 2 * count)?;
        for (index, pair) in bytes.as_chunks::<2>().0.iter().enumerate() {
            let at = frame + index as u64;
            let bin = ((at as f64 / per_bin) as usize).min(BINS - 1);
            let level = i16::from_le_bytes(*pair).unsigned_abs();
            if level > bins[bin] {
                bins[bin] = level;
            }
        }
        frame += count;
    }
    Ok(bins
        .into_iter()
        .map(|level| ((f64::from(level) / 32767.0) * 255.0).round().min(255.0) as u8)
        .collect())
}

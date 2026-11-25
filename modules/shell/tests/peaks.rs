//! A meeting's waveform per voice (#1201, #1179 B): each track's peaks over the meeting, read once
//! and kept in the session's `derived/` (`session-audio/v2` and `v3`: « Consumers; optional |
//! Whatever they make from the tracks »), placed where the session's clock puts each track. A v1
//! session has no `derived/`: its peaks are read and kept nowhere.

use std::{fs, path::Path, time::Instant};

use kitokey_shell::peaks::{self, BINS};

mod kept;
use kept::kept_as_flac;

/// A WAV of one channel of 16 bits at `rate`, `seconds` long, each frame from its time in seconds.
fn wav(path: &Path, rate: u32, seconds: f64, value: impl Fn(f64) -> f32) {
    let frames = (f64::from(rate) * seconds) as usize;
    let mut data = Vec::with_capacity(frames * 2);
    for frame in 0..frames {
        let v = value(frame as f64 / f64::from(rate)).clamp(-1.0, 1.0);
        data.extend_from_slice(&((v * 32767.0) as i16).to_le_bytes());
    }
    let mut file = b"RIFF".to_vec();
    file.extend_from_slice(&((36 + data.len()) as u32).to_le_bytes());
    file.extend_from_slice(b"WAVEfmt ");
    file.extend_from_slice(&16u32.to_le_bytes());
    file.extend_from_slice(&1u16.to_le_bytes());
    file.extend_from_slice(&1u16.to_le_bytes());
    file.extend_from_slice(&rate.to_le_bytes());
    file.extend_from_slice(&(rate * 2).to_le_bytes());
    file.extend_from_slice(&2u16.to_le_bytes());
    file.extend_from_slice(&16u16.to_le_bytes());
    file.extend_from_slice(b"data");
    file.extend_from_slice(&(data.len() as u32).to_le_bytes());
    file.extend_from_slice(&data);
    fs::write(path, file).unwrap();
}

/// A square wave of `level` while `on` says so, silence otherwise: its peak is `level` exactly.
fn square(level: f32, on: impl Fn(f64) -> bool) -> impl Fn(f64) -> f32 {
    move |t| {
        if !on(t) {
            0.0
        } else if (t * 200.0) as i64 % 2 == 0 {
            level
        } else {
            -level
        }
    }
}

fn session(folder: &Path, form: &str) {
    let tracks = if form == "v1" { "sources" } else { "tracks" };
    fs::write(
        folder.join("session.json"),
        format!(
            r#"{{"contract":"session-audio/{form}","started":"2026-11-20T09:00:00+01:00","stopped":"2026-11-20T09:00:10+01:00","{tracks}":{{"local":{{"device":"Mic"}},"remote":{{"device":"Speaker"}}}}}}"#
        ),
    )
    .unwrap();
}

/// A v3 meeting of 10 s: you speak the first half at 0.5, the meeting the second at 0.25.
fn halves(folder: &Path) {
    session(folder, "v3");
    wav(&folder.join("local.wav"), 16_000, 10.0, square(0.5, |t| t < 5.0));
    wav(&folder.join("remote.wav"), 16_000, 10.0, square(0.25, |t| t >= 5.0));
    fs::write(
        folder.join("clock.jsonl"),
        "{\"track\":\"local\",\"sample\":0,\"at\":0.0}\n{\"track\":\"remote\",\"sample\":0,\"at\":0.0}\n",
    )
    .unwrap();
}

#[test]
fn each_voice_is_its_track_s_peaks_over_the_meeting() {
    let folder = tempfile::tempdir().unwrap();
    halves(folder.path());
    let drawn = peaks::of(folder.path()).expect("two readable tracks give peaks");
    let (local, remote) = (drawn.local.unwrap(), drawn.remote.unwrap());
    assert_eq!((local.len(), remote.len()), (BINS, BINS));
    assert!((drawn.seconds - 10.0).abs() < 0.01, "{}", drawn.seconds);
    // A peak of 0.5 is 128 of 255, 0.25 is 64; silence is 0. The bin holding 5.0 s is either's.
    let (first, second) = (..BINS / 2 - 1, BINS / 2 + 1..);
    assert!(local[first].iter().all(|v| (127..=128).contains(v)), "{:?}", &local[..8]);
    assert!(local[second.clone()].iter().all(|v| *v == 0), "{:?}", &local[BINS - 8..]);
    assert!(remote[first].iter().all(|v| *v == 0));
    assert!(remote[second].iter().all(|v| (63..=64).contains(v)), "{:?}", &remote[BINS - 8..]);
}

#[test]
fn a_track_is_placed_where_the_session_s_clock_puts_it() {
    let folder = tempfile::tempdir().unwrap();
    halves(folder.path());
    // The meeting's track opened two seconds late: its frame 0 at 2.0 s, its sound from 7 s to 12 s.
    fs::write(
        folder.path().join("clock.jsonl"),
        "{\"track\":\"local\",\"sample\":0,\"at\":0.0}\n{\"track\":\"remote\",\"sample\":0,\"at\":2.0}\n",
    )
    .unwrap();
    let drawn = peaks::of(folder.path()).unwrap();
    assert!((drawn.seconds - 12.0).abs() < 0.01, "{}", drawn.seconds);
    let remote = drawn.remote.unwrap();
    let at = |seconds: f64| remote[((seconds / 12.0) * BINS as f64) as usize];
    assert_eq!((at(6.5), at(8.0), at(11.5)), (0, 64, 64));
}

#[test]
fn read_once_they_are_kept_in_derived_and_read_from_there_again() {
    let folder = tempfile::tempdir().unwrap();
    halves(folder.path());
    let first = peaks::of(folder.path()).unwrap();
    let kept = folder.path().join(peaks::KEPT);
    assert!(kept.starts_with(folder.path().join("derived")), "{}", kept.display());
    assert!(kept.is_file(), "the peaks are kept in derived/");
    // What is kept is what is read next: a value changed there comes back, the tracks not read again.
    let mut doctored: serde_json::Value = serde_json::from_str(&fs::read_to_string(&kept).unwrap()).unwrap();
    assert!(
        doctored["local"].is_array(),
        "the local peaks are kept as a list of numbers: {doctored}"
    );
    doctored["local"][0] = 7.into();
    fs::write(&kept, doctored.to_string()).unwrap();
    let again = peaks::of(folder.path()).unwrap();
    assert_eq!(again.local.as_ref().unwrap()[0], 7);
    assert_eq!(again.remote, first.remote);
}

#[test]
fn a_track_that_changed_is_read_again() {
    let folder = tempfile::tempdir().unwrap();
    halves(folder.path());
    peaks::of(folder.path()).unwrap();
    // Louder, and longer: what was kept no longer describes the track.
    wav(&folder.path().join("local.wav"), 16_000, 10.5, square(0.75, |t| t < 5.0));
    let again = peaks::of(folder.path()).unwrap();
    assert!((190..=192).contains(&again.local.unwrap()[0]));
}

#[test]
fn a_track_kept_as_flac_gives_the_same_peaks() {
    let (wavs, flacs) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    halves(wavs.path());
    halves(flacs.path());
    kept_as_flac(&flacs.path().join("local.wav"));
    kept_as_flac(&flacs.path().join("remote.wav"));
    assert!(!flacs.path().join("local.wav").exists());
    assert_eq!(peaks::of(flacs.path()), peaks::of(wavs.path()));
}

#[test]
fn a_v1_session_is_read_and_kept_nowhere() {
    let folder = tempfile::tempdir().unwrap();
    session(folder.path(), "v1");
    wav(&folder.path().join("me.wav"), 16_000, 4.0, square(0.5, |_| true));
    wav(&folder.path().join("others.wav"), 16_000, 4.0, square(0.25, |_| true));
    let drawn = peaks::of(folder.path()).unwrap();
    assert!(drawn.local.unwrap().iter().all(|v| (127..=128).contains(v)));
    assert!(!folder.path().join("derived").exists(), "version 1 names no derived/");
}

#[test]
fn no_track_to_read_gives_no_peaks_and_keeps_nothing() {
    let folder = tempfile::tempdir().unwrap();
    session(folder.path(), "v3");
    assert_eq!(peaks::of(folder.path()), None);
    // A track there but not a WAV: none either, never a waveform of noise.
    fs::write(folder.path().join("local.wav"), b"RIFF not a wave").unwrap();
    assert_eq!(peaks::of(folder.path()), None);
    assert!(!folder.path().join("derived").exists());
}

#[test]
fn one_track_alone_is_drawn_the_other_said_absent() {
    let folder = tempfile::tempdir().unwrap();
    halves(folder.path());
    fs::remove_file(folder.path().join("remote.wav")).unwrap();
    let drawn = peaks::of(folder.path()).unwrap();
    assert!(drawn.local.is_some() && drawn.remote.is_none());
}

/// A meeting of two hours, both tracks at 48 kHz as capture writes them: read once, timed. Run by
/// hand (`cargo test --test peaks -- --ignored --nocapture`): 1.4 GB of WAV are written for it.
#[test]
#[ignore]
fn two_hours_are_read_in_seconds() {
    let folder = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).unwrap();
    session(folder.path(), "v3");
    let hours = 2.0 * 3600.0;
    wav(
        &folder.path().join("local.wav"),
        48_000,
        hours,
        square(0.5, |t| (t / 60.0) as i64 % 2 == 0),
    );
    wav(
        &folder.path().join("remote.wav"),
        48_000,
        hours,
        square(0.25, |t| (t / 60.0) as i64 % 2 == 1),
    );
    let started = Instant::now();
    let drawn = peaks::of(folder.path()).unwrap();
    let read = started.elapsed();
    let started = Instant::now();
    peaks::of(folder.path()).unwrap();
    let kept = started.elapsed();
    println!("two hours, two tracks at 48 kHz: read in {read:?}, from derived/ in {kept:?}");
    assert!((drawn.seconds - hours).abs() < 1.0);
    assert!(kept.as_millis() < 100, "{kept:?}");
}

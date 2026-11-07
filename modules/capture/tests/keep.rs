//! A stopped track kept as FLAC (`contracts/session-audio/v3`, *Once stopped, a track kept as FLAC*):
//! written as `.flac.part`, read back and compared, renamed, and only then the WAV removed — on
//! sound made here, through the steps `keep::keep` says it is at.

use std::{fs, path::Path};

use kitokey_capture::{
    keep::{self, Kept, Step},
    v2,
};
use symphonia::{
    core::{
        codecs::audio::{AudioDecoder, AudioDecoderOptions},
        formats::{FormatOptions, FormatReader, TrackType},
        io::MediaSourceStream,
    },
    default::{codecs::FlacDecoder, formats::FlacReader},
};

/// A track as capture records it: a tone over noise, both full scales reached, `seconds` long.
fn recorded(folder: &Path, name: &str, rate: u32, channels: u16, seconds: f32) -> Vec<i32> {
    let mut track = v2::Track::create(&folder.join(format!("{name}.wav")), rate, channels).unwrap();
    let mut seed = 7u32;
    let frames = (seconds * rate as f32) as usize;
    let sound: Vec<f32> = (0..frames * usize::from(channels))
        .map(|at| {
            seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            let noise = (seed >> 8) as f32 / 16_777_216.0 - 0.5;
            let tone = (at as f32 / f32::from(channels) * 440.0 / rate as f32 * std::f32::consts::TAU).sin();
            match at % 9_973 {
                0 => 1.0,
                1 => -1.0,
                _ => 0.3 * tone + 0.01 * noise + 0.02,
            }
        })
        .collect();
    for piece in sound.chunks(480 * usize::from(channels)) {
        track.push(piece).unwrap();
    }
    track.finish().unwrap();
    wav_samples(&folder.join(format!("{name}.wav")))
}

/// What a WAV capture wrote holds: 24-bit samples after its 68 bytes of header.
fn wav_samples(path: &Path) -> Vec<i32> {
    let bytes = fs::read(path).unwrap();
    bytes[68..]
        .as_chunks::<3>()
        .0
        .iter()
        .map(|b| i32::from_le_bytes([0, b[0], b[1], b[2]]) >> 8)
        .collect()
}

/// A FLAC's rate, channels, depth, its frames as STREAMINFO says, and every sample decoded.
fn flac(path: &Path) -> (u32, usize, u32, Option<u64>, Vec<i32>) {
    let source = MediaSourceStream::new(Box::new(fs::File::open(path).unwrap()), Default::default());
    let mut reader = FlacReader::try_new(source, FormatOptions::default()).unwrap();
    let track = reader.default_track(TrackType::Audio).unwrap();
    let (said, frames) = (track.codec_params.as_ref().unwrap().audio().unwrap().clone(), track.num_frames);
    let mut decoder = FlacDecoder::try_new(&said, &AudioDecoderOptions::default()).unwrap();
    let (mut all, mut packet) = (Vec::new(), Vec::<i32>::new());
    while let Some(read) = reader.next_packet().unwrap() {
        decoder.decode(&read).unwrap().copy_to_vec_interleaved(&mut packet);
        all.extend(packet.iter().map(|sample| sample >> 8));
    }
    let channels = said.channels.unwrap().count();
    (said.sample_rate.unwrap(), channels, said.bits_per_sample.unwrap(), frames, all)
}

fn names(folder: &Path) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(folder)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

/// A session's two tracks, as `record` leaves them at `stopped`: their samples.
fn session(folder: &Path) -> [Vec<i32>; 2] {
    [
        recorded(folder, "local", 48_000, 2, 3.0),
        recorded(folder, "remote", 44_100, 1, 2.0),
    ]
}

/// Each track is there whole — its WAV, its FLAC, or both, each holding every sample: never neither.
fn whole(folder: &Path, samples: &[Vec<i32>; 2]) {
    for (name, samples) in ["local", "remote"].iter().zip(samples) {
        let (wav, kept) = (folder.join(format!("{name}.wav")), folder.join(format!("{name}.flac")));
        assert!(
            wav.exists() || kept.exists(),
            "{name}: neither its WAV nor its FLAC, in {:?}",
            names(folder)
        );
        if wav.exists() {
            assert_eq!(&wav_samples(&wav), samples, "{name}.wav changed");
        }
        if kept.exists() {
            assert_eq!(&flac(&kept).4, samples, "{name}.flac is not its WAV");
        }
    }
}

#[test]
fn each_track_is_kept_as_flac_holding_every_sample_of_its_wav() {
    let folder = tempfile::tempdir().unwrap();
    let samples = session(folder.path());
    let kept = keep::keep(folder.path(), &["local", "remote"], |_, _| true);
    assert_eq!(kept, [("local".to_string(), Kept::Flac), ("remote".to_string(), Kept::Flac)]);
    assert_eq!(names(folder.path()), ["local.flac", "remote.flac"]);
    let (rate, channels, bits, frames, local) = flac(&folder.path().join("local.flac"));
    assert_eq!((rate, channels, bits, frames), (48_000, 2, 24, Some(144_000)));
    assert_eq!(local, samples[0]);
    let (rate, channels, bits, frames, remote) = flac(&folder.path().join("remote.flac"));
    assert_eq!((rate, channels, bits, frames), (44_100, 1, 24, Some(88_200)));
    assert_eq!(remote, samples[1]);
}

#[test]
fn killed_at_any_step_each_track_is_whole_as_wav_flac_or_both() {
    let folder = tempfile::tempdir().unwrap();
    let samples = session(folder.path());
    // What a kill at each step leaves on disk is the folder as it is then: copied aside, read after.
    let seen = tempfile::tempdir().unwrap();
    let mut taken = Vec::new();
    keep::keep(folder.path(), &["local", "remote"], |track, step| {
        let copy = seen.path().join(taken.len().to_string());
        fs::create_dir(&copy).unwrap();
        for name in names(folder.path()) {
            fs::copy(folder.path().join(&name), copy.join(&name)).unwrap();
        }
        taken.push((track.to_string(), step, names(&copy), copy));
        true
    });
    let at = |track: &str, step: Step| taken.iter().find(|(t, s, ..)| t == track && *s == step).unwrap();
    // Before the FLAC is whole, after its rename before the WAV goes, and after.
    assert!(at("local", Step::Writing).2.contains(&"local.flac.part".to_string()));
    assert!(at("local", Step::Checking).2.contains(&"local.flac.part".to_string()));
    assert_eq!(at("local", Step::Renamed).2, ["local.flac", "local.wav", "remote.wav"]);
    assert_eq!(at("local", Step::Removed).2, ["local.flac", "remote.wav"]);
    assert_eq!(at("remote", Step::Removed).2, ["local.flac", "remote.flac"]);
    for (.., copy) in &taken {
        whole(copy, &samples);
    }
}

#[test]
fn a_second_signal_while_compressing_stops_it_and_leaves_the_rest() {
    let folder = tempfile::tempdir().unwrap();
    let samples = session(folder.path());
    let mut blocks = 0;
    let kept = keep::keep(folder.path(), &["local", "remote"], |_, step| {
        blocks += usize::from(step == Step::Writing);
        blocks < 2
    });
    assert_eq!(kept, [("local".to_string(), Kept::Stopped)]);
    assert_eq!(names(folder.path()), ["local.wav", "remote.wav"]);
    whole(folder.path(), &samples);
}

#[test]
fn a_flac_read_back_unequal_is_removed_and_the_wav_kept() {
    let folder = tempfile::tempdir().unwrap();
    let samples = session(folder.path());
    // The disk gives back a byte other than the one written, in the middle of the first FLAC.
    let mut damaged = false;
    let kept = keep::keep(folder.path(), &["local", "remote"], |track, step| {
        if track == "local" && step == Step::Checking && !damaged {
            let part = folder.path().join("local.flac.part");
            let mut bytes = fs::read(&part).unwrap();
            let middle = bytes.len() / 2;
            bytes[middle] ^= 0x10;
            fs::write(&part, bytes).unwrap();
            damaged = true;
        }
        true
    });
    assert!(matches!(&kept[0], (track, Kept::Wav(_)) if track == "local"), "{kept:?}");
    assert_eq!(kept[1], ("remote".to_string(), Kept::Flac));
    assert_eq!(names(folder.path()), ["local.wav", "remote.flac"]);
    whole(folder.path(), &samples);
}

#[test]
fn a_track_not_in_24_bits_is_left_as_its_wav() {
    let folder = tempfile::tempdir().unwrap();
    // The first form's: 16 bits, never compressed.
    let mut track = kitokey_capture::Track::create(&folder.path().join("local.wav"), 48_000, 1).unwrap();
    track.push(&[0.25; 4_800]).unwrap();
    track.finish().unwrap();
    let before = fs::read(folder.path().join("local.wav")).unwrap();
    let kept = keep::keep(folder.path(), &["local", "remote"], |_, _| true);
    assert!(matches!(&kept[..], [(track, Kept::Wav(_))] if track == "local"), "{kept:?}");
    assert_eq!(names(folder.path()), ["local.wav"]);
    assert_eq!(fs::read(folder.path().join("local.wav")).unwrap(), before);
}

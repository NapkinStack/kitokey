//! A stopped track kept as FLAC (`contracts/session-audio/v3`, *Once stopped, a track kept as FLAC*):
//! written by libFLAC, its verify on, as `<track>.flac.part`; read back from the disk by Symphonia
//! and compared with the WAV sample by sample (ADR-0016); renamed `<track>.flac` once equal; and only
//! then the WAV removed. Killed at any step, each track is a whole WAV, a whole FLAC, or both.

use std::{
    ffi::CString,
    fs::{self, File, OpenOptions},
    io::{self, BufReader, Read, Seek, SeekFrom},
    path::Path,
};

use libflac_sys::*;
use symphonia::{
    core::{
        codecs::audio::{AudioDecoder, AudioDecoderOptions},
        formats::{FormatOptions, FormatReader, TrackType},
        io::MediaSourceStream,
    },
    default::{codecs::FlacDecoder, formats::FlacReader},
};

/// Where a track is, told before each step it takes: whoever is told may stop it there.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    /// A block of frames is about to be written to the `.flac.part`; told for each.
    Writing,
    /// The `.flac.part` is whole on the disk, and a block of it is about to be read back; told for each.
    Checking,
    /// `<track>.flac` is in place, the WAV still beside it.
    Renamed,
    /// The WAV is removed: the track is its FLAC alone.
    Removed,
}

/// What a track is kept as, once `keep` is done with it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Kept {
    Flac,
    /// Its WAV alone, and why: no FLAC could be written whole, or read back equal.
    Wav(String),
    /// Its FLAC, and its WAV left beside it, and why: stopped there, or the system refused to remove it.
    Both(String),
    /// Stopped while it was compressed: its WAV, the `.flac.part` removed.
    Stopped,
}

/// Frames handed to the encoder at a time: never a whole track in memory.
const BLOCK: u64 = 4_096;

/// Keeps each of `tracks` that has a WAV in `folder` as FLAC, in that order. `go_on` is told each
/// step; false stops there: the `.flac.part` being written is removed, the rest left as it is, and no
/// other track is begun. Each track begun is returned with what it is kept as.
pub fn keep(folder: &Path, tracks: &[&str], mut go_on: impl FnMut(&str, Step) -> bool) -> Vec<(String, Kept)> {
    let mut kept = Vec::new();
    for &track in tracks {
        let wav = folder.join(format!("{track}.wav"));
        if !wav.exists() {
            continue;
        }
        let (part, flac) = (folder.join(format!("{track}.flac.part")), folder.join(format!("{track}.flac")));
        let mut go_on = |step| go_on(track, step);
        let checked = written(&wav, &part, &mut go_on).and_then(|go| Ok(go && checked(&wav, &part, &mut go_on)?));
        let outcome = match checked {
            Ok(true) => fs::rename(&part, &flac).map_err(|error| Kept::Wav(format!("not renamed: {}", error.kind()))),
            Ok(false) => Err(Kept::Stopped),
            Err(error) => Err(Kept::Wav(error.to_string())),
        };
        if let Err(outcome) = outcome {
            let _ = fs::remove_file(&part);
            let stopped = outcome == Kept::Stopped;
            kept.push((track.to_string(), outcome));
            if stopped {
                break;
            }
            continue;
        }
        synced(folder);
        if !go_on(Step::Renamed) {
            kept.push((track.to_string(), Kept::Both("stopped before its WAV was removed".into())));
            break;
        }
        // On Windows, a WAV another program holds open may be refused: both stay, either is the track.
        if let Err(error) = fs::remove_file(&wav) {
            kept.push((track.to_string(), Kept::Both(format!("its WAV not removed: {}", error.kind()))));
            continue;
        }
        synced(folder);
        kept.push((track.to_string(), Kept::Flac));
        if !go_on(Step::Removed) {
            break;
        }
    }
    kept
}

/// The folder's names on the disk, where the system can say so: a rename or a removal kept across a
/// power cut. A system that opens no folder as a file — Windows — is left to keep them itself.
fn synced(folder: &Path) {
    if let Ok(folder) = File::open(folder) {
        let _ = folder.sync_all();
    }
}

fn refused<T>(why: impl Into<String>) -> io::Result<T> {
    Err(io::Error::other(why.into()))
}

/// A WAV of 24 bits read as the contract tells a reader: its audio found by its chunks, its
/// lengths ignored, to the end of the file — whole frames alone.
struct Wav {
    file: BufReader<File>,
    rate: u32,
    channels: usize,
    /// Frames not read yet.
    frames: u64,
    bytes: Vec<u8>,
}

impl Wav {
    fn open(path: &Path) -> io::Result<Wav> {
        let mut file = BufReader::new(File::open(path)?);
        let mut head = [0u8; 12];
        file.read_exact(&mut head)?;
        if &head[..4] != b"RIFF" || &head[8..] != b"WAVE" {
            return refused("not a WAV");
        }
        let (mut form, mut at) = (None, 12u64);
        loop {
            let mut chunk = [0u8; 8];
            file.read_exact(&mut chunk)?;
            let size = u32::from_le_bytes([chunk[4], chunk[5], chunk[6], chunk[7]]);
            at += 8;
            if &chunk[..4] == b"data" {
                break;
            }
            let mut skipped = i64::from(size);
            if &chunk[..4] == b"fmt " && size >= 16 {
                let mut fmt = [0u8; 16];
                file.read_exact(&mut fmt)?;
                skipped -= 16;
                let tag = u16::from_le_bytes([fmt[0], fmt[1]]);
                let channels = usize::from(u16::from_le_bytes([fmt[2], fmt[3]]));
                let rate = u32::from_le_bytes([fmt[4], fmt[5], fmt[6], fmt[7]]);
                let bits = u16::from_le_bytes([fmt[14], fmt[15]]);
                if !matches!(tag, 1 | 0xFFFE) || bits != 24 || channels == 0 || rate == 0 {
                    return refused(format!("not PCM in 24 bits: {bits} bits"));
                }
                form = Some((rate, channels));
            }
            file.seek_relative(skipped)?;
            at += u64::from(size);
        }
        let Some((rate, channels)) = form else {
            return refused("no form said");
        };
        let end = file.get_mut().seek(SeekFrom::End(0))?;
        file.seek(SeekFrom::Start(at))?;
        let frames = end.saturating_sub(at) / (3 * channels as u64);
        Ok(Wav {
            file,
            rate,
            channels,
            frames,
            bytes: Vec::new(),
        })
    }

    /// The next `frames` frames, interleaved, each sample as it was written.
    fn read(&mut self, frames: u64, samples: &mut Vec<i32>) -> io::Result<()> {
        self.frames -= frames.min(self.frames);
        self.bytes.resize(frames as usize * self.channels * 3, 0);
        self.file.read_exact(&mut self.bytes)?;
        samples.clear();
        samples.extend(
            self.bytes
                .as_chunks::<3>()
                .0
                .iter()
                .map(|b| i32::from_le_bytes([0, b[0], b[1], b[2]]) >> 8),
        );
        Ok(())
    }
}

/// Writes the WAV's frames into `part` with libFLAC, its verify on, and syncs it: false when stopped.
fn written(wav: &Path, part: &Path, go_on: &mut impl FnMut(Step) -> bool) -> io::Result<bool> {
    let mut wav = Wav::open(wav)?;
    let Some(path) = part.to_str().and_then(|path| CString::new(path).ok()) else {
        return refused("a path libFLAC cannot be given");
    };
    let mut samples = Vec::new();
    // SAFETY: the encoder is made, used and deleted here, each call on it given samples that outlive it.
    let outcome = unsafe {
        let encoder = FLAC__stream_encoder_new();
        if encoder.is_null() {
            return refused("libFLAC made no encoder");
        }
        let set = FLAC__stream_encoder_set_channels(encoder, wav.channels as u32)
            & FLAC__stream_encoder_set_bits_per_sample(encoder, 24)
            & FLAC__stream_encoder_set_sample_rate(encoder, wav.rate)
            & FLAC__stream_encoder_set_compression_level(encoder, 5)
            & FLAC__stream_encoder_set_verify(encoder, 1)
            & FLAC__stream_encoder_set_total_samples_estimate(encoder, wav.frames);
        let mut outcome = if set == 0 {
            refused("libFLAC refused the track's form")
        } else if FLAC__stream_encoder_init_file(encoder, path.as_ptr(), None, std::ptr::null_mut()) != 0 {
            refused("libFLAC could not open the file")
        } else {
            Ok(true)
        };
        while matches!(outcome, Ok(true)) && wav.frames > 0 {
            if !go_on(Step::Writing) {
                outcome = Ok(false);
                break;
            }
            let frames = wav.frames.min(BLOCK);
            outcome = wav.read(frames, &mut samples).and_then(|()| {
                match FLAC__stream_encoder_process_interleaved(encoder, samples.as_ptr(), frames as u32) {
                    0 => refused(format!(
                        "libFLAC stopped writing (state {})",
                        FLAC__stream_encoder_get_state(encoder)
                    )),
                    _ => Ok(true),
                }
            });
        }
        if matches!(outcome, Ok(true)) && FLAC__stream_encoder_finish(encoder) == 0 {
            outcome = refused(format!(
                "libFLAC could not end the file (state {})",
                FLAC__stream_encoder_get_state(encoder)
            ));
        }
        FLAC__stream_encoder_delete(encoder);
        outcome
    };
    if matches!(outcome, Ok(true)) {
        OpenOptions::new().write(true).open(part)?.sync_all()?;
    }
    outcome
}

/// Reads `part` back from the disk with Symphonia and compares it with the WAV: its form, its frames
/// as STREAMINFO says them, every sample, and the MD5 libFLAC wrote of them. False when stopped.
fn checked(wav: &Path, part: &Path, go_on: &mut impl FnMut(Step) -> bool) -> io::Result<bool> {
    let mut wav = Wav::open(wav)?;
    let unread = |error: symphonia::core::errors::Error| io::Error::other(format!("not read back ({error})"));
    let source = MediaSourceStream::new(Box::new(File::open(part)?), Default::default());
    let mut reader = FlacReader::try_new(source, FormatOptions::default()).map_err(unread)?;
    let Some((said, frames)) = reader
        .default_track(TrackType::Audio)
        .and_then(|track| Some((track.codec_params.as_ref()?.audio()?.clone(), track.num_frames)))
    else {
        return refused("read back, no audio");
    };
    let channels = said.channels.as_ref().map_or(0, |channels| channels.count());
    if (said.sample_rate, channels, said.bits_per_sample, frames) != (Some(wav.rate), wav.channels, Some(24), Some(wav.frames)) {
        return refused("read back in another form or length");
    }
    let mut options = AudioDecoderOptions::default();
    options.verify = true;
    let mut decoder = FlacDecoder::try_new(&said, &options).map_err(unread)?;
    let (mut decoded, mut expected) = (Vec::<i32>::new(), Vec::new());
    while let Some(packet) = reader.next_packet().map_err(unread)? {
        if !go_on(Step::Checking) {
            return Ok(false);
        }
        decoder.decode(&packet).map_err(unread)?.copy_to_vec_interleaved(&mut decoded);
        let frames = (decoded.len() / channels) as u64;
        if frames > wav.frames {
            return refused("read back longer than its WAV");
        }
        wav.read(frames, &mut expected)?;
        // Symphonia gives 24 bits at the full scale of 32.
        if decoded.iter().zip(&expected).any(|(decoded, expected)| decoded >> 8 != *expected) {
            return refused("read back, a sample unequal");
        }
    }
    if wav.frames != 0 || decoder.finalize().verify_ok != Some(true) {
        return refused("read back shorter than its WAV, or not as libFLAC summed it");
    }
    Ok(true)
}

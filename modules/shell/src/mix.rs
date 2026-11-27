//! A meeting as the window plays it (ADR-0009): one sound, each track placed where the
//! session's clock puts it, its channels averaged, the tracks added — a WAV of one channel of
//! 16 bits at the highest of the tracks' rates, made as it is asked for, by any range, and kept
//! nowhere. The tracks are only read (`session-audio`: found by their chunks, their lengths
//! ignored, read to the end); one of a lower rate is brought up with rubato, as the transcription
//! brings its tracks down. A track kept as FLAC (`session-audio/v3`) is read as its WAV was: its
//! form from its STREAMINFO, its frames decoded from the one asked — and a WAV removed under the mix,
//! once kept as FLAC, is read from its FLAC, which holds the same frames.

use std::{
    fs::File,
    io::{self, Read, Seek, SeekFrom},
    path::{Path, PathBuf},
};

use crate::flac::{self, Decoded};
use rubato::{Fft, FixedSync, Resampler, WindowFunction, audioadapter_buffers::direct::InterleavedSlice};

/// The mix's header: RIFF, `fmt `, `data`.
const HEADER: u64 = 44;

/// A track as its own header says it is.
struct Track {
    path: PathBuf,
    /// A FLAC, read by its frames; a WAV, by its bytes.
    flac: bool,
    /// Where a WAV's audio starts in the file; the frame of the file this part of the track starts
    /// at; and how many whole frames it holds, to the file's end.
    data: u64,
    first: u64,
    frames: u64,
    rate: u32,
    channels: usize,
    /// Bytes a sample of a WAV: 2, 3 or 4 for PCM; 4 for float.
    width: usize,
    float: bool,
    /// The mix's frame its frame 0 falls on: before the session's start, below zero.
    at: i64,
}

pub struct Mix {
    rate: u32,
    frames: u64,
    tracks: Vec<Track>,
}

impl Mix {
    /// The mix of these tracks, each with where its first frame is on the session's clock, in
    /// seconds; none when not one of them reads as a track.
    pub fn of(tracks: &[(PathBuf, f64)]) -> Option<Mix> {
        let placed: Vec<(PathBuf, Vec<(u64, f64)>)> = tracks.iter().map(|(path, starts)| (path.clone(), vec![(0, *starts)])).collect();
        Mix::placed(&placed)
    }

    /// The mix of these tracks, each placed run by run (D71): a frame of the track, and where it
    /// is on the session's clock, in seconds. A run the clock puts before the end of the one
    /// before stops that one there: never two sounds of one track added.
    pub fn placed(tracks: &[(PathBuf, Vec<(u64, f64)>)]) -> Option<Mix> {
        // A start that is no instant, or beyond any meeting, leaves its track out.
        let read: Vec<(Track, &Vec<(u64, f64)>)> = tracks
            .iter()
            .filter(|(_, runs)| runs.iter().all(|(_, at)| at.is_finite() && at.abs() < 1e6))
            .filter_map(|(path, runs)| Some((track(path)?, runs)))
            .collect();
        let rate = read.iter().map(|(track, _)| track.rate).max()?;
        let mut tracks: Vec<Track> = Vec::new();
        for (whole, runs) in read {
            let frames_of = |seconds: f64| (seconds * f64::from(whole.rate)).round() as i64;
            for (index, &(from, at)) in runs.iter().enumerate() {
                let later = &runs[index + 1..];
                let held = later
                    .first()
                    .map_or(whole.frames, |(next, _)| (*next).min(whole.frames))
                    .saturating_sub(from);
                // Up to the earliest run the clock puts after it: any of them, not only the next.
                let room = later.iter().map(|(_, then)| frames_of(then - at).max(0)).min().unwrap_or(i64::MAX);
                let frames = held.min(u64::try_from(room).unwrap_or(u64::MAX));
                // A run with nothing left to play is no part of the sound; an empty track stays one.
                if frames > 0 || (index == 0 && whole.frames == 0) {
                    tracks.push(Track {
                        path: whole.path.clone(),
                        first: from,
                        frames,
                        at: (at * f64::from(rate)).round() as i64,
                        ..whole
                    });
                }
            }
        }
        let end = tracks
            .iter()
            .map(|track| track.at + (track.frames as f64 * f64::from(rate) / f64::from(track.rate)).round() as i64)
            .max()?;
        Some(Mix {
            rate,
            frames: end.max(0) as u64,
            tracks,
        })
    }

    pub fn rate(&self) -> u32 {
        self.rate
    }

    pub fn frames(&self) -> u64 {
        self.frames
    }

    /// Whether it is made of the very same tracks as `other`: each placed at the same frame and
    /// holding as many — a track grown since, or moved, makes another mix.
    pub fn same(&self, other: &Mix) -> bool {
        self.rate == other.rate
            && self.tracks.len() == other.tracks.len()
            && self.tracks.iter().zip(&other.tracks).all(|(one, two)| {
                (&one.path, one.first, one.at, one.frames, one.rate) == (&two.path, two.first, two.at, two.frames, two.rate)
            })
    }

    /// Its length in bytes, header included.
    pub fn size(&self) -> u64 {
        HEADER + 2 * self.frames
    }

    /// `length` bytes of the mix from byte `from`, as a file of it would hold them.
    pub fn read(&self, from: u64, length: u64) -> io::Result<Vec<u8>> {
        if from >= self.size() {
            return Ok(Vec::new());
        }
        let to = from.saturating_add(length).min(self.size());
        let mut bytes = Vec::with_capacity(to.saturating_sub(from) as usize);
        if from < HEADER {
            bytes.extend_from_slice(&self.header()[from as usize..to.min(HEADER) as usize]);
        }
        if to > HEADER {
            let start = from.max(HEADER) - HEADER;
            let (first, last) = (start / 2, (to - HEADER).div_ceil(2));
            let mut samples = Vec::with_capacity(2 * (last - first) as usize);
            for sample in self.render(first, (last - first) as usize)? {
                samples.extend_from_slice(&((sample.clamp(-1.0, 1.0) * 32767.0).round() as i16).to_le_bytes());
            }
            let skip = (start - 2 * first) as usize;
            bytes.extend_from_slice(&samples[skip..skip + (to - HEADER - start) as usize]);
        }
        Ok(bytes)
    }

    fn header(&self) -> [u8; HEADER as usize] {
        let data = u32::try_from(2 * self.frames).unwrap_or(u32::MAX);
        let mut header = [0u8; HEADER as usize];
        header[0..4].copy_from_slice(b"RIFF");
        header[4..8].copy_from_slice(&data.saturating_add(36).to_le_bytes());
        header[8..16].copy_from_slice(b"WAVEfmt ");
        header[16..20].copy_from_slice(&16u32.to_le_bytes());
        header[20..22].copy_from_slice(&1u16.to_le_bytes());
        header[22..24].copy_from_slice(&1u16.to_le_bytes());
        header[24..28].copy_from_slice(&self.rate.to_le_bytes());
        header[28..32].copy_from_slice(&(2 * self.rate).to_le_bytes());
        header[32..34].copy_from_slice(&2u16.to_le_bytes());
        header[34..36].copy_from_slice(&16u16.to_le_bytes());
        header[36..40].copy_from_slice(b"data");
        header[40..44].copy_from_slice(&data.to_le_bytes());
        header
    }

    /// `count` frames of the mix from `first`, the tracks added.
    fn render(&self, first: u64, count: usize) -> io::Result<Vec<f32>> {
        let mut sum = vec![0.0f32; count];
        for track in &self.tracks {
            if track.rate == self.rate {
                for (into, sample) in sum.iter_mut().zip(track.mono(&mut None, first as i64 - track.at, count)?) {
                    *into += sample;
                }
            } else {
                track.raised(self.rate, first as i64, &mut sum)?;
            }
        }
        Ok(sum)
    }
}

/// A track's file open for reading, kept from one piece of the track to the next.
enum Source {
    Wav(File),
    Flac(Box<Decoded>),
}

impl Track {
    /// Its file, opened: a WAV that cannot be opened — removed once kept as FLAC — is read from
    /// its FLAC beside it, the same frames (`session-audio/v3`).
    fn open(&self) -> io::Result<Source> {
        if self.flac {
            return Ok(Source::Flac(Box::new(flac::open(&self.path)?)));
        }
        match File::open(&self.path) {
            Ok(file) => Ok(Source::Wav(file)),
            Err(error) => {
                let kept = self.path.with_extension("flac");
                match flac::open(&kept) {
                    Ok(decoded) if decoded.form().frames >= self.first + self.frames => Ok(Source::Flac(Box::new(decoded))),
                    _ => Err(error),
                }
            }
        }
    }

    /// `count` frames from frame `from`, each one channel: the average of its channels; zero
    /// where the track has none.
    fn mono(&self, source: &mut Option<Source>, from: i64, count: usize) -> io::Result<Vec<f32>> {
        let mut mono = vec![0.0f32; count];
        let (start, end) = (from.max(0), (from + count as i64).min(self.frames as i64));
        if start >= end {
            return Ok(mono);
        }
        let opened = match source {
            Some(opened) => opened,
            None => source.insert(self.open()?),
        };
        let into = &mut mono[(start - from) as usize..(end - from) as usize];
        match opened {
            Source::Wav(file) => {
                let block = self.channels * self.width;
                let mut bytes = vec![0u8; into.len() * block];
                file.seek(SeekFrom::Start(self.data + (self.first + start as u64) * block as u64))?;
                file.read_exact(&mut bytes)?;
                for (frame, into) in bytes.chunks_exact(block).zip(into) {
                    let total: f32 = frame.chunks_exact(self.width).map(|sample| self.sample(sample)).sum();
                    *into = total / self.channels as f32;
                }
            }
            Source::Flac(decoded) => {
                decoded.seek(self.first + start as u64)?;
                for into in into {
                    let frame = decoded
                        .frame()?
                        .ok_or_else(|| io::Error::new(io::ErrorKind::UnexpectedEof, "a track ends before its length"))?;
                    // At the full scale of 32 bits, whatever its depth: the very value its WAV gave.
                    let total: f32 = frame.iter().map(|&sample| sample as f32 / 2_147_483_648.0).sum();
                    *into = total / self.channels as f32;
                }
            }
        }
        Ok(mono)
    }

    fn sample(&self, bytes: &[u8]) -> f32 {
        match (self.float, self.width) {
            (true, _) => f32::from_le_bytes(bytes.try_into().unwrap_or_default()),
            (false, 2) => f32::from(i16::from_le_bytes([bytes[0], bytes[1]])) / 32_768.0,
            (false, 3) => (i32::from_le_bytes([0, bytes[0], bytes[1], bytes[2]]) >> 8) as f32 / 8_388_608.0,
            _ => i32::from_le_bytes(bytes.try_into().unwrap_or_default()) as f32 / 2_147_483_648.0,
        }
    }

    /// Adds this track, of a lower rate, brought up to `rate`, into `sum`: the mix's frames from
    /// `first`. The resampler always starts on the same grid of its blocks, each on a frame of the
    /// track that falls on a frame of the mix, so a range of the mix is the same bytes as the whole.
    fn raised(&self, rate: u32, first: i64, sum: &mut [f32]) -> io::Result<()> {
        let failed = |error: &dyn std::fmt::Display| io::Error::other(format!("a track could not be brought to {rate} Hz: {error}"));
        let shared = gcd(self.rate, rate);
        let (taken, given) = (i64::from(self.rate / shared), i64::from(rate / shared));
        // Blocks of a whole number of the ratio's steps, each starting on the same grid: a block
        // starts on a frame of the mix, and the resampler is given the same blocks whoever asks.
        let chunk = taken * ((1024 + taken - 1) / taken);
        let mut resampler = Fft::<f32>::new_custom(
            self.rate as usize,
            rate as usize,
            chunk as usize,
            1,
            1,
            WindowFunction::BlackmanHarris2,
            FixedSync::Input,
        )
        .map_err(|e| failed(&e))?;
        let delay = resampler.output_delay() as i64;
        // Two blocks before the first frame asked, for the resampler to settle.
        let wanted = ((first - self.at) as f64 * taken as f64 / given as f64).floor() as i64;
        let start = (wanted - 2 * chunk).div_euclid(chunk) * chunk;
        let from = self.at + start / taken * given;
        let (mut made, mut read) = (Vec::new(), start);
        let mut out = vec![0.0f32; resampler.output_frames_max()];
        // One file opened for every piece: a FLAC decodes on from where the piece before ended.
        let mut opened = None;
        while (made.len() as i64) < delay + first + sum.len() as i64 - from {
            let piece = self.mono(&mut opened, read, chunk as usize)?;
            read += chunk;
            let source = InterleavedSlice::new(&piece, 1, piece.len()).map_err(|e| failed(&e))?;
            let room = out.len();
            let mut into = InterleavedSlice::new_mut(&mut out, 1, room).map_err(|e| failed(&e))?;
            let (_, written) = resampler.process_into_buffer(&source, &mut into, None).map_err(|e| failed(&e))?;
            made.extend_from_slice(&out[..written]);
        }
        let skip = (delay + first - from) as usize;
        for (into, sample) in sum.iter_mut().zip(&made[skip..]) {
            *into += sample;
        }
        Ok(())
    }
}

fn gcd(a: u32, b: u32) -> u32 {
    if b == 0 { a } else { gcd(b, a % b) }
}

/// A track as its header says it is, or nothing when it does not read as one.
fn track(path: &Path) -> Option<Track> {
    if flac::is_flac(path) {
        let form = flac::form(path)?;
        return (form.channels > 0 && (1_000..=768_000).contains(&form.rate)).then(|| Track {
            path: path.to_path_buf(),
            flac: true,
            data: 0,
            first: 0,
            frames: form.frames,
            rate: form.rate,
            channels: form.channels,
            width: 0,
            float: false,
            at: 0,
        });
    }
    let mut file = File::open(path).ok()?;
    let size = file.metadata().ok()?.len();
    let mut head = vec![0u8; 64 * 1024];
    let read = file.read(&mut head).ok()?;
    let head = &head[..read];
    if head.get(0..4)? != b"RIFF" || head.get(8..12)? != b"WAVE" {
        return None;
    }
    let (mut at, mut format) = (12usize, None);
    while at + 8 <= head.len() {
        let length = u32::from_le_bytes(head[at + 4..at + 8].try_into().ok()?) as usize;
        match &head[at..at + 4] {
            b"fmt " => {
                let fmt = head.get(at + 8..at + 24)?;
                // WAVE_FORMAT_EXTENSIBLE says its own in the first two bytes of its sub-format,
                // in a `fmt ` of 40 bytes.
                let tag = match u16::from_le_bytes([fmt[0], fmt[1]]) {
                    0xFFFE if length >= 40 => u16::from_le_bytes(head.get(at + 32..at + 34)?.try_into().ok()?),
                    0xFFFE => return None,
                    tag => tag,
                };
                let channels = usize::from(u16::from_le_bytes([fmt[2], fmt[3]]));
                let rate = u32::from_le_bytes(fmt[4..8].try_into().ok()?);
                // A sample's width is its block's share, which its bits fill: 20 bits in 3 bytes.
                let block = usize::from(u16::from_le_bytes([fmt[12], fmt[13]]));
                let bits = usize::from(u16::from_le_bytes([fmt[14], fmt[15]]));
                let width = block.checked_div(channels)?;
                if block != width * channels || bits.div_ceil(8) != width || !(1_000..=768_000).contains(&rate) {
                    return None;
                }
                let float = match (tag, width) {
                    (1, 2..=4) => false,
                    (3, 4) => true,
                    _ => return None,
                };
                format = Some((channels, rate, width, float));
            }
            b"data" => {
                let (channels, rate, width, float) = format?;
                if channels == 0 || rate == 0 {
                    return None;
                }
                let data = (at + 8) as u64;
                return Some(Track {
                    path: path.to_path_buf(),
                    flac: false,
                    data,
                    first: 0,
                    frames: size.saturating_sub(data) / (channels * width) as u64,
                    rate,
                    channels,
                    width,
                    float,
                    at: 0,
                });
            }
            _ => {}
        }
        at = at.checked_add(8 + length + length % 2)?;
    }
    None
}

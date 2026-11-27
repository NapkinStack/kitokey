//! A track kept as FLAC once its recording stopped (`contracts/session-audio/v3`), read with Symphonia
//! (ADR-0016): its form from its STREAMINFO, its frames decoded from any frame on, each sample at
//! the full scale of 32 bits — so that a FLAC is read as its WAV was. Symphonia passes over a
//! damaged frame without a word: a frame that does not follow the one before is an error here, never a
//! sound played shorter or later than it was heard.

use std::{
    fs::File,
    io::{self, Read},
    path::Path,
};

use symphonia::{
    core::{
        codecs::audio::{AudioDecoder, AudioDecoderOptions},
        formats::{FormatOptions, FormatReader, SeekMode, SeekTo, TrackType},
        io::MediaSourceStream,
        units::Timestamp,
    },
    default::{codecs::FlacDecoder, formats::FlacReader},
};

/// Whether the file is a FLAC, by its first bytes — never by its name.
pub fn is_flac(path: &Path) -> bool {
    let mut head = [0u8; 4];
    File::open(path).and_then(|mut file| file.read_exact(&mut head)).is_ok() && &head == b"fLaC"
}

/// What its STREAMINFO says: its rate, channels, depth and length in frames.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Form {
    pub rate: u32,
    pub channels: usize,
    pub bits: u32,
    pub frames: u64,
}

fn unread(path: &Path, why: impl std::fmt::Display) -> io::Error {
    let name = path.file_name().map(|name| name.to_string_lossy().into_owned()).unwrap_or_default();
    io::Error::new(io::ErrorKind::InvalidData, format!("{name} cannot be decoded ({why})"))
}

/// A FLAC open for reading, from frame 0 or wherever it was sought.
pub struct Decoded {
    reader: FlacReader<'static>,
    decoder: FlacDecoder,
    form: Form,
    track: u32,
    path: std::path::PathBuf,
    /// The last packet decoded, its channels interleaved, and how many of its samples were given.
    samples: Vec<i32>,
    given: usize,
    /// The frame the next one given is; whether every frame was given from frame 0 on, in order.
    at: u64,
    whole: bool,
}

/// The FLAC at `path`, open at its frame 0; none of a form a track can have — no length said, no
/// channel, no rate.
pub fn open(path: &Path) -> io::Result<Decoded> {
    let source = MediaSourceStream::new(Box::new(File::open(path)?), Default::default());
    let reader = FlacReader::try_new(source, FormatOptions::default()).map_err(|error| unread(path, error))?;
    let track = reader.default_track(TrackType::Audio).ok_or_else(|| unread(path, "no track"))?;
    let said = track
        .codec_params
        .as_ref()
        .and_then(|params| params.audio())
        .cloned()
        .ok_or_else(|| unread(path, "no audio"))?;
    let form = Form {
        rate: said.sample_rate.unwrap_or(0),
        channels: said.channels.as_ref().map_or(0, |channels| channels.count()),
        bits: said.bits_per_sample.unwrap_or(0),
        frames: track.num_frames.ok_or_else(|| unread(path, "no length said"))?,
    };
    if form.rate == 0 || form.channels == 0 || form.bits == 0 {
        return Err(unread(path, "no form said"));
    }
    let id = track.id;
    // Verified: the samples of a whole reading are checked against the MD5 libFLAC wrote.
    let decoder = FlacDecoder::try_new(&said, &AudioDecoderOptions::default().verify(true)).map_err(|error| unread(path, error))?;
    Ok(Decoded {
        reader,
        decoder,
        form,
        track: id,
        path: path.to_path_buf(),
        samples: Vec::new(),
        given: 0,
        at: 0,
        whole: true,
    })
}

/// The form of the FLAC at `path`, as its STREAMINFO says it.
pub fn form(path: &Path) -> Option<Form> {
    open(path).ok().map(|decoded| decoded.form)
}

impl Decoded {
    pub fn form(&self) -> Form {
        self.form
    }

    /// Moves to `frame`: the next frame given is that one. Symphonia's demuxer finds it by a binary
    /// search over the file, then the frames before it in its packet are dropped.
    pub fn seek(&mut self, frame: u64) -> io::Result<()> {
        if frame == self.at {
            return Ok(());
        }
        let to = SeekTo::Timestamp {
            ts: Timestamp::new(i64::try_from(frame).map_err(|error| unread(&self.path, error))?),
            track_id: self.track,
        };
        self.reader
            .seek(SeekMode::Accurate, to)
            .map_err(|error| unread(&self.path, error))?;
        self.decoder.reset();
        self.samples.clear();
        (self.given, self.at, self.whole) = (0, frame, false);
        Ok(())
    }

    /// The next frame's samples, one a channel, at the full scale of 32 bits; none past the last
    /// frame its STREAMINFO says. A frame missing, or the stream ending before, is an error.
    pub fn frame(&mut self) -> io::Result<Option<&[i32]>> {
        if self.at >= self.form.frames {
            return Ok(None);
        }
        while self.given == self.samples.len() {
            let packet = self
                .reader
                .next_packet()
                .map_err(|error| unread(&self.path, error))?
                .ok_or_else(|| unread(&self.path, "it ends before its length"))?;
            let first = u64::try_from(packet.pts.get()).map_err(|error| unread(&self.path, error))?;
            // A packet beginning after the frame wanted: frames were lost on the way, a damaged one passed over.
            if first > self.at {
                return Err(unread(&self.path, format!("frames {} to {first} are missing", self.at)));
            }
            self.decoder
                .decode(&packet)
                .map_err(|error| unread(&self.path, error))?
                .copy_to_vec_interleaved(&mut self.samples);
            self.given = (((self.at - first) as usize) * self.form.channels).min(self.samples.len());
        }
        let from = self.given;
        self.given += self.form.channels;
        self.at += 1;
        // Read whole from its frame 0, the last frame given: its samples are what libFLAC checked.
        if self.whole && self.at == self.form.frames && self.decoder.finalize().verify_ok == Some(false) {
            return Err(unread(&self.path, "its samples are not those it was written with"));
        }
        Ok(Some(&self.samples[from..from + self.form.channels]))
    }
}

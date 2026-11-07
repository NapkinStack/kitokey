//! A track kept as FLAC once its recording stopped (`contracts/session-audio/v3`), read with
//! Symphonia (ADR-0016) and handed on as the WAV held it: 24-bit frames, little-endian, the
//! channels interleaved — so that what follows reads a FLAC as it reads a WAV.

use std::{
    fs::File,
    io::{self, Read},
    path::Path,
};

use symphonia::{
    core::{
        codecs::audio::{AudioDecoder, AudioDecoderOptions},
        formats::{FormatOptions, FormatReader, TrackType},
        io::MediaSourceStream,
    },
    default::{codecs::FlacDecoder, formats::FlacReader},
};

use crate::tracks::RATES;

fn refused<T>(why: String) -> io::Result<T> {
    Err(io::Error::new(io::ErrorKind::InvalidData, why))
}

/// The track at `path`, its rate and its channels; a file that is not a track a session holds is
/// said with its name.
pub(crate) fn opened(path: &Path, name: &str) -> io::Result<(Box<dyn Read>, u32, usize)> {
    let not_a_track = || refused(format!("{name} is not a track of a session"));
    let source = MediaSourceStream::new(Box::new(File::open(path)?), Default::default());
    let Ok(reader) = FlacReader::try_new(source, FormatOptions::default()) else {
        return not_a_track();
    };
    let Some(said) = reader
        .default_track(TrackType::Audio)
        .and_then(|track| track.codec_params.as_ref())
        .and_then(|params| params.audio())
        .cloned()
    else {
        return not_a_track();
    };
    let channels = said.channels.as_ref().map_or(0, |channels| channels.count());
    let (rate, bits) = (said.sample_rate.unwrap_or(0), said.bits_per_sample.unwrap_or(0));
    if bits != 24 || channels == 0 || !RATES.contains(&rate) {
        return refused(format!(
            "{name} is not what a session holds — PCM in 24 bits: it has {channels} channel(s), {rate} Hz, {bits} bits"
        ));
    }
    let Ok(decoder) = FlacDecoder::try_new(&said, &AudioDecoderOptions::default()) else {
        return not_a_track();
    };
    let decoded = Decoded {
        reader,
        decoder,
        name: name.to_string(),
        samples: Vec::new(),
        bytes: Vec::new(),
        at: 0,
    };
    Ok((Box::new(decoded), rate, channels))
}

/// The track's frames as the WAV held them, decoded a block at a time: never held whole.
struct Decoded {
    reader: FlacReader<'static>,
    decoder: FlacDecoder,
    name: String,
    samples: Vec<i32>,
    bytes: Vec<u8>,
    at: usize,
}

impl Read for Decoded {
    fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
        while self.at == self.bytes.len() {
            let unread = |error: symphonia::core::errors::Error| {
                io::Error::new(io::ErrorKind::InvalidData, format!("{} cannot be decoded ({error})", self.name))
            };
            let Some(packet) = self.reader.next_packet().map_err(unread)? else {
                return Ok(0);
            };
            // Symphonia gives 24 bits at the full scale of 32: the low byte is always zero.
            self.decoder
                .decode(&packet)
                .map_err(unread)?
                .copy_to_vec_interleaved(&mut self.samples);
            self.bytes.clear();
            for sample in &self.samples {
                self.bytes.extend_from_slice(&(sample >> 8).to_le_bytes()[..3]);
            }
            self.at = 0;
        }
        let given = out.len().min(self.bytes.len() - self.at);
        out[..given].copy_from_slice(&self.bytes[self.at..self.at + given]);
        self.at += given;
        Ok(given)
    }
}

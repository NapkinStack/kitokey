//! A track kept as FLAC once its recording stopped, as capture keeps it (`session-audio/v3`): written by
//! libFLAC, the reference (ADR-0016), its verify on, from the WAV's own samples — and the WAV removed.

use std::{
    ffi::CString,
    fs,
    path::{Path, PathBuf},
};

use libflac_sys::*;

/// Writes `<track>.flac` from the PCM WAV at `wav`, 16 or 24 bits as it holds them, and removes the
/// WAV: the same rate, channels, depth and samples.
pub fn kept_as_flac(wav: &Path) -> PathBuf {
    let bytes = fs::read(wav).unwrap();
    let channels = u32::from(u16::from_le_bytes([bytes[22], bytes[23]]));
    let rate = u32::from_le_bytes([bytes[24], bytes[25], bytes[26], bytes[27]]);
    let bits = u32::from(u16::from_le_bytes([bytes[34], bytes[35]]));
    let data = bytes.windows(4).position(|chunk| chunk == b"data").unwrap() + 8;
    let mut samples: Vec<i32> = match bits {
        16 => bytes[data..]
            .as_chunks::<2>()
            .0
            .iter()
            .map(|b| i32::from(i16::from_le_bytes(*b)))
            .collect(),
        24 => bytes[data..]
            .as_chunks::<3>()
            .0
            .iter()
            .map(|b| i32::from_le_bytes([0, b[0], b[1], b[2]]) >> 8)
            .collect(),
        _ => panic!("a FLAC of {bits} bits is not what a session keeps"),
    };
    // A last frame cut by a crash is no frame.
    samples.truncate(samples.len() - samples.len() % channels as usize);
    let frames = samples.len() / channels as usize;
    let flac = wav.with_extension("flac");
    let path = CString::new(flac.to_str().unwrap()).unwrap();
    // SAFETY: the encoder is made, used and deleted here, on samples that outlive it.
    unsafe {
        let encoder = FLAC__stream_encoder_new();
        assert!(!encoder.is_null());
        FLAC__stream_encoder_set_channels(encoder, channels);
        FLAC__stream_encoder_set_bits_per_sample(encoder, bits);
        FLAC__stream_encoder_set_sample_rate(encoder, rate);
        FLAC__stream_encoder_set_compression_level(encoder, 5);
        FLAC__stream_encoder_set_verify(encoder, 1);
        assert_eq!(
            FLAC__stream_encoder_init_file(encoder, path.as_ptr(), None, std::ptr::null_mut()),
            0
        );
        assert_eq!(
            FLAC__stream_encoder_process_interleaved(encoder, samples.as_ptr(), frames as u32),
            1
        );
        assert_eq!(FLAC__stream_encoder_finish(encoder), 1);
        FLAC__stream_encoder_delete(encoder);
    }
    fs::remove_file(wav).unwrap();
    flac
}

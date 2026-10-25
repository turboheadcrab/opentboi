//! MP3 to PCM with Symphonia: SDL has no MP3 decoder (that is `SDL_mixer`'s job).

use std::io::Cursor;
use std::sync::Arc;

use sdl3::audio::{AudioFormat, AudioSpec};
use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::DecoderOptions;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::{MediaSourceStream, MediaSourceStreamOptions};
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

use crate::game_assets::SoundClip;

const MP3_EXTENSION: &str = "mp3";

pub struct DecodedClip {
    pub spec: AudioSpec,
    /// Interleaved samples, in `spec`'s channel count.
    pub samples: Vec<f32>,
}

/// Decodes a clip to interleaved `f32` samples without the encoder's delay and padding; `None`
/// (with a message) if it is unreadable.
pub fn decode(clip: &SoundClip) -> Option<DecodedClip> {
    let mut decoded = decode_mp3(&clip.mp3)?;
    if decoded.spec.freq != i32::try_from(clip.sample_rate).ok() {
        println!(
            "Sound clip decodes at {:?} Hz, not the {} Hz it declares; its trim may be off",
            decoded.spec.freq, clip.sample_rate
        );
    }
    let channels = usize::try_from(decoded.spec.channels.unwrap_or(1)).unwrap_or(1).max(1);
    let start = (clip.leading_samples * channels).min(decoded.samples.len());
    let end = (start + clip.sample_count * channels).min(decoded.samples.len());
    decoded.samples = decoded.samples[start..end].to_vec();
    Some(decoded)
}

/// Decodes one MP3 to interleaved `f32` samples; `None` (with a message) if it is unreadable.
pub fn decode_mp3(encoded: &Arc<[u8]>) -> Option<DecodedClip> {
    let source = MediaSourceStream::new(
        Box::new(Cursor::new(Arc::clone(encoded))),
        MediaSourceStreamOptions::default(),
    );
    let mut hint = Hint::new();
    hint.with_extension(MP3_EXTENSION);
    let probed = symphonia::default::get_probe()
        .format(&hint, source, &FormatOptions::default(), &MetadataOptions::default())
        .inspect_err(|error| println!("Unreadable sound clip: {error}"))
        .ok()?;
    let mut format = probed.format;
    let track = format.default_track()?;
    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &DecoderOptions::default())
        .ok()?;
    let mut samples = Vec::new();
    let mut spec = None;
    // The reader reports the end of the clip as an error.
    while let Ok(packet) = format.next_packet() {
        let Ok(audio) = decoder.decode(&packet) else { continue };
        let signal_spec = *audio.spec();
        let mut buffer = SampleBuffer::<f32>::new(audio.capacity() as u64, signal_spec);
        buffer.copy_interleaved_ref(audio);
        samples.extend_from_slice(buffer.samples());
        spec = Some(AudioSpec::new(
            i32::try_from(signal_spec.rate).ok(),
            i32::try_from(signal_spec.channels.count()).ok(),
            Some(AudioFormat::f32_sys()),
        ));
    }
    Some(DecodedClip { spec: spec?, samples })
}

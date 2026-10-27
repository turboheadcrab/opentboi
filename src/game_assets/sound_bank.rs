//! Loads the game's sound effects from the extracted SWF, keyed by the event that plays them.

use std::collections::HashMap;
use std::sync::Arc;

use crate::game::{MusicTrack, SoundEvent};

mod catalog;
mod music;
use catalog::SOUND_CATALOG;
pub use music::MusicClips;

/// MP3 `DefineSound` data starts with a 16-bit "seek samples" field before the MP3 frames.
const MP3_SEEK_SAMPLES_FIELD_SIZE: usize = 2;

/// One recording: a standalone MP3, and which of its decoded samples are the sound.
#[derive(Clone, Debug)]
pub struct SoundClip {
    pub mp3: Arc<[u8]>,
    /// The encoder's delay before the sound starts (`SeekSamples`), per channel.
    pub leading_samples: usize,
    /// The sound's length (`SampleCount`) per channel; the MP3 frames hold padding after it,
    /// which would leave a gap in a loop.
    pub sample_count: usize,
    /// Samples per second, per channel.
    pub sample_rate: u32,
}

/// Encoded audio files (MP3) for every sound event, ready for any audio backend to decode.
#[derive(Default)]
pub struct SoundBank {
    variants_by_event: HashMap<SoundEvent, Vec<SoundClip>>,
    music_by_track: HashMap<MusicTrack, MusicClips>,
}

impl SoundBank {
    pub fn variants(&self, event: SoundEvent) -> &[SoundClip] {
        self.variants_by_event.get(&event).map_or(&[], Vec::as_slice)
    }

    /// Every track the SWF has.
    pub fn all_music(&self) -> &HashMap<MusicTrack, MusicClips> {
        &self.music_by_track
    }

    pub fn events(&self) -> impl Iterator<Item = SoundEvent> {
        self.variants_by_event.keys().copied()
    }
}

/// SWF export names from the catalog that were not found as MP3 sounds.
#[derive(Debug)]
pub struct MissingSounds(pub Vec<&'static str>);

impl std::fmt::Display for MissingSounds {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "sounds missing from the SWF: {:?}", self.0)
    }
}

pub fn sound_bank_from_movie(movie: &swf::Swf) -> Result<SoundBank, MissingSounds> {
    let mut character_ids_by_name: HashMap<&[u8], swf::CharacterId> = HashMap::new();
    let mut sounds_by_character_id: HashMap<swf::CharacterId, &swf::Sound> = HashMap::new();
    for tag in &movie.tags {
        match tag {
            swf::Tag::ExportAssets(exported_assets) => {
                for asset in exported_assets {
                    character_ids_by_name.insert(asset.name.as_bytes(), asset.id);
                }
            }
            swf::Tag::DefineSound(sound) => {
                sounds_by_character_id.insert(sound.id, sound);
            }
            _ => {}
        }
    }

    let mut sound_bank = SoundBank {
        music_by_track: music::music_from_movie(&character_ids_by_name, &sounds_by_character_id),
        ..SoundBank::default()
    };
    let mut missing_names = Vec::new();
    for (event, export_names) in SOUND_CATALOG {
        for &export_name in export_names {
            let mp3_sound = character_ids_by_name
                .get(export_name.as_bytes())
                .and_then(|character_id| sounds_by_character_id.get(character_id))
                .filter(|sound| sound.format.compression == swf::AudioCompression::Mp3);
            match mp3_sound {
                Some(sound) => sound_bank.variants_by_event.entry(event).or_default().push(clip(sound)),
                None => missing_names.push(export_name),
            }
        }
    }
    if missing_names.is_empty() {
        Ok(sound_bank)
    } else {
        Err(MissingSounds(missing_names))
    }
}

/// Strips the SWF seek field so the remaining MP3 frames form a standalone MP3 file, keeping
/// the field's delay and the sound's length.
fn clip(sound: &swf::Sound) -> SoundClip {
    let seek = sound
        .data
        .first_chunk::<MP3_SEEK_SAMPLES_FIELD_SIZE>()
        .map_or(0, |&bytes| i16::from_le_bytes(bytes));
    SoundClip {
        mp3: Arc::from(sound.data.get(MP3_SEEK_SAMPLES_FIELD_SIZE..).unwrap_or_default()),
        leading_samples: usize::try_from(seek).unwrap_or(0),
        sample_count: sound.num_samples as usize,
        sample_rate: u32::from(sound.format.sample_rate),
    }
}

#[cfg(test)]
mod tests;

//! The game's music tracks from the SWF: each has an intro (`-0`) and a loop (`-1`); a new run opens
//! with the Basement's short opening (`m1-2`). A track the SWF lacks (the browser demo's) is left out.

use std::collections::HashMap;

use swf::CharacterId;

use super::SoundBank;
use super::{SoundClip, clip};
use crate::game::{MusicTrack, SoundEvent};

/// SWF export names of the intro and the loop of each track.
pub(super) const MUSIC_CATALOG: [(MusicTrack, &str, &str); 7] = [
    (MusicTrack::Basement, "m1-0.mp3", "m1-1.mp3"),
    (MusicTrack::Shop, "m13-0.mp3", "m13-1.mp3"),
    (MusicTrack::Library, "m30-0.mp3", "m30-1.mp3"),
    (MusicTrack::SecretRoom, "m15-0.mp3", "m15-1.mp3"),
    (MusicTrack::SuperSecretRoom, "m28-0.mp3", "m28-1.mp3"),
    (MusicTrack::DevilRoom, "m14-0.mp3", "m14-1.mp3"),
    (MusicTrack::Death, "m7-0.mp3", "m7-1.mp3"),
];
const NEW_RUN_OPENING: &str = "m1-2.mp3";

/// What a track plays: its intro once, then its loop for as long as it lasts.
#[derive(Clone, Debug)]
pub struct MusicClips {
    pub intro: SoundClip,
    pub looped: SoundClip,
    /// Only the Basement's: the opening before its intro when a run begins.
    pub opening: Option<SoundClip>,
}

pub(super) fn music_from_movie(
    character_ids_by_name: &HashMap<&[u8], CharacterId>,
    sounds_by_character_id: &HashMap<CharacterId, &swf::Sound>,
) -> HashMap<MusicTrack, MusicClips> {
    let find = |name: &str| {
        character_ids_by_name
            .get(name.as_bytes())
            .and_then(|character_id| sounds_by_character_id.get(character_id))
            .filter(|sound| sound.format.compression == swf::AudioCompression::Mp3)
            .map(|sound| clip(sound))
    };
    MUSIC_CATALOG
        .into_iter()
        .filter_map(|(track, intro, looped)| {
            let clips = MusicClips {
                intro: find(intro)?,
                looped: find(looped)?,
                opening: (track == MusicTrack::Basement).then(|| find(NEW_RUN_OPENING)).flatten(),
            };
            Some((track, clips))
        })
        .collect()
}

impl SoundBank {
    /// How long, in seconds, each jingle that takes the music down plays.
    pub fn jingle_lengths(&self) -> impl Iterator<Item = (SoundEvent, f32)> + '_ {
        self.variants_by_event.iter().filter_map(|(&event, clips)| {
            let clip = clips.first().filter(|_| event.ducks_music())?;
            Some((event, clip.sample_count as f32 / clip.sample_rate.max(1) as f32))
        })
    }
}

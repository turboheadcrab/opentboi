//! The music the game asks for. The original's sound director (`soundfra()`, Timeline 563–1030)
//! picks a track from where Isaac is: the floor's, or a special room's, or the death track. A track
//! plays its intro (`-0`) once and then its loop (`-1`); the director only changes track once the
//! one playing has faded out, which takes a few frames, and a track that follows a special room starts
//! on its loop, not its intro. The frontends play what [`MusicEvent`] says.

use std::collections::HashMap;

use crate::game::Game;
use crate::game::floor::plan::RoomRole;
use crate::game::original_units::Frames;
use crate::game::parameters::MusicParameters;
use crate::game::sound_event::SoundEvent;

mod ducking;
use ducking::MusicDucking;

/// What can play; each has an intro and a loop in the game's files.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MusicTrack {
    /// The Basement's (`m1`), the first chapter's.
    Basement,
    /// The shop's (`m13`).
    Shop,
    /// The library's (`m30`).
    Library,
    /// The secret room's (`m15`).
    SecretRoom,
    /// The super secret room's (`m28`).
    SuperSecretRoom,
    /// The devil room's (`m14`).
    DevilRoom,
    /// Isaac's death (`m7`).
    Death,
}

/// Where a track starts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MusicStart {
    /// A new run: the short opening (`m1-2`), then the intro and loop.
    NewRun,
    Intro,
    Loop,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MusicEvent {
    /// `volume` is the playback gain next to the sound effects'.
    Play {
        track: MusicTrack,
        start: MusicStart,
        volume: f32,
    },
    /// Fade what plays out over this many seconds.
    FadeOut { seconds: f32 },
    /// A room jingle holds the music down to this fraction of its volume (1: back to normal); it
    /// applies to every track until the next such event.
    Duck { factor: f32 },
}

#[derive(Clone, Debug)]
pub struct MusicDirector {
    parameters: MusicParameters,
    playing: Option<MusicTrack>,
    /// Frames of fading left (`fade`).
    fading: Option<u32>,
    /// `intro`: the next track starts on its intro.
    intro: bool,
    /// `introz`: the track after this fade is a return from a special room, so it skips its intro.
    skip_intro: bool,
    frames: Frames,
    events: Vec<MusicEvent>,
    ducking: MusicDucking,
    /// How long each jingle that ducks the music plays, known once the recordings are.
    jingle_lengths: HashMap<SoundEvent, Frames>,
}

impl MusicDirector {
    /// A new run begins with its opening.
    pub fn new(parameters: MusicParameters) -> Self {
        let mut director = Self {
            parameters,
            playing: None,
            fading: None,
            intro: false,
            skip_intro: false,
            frames: Frames::ZERO,
            events: Vec::new(),
            ducking: MusicDucking::new(parameters),
            jingle_lengths: HashMap::new(),
        };
        director.play(MusicTrack::Basement, MusicStart::NewRun);
        director
    }

    fn play(&mut self, track: MusicTrack, start: MusicStart) {
        self.playing = Some(track);
        self.events.push(MusicEvent::Play {
            track,
            start,
            volume: self.parameters.volume / self.parameters.effects_volume,
        });
    }

    /// How long a fade takes: 5 % less a frame until at most 5 % is left, and one more frame.
    fn fade_frames(&self) -> u32 {
        let steps = (self.parameters.volume - self.parameters.fade_floor) / self.parameters.fade_step;
        crate::whole_numbers::clamped_u32(steps.max(0.0).ceil()) + 1
    }

    pub fn set_jingle_lengths(&mut self, lengths: impl IntoIterator<Item = (SoundEvent, Frames)>) {
        self.jingle_lengths = lengths.into_iter().collect();
    }

    /// Sounds about to be played; a jingle among them takes the music down.
    pub fn hear(&mut self, sounds: &[SoundEvent]) {
        for sound in sounds {
            if let Some(&length) = self.jingle_lengths.get(sound) {
                self.ducking.start_jingle(length);
            }
        }
    }

    pub fn advance(&mut self, elapsed: Frames, wanted: MusicTrack) {
        if self.ducking.advance(elapsed) {
            self.events.push(MusicEvent::Duck {
                factor: self.ducking.factor(),
            });
        }
        let first = self.frames.whole() + 1;
        self.frames += elapsed;
        for _ in first..=self.frames.whole() {
            self.frame(wanted);
        }
    }

    fn frame(&mut self, wanted: MusicTrack) {
        if let Some(left) = self.fading {
            if left > 1 {
                self.fading = Some(left - 1);
            } else {
                self.fading = None;
                self.playing = None;
                if self.skip_intro {
                    self.skip_intro = false;
                } else {
                    self.intro = true;
                }
            }
            return;
        }
        match self.playing {
            None => {
                let start = if self.intro {
                    MusicStart::Intro
                } else {
                    MusicStart::Loop
                };
                self.intro = false;
                self.play(wanted, start);
            }
            Some(playing) if playing != wanted => {
                // Leaving a special room, the next track skips its intro.
                self.skip_intro = !matches!(playing, MusicTrack::Basement | MusicTrack::Death);
                let fade_frames = self.fade_frames();
                self.fading = Some(fade_frames);
                self.events.push(MusicEvent::FadeOut {
                    seconds: Frames::from_number(fade_frames).seconds(),
                });
            }
            Some(_) => {}
        }
    }

    pub fn drain(&mut self) -> impl Iterator<Item = MusicEvent> {
        self.events.drain(..)
    }
}

/// The track for a room's role.
fn track_for(role: RoomRole) -> MusicTrack {
    match role {
        RoomRole::Shop => MusicTrack::Shop,
        RoomRole::Library => MusicTrack::Library,
        RoomRole::Secret => MusicTrack::SecretRoom,
        RoomRole::SuperSecret => MusicTrack::SuperSecretRoom,
        RoomRole::Devil => MusicTrack::DevilRoom,
        RoomRole::Start
        | RoomRole::Normal
        | RoomRole::Boss
        | RoomRole::Treasure
        | RoomRole::Curse
        | RoomRole::Sacrifice
        | RoomRole::MiniBoss => MusicTrack::Basement,
    }
}

impl Game {
    pub(crate) fn update_music(&mut self, elapsed: Frames) {
        let wanted = if self.is_dying() {
            MusicTrack::Death
        } else {
            track_for(self.floor.current_role())
        };
        self.music.advance(elapsed, wanted);
    }

    /// How long, in seconds, the recordings of the jingles that take the music down are (see
    /// [`SoundEvent::ducks_music`]).
    pub fn set_jingle_lengths(&mut self, lengths: impl IntoIterator<Item = (SoundEvent, f32)>) {
        self.music.set_jingle_lengths(
            lengths
                .into_iter()
                .map(|(sound, seconds)| (sound, Frames::from_seconds(seconds))),
        );
    }

    /// What the music should do since the last call, oldest first.
    pub fn drain_music_events(&mut self) -> impl Iterator<Item = MusicEvent> {
        self.music.drain()
    }
}

#[cfg(test)]
mod tests;

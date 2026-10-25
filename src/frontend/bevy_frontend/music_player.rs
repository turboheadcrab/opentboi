//! Plays the game's music through Bevy's audio: a track's recordings played in turn, the loop for
//! good, faded out when the game says so.

use std::collections::{HashMap, VecDeque};
use std::time::Duration;

use bevy::audio::{AddAudioSource, AudioSinkPlayback, Decodable, Source, Volume};
use bevy::audio::{ChannelCount, SampleRate};
use bevy::prelude::*;

use super::GameSimulation;
use super::sound_player::TrimmedClip;
use crate::game::{MusicEvent, MusicStart, MusicTrack};
use crate::game_assets::{MusicClips, SoundBank};

/// What a room jingle leaves of the music's volume (1: all of it).
#[derive(Resource)]
struct MusicDuck(f32);

/// The game's tracks, to be played when it asks.
#[derive(Resource, Default)]
struct MusicLibrary {
    tracks: HashMap<MusicTrack, MusicClips>,
}

/// Registers the music; must be added after `DefaultPlugins`.
pub struct MusicPlayerPlugin {
    tracks: HashMap<MusicTrack, MusicClips>,
}

impl MusicPlayerPlugin {
    pub fn new(sound_bank: &SoundBank) -> Self {
        Self {
            tracks: sound_bank.all_music().clone(),
        }
    }
}

impl Plugin for MusicPlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_audio_source::<MusicSequence>()
            .insert_resource(MusicDuck(1.0))
            .insert_resource(MusicLibrary {
                tracks: self.tracks.clone(),
            })
            .add_systems(
                Update,
                (play_music_events, fade_game_music).after(super::step_simulation),
            );
    }
}

/// A track as it plays: some recordings once (the opening, the intro), then the loop for good.
#[derive(Asset, TypePath, Clone)]
struct MusicSequence {
    once: Vec<TrimmedClip>,
    looped: TrimmedClip,
}

struct SequenceDecoder {
    current: Box<dyn Source + Send>,
    waiting: VecDeque<TrimmedClip>,
    looped: TrimmedClip,
}

impl Iterator for SequenceDecoder {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        loop {
            if let Some(sample) = self.current.next() {
                return Some(sample);
            }
            let next = self.waiting.pop_front().unwrap_or_else(|| self.looped.clone());
            self.current = next.decoder();
        }
    }
}

impl Source for SequenceDecoder {
    fn current_span_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> ChannelCount {
        self.current.channels()
    }

    fn sample_rate(&self) -> SampleRate {
        self.current.sample_rate()
    }

    fn total_duration(&self) -> Option<Duration> {
        None
    }
}

impl Decodable for MusicSequence {
    type Decoder = SequenceDecoder;

    fn decoder(&self) -> Self::Decoder {
        let mut waiting: VecDeque<TrimmedClip> = self.once.iter().cloned().collect();
        let first = waiting.pop_front().unwrap_or_else(|| self.looped.clone());
        SequenceDecoder {
            current: first.decoder(),
            waiting,
            looped: self.looped.clone(),
        }
    }
}

/// The entity playing the game's music, and how loud it is.
#[derive(Component)]
struct GameMusic {
    gain: f32,
    /// How much of its gain a fade takes away each second.
    fade_per_second: Option<f32>,
}

/// Says when the music goes down under a jingle and when it is back.
fn report_duck(duck: &MusicDuck, factor: f32) {
    if (duck.0 >= 1.0) != (factor >= 1.0) {
        println!(
            "Music {} a jingle",
            if factor < 1.0 {
                "goes down under"
            } else {
                "is back after"
            }
        );
    }
}

fn play_music_events(
    mut commands: Commands,
    mut simulation: ResMut<GameSimulation>,
    library: Res<MusicLibrary>,
    mut duck: ResMut<MusicDuck>,
    mut sequences: ResMut<Assets<MusicSequence>>,
    mut playing: Query<(Entity, &mut GameMusic, Option<&mut AudioSink>)>,
) {
    for event in simulation.0.drain_music_events() {
        match event {
            MusicEvent::Play { track, start, volume } => {
                for (entity, ..) in &playing {
                    commands.entity(entity).despawn();
                }
                let Some(clips) = library.tracks.get(&track) else {
                    continue;
                };
                let once = match start {
                    MusicStart::NewRun => vec![clips.opening.as_ref(), Some(&clips.intro)],
                    MusicStart::Intro => vec![Some(&clips.intro)],
                    MusicStart::Loop => Vec::new(),
                };
                let sequence = MusicSequence {
                    once: once.into_iter().flatten().map(TrimmedClip::new).collect(),
                    looped: TrimmedClip::new(&clips.looped),
                };
                commands.spawn((
                    AudioPlayer(sequences.add(sequence)),
                    PlaybackSettings::ONCE.with_volume(Volume::Linear(volume * duck.0)),
                    GameMusic {
                        gain: volume,
                        fade_per_second: None,
                    },
                ));
            }
            MusicEvent::Duck { factor } => {
                report_duck(&duck, factor);
                duck.0 = factor;
                for (_, music, sink) in &mut playing {
                    if let (Some(mut sink), None) = (sink, music.fade_per_second) {
                        sink.set_volume(Volume::Linear(music.gain * factor));
                    }
                }
            }
            MusicEvent::FadeOut { seconds } => {
                for (_, mut music, _) in &mut playing {
                    music.fade_per_second = Some(music.gain / seconds);
                }
            }
        }
    }
}

fn fade_game_music(
    mut commands: Commands,
    time: Res<Time>,
    duck: Res<MusicDuck>,
    mut playing: Query<(Entity, &mut GameMusic, Option<&mut AudioSink>)>,
) {
    for (entity, mut music, sink) in &mut playing {
        let Some(per_second) = music.fade_per_second else {
            continue;
        };
        music.gain -= per_second * time.delta_secs();
        if music.gain <= 0.0 {
            commands.entity(entity).despawn();
        } else if let Some(mut sink) = sink {
            sink.set_volume(Volume::Linear(music.gain * duck.0));
        }
    }
}

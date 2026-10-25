//! Plays the game's sound events and looping sounds through Bevy's audio.

use std::collections::HashMap;
use std::time::Duration;

use bevy::audio::{AddAudioSource, AudioSinkPlayback, AudioSource, Decodable, Source, Volume};
use bevy::prelude::*;

use super::GameSimulation;
use crate::frontend::sound_variants::VariantChooser;
use crate::game::{LoopingSound, SoundEvent};
use crate::game_assets::{SoundBank, SoundClip};

/// A recording without the MP3 encoder's delay and padding, so loops do not stutter.
#[derive(Asset, TypePath, Clone)]
pub(super) struct TrimmedClip {
    source: AudioSource,
    skip: Duration,
    length: Duration,
}

impl TrimmedClip {
    pub(super) fn new(clip: &SoundClip) -> Self {
        let seconds = |samples: usize| Duration::from_secs_f64(samples as f64 / f64::from(clip.sample_rate.max(1)));
        Self {
            source: AudioSource {
                bytes: clip.mp3.clone(),
            },
            skip: seconds(clip.leading_samples),
            length: seconds(clip.sample_count),
        }
    }
}

impl Decodable for TrimmedClip {
    type Decoder = Box<dyn Source + Send>;

    fn decoder(&self) -> Self::Decoder {
        Box::new(
            self.source
                .decoder()
                .skip_duration(self.skip)
                .take_duration(self.length),
        )
    }
}

/// Decoded-on-demand audio handles per event, and the random choice among them.
#[derive(Resource, Default)]
struct SoundLibrary {
    variants_by_event: HashMap<SoundEvent, Vec<Handle<TrimmedClip>>>,
    variants: VariantChooser,
}

impl SoundLibrary {
    /// One of the event's recordings, chosen at random like the original.
    fn next_variant(&mut self, event: SoundEvent) -> Option<Handle<TrimmedClip>> {
        let variants = self
            .variants_by_event
            .get(&event)
            .filter(|variants| !variants.is_empty())?;
        Some(variants[self.variants.pick(variants.len())].clone())
    }
}

/// Registers every clip from the [`SoundBank`] with Bevy; must be added after `DefaultPlugins`.
pub struct SoundPlayerPlugin {
    clips_by_event: Vec<(SoundEvent, Vec<SoundClip>)>,
}

impl SoundPlayerPlugin {
    pub fn new(sound_bank: &SoundBank) -> Self {
        let clips_by_event = sound_bank
            .events()
            .map(|event| (event, sound_bank.variants(event).to_vec()))
            .collect();
        Self { clips_by_event }
    }
}

impl Plugin for SoundPlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_audio_source::<TrimmedClip>();
        let mut clips = app.world_mut().resource_mut::<Assets<TrimmedClip>>();
        let mut library = SoundLibrary::default();
        for (event, variants) in &self.clips_by_event {
            let handles = variants.iter().map(|clip| clips.add(TrimmedClip::new(clip))).collect();
            library.variants_by_event.insert(*event, handles);
        }
        app.insert_resource(library).add_systems(
            Update,
            (play_sound_events, update_looping_sounds).after(super::step_simulation),
        );
    }
}

fn play_sound_events(
    mut commands: Commands,
    mut simulation: ResMut<GameSimulation>,
    mut library: ResMut<SoundLibrary>,
) {
    for event in simulation.0.drain_sound_events() {
        if let Some(clip) = library.next_variant(event) {
            commands.spawn((AudioPlayer(clip), PlaybackSettings::DESPAWN));
        }
    }
}

/// An entity playing one of the game's looping sounds.
#[derive(Component)]
struct Looping(SoundEvent);

/// Keeps exactly the game's looping sounds playing, each at its volume.
fn update_looping_sounds(
    mut commands: Commands,
    simulation: Res<GameSimulation>,
    mut library: ResMut<SoundLibrary>,
    mut playing: Query<(Entity, &Looping, Option<&mut AudioSink>)>,
) {
    let wanted: Vec<LoopingSound> = simulation.0.looping_sounds().collect();
    for (entity, looping, sink) in &mut playing {
        match wanted.iter().find(|wanted| wanted.sound == looping.0) {
            Some(wanted) => {
                if let Some(mut sink) = sink {
                    sink.set_volume(Volume::Linear(wanted.volume));
                }
            }
            None => commands.entity(entity).despawn(),
        }
    }
    for looping in wanted {
        if playing.iter().any(|(_, playing, _)| playing.0 == looping.sound) {
            continue;
        }
        if let Some(clip) = library.next_variant(looping.sound) {
            commands.spawn((
                AudioPlayer(clip),
                PlaybackSettings::LOOP.with_volume(Volume::Linear(looping.volume)),
                Looping(looping.sound),
            ));
        }
    }
}

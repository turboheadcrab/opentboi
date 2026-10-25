//! Plays the game's sound events, its looping sounds and the intro music through SDL audio
//! streams: each play gets its own stream bound to the device, and SDL mixes them. A looping
//! sound's stream is topped up with another copy of the clip whenever less than one is queued.

use std::collections::HashMap;
use std::time::Instant;

use sdl3::audio::{AudioDevice, AudioSpec, AudioStreamOwner};
use sdl3::{AudioSubsystem, Sdl};

use crate::frontend::sdl3_frontend::audio::decoding::{DecodedClip, decode};
use crate::frontend::sound_variants::VariantChooser;
use crate::game::{LoopingSound, MusicTrack, SoundEvent};
use crate::game_assets::{MusicClips, SoundBank};
use music::GameMusic;

/// How long the intro music takes to fade out once the game starts.
const MUSIC_FADE_SECONDS: f32 = 0.6;
const FULL_GAIN: f32 = 1.0;

pub struct SoundPlayer {
    audio: AudioSubsystem,
    device: AudioDevice,
    clips_by_event: HashMap<SoundEvent, Vec<DecodedClip>>,
    variants: VariantChooser,
    /// One stream per sound still playing; dropping a stream unbinds it from the device.
    playing: Vec<AudioStreamOwner>,
    loops: HashMap<SoundEvent, AudioStreamOwner>,
    music: Option<AudioStreamOwner>,
    music_fading: bool,
    music_clips: HashMap<MusicTrack, MusicClips>,
    game_music: Option<GameMusic>,
    /// What a room jingle leaves of the music's volume (1: all of it).
    music_duck: f32,
}

impl SoundPlayer {
    /// `None` (and a printed reason) when there is no audio device; the game runs silently.
    pub fn open(context: &Sdl) -> Option<Self> {
        let opened = context.audio().and_then(|audio| {
            let device = audio.open_playback_device(&AudioSpec::default())?;
            Ok((audio, device))
        });
        let (audio, device) = opened
            .inspect_err(|error| println!("Playing without sound: {error}"))
            .ok()?;
        Some(Self {
            audio,
            device,
            clips_by_event: HashMap::new(),
            variants: VariantChooser::default(),
            playing: Vec::new(),
            loops: HashMap::new(),
            music: None,
            music_fading: false,
            music_clips: HashMap::new(),
            game_music: None,
            music_duck: FULL_GAIN,
        })
    }

    /// Decodes the game's sound effects (a few milliseconds for all of them).
    pub fn set_sound_bank(&mut self, sound_bank: &SoundBank) {
        let decoding_started = Instant::now();
        self.clips_by_event = sound_bank
            .events()
            .map(|event| {
                let clips = sound_bank.variants(event).iter().filter_map(decode).collect();
                (event, clips)
            })
            .collect();
        println!("Decoded sounds in {} ms", decoding_started.elapsed().as_millis());
        self.music_clips.clone_from(sound_bank.all_music());
    }

    pub fn play_all(&mut self, events: impl Iterator<Item = SoundEvent>) {
        self.playing
            .retain(|stream| stream.queued_bytes().is_ok_and(|queued_bytes| queued_bytes > 0));
        for event in events {
            if let Err(error) = self.play(event) {
                println!("Could not play {event:?}: {error}");
            }
        }
    }

    /// Keeps exactly the `wanted` loops playing, each at its volume.
    pub fn update_loops(&mut self, wanted: impl Iterator<Item = LoopingSound>) {
        let wanted: Vec<LoopingSound> = wanted.collect();
        self.loops
            .retain(|sound, _| wanted.iter().any(|looping| looping.sound == *sound));
        for looping in wanted {
            if let Err(error) = self.keep_looping(looping) {
                println!("Could not loop {:?}: {error}", looping.sound);
                self.loops.remove(&looping.sound);
            }
        }
    }

    fn keep_looping(&mut self, looping: LoopingSound) -> Result<(), sdl3::Error> {
        let Some(clip) = self.clips_by_event.get(&looping.sound).and_then(|clips| clips.first()) else {
            return Ok(());
        };
        let clip_bytes = clip.samples.len() * size_of::<f32>();
        if let Some(stream) = self.loops.get(&looping.sound) {
            if stream.queued_bytes()? < i32::try_from(clip_bytes).unwrap_or(i32::MAX) {
                stream.put_data_f32(&clip.samples)?;
            }
            stream.set_gain(looping.volume)?;
            return Ok(());
        }
        let stream = self.start_stream(clip)?;
        stream.set_gain(looping.volume)?;
        self.loops.insert(looping.sound, stream);
        Ok(())
    }

    /// Plays one of the event's recordings, chosen at random like the original.
    fn play(&mut self, event: SoundEvent) -> Result<(), sdl3::Error> {
        let Some(variants) = self.clips_by_event.get(&event).filter(|clips| !clips.is_empty()) else {
            return Ok(());
        };
        let clip = &variants[self.variants.pick(variants.len())];
        let stream = self.start_stream(clip)?;
        self.playing.push(stream);
        Ok(())
    }

    fn start_stream(&self, clip: &DecodedClip) -> Result<AudioStreamOwner, sdl3::Error> {
        let stream = self.audio.new_playback_stream(&clip.spec, None)?;
        self.device.bind_stream(&stream)?;
        stream.put_data_f32(&clip.samples)?;
        stream.flush()?;
        Ok(stream)
    }
}

mod music;

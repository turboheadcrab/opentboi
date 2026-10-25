//! The music: the intro's track fading out when the game starts, and the game's own tracks (an
//! intro and a loop, fading out and ducking under room jingles when the game says so).

use std::time::Instant;

use sdl3::audio::AudioStreamOwner;

use super::{FULL_GAIN, MUSIC_FADE_SECONDS, SoundPlayer};
use crate::frontend::sdl3_frontend::audio::decoding::{DecodedClip, decode};
use crate::game::{MusicEvent, MusicStart, MusicTrack};

/// The track playing during the game: its stream, one more copy of its loop to queue when the
/// last runs low, and how fast a fade takes it down.
pub(super) struct GameMusic {
    stream: AudioStreamOwner,
    looped: DecodedClip,
    gain: f32,
    fade_per_second: Option<f32>,
}

impl SoundPlayer {
    pub fn start_music(&mut self, music: &DecodedClip) {
        match self.start_stream(music) {
            Ok(stream) => self.music = Some(stream),
            Err(error) => println!("Could not play the music: {error}"),
        }
    }

    pub fn fade_out_music(&mut self) {
        self.music_fading = true;
    }

    /// Advances a running fade; the stream is dropped once silent.
    pub fn update_music(&mut self, delta_seconds: f32) {
        let Some(music) = &self.music else { return };
        if !self.music_fading {
            return;
        }
        let gain = music.get_gain().unwrap_or(FULL_GAIN) - delta_seconds / MUSIC_FADE_SECONDS;
        if gain <= 0.0 {
            self.music = None;
        } else if let Err(error) = music.set_gain(gain) {
            println!("Could not fade the music: {error}");
            self.music = None;
        }
    }

    /// Does what the game says about its music.
    pub fn play_music_events(&mut self, events: impl Iterator<Item = MusicEvent>) {
        for event in events {
            match event {
                MusicEvent::Play { track, start, volume } => self.start_game_music(track, start, volume),
                MusicEvent::Duck { factor } => {
                    if (self.music_duck >= FULL_GAIN) != (factor >= FULL_GAIN) {
                        println!(
                            "Music {} a jingle",
                            if factor < FULL_GAIN {
                                "goes down under"
                            } else {
                                "is back after"
                            }
                        );
                    }
                    self.music_duck = factor;
                    if let Some(music) = &self.game_music
                        && music.fade_per_second.is_none()
                        && let Err(error) = music.stream.set_gain(music.gain * factor)
                    {
                        println!("Could not lower the music: {error}");
                    }
                }
                MusicEvent::FadeOut { seconds } => {
                    if let Some(music) = &mut self.game_music {
                        music.fade_per_second = Some(music.gain / seconds);
                    }
                }
            }
        }
    }

    fn start_game_music(&mut self, track: MusicTrack, start: MusicStart, gain: f32) {
        let Some(clips) = self.music_clips.get(&track) else {
            return;
        };
        let started = Instant::now();
        let lead_in = match start {
            MusicStart::NewRun => [clips.opening.as_ref(), Some(&clips.intro)],
            MusicStart::Intro => [Some(&clips.intro), None],
            MusicStart::Loop => [None, None],
        };
        let Some(looped) = decode(&clips.looped) else { return };
        let result = self.audio.new_playback_stream(&looped.spec, None).and_then(|stream| {
            self.device.bind_stream(&stream)?;
            for clip in lead_in.into_iter().flatten().filter_map(decode) {
                stream.put_data_f32(&clip.samples)?;
            }
            stream.put_data_f32(&looped.samples)?;
            stream.set_gain(gain * self.music_duck)?;
            Ok(stream)
        });
        match result {
            Ok(stream) => {
                println!("Started {track:?} in {} ms", started.elapsed().as_millis());
                self.game_music = Some(GameMusic {
                    stream,
                    looped,
                    gain,
                    fade_per_second: None,
                });
            }
            Err(error) => println!("Could not play {track:?}: {error}"),
        }
    }

    /// Keeps the loop queued and takes a fading track down.
    pub fn update_game_music(&mut self, delta_seconds: f32) {
        let Some(music) = &mut self.game_music else { return };
        let loop_bytes = i32::try_from(music.looped.samples.len() * size_of::<f32>()).unwrap_or(i32::MAX);
        let outcome = match music.fade_per_second {
            Some(per_second) => {
                music.gain -= per_second * delta_seconds;
                if music.gain <= 0.0 {
                    self.game_music = None;
                    return;
                }
                music.stream.set_gain(music.gain * self.music_duck)
            }
            None => music.stream.queued_bytes().and_then(|queued| {
                if queued < loop_bytes {
                    music.stream.put_data_f32(&music.looped.samples)
                } else {
                    Ok(())
                }
            }),
        };
        if let Err(error) = outcome {
            println!("Could not keep the music going: {error}");
            self.game_music = None;
        }
    }
}

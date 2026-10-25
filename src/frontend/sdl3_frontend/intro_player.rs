//! Plays the original's intro while the game loads. A thread renders frames from the SWF just
//! ahead of a 30 fps clock (skipping ahead if it falls behind, so the music stays in sync); the
//! main loop uploads the newest due frame into one streaming texture and draws it letterboxed.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU16, Ordering};
use std::sync::mpsc::{Receiver, SyncSender, TryRecvError};
use std::time::Instant;

use sdl3::pixels::PixelFormat;
use sdl3::render::{FRect, Texture, TextureCreator, WindowCanvas};
use sdl3::video::WindowContext;

use crate::frontend::sdl3_frontend::audio::decoding::{DecodedClip, decode_mp3};
use crate::frontend::sdl3_frontend::audio::sound_player::SoundPlayer;
use crate::game_assets::{INTRO_FRAME_COUNT, RgbaImage, STAGE_HEIGHT, STAGE_WIDTH, TextureRenderer};

const FRAMES_PER_SECOND: f32 = 30.0;
/// Frames rendered ahead of the clock; more only costs memory.
const FRAMES_BUFFERED: usize = 2;

enum IntroMessage {
    Music(DecodedClip),
    Frame(u16, RgbaImage),
}

pub struct IntroPlayer<'creator> {
    messages: Receiver<IntroMessage>,
    /// The frame the clock needs next; the render thread jumps here when it is behind.
    wanted_frame: Arc<AtomicU16>,
    stop: Arc<AtomicBool>,
    texture: Option<Texture<'creator>>,
    music: Option<DecodedClip>,
    clock_started: Option<Instant>,
    skipped: bool,
    /// The render thread ended without ever delivering a frame (the art is missing).
    unavailable: bool,
}

impl<'creator> IntroPlayer<'creator> {
    /// Starts rendering frames `output_width` pixels wide, plus decoding `music`.
    pub fn start(renderer: &'static TextureRenderer, output_width: u32, music: Option<Arc<[u8]>>) -> Self {
        let (sender, messages) = std::sync::mpsc::sync_channel(FRAMES_BUFFERED);
        let wanted_frame = Arc::new(AtomicU16::new(0));
        let stop = Arc::new(AtomicBool::new(false));
        let (thread_wanted, thread_stop) = (Arc::clone(&wanted_frame), Arc::clone(&stop));
        std::thread::spawn(move || {
            render_frames(
                renderer,
                output_width,
                music.as_ref(),
                &sender,
                &thread_wanted,
                &thread_stop,
            );
        });
        Self {
            messages,
            wanted_frame,
            stop,
            texture: None,
            music: None,
            clock_started: None,
            skipped: false,
            unavailable: false,
        }
    }

    pub fn skip(&mut self) {
        self.skipped = true;
        self.stop.store(true, Ordering::Relaxed);
    }

    /// Over once skipped or played to the end; the last frame stays up until the game is ready.
    pub fn is_over(&self) -> bool {
        self.skipped || self.unavailable || self.due_frame() >= INTRO_FRAME_COUNT
    }

    fn due_frame(&self) -> u16 {
        self.clock_started.map_or(0, |started| {
            crate::whole_numbers::clamped_u16(started.elapsed().as_secs_f32() * FRAMES_PER_SECOND)
        })
    }

    /// Takes in rendered frames and shows the one the clock is at. The clock and the music start
    /// together with the first frame.
    pub fn update(&mut self, creator: &'creator TextureCreator<WindowContext>, sounds: Option<&mut SoundPlayer>) {
        if self.is_over() {
            return;
        }
        let due = self.due_frame();
        let mut newest_due_frame = None;
        loop {
            match self.messages.try_recv() {
                Ok(IntroMessage::Music(music)) => self.music = Some(music),
                Ok(IntroMessage::Frame(index, image)) => {
                    let reached_clock = index >= due;
                    newest_due_frame = Some(image);
                    if reached_clock {
                        break;
                    }
                }
                Err(TryRecvError::Disconnected) => {
                    self.unavailable = self.clock_started.is_none() && newest_due_frame.is_none();
                    break;
                }
                Err(TryRecvError::Empty) => break,
            }
        }
        let Some(image) = newest_due_frame else { return };
        self.show(creator, &image);
        if self.clock_started.is_none() {
            self.clock_started = Some(Instant::now());
            if let (Some(sounds), Some(music)) = (sounds, &self.music) {
                sounds.start_music(music);
            }
        }
        self.wanted_frame.store(self.due_frame() + 1, Ordering::Relaxed);
    }

    fn show(&mut self, creator: &'creator TextureCreator<WindowContext>, image: &RgbaImage) {
        let fits = self
            .texture
            .as_ref()
            .is_some_and(|texture| texture.width() == image.width && texture.height() == image.height);
        if !fits {
            self.texture = creator
                .create_texture_streaming(PixelFormat::RGBA32, image.width, image.height)
                .inspect_err(|error| println!("Cannot show the intro: {error}"))
                .ok();
        }
        if let Some(texture) = &mut self.texture
            && let Err(error) = texture.update(None, &image.pixels, image.width as usize * RgbaImage::BYTES_PER_PIXEL)
        {
            println!("Cannot show an intro frame: {error}");
        }
    }

    /// Draws the current frame, the stage fitted into the window with black bars.
    pub fn draw(&self, canvas: &mut WindowCanvas) -> Result<(), String> {
        let Some(texture) = &self.texture else { return Ok(()) };
        let (output_width, output_height) = canvas.output_size().map_err(|error| error.to_string())?;
        let rectangle = fitted_stage(output_width, output_height);
        canvas.copy(texture, None, rectangle).map_err(|error| error.to_string())
    }
}

impl Drop for IntroPlayer<'_> {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
    }
}

/// The intro's 800×600 stage scaled to fit `output` pixels, centered.
pub fn fitted_stage(output_width: u32, output_height: u32) -> FRect {
    let scale = (output_width as f32 / STAGE_WIDTH).min(output_height as f32 / STAGE_HEIGHT);
    let (width, height) = (STAGE_WIDTH * scale, STAGE_HEIGHT * scale);
    FRect::new(
        (output_width as f32 - width) / 2.0,
        (output_height as f32 - height) / 2.0,
        width,
        height,
    )
}

/// Intro frames are vector art rendered on the CPU while the game loads; at the original stage's
/// width (800 px, ~20 ms with 2×2 supersampling) the 30 fps intro keeps up, and the GPU scales it.
const MAXIMUM_INTRO_RENDER_WIDTH: u32 = 800;

/// How wide to render the intro for a window of `output` pixels.
pub fn intro_render_width(output_width: u32, output_height: u32) -> u32 {
    let fitted_width = crate::whole_numbers::clamped_u32(fitted_stage(output_width, output_height).w.round());
    fitted_width.min(MAXIMUM_INTRO_RENDER_WIDTH)
}

fn render_frames(
    renderer: &TextureRenderer,
    output_width: u32,
    music: Option<&Arc<[u8]>>,
    sender: &SyncSender<IntroMessage>,
    wanted_frame: &AtomicU16,
    stop: &AtomicBool,
) {
    if let Some(music) = music.and_then(decode_mp3)
        && sender.send(IntroMessage::Music(music)).is_err()
    {
        return;
    }
    let mut index = 0;
    while index < INTRO_FRAME_COUNT && !stop.load(Ordering::Relaxed) {
        index = index.max(wanted_frame.load(Ordering::Relaxed));
        let frame = match renderer.render_intro_frame(index, output_width) {
            Ok(image) => image,
            Err(render_error) => {
                println!("Cannot render the intro: {render_error}");
                return;
            }
        };
        if sender.send(IntroMessage::Frame(index, frame)).is_err() {
            return;
        }
        index += 1;
    }
}

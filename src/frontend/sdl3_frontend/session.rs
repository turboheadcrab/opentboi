//! One run of the SDL3 frontend: takes in what the loader delivers, plays the intro meanwhile,
//! then runs the game once both are done.

use std::sync::Arc;
use std::sync::mpsc::{Receiver, TryRecvError};
use std::time::{Duration, Instant};

use sdl3::render::{TextureCreator, WindowCanvas};
use sdl3::video::WindowContext;
use sdl3::{EventPump, Sdl};

use crate::frontend::sdl3_frontend::audio::sound_player::SoundPlayer;
use crate::frontend::sdl3_frontend::device_input::{DeviceInput, WindowRequests};
use crate::frontend::sdl3_frontend::graphics::rerender_indicator;
use crate::frontend::sdl3_frontend::graphics::screenshot::ScreenshotRequest;
use crate::frontend::sdl3_frontend::graphics::sprite_renderer::{self, ViewMapping};
use crate::frontend::sdl3_frontend::graphics::texture_density::TextureDensity;
use crate::frontend::sdl3_frontend::graphics::texture_library::TextureLibrary;
use crate::frontend::sdl3_frontend::graphics::vector_painter::VectorPainter;
use crate::frontend::sdl3_frontend::intro_player::IntroPlayer;
use crate::frontend::sdl3_frontend::window::pixel_size::PixelSizeKeeper;
use crate::frontend::sdl3_frontend::window::splash_window::SplashWindow;
use crate::frontend::{FrameStatistics, FrontendLaunch};
use crate::game::{DrawItem, Game, GameParameters, GameSettings};
use crate::game_assets::TextureRenderer;
use crate::loading::{LoadEvent, LoadedGame};

const FRONTEND_NAME: &str = "sdl3";
/// Longest step the game takes in one update, so a stall (window drag, breakpoint) does not
/// teleport anything; the same 250 ms cap Bevy's `Time` applies.
const MAXIMUM_STEP_SECONDS: f32 = 0.25;
/// How often the splash wait checks for events and loaded parts.
const SPLASH_POLL_INTERVAL: Duration = Duration::from_millis(10);

pub struct Session<'creator> {
    parameters: GameParameters,
    play_intro: bool,
    seed: u32,
    game_settings: GameSettings,
    screenshot: Option<ScreenshotRequest>,
    exit_after: Option<Duration>,
    process_started: Instant,
    creator: &'creator TextureCreator<WindowContext>,
    textures: TextureLibrary<'creator>,
    vectors: VectorPainter<'creator>,
    density: TextureDensity,
    sounds: Option<SoundPlayer>,
    devices: DeviceInput,
    event_pump: EventPump,
    statistics: FrameStatistics,
    pixel_size: PixelSizeKeeper,
    load_events: Receiver<LoadEvent>,
    intro: Option<IntroPlayer<'creator>>,
    /// The parsed art and music the intro needs, until it starts.
    intro_art: Option<(&'static TextureRenderer, Option<Arc<[u8]>>)>,
    /// Set once loading has finished.
    loaded: Option<LoadedGame>,
    game: Option<Game>,
    draw_items: Vec<DrawItem>,
    upcoming_textures: Vec<crate::game::TextureId>,
}

impl<'creator> Session<'creator> {
    pub fn new(
        parameters: &GameParameters,
        launch: &FrontendLaunch,
        creator: &'creator TextureCreator<WindowContext>,
        sounds: Option<SoundPlayer>,
        event_pump: EventPump,
        load_events: Receiver<LoadEvent>,
        pixels_per_world_unit: f32,
    ) -> Self {
        Self {
            parameters: *parameters,
            play_intro: launch.play_intro,
            seed: launch.seed,
            game_settings: launch.game_settings,
            screenshot: launch.screenshot_path.clone().map(ScreenshotRequest::new),
            exit_after: launch.exit_after,
            process_started: launch.process_started,
            creator,
            textures: TextureLibrary::new(creator, pixels_per_world_unit),
            vectors: VectorPainter::new(creator),
            density: TextureDensity::new(pixels_per_world_unit),
            sounds,
            devices: DeviceInput::default(),
            event_pump,
            statistics: FrameStatistics::new(FRONTEND_NAME, launch.process_started),
            pixel_size: PixelSizeKeeper::new(launch.display.window_size),
            load_events,
            intro: None,
            intro_art: None,
            loaded: None,
            game: None,
            draw_items: Vec::new(),
            upcoming_textures: Vec::new(),
        }
    }

    pub fn report(&self) -> String {
        self.statistics.report()
    }

    fn should_stop(&self, requests: &WindowRequests) -> bool {
        requests.quit
            || self
                .exit_after
                .is_some_and(|duration| self.statistics.has_run_for(duration))
    }

    /// Loads behind the splash: takes in textures, starts gamepads, and returns once the game is
    /// ready and the splash has been up long enough. `false` means the player quit meanwhile.
    pub fn wait_behind_splash(&mut self, context: &Sdl, splash: &SplashWindow) -> Result<bool, String> {
        self.devices.enable_gamepads(context);
        loop {
            let requests = self.devices.handle_events(&mut self.event_pump);
            if self.should_stop(&requests) {
                return Ok(false);
            }
            self.receive_textures_and_sounds()?;
            if self.loaded.is_some() && splash.has_shown_long_enough() {
                println!("Splash closed after {} ms", self.process_started.elapsed().as_millis());
                return Ok(true);
            }
            std::thread::sleep(SPLASH_POLL_INTERVAL);
        }
    }

    pub fn run_frames(&mut self, context: &Sdl, canvas: &mut WindowCanvas) -> Result<(), String> {
        let mut previous_frame = Instant::now();
        loop {
            let requests = self.devices.handle_events(&mut self.event_pump);
            if self.should_stop(&requests) {
                return Ok(());
            }
            self.handle_window_requests(&requests, canvas)?;
            self.receive_textures_and_sounds()?;
            if self.loaded.is_some() {
                self.density.follow_window(canvas, &mut self.textures);
            }
            self.start_intro_when_ready(canvas)?;
            let now = Instant::now();
            let delta_seconds = (now - previous_frame).as_secs_f32().min(MAXIMUM_STEP_SECONDS);
            previous_frame = now;
            if let Some(sounds) = &mut self.sounds {
                sounds.update_music(delta_seconds);
                sounds.update_game_music(delta_seconds);
            }
            if self.game.is_none() {
                self.advance_loading_screen(requests.skip_intro);
            }
            self.draw_frame(canvas, delta_seconds)?;
            self.devices.apply_cursor(context);
            if let Some(screenshot) = &mut self.screenshot {
                screenshot.capture_if_due(canvas, self.game.is_some());
            }
            canvas.present();
            self.statistics.record_frame();
            // SDL's joystick layer takes ~250 ms to start on Linux; doing it after the first
            // frame shows the window sooner, and the loader keeps working meanwhile.
            if !self.devices.gamepads_enabled() {
                self.devices.enable_gamepads(context);
            }
        }
    }

    /// Takes in what the loader sent; an error once loading broke off without finishing.
    fn receive_textures_and_sounds(&mut self) -> Result<(), String> {
        loop {
            match self.load_events.try_recv() {
                Ok(LoadEvent::ArtReady { renderer, intro_music }) => {
                    self.textures.set_renderer(renderer);
                    self.vectors.set_renderer(renderer);
                    if self.play_intro {
                        self.intro_art = Some((renderer, intro_music));
                    }
                }
                Ok(LoadEvent::Texture(texture_id, image)) => self.textures.insert(texture_id, &image),
                Ok(LoadEvent::Finished(loaded)) => {
                    if let Some(sounds) = &mut self.sounds {
                        sounds.set_sound_bank(&loaded.sound_bank);
                    }
                    self.loaded = Some(*loaded);
                }
                Ok(LoadEvent::Failed(reason)) => return Err(reason),
                Err(TryRecvError::Disconnected) if self.loaded.is_none() => {
                    return Err("Loading the game stopped before it finished.".to_string());
                }
                Err(TryRecvError::Empty | TryRecvError::Disconnected) => return Ok(()),
            }
        }
    }

    fn draw_frame(&mut self, canvas: &mut WindowCanvas, delta_seconds: f32) -> Result<(), String> {
        let Some(game) = &mut self.game else {
            sprite_renderer::clear(canvas);
            return self.intro.as_ref().map_or(Ok(()), |intro| intro.draw(canvas));
        };
        let view = ViewMapping::for_canvas(canvas)?;
        let input = self.devices.current_input(&self.event_pump, &view);
        let rerender_progress = self.density.rerender_progress();
        if rerender_progress.is_none() {
            game.update(&input, delta_seconds);
        }
        if let Some(sounds) = &mut self.sounds {
            sounds.play_all(game.drain_sound_events());
            sounds.play_music_events(game.drain_music_events());
            sounds.update_loops(game.looping_sounds());
        } else {
            game.drain_sound_events().for_each(drop);
            game.drain_music_events().for_each(drop);
        }
        if let Ok((output_width, output_height)) = canvas.output_size() {
            game.set_output_size(glam::Vec2::new(output_width as f32, output_height as f32));
        }
        self.draw_items.clear();
        game.fill_draw_items(&mut self.draw_items);
        sprite_renderer::draw(
            canvas,
            &mut self.textures,
            &mut self.vectors,
            &mut self.draw_items,
            &view,
        )?;
        self.upcoming_textures.clear();
        game.upcoming_textures(&mut self.upcoming_textures);
        let vectors = &mut self.vectors;
        self.upcoming_textures
            .retain(|&texture_id| !vectors.can_draw(texture_id));
        self.textures.prefetch(&self.upcoming_textures);
        rerender_indicator::draw(canvas, rerender_progress)
    }
}

mod loading_screen;
mod window_requests;

//! `--screenshot PATH`: saves one frame (as BMP) a second into the game, for comparing the
//! remake's look with the original's.

use std::path::PathBuf;
use std::time::{Duration, Instant};

use sdl3::render::WindowCanvas;

/// Long enough for the room to settle after the game starts.
const DELAY_AFTER_GAME_START: Duration = Duration::from_secs(1);

pub struct ScreenshotRequest {
    path: PathBuf,
    game_started: Option<Instant>,
    taken: bool,
}

impl ScreenshotRequest {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            game_started: None,
            taken: false,
        }
    }

    /// Call after drawing each frame, before presenting it.
    pub fn capture_if_due(&mut self, canvas: &WindowCanvas, game_running: bool) {
        if self.taken || !game_running {
            return;
        }
        let started = *self.game_started.get_or_insert_with(Instant::now);
        if started.elapsed() < DELAY_AFTER_GAME_START {
            return;
        }
        self.taken = true;
        let saved = canvas
            .read_pixels(None)
            .and_then(|surface| surface.save_bmp(&self.path));
        match saved {
            Ok(()) => println!("Saved a screenshot to {}", self.path.display()),
            Err(error) => println!("Could not save the screenshot: {error}"),
        }
    }
}

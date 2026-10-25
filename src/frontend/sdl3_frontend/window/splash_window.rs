//! The original projector's splash: a borderless window showing `isaacload.bmp` at its real pixel
//! size while the game loads behind it.

use std::path::Path;
use std::time::{Duration, Instant};

use sdl3::VideoSubsystem;
use sdl3::render::{BlendMode, WindowCanvas};
use sdl3::surface::Surface;

use crate::frontend::WINDOW_TITLE;
use crate::frontend::sdl3_frontend::window::pixel_size::points_for_pixels;
use crate::launch_options::SplashTime;

/// The projector's container stores 3000 ms next to the splash size (800×455): Zinc's display
/// time, kept with `--splash original` so the image is seen even when loading is faster.
const MINIMUM_DISPLAY_TIME: Duration = Duration::from_millis(3000);

pub struct SplashWindow {
    /// Kept so the window stays open; dropping it closes the splash.
    _canvas: WindowCanvas,
    shown_at: Instant,
    /// How long it stays up at least.
    display_time: Duration,
}

impl SplashWindow {
    /// `None` (with a message) if the bitmap cannot be shown; the game then starts without it.
    pub fn show(video: &VideoSubsystem, bitmap_path: &Path, splash_time: SplashTime) -> Option<Self> {
        let display_time = match splash_time {
            SplashTime::Original => MINIMUM_DISPLAY_TIME,
            SplashTime::Minimum => Duration::ZERO,
        };
        let shown = Self::open(video, bitmap_path, display_time);
        shown
            .inspect_err(|message| println!("No splash screen: {message}"))
            .ok()
    }

    fn open(video: &VideoSubsystem, bitmap_path: &Path, display_time: Duration) -> Result<Self, String> {
        let image = Surface::load_bmp(bitmap_path).map_err(|error| error.to_string())?;
        let window = video
            .window(
                WINDOW_TITLE,
                points_for_pixels(video, image.width()),
                points_for_pixels(video, image.height()),
            )
            .borderless()
            .position_centered()
            .high_pixel_density()
            .build()
            .map_err(|error| error.to_string())?;
        let mut canvas = window.into_canvas();
        let creator = canvas.texture_creator();
        let mut texture = creator
            .create_texture_from_surface(&image)
            .map_err(|error| error.to_string())?;
        // The bitmap's alpha bytes are all zero; it is meant to be opaque.
        texture.set_blend_mode(BlendMode::None);
        canvas.copy(&texture, None, None).map_err(|error| error.to_string())?;
        canvas.present();
        Ok(Self {
            _canvas: canvas,
            shown_at: Instant::now(),
            display_time,
        })
    }

    pub fn has_shown_long_enough(&self) -> bool {
        self.shown_at.elapsed() >= self.display_time
    }
}

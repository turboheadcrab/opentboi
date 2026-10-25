//! Keeps a windowed game at the requested size in real pixels, whatever the desktop scale.
//!
//! SDL sizes windows in points; on a 125 % desktop a 1024×768-point window is 1280×960 pixels. We
//! render at the window's pixel size (high pixel density), so the requested size is divided by
//! the window's pixel density. Wayland only reports the density once the window is on screen, and
//! it changes when the window moves to a display with another scale, so the size is checked each
//! frame until it matches, then again after every display change.

use sdl3::VideoSubsystem;
use sdl3::video::{FullscreenType, Window};

use crate::launch_options::WindowSize;

/// Pixel sizes come from rounding fractional point sizes; a pixel either way counts as a match.
const PIXEL_TOLERANCE: u32 = 1;
/// Stop correcting if the compositor keeps refusing the size (a tiling window manager, say).
const MAXIMUM_ATTEMPTS: u8 = 10;
const PERCENT: f32 = 100.0;

pub struct PixelSizeKeeper {
    requested: WindowSize,
    attempts_left: u8,
}

impl PixelSizeKeeper {
    pub fn new(requested: WindowSize) -> Self {
        Self {
            requested,
            attempts_left: MAXIMUM_ATTEMPTS,
        }
    }

    /// The window moved to another display (possibly with another scale): correct it again.
    pub fn display_changed(&mut self) {
        self.attempts_left = MAXIMUM_ATTEMPTS;
    }

    /// Resizes `window` if it is windowed and its pixel size is off; a no-op once it matches.
    pub fn apply(&mut self, window: &mut Window) {
        if self.attempts_left == 0 || window.fullscreen_state() != FullscreenType::Off {
            return;
        }
        let (pixel_width, pixel_height) = window.size_in_pixels();
        if pixel_width.abs_diff(self.requested.width) <= PIXEL_TOLERANCE
            && pixel_height.abs_diff(self.requested.height) <= PIXEL_TOLERANCE
        {
            self.attempts_left = 0;
            println!(
                "Window is {pixel_width}×{pixel_height} pixels ({:.0} % desktop scale)",
                window.display_scale() * PERCENT
            );
            return;
        }
        self.attempts_left -= 1;
        let pixels_per_point = window.pixel_density().max(f32::EPSILON);
        let to_points =
            |pixels: u32| crate::whole_numbers::clamped_u32((pixels as f32 / pixels_per_point).round()).max(1);
        if let Err(error) = window.set_size(to_points(self.requested.width), to_points(self.requested.height)) {
            println!("Could not resize the window: {error}");
        }
    }
}

/// Points that make `pixels` real pixels on the primary display, for sizing a window before it
/// exists (afterwards [`PixelSizeKeeper`] corrects it from the window's own density).
pub fn points_for_pixels(video: &VideoSubsystem, pixels: u32) -> u32 {
    let desktop_scale = video
        .get_primary_display()
        .and_then(|primary| primary.get_content_scale())
        .unwrap_or(1.0)
        .max(f32::EPSILON);
    crate::whole_numbers::clamped_u32((pixels as f32 / desktop_scale).round()).max(1)
}

//! What the window is asked to do between frames: fullscreen, display changes.

use sdl3::render::WindowCanvas;
use sdl3::video::FullscreenType;

use super::Session;
use crate::frontend::sdl3_frontend::device_input::WindowRequests;

impl Session<'_> {
    pub(super) fn handle_window_requests(
        &mut self,
        requests: &WindowRequests,
        canvas: &mut WindowCanvas,
    ) -> Result<(), String> {
        if requests.toggle_fullscreen {
            let window = canvas.window_mut();
            let is_fullscreen = window.fullscreen_state() != FullscreenType::Off;
            window
                .set_fullscreen(!is_fullscreen)
                .map_err(|error| error.to_string())?;
            self.pixel_size.display_changed();
        }
        if requests.display_changed {
            self.pixel_size.display_changed();
        }
        self.pixel_size.apply(canvas.window_mut());
        Ok(())
    }
}

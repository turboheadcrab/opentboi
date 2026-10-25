//! Shown while textures are rendered again for a new pixel density (after going fullscreen, say):
//! the paused game dimmed, with a progress bar in the middle.

use sdl3::pixels::Color;
use sdl3::render::{FRect, WindowCanvas};

const DIM_COLOR: Color = Color::RGBA(0, 0, 0, 160);
const BAR_COLOR: Color = Color::RGBA(255, 255, 255, 230);
/// The bar's size as fractions of the window's height.
const BAR_WIDTH_FRACTION: f32 = 0.5;
const BAR_HEIGHT_FRACTION: f32 = 0.03;
/// Gap between the outline and the fill, as a fraction of the bar's height.
const BAR_PADDING_FRACTION: f32 = 0.2;

/// Draws the indicator over the frame if `progress` (0 to 1) is set.
pub fn draw(canvas: &mut WindowCanvas, progress: Option<f32>) -> Result<(), String> {
    let Some(progress) = progress else {
        return Ok(());
    };
    let (width, height) = canvas.output_size().map_err(|error| error.to_string())?;
    let (width, height) = (width as f32, height as f32);
    canvas.set_draw_color(DIM_COLOR);
    canvas
        .fill_rect(FRect::new(0.0, 0.0, width, height))
        .map_err(|error| error.to_string())?;
    let bar_width = height * BAR_WIDTH_FRACTION;
    let bar_height = (height * BAR_HEIGHT_FRACTION).max(1.0);
    let outline = FRect::new(
        (width - bar_width) / 2.0,
        (height - bar_height) / 2.0,
        bar_width,
        bar_height,
    );
    let padding = bar_height * BAR_PADDING_FRACTION;
    let fill = FRect::new(
        outline.x + padding,
        outline.y + padding,
        (bar_width - padding * 2.0) * progress.clamp(0.0, 1.0),
        bar_height - padding * 2.0,
    );
    canvas.set_draw_color(BAR_COLOR);
    canvas.draw_rect(outline).map_err(|error| error.to_string())?;
    canvas.fill_rect(fill).map_err(|error| error.to_string())
}

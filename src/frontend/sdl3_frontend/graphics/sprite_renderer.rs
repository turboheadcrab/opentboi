//! Draws the game's draw list with SDL's 2D renderer, back to front, fitting the game's view into
//! the window with black bars (the same fit Bevy's `ScalingMode::AutoMin` gives).

use glam::Vec2;
use sdl3::pixels::Color;
use sdl3::render::{BlendMode, FRect, WindowCanvas};

use crate::frontend::sdl3_frontend::graphics::layer_target::LayerTarget;
use crate::frontend::sdl3_frontend::graphics::texture_library::TextureLibrary;
use crate::frontend::sdl3_frontend::graphics::vector_painter::VectorPainter;
use crate::game::{self, DrawItem, Rgba};

const CHANNEL_MAXIMUM: f32 = 255.0;
const BACKGROUND_COLOR: Color = Color::RGB(0, 0, 0);

/// Maps world units (origin at the room center, y up) to output pixels (origin top left, y down).
pub struct ViewMapping {
    pixels_per_unit: f32,
    view_origin_pixels: Vec2,
    /// Output pixels per window coordinate; above 1 on high-density displays.
    pixels_per_window_point: Vec2,
}

impl ViewMapping {
    pub fn for_canvas(canvas: &WindowCanvas) -> Result<Self, String> {
        let (output_width, output_height) = canvas.output_size().map_err(|error| error.to_string())?;
        let output_size = Vec2::new(output_width as f32, output_height as f32);
        let (window_width, window_height) = canvas.window().size();
        let window_size = Vec2::new(window_width as f32, window_height as f32).max(Vec2::ONE);
        let pixels_per_unit = (output_size / game::VIEW_SIZE).min_element();
        Ok(Self {
            pixels_per_unit,
            view_origin_pixels: (output_size - game::VIEW_SIZE * pixels_per_unit) / 2.0,
            pixels_per_window_point: output_size / window_size,
        })
    }

    fn to_pixels(&self, world_position: Vec2) -> Vec2 {
        let from_top_left = Vec2::new(
            world_position.x - game::VIEW_CENTER.x + game::VIEW_SIZE.x / 2.0,
            game::VIEW_CENTER.y + game::VIEW_SIZE.y / 2.0 - world_position.y,
        );
        self.view_origin_pixels + from_top_left * self.pixels_per_unit
    }

    /// The world position under a point given in window coordinates (mouse events).
    pub fn window_point_to_world(&self, window_point: Vec2) -> Vec2 {
        let from_top_left =
            (window_point * self.pixels_per_window_point - self.view_origin_pixels) / self.pixels_per_unit;
        Vec2::new(
            from_top_left.x - game::VIEW_SIZE.x / 2.0 + game::VIEW_CENTER.x,
            game::VIEW_CENTER.y + game::VIEW_SIZE.y / 2.0 - from_top_left.y,
        )
    }

    fn rectangle(&self, center: Vec2, size: Vec2) -> FRect {
        let size_pixels = size * self.pixels_per_unit;
        let top_left = self.to_pixels(center) - size_pixels / 2.0;
        FRect::new(top_left.x, top_left.y, size_pixels.x, size_pixels.y)
    }
}

/// Sorts `draw_items` by depth and draws them; items without a loaded texture become rectangles
/// of their fallback color. A run of items of one layer is drawn into the layer's picture first.
pub fn draw(
    canvas: &mut WindowCanvas,
    (textures, vectors, layers): (&mut TextureLibrary, &mut VectorPainter, &mut LayerTarget),
    draw_items: &mut [DrawItem],
    view: &ViewMapping,
) -> Result<(), String> {
    // A stable sort, so that items of equal depth keep the order they were listed in.
    draw_items.sort_by(|first, second| first.depth.total_cmp(&second.depth));
    clear(canvas);
    let mut rest: &[DrawItem] = draw_items;
    while let Some(first) = rest.first() {
        let run = rest.iter().take_while(|item| item.layer == first.layer).count();
        let (items, after) = rest.split_at(run);
        match first.layer {
            Some(layer) => {
                vectors.flush(canvas)?;
                layers.draw(canvas, layer.opacity, |layer_canvas| {
                    draw_run(layer_canvas, textures, vectors, items, view)?;
                    vectors.flush(layer_canvas)
                })?;
            }
            None => draw_run(canvas, textures, vectors, items, view)?,
        }
        rest = after;
    }
    vectors.flush(canvas)
}

/// Draws `draw_items` in order, the ones drawn as vectors queued until something else comes.
fn draw_run(
    canvas: &mut WindowCanvas,
    textures: &mut TextureLibrary,
    vectors: &mut VectorPainter,
    draw_items: &[DrawItem],
    view: &ViewMapping,
) -> Result<(), String> {
    for draw_item in draw_items {
        let rectangle = view.rectangle(draw_item.center, draw_item.size);
        if vectors.queue(draw_item, rectangle) {
            continue;
        }
        vectors.flush(canvas)?;
        // SDL rotates clockwise on a y-down screen; the game's angle is counterclockwise, y up.
        if let Some(texture) = draw_item.texture.and_then(|texture_id| textures.texture(texture_id)) {
            let tint = to_sdl_color(draw_item.tint);
            texture.set_color_mod(tint.r, tint.g, tint.b);
            texture.set_alpha_mod(tint.a);
            canvas.copy_ex(
                texture,
                None,
                rectangle,
                -f64::from(draw_item.rotation_radians.to_degrees()),
                None,
                draw_item.flip_horizontally,
                false,
            )
        } else {
            canvas.set_draw_color(to_sdl_color(draw_item.color));
            canvas.fill_rect(rectangle)
        }
        .map_err(|error| error.to_string())?;
    }
    Ok(())
}

/// Fills the whole output with the background color, ready for alpha-blended drawing.
pub fn clear(canvas: &mut WindowCanvas) {
    canvas.set_draw_color(BACKGROUND_COLOR);
    canvas.clear();
    canvas.set_blend_mode(BlendMode::Blend);
}

fn to_sdl_color(color: Rgba) -> Color {
    let to_byte = |channel: f32| crate::whole_numbers::clamped_u8((channel * CHANNEL_MAXIMUM).round());
    Color::RGBA(
        to_byte(color.red),
        to_byte(color.green),
        to_byte(color.blue),
        to_byte(color.alpha),
    )
}

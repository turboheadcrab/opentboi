//! Draw layers (`DrawLayer`): the items of one layer are drawn into an off-screen picture the size of
//! the output, as they are, and the picture is then put on the screen at the layer's opacity, the
//! way Flash shows a bitmap with an `_alpha`. Overlapping items cover each other instead of adding up.

use sdl3::pixels::{Color, PixelFormat};
use sdl3::render::{BlendMode, Texture, TextureCreator, WindowCanvas};
use sdl3::sys::blendmode::SDL_BLENDMODE_BLEND_PREMULTIPLIED;
use sdl3::video::WindowContext;

use crate::frontend::sdl3_frontend::graphics::vector_painter::bitmap_textures::set_texture_blend_mode;
use crate::game::DrawLayer;

const TRANSPARENT: Color = Color::RGBA(0, 0, 0, 0);
const CHANNEL_MAXIMUM: f32 = 255.0;

pub struct LayerTarget<'creator> {
    creator: &'creator TextureCreator<WindowContext>,
    /// Kept between frames; made again when the output grows.
    picture: Option<Texture<'creator>>,
}

impl<'creator> LayerTarget<'creator> {
    pub fn new(creator: &'creator TextureCreator<WindowContext>) -> Self {
        Self { creator, picture: None }
    }

    /// Clears the layer's picture, lets `draw` paint into it (the canvas it gets draws there), then
    /// puts it on the canvas at `layer`'s opacity.
    pub fn draw(
        &mut self,
        canvas: &mut WindowCanvas,
        layer: DrawLayer,
        draw: impl FnOnce(&mut WindowCanvas) -> Result<(), String>,
    ) -> Result<(), String> {
        let (width, height) = canvas.output_size().map_err(|error| error.to_string())?;
        let mut picture = self.take_picture(width, height)?;
        let mut drawn = Ok(());
        canvas
            .with_texture_canvas(&mut picture, |layer_canvas| {
                layer_canvas.set_blend_mode(BlendMode::None);
                layer_canvas.set_draw_color(TRANSPARENT);
                layer_canvas.clear();
                // Straight colors blended onto a clear picture leave premultiplied ones in it.
                layer_canvas.set_blend_mode(BlendMode::Blend);
                drawn = draw(layer_canvas);
            })
            .map_err(|error| error.to_string())?;
        drawn?;
        // Premultiplied: the opacity scales the colors as well as the coverage.
        let opacity = crate::whole_numbers::clamped_u8((layer.opacity * CHANNEL_MAXIMUM).round());
        picture.set_color_mod(opacity, opacity, opacity);
        picture.set_alpha_mod(opacity);
        canvas.set_blend_mode(BlendMode::Blend);
        let copied = canvas.copy(&picture, None, None).map_err(|error| error.to_string());
        self.picture = Some(picture);
        copied
    }

    fn take_picture(&mut self, width: u32, height: u32) -> Result<Texture<'creator>, String> {
        if let Some(picture) = self.picture.take() {
            let query = picture.query();
            if query.width == width && query.height == height {
                return Ok(picture);
            }
        }
        let picture = self
            .creator
            .create_texture_target(PixelFormat::RGBA32, width, height)
            .map_err(|error| error.to_string())?;
        set_texture_blend_mode(&picture, SDL_BLENDMODE_BLEND_PREMULTIPLIED);
        Ok(picture)
    }
}

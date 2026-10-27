//! Draws textures as triangles where the SWF allows it ([`VectorPicture`]), so they need no
//! rendered texture and stay sharp at any size. Pictures next to each other in depth order share
//! one off-screen layer at twice the output resolution, which is then scaled down with linear
//! filtering, averaging each 2×2 block: Flash's medium-quality edges. Bitmap fills sample the
//! SWF's bitmaps.

pub(super) mod bitmap_textures;
mod triangles;

use std::collections::HashMap;

use sdl3::pixels::{Color, PixelFormat};
use sdl3::render::{BlendMode, FRect, ScaleMode, Texture, TextureCreator, VertexIndices, WindowCanvas};
use sdl3::sys::blendmode::SDL_BLENDMODE_BLEND_PREMULTIPLIED;
use sdl3::video::WindowContext;

use crate::frontend::sdl3_frontend::graphics::vector_families::draws_as_vectors;
use crate::game::{DrawItem, Rgba, TextureId};
use crate::game_assets::{TextureRenderer, VectorPicture};
use bitmap_textures::{BitmapTextures, set_texture_blend_mode};
use triangles::{Placement, Triangles};

/// Off-screen pixels per output pixel along each axis.
const SUPERSAMPLING: f32 = 2.0;
const TRANSPARENT: Color = Color::RGBA(0, 0, 0, 0);

pub struct VectorPainter<'creator> {
    creator: &'creator TextureCreator<WindowContext>,
    renderer: Option<&'static TextureRenderer>,
    /// `None` remembers textures that cannot be drawn as vectors.
    pictures: HashMap<TextureId, Option<VectorPicture>>,
    /// The off-screen layer, covering the output at twice its resolution.
    layer: Option<Texture<'creator>>,
    bitmaps: BitmapTextures<'creator>,
    /// What is queued for the layer, and the output pixels it covers.
    triangles: Triangles,
    covered: Option<FRect>,
}

impl<'creator> VectorPainter<'creator> {
    pub fn new(creator: &'creator TextureCreator<WindowContext>) -> Self {
        Self {
            creator,
            renderer: None,
            pictures: HashMap::new(),
            layer: None,
            bitmaps: BitmapTextures::new(creator),
            triangles: Triangles::default(),
            covered: None,
        }
    }

    pub fn set_renderer(&mut self, renderer: &'static TextureRenderer) {
        self.renderer = Some(renderer);
    }

    /// Whether `texture_id` is drawn as vectors, needing no texture.
    pub fn can_draw(&mut self, texture_id: TextureId) -> bool {
        self.picture(texture_id).is_some()
    }

    fn picture(&mut self, texture_id: TextureId) -> Option<&VectorPicture> {
        let renderer = self.renderer.filter(|_| draws_as_vectors(texture_id))?;
        self.pictures
            .entry(texture_id)
            .or_insert_with(|| renderer.vector_picture(texture_id))
            .as_ref()
    }

    /// Queues `draw_item` for the layer if it is drawn as vectors, filling `rectangle` (output
    /// pixels, before rotation); `false` if it needs its texture.
    pub fn queue(&mut self, draw_item: &DrawItem, rectangle: FRect) -> bool {
        let Some(texture_id) = draw_item.texture.filter(|&texture_id| self.can_draw(texture_id)) else {
            return false;
        };
        let world_size = texture_id.world_size();
        let mirror = if draw_item.flip_horizontally { -1.0 } else { 1.0 };
        // SDL turns clockwise on a y-down screen; the game's angle is counterclockwise, y up.
        let (sine, cosine) = (-draw_item.rotation_radians).sin_cos();
        let placement = Placement {
            center: (
                (rectangle.x + rectangle.w / 2.0) * SUPERSAMPLING,
                (rectangle.y + rectangle.h / 2.0) * SUPERSAMPLING,
            ),
            scale: (
                rectangle.w / world_size.x * SUPERSAMPLING * mirror,
                rectangle.h / world_size.y * SUPERSAMPLING,
            ),
            rotation: (cosine, sine),
            tint: premultiplied(draw_item.tint),
        };
        let bounds = rotated_bounds(rectangle, cosine, sine);
        self.covered = Some(self.covered.map_or(bounds, |covered| union(covered, bounds)));
        let mut triangles = std::mem::take(&mut self.triangles);
        if let Some(picture) = self.picture(texture_id) {
            triangles.add_picture(picture, placement);
        }
        self.triangles = triangles;
        true
    }

    /// Draws what is queued onto the canvas, under anything drawn after.
    pub fn flush(&mut self, canvas: &mut WindowCanvas) -> Result<(), String> {
        let covered = self.covered.take();
        if self.triangles.is_empty() {
            return Ok(());
        }
        let (Some(renderer), Some(covered)) = (self.renderer, covered) else {
            return Ok(());
        };
        let (output_width, output_height) = canvas.output_size().map_err(|error| error.to_string())?;
        let output = FRect::new(0.0, 0.0, output_width as f32, output_height as f32);
        let Some(visible) = intersection(covered, output) else {
            self.triangles.clear();
            return Ok(());
        };
        self.bitmaps
            .upload_missing(renderer, self.triangles.batches.iter().filter_map(|batch| batch.bitmap));
        let mut layer = self.take_layer(output_width, output_height)?;
        let region = scaled(visible, SUPERSAMPLING);
        let drawn = self.draw_layer(canvas, &mut layer, region);
        self.triangles.clear();
        drawn?;
        canvas.set_blend_mode(BlendMode::Blend);
        canvas
            .copy(&layer, region, visible)
            .map_err(|error| error.to_string())?;
        self.layer = Some(layer);
        Ok(())
    }

    /// Clears `region` of the layer and draws the queued triangles into it.
    fn draw_layer(&self, canvas: &mut WindowCanvas, layer: &mut Texture, region: FRect) -> Result<(), String> {
        let mut drawn = Ok(());
        canvas
            .with_texture_canvas(layer, |layer_canvas| {
                layer_canvas.set_blend_mode(BlendMode::None);
                layer_canvas.set_draw_color(TRANSPARENT);
                drawn = layer_canvas.fill_rect(region);
                set_premultiplied_blending(layer_canvas);
                for batch in &self.triangles.batches {
                    let bitmap = batch.bitmap.and_then(|key| self.bitmaps.get(key));
                    let indices = VertexIndices::U32(&self.triangles.indices[batch.indices.clone()]);
                    let result = layer_canvas.render_geometry(&self.triangles.vertices, bitmap, indices);
                    drawn = std::mem::replace(&mut drawn, Ok(())).and(result);
                }
            })
            .map_err(|error| error.to_string())?;
        drawn.map_err(|error| error.to_string())
    }

    /// The layer for an output of this size, made anew when the output grows.
    fn take_layer(&mut self, output_width: u32, output_height: u32) -> Result<Texture<'creator>, String> {
        let width = crate::whole_numbers::clamped_u32((output_width as f32 * SUPERSAMPLING).ceil());
        let height = crate::whole_numbers::clamped_u32((output_height as f32 * SUPERSAMPLING).ceil());
        if let Some(layer) = self.layer.take() {
            let query = layer.query();
            if query.width >= width && query.height >= height {
                return Ok(layer);
            }
        }
        let mut layer = self
            .creator
            .create_texture_target(PixelFormat::RGBA32, width, height)
            .map_err(|error| error.to_string())?;
        layer.set_scale_mode(ScaleMode::Linear);
        set_texture_blend_mode(&layer, SDL_BLENDMODE_BLEND_PREMULTIPLIED);
        Ok(layer)
    }
}

/// The triangles carry premultiplied colors; the safe API has no such blend mode.
#[allow(unsafe_code, reason = "SDL's safe wrapper has no premultiplied blend mode")]
fn set_premultiplied_blending(canvas: &mut WindowCanvas) {
    // SAFETY: the renderer is alive and the mode is a valid SDL blend mode.
    unsafe { sdl3::sys::render::SDL_SetRenderDrawBlendMode(canvas.raw(), SDL_BLENDMODE_BLEND_PREMULTIPLIED) };
}

/// A tint as factors for premultiplied colors: the color channels carry its opacity too.
fn premultiplied(tint: Rgba) -> [f32; 4] {
    [
        tint.red * tint.alpha,
        tint.green * tint.alpha,
        tint.blue * tint.alpha,
        tint.alpha,
    ]
}

/// The axis-aligned box around `rectangle` turned by the given angle around its center, grown to
/// whole pixels so linear filtering at its edges stays inside the cleared region.
fn rotated_bounds(rectangle: FRect, cosine: f32, sine: f32) -> FRect {
    let half_width = (rectangle.w * cosine).abs() / 2.0 + (rectangle.h * sine).abs() / 2.0;
    let half_height = (rectangle.w * sine).abs() / 2.0 + (rectangle.h * cosine).abs() / 2.0;
    let center = (rectangle.x + rectangle.w / 2.0, rectangle.y + rectangle.h / 2.0);
    let left = (center.0 - half_width).floor() - 1.0;
    let top = (center.1 - half_height).floor() - 1.0;
    let right = (center.0 + half_width).ceil() + 1.0;
    let bottom = (center.1 + half_height).ceil() + 1.0;
    FRect::new(left, top, right - left, bottom - top)
}

fn union(first: FRect, second: FRect) -> FRect {
    let left = first.x.min(second.x);
    let top = first.y.min(second.y);
    let right = (first.x + first.w).max(second.x + second.w);
    let bottom = (first.y + first.h).max(second.y + second.h);
    FRect::new(left, top, right - left, bottom - top)
}

fn intersection(first: FRect, second: FRect) -> Option<FRect> {
    let left = first.x.max(second.x);
    let top = first.y.max(second.y);
    let right = (first.x + first.w).min(second.x + second.w);
    let bottom = (first.y + first.h).min(second.y + second.h);
    (right > left && bottom > top).then(|| FRect::new(left, top, right - left, bottom - top))
}

fn scaled(rectangle: FRect, factor: f32) -> FRect {
    FRect::new(
        rectangle.x * factor,
        rectangle.y * factor,
        rectangle.w * factor,
        rectangle.h * factor,
    )
}

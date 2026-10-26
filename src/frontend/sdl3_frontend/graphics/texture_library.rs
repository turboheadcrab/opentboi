//! Resolves the game's texture ids to SDL textures. The loader renders every texture during the
//! intro and they are uploaded as they arrive; anything still missing is rendered the first time it
//! is drawn. Only the GPU copy is kept.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use sdl3::pixels::PixelFormat;
use sdl3::render::{BlendMode, Texture, TextureCreator};
use sdl3::video::WindowContext;

use crate::frontend::texture_prefetch::TexturePrefetcher;
use crate::game::TextureId;
use crate::game_assets::{RgbaImage, TextureRenderer};

#[derive(Default)]
struct RenderStatistics {
    rendered_count: u32,
    total_time: Duration,
    slowest_time: Duration,
    slowest_texture: Option<TextureId>,
}

pub struct TextureLibrary<'creator> {
    creator: &'creator TextureCreator<WindowContext>,
    renderer: Option<&'static TextureRenderer>,
    prefetcher: Option<TexturePrefetcher>,
    /// Density textures are rendered at; follows the window (see `Session`).
    pixels_per_world_unit: f32,
    /// `None` remembers textures that failed to render, so they are not retried every frame.
    textures_by_id: HashMap<TextureId, Option<Texture<'creator>>>,
    statistics: RenderStatistics,
}

impl<'creator> TextureLibrary<'creator> {
    pub fn new(creator: &'creator TextureCreator<WindowContext>, pixels_per_world_unit: f32) -> Self {
        Self {
            creator,
            renderer: None,
            prefetcher: None,
            pixels_per_world_unit,
            textures_by_id: HashMap::new(),
            statistics: RenderStatistics::default(),
        }
    }

    /// Enables rendering on first use once the SWF is parsed.
    pub fn set_renderer(&mut self, renderer: &'static TextureRenderer) {
        self.renderer = Some(renderer);
        self.prefetcher = Some(TexturePrefetcher::new(renderer));
    }

    pub fn renderer(&self) -> Option<&'static TextureRenderer> {
        self.renderer
    }

    /// Textures rendered from now on (on first use) use this density.
    pub fn set_pixels_per_world_unit(&mut self, pixels_per_world_unit: f32) {
        self.pixels_per_world_unit = pixels_per_world_unit;
    }

    /// Uploads a texture the loader rendered ahead of time (replacing any older version).
    pub fn insert(&mut self, texture_id: TextureId, rgba_image: &RgbaImage) {
        let texture = upload(self.creator, texture_id, rgba_image);
        self.textures_by_id.insert(texture_id, texture);
    }

    /// The texture, rendering and uploading it now if this is its first use.
    pub fn texture(&mut self, texture_id: TextureId) -> Option<&mut Texture<'creator>> {
        if !self.textures_by_id.contains_key(&texture_id) {
            let started = Instant::now();
            let texture = self.render(texture_id);
            self.record_render_time(texture_id, started.elapsed());
            self.textures_by_id.insert(texture_id, texture);
        }
        self.textures_by_id.get_mut(&texture_id)?.as_mut()
    }

    /// Uploads what the background renderers finished, and asks them for `upcoming`.
    pub fn prefetch(&mut self, upcoming: &[TextureId]) {
        let Some(prefetcher) = &mut self.prefetcher else { return };
        for (texture_id, image) in prefetcher.take_finished(self.pixels_per_world_unit) {
            if !self.textures_by_id.contains_key(&texture_id) {
                let texture = upload(self.creator, texture_id, &image);
                self.textures_by_id.insert(texture_id, texture);
            }
        }
        let known = &self.textures_by_id;
        prefetcher.request(upcoming, self.pixels_per_world_unit, |texture_id| {
            known.contains_key(&texture_id)
        });
    }

    fn render(&self, texture_id: TextureId) -> Option<Texture<'creator>> {
        let renderer = self.renderer?;
        match renderer.render(texture_id, self.pixels_per_world_unit) {
            Ok((rgba_image, unsupported)) => {
                if !unsupported.is_empty() {
                    println!("{texture_id:?}: art features skipped: {unsupported:?}");
                }
                upload(self.creator, texture_id, &rgba_image)
            }
            Err(render_error) => {
                println!("{texture_id:?} artwork unavailable, using a placeholder: {render_error}");
                None
            }
        }
    }

    fn record_render_time(&mut self, texture_id: TextureId, elapsed: Duration) {
        let statistics = &mut self.statistics;
        statistics.rendered_count += 1;
        statistics.total_time += elapsed;
        if elapsed > statistics.slowest_time {
            statistics.slowest_time = elapsed;
            statistics.slowest_texture = Some(texture_id);
        }
    }
}

impl Drop for TextureLibrary<'_> {
    fn drop(&mut self) {
        let statistics = &self.statistics;
        if let Some(slowest_texture) = statistics.slowest_texture {
            println!(
                "Rendered {} textures on demand in {} ms total; slowest {:?} took {:.1} ms",
                statistics.rendered_count,
                statistics.total_time.as_millis(),
                slowest_texture,
                statistics.slowest_time.as_secs_f64() * 1000.0
            );
        }
    }
}

/// Our images are straight-alpha RGBA bytes, which SDL calls `RGBA32` and blends with `Blend`.
pub fn upload<'creator>(
    creator: &'creator TextureCreator<WindowContext>,
    texture_id: TextureId,
    rgba_image: &RgbaImage,
) -> Option<Texture<'creator>> {
    let uploaded = creator
        .create_texture_static(PixelFormat::RGBA32, rgba_image.width, rgba_image.height)
        .map_err(|error| error.to_string())
        .and_then(|mut texture| {
            let row_bytes = rgba_image.width as usize * RgbaImage::BYTES_PER_PIXEL;
            texture
                .update(None, &rgba_image.pixels, row_bytes)
                .map_err(|error| error.to_string())?;
            texture.set_blend_mode(BlendMode::Blend);
            Ok(texture)
        });
    uploaded
        .inspect_err(|message| println!("{texture_id:?} could not be uploaded: {message}"))
        .ok()
}

//! Resolves the game's texture ids to Bevy images, rendering each one from the SWF the first time
//! it is drawn and keeping only the GPU copy.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

use crate::frontend::texture_prefetch::TexturePrefetcher;
use crate::game::TextureId;
use crate::game_assets::{RgbaImage, TextureSupply};

const SINGLE_LAYER: u32 = 1;

#[derive(Default)]
struct RenderStatistics {
    rendered_count: u32,
    total_time: Duration,
    slowest_time: Duration,
    slowest_texture: Option<TextureId>,
}

#[derive(Resource)]
pub struct TextureLibrary {
    supply: TextureSupply,
    /// `None` remembers textures that failed to render, so they are not retried every frame.
    handles_by_id: HashMap<TextureId, Option<Handle<Image>>>,
    statistics: RenderStatistics,
    prefetcher: Option<TexturePrefetcher>,
}

impl TextureLibrary {
    pub fn new(mut supply: TextureSupply, images: &mut Assets<Image>) -> Self {
        let handles_by_id = std::mem::take(&mut supply.prerendered)
            .into_iter()
            .map(|(texture_id, rgba_image)| (texture_id, Some(images.add(to_bevy_image(rgba_image)))))
            .collect();
        let prefetcher = supply.renderer.map(TexturePrefetcher::new);
        Self {
            supply,
            handles_by_id,
            statistics: RenderStatistics::default(),
            prefetcher,
        }
    }

    /// The texture's handle, rendering and uploading it now if this is its first use.
    pub fn handle(&mut self, texture_id: TextureId, images: &mut Assets<Image>) -> Option<Handle<Image>> {
        if let Some(known) = self.handles_by_id.get(&texture_id) {
            return known.clone();
        }
        let renderer = self.supply.renderer.as_ref()?;
        let started = Instant::now();
        let handle = match renderer.render(texture_id, self.supply.pixels_per_world_unit) {
            Ok((rgba_image, unsupported)) => {
                if !unsupported.is_empty() {
                    println!("{texture_id:?}: art features skipped: {unsupported:?}");
                }
                Some(images.add(to_bevy_image(rgba_image)))
            }
            Err(render_error) => {
                println!("{texture_id:?} artwork unavailable, using a placeholder: {render_error}");
                None
            }
        };
        self.record_render_time(texture_id, started.elapsed());
        self.handles_by_id.insert(texture_id, handle.clone());
        handle
    }

    /// Uploads what the background renderers finished, and asks them for `upcoming`.
    pub fn prefetch(&mut self, upcoming: &[TextureId], images: &mut Assets<Image>) {
        let Some(prefetcher) = &mut self.prefetcher else { return };
        let density = self.supply.pixels_per_world_unit;
        for (texture_id, image) in prefetcher.take_finished(density) {
            self.handles_by_id
                .entry(texture_id)
                .or_insert_with(|| Some(images.add(to_bevy_image(image))));
        }
        let known = &self.handles_by_id;
        prefetcher.request(upcoming, density, |texture_id| known.contains_key(&texture_id));
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

impl Drop for TextureLibrary {
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

fn to_bevy_image(rgba_image: RgbaImage) -> Image {
    Image::new(
        Extent3d {
            width: rgba_image.width,
            height: rgba_image.height,
            depth_or_array_layers: SINGLE_LAYER,
        },
        TextureDimension::D2,
        rgba_image.pixels,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    )
}

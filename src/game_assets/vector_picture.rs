//! Textures drawn as triangles instead of pixels: a texture's frame as the meshes of the shapes it
//! places, each with its transform and color effect. Frontends draw these at any size without
//! rendering the texture first; bitmap fills sample the SWF's bitmaps ([`TextureRenderer::bitmap`]).

use crate::game_assets::vector_art::TWIPS_PER_PIXEL;
mod animation;
use std::sync::Arc;

use super::TextureRenderer;
use super::input_glyphs::{self, GlyphLayer};
use super::texture_catalog::place_texture;
use super::vector_art::{Character, ColorEffect, GlyphKey, RgbaImage, ShapeSource, decode_bitmap};
use super::vector_mesh::{MeshFill, RuffleMesher, ShapeMesh, ShapeMesher};
use crate::game::TextureId;

/// What a cached mesh was made from.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MeshKey {
    Character(u16),
    Glyph(GlyphKey),
}

/// One texture frame as meshes, back to front.
pub struct VectorPicture {
    pub parts: Vec<VectorPart>,
}

pub struct VectorPart {
    pub mesh: Arc<ShapeMesh>,
    /// `[a, b, c, d, tx, ty]`: a mesh point `(x, y)` lands at `(a x + c y + tx, b x + d y + ty)`,
    /// in Flash pixels (y down) from the texture's center.
    pub transform: [f32; 6],
    color_effect: ColorEffect,
}

impl VectorPart {
    pub fn place(&self, [x, y]: [f32; 2]) -> [f32; 2] {
        let [scale_x, skew_y, skew_x, scale_y, translate_x, translate_y] = self.transform;
        [
            scale_x * x + skew_x * y + translate_x,
            skew_y * x + scale_y * y + translate_y,
        ]
    }

    /// How a bitmap fill is tinted: texel multipliers, and the additive pass's color.
    pub fn bitmap_tint(&self) -> ([f32; 4], [f32; 4]) {
        self.color_effect.bitmap_tint()
    }

    /// A vertex's straight RGBA color through the clip's color effects, premultiplied.
    pub fn color(&self, rgba: [u8; 4]) -> [f32; 4] {
        self.color_effect.apply_premultiplied(rgba)
    }
}

impl TextureRenderer {
    /// `texture_id` as meshes, if everything in it can be drawn that way: solid and bitmap fills
    /// and strokes (gradients and text are left to textures for now). Isaac's animations are drawn
    /// one way throughout: frames with gradients would otherwise alternate with flat ones that
    /// look different, which shows as flicker.
    pub fn vector_picture(&self, texture_id: TextureId) -> Option<VectorPicture> {
        if !self.animation_is_drawable(texture_id) {
            return None;
        }
        self.frame_picture(texture_id)
    }

    pub(super) fn frame_picture(&self, texture_id: TextureId) -> Option<VectorPicture> {
        if let Some(layers) = input_glyphs::glyph_layers(texture_id) {
            return Some(self.glyph_picture(&layers.ok()?));
        }
        let placements = place_texture(&self.library, texture_id)?;
        let parts = placements
            .into_iter()
            .map(|placement| {
                let transform = placement.transform.pre_scale(TWIPS_PER_PIXEL, TWIPS_PER_PIXEL);
                Some(VectorPart {
                    mesh: self.drawable_mesh(&placement.source)?,
                    transform: [
                        transform.sx,
                        transform.ky,
                        transform.kx,
                        transform.sy,
                        transform.tx,
                        transform.ty,
                    ],
                    color_effect: placement.color_effect,
                })
            })
            .collect::<Option<Vec<_>>>()?;
        Some(VectorPicture { parts })
    }

    /// A pad's button picture: its layers meshed like any shape of the SWF.
    fn glyph_picture(&self, layers: &[GlyphLayer]) -> VectorPicture {
        let mesh = RuffleMesher::new(&self.library).mesh(&input_glyphs::layers_to_shape(layers));
        let scale = 1.0 / input_glyphs::MESH_UNITS_PER_WORLD_UNIT;
        VectorPicture {
            parts: vec![VectorPart {
                mesh: Arc::new(mesh),
                transform: [scale, 0.0, 0.0, scale, 0.0, 0.0],
                color_effect: ColorEffect::IDENTITY,
            }],
        }
    }

    /// The SWF bitmap `bitmap_id`, premultiplied, for bitmap fills.
    pub fn bitmap(&self, bitmap_id: u16) -> Option<RgbaImage> {
        let pixmap = decode_bitmap(self.library.bitmap(bitmap_id)?)?;
        Some(RgbaImage {
            width: pixmap.width(),
            height: pixmap.height(),
            pixels: pixmap.take(),
        })
    }

    /// The shape's mesh, made once; `None` if it has gradients or bitmaps that do not decode.
    fn drawable_mesh(&self, source: &ShapeSource) -> Option<Arc<ShapeMesh>> {
        let key = match source {
            ShapeSource::Character(shape_id) => MeshKey::Character(*shape_id),
            ShapeSource::Glyph(glyph, _) => MeshKey::Glyph(*glyph),
        };
        let mut meshes = self.meshes.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        meshes
            .entry(key)
            .or_insert_with(|| {
                let shape = match source {
                    ShapeSource::Character(shape_id) => match self.library.character(*shape_id) {
                        Some(Character::Shape(shape)) => *shape,
                        _ => return None,
                    },
                    ShapeSource::Glyph(_, shape) => shape,
                };
                let mesh = RuffleMesher::new(&self.library).mesh(shape);
                let drawable = mesh.draws.iter().all(|draw| match draw.fill {
                    MeshFill::Solid => true,
                    MeshFill::Bitmap { bitmap_id, .. } => self.library.bitmap(bitmap_id).is_some(),
                    MeshFill::Gradient { .. } => false,
                });
                drawable.then(|| Arc::new(mesh))
            })
            .clone()
    }
}

#[cfg(test)]
mod tests {
    use crate::game::TextureId;
    use crate::game_assets::{EXTRACTED_SWF_FILE_NAME, cache_directory, load_assets};

    /// Needs the SWF extracted from a real Steam install; run with `cargo test -- --ignored`.
    #[test]
    #[ignore = "requires the game SWF extracted into the user cache"]
    fn full_screen_overlays_draw_as_vectors() {
        let swf_path = cache_directory()
            .expect("no cache directory")
            .join(EXTRACTED_SWF_FILE_NAME);
        let renderer = load_assets(&swf_path)
            .expect("extracted SWF should parse")
            .texture_renderer;
        let overlays: Vec<TextureId> = TextureId::all()
            .into_iter()
            .filter(|texture_id| {
                matches!(
                    texture_id,
                    TextureId::CardAnimation { .. } | TextureId::EternalHeartUnion(_)
                )
            })
            .collect();
        let textures_only: Vec<TextureId> = overlays
            .iter()
            .copied()
            .filter(|&texture_id| renderer.vector_picture(texture_id).is_none())
            .collect();
        println!(
            "{} of {} overlay frames need textures: {textures_only:?}",
            textures_only.len(),
            overlays.len()
        );
        assert!(textures_only.is_empty(), "{textures_only:?}");
    }
    /// Density at which the SDL3 frontend renders on a typical window (see `docs/texture-streaming.md`).
    const MEASURED_PIXELS_PER_UNIT: f32 = 1.6;
    const MEBIBYTE: f32 = 1024.0 * 1024.0;

    /// Development aid: which texture families the mesh path can draw, and how much video memory
    /// the textures it could replace take. Run with
    /// `cargo test vector_drawable_share -- --ignored --nocapture`.
    #[test]
    #[ignore = "requires the game SWF extracted into the user cache"]
    fn vector_drawable_share_per_texture_family() {
        let swf_path = cache_directory()
            .expect("no cache directory")
            .join(EXTRACTED_SWF_FILE_NAME);
        let renderer = load_assets(&swf_path)
            .expect("extracted SWF should parse")
            .texture_renderer;
        // family -> (frames, drawable frames, bytes, drawable bytes)
        let mut families: std::collections::BTreeMap<String, (u32, u32, f32, f32)> = std::collections::BTreeMap::new();
        for texture_id in TextureId::all().into_iter().chain(TextureId::on_demand_textures()) {
            let debug = format!("{texture_id:?}");
            let family = debug.split(['(', ' ', '{']).next().unwrap_or_default().to_string();
            let size = texture_id.world_size() * MEASURED_PIXELS_PER_UNIT;
            let bytes = size.x * size.y * 4.0;
            let entry = families.entry(family).or_default();
            entry.0 += 1;
            entry.2 += bytes;
            if renderer.vector_picture(texture_id).is_some() {
                entry.1 += 1;
                entry.3 += bytes;
            }
        }
        let mut rows: Vec<_> = families.into_iter().collect();
        rows.sort_by(|first, second| second.1.2.total_cmp(&first.1.2));
        println!(
            "{:<28} {:>7} {:>9} {:>10} {:>12}",
            "family", "frames", "drawable", "MiB", "drawable MiB"
        );
        for (family, (frames, drawable, bytes, drawable_bytes)) in rows {
            println!(
                "{family:<28} {frames:>7} {drawable:>9} {:>10.1} {:>12.1}",
                bytes / MEBIBYTE,
                drawable_bytes / MEBIBYTE
            );
        }
    }
}

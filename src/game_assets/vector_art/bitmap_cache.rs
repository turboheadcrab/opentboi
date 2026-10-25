//! Decodes the bitmaps a shape fills with, once per render pass, and turns them into shaders.
//! Bitmaps under a color effect are recolored into a separate copy per effect, because a pattern
//! shader can only fade a bitmap, while Flash also tints it (e.g. the fireplaces' orange shadow).

use super::RgbaImage;
use std::collections::HashMap;

use swf::{CharacterId, FillStyle, Matrix, Shape, ShapeRecord, ShapeStyles};
use tiny_skia::{FilterQuality, Pattern, Pixmap, Shader, SpreadMode};

use super::bitmap_decoder::decode_bitmap;
use super::character_library::CharacterLibrary;
use super::color_effect::ColorEffect;
use super::painter::to_skia_transform;

const FULL_OPACITY: f32 = 1.0;

type EffectKey = [u32; 8];

#[derive(Default)]
pub struct BitmapCache {
    /// `None` marks bitmaps that are missing or could not be decoded.
    decoded: HashMap<CharacterId, Option<Pixmap>>,
    recolored: HashMap<(CharacterId, EffectKey), Pixmap>,
}

impl BitmapCache {
    /// Prepares every bitmap `shape` fills with under `color_effect`, so painting can borrow them
    /// immutably.
    pub fn prepare_bitmaps_used_by(&mut self, shape: &Shape, color_effect: ColorEffect, library: &CharacterLibrary) {
        let style_tables = std::iter::once(&shape.styles).chain(shape.shape.iter().filter_map(|record| match record {
            ShapeRecord::StyleChange(change) => change.new_styles.as_ref(),
            _ => None,
        }));
        for styles in style_tables {
            for bitmap_id in bitmap_ids(styles) {
                let decoded = self
                    .decoded
                    .entry(bitmap_id)
                    .or_insert_with(|| library.bitmap(bitmap_id).and_then(decode_bitmap));
                if color_effect == ColorEffect::IDENTITY {
                    continue;
                }
                let Some(original) = decoded else { continue };
                self.recolored
                    .entry((bitmap_id, color_effect.cache_key()))
                    .or_insert_with(|| recolor(original, color_effect));
            }
        }
    }

    /// `matrix` maps bitmap pixels to shape coordinates (twips), as stored in the fill style.
    pub fn shader(
        &self,
        bitmap_id: CharacterId,
        matrix: &Matrix,
        is_repeating: bool,
        is_smoothed: bool,
        color_effect: ColorEffect,
    ) -> Option<Shader<'_>> {
        let pixmap = if color_effect == ColorEffect::IDENTITY {
            self.decoded.get(&bitmap_id)?.as_ref()?
        } else {
            self.recolored.get(&(bitmap_id, color_effect.cache_key()))?
        };
        let spread_mode = if is_repeating {
            SpreadMode::Repeat
        } else {
            SpreadMode::Pad
        };
        let quality = if is_smoothed {
            FilterQuality::Bilinear
        } else {
            FilterQuality::Nearest
        };
        Some(Pattern::new(
            pixmap.as_ref(),
            spread_mode,
            quality,
            FULL_OPACITY,
            to_skia_transform(matrix),
        ))
    }
}

fn recolor(original: &Pixmap, color_effect: ColorEffect) -> Pixmap {
    let mut recolored = original.clone();
    let (pixels, _) = recolored.data_mut().as_chunks_mut::<{ RgbaImage::BYTES_PER_PIXEL }>();
    for pixel in pixels {
        *pixel = color_effect.apply_to_premultiplied_pixel(*pixel);
    }
    recolored
}

fn bitmap_ids(styles: &ShapeStyles) -> impl Iterator<Item = CharacterId> + '_ {
    let fill_styles = styles.fill_styles.iter();
    let line_fill_styles = styles.line_styles.iter().map(swf::LineStyle::fill_style);
    fill_styles
        .chain(line_fill_styles)
        .filter_map(|fill_style| match fill_style {
            FillStyle::Bitmap { id: bitmap_id, .. } => Some(*bitmap_id),
            _ => None,
        })
}

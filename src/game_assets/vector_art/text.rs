//! Text fields (`DefineEditText`) drawn with their embedded fonts, enough for the game's HUD: one
//! line, the HTML subset Flash wrote (`<p align><font size color>`), left, center or right
//! alignment within the field's bounds.

use super::TWIPS_PER_PIXEL;
use swf::{EditText, FillStyle, Font, Rectangle, Shape, ShapeFlag, ShapeStyles, TextAlign, Twips};
use tiny_skia::Transform;

use super::bitmap_cache::BitmapCache;
use super::color_effect::ColorEffect;
use super::painter::{self, PaintTarget, UnsupportedFeatureCounts};
use html::paragraph_of;

mod html;

/// Flash keeps text 2 px inside the field on every side.
const GUTTER_TWIPS: f32 = 2.0 * TWIPS_PER_PIXEL;
/// `DefineFont3` glyphs use 20 units per twip (an em of 20480); older fonts 1 (an em of 1024).
const FONT3_EM_UNITS: f32 = 20480.0;
const FONT2_EM_UNITS: f32 = 1024.0;
const FONT3_VERSION: u8 = 3;
/// Without layout metrics, assume the usual ascent of 80 % of the em.
const DEFAULT_ASCENT_FRACTION: f32 = 0.8;

/// One glyph to draw: which (for caching its mesh), its outline as a one-color shape, and where.
pub struct LaidOutGlyph {
    pub key: GlyphKey,
    pub shape: Shape,
    pub transform: Transform,
}

/// A font's glyph in one color.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct GlyphKey {
    pub font_id: swf::CharacterId,
    pub glyph_index: usize,
    pub color: [u8; 4],
}

/// Draws `edit_text` (or `text_override`, keeping its formatting) with `font`.
pub fn paint_text(
    edit_text: &EditText,
    font: &Font,
    text_override: Option<&str>,
    transform: Transform,
    color_effect: ColorEffect,
    target: PaintTarget,
    unsupported: &mut UnsupportedFeatureCounts,
) {
    let PaintTarget {
        pixmap,
        minimum_stroke_pixels,
    } = target;
    let bitmaps = BitmapCache::default();
    lay_out_text(edit_text, font, text_override, transform, |glyph| {
        let mut target = PaintTarget {
            pixmap: &mut *pixmap,
            minimum_stroke_pixels,
        };
        painter::paint_shape(
            &glyph.shape,
            glyph.transform,
            color_effect,
            &mut target,
            &bitmaps,
            unsupported,
        );
    });
}

/// `edit_text`'s glyphs (or `text_override`'s, keeping its formatting) laid out in `font`.
pub fn lay_out_text(
    edit_text: &EditText,
    font: &Font,
    text_override: Option<&str>,
    transform: Transform,
    mut emit: impl FnMut(LaidOutGlyph),
) {
    let mut paragraph = paragraph_of(edit_text);
    if let Some(text) = text_override {
        paragraph.text = text.to_string();
    }
    let em_units = if font.version >= FONT3_VERSION {
        FONT3_EM_UNITS
    } else {
        FONT2_EM_UNITS
    };
    let scale = paragraph.size_pixels * TWIPS_PER_PIXEL / em_units;
    let glyphs: Vec<(usize, &swf::Glyph)> = paragraph
        .text
        .chars()
        .filter_map(|character| {
            font.glyphs
                .iter()
                .enumerate()
                .find(|(_, glyph)| u32::from(glyph.code) == u32::from(character))
        })
        .collect();
    let width: f32 = glyphs.iter().map(|(_, glyph)| f32::from(glyph.advance) * scale).sum();
    let bounds = edit_text.bounds();
    let ascent = font
        .layout
        .as_ref()
        .map_or(em_units * DEFAULT_ASCENT_FRACTION, |layout| f32::from(layout.ascent));
    let mut x = line_start(bounds, width, paragraph.align);
    let baseline = bounds.y_min.get() as f32 + GUTTER_TWIPS + ascent * scale;
    for (glyph_index, glyph) in glyphs {
        emit(LaidOutGlyph {
            key: glyph_key(font, glyph_index, paragraph.color),
            shape: glyph_shape(glyph, paragraph.color),
            transform: transform.pre_translate(x, baseline).pre_scale(scale, scale),
        });
        x += f32::from(glyph.advance) * scale;
    }
}

/// Static text (`DefineText`): glyph runs already laid out by the authoring tool; `font_for` finds
/// the fonts the runs name.
pub fn paint_static_text<'font>(
    text: &swf::Text,
    font_for: impl Fn(swf::CharacterId) -> Option<&'font Font<'font>>,
    transform: Transform,
    color_effect: ColorEffect,
    target: PaintTarget,
    unsupported: &mut UnsupportedFeatureCounts,
) {
    let PaintTarget {
        pixmap,
        minimum_stroke_pixels,
    } = target;
    let bitmaps = BitmapCache::default();
    let mut glyphs = Vec::new();
    lay_out_static_text(text, font_for, transform, unsupported, |glyph| glyphs.push(glyph));
    for glyph in glyphs {
        let mut target = PaintTarget {
            pixmap: &mut *pixmap,
            minimum_stroke_pixels,
        };
        painter::paint_shape(
            &glyph.shape,
            glyph.transform,
            color_effect,
            &mut target,
            &bitmaps,
            unsupported,
        );
    }
}

/// Static text's glyphs as its runs place them; runs without a known font count as unsupported.
pub fn lay_out_static_text<'font>(
    text: &swf::Text,
    font_for: impl Fn(swf::CharacterId) -> Option<&'font Font<'font>>,
    transform: Transform,
    unsupported: &mut UnsupportedFeatureCounts,
    mut emit: impl FnMut(LaidOutGlyph),
) {
    let text_transform = transform.pre_concat(super::painter::to_skia_transform(&text.matrix));
    let (mut font, mut color, mut height) = (None, swf::Color::BLACK, 0.0);
    let (mut x, mut y) = (0.0, 0.0);
    for record in &text.records {
        font = record.font_id.and_then(&font_for).or(font);
        color = record.color.unwrap_or(color);
        height = record.height.map_or(height, |twips| twips.get() as f32);
        x = record.x_offset.map_or(x, |twips| twips.get() as f32);
        y = record.y_offset.map_or(y, |twips| twips.get() as f32);
        let Some(font) = font else {
            unsupported.other_characters += 1;
            continue;
        };
        let em_units = if font.version >= FONT3_VERSION {
            FONT3_EM_UNITS
        } else {
            FONT2_EM_UNITS
        };
        let scale = height / em_units;
        for entry in &record.glyphs {
            if let Some(glyph) = font.glyphs.get(entry.index as usize) {
                emit(LaidOutGlyph {
                    key: glyph_key(font, entry.index as usize, color),
                    shape: glyph_shape(glyph, color),
                    transform: text_transform.pre_translate(x, y).pre_scale(scale, scale),
                });
            }
            x += entry.advance as f32;
        }
    }
}

fn line_start(bounds: &Rectangle<Twips>, width: f32, align: TextAlign) -> f32 {
    let left = bounds.x_min.get() as f32 + GUTTER_TWIPS;
    let right = bounds.x_max.get() as f32 - GUTTER_TWIPS;
    match align {
        TextAlign::Center => (left + right - width) / 2.0,
        TextAlign::Right => right - width,
        TextAlign::Left | TextAlign::Justify => left,
    }
}

fn glyph_key(font: &Font, glyph_index: usize, color: swf::Color) -> GlyphKey {
    GlyphKey {
        font_id: font.id,
        glyph_index,
        color: [color.r, color.g, color.b, color.a],
    }
}

/// A glyph as a one-color shape (glyph outlines use fill style 1).
fn glyph_shape(glyph: &swf::Glyph, color: swf::Color) -> Shape {
    Shape {
        version: 1,
        id: 0,
        shape_bounds: Rectangle::default(),
        edge_bounds: Rectangle::default(),
        flags: ShapeFlag::empty(),
        styles: ShapeStyles {
            fill_styles: vec![FillStyle::Color(color)],
            line_styles: Vec::new(),
        },
        shape: glyph.shape_records.clone(),
    }
}

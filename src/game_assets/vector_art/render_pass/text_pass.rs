//! Text in a render pass: drawn into pixels, or listed glyph by glyph for meshes.

use swf::EditText;
use tiny_skia::Transform;

use super::{PlacedShape, RenderPass, ShapeSource, Target};
use crate::game_assets::vector_art::color_effect::ColorEffect;
use crate::game_assets::vector_art::text::LaidOutGlyph;
use crate::game_assets::vector_art::{painter, text};

impl RenderPass<'_, '_> {
    pub(super) fn render_static_text(
        &mut self,
        static_text: &swf::Text,
        transform: Transform,
        color_effect: ColorEffect,
        target: &mut Target,
    ) {
        let library = self.library;
        let Target::Pixmap(pixmap) = target else {
            let Target::Placements(placements) = target else { return };
            let font_for = |font_id| library.font(font_id);
            text::lay_out_static_text(static_text, font_for, transform, self.unsupported, |glyph| {
                placements.push(placed_glyph(glyph, color_effect));
            });
            return;
        };
        let paint_target = painter::PaintTarget {
            pixmap,
            minimum_stroke_pixels: self.device_pixels_per_output_pixel,
        };
        text::paint_static_text(
            static_text,
            |font_id| library.font(font_id),
            transform,
            color_effect,
            paint_target,
            self.unsupported,
        );
    }

    pub(super) fn render_text(
        &mut self,
        edit_text: &EditText,
        transform: Transform,
        color_effect: ColorEffect,
        target: &mut Target,
    ) {
        let Some(font) = edit_text.font_id().and_then(|font_id| self.library.font(font_id)) else {
            self.unsupported.other_characters += 1;
            return;
        };
        let variable = edit_text.variable_name().to_str_lossy(swf::UTF_8);
        let variable_name = variable.rsplit(['.', ':', '/']).next().unwrap_or_default();
        let text_override = self
            .overrides
            .texts
            .iter()
            .find(|(name, _)| !variable_name.is_empty() && *name == variable_name)
            .map(|&(_, text)| text);
        let pixmap = match target {
            Target::Pixmap(pixmap) => pixmap,
            Target::Placements(placements) => {
                text::lay_out_text(edit_text, font, text_override, transform, |glyph| {
                    placements.push(placed_glyph(glyph, color_effect));
                });
                return;
            }
        };
        let target = painter::PaintTarget {
            pixmap,
            minimum_stroke_pixels: self.device_pixels_per_output_pixel,
        };
        text::paint_text(
            edit_text,
            font,
            text_override,
            transform,
            color_effect,
            target,
            self.unsupported,
        );
    }
}

fn placed_glyph(glyph: LaidOutGlyph, color_effect: ColorEffect) -> PlacedShape {
    PlacedShape {
        source: ShapeSource::Glyph(glyph.key, Box::new(glyph.shape)),
        transform: glyph.transform,
        color_effect,
    }
}

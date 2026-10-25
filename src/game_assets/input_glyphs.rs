//! Button pictures for the on-screen hints: Kenney's CC0 "Input Prompts" (`assets/input-glyphs/kenney`,
//! with its licence and source). Each picture is one SVG with a single flat-filled path that uses only
//! the move, line and quadratic-curve commands. A hint is a few flat-colored layers (the glyph, maybe
//! a backdrop) in world units around the texture's center; frontends that draw vectors mesh them
//! (`shape`), the others fill them into an image ([`render_layers`]).

mod pictures;
mod shape;
mod svg_path;

use glam::Vec2;
use tiny_skia::{FillRule, Paint, Path, Pixmap, Stroke, Transform};

use super::vector_art::{RenderError, RgbaImage};
use crate::game::{InputDevice, TextureId};
use pictures::{left_trigger, right_bumper, start_room_glyphs};
pub use shape::{MESH_UNITS_PER_WORLD_UNIT, layers_to_shape};
use svg_path::paths_of;

/// Empty space kept around the picture inside its texture, in world units.
const MARGIN: f32 = 1.0;
/// The ink of the start room drawing's lines and lettering as it looks on the floor (#2B2220).
const START_ROOM_INK: [u8; 3] = [43, 34, 32];
/// The HUD's label above the active item's box, like its "SPACE": light grey lettering (#CCCCCC)
/// in a thick black outline, on nothing.
const LABEL_INK: [u8; 3] = [204, 204, 204];
const LABEL_OUTLINE_COLOR: [u8; 3] = [0, 0, 0];
/// Width of the outline's stroke, centered on the picture's edges (half of it shows outside, about
/// as thick as the black around "SPACE").
const LABEL_OUTLINE_WIDTH: f32 = 4.0;
/// Kenney's filled D-pad marks the pressed arm in a second colour, which one ink would lose; the arm
/// is outlined instead, as thick as the pack's outlined pictures draw their lines (in SVG units), so
/// the pad reads like Rebirth's hint: solid, the pressed arm hollow.
const HIGHLIGHT_OUTLINE_WIDTH: f32 = 3.0;

/// One flat-colored shape of a hint, in world units from the texture's center (y down).
pub struct GlyphLayer {
    pub path: Path,
    pub color: [u8; 3],
}

/// The layers of `texture_id`, back to front, if it is a pad's button picture made here.
pub fn glyph_layers(texture_id: TextureId) -> Option<Result<Vec<GlyphLayer>, RenderError>> {
    let size = texture_id.world_size();
    match texture_id {
        TextureId::StartRoomHint(control, InputDevice::Gamepad(kind)) => {
            Some(glyph_row(&start_room_glyphs(control, kind), size))
        }
        TextureId::ActiveItemLabel(InputDevice::Gamepad(kind)) => Some(outlined_label(left_trigger(kind), size)),
        TextureId::HeldCardLabel(InputDevice::Gamepad(kind)) => Some(outlined_label(right_bumper(kind), size)),
        _ => None,
    }
}

/// The glyphs side by side, each as big as fits an equal share of the width.
fn glyph_row(svgs: &[&str], size: Vec2) -> Result<Vec<GlyphLayer>, RenderError> {
    let cell = Vec2::new(size.x / svgs.len() as f32, size.y);
    let mut layers = Vec::new();
    for (index, svg) in svgs.iter().enumerate() {
        let cells_before_middle = index as f32 + 0.5;
        let cell_center = -size.x / 2.0 + cell.x * cells_before_middle;
        for layer in fitted(picture_paths(svg)?, cell, MARGIN, START_ROOM_INK)? {
            let path = layer
                .path
                .transform(Transform::from_translate(cell_center, 0.0))
                .ok_or(RenderError::InvalidGlyph)?;
            layers.push(GlyphLayer { path, ..layer });
        }
    }
    Ok(layers)
}

/// The glyph in the label's ink over the same shape stroked black, so it reads like "SPACE".
fn outlined_label(svg: &str, size: Vec2) -> Result<Vec<GlyphLayer>, RenderError> {
    let glyphs = fitted(picture_paths(svg)?, size, LABEL_OUTLINE_WIDTH / 2.0, LABEL_INK)?;
    let stroke = Stroke {
        width: LABEL_OUTLINE_WIDTH,
        ..Stroke::default()
    };
    let mut layers = Vec::new();
    for glyph in &glyphs {
        layers.push(GlyphLayer {
            path: glyph.path.stroke(&stroke, 1.0).ok_or(RenderError::InvalidGlyph)?,
            color: LABEL_OUTLINE_COLOR,
        });
    }
    layers.extend(glyphs);
    Ok(layers)
}

/// The picture's shapes in SVG units: its plain parts filled, a part the pack colours differently
/// (the pressed arm) outlined.
fn picture_paths(svg: &str) -> Result<Vec<Path>, RenderError> {
    let stroke = Stroke {
        width: HIGHLIGHT_OUTLINE_WIDTH,
        ..Stroke::default()
    };
    paths_of(svg)
        .ok_or(RenderError::InvalidGlyph)?
        .into_iter()
        .map(|svg_path| {
            if svg_path.highlighted {
                svg_path.path.stroke(&stroke, 1.0).ok_or(RenderError::InvalidGlyph)
            } else {
                Ok(svg_path.path)
            }
        })
        .collect()
}

/// The paths together as big as fits `size` world units (less `margin` on every side), centered.
fn fitted(paths: Vec<Path>, size: Vec2, margin: f32, color: [u8; 3]) -> Result<Vec<GlyphLayer>, RenderError> {
    let bounds = paths
        .iter()
        .map(Path::bounds)
        .reduce(|first, second| {
            tiny_skia::Rect::from_ltrb(
                first.left().min(second.left()),
                first.top().min(second.top()),
                first.right().max(second.right()),
                first.bottom().max(second.bottom()),
            )
            .unwrap_or(first)
        })
        .ok_or(RenderError::InvalidGlyph)?;
    let scale = ((size.x - 2.0 * margin) / bounds.width()).min((size.y - 2.0 * margin) / bounds.height());
    let center = (bounds.x() + bounds.width() / 2.0, bounds.y() + bounds.height() / 2.0);
    let centered = Transform::from_row(scale, 0.0, 0.0, scale, -center.0 * scale, -center.1 * scale);
    paths
        .into_iter()
        .map(|path| {
            let path = path.transform(centered).ok_or(RenderError::InvalidGlyph)?;
            Ok(GlyphLayer { path, color })
        })
        .collect()
}

/// Fills `layers` into an image covering `size` world units at `pixels_per_world_unit`.
pub fn render_layers(layers: &[GlyphLayer], size: Vec2, pixels_per_world_unit: f32) -> Result<RgbaImage, RenderError> {
    let (width, height) = (
        (size.x * pixels_per_world_unit).round(),
        (size.y * pixels_per_world_unit).round(),
    );
    let mut pixmap = Pixmap::new(
        crate::whole_numbers::clamped_u32(width),
        crate::whole_numbers::clamped_u32(height),
    )
    .ok_or(RenderError::InvalidCanvasSize)?;
    let to_pixels = Transform::from_row(
        pixels_per_world_unit,
        0.0,
        0.0,
        pixels_per_world_unit,
        width / 2.0,
        height / 2.0,
    );
    for layer in layers {
        let mut paint = Paint::default();
        let [red, green, blue] = layer.color;
        paint.set_color_rgba8(red, green, blue, u8::MAX);
        pixmap.fill_path(&layer.path, &paint, FillRule::Winding, to_pixels, None);
    }
    Ok(RgbaImage {
        width: pixmap.width(),
        height: pixmap.height(),
        pixels: pixmap.take_demultiplied(),
    })
}

#[cfg(test)]
mod hint_tests;
#[cfg(test)]
mod tests;

//! Paints rebuilt shape paths onto a `tiny-skia` pixmap. All geometry stays in twips; `transform`
//! maps twips to output pixels.

use swf::{FillStyle, Gradient, GradientSpread, LineCapStyle, LineJoinStyle, LineStyle, Shape, ShapeFlag};
use tiny_skia::{
    FillRule, GradientStop, LineCap, LineJoin, LinearGradient, Paint, Path, PathBuilder, Pixmap, Point, RadialGradient,
    Shader, SpreadMode, Stroke, Transform,
};

use super::bitmap_cache::BitmapCache;
use super::color_effect::ColorEffect;
use super::shape_paths::{EdgeRun, style_groups};

/// Flash gradients span this square in gradient space before their matrix is applied.
const GRADIENT_SQUARE_HALF_SIZE_TWIPS: f32 = 16_384.0;
const GRADIENT_RATIO_MAXIMUM: f32 = 255.0;
const FOCAL_POINT_UNIT: f32 = 1.0;

/// Average linear scale of `transform`, i.e. how many output pixels one twip becomes.
fn device_scale(transform: Transform) -> f32 {
    (transform.sx * transform.sy - transform.kx * transform.ky).abs().sqrt()
}

/// Art the renderer skips for now; reported so missing detail is visible.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct UnsupportedFeatureCounts {
    /// Bitmap fills whose bitmap is missing or in an undecodable format.
    pub bitmap_fills: usize,
    /// Placed characters that are neither shapes nor sprites (text, morph shapes, bitmaps, ...).
    pub other_characters: usize,
}

impl UnsupportedFeatureCounts {
    /// Whether everything could be drawn.
    pub fn is_empty(self) -> bool {
        self == Self::default()
    }
}

/// Where shapes are painted, rasterized without anti-aliasing: `supersampling` averages the samples.
pub struct PaintTarget<'pixmap> {
    pub pixmap: &'pixmap mut Pixmap,
    /// Strokes are never drawn thinner than one output pixel, this many device pixels; width 0 is
    /// a hairline of that width too.
    pub minimum_stroke_pixels: f32,
}

pub fn paint_shape(
    shape: &Shape,
    transform: Transform,
    color_effect: ColorEffect,
    target: &mut PaintTarget,
    bitmaps: &BitmapCache,
    unsupported: &mut UnsupportedFeatureCounts,
) {
    let fill_rule = if shape.flags.contains(ShapeFlag::NON_ZERO_WINDING_RULE) {
        FillRule::Winding
    } else {
        FillRule::EvenOdd
    };
    for group in style_groups(shape) {
        for (fill_style, contours) in &group.fills {
            let (Some(path), Some(shader)) = (
                build_path(contours, true),
                shader_for(fill_style, color_effect, bitmaps, unsupported),
            ) else {
                continue;
            };
            target
                .pixmap
                .fill_path(&path, &aliased(shader), fill_rule, transform, None);
        }
        for (line_style, runs) in &group.strokes {
            let (Some(path), Some(shader)) = (
                build_path(runs, false),
                shader_for(line_style.fill_style(), color_effect, bitmaps, unsupported),
            ) else {
                continue;
            };
            let mut stroke = stroke_for(line_style);
            // Flash never draws strokes thinner than one screen pixel.
            let scale = device_scale(transform).max(f32::EPSILON);
            stroke.width = stroke.width.max(target.minimum_stroke_pixels / scale);
            target
                .pixmap
                .stroke_path(&path, &aliased(shader), &stroke, transform, None);
        }
    }
}

fn aliased(shader: Shader<'_>) -> Paint<'_> {
    Paint {
        shader,
        anti_alias: false,
        ..Paint::default()
    }
}

fn build_path(runs: &[EdgeRun], close_runs: bool) -> Option<Path> {
    let mut builder = PathBuilder::new();
    for run in runs {
        let Some(first_edge) = run.first() else { continue };
        builder.move_to(first_edge.start.0 as f32, first_edge.start.1 as f32);
        for edge in run {
            match edge.control {
                Some(control) => {
                    builder.quad_to(control.0 as f32, control.1 as f32, edge.end.0 as f32, edge.end.1 as f32);
                }
                None => builder.line_to(edge.end.0 as f32, edge.end.1 as f32),
            }
        }
        if close_runs {
            builder.close();
        }
    }
    builder.finish()
}

fn shader_for<'bitmaps>(
    fill_style: &FillStyle,
    color_effect: ColorEffect,
    bitmaps: &'bitmaps BitmapCache,
    unsupported: &mut UnsupportedFeatureCounts,
) -> Option<Shader<'bitmaps>> {
    let gradient_start = Point::from_xy(-GRADIENT_SQUARE_HALF_SIZE_TWIPS, 0.0);
    let gradient_end = Point::from_xy(GRADIENT_SQUARE_HALF_SIZE_TWIPS, 0.0);
    match fill_style {
        FillStyle::Color(color) => Some(Shader::SolidColor(color_effect.apply(*color))),
        FillStyle::LinearGradient(gradient) => LinearGradient::new(
            gradient_start,
            gradient_end,
            gradient_stops(gradient, color_effect),
            spread_mode(gradient.spread),
            gradient_transform(gradient),
        ),
        FillStyle::RadialGradient(gradient) => radial_gradient(gradient, 0.0, color_effect),
        FillStyle::FocalGradient { gradient, focal_point } => {
            radial_gradient(gradient, focal_point.to_f32(), color_effect)
        }
        FillStyle::Bitmap {
            id: bitmap_id,
            matrix,
            is_smoothed,
            is_repeating,
        } => {
            let shader = bitmaps.shader(*bitmap_id, matrix, *is_repeating, *is_smoothed, color_effect);
            if shader.is_none() {
                unsupported.bitmap_fills += 1;
            }
            shader
        }
    }
}

/// `focal_point` is the focus position along the x axis, from -1 to 1 of the radius.
fn radial_gradient(gradient: &Gradient, focal_point: f32, color_effect: ColorEffect) -> Option<Shader<'static>> {
    let focus = Point::from_xy(
        focal_point.clamp(-FOCAL_POINT_UNIT, FOCAL_POINT_UNIT) * GRADIENT_SQUARE_HALF_SIZE_TWIPS,
        0.0,
    );
    RadialGradient::new(
        focus,
        0.0,
        Point::zero(),
        GRADIENT_SQUARE_HALF_SIZE_TWIPS,
        gradient_stops(gradient, color_effect),
        spread_mode(gradient.spread),
        gradient_transform(gradient),
    )
}

fn gradient_stops(gradient: &Gradient, color_effect: ColorEffect) -> Vec<GradientStop> {
    gradient
        .records
        .iter()
        .map(|record| {
            GradientStop::new(
                f32::from(record.ratio) / GRADIENT_RATIO_MAXIMUM,
                color_effect.apply(record.color),
            )
        })
        .collect()
}

fn gradient_transform(gradient: &Gradient) -> Transform {
    to_skia_transform(&gradient.matrix)
}

fn spread_mode(spread: GradientSpread) -> SpreadMode {
    match spread {
        GradientSpread::Pad => SpreadMode::Pad,
        GradientSpread::Reflect => SpreadMode::Reflect,
        GradientSpread::Repeat => SpreadMode::Repeat,
    }
}

fn stroke_for(line_style: &LineStyle) -> Stroke {
    let mut stroke = Stroke {
        width: line_style.width().get() as f32,
        line_cap: match line_style.start_cap() {
            LineCapStyle::Round => LineCap::Round,
            LineCapStyle::None => LineCap::Butt,
            LineCapStyle::Square => LineCap::Square,
        },
        ..Stroke::default()
    };
    match line_style.join_style() {
        LineJoinStyle::Round => stroke.line_join = LineJoin::Round,
        LineJoinStyle::Bevel => stroke.line_join = LineJoin::Bevel,
        LineJoinStyle::Miter(miter_limit) => {
            stroke.line_join = LineJoin::Miter;
            stroke.miter_limit = miter_limit.to_f32();
        }
    }
    stroke
}

pub fn to_skia_transform(matrix: &swf::Matrix) -> Transform {
    Transform::from_row(
        matrix.a.to_f32(),
        matrix.b.to_f32(),
        matrix.c.to_f32(),
        matrix.d.to_f32(),
        matrix.tx.get() as f32,
        matrix.ty.get() as f32,
    )
}

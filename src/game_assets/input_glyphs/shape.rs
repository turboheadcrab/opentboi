//! A hint's layers as a Flash shape, so the same tessellator that meshes the SWF's own shapes can
//! mesh them (see `vector_mesh`).

use swf::{
    Color, FillStyle, Point, PointDelta, Rectangle, Shape, ShapeFlag, ShapeRecord, ShapeStyles, StyleChangeData, Twips,
};
use tiny_skia::{PathSegment, Point as SkiaPoint};

use super::GlyphLayer;

/// Mesh coordinates are world units times this, so twips (a twentieth of a pixel) keep curves smooth
/// when the hint is drawn many times larger than its world size.
pub const MESH_UNITS_PER_WORLD_UNIT: f32 = 16.0;
const SHAPE_VERSION: u8 = 4;
/// The shape has no character of its own.
const NO_CHARACTER: u16 = 0;

/// Smallest and largest x, then y, of the points so far.
type Bounds = Option<(Twips, Twips, Twips, Twips)>;

pub fn layers_to_shape(layers: &[GlyphLayer]) -> Shape {
    let mut records = Vec::new();
    let mut bounds = Bounds::None;
    for (layer, fill_style) in layers.iter().zip(1..) {
        append_layer(layer, fill_style, &mut records, &mut bounds);
    }
    let (x_min, x_max, y_min, y_max) = bounds.unwrap_or_default();
    let shape_bounds = Rectangle {
        x_min,
        x_max,
        y_min,
        y_max,
    };
    Shape {
        version: SHAPE_VERSION,
        id: NO_CHARACTER,
        shape_bounds,
        edge_bounds: shape_bounds,
        flags: ShapeFlag::empty(),
        styles: ShapeStyles {
            fill_styles: layers
                .iter()
                .map(|layer| {
                    let [red, green, blue] = layer.color;
                    FillStyle::Color(Color {
                        r: red,
                        g: green,
                        b: blue,
                        a: u8::MAX,
                    })
                })
                .collect(),
            line_styles: Vec::new(),
        },
        shape: records,
    }
}

/// The layer's outlines, filled with the style numbered `fill_style`.
fn append_layer(layer: &GlyphLayer, fill_style: u32, records: &mut Vec<ShapeRecord>, bounds: &mut Bounds) {
    let mut subpath = Subpath::default();
    for segment in layer.path.segments() {
        match segment {
            PathSegment::MoveTo(point) => {
                subpath.close(records);
                let start = twips_point(point);
                records.push(ShapeRecord::StyleChange(Box::new(StyleChangeData {
                    move_to: Some(start),
                    fill_style_0: None,
                    fill_style_1: Some(fill_style),
                    line_style: None,
                    new_styles: None,
                })));
                subpath = Subpath::starting_at(start);
            }
            PathSegment::LineTo(point) => subpath.line_to(twips_point(point), records, bounds),
            PathSegment::QuadTo(control, point) => {
                subpath.curve_to(twips_point(control), twips_point(point), records, bounds);
            }
            PathSegment::CubicTo(..) => {}
            PathSegment::Close => subpath.close(records),
        }
    }
    subpath.close(records);
}

fn twips_point(point: SkiaPoint) -> Point<Twips> {
    Point::from_pixels(
        f64::from(point.x * MESH_UNITS_PER_WORLD_UNIT),
        f64::from(point.y * MESH_UNITS_PER_WORLD_UNIT),
    )
}

/// The edges of one outline so far; Flash fills close an open outline, and so does this.
#[derive(Default)]
struct Subpath {
    start: Option<Point<Twips>>,
    current: Option<Point<Twips>>,
}

impl Subpath {
    fn starting_at(start: Point<Twips>) -> Self {
        Self {
            start: Some(start),
            current: Some(start),
        }
    }

    fn line_to(&mut self, point: Point<Twips>, records: &mut Vec<ShapeRecord>, bounds: &mut Bounds) {
        let Some(from) = self.current else { return };
        records.push(ShapeRecord::StraightEdge {
            delta: delta(from, point),
        });
        grow(bounds, point);
        self.current = Some(point);
    }

    fn curve_to(
        &mut self,
        control: Point<Twips>,
        point: Point<Twips>,
        records: &mut Vec<ShapeRecord>,
        bounds: &mut Bounds,
    ) {
        let Some(from) = self.current else { return };
        records.push(ShapeRecord::CurvedEdge {
            control_delta: delta(from, control),
            anchor_delta: delta(control, point),
        });
        grow(bounds, control);
        grow(bounds, point);
        self.current = Some(point);
    }

    fn close(&mut self, records: &mut Vec<ShapeRecord>) {
        if let (Some(start), Some(current)) = (self.start, self.current)
            && start != current
        {
            records.push(ShapeRecord::StraightEdge {
                delta: delta(current, start),
            });
        }
        *self = Self::default();
    }
}

fn delta(from: Point<Twips>, target: Point<Twips>) -> PointDelta<Twips> {
    PointDelta::new(
        Twips::new(target.x.get() - from.x.get()),
        Twips::new(target.y.get() - from.y.get()),
    )
}

fn grow(bounds: &mut Bounds, point: Point<Twips>) {
    let (x_min, x_max, y_min, y_max) = bounds.unwrap_or((point.x, point.x, point.y, point.y));
    *bounds = Some((
        x_min.min(point.x),
        x_max.max(point.x),
        y_min.min(point.y),
        y_max.max(point.y),
    ));
}

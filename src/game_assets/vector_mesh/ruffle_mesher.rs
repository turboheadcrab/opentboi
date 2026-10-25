//! [`ShapeMesher`] backed by Ruffle's tessellator (`ruffle_render::tessellator`, built on lyon),
//! converting its meshes into ours.

use ruffle_render::backend::RenderBackend;
use ruffle_render::bitmap::{BitmapHandle, BitmapSize, BitmapSource};
use ruffle_render::shape_utils::{DistilledShape, GradientType};
use ruffle_render::tessellator::{self, DrawType, ShapeTessellator};

use super::{Gradient, GradientShape, GradientSpread, MeshDraw, MeshFill, MeshVertex, ShapeMesh, ShapeMesher};
use crate::game_assets::vector_art::CharacterLibrary;

const FIXED8_SCALE: f32 = 256.0;

pub struct RuffleMesher<'library> {
    tessellator: ShapeTessellator,
    bitmaps: Bitmaps<'library>,
}

impl<'library> RuffleMesher<'library> {
    pub fn new(library: &'library CharacterLibrary<'library>) -> Self {
        Self {
            tessellator: ShapeTessellator::new(),
            bitmaps: Bitmaps { library },
        }
    }
}

impl ShapeMesher for RuffleMesher<'_> {
    fn mesh(&mut self, shape: &swf::Shape) -> ShapeMesh {
        let mesh = self
            .tessellator
            .tessellate_shape(DistilledShape::from(shape), &self.bitmaps);
        let draws = mesh
            .draws
            .into_iter()
            .map(|draw| MeshDraw {
                fill: fill(draw.draw_type, &mesh.gradients),
                vertices: draw
                    .vertices
                    .iter()
                    .map(|vertex| MeshVertex {
                        position: [vertex.x, vertex.y],
                        color: [vertex.color.r, vertex.color.g, vertex.color.b, vertex.color.a],
                    })
                    .collect(),
                indices: draw.indices,
            })
            .collect();
        ShapeMesh { draws }
    }
}

fn fill(draw_type: DrawType, gradients: &[tessellator::Gradient]) -> MeshFill {
    match draw_type {
        DrawType::Color => MeshFill::Solid,
        DrawType::Gradient { matrix, gradient } => MeshFill::Gradient {
            matrix,
            gradient: gradients.get(gradient).map_or_else(
                || Gradient {
                    shape: GradientShape::Linear,
                    spread: GradientSpread::Pad,
                    focal_point: 0.0,
                    linear_rgb: false,
                    stops: Vec::new(),
                },
                convert_gradient,
            ),
        },
        DrawType::Bitmap(bitmap) => MeshFill::Bitmap {
            matrix: bitmap.matrix,
            bitmap_id: bitmap.bitmap_id,
            smoothed: bitmap.is_smoothed,
            repeating: bitmap.is_repeating,
        },
    }
}

fn convert_gradient(gradient: &tessellator::Gradient) -> Gradient {
    Gradient {
        shape: match gradient.gradient_type {
            GradientType::Linear => GradientShape::Linear,
            GradientType::Radial => GradientShape::Radial,
            GradientType::Focal => GradientShape::Focal,
        },
        spread: match gradient.repeat_mode {
            swf::GradientSpread::Pad => GradientSpread::Pad,
            swf::GradientSpread::Reflect => GradientSpread::Reflect,
            swf::GradientSpread::Repeat => GradientSpread::Repeat,
        },
        focal_point: f32::from(gradient.focal_point.get()) / FIXED8_SCALE,
        linear_rgb: gradient.interpolation == swf::GradientInterpolation::LinearRgb,
        stops: gradient
            .records
            .iter()
            .map(|record| {
                (
                    record.ratio,
                    [record.color.r, record.color.g, record.color.b, record.color.a],
                )
            })
            .collect(),
    }
}

/// Bitmap sizes for fills; handles are the renderer's business, so none are given.
struct Bitmaps<'library> {
    library: &'library CharacterLibrary<'library>,
}

impl BitmapSource for Bitmaps<'_> {
    fn bitmap_size(&self, id: u16) -> Option<BitmapSize> {
        let bitmap = self.library.bitmap(id)?;
        Some(BitmapSize {
            width: u32::from(bitmap.width),
            height: u32::from(bitmap.height),
        })
    }

    fn bitmap_handle(&self, _id: u16, _renderer: &mut dyn RenderBackend) -> Option<BitmapHandle> {
        None
    }
}

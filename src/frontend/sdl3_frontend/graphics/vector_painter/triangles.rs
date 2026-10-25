//! The triangles of the pictures queued for one layer, in off-screen pixels, grouped into draw
//! calls by bitmap.

use std::ops::Range;

use sdl3::pixels::FColor;
use sdl3::render::{FPoint, Vertex};

use super::bitmap_textures::BitmapKey;
use crate::game_assets::{MeshFill, VectorPicture};

const NO_BITMAP: [[f32; 3]; 3] = [[0.0; 3]; 3];

/// Triangles drawn in one call: `indices` into the frame's vertices, with a bitmap or plain colors.
pub struct Batch {
    pub bitmap: Option<BitmapKey>,
    pub indices: Range<usize>,
}

/// Where a picture lands in off-screen pixels: its center, scale (negative x mirrors it) and
/// clockwise rotation as cosine and sine.
#[derive(Clone, Copy)]
pub struct Placement {
    pub center: (f32, f32),
    pub scale: (f32, f32),
    pub rotation: (f32, f32),
}

impl Placement {
    /// A point in the picture's Flash pixels (y down, from its center) in off-screen pixels.
    pub fn apply(self, [x, y]: [f32; 2]) -> (f32, f32) {
        let (scaled_x, scaled_y) = (x * self.scale.0, y * self.scale.1);
        let (cosine, sine) = self.rotation;
        (
            self.center.0 + scaled_x * cosine - scaled_y * sine,
            self.center.1 + scaled_x * sine + scaled_y * cosine,
        )
    }
}

#[derive(Default)]
pub struct Triangles {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub batches: Vec<Batch>,
}

impl Triangles {
    pub fn clear(&mut self) {
        self.vertices.clear();
        self.indices.clear();
        self.batches.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.batches.is_empty()
    }

    /// Adds `picture` at `placement` on top of what is queued.
    pub fn add_picture(&mut self, picture: &VectorPicture, placement: Placement) {
        for part in &picture.parts {
            for draw in &part.mesh.draws {
                let MeshFill::Bitmap { bitmap_id, matrix, .. } = draw.fill else {
                    let colors = draw.vertices.iter().map(|vertex| part.color(vertex.color));
                    self.add(part, draw, placement, colors, (None, NO_BITMAP));
                    continue;
                };
                let (multiply, add) = part.bitmap_tint();
                let plain = BitmapKey {
                    bitmap_id,
                    silhouette: false,
                };
                self.add(
                    part,
                    draw,
                    placement,
                    std::iter::repeat(multiply),
                    (Some(plain), matrix),
                );
                if add.iter().any(|&channel| channel > 0.0) {
                    let silhouette = BitmapKey {
                        bitmap_id,
                        silhouette: true,
                    };
                    self.add(
                        part,
                        draw,
                        placement,
                        std::iter::repeat(add),
                        (Some(silhouette), matrix),
                    );
                }
            }
        }
    }

    /// One draw's triangles with the given vertex colors; bitmap fills map mesh pixels to the
    /// bitmap's 0–1 coordinates with the column-major `matrix`.
    fn add(
        &mut self,
        part: &crate::game_assets::VectorPart,
        draw: &crate::game_assets::MeshDraw,
        placement: Placement,
        colors: impl Iterator<Item = [f32; 4]>,
        (bitmap, matrix): (Option<BitmapKey>, [[f32; 3]; 3]),
    ) {
        let first_vertex = u32::try_from(self.vertices.len()).expect("a frame has fewer than 2^32 vertices");
        self.vertices.extend(
            draw.vertices
                .iter()
                .zip(colors)
                .map(|(vertex, [red, green, blue, alpha])| {
                    let (x, y) = placement.apply(part.place(vertex.position));
                    let [mesh_x, mesh_y] = vertex.position;
                    Vertex {
                        position: FPoint::new(x, y),
                        color: FColor::RGBA(red, green, blue, alpha),
                        tex_coord: FPoint::new(
                            matrix[0][0] * mesh_x + matrix[1][0] * mesh_y + matrix[2][0],
                            matrix[0][1] * mesh_x + matrix[1][1] * mesh_y + matrix[2][1],
                        ),
                    }
                }),
        );
        let start = self.indices.len();
        self.indices
            .extend(draw.indices.iter().map(|index| first_vertex + index));
        match self.batches.last_mut() {
            Some(batch) if batch.bitmap == bitmap => batch.indices.end = self.indices.len(),
            _ => self.batches.push(Batch {
                bitmap,
                indices: start..self.indices.len(),
            }),
        }
    }
}

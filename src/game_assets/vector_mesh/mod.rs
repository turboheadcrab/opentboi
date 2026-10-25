//! Flash shapes as triangles, for drawing vectors on the GPU instead of rasterizing every frame
//! (see `docs/vector-rendering.md`). The mesh types here are ours; making them is behind
//! [`ShapeMesher`], so Ruffle's tessellator ([`RuffleMesher`]) can be swapped for our own (lyon)
//! without touching the frontends.

mod ruffle_mesher;

pub use ruffle_mesher::RuffleMesher;

/// One shape, ready to draw: its fills and strokes in drawing order.
#[derive(Clone, Debug, Default)]
pub struct ShapeMesh {
    pub draws: Vec<MeshDraw>,
}

/// Triangles that share one fill.
#[derive(Clone, Debug)]
pub struct MeshDraw {
    pub fill: MeshFill,
    pub vertices: Vec<MeshVertex>,
    /// Three per triangle, into `vertices`.
    pub indices: Vec<u32>,
}

/// A corner, in the shape's own Flash pixels (y down), with its color (premultiplied: no;
/// straight RGBA) for solid fills.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MeshVertex {
    pub position: [f32; 2],
    pub color: [u8; 4],
}

/// How the triangles are colored. Matrices map shape pixels to the fill's own space (gradient
/// square or bitmap pixels), as 3×3 column-major.
#[derive(Clone, Debug)]
pub enum MeshFill {
    /// Each vertex's color.
    Solid,
    Gradient {
        matrix: [[f32; 3]; 3],
        gradient: Gradient,
    },
    Bitmap {
        matrix: [[f32; 3]; 3],
        bitmap_id: u16,
        smoothed: bool,
        repeating: bool,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GradientShape {
    Linear,
    Radial,
    /// Radial with its focus moved along the x axis by `focal_point` (-1 to 1).
    Focal,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GradientSpread {
    Pad,
    Reflect,
    Repeat,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Gradient {
    pub shape: GradientShape,
    pub spread: GradientSpread,
    pub focal_point: f32,
    /// Interpolate in linear RGB rather than sRGB.
    pub linear_rgb: bool,
    /// Ratio (0–255) and straight RGBA color of each stop.
    pub stops: Vec<(u8, [u8; 4])>,
}

/// Turns shapes into meshes.
pub trait ShapeMesher {
    fn mesh(&mut self, shape: &swf::Shape) -> ShapeMesh;
}

#[cfg(test)]
mod tests;

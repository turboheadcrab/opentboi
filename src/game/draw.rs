use glam::Vec2;

use crate::game::texture_id::TextureId;

/// sRGB color with alpha, each channel in `0.0..=1.0`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rgba {
    pub red: f32,
    pub green: f32,
    pub blue: f32,
    pub alpha: f32,
}

impl Rgba {
    /// As a tint, it leaves what it tints as it is.
    pub const WHITE: Self = Self::opaque(1.0, 1.0, 1.0);

    pub const fn opaque(red: f32, green: f32, blue: f32) -> Self {
        Self {
            red,
            green,
            blue,
            alpha: 1.0,
        }
    }
}

/// Items drawn together into one picture first, which is then shown at `opacity` (a Flash bitmap
/// with an `_alpha`, such as the floor's splat bitmap): where they overlap, they cover each other
/// instead of adding up.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DrawLayer {
    pub opacity: f32,
}

/// One axis-aligned rectangle to draw this frame, in world units (origin at the room center, y up).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DrawItem {
    pub center: Vec2,
    pub size: Vec2,
    /// Fill color; for textured items it is only used when the texture is unavailable.
    pub color: Rgba,
    pub texture: Option<TextureId>,
    /// Multiplies the texture's colors and opacity (a Flash color transform's multipliers); white
    /// draws it as it is.
    pub tint: Rgba,
    /// The layer it is drawn into; consecutive items (by depth) of the same layer go together.
    pub layer: Option<DrawLayer>,
    /// Counter-clockwise rotation around `center`.
    pub rotation_radians: f32,
    /// Mirror the texture left-to-right around `center`.
    pub flip_horizontally: bool,
    /// Larger values are drawn on top.
    pub depth: f32,
    /// How far it moved during the last tick; between ticks it is drawn that much further back.
    pub motion: Vec2,
}

impl DrawItem {
    pub fn solid(center: Vec2, size: Vec2, color: Rgba, depth: f32) -> Self {
        Self {
            center,
            size,
            color,
            texture: None,
            tint: Rgba::WHITE,
            layer: None,
            rotation_radians: 0.0,
            flip_horizontally: false,
            depth,
            motion: Vec2::ZERO,
        }
    }

    /// `texture` at `scale` times its world size; `fallback_color` fills the area without art.
    pub fn textured(texture: TextureId, center: Vec2, scale: f32, fallback_color: Rgba, depth: f32) -> Self {
        Self {
            center,
            size: texture.world_size() * scale,
            color: fallback_color,
            texture: Some(texture),
            tint: Rgba::WHITE,
            layer: None,
            rotation_radians: 0.0,
            flip_horizontally: false,
            depth,
            motion: Vec2::ZERO,
        }
    }
}

pub const ROOM_BACKGROUND_DEPTH: f32 = 0.0;
pub const FLOOR_DEPTH: f32 = 1.0;
/// Flat things on the floor: pits, spikes, webs.
pub const FLOOR_DECAL_DEPTH: f32 = 2.0;
pub const TEAR_SHADOW_DEPTH: f32 = 6.0;
pub const TEAR_DEPTH: f32 = 20.0;
pub const SPLASH_DEPTH: f32 = 21.0;

/// Actors standing on the floor (player, grid objects) are sorted by how low they stand, so the one
/// nearer the bottom of the screen is drawn in front, as in the original.
const ACTOR_DEPTH_BASE: f32 = 10.0;
/// Any y above the room's floor; depths grow toward the bottom of the screen.
const ACTOR_DEPTH_REFERENCE_Y: f32 = 200.0;
const ACTOR_DEPTH_PER_WORLD_UNIT: f32 = 0.01;
/// Keeps an actor's own parts (body, head, face) in order without crossing other actors.
pub const ACTOR_PART_DEPTH_STEP: f32 = 0.001;

pub fn actor_depth(ground_y: f32) -> f32 {
    ACTOR_DEPTH_BASE + (ACTOR_DEPTH_REFERENCE_Y - ground_y) * ACTOR_DEPTH_PER_WORLD_UNIT
}

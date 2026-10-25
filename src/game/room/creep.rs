//! Blood creep on the floor (`bloo`, parc's particle): a puddle holds its size for 101 frames, shrinks
//! over the next 40 and is gone at its 141st. Standing in one hurts, or in dark rooms and spider boss
//! fights slows (`firecheck()`).

use glam::Vec2;

use crate::game::draw::{DrawItem, FLOOR_DECAL_DEPTH, Rgba};
use crate::game::original_units::{Frames, distance};
use crate::game::parameters::CreepParameters;
use crate::game::texture_id::{BLOOD_CREEP_LOOK_COUNT, TextureId};

const FALLBACK_COLOR: Rgba = Rgba::opaque(0.45, 0.02, 0.02);
/// Above the rubble and cracks, which it covers.
const DEPTH_ABOVE_DECALS: f32 = 0.5;

#[derive(Clone, Debug)]
struct Puddle {
    /// World coordinates (y up).
    center: Vec2,
    /// 1.0 for the 100 % the clip is placed at.
    scale: f32,
    age: Frames,
}

impl Puddle {
    /// The clip's own shrinking at the end of its life.
    fn size(&self, parameters: &CreepParameters) -> f32 {
        let shrinking_for = (self.age - parameters.full_size_frames).get().max(0.0).ceil();
        self.scale * (1.0 - parameters.shrink_per_frame * shrinking_for).max(0.0)
    }
}

#[derive(Clone, Debug, Default)]
pub struct Creep {
    puddles: Vec<Puddle>,
    /// Dark rooms and spider boss fights: it slows instead of hurting (`blackout == 2 || spidboss`).
    slows: bool,
}

impl Creep {
    /// `parc("bloo", ...)`: a puddle at `center`, `scale` times its usual size. Nothing we have leaves
    /// creep yet.
    #[cfg(test)]
    pub fn spawn(&mut self, center: Vec2, scale: f32) {
        self.puddles.push(Puddle {
            center,
            scale,
            age: Frames::ZERO,
        });
    }

    #[cfg(test)]
    pub fn make_slowing(&mut self) {
        self.slows = true;
    }

    pub fn slows(&self) -> bool {
        self.slows
    }

    pub fn advance(&mut self, elapsed: Frames, parameters: &CreepParameters) {
        for puddle in &mut self.puddles {
            puddle.age += elapsed;
        }
        self.puddles.retain(|puddle| puddle.age < parameters.lifetime);
    }

    /// `shit()`: whether a puddle is under `point`.
    pub fn is_under(&self, point: Vec2, parameters: &CreepParameters) -> bool {
        self.puddles
            .iter()
            .any(|puddle| distance(puddle.center, point) < parameters.hit_radius * puddle.size(parameters))
    }

    pub fn append_draw_items(&self, parameters: &CreepParameters, draw_items: &mut Vec<DrawItem>) {
        draw_items.extend(self.puddles.iter().map(|puddle| {
            // The splat's clip cycles through its four looks each frame.
            let look = puddle.age.looped_frame(BLOOD_CREEP_LOOK_COUNT);
            DrawItem::textured(
                TextureId::BloodCreep(look),
                puddle.center,
                puddle.size(parameters),
                FALLBACK_COLOR,
                FLOOR_DECAL_DEPTH + DEPTH_ABOVE_DECALS,
            )
        }));
    }
}

#[cfg(test)]
mod tests;

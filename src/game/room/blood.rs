//! Blood splats on the floor (`splater()`): each is drawn into the room's `splat` bitmap with the
//! blood tint of the moment (`bloc`), randomly turned upside down, a few pixels off and 5 up. The
//! bitmap is made anew for every room, so a room entered again shows none of its old splats (unlike
//! the craters, which the original keeps per room and draws again).

use glam::Vec2;

use crate::game::draw::{DrawItem, DrawLayer, FLOOR_DECAL_DEPTH, Rgba};
use crate::game::original_units::Frames;
use crate::game::parameters::{SplatParameters, SplatSize};
use crate::game::random::Random;
use crate::game::room::stage_to_world;
use crate::game::texture_id::TextureId;

const FALLBACK_COLOR: Rgba = Rgba::opaque(0.45, 0.02, 0.02);
/// Under the craters, which share the bitmap (and are mostly drawn after the blood around them).
const DEPTH_ABOVE_DECALS: f32 = 0.45;

/// One call of `splater()`: where (stage coordinates), which look (`None` keeps the one drawn last,
/// as a look of 0 does in the original) and how big.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Splatter {
    pub stage_point: Vec2,
    pub look: Option<u8>,
    pub scale: f32,
}

impl Splatter {
    /// Any of the red looks (`1 + random(10)`); the scale is drawn first, as the original's arguments
    /// are worked out last to first.
    pub fn any_look(stage_point: Vec2, size: SplatSize, parameters: &SplatParameters, random: &mut Random) -> Self {
        let scale = size.least + random.next_unit() * size.spread;
        let look = Some(random.below_u8(parameters.look_count));
        Self {
            stage_point,
            look,
            scale,
        }
    }

    /// `random(10)` as the frame: any red look but the last, or (on 0) the look drawn last.
    pub fn any_look_or_last(
        stage_point: Vec2,
        size: SplatSize,
        parameters: &SplatParameters,
        random: &mut Random,
    ) -> Self {
        let scale = size.least + random.next_unit() * size.spread;
        let look = random.below_u8(parameters.look_count).checked_sub(1);
        Self {
            stage_point,
            look,
            scale,
        }
    }

    pub fn scaled(self, factor: f32) -> Self {
        Self {
            scale: self.scale * factor,
            ..self
        }
    }
}

#[derive(Clone, Debug)]
struct Splat {
    /// World coordinates (y up).
    center: Vec2,
    look: u8,
    upside_down: bool,
    scale: f32,
    tint: Rgba,
}

#[derive(Clone, Debug)]
pub struct BloodSplats {
    splats: Vec<Splat>,
    /// The frame the blood clip was last sent to (it starts on its first).
    last_look: u8,
    /// `bloc`.
    tint: Rgba,
}

impl Default for BloodSplats {
    fn default() -> Self {
        Self {
            splats: Vec::new(),
            last_look: 0,
            tint: Rgba::WHITE,
        }
    }
}

impl BloodSplats {
    /// `bloc` is made anew every seventh frame: each channel a random 85 to 100 % (dark rooms, which
    /// darken it, are not made yet).
    pub fn retint(&mut self, frame: Frames, parameters: &SplatParameters, random: &mut Random) {
        if frame.is_on_cycle(parameters.tint_every, Frames::ZERO) {
            let mut channel = || parameters.tint_least + random.next_unit() * parameters.tint_spread;
            self.tint = Rgba::opaque(channel(), channel(), channel());
        }
    }

    /// `splater()`.
    pub fn splatter(&mut self, splatter: Splatter, parameters: &SplatParameters, random: &mut Random) {
        if let Some(look) = splatter.look {
            self.last_look = look;
        }
        let upside_down = random.happens(crate::game::odds::Odds::EVEN);
        // `crand()` gives the cosine first, and the y offset is worked out before the x.
        let jitter = random.direction(parameters.jitter.get());
        let offset = Vec2::new(jitter.y, jitter.x - parameters.lift.get());
        self.splats.push(Splat {
            center: stage_to_world(splatter.stage_point + offset),
            look: self.last_look,
            upside_down,
            scale: splatter.scale,
            tint: self.tint,
        });
    }

    /// Leaving the room: its bitmap goes.
    pub fn clear(&mut self) {
        self.splats.clear();
    }

    #[cfg(test)]
    pub fn count(&self) -> usize {
        self.splats.len()
    }

    pub fn append_draw_items(&self, parameters: &SplatParameters, draw_items: &mut Vec<DrawItem>) {
        draw_items.extend(self.splats.iter().map(|splat| DrawItem {
            // Turned upside down: mirrored and turned half a turn.
            flip_horizontally: splat.upside_down,
            rotation_radians: if splat.upside_down { std::f32::consts::PI } else { 0.0 },
            tint: splat.tint,
            layer: Some(splat_layer(parameters)),
            ..DrawItem::textured(
                TextureId::BloodSplat(splat.look),
                splat.center,
                splat.scale,
                FALLBACK_COLOR,
                FLOOR_DECAL_DEPTH + DEPTH_ABOVE_DECALS,
            )
        }));
    }
}

/// The floor's `splat` bitmap, which holds the blood and the craters and is shown at 70 %.
pub fn splat_layer(parameters: &SplatParameters) -> DrawLayer {
    DrawLayer {
        opacity: parameters.layer_opacity,
    }
}

#[cfg(test)]
mod tests;

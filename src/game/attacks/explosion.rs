//! Explosions: the blast animation and the scorch mark it leaves (`gosplash()` for type 4).

use glam::Vec2;

use crate::game::draw::{DrawItem, DrawLayer, FLOOR_DECAL_DEPTH, Rgba};
use crate::game::original_units::{Frames, Pixels, PixelsPerFrame, length};
use crate::game::parameters::ExplosionParameters;
use crate::game::player::Player;
use crate::game::texture_id::{EXPLOSION_FRAME_COUNT, TextureId};

/// The fireball is drawn above everything on the floor.
const EXPLOSION_DEPTH: f32 = 22.0;
/// Scorch marks lie on the floor just above other floor decals.
const SCORCH_DEPTH_ABOVE_DECALS: f32 = 0.5;
const FIREBALL_FALLBACK_COLOR: Rgba = Rgba::opaque(1.0, 0.6, 0.1);
const SCORCH_FALLBACK_COLOR: Rgba = Rgba {
    red: 0.0,
    green: 0.0,
    blue: 0.0,
    alpha: 0.4,
};

#[derive(Clone, Debug)]
pub struct Explosion {
    center: Vec2,
    age_frames: Frames,
}

impl Explosion {
    pub fn starting_at(center: Vec2) -> Self {
        Self {
            center,
            age_frames: Frames::ZERO,
        }
    }

    pub fn advance(&mut self, elapsed: Frames) {
        self.age_frames += elapsed;
    }

    fn frame(&self) -> u32 {
        self.age_frames.whole()
    }

    pub fn is_finished(&self) -> bool {
        self.frame() >= u32::from(EXPLOSION_FRAME_COUNT)
    }

    pub fn draw_item(&self) -> DrawItem {
        let frame = self.age_frames.held_frame(EXPLOSION_FRAME_COUNT);
        DrawItem::textured(
            TextureId::Explosion(frame),
            self.center,
            1.0,
            FIREBALL_FALLBACK_COLOR,
            EXPLOSION_DEPTH,
        )
    }
}

/// Permanent mark on the floor where something exploded (`crater`), drawn into the floor's splat
/// bitmap (`layer`).
pub fn scorch_draw_item(center: Vec2, layer: DrawLayer) -> DrawItem {
    DrawItem {
        layer: Some(layer),
        ..DrawItem::textured(
            TextureId::ExplosionScorch,
            center,
            1.0,
            SCORCH_FALLBACK_COLOR,
            FLOOR_DECAL_DEPTH + SCORCH_DEPTH_ABOVE_DECALS,
        )
    }
}

/// Pushes the player away from a blast at `center` (the original's case for the player);
/// returns whether the blast reached them.
pub fn push_player(player: &mut Player, center: Vec2, parameters: ExplosionParameters) -> bool {
    let Some(push) = blast_push(player.position() - center, parameters.player_push_reach, parameters) else {
        return false;
    };
    player.knock_back(parameters.velocity_retention, push);
    true
}

/// The push a blast gives something `away` from its center, if within `reach` (85 plus its size):
/// the reach left over, at most 15 px per frame, straight away. Right at the center the original
/// divides by zero and nothing visibly moves, so there is no push there.
pub fn blast_push(away: Vec2, reach: Pixels, parameters: ExplosionParameters) -> Option<Vec2> {
    let distance = length(away);
    if distance >= reach {
        return None;
    }
    if distance.get() <= f32::EPSILON {
        return Some(Vec2::ZERO);
    }
    // The original turns the overlap in pixels straight into a speed.
    let push = PixelsPerFrame((reach - distance).get()).min(parameters.maximum_push_per_frame);
    Some(away / distance.get() * push)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::parameters::GameParameters;

    #[test]
    fn a_blast_pushes_a_nearby_player_away_and_leaves_a_distant_one() {
        let parameters = GameParameters::ORIGINAL;
        let mut near = Player::new(Vec2::new(40.0, 0.0), &parameters);
        push_player(&mut near, Vec2::ZERO, parameters.explosion);
        // 55 px of reach left, capped at 15 px per frame, straight away from the blast.
        assert_eq!(near.velocity(), Vec2::new(15.0, 0.0));
        let mut far = Player::new(Vec2::new(100.0, 0.0), &parameters);
        push_player(&mut far, Vec2::ZERO, parameters.explosion);
        assert_eq!(far.velocity(), Vec2::ZERO);
    }

    #[test]
    fn a_blast_right_under_a_still_player_hurts_but_does_not_push() {
        let parameters = GameParameters::ORIGINAL;
        let mut player = Player::new(Vec2::ZERO, &parameters);
        assert!(push_player(&mut player, Vec2::ZERO, parameters.explosion));
        assert_eq!(player.velocity(), Vec2::ZERO);
    }
}

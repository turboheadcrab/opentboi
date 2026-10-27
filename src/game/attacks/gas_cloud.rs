//! The cloud Bad Gas lets out (`fart()`, DoAction 26697–26719): the bomb clip's fifth frame at 70 % of
//! its size, where Isaac stood, playing its 46 frames once.

use glam::Vec2;

use crate::game::draw::{DrawItem, Rgba, actor_depth};
use crate::game::original_units::Frames;
use crate::game::texture_id::{GAS_CLOUD_FRAME_COUNT, TextureId};

const FALLBACK_COLOR: Rgba = Rgba::opaque(0.45, 0.6, 0.2);

#[derive(Clone, Debug)]
pub struct GasCloud {
    center: Vec2,
    scale: f32,
    age_frames: Frames,
}

impl GasCloud {
    /// A cloud at `center`, drawn at `scale` of its size.
    pub fn starting_at(center: Vec2, scale: f32) -> Self {
        Self {
            center,
            scale,
            age_frames: Frames::ZERO,
        }
    }

    pub fn advance(&mut self, elapsed: Frames) {
        self.age_frames += elapsed;
    }

    pub fn is_finished(&self) -> bool {
        self.age_frames.whole() >= u32::from(GAS_CLOUD_FRAME_COUNT)
    }

    pub fn draw_item(&self) -> DrawItem {
        DrawItem::textured(
            TextureId::GasCloud(self.age_frames.held_frame(GAS_CLOUD_FRAME_COUNT)),
            self.center,
            self.scale,
            FALLBACK_COLOR,
            actor_depth(self.center.y),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_cloud_plays_its_frames_once_and_is_gone() {
        let mut cloud = GasCloud::starting_at(Vec2::ZERO, 0.7);
        assert!(!cloud.is_finished());
        cloud.advance(Frames::from(GAS_CLOUD_FRAME_COUNT - 1));
        assert!(!cloud.is_finished());
        cloud.advance(Frames::ONE);
        assert!(cloud.is_finished());
    }
}

//! The fly that circles a corny poop (`efly()`, object type 96).

use std::f32::consts::PI;

use glam::Vec2;

use crate::game::draw::{DrawItem, Rgba};
use crate::game::original_units::{Frames, Pixels};
use crate::game::texture_id::{ORBITING_FLY_FRAME_COUNT, TextureId};

/// Around a poop, the target point goes round an ellipse 60 wide and 50 high, turning 0.02π per
/// frame, each fly half a radian further on, the even ones mirrored.
const ORBIT_RADIUS_X: Pixels = Pixels(60.0);
const ORBIT_RADIUS_Y: Pixels = Pixels(50.0);
const ANGLE_PER_FRAME: f32 = 0.02 * PI;
const ANGLE_OFFSET_PER_FLY: f32 = 0.5;
/// Flies hover above the floor and are drawn over grid objects and the player.
const FLY_DEPTH: f32 = 19.0;
const FALLBACK_COLOR: Rgba = Rgba::opaque(0.9, 0.9, 0.9);

#[derive(Clone, Debug)]
pub struct OrbitingFly {
    /// Index of the grid object it circles.
    pub anchor_index: usize,
    /// The original's running fly counter (`flyer++`), which sets the phase and direction.
    fly_number: u32,
    age_frames: Frames,
    /// Its age when the current tick began.
    previous_age_frames: Frames,
}

impl OrbitingFly {
    pub fn new(anchor_index: usize, fly_number: u32) -> Self {
        Self {
            anchor_index,
            fly_number,
            age_frames: Frames::ZERO,
            previous_age_frames: Frames::ZERO,
        }
    }

    pub fn advance(&mut self, elapsed: Frames) {
        self.previous_age_frames = self.age_frames;
        self.age_frames += elapsed;
    }

    pub fn draw_item(&self, anchor_center: Vec2) -> DrawItem {
        let offset = self.offset_at(self.age_frames);
        let wing_frame = self.age_frames.looped_frame(ORBITING_FLY_FRAME_COUNT);
        DrawItem {
            motion: offset - self.offset_at(self.previous_age_frames),
            ..DrawItem::textured(
                TextureId::OrbitingFly(wing_frame),
                anchor_center + offset,
                1.0,
                FALLBACK_COLOR,
                FLY_DEPTH,
            )
        }
    }

    /// Where it is around its poop at `age`.
    fn offset_at(&self, age: Frames) -> Vec2 {
        let mut angle = -age.get() * ANGLE_PER_FRAME + self.fly_number as f32 * ANGLE_OFFSET_PER_FLY;
        if self.fly_number.is_multiple_of(2) {
            angle = -angle;
        }
        // The original's y axis points down; ours points up.
        Vec2::new(angle.sin() * ORBIT_RADIUS_X.get(), -angle.cos() * ORBIT_RADIUS_Y.get())
    }
}

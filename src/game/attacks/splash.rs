//! Tear splash effects, played from the tear clip's splash frames at the original frame rate, and
//! the blood splash enemy shots leave (`bloodsplash`).

use std::f32::consts::TAU;

use glam::Vec2;

use crate::game::draw::{DrawItem, Rgba, SPLASH_DEPTH};
use crate::game::original_units::Frames;
use crate::game::random::Random;
use crate::game::texture_id::{
    BLOOD_SPLASH_FRAME_COUNT, TEAR_GROUND_SPLASH_FRAME_COUNT, TEAR_HIT_SPLASH_FRAME_COUNT, TextureId,
};

const FALLBACK_COLOR: Rgba = Rgba {
    red: 0.55,
    green: 0.75,
    blue: 1.0,
    alpha: 0.4,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SplashKind {
    /// Against a wall or obstacle.
    Hit,
    /// Fell to the floor (`groundhit`).
    Ground,
    /// An enemy shot ending anywhere.
    Blood,
}

#[derive(Clone, Debug)]
pub struct Splash {
    kind: SplashKind,
    /// Where the tear was drawn when it ended, i.e. including its height above the floor.
    center: Vec2,
    scale: f32,
    /// The original gives each finished tear a random rotation.
    rotation_radians: f32,
    age_frames: Frames,
}

impl Splash {
    pub fn new(kind: SplashKind, center: Vec2, scale: f32, random: &mut Random) -> Self {
        Self {
            kind,
            center,
            scale,
            rotation_radians: random.next_unit() * TAU,
            age_frames: Frames::ZERO,
        }
    }

    pub fn advance(&mut self, elapsed: Frames) {
        self.age_frames += elapsed;
    }

    fn frame(&self) -> u8 {
        self.age_frames.frame_number()
    }

    fn frame_count(&self) -> u8 {
        match self.kind {
            SplashKind::Hit => TEAR_HIT_SPLASH_FRAME_COUNT,
            SplashKind::Ground => TEAR_GROUND_SPLASH_FRAME_COUNT,
            SplashKind::Blood => BLOOD_SPLASH_FRAME_COUNT,
        }
    }

    pub fn is_finished(&self) -> bool {
        self.frame() >= self.frame_count()
    }

    pub fn draw_item(&self) -> DrawItem {
        let frame = self.frame().min(self.frame_count() - 1);
        let texture = match self.kind {
            SplashKind::Hit => TextureId::TearHitSplash(frame),
            SplashKind::Ground => TextureId::TearGroundSplash(frame),
            SplashKind::Blood => TextureId::BloodSplash(frame),
        };
        DrawItem {
            rotation_radians: self.rotation_radians,
            ..DrawItem::textured(texture, self.center, self.scale, FALLBACK_COLOR, SPLASH_DEPTH)
        }
    }
}

//! What scaling vectors by the units means: velocities slowed, moved over time, and lengths.

use std::ops::{Div, Mul, MulAssign};

use glam::Vec2;

use super::{Frames, Pixels, PixelsPerFrame, PixelsPerFrameSquared, Retention};

/// A velocity keeping `retention` of itself.
impl MulAssign<Retention> for Vec2 {
    fn mul_assign(&mut self, retention: Retention) {
        *self *= retention.0;
    }
}

impl Mul<Retention> for PixelsPerFrame {
    type Output = Self;
    fn mul(self, retention: Retention) -> Self {
        Self(self.0 * retention.0)
    }
}

impl Mul<Frames> for PixelsPerFrame {
    type Output = Pixels;
    fn mul(self, elapsed: Frames) -> Pixels {
        Pixels(self.0 * elapsed.0)
    }
}

impl Mul<Frames> for PixelsPerFrameSquared {
    type Output = PixelsPerFrame;
    fn mul(self, elapsed: Frames) -> PixelsPerFrame {
        PixelsPerFrame(self.0 * elapsed.0)
    }
}

/// A velocity (pixels per frame) over a duration: the distance covered.
impl Mul<Frames> for Vec2 {
    type Output = Vec2;
    fn mul(self, elapsed: Frames) -> Vec2 {
        self * elapsed.0
    }
}

/// A direction scaled to a distance.
impl Mul<Pixels> for Vec2 {
    type Output = Vec2;
    fn mul(self, distance: Pixels) -> Vec2 {
        self * distance.0
    }
}

/// An offset measured in whole lengths of `distance`.
impl Div<Pixels> for Vec2 {
    type Output = Vec2;
    fn div(self, distance: Pixels) -> Vec2 {
        self / distance.0
    }
}

/// A direction scaled to a speed.
impl Mul<PixelsPerFrame> for Vec2 {
    type Output = Vec2;
    fn mul(self, speed: PixelsPerFrame) -> Vec2 {
        self * speed.0
    }
}

/// A direction scaled to an acceleration.
impl Mul<PixelsPerFrameSquared> for Vec2 {
    type Output = Vec2;
    fn mul(self, acceleration: PixelsPerFrameSquared) -> Vec2 {
        self * acceleration.0
    }
}

/// How far apart two points are.
pub fn distance(first: Vec2, second: Vec2) -> Pixels {
    Pixels(first.distance(second))
}

/// How long a vector is.
pub fn length(vector: Vec2) -> Pixels {
    Pixels(vector.length())
}

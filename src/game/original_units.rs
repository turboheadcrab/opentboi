//! The original ran its logic once per Flash frame, 30 times a second, and its constants are per
//! frame. The game keeps every value in those units and advances in ticks of a fixed number of
//! original frames: `30 / ticks per second`.
//!
//! Scalars carry their unit in their type: [`Frames`] (durations and counters), [`Pixels`]
//! (distances), [`PixelsPerFrame`] (speeds), [`PixelsPerFrameSquared`] (what is added to a speed
//! every frame), [`Retention`] (what a value keeps of itself every frame) and [`Hearts`] (health).
//! Positions and velocities stay `Vec2`, in pixels and pixels per frame.

use std::ops::{Add, AddAssign, Div, Mul, Neg, Sub, SubAssign};

pub const ORIGINAL_FRAMES_PER_SECOND: f32 = 30.0;
/// Ticks per second unless the launch options say otherwise (`--tick-rate`): the original's own
/// rate, so every tick is exactly one of its frames.
pub const DEFAULT_TICKS_PER_SECOND: u32 = 30;

/// Original frames one tick covers at `ticks_per_second`.
pub fn frames_per_tick(ticks_per_second: u32) -> Frames {
    Frames(ORIGINAL_FRAMES_PER_SECOND / ticks_per_second.max(1) as f32)
}

/// Implements the arithmetic every unit shares: adding and subtracting its own kind, scaling by a
/// plain number, ordering, and the plain value (`get`).
macro_rules! unit {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
        pub struct $name(pub f32);

        #[allow(dead_code, reason = "every unit offers the same basics, used or not")]
        impl $name {
            pub const ZERO: Self = Self(0.0);

            pub const fn get(self) -> f32 {
                self.0
            }

            pub fn min(self, other: Self) -> Self {
                Self(self.0.min(other.0))
            }

            pub fn max(self, other: Self) -> Self {
                Self(self.0.max(other.0))
            }
        }

        impl Add for $name {
            type Output = Self;
            fn add(self, other: Self) -> Self {
                Self(self.0 + other.0)
            }
        }

        impl AddAssign for $name {
            fn add_assign(&mut self, other: Self) {
                self.0 += other.0;
            }
        }

        impl Sub for $name {
            type Output = Self;
            fn sub(self, other: Self) -> Self {
                Self(self.0 - other.0)
            }
        }

        impl SubAssign for $name {
            fn sub_assign(&mut self, other: Self) {
                self.0 -= other.0;
            }
        }

        impl Neg for $name {
            type Output = Self;
            fn neg(self) -> Self {
                Self(-self.0)
            }
        }

        impl Mul<f32> for $name {
            type Output = Self;
            fn mul(self, factor: f32) -> Self {
                Self(self.0 * factor)
            }
        }

        impl Div<f32> for $name {
            type Output = Self;
            fn div(self, divisor: f32) -> Self {
                Self(self.0 / divisor)
            }
        }

        impl Div for $name {
            type Output = f32;
            fn div(self, divisor: Self) -> f32 {
                self.0 / divisor.0
            }
        }
    };
}

unit!(
    /// A duration or a count of the original's frames.
    Frames
);
unit!(
    /// A distance in the original's pixels (the stage is 640×480).
    Pixels
);
unit!(
    /// A speed: pixels moved each frame.
    PixelsPerFrame
);
unit!(
    /// A change in speed each frame.
    PixelsPerFrameSquared
);

unit!(
    /// Health, in whole hearts that go up and down by halves, as the original counts it.
    Hearts
);

impl Hearts {
    pub const HALF: Self = Self(0.5);
    pub const ONE: Self = Self(1.0);
    const HALVES_PER_HEART: f32 = 2.0;

    /// The half hearts it makes up, for drawing.
    pub fn halves(self) -> i32 {
        crate::whole_numbers::clamped_i32((self.0 * Self::HALVES_PER_HEART).round())
    }
}

impl Frames {
    /// The whole frames elapsed, for picking an animation frame.
    pub fn whole(self) -> u32 {
        crate::whole_numbers::clamped_u32(self.0)
    }

    /// The frame of a looping clip of `count` frames this far in.
    pub fn looped_frame(self, count: u8) -> u8 {
        u8::try_from(self.whole() % u32::from(count.max(1))).unwrap_or_default()
    }

    /// The whole frames elapsed, as a clip's frame number (held on the last one a clip can have).
    pub fn frame_number(self) -> u8 {
        u8::try_from(self.whole()).unwrap_or(u8::MAX)
    }

    /// The frame of a clip of `count` frames this far in, held on its last however long it has
    /// been (clamped before narrowing, so a long wait never wraps around).
    pub fn held_frame(self, count: u8) -> u8 {
        let last = count.saturating_sub(1);
        u8::try_from(self.whole()).map_or(last, |frame| frame.min(last))
    }
}

impl Frames {
    pub const ONE: Self = Self(1.0);

    /// The whole frames elapsed, still as a time (the room's frame number).
    pub fn floor(self) -> Self {
        Self(self.0.floor())
    }

    /// The `number`th frame of a room or other clock, counted from 1.
    #[allow(clippy::cast_precision_loss, reason = "frame numbers stay far below 2^24")]
    pub fn from_number(number: u32) -> Self {
        Self(number as f32)
    }

    /// How long that is in real time, at the original's 30 frames a second.
    pub fn seconds(self) -> f32 {
        self.0 / ORIGINAL_FRAMES_PER_SECOND
    }

    /// The frames that fit in `seconds` of real time.
    pub fn from_seconds(seconds: f32) -> Self {
        Self(seconds * ORIGINAL_FRAMES_PER_SECOND)
    }

    /// Whether this frame falls on `phase` of a cycle `period` frames long (a period of nothing
    /// counts as every frame).
    pub fn is_on_cycle(self, period: Self, phase: Self) -> bool {
        self.whole() % period.whole().max(1) == phase.whole()
    }
}

impl From<u16> for Frames {
    fn from(frame_count: u16) -> Self {
        Self(f32::from(frame_count))
    }
}

impl From<u8> for Frames {
    fn from(frame_count: u8) -> Self {
        Self(f32::from(frame_count))
    }
}

/// What a value keeps of itself each frame (a speed slowed to 95 % keeps 0.95).
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Retention(pub f32);

impl Retention {
    /// What is left after `elapsed`: `0.95` over two frames keeps `0.95 * 0.95`.
    pub fn over(self, elapsed: Frames) -> f32 {
        self.0.powf(elapsed.0)
    }
}

/// Flash clips start on frame 1.
pub const FIRST_CLIP_FRAME: Frames = Frames(1.0);

mod combat;
mod vectors;

pub use combat::{Damage, HitPoints, Mass};
pub use vectors::{distance, length};

#[cfg(test)]
mod tests;

//! Horf (type 12, `b12`): the post-switch part of `smarts()` that stationary shooters share
//! (DoAction 25235–25285), firing with `firemode()` (9515–9690) as a Pooter does.

use crate::game::original_units::{Damage, Frames, Hearts, HitPoints, Mass, Pixels, Retention};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HorfParameters {
    /// `hps[12]`.
    pub hit_points: HitPoints,
    /// `sizes[12]` and `masses[12]`.
    pub size: Pixels,
    pub mass: Mass,
    pub scale: f32,
    /// The appearing clip (`d` frame 4, 5013: 60 frames).
    pub appearing_frames: Frames,
    /// Until then it keeps 45 % of its speed each frame.
    pub appearing_retention: Retention,
    /// Dying plays `d` frame 3 (566, 36 frames).
    pub dying_frames: Frames,
    pub contact_damage: Hearts,
    pub trample_damage: Damage,
    /// Each frame it keeps 70 % of its speed; it never moves of its own accord.
    pub velocity_retention: Retention,
    /// `d` frame 5, 5023: its shot, 20 frames; frame 1's head has 3.
    pub shooting_frames: Frames,
    pub idle_frame_count: u8,
}

pub const ORIGINAL_HORFS: HorfParameters = HorfParameters {
    hit_points: HitPoints(10.0),
    size: Pixels(13.0),
    mass: Mass(3.0),
    scale: 1.0,
    appearing_frames: Frames(59.0),
    appearing_retention: Retention(0.45),
    dying_frames: Frames(36.0),
    contact_damage: Hearts::HALF,
    trample_damage: Damage(40.0),
    velocity_retention: Retention(0.7),
    shooting_frames: Frames(20.0),
    idle_frame_count: 3,
};

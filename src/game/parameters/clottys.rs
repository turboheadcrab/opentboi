//! Clotty (type 15, `b15`): the case for it in `smarts()` (DoAction 24381–24470). It alternates
//! between wandering (`randrun()`) and a shooting mode that plays its shot clip once and fires
//! four shots when the clip is well on its way (`quadf()`, 9180).

use crate::game::odds::Odds;
use crate::game::original_units::{Damage, Frames, Hearts, HitPoints, Mass, Pixels, PixelsPerFrame, Retention};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ClottyParameters {
    /// `hps[15]`: dies once below zero.
    pub hit_points: HitPoints,
    /// `sizes[15]` and `masses[15]`.
    pub size: Pixels,
    pub mass: Mass,
    pub scale: f32,
    /// The appearing clip (`d` frame 4, 4895: 27 frames) plays out before it acts; meanwhile it keeps
    /// this much of its speed each frame.
    pub appearing_frames: Frames,
    pub appearing_retention: Retention,
    /// Dying plays `d` frame 3 (566, 36 frames).
    pub dying_frames: Frames,
    pub contact_damage: Hearts,
    pub trample_damage: Damage,
    /// Each frame it keeps 90 % of its speed.
    pub velocity_retention: Retention,
    /// `d` frame 5, 4900: its shot clip, 24 frames, which fires once it is past its tenth frame.
    pub shooting_frames: Frames,
    pub shot_after: Frames,
    pub shot_speed: PixelsPerFrame,
    /// While shooting it moves on to wandering one frame in 30, once 30 frames have passed since it
    /// last wandered; a finished shot clip moves it on at once.
    pub mode_change_odds: Odds,
    pub mode_minimum_frames: Frames,
    /// Frame 1 idles (4877, 24 frames) and frame 2 walks (4894, 20 frames).
    pub idle_frame_count: u8,
    pub walking_frame_count: u8,
}

pub const ORIGINAL_CLOTTYS: ClottyParameters = ClottyParameters {
    hit_points: HitPoints(15.0),
    size: Pixels(13.0),
    mass: Mass(10.0),
    scale: 1.0,
    appearing_frames: Frames(26.0),
    appearing_retention: Retention(0.45),
    dying_frames: Frames(36.0),
    contact_damage: Hearts::HALF,
    trample_damage: Damage(40.0),
    velocity_retention: Retention(0.9),
    shooting_frames: Frames(23.0),
    shot_after: Frames(10.0),
    shot_speed: PixelsPerFrame(7.0),
    mode_change_odds: Odds::one_in(30),
    mode_minimum_frames: Frames(30.0),
    idle_frame_count: 24,
    walking_frame_count: 20,
};

//! The headless body (type 11, `b11`; a Gaper's body once it lost its head, or a layout's own):
//! the case of `smarts()` that wanders it about (DoAction 24820–24850) with `randrun()` (8614) and
//! `firewalk()` (9712), and the turn of a Gaper into one in `hurt()` (4495–4515).

use glam::Vec2;

use crate::game::odds::Odds;
use crate::game::original_units::{
    Damage, Frames, Hearts, HitPoints, Mass, Pixels, PixelsPerFrame, PixelsPerFrameSquared, Retention,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HeadlessParameters {
    /// `hps[11]`: a Gaper turning into one starts over with this many (its own full hit points).
    pub hit_points: HitPoints,
    /// `sizes[11]` and `masses[11]`.
    pub size: Pixels,
    pub mass: Mass,
    /// A layout's own is drawn at the clip's size (a converted Gaper keeps its 0.9).
    pub scale: f32,
    /// The appearing clip (`d` frame 4, 3482: 26 frames).
    pub appearing_frames: Frames,
    /// Until then it keeps 45 % of its speed each frame.
    pub appearing_retention: Retention,
    /// Dying plays `d` frame 3 (566, 36 frames).
    pub dying_frames: Frames,
    pub contact_damage: Hearts,
    pub trample_damage: Damage,
    /// A Gaper's lethal hit turns it into one on a coin flip, unless the hit left it below this many
    /// hit points.
    pub conversion_odds: Odds,
    pub conversion_floor: HitPoints,
    /// Each frame it keeps 75 % of its speed.
    pub velocity_retention: Retention,
    /// `randrun()`: with no goal, or none picked for 10 frames, it picks a spot 40 pixels away in
    /// a random direction (kept in the room's middle, x 120–540 and y 210–360, one time in two),
    /// if the tile there is free, and pulls towards it by half a pixel per frame; it gives the goal
    /// up within 3 pixels, or when nearly stopped (below 0.2) one frame in 10.
    pub goal_distance: Pixels,
    pub goal_retry_frames: Frames,
    pub goal_middle_odds: Odds,
    pub middle_minimum: Vec2,
    pub middle_maximum: Vec2,
    pub room_minimum: Vec2,
    pub room_maximum: Vec2,
    pub pull: PixelsPerFrameSquared,
    pub arrival_distance: Pixels,
    pub stalled_speed: PixelsPerFrame,
    pub stalled_odds: Odds,
    /// `walkframe(2)`: its clip shows walking while twice its speed is above 2 pixels per frame.
    pub walking_speed_scale: f32,
    pub walking_speed: PixelsPerFrame,
    /// `bb`, the stump's blood, has 10 frames.
    pub spurt_frame_count: u8,
    /// A Gaper's body spurts one time in three (`bnuts`); a layout's one unless its marker has a
    /// fractional value.
    pub spurting_odds: Odds,
    /// `firewalk()`: while it spurts, it shoots a blood ball ahead of it every 40 frames plus a
    /// random 0–19.
    pub spurt_cooldown_base: Frames,
    pub spurt_cooldown_range: Frames,
}

pub const ORIGINAL_HEADLESS: HeadlessParameters = HeadlessParameters {
    hit_points: HitPoints(10.0),
    size: Pixels(13.0),
    mass: Mass(3.0),
    scale: 1.0,
    appearing_frames: Frames(25.0),
    appearing_retention: Retention(0.45),
    dying_frames: Frames(36.0),
    contact_damage: Hearts::HALF,
    trample_damage: Damage(40.0),
    conversion_odds: Odds::one_in(2),
    conversion_floor: HitPoints(-25.0),
    velocity_retention: Retention(0.75),
    goal_distance: Pixels(40.0),
    goal_retry_frames: Frames(10.0),
    goal_middle_odds: Odds::one_in(2),
    middle_minimum: Vec2::new(120.0, 210.0),
    middle_maximum: Vec2::new(540.0, 360.0),
    room_minimum: Vec2::new(20.0, 110.0),
    room_maximum: Vec2::new(620.0, 447.0),
    pull: PixelsPerFrameSquared(0.5),
    arrival_distance: Pixels(3.0),
    stalled_speed: PixelsPerFrame(0.2),
    stalled_odds: Odds::one_in(10),
    walking_speed_scale: 2.0,
    walking_speed: PixelsPerFrame(2.0),
    spurt_frame_count: 10,
    spurting_odds: Odds::one_in(3),
    spurt_cooldown_base: Frames(40.0),
    spurt_cooldown_range: Frames(20.0),
};

//! The Hopper (type 29, `b29`): `randruny()` (DoAction 8662–8830), called from `smarts5()`
//! (DoAction 18172). It hops about in leaps driven by the frames of its hop clip.

use glam::Vec2;

use crate::game::original_units::{Damage, Frames, Hearts, HitPoints, Mass, Pixels, Retention};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HopperParameters {
    /// `hps[29]`: dies once below zero.
    pub hit_points: HitPoints,
    /// `sizes[29]` and `masses[29]`.
    pub size: Pixels,
    pub mass: Mass,
    pub scale: f32,
    /// The appearing clip (`d` frame 4, 4934: 26 frames) plays out before it hops; meanwhile it
    /// keeps this much of its speed each frame.
    pub appearing_frames: Frames,
    pub appearing_retention: Retention,
    /// Dying plays `d` frame 3 (566, 36 frames).
    pub dying_frames: Frames,
    pub contact_damage: Hearts,
    pub trample_damage: Damage,
    /// Without a leap in progress it picks a spot: the player's, if the player (taken two frames
    /// ahead at his speed) is within this reach, else a point this far off in a random direction.
    pub player_reach: Pixels,
    pub wander_distance: Pixels,
    pub player_lead: Frames,
    /// The spot is kept within these stage coordinates, then must be on a free tile; it lands
    /// within `scatter` of the tile's center.
    pub stage_minimum: Vec2,
    pub stage_maximum: Vec2,
    pub scatter: Pixels,
    /// The hop clip (`d` frame 2, 4933) has this many frames. It steps on a frame a frame, starting
    /// from its second.
    pub hop_frame_count: u8,
    /// Before this frame of the clip it crouches (a tenth of the pull); after, it pulls with the
    /// full strength until the last frames.
    pub pull_from_frame: u8,
    pub crouch_pull: f32,
    /// From the frame this far before the clip's end it lands: keeps this much of its speed, no
    /// longer pulls, and the clip steps on a frame more; two frames further it ends the leap.
    pub landing_frames: u8,
    pub landing_retention: Retention,
    pub ending_frames: u8,
    /// The pull towards the spot is a tenth of the distance, at most this much, each frame.
    pub pull_per_distance: f32,
    pub pull_maximum: f32,
    /// It turns to face a spot further sideways than this.
    pub turn_beyond: Pixels,
    /// Each frame it keeps this much of its speed.
    pub velocity_retention: Retention,
    /// The clip frame at which it lands with `Meat_impacts_` (0–2).
    pub landing_sound_frame: u8,
}

pub const ORIGINAL_HOPPERS: HopperParameters = HopperParameters {
    hit_points: HitPoints(10.0),
    size: Pixels(13.0),
    mass: Mass(10.0),
    scale: 1.0,
    appearing_frames: Frames(25.0),
    appearing_retention: Retention(0.45),
    dying_frames: Frames(36.0),
    contact_damage: Hearts::HALF,
    trample_damage: Damage(40.0),
    player_reach: Pixels(130.0),
    wander_distance: Pixels(95.0),
    player_lead: Frames(2.0),
    stage_minimum: Vec2::new(20.0, 110.0),
    stage_maximum: Vec2::new(620.0, 447.0),
    scatter: Pixels(3.0),
    hop_frame_count: 26,
    pull_from_frame: 5,
    crouch_pull: 0.1,
    landing_frames: 6,
    landing_retention: Retention(0.4),
    ending_frames: 7,
    pull_per_distance: 0.1,
    pull_maximum: 3.0,
    turn_beyond: Pixels(8.0),
    velocity_retention: Retention(0.7),
    landing_sound_frame: 19,
};

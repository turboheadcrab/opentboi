//! The flesh mound (type 77, `b74`'s last and smallest piece of the family that splits): it
//! wanders with `randrun()` (DoAction 8614) in the loop of its walking clip, in the shared tail of
//! `bawssmart2()` (DoAction 15677–15954).

use crate::game::original_units::{Damage, Frames, Hearts, HitPoints, Mass, Pixels, Retention};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FleshMoundParameters {
    /// `hps[77]`: dies once below zero.
    pub hit_points: HitPoints,
    /// `sizes[77]` and `masses[77]`.
    pub size: Pixels,
    pub mass: Mass,
    pub scale: f32,
    /// The appearing clip (`d` frame 15, 1903: 25 frames) plays out before it wanders; meanwhile it
    /// keeps this much of its speed each frame, on top of the rest.
    pub appearing_frames: Frames,
    pub appearing_retention: Retention,
    /// Dying plays `d` frame 14 (566, 36 frames).
    pub dying_frames: Frames,
    pub contact_damage: Hearts,
    pub trample_damage: Damage,
    /// Each frame it keeps this much of its speed, and this much more while its walking clip (`d`
    /// frame 13, 1902: 24 frames) is past its 19th.
    pub velocity_retention: Retention,
    pub late_clip_retention: Retention,
    pub late_clip_frame: u8,
    /// The walking clip's length: it picks a new spot each time it starts over.
    pub walking_frame_count: u8,
}

pub const ORIGINAL_FLESH_MOUNDS: FleshMoundParameters = FleshMoundParameters {
    hit_points: HitPoints(12.0),
    size: Pixels(13.0),
    mass: Mass(3.0),
    scale: 1.0,
    appearing_frames: Frames(24.0),
    appearing_retention: Retention(0.45),
    dying_frames: Frames(36.0),
    contact_damage: Hearts::HALF,
    trample_damage: Damage(40.0),
    velocity_retention: Retention(0.8),
    late_clip_retention: Retention(0.6),
    late_clip_frame: 19,
    walking_frame_count: 24,
};

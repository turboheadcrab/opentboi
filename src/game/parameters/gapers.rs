//! The Gaper (type 10, `b10`): the walker case of `smarts()` (DoAction 24985–25140) steering with
//! `pathfind()` (5777–6000), its walking clip choice in `walkframe()` (8528), and its entry in
//! `ballhit()`'s contact rules (7395–7500).

use crate::game::odds::Odds;
use crate::game::original_units::{Damage, Frames, Hearts, HitPoints, Mass, Pixels, PixelsPerFrame, Retention};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GaperParameters {
    /// `hps[10]`: dies once below zero.
    pub hit_points: HitPoints,
    /// `sizes[10]` (13) and `masses[10]`.
    pub size: Pixels,
    pub mass: Mass,
    /// `create()` draws it at 0.9 of the clip's size.
    pub scale: f32,
    /// The appearing clip (`d` frame 4, 5050: 46 frames) plays out before it walks; meanwhile it keeps
    /// this much of its speed each frame.
    pub appearing_frames: Frames,
    pub appearing_retention: Retention,
    /// Dying plays `d` frame 3 (566, 36 frames).
    pub dying_frames: Frames,
    /// Touching it hurts by half a heart.
    pub contact_damage: Hearts,
    /// Riding the unicorn, or a blast, hurts it by this much.
    pub trample_damage: Damage,
    /// `pathfind()`'s pull towards where it is heading: 0.6 of a step.
    pub chase_strength: f32,
    /// A calm Gaper (a marker with a fractional value) walks at 0.83 of that; a frantic one keeps all
    /// of it, and 1.1 times that while it sees the player.
    pub calm_factor: f32,
    pub frantic_sight_factor: f32,
    /// Once its head's shouting has started (`h` past frame 2) it is 1.2 times as fast.
    pub shouting_factor: f32,
    /// The pull is a tenth of the distance, at most 2 pixels per frame.
    pub pull_per_pixel: f32,
    pub pull_maximum: f32,
    /// Pulling it first keeps 70 % of its speed; with nowhere to go, 50 %; standing on its goal, 85 %.
    pub pulling_retention: Retention,
    pub stuck_retention: Retention,
    pub arrived_retention: Retention,
    /// Its clip shows walking while its speed (times 1) is above 2 pixels per frame.
    pub walking_speed: PixelsPerFrame,
    /// The head clip `h` (5043) has 22 frames: 1 calm, 2 frantic, then the shouting.
    pub head_frame_count: u8,
    /// Every sixth frame (offset by its number in the room) it checks for a clear line to the player,
    /// from tile centre to tile centre.
    pub sight_check_every: Frames,
    pub sight_check_phase: Frames,
    /// Its flood fill over the room's tiles is redone when its timer runs out: at most 20, plus 7,
    /// plus 2 per enemy in the room; each frame spent walking costs 3 times the pull factor, and
    /// each frame seeing the player costs 10.
    pub path_timer_longest: Frames,
    pub path_timer_base: Frames,
    pub path_timer_per_enemy: Frames,
    pub path_timer_per_step: f32,
    pub path_timer_sighted_cost: Frames,
    /// Having a clear line to the player on a look, it growls with 1 chance in 3, at most once
    /// in 40 frames.
    pub growl_spacing: Frames,
    pub growl_odds: Odds,
    /// Nothing is pulled during the room's first frames (`fra > 5`).
    pub idle_frames: Frames,
}

pub const ORIGINAL_GAPERS: GaperParameters = GaperParameters {
    hit_points: HitPoints(10.0),
    size: Pixels(13.0),
    mass: Mass(5.0),
    scale: 0.9,
    appearing_frames: Frames(45.0),
    appearing_retention: Retention(0.45),
    dying_frames: Frames(36.0),
    contact_damage: Hearts::HALF,
    trample_damage: Damage(40.0),
    chase_strength: 0.6,
    calm_factor: 0.83,
    frantic_sight_factor: 1.1,
    shouting_factor: 1.2,
    pull_per_pixel: 0.1,
    pull_maximum: 2.0,
    pulling_retention: Retention(0.7),
    stuck_retention: Retention(0.5),
    arrived_retention: Retention(0.85),
    walking_speed: PixelsPerFrame(2.0),
    head_frame_count: 22,
    sight_check_every: Frames(6.0),
    sight_check_phase: Frames(1.0),
    path_timer_longest: Frames(20.0),
    path_timer_base: Frames(7.0),
    path_timer_per_enemy: Frames(2.0),
    path_timer_per_step: 3.0,
    path_timer_sighted_cost: Frames(10.0),
    growl_spacing: Frames(40.0),
    growl_odds: Odds::one_in(3),
    idle_frames: Frames(5.0),
};

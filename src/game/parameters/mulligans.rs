//! Mulligan (type 16, `b16`): the case for it in `smarts()` (DoAction 24276–24372), fleeing the
//! player with `angstfind()` (9766–9921), bursting when cornered, and leaving flies behind
//! (`flya()`, 10509).

use crate::game::odds::Odds;
use crate::game::original_units::{Damage, Frames, Hearts, HitPoints, Mass, Pixels, PixelsPerFrame, Retention};
use crate::game::texture_id::FlyKind;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MulliganParameters {
    /// `hps[16]`: dies once below zero.
    pub hit_points: HitPoints,
    /// `sizes[16]` and `masses[16]`.
    pub size: Pixels,
    pub mass: Mass,
    pub scale: f32,
    /// The appearing clip (`d` frame 4, 4923: 22 frames).
    pub appearing_frames: Frames,
    pub appearing_retention: Retention,
    /// Dying plays `d` frame 3 (566, 36 frames).
    pub dying_frames: Frames,
    /// Touching it does not hurt: its type is not among those `ballhit()` hurts the player with.
    pub contact_damage: Hearts,
    pub trample_damage: Damage,
    /// Each frame it keeps 80 % of its speed (and 90 % more while wandering).
    pub velocity_retention: Retention,
    pub wandering_retention: Retention,
    /// `angstfind()`: every 14th frame (offset by its number in the room) it looks for the player; one
    /// in clear view within 250 pixels, or hidden within 170, scares it for three such looks.
    pub look_every: Frames,
    pub look_phase: Frames,
    pub sighted_fear_reach: Pixels,
    pub hidden_fear_reach: Pixels,
    pub fear_looks: i32,
    /// Seeing the player it whimpers, once 45 frames plus a random 100 have passed since the last
    /// whimper, one time in four.
    pub whimper_pause_base: Frames,
    pub whimper_pause_range: Frames,
    pub whimper_odds: Odds,
    /// Scared, it is pushed away from the player (while the player is within 500 pixels) by 1.5 over
    /// the distance, and looks for a place to run to: up to 7 tries, each a random 30–480 pixels
    /// from a point 30 pixels further away, out of the player's view.
    pub fear_reach: Pixels,
    pub flee_push: f32,
    pub flee_anchor_distance: Pixels,
    pub flee_tries: f32,
    pub flee_try_penalty: f32,
    pub flee_spread_minimum: Pixels,
    pub flee_spread_range: Pixels,
    pub flee_height_scale: f32,
    pub failed_search_cost: f32,
    pub failed_search_limit: f32,
    pub failed_search_decay: f32,
    /// It gives up its refuge, checking every tenth frame, once the player could see it or is within
    /// 200 pixels of it; it runs there at 1.3 times the Gaper's pull.
    pub refuge_check_every: Frames,
    pub refuge_check_phase: Frames,
    pub refuge_danger_reach: Pixels,
    pub flee_strength: f32,
    /// Pressed against a wall or something else it bursts one frame in 300 (`d` frame 5, 4930: 70
    /// frames), firing four shots when the clip is past its 20th frame and dying.
    pub burst_odds: Odds,
    pub bursting_frames: Frames,
    pub burst_after: Frames,
    pub shot_speed: PixelsPerFrame,
    /// Dying with more than this many hit points left (not blown far past zero) it leaves flies: four,
    /// each at a random spot within 6 pixels, a black fly seven times in eleven, a Pooter once and
    /// an attack fly three times.
    pub flies_above: HitPoints,
    pub fly_count: u8,
    pub fly_spread: Pixels,
    pub fly_kinds: [FlyKind; 11],
}

pub const ORIGINAL_MULLIGANS: MulliganParameters = MulliganParameters {
    hit_points: HitPoints(13.0),
    size: Pixels(13.0),
    mass: Mass(6.0),
    scale: 1.0,
    appearing_frames: Frames(21.0),
    appearing_retention: Retention(0.45),
    dying_frames: Frames(36.0),
    contact_damage: Hearts::ZERO,
    trample_damage: Damage(40.0),
    velocity_retention: Retention(0.8),
    wandering_retention: Retention(0.9),
    look_every: Frames(14.0),
    look_phase: Frames(1.0),
    sighted_fear_reach: Pixels(250.0),
    hidden_fear_reach: Pixels(170.0),
    fear_looks: 3,
    whimper_pause_base: Frames(45.0),
    whimper_pause_range: Frames(100.0),
    whimper_odds: Odds::one_in(4),
    fear_reach: Pixels(500.0),
    flee_push: 1.5,
    flee_anchor_distance: Pixels(30.0),
    flee_tries: 7.0,
    flee_try_penalty: 0.3,
    flee_spread_minimum: Pixels(30.0),
    flee_spread_range: Pixels(450.0),
    flee_height_scale: 0.5,
    failed_search_cost: 0.35,
    failed_search_limit: 20.0,
    failed_search_decay: 0.9,
    refuge_check_every: Frames(10.0),
    refuge_check_phase: Frames(1.0),
    refuge_danger_reach: Pixels(200.0),
    flee_strength: 1.3,
    burst_odds: Odds::one_in(300),
    bursting_frames: Frames(69.0),
    burst_after: Frames(20.0),
    shot_speed: PixelsPerFrame(7.0),
    flies_above: HitPoints(-20.0),
    fly_count: 4,
    fly_spread: Pixels(6.0),
    fly_kinds: [
        FlyKind::Black,
        FlyKind::Black,
        FlyKind::Black,
        FlyKind::Black,
        FlyKind::Black,
        FlyKind::Black,
        FlyKind::Black,
        FlyKind::Pooter,
        FlyKind::Attack,
        FlyKind::Attack,
        FlyKind::Attack,
    ],
};

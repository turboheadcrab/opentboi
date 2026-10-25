//! The Pooter (type 14, `b14`) and enemy shots (type 9, `bullet1`): `alive()`'s fly branch
//! (DoAction 13911–13925), `firemode()` (9515–9690), `shots()` and `ffmo()` (9199–9512), and
//! the shot's flight (14958–15012).

use crate::game::odds::Odds;
use crate::game::original_units::{Frames, Hearts, Pixels, PixelsPerFrame, PixelsPerFrameSquared, Retention};
use crate::game::room::stage_area::Border;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PooterParameters {
    /// `bord()`: kept within stage x 60–575, y 150–410.
    pub border: Border,
    /// `firemode()`: every seventh frame (offset by the enemy's number), with the cooldown spent and
    /// the player within 200 and in sight, it starts a shot (`d` frame 5) and waits 4 such frames
    /// before the next.
    pub shot_check_every: Frames,
    pub sight_reach: Pixels,
    pub cooldown_checks: u8,
    /// The shot leaves 8 frames after it started.
    pub shot_delay: Frames,
    /// Starting a shot nudges its drift towards the player by 0.1.
    pub shot_drift_nudge: f32,
    /// While the cooldown runs it keeps 80 % of its speed and 70 % of its drift each frame.
    pub firing_velocity_retention: Retention,
    pub firing_drift_retention: Retention,
    /// The two-shot Pooter: one roll in 21, 25 or 100 (three rolls), or a marker with a fractional
    /// value; 1.5 times the hit points.
    pub variant_odds: [i32; 3],
    pub variant_hit_point_factor: f32,
    /// A black fly turning into a Pooter is the two-shot one one time in three.
    pub transformed_variant_odds: Odds,
    /// `ffmo()`: the two shots turn the aim by the same small angle each way, 0.8 forward and 0.2
    /// sideways.
    pub spread_forward: f32,
    pub spread_side: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EnemyShotParameters {
    /// The shot heads for the player at 7 pixels per frame.
    pub speed: PixelsPerFrame,
    /// Height (negative is up) and its rate: it starts at 23 above the floor rising at 3; each
    /// frame the rate keeps 90 % of itself and gains 0.2, and the height moves by it.
    pub start_height: Pixels,
    pub start_rise: PixelsPerFrame,
    pub fall_retention: Retention,
    pub fall_gain: PixelsPerFrameSquared,
    pub gravity: PixelsPerFrameSquared,
    /// Lower than this it has reached the floor.
    pub landing_height: Pixels,
    /// The headless body's blood ball (`firewalk()`): 5 pixels per frame ahead, rising at 5, and
    /// falling by 0.3 more each frame.
    pub spurt_speed: PixelsPerFrame,
    pub spurt_start_rise: PixelsPerFrame,
    pub spurt_gravity: PixelsPerFrameSquared,
    /// The size of object type 9.
    pub size: Pixels,
    /// A shot hurts by half a heart.
    pub damage: Hearts,
    /// The shots' brightness, cycling every three frames (`actions2()`).
    pub brightness: [f32; 3],
}

pub const ORIGINAL_POOTERS: PooterParameters = PooterParameters {
    border: Border::ORIGINAL,
    shot_check_every: Frames(7.0),
    sight_reach: Pixels(200.0),
    cooldown_checks: 4,
    shot_delay: Frames(8.0),
    shot_drift_nudge: 0.1,
    firing_velocity_retention: Retention(0.8),
    firing_drift_retention: Retention(0.7),
    variant_odds: [21, 25, 100],
    variant_hit_point_factor: 1.5,
    transformed_variant_odds: Odds::one_in(3),
    spread_forward: 0.8,
    spread_side: 0.2,
};

pub const ORIGINAL_ENEMY_SHOTS: EnemyShotParameters = EnemyShotParameters {
    speed: PixelsPerFrame(7.0),
    start_height: Pixels(-23.0),
    start_rise: PixelsPerFrame(-3.0),
    fall_retention: Retention(0.9),
    fall_gain: PixelsPerFrameSquared(0.1),
    gravity: PixelsPerFrameSquared(0.1),
    landing_height: Pixels(-5.0),
    spurt_speed: PixelsPerFrame(5.0),
    spurt_start_rise: PixelsPerFrame(-5.0),
    spurt_gravity: PixelsPerFrameSquared(0.3),
    size: Pixels(5.0),
    damage: Hearts::HALF,
    brightness: [0.45, 1.0, 1.85],
};

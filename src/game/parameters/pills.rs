//! What pills change (`pillc()`'s use branch, DoAction 27990–28240) and how the stats they move
//! count in the formulas for tears (`firr()`), range, speed (`playsp`) and shot speed.

use crate::game::original_units::{Damage, Frames, Hearts, Pixels, PixelsPerFrame};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PillParameters {
    /// A good pill adds, a bad one takes away, of the stat's own points (`ups[32]`, `ups[31]`,
    /// `ups[27]`, `ups[165]`).
    pub tears_up: f32,
    pub tears_down: f32,
    pub range_up: f32,
    pub range_down: f32,
    pub speed_up: f32,
    pub speed_down: f32,
    pub shot_speed_up: f32,
    pub shot_speed_down: f32,
    /// Balls of Steel: this many soul hearts.
    pub soul_hearts: Hearts,
    /// Pretty Fly: the flies' count grows by this.
    pub fly_gain: f32,
    /// Friends till the end!: this many blue flies.
    pub blue_flies: u8,
    /// A bad trip hurts by this much, even in the blink after a hit.
    pub bad_trip_damage: Hearts,
    /// Each point of tears counts this much in the fire rate's sum (`firr()`).
    pub delay_bonus_per_tear: f32,
    /// Each point of speed counts this much in `playsp` (before its cap).
    pub speed_per_point: f32,
    /// Each point of range lifts a tear's start by this (`dy`) and gives it this much of a first
    /// rise (`dm`), which lifts the start by this fraction of itself again.
    pub range_height_per_point: Pixels,
    pub range_rise_per_point: PixelsPerFrame,
    /// Height gained per pixel per frame of rise: a length of time.
    pub range_height_per_rise: Frames,
    /// Each point of shot speed multiplies a tear's velocity by this much more, within the limits.
    pub shot_speed_per_point: f32,
    pub shot_speed_multiplier_minimum: f32,
    pub shot_speed_multiplier_maximum: f32,
    /// Bad Gas (`fart()`, DoAction 26697–26719): the cloud is drawn this big, hurts every enemy within
    /// this much of Isaac (plus the enemy's own size) once, and poisons it.
    pub gas_cloud_scale: f32,
    pub gas_reach: Pixels,
    pub gas_damage: Damage,
    /// What the poison does: this much every `poison_every` frames, for `poison_frames`.
    pub poison_damage: Damage,
    pub poison_frames: Frames,
    pub poison_every: Frames,
}

pub const ORIGINAL_PILLS: PillParameters = PillParameters {
    tears_up: 0.5,
    tears_down: 0.4,
    range_up: 0.5,
    range_down: 0.4,
    speed_up: 1.0,
    speed_down: 0.8,
    shot_speed_up: 1.0,
    shot_speed_down: 1.4,
    soul_hearts: Hearts(2.0),
    fly_gain: 0.5,
    blue_flies: 3,
    bad_trip_damage: Hearts(1.0),
    delay_bonus_per_tear: 0.7,
    speed_per_point: 0.3,
    range_height_per_point: Pixels(5.0),
    range_rise_per_point: PixelsPerFrame(0.5),
    range_height_per_rise: Frames(0.5),
    shot_speed_per_point: 0.23,
    shot_speed_multiplier_minimum: 0.6,
    shot_speed_multiplier_maximum: 1.42,
    gas_cloud_scale: 0.7,
    gas_reach: Pixels(85.0),
    gas_damage: Damage(5.0),
    poison_damage: Damage(3.5),
    poison_frames: Frames(200.0),
    poison_every: Frames(20.0),
};

//! What cards change for the room they are used in (the room's own item counts, and `doub`).

use crate::game::original_units::{Damage, Frames, Pixels};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CardParameters {
    /// The Devil (`doub`): `dmgdo()` adds 0.75 to the damage bonus.
    pub devil_damage_bonus: f32,
    /// Strength (counted like the Magic Mushroom, item 12): a heart container, 0.3 damage bonus,
    /// 0.3 more speed (up to double), Isaac drawn 25 % larger, and tears start 5 px higher.
    pub strength_damage_bonus: f32,
    pub strength_speed_bonus: f32,
    pub maximum_speed_bonus: f32,
    pub strength_size_bonus: f32,
    pub strength_range_bonus: Pixels,
    /// The Empress (`curs()`, counted as 1.5 of the Whore of Babylon, item 122): `dmgdo()` adds
    /// 1.5 times its square, speed counts it like Strength, and Isaac grows by 25 % for each unit.
    pub empress_strength: f32,
    pub empress_damage_scale: f32,
    /// The unicorn ride, from the Chariot or My Little Unicorn (`unic`): invincible this long,
    /// and faster by this much.
    pub unicorn_ride_frames: Frames,
    pub unicorn_speed_bonus: f32,
    /// Death hurts every enemy in the room by this much.
    pub death_damage: Damage,
    /// The Sun hurts every enemy in the room by this much (DoAction 27854–27870).
    pub sun_damage: Damage,
}

pub const ORIGINAL_CARDS: CardParameters = CardParameters {
    devil_damage_bonus: 0.75,
    strength_damage_bonus: 0.3,
    strength_speed_bonus: 0.3,
    maximum_speed_bonus: 1.0,
    strength_size_bonus: 0.25,
    strength_range_bonus: Pixels(5.0),
    empress_strength: 1.5,
    empress_damage_scale: 1.5,
    unicorn_ride_frames: Frames(180.0),
    unicorn_speed_bonus: 0.28,
    death_damage: Damage(60.0),
    sun_damage: Damage(100.0),
};

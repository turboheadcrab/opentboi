//! What the trinkets add or change: `playsp` (DoAction 28938), the tear damage (DoAction 241,
//! DoAction 245) and the fire delay (`firr()`, DoAction 11062, DoAction 11064), and their effects when Isaac is hurt
//! (`playerhurt()`, DoAction 4781–4885), kills an enemy (`hurt()`, DoAction 4411–4419), clears a room
//! (DoAction 30835–30880), opens a chest (DoAction 31347) or enters a room (DoAction 32842).

use crate::game::odds::Odds;
use crate::game::original_units::{Damage, Frames, Hearts};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TrinketParameters {
    /// The Goat Hoof adds this to the speed stat.
    pub goat_hoof_speed: f32,
    /// The Curved Horn adds this much damage to every tear, after the bonuses are summed.
    pub curved_horn_damage: Damage,
    /// So does the Game Squid, by this much.
    pub game_squid_damage: Damage,
    /// Cancer takes this off the fire delay (a negative number of frames, below the usual minimum).
    pub cancer_delay: Frames,
    /// A hit with the Red Patch held: one time in this many it adds `red_patch_points` to the room's
    /// damage bonus (`razor`), each worth `razor_damage_per_point`.
    pub red_patch_odds: Odds,
    pub red_patch_points: f32,
    pub razor_damage_per_point: f32,
    /// A hit with A Missing Page held: one time in this many it plays the death animation and hurts
    /// every enemy this much.
    pub missing_page_odds: Odds,
    pub missing_page_damage: Damage,
    /// Samson's Lock: a kill raises the room's rage (the damage bonus) by `rage_step` one time in
    /// this many, up to `rage_cap` above the usual.
    pub samsons_lock_odds: Odds,
    pub rage_step: f32,
    pub rage_cap: f32,
    /// The Polaroid shields Isaac this long when a hit leaves him at half a heart or less.
    pub polaroid_shield: Frames,
    /// Mom's Pearl: every heart made is a soul heart one time in this many, besides the usual.
    pub moms_pearl_odds: Odds,
    /// Isaac's Fork heals half a heart one time in this many room-clear rolls.
    pub isaacs_fork_odds: Odds,
    /// The Tick heals this on first entering the boss room.
    pub tick_heal: Hearts,
}

pub const ORIGINAL_TRINKETS: TrinketParameters = TrinketParameters {
    goat_hoof_speed: 0.15,
    curved_horn_damage: Damage(2.0),
    game_squid_damage: Damage(1.0),
    cancer_delay: Frames(-2.0),
    red_patch_odds: Odds::one_in(5),
    red_patch_points: 3.0,
    razor_damage_per_point: 0.6,
    missing_page_odds: Odds::one_in(20),
    missing_page_damage: Damage(40.0),
    samsons_lock_odds: Odds::one_in(15),
    rage_step: 0.5,
    rage_cap: 2.3,
    polaroid_shield: Frames(150.0),
    moms_pearl_odds: Odds::one_in(10),
    isaacs_fork_odds: Odds::one_in(10),
    tick_heal: Hearts::ONE,
};

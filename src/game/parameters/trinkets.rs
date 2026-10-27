//! What the trinkets add to Isaac's stats: `playsp` (DoAction 28938), the tear damage (DoAction 241,
//! DoAction 245) and the fire delay (`firr()`, DoAction 11062).

use crate::game::original_units::{Damage, Frames};

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
}

pub const ORIGINAL_TRINKETS: TrinketParameters = TrinketParameters {
    goat_hoof_speed: 0.15,
    curved_horn_damage: Damage(2.0),
    game_squid_damage: Damage(1.0),
    cancer_delay: Frames(-2.0),
};

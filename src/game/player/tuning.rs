//! What Isaac's stats make of his firing and his tears.

use crate::game::parameters::{FiringParameters, TearParameters};
use crate::game::player::Player;
use crate::game::player::boosts::PlayerStats;

impl Player {
    pub fn set_stats(&mut self, stats: PlayerStats) {
        self.stats = stats;
        let firing = self.firing();
        self.look.set_firing(firing);
    }

    /// The firing rules with what the stats add to the fire rate's sum.
    pub(super) fn firing(&self) -> FiringParameters {
        FiringParameters {
            delay_bonus: self.parameters.firing.delay_bonus + self.stats.delay_bonus,
            ..self.parameters.firing
        }
    }

    /// Tears as this room's boosts make them: their damage sets their size, range their height.
    pub(super) fn boosted_tears(&self) -> TearParameters {
        TearParameters {
            base_damage: self.stats.damage,
            start_height: self.parameters.tear.start_height + self.stats.tear_height_bonus,
            initial_rise: self.stats.tear_rise_bonus,
            speed_multiplier: self.stats.shot_speed_multiplier,
            ..self.parameters.tear
        }
    }
}

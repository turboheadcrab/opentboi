//! Paying blood to the machines.

use super::Inventory;
use crate::game::original_units::Hearts;

impl Inventory {
    /// `ader()` (DoAction 7211–7235): half a heart for a blood machine or the devil beggar, taken
    /// from red hearts while there is more than half a red heart or no soul heart. In a late chapter
    /// a lone half red heart is first made whole with half a soul heart, handed back if it survives.
    pub fn give_blood(&mut self, chapter: u8) -> bool {
        // DoAction 7214: borrowing needs a late chapter, a half red heart and a soul heart.
        let borrows = chapter > self.parameters.late_chapter_after
            && self.red_hearts < Hearts::ONE
            && self.soul_hearts > Hearts::ZERO
            && self.maximum_red_hearts > Hearts::ZERO;
        if borrows {
            // DoAction 7216, DoAction 7217.
            self.soul_hearts -= Hearts::HALF;
            self.red_hearts += Hearts::HALF;
        }
        // DoAction 7220: red hearts alone pay while there is more than half a red heart or no soul heart.
        let hurt = if self.red_hearts <= Hearts::HALF && self.soul_hearts > Hearts::ZERO {
            self.hurt(Hearts::HALF)
        } else {
            let soul_hearts = std::mem::take(&mut self.soul_hearts);
            let hurt = self.hurt(Hearts::HALF);
            self.soul_hearts = soul_hearts;
            hurt
        };
        // DoAction 7233, DoAction 7234: handed back if the red heart is whole again.
        if borrows && self.red_hearts >= Hearts::ONE {
            self.soul_hearts += Hearts::HALF;
            self.red_hearts -= Hearts::HALF;
        }
        hurt
    }
}

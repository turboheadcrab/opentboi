//! What Isaac carries that changes what the room gives: the Lucky Foot (item 46) and luck
//! squeeze the room-clear roll towards the good end, the Lucky Toe does too and adds a
//! roll to chests, and Mom's Pearl turns hearts into soul hearts now and then.

use crate::game::pickups::trinket::Trinket;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PickupLuck {
    pub lucky_foot: bool,
    /// Luck, from pills and items.
    pub luck: i32,
    pub lucky_toe: bool,
    pub moms_pearl: bool,
    /// The trinkets held (`trix`, `atrix`), for the ones that change what a reward is
    /// (see `trinket_swaps`).
    pub trinkets: [Option<Trinket>; 2],
}

impl PickupLuck {
    pub fn holds(self, trinket: Trinket) -> bool {
        self.trinkets.contains(&Some(trinket))
    }

    /// The room-clear roll's random fraction `roll` after luck (DoAction 30835–30858): the Lucky Foot
    /// and good luck each squeeze it into the top nine tenths, bad luck lowers it by a tenth per
    /// point; the Lucky Toe squeezes it again, harder if the foot and good luck already help.
    pub fn squeeze_clear_roll(self, roll: f32) -> f32 {
        const KEPT: f32 = 0.9;
        const KEPT_AGAIN: f32 = 0.98;
        const LOWERED_PER_POINT: f32 = 0.1;
        let toward_good = |roll: f32, kept: f32| roll * kept + (1.0 - kept);
        let mut roll = roll;
        if self.lucky_foot {
            roll = toward_good(roll, KEPT);
        }
        if self.luck > 0 {
            roll = toward_good(roll, KEPT);
        }
        if self.luck < 0 {
            roll += f32::from(i16::try_from(self.luck).unwrap_or(i16::MIN)) * LOWERED_PER_POINT;
        }
        if self.lucky_toe {
            let helped = self.lucky_foot && self.luck > 0;
            roll = toward_good(roll, if helped { KEPT_AGAIN } else { KEPT });
        }
        roll
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_foot_good_luck_and_the_toe_each_squeeze_the_roll_upward() {
        let none = PickupLuck::default();
        assert!((none.squeeze_clear_roll(0.0) - 0.0).abs() < f32::EPSILON);
        let foot = PickupLuck {
            lucky_foot: true,
            ..none
        };
        assert!((foot.squeeze_clear_roll(0.0) - 0.1).abs() < 1e-6);
        let toe = PickupLuck {
            lucky_toe: true,
            ..none
        };
        assert!((toe.squeeze_clear_roll(0.0) - 0.1).abs() < 1e-6);
        let all = PickupLuck {
            lucky_foot: true,
            luck: 1,
            lucky_toe: true,
            moms_pearl: false,
            trinkets: [None, None],
        };
        // Foot, luck, then the toe's lighter squeeze: ((0 * 0.9 + 0.1) * 0.9 + 0.1) * 0.98 + 0.02.
        assert!(
            (all.squeeze_clear_roll(0.0) - 0.2).abs() < 0.01,
            "{}",
            all.squeeze_clear_roll(0.0)
        );
    }

    #[test]
    fn bad_luck_lowers_the_roll() {
        let unlucky = PickupLuck {
            luck: -2,
            ..PickupLuck::default()
        };
        assert!((unlucky.squeeze_clear_roll(0.5) - 0.3).abs() < 1e-6);
    }
}

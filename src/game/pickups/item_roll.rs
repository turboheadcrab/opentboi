//! Which items a pedestal may offer: `itz()` (DoAction 1457–1512) tests each candidate the room's
//! item pool draws, and `itzz()` (DoAction 1452–1456) tells the minor items from the major ones. How
//! many major items Isaac has taken sets the odds of being offered another (DoAction 6788–6798), and
//! the pools the candidates come from are not in the game yet, so nothing draws them. What the
//! pedestal looks like in the devil room (DoAction 1459–1470) waits for the devil's items.

use crate::game::odds::Odds;
use crate::game::pickups::item::ItemNumber;
use crate::game::random::Random;

/// The items that make Isaac stronger by more than a little: offered at the power level's odds, and
/// each taken raises it (everything else, `itzz()`, is minor).
const MAJOR_ITEMS: [u16; 40] = [
    12, 16, 48, 50, 51, 52, 68, 70, 79, 80, 88, 89, 90, 99, 101, 103, 108, 109, 110, 114, 118, 138, 150, 151, 152, 153,
    154, 157, 159, 168, 169, 182, 184, 185, 189, 190, 191, 193, 196, 197,
];
/// The familiars: at most three at a time (DoAction 29621), so a pool offers no fourth.
const FAMILIARS: [u16; 10] = [8, 67, 95, 99, 100, 113, 163, 167, 174, 188];
const MOST_FAMILIARS: u32 = 3;
/// Taking these lowers the power level (DoAction 6790–6793), taking these raises it a little
/// (DoAction 6794–6797).
const POWER_LOWERING: [u16; 4] = [165, 183, 143, 194];
const POWER_RAISING: [u16; 5] = [114, 168, 182, 52, 118];
const LOWERED_BY: f32 = 0.75;
const RAISED_BY: f32 = 0.3;
/// A major item taken raises the power level by one, unless the devil offered it.
const MAJOR_RAISES_BY: f32 = 1.0;
/// Items below 8 are all major but item 5 (My Reflection), which is minor (DoAction 1456).
const FIRST_ORDINARY_ITEM: u16 = 8;
const ALWAYS_MINOR: u16 = 5;
/// Odds of the offer passing at power level `p`: one over `1.2 + 3p`, not worse than one in five.
const BASE_GATE: f32 = 1.2;
const GATE_PER_POWER: f32 = 3.0;
const HARD_MODE_GATE_SHARE: f32 = 0.6;
const HARD_MODE_GATE_PER_SQUARED_POWER: f32 = 0.2;
const WORST_GATE: f32 = 0.2;
/// An item found in an earlier run is turned down one time in this many.
const SEEN_BEFORE_REJECTED: Odds = Odds::one_in(5);

/// What the roll needs to know about the run, and about the candidate.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OfferFacts {
    /// `wiptz`: how strong Isaac has become by the major items taken.
    pub power: f32,
    pub hard_mode: bool,
    /// Isaac has the candidate already.
    pub held: bool,
    /// How many familiars he has, if the candidate is one.
    pub familiars: u32,
    /// The candidate was found in a run before (the save's `colss`).
    pub found_before: bool,
}

/// Whether `item` is minor: it needs no luck to be offered (`itzz()`).
pub fn is_minor(item: ItemNumber) -> bool {
    item.0 == ALWAYS_MINOR || (item.0 >= FIRST_ORDINARY_ITEM && !MAJOR_ITEMS.contains(&item.0))
}

/// Whether `item` is a familiar.
pub fn is_familiar(item: ItemNumber) -> bool {
    FAMILIARS.contains(&item.0)
}

/// The familiars, for counting those held.
pub fn familiars() -> impl Iterator<Item = ItemNumber> {
    FAMILIARS.into_iter().map(ItemNumber)
}

/// `itz()`: whether the pool's candidate `item` is offered, given `facts`; a refusal sends the pool
/// to its next draw.
pub fn accepts(item: ItemNumber, facts: OfferFacts, random: &mut Random) -> bool {
    if is_familiar(item) && facts.familiars > MOST_FAMILIARS - 1 {
        return false;
    }
    if facts.held || (facts.found_before && random.happens(SEEN_BEFORE_REJECTED)) {
        return false;
    }
    is_minor(item) || gate_odds(facts) > random.next_unit()
}

/// The chance a major item passes: it falls as the power level rises; in hard mode, with some power,
/// the gate is smaller, so the odds are better.
fn gate_odds(facts: OfferFacts) -> f32 {
    let mut gate = BASE_GATE + facts.power * GATE_PER_POWER;
    if facts.hard_mode && facts.power > 0.0 {
        gate = gate * HARD_MODE_GATE_SHARE + facts.power * facts.power * HARD_MODE_GATE_PER_SQUARED_POWER;
    }
    1.0 / gate.max(WORST_GATE)
}

/// How taking the passive `item` changes the power level (DoAction 6788–6798); `from_devil` for an
/// item the devil offered, which does not count.
pub fn power_change(item: ItemNumber, from_devil: bool) -> f32 {
    let mut change = 0.0;
    if !is_minor(item) && !from_devil {
        change += MAJOR_RAISES_BY;
    }
    if POWER_LOWERING.contains(&item.0) {
        change -= LOWERED_BY;
    }
    if POWER_RAISING.contains(&item.0) {
        change += RAISED_BY;
    }
    change
}

#[cfg(test)]
mod tests {
    use super::*;

    fn facts(power: f32) -> OfferFacts {
        OfferFacts {
            power,
            hard_mode: false,
            held: false,
            familiars: 0,
            found_before: false,
        }
    }

    #[test]
    fn the_first_items_and_the_ordinary_ones_are_minor_and_the_strong_ones_are_not() {
        assert!(is_minor(ItemNumber(5)));
        assert!(!is_minor(ItemNumber(3)), "the early stat items cost luck too");
        assert!(is_minor(ItemNumber(46)));
        assert!(!is_minor(ItemNumber(114)));
        assert!(!is_minor(ItemNumber(12)));
    }

    #[test]
    fn a_minor_item_is_offered_unless_held_and_a_major_one_gets_rarer_with_power() {
        let mut random = Random::with_seed(3);
        let offered = |item, facts, random: &mut Random| (0..2000).filter(|_| accepts(item, facts, random)).count();
        assert_eq!(offered(ItemNumber(46), facts(5.0), &mut random), 2000);
        let held = OfferFacts {
            held: true,
            ..facts(0.0)
        };
        assert_eq!(offered(ItemNumber(46), held, &mut random), 0);
        let weak = offered(ItemNumber(114), facts(0.0), &mut random);
        let strong = offered(ItemNumber(114), facts(2.0), &mut random);
        assert!(weak > 1500, "{weak}");
        assert!(strong < weak / 3, "{strong}");
    }

    #[test]
    fn hard_mode_eases_the_fall_of_the_odds_and_an_item_found_before_is_sometimes_refused() {
        let mut random = Random::with_seed(4);
        let offered =
            |facts, random: &mut Random| (0..4000).filter(|_| accepts(ItemNumber(114), facts, random)).count();
        let easy = offered(facts(1.0), &mut random);
        let hard = offered(
            OfferFacts {
                hard_mode: true,
                ..facts(1.0)
            },
            &mut random,
        );
        assert!(hard > easy, "{hard} against {easy}");
        let seen = offered(
            OfferFacts {
                found_before: true,
                ..facts(0.0)
            },
            &mut random,
        );
        // Four in five get past the memory of the last run, and 1 / 1.2 of those the gate.
        assert!((2500..2850).contains(&seen), "{seen}");
    }

    #[test]
    fn a_fourth_familiar_is_never_offered() {
        let mut random = Random::with_seed(5);
        let three = OfferFacts {
            familiars: 3,
            ..facts(0.0)
        };
        assert!((0..500).all(|_| !accepts(ItemNumber(8), three, &mut random)));
        let two = OfferFacts {
            familiars: 2,
            ..facts(0.0)
        };
        assert!(accepts(ItemNumber(8), two, &mut random));
    }

    #[test]
    fn taking_a_major_item_raises_the_power_unless_the_devil_offered_it() {
        assert!((power_change(ItemNumber(79), false) - 1.0).abs() < f32::EPSILON);
        assert!(power_change(ItemNumber(79), true).abs() < f32::EPSILON);
        assert!((power_change(ItemNumber(114), false) - 1.3).abs() < 1e-6);
        assert!((power_change(ItemNumber(165), false) + 0.75).abs() < 1e-6);
        assert!(power_change(ItemNumber(46), false).abs() < f32::EPSILON);
    }
}

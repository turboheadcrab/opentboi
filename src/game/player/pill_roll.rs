//! The roll of a pill's effect (`piller()`, DoAction 26755–26836): a chain of odds, the first to come
//! up wins, each link depending on how Isaac is doing and what he carries.

use crate::game::odds::Odds;
use crate::game::original_units::Hearts;
use crate::game::player::pill_state::{PillEffect, PillKind};
use crate::game::random::Random;

/// What the roll of an effect depends on (`piller()`).
#[derive(Clone, Copy, Debug)]
#[allow(
    clippy::struct_excessive_bools,
    reason = "independent facts the roll asks about, each read on its own"
)]
pub struct PillRollContext {
    pub hurt: bool,
    /// Red and soul hearts together, in whole hearts.
    pub hearts_in_all: Hearts,
    pub has_bombs_or_keys: bool,
    /// The Lucky Foot or PHD: every pill helps, and none hurts (`f10`, DoAction 26757).
    pub lucky: bool,
    /// PHD (item 75): also no Puberty or Explosive Diarrhea, and a sure Speed pill when the chain gets
    /// to it (DoAction 26761, DoAction 26779, DoAction 26783, DoAction 26815).
    pub phd: bool,
}

/// The odds in `piller()`'s chain, each "one in" this many (DoAction 26763, DoAction 26767, DoAction 26769, DoAction 26771, DoAction 26773, DoAction 26775, DoAction 26777, DoAction 26781, DoAction 26785, DoAction 26787, DoAction 26789, DoAction 26791, DoAction 26793, DoAction 26795, DoAction 26797, DoAction 26799, DoAction 26801, DoAction 26803, DoAction 26805, DoAction 26807, DoAction 26809, DoAction 26811, DoAction 26813, DoAction 26817, DoAction 26819, DoAction 26821).
const FULL_HEALTH_ODDS: Odds = Odds::one_in(10);
const BAD_GAS_ODDS: Odds = Odds::one_in(13);
const FRIENDS_ODDS: Odds = Odds::one_in(17);
const LUCK_ODDS: Odds = Odds::one_in(17);
const PUBERTY_ODDS: Odds = Odds::one_in(15);
const DIARRHEA_ODDS: Odds = Odds::one_in(15);
const BALLS_OF_STEEL_ODDS: Odds = Odds::one_in(10);
const PRETTY_FLY_ODDS: Odds = Odds::one_in(20);
const BAD_TRIP_ODDS: Odds = Odds::one_in(9);
const HEALTH_ODDS: Odds = Odds::one_in(7);
const BOMBS_ARE_KEYS_ODDS: Odds = Odds::one_in(6);
const TEARS_ODDS: Odds = Odds::one_in(5);
const TEARS_SECOND_CHANCE_ODDS: Odds = Odds::one_in(4);
const OTHER_STATS_ODDS: Odds = Odds::one_in(3);
/// A bad trip needs more than a heart of health to spare.
const BAD_TRIP_MINIMUM_HEARTS: Hearts = Hearts::ONE;
/// Either version, "one in" this many.
const GOOD_VERSION_ODDS: Odds = Odds::one_in(2);

/// The chain of odds in `piller()`: each link is tried in turn, and the first to come up wins.
pub(super) fn roll_effect(context: PillRollContext, random: &mut Random) -> PillEffect {
    if context.hurt && random.happens(FULL_HEALTH_ODDS) {
        return plain(PillKind::FullHealth);
    }
    for (odds, kind) in [(BAD_GAS_ODDS, PillKind::BadGas), (FRIENDS_ODDS, PillKind::Friends)] {
        if random.happens(odds) {
            return plain(kind);
        }
    }
    if random.happens(LUCK_ODDS) {
        return either(PillKind::Luck, random);
    }
    for (odds, kind) in [
        (PUBERTY_ODDS, PillKind::Puberty),
        (DIARRHEA_ODDS, PillKind::Diarrhea),
        (BALLS_OF_STEEL_ODDS, PillKind::BallsOfSteel),
        (PRETTY_FLY_ODDS, PillKind::PrettyFly),
    ] {
        let phd_refuses = context.phd && matches!(kind, PillKind::Puberty | PillKind::Diarrhea);
        if random.happens(odds) && !phd_refuses {
            return plain(kind);
        }
    }
    if random.happens(BAD_TRIP_ODDS) && context.hearts_in_all > BAD_TRIP_MINIMUM_HEARTS && !context.lucky {
        return plain(PillKind::BadTrip);
    }
    if random.happens(HEALTH_ODDS) {
        return either(PillKind::Health, random);
    }
    if random.happens(BOMBS_ARE_KEYS_ODDS) && context.has_bombs_or_keys {
        return plain(PillKind::BombsAreKeys);
    }
    if random.happens(TEARS_ODDS) || random.happens(TEARS_SECOND_CHANCE_ODDS) {
        return either(PillKind::Tears, random);
    }
    for kind in [PillKind::Range, PillKind::Speed, PillKind::ShotSpeed] {
        let rolled = random.happens(OTHER_STATS_ODDS);
        if rolled || (context.phd && kind == PillKind::Speed) {
            return either(kind, random);
        }
    }
    plain(PillKind::TelePills)
}

fn plain(kind: PillKind) -> PillEffect {
    PillEffect { kind, good: false }
}

/// A kind that has a good and a bad version, either one.
fn either(kind: PillKind, random: &mut Random) -> PillEffect {
    PillEffect {
        kind,
        good: random.happens(GOOD_VERSION_ODDS),
    }
}

#[cfg(test)]
pub(super) const TEST_CONTEXT: PillRollContext = PillRollContext {
    hurt: true,
    hearts_in_all: Hearts(3.0),
    has_bombs_or_keys: true,
    lucky: false,
    phd: false,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_lucky_foot_and_full_health_leave_out_the_bad_trip_and_the_heal() {
        let healthy_and_lucky = PillRollContext {
            hurt: false,
            lucky: true,
            ..TEST_CONTEXT
        };
        let mut random = Random::with_seed(9);
        for _ in 0..2000 {
            let effect = roll_effect(healthy_and_lucky, &mut random);
            assert!(!matches!(effect.kind, PillKind::BadTrip | PillKind::FullHealth));
        }
    }

    #[test]
    /// DoAction 26779, 26783 and 26815: PHD never gives Puberty or Explosive Diarrhea, and always
    /// a Speed pill where the chain reaches it.
    fn phd_leaves_out_puberty_and_diarrhea_and_makes_speed_sure() {
        let phd = PillRollContext {
            lucky: true,
            phd: true,
            ..TEST_CONTEXT
        };
        let mut random = Random::with_seed(11);
        let kinds: Vec<PillKind> = (0..4000).map(|_| roll_effect(phd, &mut random).kind).collect();
        assert!(!kinds.contains(&PillKind::Puberty) && !kinds.contains(&PillKind::Diarrhea));
        assert!(!kinds.contains(&PillKind::ShotSpeed) && !kinds.contains(&PillKind::TelePills));
        assert!(kinds.contains(&PillKind::Speed));
    }

    #[test]
    fn only_some_kinds_have_a_bad_version() {
        let mut random = Random::with_seed(3);
        for _ in 0..2000 {
            let effect = roll_effect(TEST_CONTEXT, &mut random);
            assert!(effect.kind.has_good_and_bad() || !effect.good, "{effect:?}");
        }
    }
}

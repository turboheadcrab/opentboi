//! What the pills have done this run: which effect each look has (`piller`, assigned the first time
//! the look is used) and the stat changes they left (`ups`).

use crate::game::odds::Odds;
use crate::game::original_units::Hearts;
use crate::game::pickups::pill::Pill;
use crate::game::random::Random;

/// What a pill does, by its number in `piller()`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PillKind {
    /// 1: all red hearts back.
    FullHealth,
    /// 2: a hat, or the pill-eater's face.
    Puberty,
    /// 3: bombs for a while ("Explosive Diarrhea!").
    Diarrhea,
    /// 4: two soul hearts.
    BallsOfSteel,
    /// 5: another fly circling Isaac.
    PrettyFly,
    /// 6: a heart of damage.
    BadTrip,
    /// 7: a heart container more or less.
    Health,
    /// 8: bombs and keys trade places.
    BombsAreKeys,
    /// 9, 10, 11 and 17: a stat up or down.
    Tears,
    Range,
    Speed,
    ShotSpeed,
    /// 12: a cloud of poison.
    BadGas,
    /// 13: blue flies.
    Friends,
    /// 14: one luck more or less.
    Luck,
    /// 126: a teleport.
    TelePills,
}

impl PillKind {
    /// The kinds the original gives a good and a bad version (`n + random(2) * 0.2`).
    #[cfg(test)]
    pub fn has_good_and_bad(self) -> bool {
        matches!(
            self,
            Self::Health | Self::Tears | Self::Range | Self::Speed | Self::ShotSpeed | Self::Luck
        )
    }
}

/// A look's effect: `good` is the version that helps, for kinds that have both (the original's
/// `.2` fraction).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PillEffect {
    pub kind: PillKind,
    pub good: bool,
}

/// What the roll of an effect depends on (`piller()`).
#[derive(Clone, Copy, Debug)]
pub struct PillRollContext {
    pub hurt: bool,
    /// Red and soul hearts together, in whole hearts.
    pub hearts_in_all: Hearts,
    pub has_bombs_or_keys: bool,
    /// The Lucky Foot: every pill helps, and none hurts.
    pub lucky: bool,
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

/// Rolled again until it differs from every effect already given to a look.
const MAXIMUM_ROLLS: u32 = 1000;

#[derive(Clone, Debug, Default)]
pub struct PillState {
    effects: [Option<PillEffect>; Pill::COUNT as usize],
    /// `ups[32]`, `ups[31]`, `ups[27]`, `ups[165]`: tears, range, speed and shot speed gained.
    pub tears: f32,
    pub range: f32,
    pub speed: f32,
    pub shot_speed: f32,
    /// How lucky Isaac is (`luck`).
    pub luck: i32,
    /// `ups[10]`: the flies circling Isaac, in halves.
    pub flies: f32,
}

impl PillState {
    pub fn effect_of(&self, pill: Pill) -> Option<PillEffect> {
        self.effects[pill.index()]
    }

    /// For tests: `pill` has `effect` already.
    #[cfg(test)]
    pub fn give_effect(&mut self, pill: Pill, effect: PillEffect) {
        self.effects[pill.index()] = Some(effect);
    }

    /// `piller()`: the effect of `pill`, given to it now if it had none.
    pub fn assign(&mut self, pill: Pill, context: PillRollContext, random: &mut Random) -> PillEffect {
        if let Some(effect) = self.effect_of(pill) {
            return effect;
        }
        let mut effect = roll_effect(context, random);
        for _ in 0..MAXIMUM_ROLLS {
            if !self.effects.contains(&Some(effect)) {
                break;
            }
            effect = roll_effect(context, random);
        }
        self.effects[pill.index()] = Some(effect);
        effect
    }
}

/// The chain of odds in `piller()`: each link is tried in turn, and the first to come up wins.
fn roll_effect(context: PillRollContext, random: &mut Random) -> PillEffect {
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
        if random.happens(odds) {
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
        if random.happens(OTHER_STATS_ODDS) {
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
mod tests {
    use super::*;

    const CONTEXT: PillRollContext = PillRollContext {
        hurt: true,
        hearts_in_all: Hearts(3.0),
        has_bombs_or_keys: true,
        lucky: false,
    };

    #[test]
    fn each_look_gets_its_own_effect_and_keeps_it() {
        let mut state = PillState::default();
        let mut random = Random::with_seed(5);
        let effects: Vec<PillEffect> = Pill::ALL
            .iter()
            .map(|&pill| state.assign(pill, CONTEXT, &mut random))
            .collect();
        for (index, effect) in effects.iter().enumerate() {
            assert!(!effects[index + 1..].contains(effect), "{effects:?}");
        }
        assert_eq!(state.assign(Pill::ALL[2], CONTEXT, &mut random), effects[2]);
        assert_eq!(state.effect_of(Pill::ALL[0]), Some(effects[0]));
    }

    #[test]
    fn the_lucky_foot_and_full_health_leave_out_the_bad_trip_and_the_heal() {
        let healthy_and_lucky = PillRollContext {
            hurt: false,
            lucky: true,
            ..CONTEXT
        };
        let mut random = Random::with_seed(9);
        for _ in 0..2000 {
            let effect = roll_effect(healthy_and_lucky, &mut random);
            assert!(!matches!(effect.kind, PillKind::BadTrip | PillKind::FullHealth));
        }
    }

    #[test]
    fn only_some_kinds_have_a_bad_version() {
        let mut random = Random::with_seed(3);
        for _ in 0..2000 {
            let effect = roll_effect(CONTEXT, &mut random);
            assert!(effect.kind.has_good_and_bad() || !effect.good, "{effect:?}");
        }
    }
}

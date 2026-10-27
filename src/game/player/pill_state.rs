//! What the pills have done this run: which effect each look has (`piller`, assigned the first time
//! the look is used) and the stat changes they left (`ups`).

use crate::game::pickups::pill::Pill;
use crate::game::player::pill_roll::{PillRollContext, roll_effect};
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

/// Rolled again until it differs from every effect already given to a look (the loop over the effects
/// given, DoAction 26827–26833).
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

    /// `piller()`: the effect of `pill`, given to it now if it had none (DoAction 26761).
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::player::pill_roll::TEST_CONTEXT;

    #[test]
    fn each_look_gets_its_own_effect_and_keeps_it() {
        let mut state = PillState::default();
        let mut random = Random::with_seed(5);
        let effects: Vec<PillEffect> = Pill::ALL
            .iter()
            .map(|&pill| state.assign(pill, TEST_CONTEXT, &mut random))
            .collect();
        for (index, effect) in effects.iter().enumerate() {
            assert!(!effects[index + 1..].contains(effect), "{effects:?}");
        }
        assert_eq!(state.assign(Pill::ALL[2], TEST_CONTEXT, &mut random), effects[2]);
        assert_eq!(state.effect_of(Pill::ALL[0]), Some(effects[0]));
    }
}

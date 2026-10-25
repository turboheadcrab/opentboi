//! What chests, the tinted rock, golden poop, broken poop and fireplaces, and cleared rooms give
//! (`chestopen` handling, `killshit()`, and the room-clear roll when the doors open). Trinkets do
//! not exist yet: where the original gives one, nothing appears; items get an empty pedestal.
//! Rolls assume no luck, items or trinkets.

use crate::game::odds::Odds;
use crate::game::pickups::kind::{
    BOMB_FRAME, BROWN_CHEST_FRAME, CARD_MARKER_FRAME, COIN_FRAME, GOLDEN_CHEST_FRAME, HEART_FRAME, ITEM_PEDESTAL_FRAME,
    KEY_FRAME, PILL_FRAME,
};
use crate::game::random::Random;

const TRINKET_ODDS: Odds = Odds::one_in(2);
/// The room-clear roll's pill, card or trinket: a card one time in three, else a trinket one time in
/// two, else a pill.
const ROOM_CLEAR_CARD_ODDS: Odds = Odds::one_in(3);

/// Chest rolls: a golden chest rolls one less than a roll below 8 times, at least 2; a brown one
/// one less than a roll below 4; each roll a random fraction against these thresholds.
const GOLDEN_CHEST_ROLL_RANGE: i32 = 8;
const CHEST_ROLL_RANGE: i32 = 4;
const MINIMUM_CHEST_ROLLS: i32 = 2;
const BLOOD_MACHINE_EXTRA_ROLLS: i32 = 2;
const TRINKET_CHECK_ODDS: Odds = Odds::one_in(3);
const ITEM_ODDS: Odds = Odds::one_in(5);
const MORE_COINS_RANGE: u8 = 3;
const FEWER_ROLLS_ODDS: Odds = Odds::one_in(2);
const CHEST_COIN_BELOW: f32 = 0.35;
const CHEST_HEART_BELOW: f32 = 0.55;
const TINTED_HEART_BELOW: f32 = 0.6;
const CHEST_KEY_BELOW: f32 = 0.7;
const CHEST_BROWN_CHEST_BELOW: f32 = 0.71;
const CHEST_GOLDEN_CHEST_BELOW: f32 = 0.72;
const TINTED_GOLDEN_CHEST_BELOW: f32 = 0.9;

/// Pickups to create: `count` of `b5` frame `frame`. `soul_heart` turns hearts into soul hearts;
/// `thrown` ones fly out (`cspawn()`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Drop {
    pub frame: u8,
    pub count: u8,
    pub soul_heart: bool,
}

impl Drop {
    pub fn is_chest(self) -> bool {
        matches!(self.frame, BROWN_CHEST_FRAME | GOLDEN_CHEST_FRAME | ITEM_PEDESTAL_FRAME)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RewardSource {
    BrownChest,
    GoldenChest,
    /// `chestopen = 2`.
    TintedRock,
    /// A blood donation machine broken open: like a brown chest with one or two rolls.
    BloodDonationMachine,
}

/// `killshit()`: a poop or fireplace broken by tears (not blasts) drops something one time in
/// ten: a heart one time in four, else a coin.
pub fn breakable_drop(random: &mut Random) -> Option<u8> {
    const DROP_CHANCE: f32 = 0.1;
    const HEART_CHANCE: f32 = 0.25;
    (random.next_unit() < DROP_CHANCE).then(|| {
        if random.next_unit() < HEART_CHANCE {
            HEART_FRAME
        } else {
            COIN_FRAME
        }
    })
}

/// Golden poop opens like a chest that holds 5 to 7 coins (`cspawn()`).
pub fn golden_poop_coins(random: &mut Random) -> Drop {
    const MINIMUM_COINS: u8 = 5;
    const EXTRA_COINS: u8 = 3;
    Drop {
        frame: COIN_FRAME,
        count: MINIMUM_COINS + random.below_u8(EXTRA_COINS),
        soul_heart: false,
    }
}

/// The room-clear roll (chapter 1), a random fraction: nothing below 0.22, a pill, card or trinket
/// below 0.3, then a coin below 0.45, a heart below 0.58 (or 1 in 30 when missing more than a
/// heart), a key below 0.76 less 0.007 per chapter and 0.01 per key held (up to 10), a bomb below
/// 0.95, else a chest. The Lucky Foot squeezes the roll into its top nine tenths.
pub fn room_clear_reward(
    random: &mut Random,
    keys: u8,
    missing_more_than_a_heart: bool,
    lucky_foot: bool,
) -> Option<u8> {
    const NOTHING_BELOW: f32 = 0.22;
    const TRINKET_BELOW: f32 = 0.3;
    const COIN_BELOW: f32 = 0.45;
    const HEART_BELOW: f32 = 0.58;
    const LOW_HEALTH_HEART_ODDS: Odds = Odds::one_in(30);
    const KEY_BELOW: f32 = 0.76 - 0.007;
    const KEY_LESS_PER_KEY: f32 = 0.01;
    const KEYS_COUNTED: u8 = 10;
    const BOMB_BELOW: f32 = 0.95;
    const LUCKY_FOOT_KEPT: f32 = 0.9;
    let roll = random.next_unit();
    let roll = if lucky_foot {
        roll * LUCKY_FOOT_KEPT + (1.0 - LUCKY_FOOT_KEPT)
    } else {
        roll
    };
    if roll <= NOTHING_BELOW {
        return None;
    }
    if roll < TRINKET_BELOW {
        return room_clear_consumable(random);
    }
    let reward = if roll < COIN_BELOW {
        COIN_FRAME
    } else if roll < HEART_BELOW || (random.happens(LOW_HEALTH_HEART_ODDS) && missing_more_than_a_heart) {
        HEART_FRAME
    } else if roll < KEY_BELOW - f32::from(keys.min(KEYS_COUNTED)) * KEY_LESS_PER_KEY {
        KEY_FRAME
    } else if roll < BOMB_BELOW {
        BOMB_FRAME
    } else {
        BROWN_CHEST_FRAME
    };
    Some(reward)
}

/// A card, a pill, or (not made yet, so nothing) a trinket.
fn room_clear_consumable(random: &mut Random) -> Option<u8> {
    if random.happens(ROOM_CLEAR_CARD_ODDS) {
        Some(CARD_MARKER_FRAME)
    } else if random.happens(TRINKET_ODDS) {
        None
    } else {
        Some(PILL_FRAME)
    }
}

/// A chest's or the tinted rock's contents.
pub fn contents(source: RewardSource, random: &mut Random) -> Vec<Drop> {
    let tinted = source == RewardSource::TintedRock;
    let golden = source == RewardSource::GoldenChest;
    let roll_range = if golden {
        GOLDEN_CHEST_ROLL_RANGE
    } else {
        CHEST_ROLL_RANGE
    };
    let mut rolls = if source == RewardSource::BloodDonationMachine {
        1 + random.below(BLOOD_MACHINE_EXTRA_ROLLS)
    } else {
        (random.below(roll_range) - 1).max(MINIMUM_CHEST_ROLLS)
    };
    // A roll for trinkets (`trixies()`): only matters with trinkets.
    random.happens(TRINKET_CHECK_ODDS);
    let mut drops = Vec::new();
    let mut roll_index = 0;
    while roll_index < rolls {
        let first = roll_index == 0;
        let roll = random.next_unit();
        // An item (golden chest, one time in five): it ends the chest. The random items do not exist
        // yet, so its pedestal stands empty.
        if random.happens(ITEM_ODDS) && golden && first {
            drops.push(Drop {
                frame: ITEM_PEDESTAL_FRAME,
                count: 1,
                soul_heart: false,
            });
            break;
        }
        // A pill (brown chest), card (golden chest) or, one time in two, a trinket on the first roll
        // ends it too; trinkets do not exist yet.
        if random.happens(ITEM_ODDS) && first && !tinted {
            let trinket = random.happens(TRINKET_ODDS);
            if !trinket {
                drops.push(Drop {
                    frame: if golden { CARD_MARKER_FRAME } else { PILL_FRAME },
                    count: 1,
                    soul_heart: false,
                });
            }
            break;
        }
        let drop = chest_drop(roll, first, tinted, &mut rolls, random);
        drops.push(drop);
        roll_index += 1;
    }
    drops
}

/// One roll of a chest: what `f1` gives, adjusting how many rolls are left.
fn chest_drop(roll: f32, first: bool, tinted: bool, rolls: &mut i32, random: &mut Random) -> Drop {
    let one = |frame| Drop {
        frame,
        count: 1,
        soul_heart: false,
    };
    if roll < CHEST_COIN_BELOW && !tinted {
        Drop {
            count: random.below_u8(MORE_COINS_RANGE) + 1,
            ..one(COIN_FRAME)
        }
    } else if roll < CHEST_HEART_BELOW || (roll < TINTED_HEART_BELOW && tinted) {
        if random.happens(FEWER_ROLLS_ODDS) {
            *rolls -= 1;
        }
        Drop {
            soul_heart: tinted,
            ..one(HEART_FRAME)
        }
    } else if roll < CHEST_KEY_BELOW {
        if tinted {
            *rolls = -1;
        }
        one(KEY_FRAME)
    } else if roll < CHEST_BROWN_CHEST_BELOW && first {
        *rolls = -1;
        one(BROWN_CHEST_FRAME)
    } else if (roll < CHEST_GOLDEN_CHEST_BELOW || (tinted && roll < TINTED_GOLDEN_CHEST_BELOW)) && first {
        *rolls = -1;
        one(GOLDEN_CHEST_FRAME)
    } else {
        one(BOMB_FRAME)
    }
}

#[cfg(test)]
mod tests;

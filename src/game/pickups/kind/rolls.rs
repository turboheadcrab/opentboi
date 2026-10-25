//! The variant rolls of `create()` for hearts, coins and bombs.

use super::{BombKind, CoinKind, HeartKind};
use crate::game::odds::Odds;
use crate::game::random::Random;

/// The second variant (half hearts, nickels, double bombs) comes one time in 2, 20 and 7.
const HALF_HEART_ODDS: Odds = Odds::one_in(2);
const NICKEL_ODDS: Odds = Odds::one_in(20);
const DOUBLE_BOMB_ODDS: Odds = Odds::one_in(7);
/// One in 50 a mega troll bomb; one in 100 a dime (fewer with luck, at least 50); one in 50 an
/// eternal heart (fewer with luck, at least 20); one in 10 a troll bomb or a soul heart; keys: one
/// in 50 golden.
const MEGA_TROLL_BOMB_ODDS: Odds = Odds::one_in(50);
const DIME_ODDS: Odds = Odds::one_in(100);
const ETERNAL_HEART_ODDS: Odds = Odds::one_in(50);
const TROLL_BOMB_ODDS: Odds = Odds::one_in(10);
const SOUL_HEART_ODDS: Odds = Odds::one_in(10);
pub(super) const GOLDEN_KEY_ODDS: Odds = Odds::one_in(50);

pub(super) fn roll_heart(random: &mut Random) -> HeartKind {
    let mut heart = if random.happens(HALF_HEART_ODDS) {
        HeartKind::Half
    } else {
        HeartKind::Full
    };
    if random.happens(ETERNAL_HEART_ODDS) {
        heart = HeartKind::Eternal;
    }
    if random.happens(SOUL_HEART_ODDS) {
        heart = HeartKind::Soul;
    }
    heart
}

pub(super) fn roll_coin(random: &mut Random) -> CoinKind {
    let mut coin = if random.happens(NICKEL_ODDS) {
        CoinKind::Nickel
    } else {
        CoinKind::Penny
    };
    if random.happens(DIME_ODDS) {
        coin = CoinKind::Dime;
    }
    coin
}

pub(super) fn roll_bomb(random: &mut Random) -> BombKind {
    let mut bomb = if random.happens(DOUBLE_BOMB_ODDS) {
        BombKind::Double
    } else {
        BombKind::Single
    };
    if random.happens(MEGA_TROLL_BOMB_ODDS) {
        bomb = BombKind::MegaTroll;
    }
    if random.happens(TROLL_BOMB_ODDS) {
        bomb = BombKind::Troll;
    }
    bomb
}

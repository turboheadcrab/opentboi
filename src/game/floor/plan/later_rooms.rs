//! Rooms only later floors have (DoAction 32213–32232): the second treasure room, the challenge
//! room (a boss rush on some floors) and the arcade. Placeholders: the run knows nothing yet of
//! the state these wait on, so every condition is the one the first floor has, and chapter 1's
//! rolls are unchanged. Replace each with the real room.

use crate::game::floor::cell::FloorCell;
use crate::game::floor::plan::{Growth, RoomRole};
use crate::game::odds::Odds;
use crate::game::random::Random;

/// `double`: whether the floor is the first half of a two-part chapter (not modelled yet).
const DOUBLE_FLOOR: bool = false;
/// `fullhp`: Isaac's hearts are full, which the first layout is made before knowing.
const FULL_HEALTH: bool = false;
/// `coins` at layout time.
const COINS: u32 = 0;
const ARCADE_MINIMUM_COINS: u32 = 5;

/// The coins Isaac carries when the floor is laid out (none known yet).
fn coins_at_layout() -> u32 {
    COINS
}
/// `altch == 2`: the hard-mode variant that makes the challenge room a boss rush.
const BOSS_RUSH: bool = false;
const CHALLENGE_FIRST_ODDS: Odds = Odds::EVEN;
/// The challenge room is always rolled in from this chapter on, else by the odds above.
const CHALLENGE_ALWAYS_FROM_CHAPTER: u8 = 3;
const ARCADE_CHAPTER_STEP: u8 = 2;

impl Growth {
    /// The second treasure room (`boner2`), taken right after the first.
    pub(super) fn second_treasure_room(&mut self, random: &mut Random) -> Option<FloorCell> {
        DOUBLE_FLOOR.then(|| self.take_dead_end(random)).flatten()
    }

    /// The challenge room (`chamb`) and the arcade (`gamb`), after the mini-boss: each takes a
    /// dead end if one is left. Returns the role the challenge room has and the arcade's cell.
    pub(super) fn challenge_and_arcade(
        &mut self,
        chapter: u8,
        random: &mut Random,
    ) -> [(Option<FloorCell>, RoomRole); 2] {
        let challenge = (!self.dead_ends.is_empty()
            && FULL_HEALTH
            && chapter > 1
            && (random.happens(CHALLENGE_FIRST_ODDS) || chapter >= CHALLENGE_ALWAYS_FROM_CHAPTER))
            .then(|| self.take_dead_end(random))
            .flatten();
        let arcade = (!self.dead_ends.is_empty()
            && coins_at_layout() >= ARCADE_MINIMUM_COINS
            && chapter.is_multiple_of(ARCADE_CHAPTER_STEP))
        .then(|| self.take_dead_end(random))
        .flatten();
        let challenge_role = if BOSS_RUSH {
            RoomRole::BossRush
        } else {
            RoomRole::Challenge
        };
        [(challenge, challenge_role), (arcade, RoomRole::Arcade)]
    }
}

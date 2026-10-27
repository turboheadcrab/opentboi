//! Trinkets that make a room's reward, or a chest's, something else now and then (`trixies()`,
//! DoAction 29196–29228): the Ace of Spades turns it into a card, the Safety Cap a pill, the Match Stick
//! a bomb, the Rusted Key a key or a golden chest, and Child's Heart a heart.

use crate::game::odds::Odds;
use crate::game::pickups::kind::{
    BOMB_FRAME, CARD_MARKER_FRAME, GOLDEN_CHEST_FRAME, HEART_FRAME, KEY_FRAME, PILL_FRAME,
};
use crate::game::pickups::luck::PickupLuck;
use crate::game::pickups::trinket::Trinket;
use crate::game::random::Random;

/// `xox()` (DoAction 29192–29194): one time in seven, and in a chest one time in two besides.
const SWAP_ODDS: Odds = Odds::one_in(7);
/// The Rusted Key also swaps a room's reward one time in this many.
const RUSTED_KEY_ROOM_ODDS: Odds = Odds::one_in(17);

/// `xox(f3)` (DoAction 29194): `in_chest` is `f3`.
fn comes_up(in_chest: bool, random: &mut Random) -> bool {
    random.happens(SWAP_ODDS) || (in_chest && random.happens(Odds::EVEN))
}

/// The `b5` frame the reward becomes, if a trinket held makes it something else.
pub fn swapped_frame(luck: PickupLuck, in_chest: bool, random: &mut Random) -> Option<u8> {
    const SWAPPERS: [Trinket; 5] = [
        Trinket::ChildsHeart,
        Trinket::RustedKey,
        Trinket::MatchStick,
        Trinket::SafetyCap,
        Trinket::AceOfSpades,
    ];
    // Only with one of the five held (DoAction 29198).
    if !SWAPPERS.iter().any(|&trinket| luck.holds(trinket)) {
        return None;
    }
    if comes_up(in_chest, random) && luck.holds(Trinket::AceOfSpades) {
        // A card (DoAction 29202).
        Some(CARD_MARKER_FRAME)
    } else if comes_up(in_chest, random) && luck.holds(Trinket::SafetyCap) {
        // A pill (DoAction 29206).
        Some(PILL_FRAME)
    } else if comes_up(in_chest, random) && luck.holds(Trinket::MatchStick) {
        // A bomb (DoAction 29210).
        Some(BOMB_FRAME)
    } else if (comes_up(in_chest, random) || (random.happens(RUSTED_KEY_ROOM_ODDS) && !in_chest))
        && luck.holds(Trinket::RustedKey)
    {
        // A golden chest (DoAction 29216) or a key (DoAction 29220), one time in two (DoAction 29214).
        Some(if random.happens(Odds::EVEN) {
            GOLDEN_CHEST_FRAME
        } else {
            KEY_FRAME
        })
    } else if comes_up(in_chest, random) && luck.holds(Trinket::ChildsHeart) {
        // A heart (DoAction 29225).
        Some(HEART_FRAME)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn holding(trinket: Trinket) -> PickupLuck {
        PickupLuck {
            trinkets: [Some(trinket), None],
            ..PickupLuck::default()
        }
    }

    fn frames(trinket: Trinket, in_chest: bool) -> Vec<Option<u8>> {
        let mut random = Random::with_seed(6);
        (0..4000)
            .map(|_| swapped_frame(holding(trinket), in_chest, &mut random))
            .collect()
    }

    #[test]
    fn without_one_of_the_five_nothing_changes() {
        let mut random = Random::with_seed(1);
        assert!((0..500).all(|_| swapped_frame(holding(Trinket::Cancer), false, &mut random).is_none()));
        assert!((0..500).all(|_| swapped_frame(PickupLuck::default(), true, &mut random).is_none()));
    }

    #[test]
    fn each_trinket_makes_its_own_kind_of_reward_now_and_then() {
        let cases = [
            (Trinket::AceOfSpades, CARD_MARKER_FRAME),
            (Trinket::SafetyCap, PILL_FRAME),
            (Trinket::MatchStick, BOMB_FRAME),
            (Trinket::ChildsHeart, HEART_FRAME),
        ];
        for (trinket, expected) in cases {
            let seen = frames(trinket, false);
            let swapped = seen.iter().flatten().count();
            assert!(seen.iter().flatten().all(|&frame| frame == expected), "{trinket:?}");
            // One time in seven.
            assert!((450..700).contains(&swapped), "{trinket:?}: {swapped}");
        }
    }

    #[test]
    fn the_rusted_key_makes_keys_and_golden_chests() {
        let seen = frames(Trinket::RustedKey, false);
        assert!(seen.contains(&Some(KEY_FRAME)) && seen.contains(&Some(GOLDEN_CHEST_FRAME)));
        assert!(
            seen.iter()
                .flatten()
                .all(|&frame| frame == KEY_FRAME || frame == GOLDEN_CHEST_FRAME)
        );
    }

    #[test]
    fn in_a_chest_the_swap_comes_up_more_often() {
        let room = frames(Trinket::MatchStick, false).iter().flatten().count();
        let chest = frames(Trinket::MatchStick, true).iter().flatten().count();
        assert!(chest > room * 2, "{chest} against {room}");
    }
}

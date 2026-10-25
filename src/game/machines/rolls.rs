//! The machines' rolls: which slot result shows, and whether a beggar gives.

use crate::game::machines::MachineKind;
use crate::game::machines::clip;
use crate::game::machines::machine::Machine;
use crate::game::odds::Odds;
use crate::game::random::Random;

/// The devil beggar gives one time in four, one in three with the Lucky Foot (`beggar_gives`).
const DEVIL_GIFT_CHANCES: u32 = 4;
/// A beggar starts counting coins at 2 and gives when the count (before adding the coin) beats the
/// sum of three rolls below 2, 4 and 4; it then starts again at 2 or 3, counting at most to 9.
pub(super) const BEGGAR_FIRST_COUNT: u8 = 2;
const BEGGAR_COUNT_RANGES: [i32; 3] = [2, 4, 4];
const BEGGAR_RESTART_RANGE: u8 = 2;
const BEGGAR_MAXIMUM_COUNT: u8 = 9;

impl Machine {
    pub(super) fn beggar_gives(&mut self, random: &mut Random, lucky_foot: bool) -> bool {
        if self.kind == MachineKind::DevilBeggar {
            return random.happens(Odds::one_in(DEVIL_GIFT_CHANCES - u32::from(lucky_foot)));
        }
        let threshold: i32 = BEGGAR_COUNT_RANGES.iter().map(|&range| random.below(range)).sum();
        let gives = i32::from(self.beggar_count) > threshold;
        self.beggar_count = if gives {
            BEGGAR_FIRST_COUNT + random.below_u8(BEGGAR_RESTART_RANGE)
        } else {
            (self.beggar_count + 1).min(BEGGAR_MAXIMUM_COUNT)
        };
        gives
    }
}

/// `slotf()` (DoAction 12654, DoAction 12656): frames 3–23 (3–20 with the Lucky Foot, which leaves out the
/// last three; DoAction 12660), then 9 and up (DoAction 12662) move one on nine times in ten (always
/// for the Portable Slot, which rolls without a machine), and 7 (DoAction 12666) becomes 10
/// (DoAction 12668) one time in three.
pub fn slot_result(random: &mut Random, lucky_foot: bool, portable_slot: bool) -> u8 {
    const RESULTS: u8 = 21;
    const LUCKY_RESULTS: u8 = 18;
    const SHIFTED_FROM: u8 = 9;
    const UNSHIFTED_ODDS: Odds = Odds::one_in(10);
    const KEY_RESULT: u8 = 7;
    const KEY_TO_COINS_ODDS: Odds = Odds::one_in(3);
    const COINS_RESULT: u8 = 10;
    let mut result = if lucky_foot {
        clip::FIRST_RESULT + random.below_u8(LUCKY_RESULTS)
    } else {
        clip::FIRST_RESULT + random.below_u8(RESULTS)
    };
    // The original rolls even when the Portable Slot would shift anyway.
    if result >= SHIFTED_FROM && (!random.happens(UNSHIFTED_ODDS) || portable_slot) {
        result += 1;
    }
    if result == KEY_RESULT && random.happens(KEY_TO_COINS_ODDS) {
        result = COINS_RESULT;
    }
    result
}

//! The mark a room gets on the minimap for what is left in it (`levcol`, worked out every
//! frame in `aicol()` while the player is in the room, DoAction 14680–14708).

use crate::game::pickups::kind::PickupKind;
use crate::game::room::contents::RoomContents;

/// `itmc` frames, by the pickup clip's frame: cards 7, pills 8, machines 9.
const HEART_MARK: u8 = 1;
const FIRST_CHEST_MARK: u8 = 5;
const CARD_MARK: u8 = 7;
const PILL_MARK: u8 = 8;
const MACHINE_MARK: u8 = 9;

impl RoomContents {
    /// The `itmc` frame for the room, or `None` when nothing is left.
    pub fn map_mark(&self) -> Option<u8> {
        // An emptied pedestal is not marked.
        let lying = self
            .pickups
            .lying_untaken()
            .filter(|&kind| kind != PickupKind::ItemPedestal)
            .map(|kind| match kind {
                PickupKind::Card(_) => CARD_MARK,
                PickupKind::Pill(_) => PILL_MARK,
                other => other.clip_frame(),
            });
        let machines = self.machines.with_something_to_give().map(|_| MACHINE_MARK);
        let mark = lying.chain(machines).fold(0, combine);
        (mark > 0).then_some(mark)
    }
}

/// The higher mark wins, except that a heart outranks coins, keys and bombs; chests and the rest
/// outrank hearts.
fn combine(mark: u8, next: u8) -> u8 {
    let mark = if next == HEART_MARK && mark < FIRST_CHEST_MARK {
        0
    } else {
        mark
    };
    let next = if mark == HEART_MARK && next < FIRST_CHEST_MARK {
        0
    } else {
        next
    };
    mark.max(next)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_emptied_pedestal_leaves_no_mark() {
        let mut room = RoomContents::from_layout(
            &crate::game::room::layout::RoomLayout::default(),
            &crate::game::parameters::GameParameters::ORIGINAL,
            None,
            &mut crate::game::random::Random::default(),
        );
        room.pickups.place(PickupKind::ItemPedestal, glam::Vec2::ZERO);
        assert_eq!(room.map_mark(), None);
        room.pickups.place(
            PickupKind::Item(crate::game::pickups::item::Item::LuckyFoot),
            glam::Vec2::ZERO,
        );
        assert_eq!(room.map_mark(), Some(PickupKind::ItemPedestal.clip_frame()));
    }

    #[test]
    fn hearts_outrank_coins_and_chests_outrank_hearts() {
        assert_eq!([2, 1].into_iter().fold(0, combine), HEART_MARK);
        assert_eq!([1, 2].into_iter().fold(0, combine), HEART_MARK);
        assert_eq!([1, 5].into_iter().fold(0, combine), FIRST_CHEST_MARK);
    }
}

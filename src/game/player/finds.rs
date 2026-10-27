//! What Isaac's finds change beyond the moment: the save remembers an item taken (`colss`, DoAction
//! 6318–6320), taking a pill counts as finding "Pills here!" (item 43, DoAction 7110) and a card as
//! finding "Tarot Card" (item 61, DoAction 7115), and a passive item taken changes how strong the item
//! rolls think Isaac is (DoAction 6788–6798).

use crate::game::Game;
use crate::game::floor::plan::RoomRole;
use crate::game::pickups::Collection;
use crate::game::pickups::consumable::Consumable;
use crate::game::pickups::item::ItemNumber;

const PILLS_HERE: ItemNumber = ItemNumber(43);
const TAROT_CARD: ItemNumber = ItemNumber(61);

impl Game {
    /// A card or pill taken, by whatever means.
    pub(crate) fn remember_consumable_taken(&mut self, taken: Consumable) {
        self.progress.record_item(match taken {
            Consumable::Pill(_) => PILLS_HERE,
            Consumable::Card(_) => TAROT_CARD,
        });
    }

    pub(super) fn remember_finds(&mut self, collection: &Collection) {
        if let Some(taken) = collection.card_taken {
            self.remember_consumable_taken(taken);
        }
        if let Some(item) = collection.item_taken {
            self.progress.record_item(item.into());
            let from_devil = self.floor.current_role() == RoomRole::Devil;
            self.inventory.note_item_taken(item, from_devil);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::pickups::item::Item;
    use crate::game::pickups::pill::Pill;
    use crate::game::{GameParameters, RoomAppearance, RoomLayouts};

    fn game() -> Game {
        Game::new(
            RoomAppearance::Placeholder,
            &GameParameters::ORIGINAL,
            &RoomLayouts::default(),
            1,
        )
    }

    #[test]
    fn items_pills_and_cards_taken_are_remembered_for_later_runs() {
        let mut game = game();
        assert!(!game.progress.has_collected(Item::LuckyFoot.into()));
        game.remember_finds(&Collection {
            item_taken: Some(Item::LuckyFoot),
            card_taken: Some(Consumable::Pill(Pill::ALL[0])),
            ..Collection::default()
        });
        assert!(game.progress.has_collected(Item::LuckyFoot.into()));
        assert!(game.progress.has_collected(PILLS_HERE));
        assert!(!game.progress.has_collected(TAROT_CARD));
    }
}

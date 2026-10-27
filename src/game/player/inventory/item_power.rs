//! What the items taken say to the item rolls: how many of each Isaac holds (`ups`) and how strong
//! the major ones have made him (`wiptz`).

use crate::game::pickups::item::{Item, ItemNumber};
use crate::game::pickups::item_roll::{self, OfferFacts};
use crate::game::player::inventory::Inventory;

#[cfg_attr(
    not(test),
    allow(dead_code, reason = "what the item pools ask, which are not in the game yet")
)]
impl Inventory {
    /// `wiptz`.
    pub fn item_power(&self) -> f32 {
        self.item_power
    }

    /// How many of the item with this number Isaac has (`ups[number]`).
    pub fn count_of(&self, number: ItemNumber) -> u32 {
        let passive = self
            .items
            .iter()
            .filter(|&&item| ItemNumber::from(item) == number)
            .count();
        let active = self
            .active
            .held()
            .filter(|&item| ItemNumber::from(item) == number)
            .iter()
            .count();
        u32::try_from(passive + active).unwrap_or(u32::MAX)
    }

    /// How many familiars he has (DoAction 1478–1486).
    pub fn familiars_held(&self) -> u32 {
        item_roll::familiars().map(|number| self.count_of(number)).sum()
    }

    /// A passive item taken from a pedestal changes the power level (DoAction 6788–6798); the devil's
    /// items do not count towards it.
    pub fn note_item_taken(&mut self, item: Item, from_devil: bool) {
        if item.charges().is_none() {
            self.item_power += item_roll::power_change(item.into(), from_devil);
        }
    }

    /// What a pedestal's draw of `candidate` needs to know (DoAction 1488–1498): `found_before` from
    /// the save, `hard_mode` from the run.
    pub fn offer_facts(&self, candidate: ItemNumber, found_before: bool, hard_mode: bool) -> OfferFacts {
        OfferFacts {
            power: self.item_power,
            hard_mode,
            held: self.count_of(candidate) > 0,
            familiars: self.familiars_held(),
            found_before,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::parameters::GameParameters;

    fn inventory() -> Inventory {
        Inventory::new(GameParameters::ORIGINAL.inventory)
    }

    #[test]
    fn items_held_are_counted_by_number_and_only_major_passive_ones_raise_the_power() {
        let mut inventory = inventory();
        assert_eq!(inventory.count_of(Item::LuckyFoot.into()), 0);
        inventory.take_item(Item::LuckyFoot);
        inventory.note_item_taken(Item::LuckyFoot, false);
        assert_eq!(inventory.count_of(Item::LuckyFoot.into()), 1);
        assert!(inventory.item_power().abs() < f32::EPSILON, "the foot is a minor item");
        assert!(inventory.offer_facts(Item::LuckyFoot.into(), false, false).held);
    }

    #[test]
    fn an_active_item_held_counts_and_changes_no_power() {
        let mut inventory = inventory();
        inventory.take_item(Item::MyLittleUnicorn);
        inventory.note_item_taken(Item::MyLittleUnicorn, false);
        assert_eq!(inventory.count_of(Item::MyLittleUnicorn.into()), 1);
        assert!(inventory.item_power().abs() < f32::EPSILON);
    }
}

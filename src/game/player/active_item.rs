//! The active item: one held at a time, used with its own button once cleared rooms have charged
//! it (`spaceitem()`). Only active items have a charge; the original keeps each one's charge while
//! it lies elsewhere, so an item put down and taken back has what it had.

use std::collections::BTreeMap;

use crate::game::pickups::item::Item;
use crate::game::sound_event::SoundEvent;

#[derive(Clone, Debug, Default)]
pub struct ActiveSlot {
    held: Option<Item>,
    /// Rooms cleared towards each active item's next use; an item never held starts full.
    charges: BTreeMap<Item, u8>,
    /// The use button was held last tick.
    button_held: bool,
}

impl ActiveSlot {
    pub fn held(&self) -> Option<Item> {
        self.held
    }

    /// Rooms cleared towards the held item's use, and how many it needs.
    pub fn charge(&self) -> Option<(u8, u8)> {
        let item = self.held?;
        let needed = item.charges()?;
        Some((self.charges.get(&item).copied().unwrap_or(needed), needed))
    }

    pub fn is_charged(&self) -> bool {
        self.charge().is_some_and(|(charge, needed)| charge >= needed)
    }

    /// Whether the use button was pressed just now: holding it uses the item only once.
    pub fn press(&mut self, button_held: bool) -> bool {
        let pressed = button_held && !self.button_held;
        self.button_held = button_held;
        pressed
    }

    /// Takes an active item; returns the one it replaces, which goes onto the pedestal.
    pub fn take(&mut self, item: Item) -> Option<Item> {
        self.held.replace(item)
    }

    /// Uses the held item if it is charged: its charge starts again from nothing.
    pub fn use_held(&mut self) -> Option<Item> {
        if !self.is_charged() {
            return None;
        }
        let item = self.held?;
        self.charges.insert(item, 0);
        Some(item)
    }

    /// A room cleared: one more charge for the held item; the beep, or the recharge jingle once
    /// it is full. `None` when nothing needed charging.
    pub fn recharge(&mut self) -> Option<SoundEvent> {
        let (charge, needed) = self.charge()?;
        if charge >= needed {
            return None;
        }
        let item = self.held?;
        self.charges.insert(item, charge + 1);
        Some(if charge + 1 >= needed {
            SoundEvent::ItemRecharged
        } else {
            SoundEvent::ItemChargeGained
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_used_item_needs_its_rooms_again_and_keeps_its_charge_while_put_down() {
        let mut slot = ActiveSlot::default();
        assert_eq!(slot.take(Item::MyLittleUnicorn), None);
        assert!(slot.is_charged(), "a new item starts full");
        assert_eq!(slot.use_held(), Some(Item::MyLittleUnicorn));
        assert_eq!(slot.use_held(), None, "empty until recharged");
        for _ in 0..5 {
            assert_eq!(slot.recharge(), Some(SoundEvent::ItemChargeGained));
        }
        assert_eq!(slot.recharge(), Some(SoundEvent::ItemRecharged));
        assert_eq!(slot.recharge(), None);
        assert_eq!(slot.charge(), Some((6, 6)));
    }
}

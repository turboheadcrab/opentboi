//! What the shop and cards change in the inventory.

use crate::game::original_units::{Frames, Hearts};
use crate::game::pickups::item::Item;
use crate::game::player::inventory::Inventory;
use crate::game::shop::WareKind;
use crate::game::sound_event::SoundEvent;

/// The 2 of Hearts leaves at least this much, containers allowing.
const MINIMUM_AFTER_DOUBLING: Hearts = Hearts(2.0);

impl Inventory {
    /// Takes an item from its pedestal: a passive one is kept for the run, an active one goes in
    /// the slot. Returns the active item it replaces, which the pedestal then holds.
    pub fn take_item(&mut self, item: Item) -> Option<Item> {
        if item.charges().is_some() {
            self.active.take(item)
        } else {
            self.items.push(item);
            None
        }
    }

    /// A ware from the shop: a heart (up to the containers), a soul heart, a bomb or a key.
    /// `None` for what is not sold for coins (the devil's deals).
    pub fn buy(&mut self, ware: WareKind) -> Option<SoundEvent> {
        let sound = match ware {
            WareKind::Heart => {
                self.red_hearts = (self.red_hearts + Hearts::ONE).min(self.maximum_red_hearts);
                SoundEvent::HeartPickedUp
            }
            WareKind::SoulHeart => {
                self.soul_hearts += Hearts::ONE;
                SoundEvent::SoulHeartPickedUp
            }
            WareKind::Bomb => {
                self.bombs = self.bombs.saturating_add(1).min(self.parameters.maximum_count);
                SoundEvent::BombPickedUp
            }
            WareKind::Key => {
                self.keys = self.keys.saturating_add(1).min(self.parameters.maximum_count);
                SoundEvent::KeyPickedUp
            }
            WareKind::Consumable(consumable) => return self.take(consumable.pickup_kind()),
            WareKind::DevilDeal(_) => return None,
        };
        Some(sound)
    }

    /// Red hearts back, up to the heart containers.
    pub fn heal(&mut self, hearts: Hearts) {
        self.red_hearts = (self.red_hearts + hearts).min(self.maximum_red_hearts);
    }

    /// Red hearts back up to every container.
    pub fn heal_fully(&mut self) {
        self.red_hearts = self.maximum_red_hearts;
    }

    /// The 2 of Hearts (`toot()`): red hearts doubled, at least two, up to the containers.
    pub fn double_red_hearts(&mut self) {
        let doubled = (self.red_hearts * 2.0).max(MINIMUM_AFTER_DOUBLING);
        self.red_hearts = doubled.min(self.maximum_red_hearts);
    }

    /// The unicorn ride: nothing hurts for a while. Unlike the blink after a hit, the sacrifice
    /// room's spikes still pay out meanwhile.
    pub fn ride_unicorn(&mut self, frames: Frames) {
        self.ride.start(frames);
    }
}

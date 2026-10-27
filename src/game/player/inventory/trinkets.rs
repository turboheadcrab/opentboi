//! The trinkets held (`trix`, and `atrix` for a second slot): `pillc()` takes a trinket into the
//! slot, handing back the one it replaces.

use crate::game::pickups::trinket::Trinket;
use crate::game::player::inventory::Inventory;
use crate::game::sound_event::SoundEvent;

/// Trinkets carried at once; Mom's Purse (not in the game yet) would add a second slot.
const TRINKET_SLOTS: usize = 1;

impl Inventory {
    /// How many trinkets fit.
    #[allow(clippy::unused_self, reason = "Mom's Purse, not in the game yet, will add a slot")]
    pub fn trinket_slots(&self) -> usize {
        TRINKET_SLOTS
    }

    /// The trinkets held, the newest last.
    pub fn trinkets(&self) -> &[Trinket] {
        &self.trinkets
    }

    /// Whether a trinket is held (`trixx()`).
    pub fn has_trinket(&self, trinket: Trinket) -> bool {
        self.trinkets.contains(&trinket)
    }

    /// `pillc()`'s trinket branch (DoAction 7053–7170): the trinket goes into the slot and the
    /// sound of the shell game plays. One already held, or the Tick when there is no second slot (it
    /// cannot be swapped out), makes the pickup stay: `None`. A full slot gives up its oldest trinket
    /// to `dropped_trinket`.
    pub fn take_trinket(&mut self, trinket: Trinket) -> Option<SoundEvent> {
        if self.has_trinket(trinket) || (self.has_trinket(Trinket::Tick) && self.trinket_slots() < 2) {
            return None;
        }
        if self.trinkets.len() >= self.trinket_slots() {
            self.dropped_trinket = Some(self.trinkets.remove(0));
        }
        self.trinkets.push(trinket);
        self.taken_trinket = Some(trinket);
        // The Tick is kept in the first slot, where it can't be pushed out.
        if self.trinkets.len() > 1 && self.trinkets[1] == Trinket::Tick {
            self.trinkets.swap(0, 1);
        }
        Some(SoundEvent::ShellsShuffled)
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
    fn a_trinket_taken_goes_into_the_slot() {
        let mut inventory = inventory();
        assert!(inventory.take_trinket(Trinket::FishHead).is_some());
        assert_eq!(inventory.trinkets(), [Trinket::FishHead]);
        assert_eq!(inventory.taken_trinket, Some(Trinket::FishHead));
        assert_eq!(inventory.dropped_trinket, None);
    }

    #[test]
    fn a_full_slot_hands_back_its_trinket() {
        let mut inventory = inventory();
        inventory.take_trinket(Trinket::FishHead);
        assert!(inventory.take_trinket(Trinket::PushPin).is_some());
        assert_eq!(inventory.trinkets(), [Trinket::PushPin]);
        assert_eq!(inventory.dropped_trinket, Some(Trinket::FishHead));
    }

    #[test]
    fn the_tick_stays_and_one_held_is_not_taken_again() {
        let mut inventory = inventory();
        inventory.take_trinket(Trinket::Tick);
        assert!(inventory.take_trinket(Trinket::PushPin).is_none());
        assert_eq!(inventory.trinkets(), [Trinket::Tick]);
        let mut inventory = self::inventory();
        inventory.take_trinket(Trinket::PushPin);
        assert!(inventory.take_trinket(Trinket::PushPin).is_none());
    }
}

//! The trinkets held (`trix`, and `atrix` for a second slot): `pillc()` takes a trinket into the
//! slot, handing back the one it replaces. The bubble the Polaroid puts round Isaac (`playsave`)
//! lives here too.

use crate::game::original_units::{Frames, Hearts};
use crate::game::pickups::item::Item;
use crate::game::pickups::trinket::Trinket;
use crate::game::player::inventory::Inventory;
use crate::game::sound_event::SoundEvent;

/// Trinkets carried at once; Mom's Purse (item 139, `ups[139]`) adds a second slot.
const TRINKET_SLOTS: usize = 1;
const TRINKET_SLOTS_WITH_PURSE: usize = 2;

impl Inventory {
    /// How many trinkets fit (DoAction 7076, DoAction 7085–7092, DoAction 7128).
    pub fn trinket_slots(&self) -> usize {
        if self.has(Item::MomsPurse) {
            TRINKET_SLOTS_WITH_PURSE
        } else {
            TRINKET_SLOTS
        }
    }

    /// The trinkets held, the newest first: `trix`, then `atrix`.
    pub fn trinkets(&self) -> &[Trinket] {
        &self.trinkets
    }

    /// Whether a trinket is held, in either slot (`trixx()`, DoAction 4697).
    pub fn has_trinket(&self, trinket: Trinket) -> bool {
        self.trinkets.contains(&trinket)
    }

    /// `pillc()`'s trinket branch (DoAction 7053–7170): the trinket goes into the first slot, the one
    /// there moving to the second, and the sound of the shell game plays (DoAction 7084). One already
    /// held, or the Tick when there is no second slot (it cannot be swapped out), makes the pickup
    /// stay: `None`. A full set of slots gives up the oldest trinket to `dropped_trinket`. Taking the
    /// Tick puts its hat on Isaac (DoAction 7093, DoAction 7095, see `PlayerStats::tick_hat`).
    pub fn take_trinket(&mut self, trinket: Trinket) -> Option<SoundEvent> {
        if self.has_trinket(trinket) || (self.has_trinket(Trinket::Tick) && self.trinket_slots() < 2) {
            return None;
        }
        if self.trinkets.len() >= self.trinket_slots() {
            self.dropped_trinket = self.trinkets.pop();
        }
        self.trinkets.insert(0, trinket);
        self.taken_trinket = Some(trinket);
        // The Tick taken into the second slot (DoAction 7174) is put in the first (DoAction 7177), where
        // the next trinket taken can't push it out.
        if self.trinkets.get(1) == Some(&Trinket::Tick) {
            self.trinkets.swap(0, 1);
        }
        Some(SoundEvent::ShellsShuffled)
    }

    /// Whether Isaac is at half a heart of red or less, or has no health container and no soul heart
    /// (DoAction 4872).
    pub fn is_near_death(&self) -> bool {
        self.red_hearts <= Hearts::HALF || (self.maximum_red_hearts <= Hearts::ZERO && self.soul_hearts <= Hearts::ZERO)
    }

    /// Whether soul hearts offer less than half a heart, or, with no container, under a whole one
    /// (DoAction 4877).
    pub fn has_little_armor(&self) -> bool {
        self.soul_hearts < Hearts::HALF || (self.maximum_red_hearts <= Hearts::ZERO && self.soul_hearts < Hearts::ONE)
    }

    /// The bubble shields from every hit for this long (`playsave`).
    pub fn shield_for(&mut self, frames: Frames) {
        self.shield_frames = self.shield_frames.max(frames);
    }

    pub fn is_shielded(&self) -> bool {
        self.shield_frames > Frames::ZERO
    }

    /// The bubble is drawn until it is gone, flickering in its last 50 frames (DoAction 29487–29500).
    pub fn shield_frames(&self) -> Frames {
        self.shield_frames
    }

    pub(super) fn advance_shield(&mut self, elapsed: Frames) {
        self.shield_frames = (self.shield_frames - elapsed).max(Frames::ZERO);
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

    #[test]
    fn mom_s_purse_makes_room_for_a_second_trinket_and_the_oldest_goes_first() {
        let mut inventory = inventory();
        inventory.take_item(Item::MomsPurse);
        assert_eq!(inventory.trinket_slots(), 2);
        inventory.take_trinket(Trinket::FishHead);
        inventory.take_trinket(Trinket::PushPin);
        assert_eq!(inventory.trinkets(), [Trinket::PushPin, Trinket::FishHead]);
        assert_eq!(inventory.dropped_trinket, None);
        inventory.take_trinket(Trinket::Cancer);
        assert_eq!(inventory.trinkets(), [Trinket::Cancer, Trinket::PushPin]);
        assert_eq!(inventory.dropped_trinket, Some(Trinket::FishHead));
    }

    #[test]
    fn the_tick_taken_second_moves_to_the_first_slot_and_is_never_dropped() {
        let mut inventory = inventory();
        inventory.take_item(Item::MomsPurse);
        inventory.take_trinket(Trinket::Tick);
        inventory.take_trinket(Trinket::PushPin);
        assert_eq!(inventory.trinkets(), [Trinket::Tick, Trinket::PushPin]);
        // The Tick holds the slot with a purse, so a third trinket takes the other slot.
        inventory.take_trinket(Trinket::Cancer);
        assert_eq!(inventory.trinkets(), [Trinket::Tick, Trinket::Cancer]);
        assert_eq!(inventory.dropped_trinket, Some(Trinket::PushPin));
    }

    #[test]
    fn the_shield_wears_off() {
        let mut inventory = inventory();
        inventory.shield_for(Frames(10.0));
        assert!(inventory.is_shielded());
        inventory.advance(Frames(10.0));
        assert!(!inventory.is_shielded());
    }
}

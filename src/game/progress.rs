//! What carries over from run to run (the original's save data and its unlock flags). It is not
//! written to disk yet, so it lasts as long as the program runs.

use std::collections::BTreeSet;

use crate::game::pickups::item::ItemNumber;

/// Something earned that changes later runs.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Unlock {
    /// Paying blood often enough (`ader()`): an item joins the pool the rewards draw from.
    BloodDonorItem,
}

#[derive(Clone, Debug, Default)]
pub struct Progress {
    blood_donations: u32,
    unlocks: BTreeSet<Unlock>,
    /// The items found in earlier runs (`colss`, DoAction 6319): each is turned down now and then
    /// when a pedestal draws it.
    collected_items: BTreeSet<ItemNumber>,
}

impl Progress {
    /// An item is found (DoAction 6318–6320), or one of the two that pills and cards stand for is taken
    /// (DoAction 7110, DoAction 7115).
    pub fn record_item(&mut self, item: ItemNumber) {
        self.collected_items.insert(item);
    }

    #[cfg_attr(
        not(test),
        allow(dead_code, reason = "the item pools that ask are not in the game yet")
    )]
    pub fn has_collected(&self, item: ItemNumber) -> bool {
        self.collected_items.contains(&item)
    }

    /// `ader()` counts a donation (DoAction 7236) and, past the limit (DoAction 7237), unlocks the
    /// item (DoAction 7239).
    pub fn record_blood_donation(&mut self, donations_before_unlock: u32) {
        self.blood_donations += 1;
        if self.blood_donations > donations_before_unlock {
            self.unlocks.insert(Unlock::BloodDonorItem);
        }
    }

    /// Nothing reads it yet: the item it unlocks is not in the game.
    #[cfg(test)]
    pub fn has(&self, unlock: Unlock) -> bool {
        self.unlocks.contains(&unlock)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_twenty_first_blood_donation_unlocks_the_item() {
        let mut progress = Progress::default();
        for _ in 0..20 {
            progress.record_blood_donation(20);
        }
        assert!(!progress.has(Unlock::BloodDonorItem));
        progress.record_blood_donation(20);
        assert!(progress.has(Unlock::BloodDonorItem));
    }
}

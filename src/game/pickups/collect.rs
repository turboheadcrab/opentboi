//! The player touching pickups: `balljunk()` between the player and a pickup (DoAction 7618–8000).
//! A pickup is touched within the two sizes once it can be (see `PickupParameters`), and its clip
//! frame decides: taken (it plays its pickup animation), or left lying and bumped. The shop's wares
//! (case 15) and machines (case 8) are in `shop` and `machines`; hearts, coins, keys and bombs are
//! counted by `Inventory::take`.

use glam::Vec2;

use crate::game::hud::message::PickupMessage;
use crate::game::original_units::{Frames, distance};
use crate::game::pickups::item::Item;
use crate::game::pickups::kind::{ChestKind, PickupKind};
use crate::game::pickups::pickup::Pickup;
use crate::game::pickups::trinket::Trinket;
use crate::game::pickups::{Collection, Pickups};
use crate::game::player::inventory::Inventory;
use crate::game::sound_event::SoundEvent;

/// How long a handed-back card or item stays untouchable (DoAction 7102, DoAction 7147).
pub(super) const SWAP_BLOCK: Frames = Frames(10.0);

impl Pickups {
    /// Gives the player whatever they touch and can take, and opens chests (a golden one only
    /// `with_key`).
    pub fn collect(
        &mut self,
        player_position: Vec2,
        inventory: &mut Inventory,
        with_key: bool,
        sounds: &mut Vec<SoundEvent>,
    ) -> Collection {
        let parameters = self.parameters;
        let mut collection = Collection::default();
        for pickup in &mut self.lying {
            let touching = distance(pickup.center, player_position) < parameters.collect_reach;
            if !pickup.is_lying() || !touching || pickup.age_frames <= parameters.frames_until_collectable {
                continue;
            }
            let sound = match pickup.kind {
                // A golden chest needs a key (or the golden key), and one chest opens a frame.
                PickupKind::Chest(ChestKind::Golden) if !with_key || collection.used_key => None,
                PickupKind::Chest(_) if !collection.opened_chests.is_empty() => None,
                PickupKind::Chest(chest) => Some(open_chest(pickup.center, chest, &mut collection, sounds)),
                // Case 7: a card handed back is blocked for a moment, and touching it starts the
                // block again; else `pillc()` takes it.
                PickupKind::Card(_) | PickupKind::Pill(_) | PickupKind::Trinket(_) | PickupKind::Item(_)
                    if pickup.swap_blocked_frames > Frames::ZERO =>
                {
                    pickup.swap_blocked_frames = SWAP_BLOCK;
                    continue;
                }
                PickupKind::Card(_) | PickupKind::Pill(_) => {
                    collection.card_taken = pickup.kind.consumable();
                    let sound = inventory.take(pickup.kind);
                    if let Some(held) = inventory.dropped.take() {
                        pickup.hand_back(held.pickup_kind(), player_position, SWAP_BLOCK);
                        sounds.extend(sound);
                        continue;
                    }
                    sound
                }
                // A trinket already held (or the Tick in the only slot) is left lying.
                PickupKind::Trinket(trinket) => {
                    pickup.swap_for_trinket(trinket, player_position, inventory, &mut collection, sounds)
                }
                PickupKind::Item(item) => {
                    collection.item_taken = Some(item);
                    sounds.push(SoundEvent::ItemTaken);
                    take_from_pedestal(pickup, item, inventory);
                    continue;
                }
                kind => inventory.take(kind),
            };
            // Troll bombs say so the first time they are touched (`st22()`).
            if sound.is_some() || (pickup.kind.is_troll_bomb() && !std::mem::replace(&mut pickup.troll_announced, true))
            {
                collection.message = PickupMessage::for_pickup(pickup.kind).or(collection.message);
            }
            if let Some(sound) = sound {
                // The pickup plays its collected animation (DoAction 7172).
                pickup.collected_frames = Some(Frames::ZERO);
                sounds.push(sound);
            }
        }
        collection
    }
}

impl Pickups {
    /// `pillc()` called by an item rather than a pickup (DoAction 7141, DoAction 7143–7148): the card or
    /// pill that was held lies where Isaac stands, out of reach until he steps off it.
    pub fn drop_held(&mut self, held: PickupKind, center: Vec2) {
        let mut pickup = Pickup::new(held, center, Vec2::ZERO);
        pickup.swap_blocked_frames = SWAP_BLOCK;
        self.lying.push(pickup);
    }
}

impl Pickup {
    /// `pillc()`'s trinket branch for this pickup, which the player (standing at `player_position`)
    /// touches. Returns the sound for the pickup to be taken; `None` when it stays: the trinket is
    /// not taken (one already held, or the Tick in the only slot), or this pickup became the trinket
    /// it swapped for (its sound is made here, and it lies on).
    fn swap_for_trinket(
        &mut self,
        trinket: Trinket,
        player_position: Vec2,
        inventory: &mut Inventory,
        collection: &mut Collection,
        sounds: &mut Vec<SoundEvent>,
    ) -> Option<SoundEvent> {
        let sound = inventory.take_trinket(trinket)?;
        collection.trinket_taken = Some(trinket);
        let Some(held) = inventory.dropped_trinket.take() else {
            return Some(sound);
        };
        self.hand_back(PickupKind::Trinket(held), player_position, SWAP_BLOCK);
        sounds.push(sound);
        None
    }
}

/// A chest touched opens (`Chest_Open.mp`); a golden one turns its key (`Unlock00.wav`), which
/// is used up.
fn open_chest(center: Vec2, chest: ChestKind, collection: &mut Collection, sounds: &mut Vec<SoundEvent>) -> SoundEvent {
    collection.opened_chests.push((center, chest));
    if chest == ChestKind::Golden {
        collection.used_key = true;
        sounds.push(SoundEvent::KeyUsed);
    }
    SoundEvent::ChestOpened
}

/// Takes `item` off its pedestal, which is left empty, or holding the active item swapped out.
fn take_from_pedestal(pedestal: &mut Pickup, item: Item, inventory: &mut Inventory) {
    pedestal.kind = match inventory.take_item(item) {
        Some(held) => {
            pedestal.swap_blocked_frames = SWAP_BLOCK;
            PickupKind::Item(held)
        }
        None => PickupKind::ItemPedestal,
    };
}

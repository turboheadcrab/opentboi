//! What touching the pickups did, for the room to follow up.

use glam::Vec2;

use crate::game::hud::message::PickupMessage;
use crate::game::pickups::consumable::Consumable;
use crate::game::pickups::item::Item;
use crate::game::pickups::kind::ChestKind;
use crate::game::pickups::trinket::Trinket;

#[derive(Default)]
pub struct Collection {
    /// Chests opened: their contents spill out here.
    pub opened_chests: Vec<(Vec2, ChestKind)>,
    /// A golden chest took a key.
    pub used_key: bool,
    pub message: Option<PickupMessage>,
    /// A card or pill taken: Isaac holds it up (`pillc()`).
    pub card_taken: Option<Consumable>,
    /// An item taken: Isaac holds it up and its name shows; the pedestal stays, empty.
    pub item_taken: Option<Item>,
    /// A trinket taken: its name shows (`st11()`), and Isaac holds it up.
    pub trinket_taken: Option<Trinket>,
}

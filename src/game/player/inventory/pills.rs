//! The pills the inventory holds and has learned.

use crate::game::hud::pill_text::PillText;
use crate::game::pickups::consumable::Consumable;
use crate::game::pickups::item::Item;
use crate::game::pickups::pill::Pill;
use crate::game::player::inventory::Inventory;
use crate::game::sound_event::SoundEvent;
use crate::game::texture_id::TextureId;

impl Inventory {
    /// `pillc()`: the card or pill goes into the slot; one held already is dropped. A pill (DoAction 7105)
    /// makes the shell game's shuffle.
    pub(super) fn take_held(&mut self, held: Consumable) -> SoundEvent {
        self.dropped = self.held.replace(held);
        self.taken = Some(held);
        match held {
            Consumable::Card(_) => SoundEvent::CardPickedUp,
            // A pill taken shuffles the shell game's shells (DoAction 7109).
            Consumable::Pill(_) => SoundEvent::ShellsShuffled,
        }
    }

    /// Whether pills only help (the Lucky Foot, `ups[46]`).
    pub fn pills_only_help(&self) -> bool {
        self.has(Item::LuckyFoot)
    }

    /// What the HUD calls `pill`: its effect once it has one, else the unknown pill.
    pub fn pill_name(&self, pill: Pill) -> PillText {
        match self.pills.effect_of(pill) {
            Some(effect) => PillText::name_of(effect, effect.good || self.pills_only_help()),
            None => PillText::Unknown,
        }
    }

    /// The picture and name of what is held, for the HUD.
    pub fn held_textures(&self) -> Option<(TextureId, TextureId)> {
        self.held.map(|held| match held {
            Consumable::Card(card) => (TextureId::HeldCard(card), TextureId::HeldCardName(card)),
            Consumable::Pill(pill) => (TextureId::HeldPill(pill), TextureId::HeldPillName(self.pill_name(pill))),
        })
    }
}

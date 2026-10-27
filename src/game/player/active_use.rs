//! Using the active item: Space (or the left trigger) uses it once it is charged, from the room's
//! eleventh frame on (`spaceitem()`), and Isaac holds it up. Cleared rooms charge it.

use glam::Vec2;

use crate::game::Game;
use crate::game::InputState;
use crate::game::original_units::Frames;
use crate::game::pickups::card::Card;
use crate::game::pickups::consumable::Consumable;
use crate::game::pickups::item::Item;
use crate::game::pickups::kind::PickupKind;
use crate::game::player::animation::{Held, PlayerAnimation};
use crate::game::sound_event::SoundEvent;

/// No item is used in a room's first ten frames.
const FRAMES_BEFORE_USE: Frames = Frames(10.0);

impl Game {
    pub(crate) fn update_active_item(&mut self, input: &InputState) {
        let pressed = self.inventory.active.press(input.use_item);
        if !pressed || self.frames_in_room <= FRAMES_BEFORE_USE {
            return;
        }
        let Some(item) = self.inventory.active.use_held() else {
            return;
        };
        // The deck shows the card it gives instead (`showit = false`, DoAction 27214).
        if item != Item::DeckOfCards {
            self.player.play_animation(PlayerAnimation::HoldingUp(Held::Item(item)));
        }
        match item {
            Item::MyLittleUnicorn => self.ride_unicorn(),
            Item::DeckOfCards => self.use_deck_of_cards(),
            Item::LuckyFoot | Item::MomsPurse => {}
        }
    }

    /// Deck of Cards (DoAction 27213–27216, `pillc(3)`): takes a random card as if it were picked up.
    /// A card or pill held is put down where Isaac stands, out of reach until he steps off it.
    fn use_deck_of_cards(&mut self) {
        let card = Card::roll(&mut self.random);
        let taken = Consumable::Card(card);
        let sound = self.inventory.take(PickupKind::Card(card));
        self.pending_sound_events.extend(sound);
        if let Some(held) = self.inventory.dropped.take() {
            let center = self.player.position();
            self.floor
                .current_room_mut()
                .pickups
                .drop_held(held.pickup_kind(), center);
        }
        self.remember_consumable_taken(taken);
        self.player
            .play_animation(PlayerAnimation::PickingUp(Held::from(taken)));
    }

    /// The Chariot's and My Little Unicorn's ride, with its tune.
    pub(crate) fn ride_unicorn(&mut self) {
        self.inventory.ride_unicorn(self.parameters.cards.unicorn_ride_frames);
        self.pending_sound_events.push(SoundEvent::ChariotTune);
    }

    /// On the unicorn, every frame Isaac moves, the poop or fireplace one reach ahead of him
    /// takes a tear's weak hit (`killshit()`), as he tramples it.
    pub(crate) fn trample(&mut self, elapsed: Frames) {
        let new_frame = (self.frames_in_room - elapsed).whole() != self.frames_in_room.whole();
        let direction = self.player.velocity().normalize_or_zero();
        if !self.inventory.rides_unicorn() || !new_frame || direction == Vec2::ZERO {
            return;
        }
        let ahead = self.player.position() + direction * self.parameters.keys.reach;
        let room = self.floor.current_room_mut();
        room.hit_breakable_at(ahead, &mut self.random, &mut self.pending_sound_events);
    }

    /// A room cleared: the active item gains a charge.
    pub(crate) fn recharge_active_item(&mut self) {
        if let Some(sound) = self.inventory.active.recharge() {
            self.pending_sound_events.push(sound);
            self.overlay.item_box.charge_gained();
        }
    }
}

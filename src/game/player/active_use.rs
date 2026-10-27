//! Using the active item: Space (or the left trigger) uses it once it is charged, from the room's
//! eleventh frame on (`spaceitem()`), and Isaac holds it up. Cleared rooms charge it.

use glam::Vec2;

use crate::game::Game;
use crate::game::InputState;
use crate::game::original_units::{Frames, Pixels};
use crate::game::pickups::card::Card;
use crate::game::pickups::consumable::Consumable;
use crate::game::pickups::item::Item;
use crate::game::pickups::pill::Pill;
use crate::game::player::animation::{Held, PlayerAnimation};
use crate::game::room::contents::SpawOptions;
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
        // The deck and the bottle show what they give instead (`showit = false`, DoAction 27174,
        // DoAction 27216).
        if !matches!(item, Item::DeckOfCards | Item::MomsBottleOfPills) {
            self.player.play_animation(PlayerAnimation::HoldingUp(Held::Item(item)));
        }
        match item {
            Item::MyLittleUnicorn => self.ride_unicorn(),
            // `pillc(3)` (DoAction 7064, DoAction 7066): a random card, `tater()`.
            Item::DeckOfCards => {
                let card = Card::roll(&mut self.random);
                self.take_given(Consumable::Card(card));
            }
            // `pillc(4)` (DoAction 7060, DoAction 7062): a random pill.
            Item::MomsBottleOfPills => {
                let pill = Pill::roll(&mut self.random);
                self.take_given(Consumable::Pill(pill));
            }
            Item::LuckyFoot | Item::MomsPurse | Item::Phd => {}
        }
    }

    /// What Deck of Cards and Mom's Bottle of Pills give (DoAction 27173, DoAction 27215): `taken` as if
    /// it were picked up. A card or pill held is put down where Isaac stands (DoAction 7139), out of
    /// reach until he steps off it, and the slot shows the name (DoAction 7156).
    fn take_given(&mut self, taken: Consumable) {
        let sound = self.inventory.take(taken.pickup_kind());
        self.pending_sound_events.extend(sound);
        if let Some(held) = self.inventory.dropped.take() {
            // It lies on the nearest free tile (`spaw(player.xp, player.yp, 0, 5.3)`, DoAction 7141),
            // not under Isaac's feet.
            let near = self.player.position();
            let room = self.floor.current_room_mut();
            let spot = room.spaw(near, Pixels::ZERO, SpawOptions::default(), &mut self.random);
            room.pickups.drop_held(held.pickup_kind(), spot);
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

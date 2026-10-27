//! Taking and using cards: Q (or the right shoulder) uses the card held (`pilc`), whose name
//! shows on the banner (`tart()` → `st11()`); taking a card while holding one drops the old one.
//! Teleports wait for Isaac's teleport animation (`teller()`), The Tower's bombs fall over a second.

use glam::Vec2;

use crate::game::Game;
use crate::game::InputState;
use crate::game::floor::FloorCell;
use crate::game::hud::name_banner::BannerName;
use crate::game::hud::pill_text::PillText;
use crate::game::original_units::{Frames, Pixels};
use crate::game::pickups::consumable::Consumable;
use crate::game::pickups::kind::{BombKind, PickupKind};
use crate::game::player::animation::{Held, PlayerAnimation};
use crate::game::room::contents::SpawOptions;
use crate::game::sound_event::SoundEvent;

/// No card is used in a room's first second.
pub(crate) const FRAMES_BEFORE_USE: Frames = Frames(30.0);
/// The teleport ends three frames before its clip (5713, 19 frames) does.
const TELEPORT_FRAMES: Frames = Frames(16.0);
/// The Tower: for 30 frames, every fifth frame a troll bomb falls somewhere within 300 px of the
/// room's center.
const TOWER_FRAMES: Frames = Frames(30.0);
const TOWER_BOMB_EVERY: f32 = 5.0;
const TOWER_REACH: Pixels = Pixels(300.0);

/// What used cards still have going.
#[derive(Clone, Copy, Debug, Default)]
pub struct CardEffects {
    /// Where a teleport goes and how long until it does.
    pub teleport: Option<(FloorCell, Frames)>,
    /// The Tower's time left.
    pub tower_frames: Frames,
    pub boosts: super::boosts::RoomBoosts,
    /// The use button was held last tick.
    pub use_held: bool,
}

impl Game {
    /// Cards taken this tick, the card used, and what used cards still do.
    pub(crate) fn update_cards(&mut self, input: &InputState, elapsed: Frames) {
        if let Some(taken) = self.inventory.taken.take() {
            self.overlay.show_name(match taken {
                Consumable::Card(card) => BannerName::Card(card),
                // A pill taken names itself "Pills here!" (DoAction 7108), whatever its look.
                Consumable::Pill(_) => BannerName::Pill(PillText::PillsHere),
            });
            self.overlay.animate_card_slot(false);
        }
        // A press uses a card; holding the button on does not use the next one.
        let pressed = input.use_card && !self.cards.use_held;
        self.cards.use_held = input.use_card;
        if pressed
            && self.frames_in_room > FRAMES_BEFORE_USE
            && let Some(held) = self.inventory.held.take()
        {
            self.overlay.animate_card_slot(true);
            match held {
                Consumable::Card(card) => {
                    self.overlay.show_name(BannerName::Card(card));
                    self.player
                        .play_animation(PlayerAnimation::HoldingUp(card.held_picture()));
                    self.use_card(card);
                }
                Consumable::Pill(pill) => {
                    let look = self.inventory.pills.looks.look_of(pill);
                    self.player.play_animation(PlayerAnimation::HoldingUp(Held::Pill(look)));
                    self.use_pill(pill);
                }
            }
        }
        self.identify_held_pill(elapsed);
        self.advance_tower(elapsed);
        self.advance_teleport(elapsed);
    }

    /// `teller()`: Isaac's teleport animation, then `cell`.
    pub(crate) fn teleport(&mut self, cell: FloorCell) {
        self.pending_sound_events.push(SoundEvent::TeleportStarted);
        self.player.play_animation(PlayerAnimation::Teleport);
        self.cards.teleport = Some((cell, TELEPORT_FRAMES));
    }

    fn advance_teleport(&mut self, elapsed: Frames) {
        let Some((cell, frames_left)) = self.cards.teleport else {
            return;
        };
        let frames_left = frames_left - elapsed;
        if frames_left > Frames::ZERO {
            self.cards.teleport = Some((cell, frames_left));
            return;
        }
        self.cards.teleport = None;
        self.pending_sound_events.push(SoundEvent::TeleportArrived);
        self.room_transition = None;
        let arrival = self.floor.teleport_arrival();
        let entry = self.floor.teleport_to(cell, arrival);
        self.player.play_animation(PlayerAnimation::TeleportArrival);
        self.settle_into(&entry);
    }

    fn advance_tower(&mut self, elapsed: Frames) {
        if self.cards.tower_frames <= Frames::ZERO {
            return;
        }
        let before = self.cards.tower_frames;
        self.cards.tower_frames -= elapsed;
        let falls =
            (before.get() / TOWER_BOMB_EVERY).ceil() - (self.cards.tower_frames.get() / TOWER_BOMB_EVERY).ceil();
        for _ in 0..crate::whole_numbers::clamped_u32(falls) {
            // Around the room's center (`spaw()`).
            let radius = self.random.pixels_below(TOWER_REACH);
            let room = self.floor.current_room_mut();
            let spot = room.spaw(Vec2::ZERO, radius, SpawOptions::default(), &mut self.random);
            room.pickups.spawn(PickupKind::Bomb(BombKind::Troll), spot, Vec2::ZERO);
        }
    }

    pub(crate) fn start_tower(&mut self) {
        self.cards.tower_frames = TOWER_FRAMES;
    }
}

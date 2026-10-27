//! Playing the machines and beggars (paying with coins or blood, and handing out what they give),
//! and buying in the shop.

use crate::game::machines::clip;
use crate::game::pickups::item::Item;
use crate::game::player::death::Killer;
use glam::Vec2;

use crate::game::Game;
use crate::game::ball_contact::{Body, Contact};
use crate::game::floor::PLAYED_CHAPTER;
use crate::game::hud::message::PickupMessage;
use crate::game::hud::name_banner::BannerName;
use crate::game::hud::pill_text::PillText;
use crate::game::machines::{Cost, MachineEvent, Throw};
use crate::game::original_units::{Frames, PixelsPerFrame};
use crate::game::pickups::kind::{HeartKind, PickupKind};
use crate::game::pickups::rewards::Drop;
use crate::game::player::animation::{Held, PlayerAnimation};
use crate::game::random::Random;
use crate::game::shop::WareKind;
use crate::game::sound_event::SoundEvent;

/// A beggar's item is set down this far below him (`spaw()`).
const PEDESTAL_BELOW: Vec2 = Vec2::new(0.0, -80.0);
/// Sprayed prizes: 5 px/frame in a random direction, the vertical part a fifth of that plus 3 down.
const SPRAY_SPEED: PixelsPerFrame = PixelsPerFrame(5.0);
const SPRAY_VERTICAL_SCALE: f32 = 0.2;
const SPRAY_DOWNWARD: f32 = 3.0;
/// Out of a shell: 4 px/frame down and 4 sideways per shell from the middle one, plus 2 in a random
/// direction; created five frames' worth to the side.
const SHELL_SPEED: PixelsPerFrame = PixelsPerFrame(2.0);
const SHELL_DOWNWARD: f32 = 4.0;
const SHELL_SIDEWAYS_PER_SHELL: f32 = 4.0;
const SHELL_SIDE_OFFSET_PER_SPEED: f32 = 5.0;

impl Game {
    /// Bumping into the machines, and paying for the game a touch starts.
    pub(crate) fn play_machines(&mut self, player: Body, elapsed: Frames) -> Contact {
        let coins = self.inventory.coins;
        let room = self.floor.current_room_mut();
        let (contact, cost) =
            room.machines
                .bump(player, coins, elapsed, &mut self.random, &mut self.pending_sound_events);
        match cost {
            Some(Cost::Coin) => self.inventory.coins -= 1,
            Some(Cost::Blood) => {
                self.pending_sound_events.push(SoundEvent::BloodGiven);
                self.progress
                    .record_blood_donation(self.parameters.inventory.blood_donations_before_unlock);
                if self.inventory.give_blood(PLAYED_CHAPTER) {
                    self.react_to_hurt(Killer::BloodDonation);
                }
            }
            None => {}
        }
        contact
    }

    /// Buying a ware from the shop by touching it with enough coins.
    pub(crate) fn shop(&mut self) {
        let coins = self.inventory.coins;
        let position = self.player.position();
        if let Some((ware, center)) = self.floor.current_room_mut().wares.buy(position, coins) {
            self.inventory.coins -= ware.price();
            self.pending_sound_events.extend(self.inventory.buy(ware.kind));
            if let Some(pushed_out) = self.inventory.dropped.take() {
                self.floor
                    .current_room_mut()
                    .pickups
                    .drop_swapped(pushed_out, center, position);
            }
            // Isaac holds up what he bought.
            let held = match ware.kind {
                WareKind::Heart => Some(Held::Heart),
                WareKind::Bomb => Some(Held::Bomb),
                WareKind::Key => Some(Held::Key),
                WareKind::SoulHeart => Some(Held::SoulHeart),
                WareKind::Consumable(consumable) => Some(Held::holding(consumable, self.inventory.pills.looks)),
                WareKind::DevilDeal(_) => None,
            };
            if let Some(held) = held {
                self.player.play_animation(PlayerAnimation::PickingUp(held));
            }
        }
    }

    /// Textures the frontend should render ahead of time when it has a moment (see
    /// [`crate::frontend`]'s texture libraries), most urgent first.
    pub fn upcoming_textures(&self, textures: &mut Vec<crate::game::TextureId>) {
        self.floor.current_room().machines.upcoming_textures(textures);
    }

    pub(crate) fn update_machines(&mut self, elapsed: Frames) {
        let lucky_foot = self.inventory.has(Item::LuckyFoot);
        let events = self
            .floor
            .current_room_mut()
            .machines
            .advance(elapsed, &mut self.random, lucky_foot);
        self.carry_out(events);
    }

    /// What machines did: sounds, prizes thrown, items set down, contents spilled.
    pub(crate) fn carry_out(&mut self, events: Vec<(Vec2, MachineEvent)>) {
        let room = self.floor.current_room_mut();
        for (center, event) in events {
            match event {
                MachineEvent::Sound(sound) => self.pending_sound_events.push(sound),
                MachineEvent::Prize(drop, throw) => {
                    for (kind, position, velocity) in prizes(drop, throw, center, &mut self.random) {
                        room.pickups.spawn(kind, position, velocity);
                    }
                }
                MachineEvent::PedestalBelow => room.pickups.place(PickupKind::ItemPedestal, center + PEDESTAL_BELOW),
                MachineEvent::Fortune(index) => self.overlay.show_name(BannerName::Fortune(index)),
                // The same orbiting flies the pill gives (`ups[10]`), and `emo()`'s thumbs up.
                MachineEvent::PrettyFly => {
                    self.inventory.pills.flies += self.parameters.pills.fly_gain;
                    self.overlay.show_message(PickupMessage::Pill(PillText::PrettyFly));
                    self.player.play_animation(PlayerAnimation::ThumbsUp);
                }
                MachineEvent::Fly(kind) => {
                    room.enemies.add_fly(kind, center, &mut self.random);
                    self.player.play_animation(PlayerAnimation::ThumbsDown);
                }
                MachineEvent::Contents(source) => room.spill(source, center, &mut self.random),
            }
        }
    }
}

/// Each prize's kind, starting point and velocity (y up).
fn prizes(drop: Drop, throw: Throw, center: Vec2, random: &mut Random) -> Vec<(PickupKind, Vec2, Vec2)> {
    (0..drop.count)
        .filter_map(|_| {
            let mut kind = PickupKind::roll(drop.frame, random)?;
            if drop.soul_heart && matches!(kind, PickupKind::Heart(_)) {
                kind = PickupKind::Heart(HeartKind::Soul);
            }
            let Vec2 { x: cosine, y: sine } = random.unit_direction();
            let (position, velocity) = match throw {
                Throw::Spray => {
                    let down = sine * SPRAY_SPEED.get() * SPRAY_VERTICAL_SCALE + SPRAY_DOWNWARD;
                    (center, Vec2::new(cosine * SPRAY_SPEED.get(), -down))
                }
                Throw::Shell(lifted) => {
                    let down = SHELL_DOWNWARD + cosine * SHELL_SPEED.get();
                    let sideways = (f32::from(lifted) - f32::from(clip::MIDDLE_SHELL)) * SHELL_SIDEWAYS_PER_SHELL
                        + sine * SHELL_SPEED.get();
                    let start = center + Vec2::new(sideways * SHELL_SIDE_OFFSET_PER_SPEED, 0.0);
                    (start, Vec2::new(sideways, -down))
                }
            };
            Some((kind, position, velocity))
        })
        .collect()
}

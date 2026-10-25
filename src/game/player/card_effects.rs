//! What each card does when used (DoAction 27830–27990). Effects that need what does not exist
//! yet (enemies, items, flying, Mom's foot, the devil room) only show the card's name.

use crate::game::texture_id::EMPRESS_ANIMATION;
use glam::Vec2;

use crate::game::Game;
use crate::game::floor::FloorCell;
use crate::game::floor::plan::RoomRole;
use crate::game::machines::MachineKind;
use crate::game::original_units::Pixels;
use crate::game::pickups::card::Card;
use crate::game::pickups::kind::{HeartKind, PickupKind};
use crate::game::sound_event::SoundEvent;

/// The Lovers and The Hierophant: two hearts 40 to each side of Isaac (`spaw()`).
const HEART_PAIR_OFFSET: Vec2 = Vec2::new(40.0, 0.0);
/// Summons and Justice's gifts appear within this of Isaac (`spaw()`).
const SPAWN_RADIUS: Pixels = Pixels(45.0);
/// `over` frames of the cards' full-screen animations.
const DEVIL_ANIMATION: u8 = 2;
const DEATH_ANIMATION: u8 = 11;
const SUN_ANIMATION: u8 = 15;
/// `b5` frames of Justice's gifts: a heart, a coin, a key and a bomb.
const JUSTICE_GIFTS: [u8; 4] = [1, 2, 3, 4];

impl Game {
    pub(crate) fn use_card(&mut self, card: Card) {
        match card {
            Card::WheelOfFortune => self.summon(MachineKind::SlotMachine),
            Card::Judgement => self.summon(MachineKind::Beggar),
            Card::Temperance => self.summon(MachineKind::BloodDonation),
            Card::TheDevil => {
                self.cards.boosts.devil = true;
                self.play_card_animation(DEVIL_ANIMATION, SoundEvent::DevilCard);
            }
            Card::TheHangedMan => self.cards.boosts.flying = true,
            Card::Death => {
                self.play_card_animation(DEATH_ANIMATION, SoundEvent::DeathCard);
                let damage = self.parameters.cards.death_damage;
                self.floor.current_room_mut().enemies.hurt_all(
                    damage,
                    &mut self.random,
                    &mut self.pending_sound_events,
                );
            }
            Card::Strength => self.use_strength(),
            Card::TheSun => {
                self.play_card_animation(SUN_ANIMATION, SoundEvent::HeartPickedUp);
                self.inventory.heal_fully();
                self.floor.reveal_map();
            }
            Card::TheWorld => self.floor.reveal_map(),
            Card::TheLovers => self.place_heart_pair(HeartKind::Full),
            Card::TheHierophant => self.place_heart_pair(HeartKind::Soul),
            Card::Justice => self.give_justice(),
            Card::TheChariot => {
                self.ride_unicorn();
            }
            Card::TheTower => self.start_tower(),
            Card::TwoOfSpades => self.inventory.keys = doubled(self.inventory.keys),
            Card::TwoOfClubs => self.inventory.bombs = doubled(self.inventory.bombs),
            Card::TwoOfDiamonds => self.inventory.coins = doubled(self.inventory.coins),
            Card::TwoOfHearts => self.inventory.double_red_hearts(),
            Card::TheFool => self.teleport(FloorCell::START),
            Card::TheHermit => self.teleport_to_role(RoomRole::Shop),
            Card::TheMoon => self.teleport_to_role(RoomRole::Secret),
            Card::TheEmperor => self.teleport_to_role(RoomRole::Boss),
            Card::TheStars => self.teleport_to_role(RoomRole::Treasure),
            Card::TheJoker => self.teleport_to_role(RoomRole::Devil),
            // Homing tears and range, Mom's foot: not yet.
            Card::TheEmpress => {
                self.cards.boosts.empress = true;
                self.play_card_animation(EMPRESS_ANIMATION, SoundEvent::EmpressCard);
            }
            Card::TheMagician | Card::TheHighPriestess => {}
        }
    }

    /// Set down near Isaac (`spaw()`), with `summonsound.wav`.
    fn summon(&mut self, kind: MachineKind) {
        let position = self.player.position();
        let room = self.floor.current_room_mut();
        let spot = room.spawn_spot(position, SPAWN_RADIUS, &mut self.random);
        room.machines.place(kind, spot);
        self.pending_sound_events.push(SoundEvent::Summoned);
    }

    fn play_card_animation(&mut self, animation: u8, sound: SoundEvent) {
        self.overlay.play_card_animation(animation);
        self.pending_sound_events.push(sound);
    }

    /// One heart on each side of Isaac (`spaw()`).
    fn place_heart_pair(&mut self, heart: HeartKind) {
        let position = self.player.position();
        for side in [HEART_PAIR_OFFSET, -HEART_PAIR_OFFSET] {
            let room = self.floor.current_room_mut();
            let spot = room.spawn_spot(position + side, Pixels::ZERO, &mut self.random);
            room.pickups.spawn(PickupKind::Heart(heart), spot, Vec2::ZERO);
        }
    }

    /// A heart, a coin, a key and a bomb near Isaac (`spaw()`), each rolled as usual.
    fn give_justice(&mut self) {
        let position = self.player.position();
        for frame in JUSTICE_GIFTS {
            let Some(kind) = PickupKind::roll(frame, &mut self.random) else {
                continue;
            };
            let room = self.floor.current_room_mut();
            let spot = room.spawn_spot(position, SPAWN_RADIUS, &mut self.random);
            room.pickups.spawn(kind, spot, Vec2::ZERO);
        }
    }

    fn teleport_to_role(&mut self, role: RoomRole) {
        match self.floor.cell_with(role) {
            Some(cell) => self.teleport(cell),
            None => self.teleport_anywhere(),
        }
    }

    /// Like a teleport without a target: a random other room.
    pub(super) fn teleport_anywhere(&mut self) {
        let current = self.floor.current_cell();
        let cells: Vec<FloorCell> = self.floor.cells().into_iter().filter(|&cell| cell != current).collect();
        if !cells.is_empty() {
            let cell = cells[self.random.index(cells.len())];
            self.teleport(cell);
        }
    }
}

/// `toot()`: doubled, at least 2, up to the counter's 99.
fn doubled(count: u8) -> u8 {
    const MINIMUM: u8 = 2;
    const MAXIMUM: u8 = 99;
    count.saturating_mul(2).clamp(MINIMUM, MAXIMUM)
}

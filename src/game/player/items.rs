//! What the player carries, in play: picking things up, spending bombs and keys, getting hurt, and
//! starting a new run after dying.

use glam::Vec2;

use crate::game::ball_contact::Body;
use crate::game::hud::name_banner::BannerName;
use crate::game::original_units::{Frames, Hearts};
use crate::game::pickups::kind::ChestKind;
use crate::game::pickups::rewards::RewardSource;
use crate::game::pickups::{Collection, PickupSurroundings};
use crate::game::player::animation::{Held, PlayerAnimation};
use crate::game::player::death::Killer;
use crate::game::room;
use crate::game::room::grid_object::GridObject;
use crate::game::sound_event::SoundEvent;
use crate::game::{Game, GameSettings};

/// Random seeds for the next run come from the current run's generator.
const MAXIMUM_SEED: i32 = i32::MAX;

impl Game {
    /// Takes touched pickups, opens chests, bumps what stays, sets off troll bombs whose time has
    /// come, and hands out the room-clear reward.
    pub(crate) fn update_pickups(&mut self, elapsed: Frames) {
        self.inventory.advance(elapsed);
        let recolor = !self.player.is_animating();
        self.inventory.ride.advance(elapsed, recolor, &mut self.random);
        let with_key = self.has_key();
        let player_position = self.player.position();
        let luck = self.pickup_luck();
        let room = self.floor.current_room_mut();
        room.pickups.luck = luck;
        let collection = room.pickups.collect(
            player_position,
            &mut self.inventory,
            with_key,
            &mut self.pending_sound_events,
        );
        let mut red_chests = Vec::new();
        for &(center, chest) in &collection.opened_chests {
            let source = match chest {
                ChestKind::Brown => RewardSource::BrownChest,
                ChestKind::Golden => RewardSource::GoldenChest,
                ChestKind::Red => {
                    red_chests.push(center);
                    continue;
                }
            };
            room.spill(source, center, &mut self.random);
        }
        for center in red_chests {
            self.open_red_chest(center);
        }
        self.bump_into_things(elapsed);
        self.update_machines(elapsed);
        let room = self.floor.current_room_mut();
        let solid_tiles = room.bounds_where(GridObject::blocks_walking);
        let surroundings = PickupSurroundings {
            room_interior: room::interior_bounds(),
            solid_tiles: &solid_tiles,
            frames_in_room: self.frames_in_room,
            player_position: self.player.position(),
            walls: self.parameters.walls,
        };
        let troll_bombs = room
            .pickups
            .advance(elapsed, surroundings, &mut self.pending_sound_events);
        for center in troll_bombs {
            room.explode_at(center, &mut self.random, &mut self.pending_sound_events);
        }
        self.give_clear_reward();
        self.follow_up(&collection);
    }

    fn give_clear_reward(&mut self) {
        let missing_red_hearts = self.inventory.maximum_red_hearts - self.inventory.red_hearts;
        let cleared = self.floor.current_room_mut().give_clear_reward(
            self.frames_in_room,
            self.inventory.keys,
            missing_red_hearts,
            &mut self.random,
        );
        if cleared {
            self.recharge_active_item();
            self.trinkets_react_to_clearing();
        }
    }

    /// What taking pickups leads to: the eternal hearts' animation, the pickup's message, the
    /// golden chest's key.
    fn follow_up(&mut self, collection: &Collection) {
        if std::mem::take(&mut self.inventory.eternal_hearts_united) {
            self.overlay.play_eternal_heart_union();
        }
        if let Some(message) = collection.message {
            self.overlay.show_message(message);
        }
        if collection.used_key {
            self.spend_key();
        }
        if let Some(taken) = collection.card_taken {
            self.player
                .play_animation(PlayerAnimation::PickingUp(Held::from(taken)));
        }
        if let Some(trinket) = collection.trinket_taken {
            self.player
                .play_animation(PlayerAnimation::PickingUp(Held::Trinket(trinket)));
            self.overlay.show_name(BannerName::Trinket(trinket));
            self.overlay.animate_trinket_slot();
        }
        if let Some(item) = collection.item_taken {
            self.player.play_animation(PlayerAnimation::PickingUp(Held::Item(item)));
            self.overlay.show_name(BannerName::Item(item));
            if item.charges().is_some() {
                self.overlay.item_box.item_taken();
            }
        }
    }

    /// `ballhit()` between the player and what lies about: pickups that stay, and bombs.
    fn bump_into_things(&mut self, elapsed: Frames) {
        // A dying Isaac lies where he fell; shoves from walkers crowding him would make him jitter.
        if self.is_dying() {
            return;
        }
        let player_position = self.player.position();
        let player_body = Body {
            position: player_position,
            velocity: self.player.velocity(),
            mass: self.parameters.pickups.player_mass,
        };
        let room = self.floor.current_room_mut();
        let solid_tiles = room.bounds_where(GridObject::blocks_walking);
        let pickup_contact = room
            .pickups
            .bump(player_body, elapsed, room::interior_bounds(), &solid_tiles);
        let bomb_contact = self.bombs.bump(player_body, elapsed);
        let enemy_bump = self.floor.current_room_mut().enemies.bump(
            player_body,
            self.inventory.rides_unicorn(),
            elapsed,
            &mut self.random,
            &mut self.pending_sound_events,
        );
        if let Some((damage, killer)) = enemy_bump.hurt {
            self.hurt_player(damage, killer);
        }
        let enemy_contact = enemy_bump.contact;
        let machine_contact = self.play_machines(player_body, elapsed);
        self.shop();
        let contacts = [pickup_contact, bomb_contact, machine_contact, enemy_contact];
        let offset: Vec2 = contacts.iter().map(|contact| contact.first_offset).sum();
        let velocity_change: Vec2 = contacts.iter().map(|contact| contact.first_velocity_change).sum();
        let mut solid_tiles = self.floor.current_room().bounds_where(GridObject::blocks_walking);
        solid_tiles.extend(self.floor.wall_tiles());
        let room_interior = room::interior_bounds().shrunk_by(-room::TILE_SIZE);
        self.player.shove(offset, room_interior, &solid_tiles);
        self.player.accelerate(velocity_change);
    }

    /// Bombs placed and going off, and cards taken and used.
    pub(crate) fn update_items(&mut self, input: &crate::game::InputState, elapsed: Frames) {
        self.update_bombs(input.place_bomb, elapsed);
        self.update_cards(input, elapsed);
        self.update_active_item(input);
        self.trinkets_react_to_kills();
    }

    fn update_bombs(&mut self, place_requested: bool, elapsed: Frames) {
        if self
            .bombs
            .try_place(place_requested && self.has_bomb(), self.player.position())
        {
            self.spend_bomb();
            self.pending_sound_events.push(SoundEvent::BombPlaced);
        }
        for center in self.bombs.advance(elapsed, room::interior_bounds()) {
            self.floor
                .current_room_mut()
                .explode_at(center, &mut self.random, &mut self.pending_sound_events);
        }
    }

    pub(crate) fn has_key(&self) -> bool {
        self.parameters.keys.infinite_keys || self.inventory.has_key()
    }

    pub(crate) fn spend_key(&mut self) {
        if !self.parameters.keys.infinite_keys {
            self.inventory.use_key();
        }
    }

    pub(crate) fn has_bomb(&self) -> bool {
        self.parameters.bombs.infinite_bombs || self.inventory.bombs > 0
    }

    pub(crate) fn spend_bomb(&mut self) {
        if !self.parameters.bombs.infinite_bombs {
            self.inventory.use_bomb();
        }
    }

    /// `playerhurt()`; at no health left Isaac dies. A dying Isaac is not hurt again.
    pub(crate) fn hurt_player(&mut self, damage: Hearts, killer: Killer) {
        if !self.is_dying() && self.inventory.hurt(damage) {
            self.react_to_hurt(killer);
        }
    }

    /// The grunt, and dying at no health left.
    pub(crate) fn react_to_hurt(&mut self, killer: Killer) {
        self.pending_sound_events.push(SoundEvent::PlayerHurt);
        self.trinkets_react_to_hurt();
        if self.inventory.is_dead() {
            self.die(killer);
        } else {
            self.trinkets_react_to_near_death();
        }
    }

    /// A new run (after dying, or holding the restart key): a fresh floor and inventory with the
    /// same settings; sounds not yet played are kept.
    pub fn restart(&mut self) {
        let seed = self.random.below(MAXIMUM_SEED).unsigned_abs();
        let layouts = self.layouts.clone();
        let settings: GameSettings = self.settings;
        let sounds = std::mem::take(&mut self.pending_sound_events);
        let parameters = self.parameters;
        let progress = std::mem::take(&mut self.progress);
        *self = Self::new(self.room_appearance, &parameters, &layouts, seed);
        self.progress = progress;
        self.apply_settings(settings);
        self.pending_sound_events = sounds;
        // The new run comes up out of black, as every change of screen does (`goblack()`).
        self.overlay.flash_black(parameters.screen_fade);
    }
}

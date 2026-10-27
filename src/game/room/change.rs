//! Moving between rooms: the original's pause on entering, and the slide; what blasts and keys do
//! to the doors on the way.

use crate::game::attacks::explosion;
use crate::game::floor::RoomEntry;
use crate::game::floor::plan::RoomRole;
use crate::game::original_units::Frames;
use crate::game::player::death::Killer;
use crate::game::room::transition::RoomTransition;
use crate::game::sound_event::SoundEvent;
use crate::game::tick_clock::TickClock;
use glam::Vec2;

use crate::game::{Game, GameSettings, InputState};

impl Game {
    /// Applies the player's choices of original or alternative behavior.
    pub fn apply_settings(&mut self, settings: GameSettings) {
        self.settings = settings;
        self.last_input_device = settings.initial_input_device;
        self.clock = TickClock::new(settings.ticks_per_second, settings.input_sampling);
        self.slide_between_rooms = settings.slide_between_rooms;
        self.floor
            .set_door_rules(settings.secret_entrances, settings.curse_secret_door);
        if settings.showcase {
            self.floor.add_showcases();
        }
    }

    /// Blasts push the player and can open secret walls.
    pub(crate) fn apply_blasts(&mut self) {
        let (blasts, blast) = self.floor.current_room_mut().take_blasts();
        for center in blasts {
            // `gosplash()` breaks machines in reach whether it sees them or not; what it pushes and
            // hurts, it must see (`linecheckxx()`).
            let sight = self.floor.blast_sight();
            let sees = |target: Vec2| sight.blast_reaches(center, target);
            self.floor
                .current_room_mut()
                .pickups
                .push_from_blast(center, blast, sees);
            let room = self.floor.current_room_mut();
            room.enemies.blast(
                center,
                blast.radius,
                sees,
                &mut self.random,
                &mut self.pending_sound_events,
            );
            let broken = self.floor.current_room_mut().machines.blow_up_near(center);
            self.carry_out(broken);
            if sees(self.player.position()) && explosion::push_player(&mut self.player, center, blast) {
                self.hurt_player(self.parameters.inventory.blast_damage, Killer::Bomb);
            }
            self.floor.blow_doors(center, blast.radius);
            if let Some(jingle) = self.floor.blast_walls(center, blast.radius, &sight) {
                self.pending_sound_events.push(jingle);
            }
        }
    }

    /// Door animations, unlocking the shop with a key, and walking through doors.
    pub(crate) fn update_doors(&mut self, elapsed: Frames) {
        self.floor.advance_door_animations(elapsed);
        self.floor
            .update_doors(elapsed, self.frames_in_room, &mut self.pending_sound_events);
        if self.has_key() && self.floor.unlock_doors(self.player.position(), self.player.velocity()) {
            self.spend_key();
            self.pending_sound_events.push(SoundEvent::KeyUsed);
        }
        self.go_through_doors();
    }

    /// Walking into a door moves to the next room; whatever was flying stays behind.
    fn go_through_doors(&mut self) {
        self.floor.track_player(self.player.position());
        // Not while a machine keeps the doors shut, nor in the room's first second.
        let machines_busy = self.floor.current_room().machines.doors_locked();
        if self.frames_in_room <= self.room_entry.doors_closed_frames || machines_busy {
            return;
        }
        if let Some(side) = self.floor.door_reached(self.player.position()) {
            if self.slide_between_rooms {
                let mut old_room = Vec::new();
                self.fill_room_draw_items(&mut old_room);
                self.room_transition = Some(RoomTransition::start(side, old_room));
            }
            let entry = self.floor.go_through(side);
            self.settle_into(&entry);
        }
    }

    /// After arriving in a room (by door or teleport): the player stands at the entry, its
    /// jingle plays, and whatever was flying stays behind.
    pub(crate) fn settle_into(&mut self, entry: &RoomEntry) {
        self.frames_in_room = Frames::ZERO;
        self.blue_flies.leave_room(&self.parameters.blue_flies);
        self.end_room_boosts();
        self.trinkets_welcome(entry);
        self.pending_sound_events.extend(entry.jingle);
        self.player.place_at(entry.position);
        // The floor's splat bitmap is made anew for each room: old blood is gone.
        self.floor.current_room_mut().blood.clear();
        let pickups = &mut self.floor.current_room_mut().pickups;
        pickups.remember_positions();
        pickups.red_chests.first_visit = entry.first_visit;
        // A room's sound on the first visit (DoAction ~32430).
        let first_visit_sound = match entry.role {
            RoomRole::Treasure => Some(SoundEvent::TreasureRoomEntered),
            RoomRole::Boss => Some(SoundEvent::BossRoomEntered),
            RoomRole::MiniBoss => Some(SoundEvent::MiniBossRoomEntered),
            RoomRole::Devil => Some(SoundEvent::DevilRoomEntered),
            _ => None,
        };
        self.pending_sound_events
            .extend(first_visit_sound.filter(|_| entry.first_visit));
        self.tears.clear();
        self.splashes.clear();
        self.bombs.clear();
    }

    /// Right after entering a room, the movement and fire keys do nothing.
    pub(crate) fn input_after_entry_freeze(&self, input: &InputState) -> InputState {
        if self.frames_in_room >= self.room_entry.movement_frozen_frames {
            return *input;
        }
        InputState {
            movement: Vec2::ZERO,
            shoot_axes: Vec2::ZERO,
            aim_target: None,
            ..*input
        }
    }
}

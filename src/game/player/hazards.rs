//! What hurts the player to touch (`firecheck()`): burning fireplaces, spikes, the curse room's
//! spiked doorways, and the sacrifice room's spikes that may pay out a chest.

use glam::Vec2;

use crate::game::Game;
use crate::game::floor::plan::RoomRole;
use crate::game::original_units::{Frames, Hearts, distance};
use crate::game::pickups::kind::{ChestKind, PickupKind};
use crate::game::player::death::Killer;
use crate::game::room::stage_to_world;
use crate::game::sound_event::SoundEvent;

const CHEST_KINDS: [ChestKind; 2] = [ChestKind::Brown, ChestKind::Golden];

impl Game {
    /// Fire first, then spikes, as `firecheck()` checks them; blood creep under him hurts instead of
    /// either (`relf = 2`, DoAction 10008).
    pub(crate) fn touch_hazards(&mut self, elapsed: Frames) {
        let on_creep = self.touch_creep();
        if !self.touch_fire(elapsed, !on_creep) && !on_creep {
            self.touch_spikes();
        }
        if on_creep {
            self.hurt_player(self.parameters.creep.damage, Killer::Creep);
        }
    }

    /// Blood creep (DoAction 9992–10011) under a player who is not flying (or is in the sacrifice or
    /// curse room) hurts him, or, where the room is dark or a spider boss fights (DoAction 9998), slows
    /// him instead (DoAction 10002). Returns whether it hurts.
    fn touch_creep(&mut self) -> bool {
        let role = self.floor.current_role();
        let flying_safe = self.player_stats().flying && !matches!(role, RoomRole::Sacrifice | RoomRole::Curse);
        let creep = &self.floor.current_room().creep;
        if flying_safe || !creep.is_under(self.player.position(), &self.parameters.creep) {
            return false;
        }
        if creep.slows() {
            self.player.slow_down_for(self.parameters.creep.slow_frames);
            return false;
        }
        true
    }

    /// `firecheck()` (DoAction 9976): a burning fireplace within 38 pixels of one of the four corners
    /// 20 pixels from the player (DoAction 9977) pushes him away, 4 pixels a frame (DoAction 9987), and
    /// hurts (unless creep does instead).
    fn touch_fire(&mut self, elapsed: Frames, hurts: bool) -> bool {
        let fire = self.parameters.fire;
        self.frames_since_hiss += elapsed;
        let Some(away) = self.floor.current_room().fire_contact(self.player.position(), fire) else {
            return false;
        };
        let push = away.normalize_or_zero() * fire.push_per_frame;
        self.player.accelerate(push * elapsed);
        if self.frames_since_hiss >= fire.hiss_interval_frames {
            self.frames_since_hiss = Frames::ZERO;
            self.pending_sound_events.push(SoundEvent::FireHiss);
        }
        if hurts {
            self.hurt_player(fire.damage, Killer::Fire);
        }
        true
    }

    /// Spikes hurt by a heart, a curse doorway by half a heart; in the sacrifice room the hit may
    /// first give a chest.
    fn touch_spikes(&mut self) {
        let spikes = self.parameters.spikes;
        let position = self.player.position();
        let touches = |center: &Vec2| distance(*center, position) + spikes.reach_handicap < self.parameters.fire.reach;
        let on_doorway = self.floor.curse_doorways().iter().any(touches);
        let on_spikes = self.floor.current_room().spike_centers().iter().any(touches);
        // Flying over spikes only hurts in the sacrifice and curse rooms.
        let role = self.floor.current_role();
        let flying_safe = self.player_stats().flying && !matches!(role, RoomRole::Sacrifice | RoomRole::Curse);
        if (!on_doorway && !on_spikes) || flying_safe {
            return;
        }
        if on_spikes && self.floor.current_role() == RoomRole::Sacrifice && !self.inventory.is_invulnerable() {
            self.sacrifice();
        }
        let damage = if on_doorway {
            spikes.curse_door_damage
        } else {
            spikes.damage
        };
        self.hurt_player(damage, Killer::Spikes);
    }

    /// The sacrifice room: one chest per floor, one time in five or when nearly dead.
    fn sacrifice(&mut self) {
        let spikes = self.parameters.spikes;
        if self.floor.sacrifice_paid {
            return;
        }
        let nearly_dead =
            self.inventory.red_hearts < spikes.sacrifice_low_health && self.inventory.soul_hearts <= Hearts::ZERO;
        if !self.random.happens(spikes.sacrifice_chest_odds) && !nearly_dead {
            return;
        }
        self.floor.sacrifice_paid = true;
        let chest = CHEST_KINDS[self.random.index(CHEST_KINDS.len())];
        let (x, y) = spikes.sacrifice_chest_stage_point;
        let spot = stage_to_world(Vec2::new(x, y));
        let pickups = &mut self.floor.current_room_mut().pickups;
        let kind = pickups.red_chests.apply(PickupKind::Chest(chest), &mut self.random);
        pickups.spawn(kind, spot, Vec2::ZERO);
    }
}

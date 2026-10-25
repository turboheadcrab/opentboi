//! What a room hands out: drops from broken poop and fireplaces, chest and tinted rock contents,
//! the reward for clearing it, and the burning fireplaces that hurt.

use glam::Vec2;

use crate::game::original_units::{Frames, Hearts, length};
use crate::game::parameters::FireParameters;
use crate::game::pickups::kind::{HeartKind, PickupKind};
use crate::game::pickups::rewards::{self, Drop, RewardSource};
use crate::game::random::Random;
use crate::game::room::contents::RoomContents;
use crate::game::room::grid_kind::BreakableKind;
use crate::game::room::grid_object::HitStrength;
use crate::game::room::{COLUMN_COUNT, ROW_COUNT, tile_center};

/// `cspawn()` sets a chest from a chest down 10 px lower each time, and does not throw it.
const NESTED_CHEST_DROP: f32 = 10.0;

impl RoomContents {
    /// `killshit()`: broken by tears, a poop or fireplace sometimes drops a heart or coin; golden
    /// poop always spills coins.
    pub(super) fn drop_from_breakable(
        &mut self,
        kind: BreakableKind,
        strength: HitStrength,
        center: Vec2,
        random: &mut Random,
    ) {
        match kind {
            BreakableKind::GoldenPoop => {
                let coins = rewards::golden_poop_coins(random);
                self.spawn_drop(coins, center, true, random);
            }
            BreakableKind::Poop | BreakableKind::CornyPoop | BreakableKind::Fireplace | BreakableKind::RedFireplace
                if strength == HitStrength::Tear =>
            {
                if let Some(frame) = rewards::breakable_drop(random) {
                    let drop = Drop {
                        frame,
                        count: 1,
                        soul_heart: false,
                    };
                    self.spawn_drop(drop, center, false, random);
                }
            }
            _ => {}
        }
    }

    /// A chest's or the tinted rock's contents fly out from `center`.
    pub fn spill(&mut self, source: RewardSource, center: Vec2, random: &mut Random) {
        for drop in rewards::contents(source, random) {
            let nested_chest = drop.is_chest();
            if nested_chest && source != RewardSource::TintedRock {
                self.spawn_drop(drop, center - Vec2::new(0.0, NESTED_CHEST_DROP), false, random);
            } else {
                self.spawn_drop(drop, center, !nested_chest, random);
            }
        }
    }

    /// The reward for clearing a room with enemies, once: when the doors open after the last one
    /// dies. Rooms whose enemies the game does not have yet count as cleared a moment after the
    /// first entry (a stand-in). Returns whether the room was cleared just now.
    pub fn give_clear_reward(
        &mut self,
        frames_in_room: Frames,
        keys: u8,
        missing_red_hearts: Hearts,
        lucky_foot: bool,
        random: &mut Random,
    ) -> bool {
        const CLEARED_AFTER: Frames = Frames(7.0);
        let due = if self.enemies.spawned_any {
            !self.enemies.any_alive() && frames_in_room > CLEARED_AFTER
        } else {
            frames_in_room > self.parameters.pickups.room_clear_reward_frames
        };
        if !self.clear_reward_pending || !due {
            return false;
        }
        self.clear_reward_pending = false;
        if let Some(frame) = rewards::room_clear_reward(random, keys, missing_red_hearts > Hearts::ONE, lucky_foot) {
            let drop = Drop {
                frame,
                count: 1,
                soul_heart: false,
            };
            let spot = self.free_spot_near(Vec2::ZERO);
            self.spawn_drop(drop, spot, false, random);
        }
        true
    }

    /// `firecheck()`: the direction away from a burning fireplace the player touches.
    pub fn fire_contact(&self, player_position: Vec2, fire: FireParameters) -> Option<Vec2> {
        let corners = [Vec2::ONE, Vec2::new(-1.0, 1.0), Vec2::new(1.0, -1.0), Vec2::NEG_ONE]
            .map(|corner| player_position + corner * fire.probe_offset);
        self.grid_objects
            .iter()
            .filter(|object| object.is_burning_fireplace())
            .filter(|object| corners.iter().any(|&corner| object.tile_bounds().contains(corner)))
            .map(|object| player_position - object.center())
            .find(|&away| length(away) < fire.reach)
    }

    fn spawn_drop(&mut self, drop: Drop, center: Vec2, thrown: bool, random: &mut Random) {
        for _ in 0..drop.count {
            let Some(mut kind) = PickupKind::roll(drop.frame, random) else {
                continue;
            };
            if drop.soul_heart && matches!(kind, PickupKind::Heart(_)) {
                kind = PickupKind::Heart(HeartKind::Soul);
            }
            let kind = self.pickups.red_chests.apply(kind, random);
            self.spawn_pickup(kind, center, thrown, random);
        }
    }

    /// `kind` set down at `center`, or thrown out from it in a random direction at 5 px per frame
    /// (`cspawn()`: DoAction 29253, DoAction 29254).
    pub(crate) fn spawn_pickup(&mut self, kind: PickupKind, center: Vec2, thrown: bool, random: &mut Random) {
        let velocity = if thrown {
            random.direction(self.parameters.pickups.throw_speed_per_frame.get())
        } else {
            Vec2::ZERO
        };
        self.pickups.spawn(kind, center, velocity);
    }

    /// `posw()`: the nearest tile center no grid object blocks.
    pub(crate) fn free_spot_near(&self, target: Vec2) -> Vec2 {
        let blocked = |spot: Vec2| {
            self.grid_objects
                .iter()
                .filter(|object| object.blocks_walking())
                .any(|object| object.tile_bounds().contains(spot))
        };
        (0..ROW_COUNT)
            .flat_map(|row| (0..COLUMN_COUNT).map(move |column| tile_center(column, row)))
            .filter(|&spot| !blocked(spot))
            .min_by(|first, second| first.distance(target).total_cmp(&second.distance(target)))
            .unwrap_or(target)
    }
}

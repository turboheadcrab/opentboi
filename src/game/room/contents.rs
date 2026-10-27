//! Everything in the room besides the player and tears: grid objects, the flies circling them,
//! explosions with their scorch marks, and opening key blocks.

use glam::Vec2;

use crate::game::attacks::explosion::Explosion;
use crate::game::bounds::Bounds;
use crate::game::machines::Machines;
use crate::game::original_units::Frames;
use crate::game::parameters::GameParameters;
use crate::game::pickups::Pickups;
use crate::game::pickups::rewards::RewardSource;
use crate::game::random::Random;
use crate::game::room::PlayerSighting;
use crate::game::room::blood::BloodSplats;
use crate::game::room::creep::Creep;
use crate::game::room::enemies::{Enemies, EnemyParameters};
use crate::game::room::grid_kind::{BreakableKind, GridObjectKind, TileKind};
use crate::game::room::grid_object::{GridObject, HitOutcome, HitStrength};
use crate::game::room::layout::{Decoration, RoomLayout};
use crate::game::room::orbiting_fly::OrbitingFly;
use crate::game::room::stage_to_world;
use crate::game::shop::Wares;
use crate::game::sound_event::SoundEvent;

/// An object to hit, how hard, and the blast center when an explosion deals it.
pub(super) type PendingHit = (usize, HitStrength, Option<Vec2>);

pub struct RoomContents {
    parameters: GameParameters,
    grid_objects: Vec<GridObject>,
    flies: Vec<OrbitingFly>,
    explosions: Vec<Explosion>,
    scorch_marks: Vec<Vec2>,
    /// Tiles broken between pits by a blast, where `breakall()` draws the floor's break.
    broken_floors: Vec<Vec2>,
    /// Blasts since the last `take_blasts` (center), for effects outside the room's objects.
    blasts: Vec<Vec2>,
    /// `keypow`/`keypoww`: which key block the player is pushing into, and for how long.
    pushed_key_block: Option<usize>,
    key_push_frames: Frames,
    pub pickups: Pickups,
    pub machines: Machines,
    pub wares: Wares,
    pub enemies: Enemies,
    pub creep: Creep,
    pub blood: BloodSplats,
    /// Figures standing in the room, each with the look it picked, at a world point.
    pub(super) decorations: Vec<(Decoration, u8, Vec2)>,
    /// Time spent in the room, for what loops on its figures.
    age_frames: Frames,
    /// Rooms with enemies give a reward once cleared (see [`RoomContents::give_clear_reward`]).
    clear_reward_pending: bool,
}

impl RoomContents {
    pub fn from_layout(
        layout: &RoomLayout,
        parameters: &GameParameters,
        tinted_rock: Option<(u32, u32)>,
        random: &mut Random,
    ) -> Self {
        let grid_objects = layout.grid_objects(parameters.breakables, tinted_rock);
        // `turd()` gives every intact corny poop a fly (`efly`), numbered by the fly counter.
        let flies = grid_objects
            .iter()
            .enumerate()
            .filter(|(_, object)| object.kind() == GridObjectKind::Breakable(BreakableKind::CornyPoop))
            .zip(0..)
            .map(|((object_index, _), fly_number)| OrbitingFly::new(object_index, fly_number))
            .collect();
        Self {
            parameters: *parameters,
            grid_objects,
            flies,
            explosions: Vec::new(),
            scorch_marks: Vec::new(),
            broken_floors: Vec::new(),
            blasts: Vec::new(),
            pushed_key_block: None,
            key_push_frames: Frames::ZERO,
            pickups: Pickups::from_layout(layout, parameters.pickups, random),
            machines: Machines::from_layout(layout, parameters.pickups, random),
            wares: Wares::from_layout(layout, parameters.pickups, random),
            enemies: Enemies::from_layout(layout, &EnemyParameters::from_game(parameters), random),
            creep: Creep::default(),
            blood: BloodSplats::default(),
            decorations: layout
                .decorations
                .iter()
                .map(|&(decoration, point)| {
                    let look = random.below_u8(decoration.look_count());
                    (decoration, look, stage_to_world(point))
                })
                .collect(),
            age_frames: Frames::ZERO,
            clear_reward_pending: layout.has_enemies(),
        }
    }

    #[cfg(test)]
    pub fn add_grid_object(&mut self, object: GridObject) {
        self.grid_objects.push(object);
    }

    /// Centers of the spike tiles (`levz` 0.99, not webs).
    pub fn spike_centers(&self) -> Vec<Vec2> {
        self.grid_objects
            .iter()
            .filter(|object| object.kind() == GridObjectKind::Tile(TileKind::Spikes))
            .map(GridObject::center)
            .collect()
    }

    pub fn bounds_where(&self, include: impl Fn(&GridObject) -> bool) -> Vec<Bounds> {
        self.grid_objects
            .iter()
            .filter(|object| include(object))
            .map(GridObject::tile_bounds)
            .collect()
    }

    /// Objects tears collide with, as (object index, tile bounds).
    pub fn tear_blockers(&self) -> Vec<(usize, Bounds)> {
        (0..self.grid_objects.len())
            .filter(|&object_index| self.grid_objects[object_index].blocks_tears())
            .map(|object_index| (object_index, self.grid_objects[object_index].tile_bounds()))
            .collect()
    }

    /// Damages one object; a destroyed TNT barrel explodes and damages its neighbors in turn.
    /// Returns what the hit did to that object.
    pub fn hit(
        &mut self,
        object_index: usize,
        strength: HitStrength,
        random: &mut Random,
        sounds: &mut Vec<SoundEvent>,
    ) -> Option<HitOutcome> {
        self.resolve_hits(vec![(object_index, strength, None)], random, sounds)
    }

    /// A bomb going off at `center`: the same blast a TNT barrel makes (`bombfail`).
    pub fn explode_at(&mut self, center: Vec2, random: &mut Random, sounds: &mut Vec<SoundEvent>) {
        let pending_hits = self.start_explosion(center, sounds);
        self.resolve_hits(pending_hits, random, sounds);
    }

    /// Returns the outcome of the last of `pending_hits` (the only one, for a single hit).
    fn resolve_hits(
        &mut self,
        mut pending_hits: Vec<PendingHit>,
        random: &mut Random,
        sounds: &mut Vec<SoundEvent>,
    ) -> Option<HitOutcome> {
        let mut first_outcome = None;
        let mut is_first = true;
        while let Some((index, hit_strength, blast_center)) = pending_hits.pop() {
            let center = self.grid_objects[index].center();
            let outcome = self.grid_objects[index].take_hit(hit_strength, random);
            if std::mem::take(&mut is_first) {
                first_outcome = outcome;
            }
            match outcome {
                Some(HitOutcome::Destroyed(BreakableKind::TntBarrel)) => {
                    pending_hits.extend(self.start_explosion(center, sounds));
                }
                Some(HitOutcome::Destroyed(kind)) => {
                    sounds.push(if kind.is_fireplace() {
                        SoundEvent::FireExtinguished
                    } else {
                        SoundEvent::PoopDestroyed
                    });
                    self.drop_from_breakable(kind, hit_strength, center, random);
                }
                Some(HitOutcome::Cleared(tile_kind)) => {
                    if tile_kind.is_rock() {
                        sounds.push(SoundEvent::RockCrumbled);
                        if tile_kind == TileKind::TintedRock {
                            self.spill(RewardSource::TintedRock, center, random);
                        }
                        if let Some(blast) = blast_center {
                            self.bridge_pit_behind(index, blast);
                        }
                    }
                    if let Some(blast) = blast_center {
                        self.break_floor_between_pits(center, blast);
                    }
                }
                Some(HitOutcome::Damaged | HitOutcome::Resisted) | None => {}
            }
        }
        first_outcome
    }

    /// `player` is where the player is and how fast he moves (world coordinates), which the enemies
    /// that go for him aim at; a bare position is a player standing still.
    pub fn advance(
        &mut self,
        elapsed: Frames,
        player: impl Into<PlayerSighting>,
        random: &mut Random,
        sounds: &mut Vec<SoundEvent>,
    ) {
        self.pass_floor_frames(elapsed, random);
        self.advance_enemies(elapsed, player.into(), random, sounds);
        self.creep.advance(elapsed, &self.parameters.creep);
        for object in &mut self.grid_objects {
            object.advance(elapsed);
        }
        // Without enemies yet, a fly whose poop breaks simply disappears.
        let grid_objects = &self.grid_objects;
        self.flies
            .retain(|fly| grid_objects[fly.anchor_index].is_intact_breakable());
        for fly in &mut self.flies {
            fly.advance(elapsed);
        }
        for explosion in &mut self.explosions {
            explosion.advance(elapsed);
        }
        self.explosions.retain(|explosion| !explosion.is_finished());
    }
}

mod blasts;
mod broken_floor;
mod drawing;
mod drops;
mod enemy_turns;
mod floor_frames;
mod key_blocks;
mod map_mark;
mod spawn_spot;
pub use spawn_spot::SpawOptions;

#[cfg(test)]
mod tests;

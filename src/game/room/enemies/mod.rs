//! The enemies of a room: for now the flies and Pooters, and the shots Pooters fire. They are
//! created from the layout's enemy markers on the room's first visit (see `spawn.rs`), think once
//! per original frame, take hits from tears and blasts, bump the player, and keep the doors shut
//! while any lives (`shutdoor++` in `alive()`).

mod births;
mod blob_drawing;
mod contact;
mod crowding;
mod deaths;
mod drawing;
mod fly;
mod friends;
mod ground_drawing;
mod hazards;
mod marks;
mod pooter;
mod shooter;
mod shooting;
mod shot;
mod spawn;
mod surroundings;
mod walk_map;
mod walker;

use glam::Vec2;

use crate::game::attacks::splash::Splash;
use crate::game::ball_contact::Contact;
use crate::game::bounds::Bounds;
use crate::game::original_units::{Frames, Hearts, HitPoints, Mass, Pixels};
use crate::game::parameters::{
    ClottyParameters, EnemyShotParameters, FireParameters, FleshMoundParameters, FlyParameters, GameParameters,
    GaperParameters, HeadlessParameters, HopperParameters, HorfParameters, MulliganParameters, PooterParameters,
    SpikeParameters, SplatParameters,
};
use crate::game::player::death::Killer;
use crate::game::random::Random;
use crate::game::room::blood::Splatter;
use crate::game::room::grid_object::GridObject;
use crate::game::room::layout::RoomLayout;
use crate::game::room::{sight::TileMarks, stage_to_world, world_to_stage};
use crate::game::sound_event::SoundEvent;
pub use fly::{Fly, FlyState};
pub use friends::FriendTarget;
use shot::EnemyShot;
use spawn::Spawned;
use surroundings::Grid;
pub use walker::{Hurt, Walker, WalkerClip, WalkerKind, WalkerShot, WalkerState};

/// Sizes: the player 10, a tear 8 (a tear's mass is 8 too).
pub(super) const PLAYER_SIZE: Pixels = Pixels(10.0);
pub(super) const TEAR_SIZE: Pixels = Pixels(8.0);
pub(super) const TEAR_MASS: Mass = Mass(8.0);

/// The values the enemies and their shots follow.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EnemyParameters {
    pub flies: FlyParameters,
    pub gapers: GaperParameters,
    pub headless: HeadlessParameters,
    pub horfs: HorfParameters,
    pub clottys: ClottyParameters,
    pub hoppers: HopperParameters,
    pub flesh_mounds: FleshMoundParameters,
    pub mulligans: MulliganParameters,
    pub pooters: PooterParameters,
    pub shots: EnemyShotParameters,
    pub fire: FireParameters,
    pub spikes: SpikeParameters,
    pub splats: SplatParameters,
}

impl EnemyParameters {
    pub fn from_game(parameters: &GameParameters) -> Self {
        Self {
            flies: parameters.flies,
            gapers: parameters.gapers,
            headless: parameters.headless,
            horfs: parameters.horfs,
            clottys: parameters.clottys,
            hoppers: parameters.hoppers,
            flesh_mounds: parameters.flesh_mounds,
            mulligans: parameters.mulligans,
            pooters: parameters.pooters,
            shots: parameters.enemy_shots,
            fire: parameters.fire,
            spikes: parameters.spikes,
            splats: parameters.splats,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Enemies {
    parameters: EnemyParameters,
    flies: Vec<Fly>,
    walkers: Vec<Walker>,
    shots: Vec<EnemyShot>,
    splashes: Vec<Splash>,
    /// Whether the layout placed any enemy the game has (rooms with only other enemies are not
    /// fought yet).
    pub spawned_any: bool,
    /// `fra`: time in the room, for the frames the AI acts on.
    frames: Frames,
    /// Each enemy's number and hit points when last asked who was hurt (see `friends.rs`).
    health_seen: Vec<(u32, HitPoints)>,
    /// The numbers of the enemies alive when `take_deaths` last looked.
    living_seen: Vec<u32>,
    /// Where the player was when last seen, and how fast he moved (stage coordinates).
    player: Vec2,
    player_velocity: Vec2,
    /// The cells walkers marked (`pathfind()`), fading.
    tile_marks: TileMarks,
    /// Blood the enemies and their shots left since the room last took it.
    splatters: Vec<Splatter>,
}

/// What touching the enemies did to the player.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Bump {
    pub contact: Contact,
    /// Half hearts of damage to the player, and who dealt them.
    pub hurt: Option<(Hearts, Killer)>,
}

impl Enemies {
    pub fn from_layout(layout: &RoomLayout, parameters: &EnemyParameters, random: &mut Random) -> Self {
        let mut flies = Vec::new();
        let mut walkers = Vec::new();
        for cell in &layout.cells {
            match spawn::spawn(
                cell,
                u32::try_from(flies.len() + walkers.len()).unwrap_or(u32::MAX),
                parameters,
                random,
            ) {
                Some(Spawned::Fly(fly)) => flies.push(fly),
                Some(Spawned::Walker(walker)) => walkers.push(*walker),
                None => {}
            }
        }
        Self {
            parameters: *parameters,
            spawned_any: !flies.is_empty() || !walkers.is_empty(),
            flies,
            walkers,
            shots: Vec::new(),
            splashes: Vec::new(),
            frames: Frames::ZERO,
            health_seen: Vec::new(),
            living_seen: Vec::new(),
            player: Vec2::ZERO,
            player_velocity: Vec2::ZERO,
            tile_marks: TileMarks::default(),
            splatters: Vec::new(),
        }
    }

    /// How fast the player moves, for the enemies that aim ahead of him (world coordinates).
    pub fn watch_player_velocity(&mut self, velocity: Vec2) {
        self.player_velocity = Vec2::new(velocity.x, -velocity.y);
    }

    /// The original's `flyshut`: live enemies that buzz (all of ours are flies).
    pub fn buzzing_count(&self) -> usize {
        self.flies.iter().filter(|fly| fly.is_alive()).count()
    }

    #[cfg(test)]
    pub fn has_pooter(&self) -> bool {
        self.flies.iter().any(|fly| fly.kind.is_pooter())
    }

    /// For tests elsewhere that need a walker pressed against something.
    #[cfg(test)]
    pub fn add_walker(&mut self, walker: Walker) {
        self.walkers.push(walker);
    }

    /// Any enemy still alive keeps the doors shut.
    pub fn any_alive(&self) -> bool {
        self.flies.iter().any(Fly::is_alive) || self.walkers.iter().any(Walker::is_alive)
    }

    /// Thinks once per original frame crossed, then moves; finished deaths are removed. `player`
    /// is the player's world position. Returns the cells (world) flies pecked, with the pecking
    /// fly's index, for the room to resolve with [`Enemies::peck_landed`].
    pub fn advance(
        &mut self,
        elapsed: Frames,
        grid_objects: &[GridObject],
        player: Vec2,
        room_interior: Bounds,
        random: &mut Random,
        sounds: &mut Vec<SoundEvent>,
    ) -> Vec<(usize, Vec2)> {
        let first_frame = self.frames.whole() + 1;
        self.frames += elapsed;
        self.player = world_to_stage(player);
        let mut grid = Grid::new(grid_objects, self.player)
            .with_player_velocity(self.player_velocity)
            .with_marks(std::mem::take(&mut self.tile_marks));
        let enemy_count = self.flies.len() + self.walkers.len();
        let mut pecks = Vec::new();
        for number in first_frame..=self.frames.whole() {
            let frame = Frames::from_number(number);
            grid.sight.fade_marks(frame);
            self.bleed_dying(Frames::from_number(number - first_frame), random);
            for index in 0..self.flies.len() {
                let turn = self.flies[index].think(frame, &grid, &self.parameters, random);
                if let Some(cell) = turn.peck {
                    pecks.push((index, stage_to_world(cell)));
                }
                if let Some(target) = turn.shoots_at {
                    self.fire(index, target, sounds);
                }
            }
            for index in 0..self.walkers.len() {
                let turn = self.walkers[index].think(frame, &mut grid, &self.parameters, enemy_count, random);
                self.walker_turned(index, turn, sounds);
            }
            self.feel_the_floor(frame, grid_objects, random, sounds);
            self.release_flies(random);
        }
        for fly in &mut self.flies {
            fly.advance(elapsed);
        }
        for walker in &mut self.walkers {
            walker.advance(elapsed, &grid.solid_tiles, room_interior, &self.parameters);
        }
        self.separate_enemies(elapsed);
        for fly in &mut self.flies {
            // Kept inside the room, so nothing ever ends up beyond its walls.
            let world = room_interior.clamp(stage_to_world(fly.position));
            fly.position = world_to_stage(world);
        }
        let frames_crossed = (self.frames.whole() + 1).saturating_sub(first_frame);
        self.advance_shots((elapsed, frames_crossed), &grid, random, sounds);
        self.tile_marks = grid.sight.marks().clone();
        pecks
    }
}

#[cfg(test)]
mod blood_tests;
#[cfg(test)]
mod chaser_tests;
#[cfg(test)]
mod tests;

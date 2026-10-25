//! What the walkers' tests share: rooms with one marker, running the room, and the player's body.

use glam::Vec2;

use crate::game::original_units::Frames;
use crate::game::parameters::GameParameters;
use crate::game::random::Random;
use crate::game::room::enemies::{Enemies, EnemyParameters};
use crate::game::room::grid_kind::{GridObjectKind, TileKind};
use crate::game::room::grid_object::GridObject;
use crate::game::room::layout::{LayoutCell, RoomLayout};
use crate::game::room::{interior_bounds, stage_to_world};
use crate::game::sound_event::SoundEvent;

pub(super) const GAPER_MARKER: f32 = -1.0;
pub(super) const CALM_GAPER_MARKER: f32 = -1.1;
pub(super) const HEADLESS_MARKER: f32 = -2.0;
pub(super) const HEADLESS_MARKER_WITHOUT_BLOOD: f32 = -2.1;
pub(super) const HORF_MARKER: f32 = -3.0;
pub(super) const ONE_FRAME: Frames = Frames(1.0);
/// The appearing clip, then a little walking.
pub(super) const FRAMES_TO_WALK: u32 = 150;

pub(super) fn parameters() -> EnemyParameters {
    EnemyParameters::from_game(&GameParameters::ORIGINAL)
}

pub(super) fn room_with_marker(value: f32, column: u32, row: u32) -> Enemies {
    let layout = RoomLayout {
        cells: vec![LayoutCell { column, row, value }],
        ..RoomLayout::default()
    };
    Enemies::from_layout(&layout, &parameters(), &mut Random::with_seed(1))
}

pub(super) fn rock_on(column: u32, row: u32) -> GridObject {
    GridObject::on_tile(
        GridObjectKind::Tile(TileKind::Rock),
        column,
        row,
        GameParameters::ORIGINAL.breakables,
    )
}

/// Runs the room `frames` frames with the player at `player` (world), the sounds it made.
pub(super) fn run(enemies: &mut Enemies, frames: u32, player: Vec2, objects: &[GridObject]) -> Vec<SoundEvent> {
    let mut random = Random::with_seed(2);
    let mut sounds = Vec::new();
    for _ in 0..frames {
        enemies.advance(ONE_FRAME, objects, player, interior_bounds(), &mut random, &mut sounds);
        enemies.remove_finished();
    }
    sounds
}

pub(super) fn walker_world_position(enemies: &Enemies) -> Vec2 {
    stage_to_world(enemies.walkers[0].position)
}

/// The player's body at `position`, standing still.
pub(super) fn player_body(position: Vec2) -> crate::game::ball_contact::Body {
    crate::game::ball_contact::Body {
        position,
        velocity: Vec2::ZERO,
        mass: GameParameters::ORIGINAL.pickups.player_mass,
    }
}

//! The number the original keeps for each cell of the grid (`levz`): 0 for floor, 1 for a rock, up to
//! 1.9 for the other blocks, 3 for a pit; what a breakable is worth falls as it is hurt.

use super::GridObject;
use crate::game::room::grid_kind::{GridObjectKind, TileKind};

/// A rock (and the level's marked rock, which is the same).
const ROCK_VALUE: f32 = 1.0;
/// The dark rock nothing breaks.
const BLOCK_VALUE: f32 = 1.9;
const PIT_VALUE: f32 = 3.0;
/// Spikes and webs, which can be walked over.
const FLOOR_HAZARD_VALUE: f32 = 0.99;
/// A key block until it is opened.
const KEY_BLOCK_VALUE: f32 = 2.0;
const FLOOR_VALUE: f32 = 0.0;

impl GridObject {
    /// The value its cell holds.
    pub fn level_value(&self) -> f32 {
        match (self.kind, &self.breakable) {
            (GridObjectKind::Breakable(_), Some(state)) => state.durability,
            (GridObjectKind::Tile(_), _) if self.cleared => FLOOR_VALUE,
            (GridObjectKind::Tile(TileKind::Rock | TileKind::TintedRock), _) => ROCK_VALUE,
            (GridObjectKind::Tile(TileKind::Block), _) => BLOCK_VALUE,
            (GridObjectKind::Tile(TileKind::Pit), _) => PIT_VALUE,
            (GridObjectKind::Tile(TileKind::Spikes | TileKind::SpiderWeb), _) => FLOOR_HAZARD_VALUE,
            _ if self.is_closed_key_block() => KEY_BLOCK_VALUE,
            _ => FLOOR_VALUE,
        }
    }
}

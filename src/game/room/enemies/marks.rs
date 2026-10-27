//! The room's tile marks as the rest of the room writes and reads them: `spaw()` marks the tile it
//! gives away (DoAction 29095), and `posw()` skips a tile that holds one (DoAction 29037).

use glam::Vec2;

use crate::game::room::enemies::Enemies;
use crate::game::room::world_to_stage;

impl Enemies {
    /// Writes `value` into the cell under `world_point`; the marks fade as the walkers' do.
    pub fn mark_tile(&mut self, world_point: Vec2, value: f32) {
        self.tile_marks.mark(world_to_stage(world_point), value);
    }

    /// What the cell under `world_point` holds of the marks.
    pub fn tile_mark_at(&self, world_point: Vec2) -> f32 {
        self.tile_marks.value_at(world_to_stage(world_point))
    }

    /// Where the enemies stand (world).
    pub fn positions(&self) -> Vec<Vec2> {
        self.friend_targets().iter().map(|target| target.position).collect()
    }
}

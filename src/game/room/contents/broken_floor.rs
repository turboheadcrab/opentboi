//! The floor a blast breaks between pits (`breakfloor()` and `breakall()`): a rock or web cleared
//! within reach of the blast, with a pit on both sides of it along a row or a column, gets a
//! broken-floor tile drawn over the rubble at the end of the frame.

use glam::Vec2;

use crate::game::original_units::distance;
use crate::game::room::TILE_SIZE;
use crate::game::room::contents::RoomContents;
use crate::game::room::grid_kind::{GridObjectKind, TileKind};

impl RoomContents {
    /// Marks the cleared tile at `cleared_center` as broken floor if the blast at `blast_center`
    /// reached it and pits lie on opposite sides of it.
    pub(super) fn break_floor_between_pits(&mut self, cleared_center: Vec2, blast_center: Vec2) {
        let reach = self.parameters.explosion.floor_break_reach;
        let tile = crate::game::bounds::Bounds::centered(cleared_center, Vec2::splat(TILE_SIZE.get()));
        if distance(tile.clamp(blast_center), blast_center) >= reach {
            return;
        }
        let step = TILE_SIZE.get();
        let pit_beside = |offset: Vec2| self.is_open_pit_at(cleared_center + offset);
        let between_pits = (pit_beside(Vec2::new(step, 0.0)) && pit_beside(Vec2::new(-step, 0.0)))
            || (pit_beside(Vec2::new(0.0, step)) && pit_beside(Vec2::new(0.0, -step)));
        if between_pits && !self.broken_floors.contains(&cleared_center) {
            self.broken_floors.push(cleared_center);
        }
    }

    fn is_open_pit_at(&self, point: Vec2) -> bool {
        self.grid_objects.iter().any(|object| {
            object.kind() == GridObjectKind::Tile(TileKind::Pit)
                && !object.is_cleared()
                && object.tile_bounds().contains(point)
        })
    }
}

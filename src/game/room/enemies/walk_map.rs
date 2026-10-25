//! The room's tiles as walkers see them: which are blocked (`levz` values of 1 and above, pits
//! included), whether a straight line between two points is clear (`linecheck()`, DoAction 6155) and
//! how many steps each tile is from a goal (the flood fill in `pathfind()`, 5777–5990). Points are in
//! stage coordinates, y down.

use std::collections::VecDeque;

use glam::Vec2;

use crate::game::room::grid_cells::{Mover, doorway_cells, ingrid, levzz, outgrid};
use crate::game::room::grid_object::GridObject;
use crate::game::room::{COLUMN_COUNT, ROW_COUNT, tile_center, world_to_stage};
use crate::whole_numbers::clamped_u32;

const COLUMNS: usize = COLUMN_COUNT as usize;
const ROWS: usize = ROW_COUNT as usize;
/// `linecheck()` samples the line every 6 pixels, at the corners of a 10-pixel square.
const SIGHT_STEP: f32 = 6.0;
const SIGHT_CORNER: f32 = 5.0;
/// What the grid holds where something stands in the way, and where nothing does.
const BLOCKED_VALUE: f32 = 1.0;
const FLOOR_VALUE: f32 = 0.0;

/// The index of the tile `tiles_in` tiles from the edge, held within `count` tiles.
fn index_within(tiles_in: f32, count: usize) -> usize {
    usize::try_from(clamped_u32(tiles_in))
        .unwrap_or_default()
        .min(count - 1)
}

/// A tile: its column and row, from the top left.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Tile {
    column: usize,
    row: usize,
}

impl Tile {
    /// The tile holding `stage_point`, the nearest one for points beyond the room.
    pub fn containing(stage_point: Vec2) -> Self {
        let (column, row) = ingrid(stage_point, true).tile().unwrap_or_default();
        Self {
            column: index_within(column as f32, COLUMNS),
            row: index_within(row as f32, ROWS),
        }
    }

    /// Where its center is.
    pub fn center(self) -> Vec2 {
        let column = u32::try_from(self.column).unwrap_or_default();
        let row = u32::try_from(self.row).unwrap_or_default();
        world_to_stage(tile_center(column, row))
    }

    fn index(self) -> usize {
        self.row * COLUMNS + self.column
    }

    /// The tiles beside it, east first, then west, north and south (the order `pathfind()` weighs
    /// them in).
    fn neighbors(self) -> impl Iterator<Item = Self> {
        let Self { column, row } = self;
        [
            (column + 1 < COLUMNS).then(|| Self {
                column: column + 1,
                row,
            }),
            column.checked_sub(1).map(|column| Self { column, row }),
            row.checked_sub(1).map(|row| Self { column, row }),
            (row + 1 < ROWS).then(|| Self { column, row: row + 1 }),
        ]
        .into_iter()
        .flatten()
    }
}

/// Steps from each tile to a goal, by walking around what is blocked.
#[derive(Clone, Debug)]
pub(super) struct Distances {
    steps: [Option<u16>; COLUMNS * ROWS],
}

impl Distances {
    pub fn steps_from(&self, tile: Tile) -> Option<u16> {
        self.steps[tile.index()]
    }

    /// The neighbor of `tile` nearest the goal, the first of equals (none if it cannot get there).
    pub fn next_step(&self, tile: Tile) -> Option<Tile> {
        let mut best: Option<(u16, Tile)> = None;
        for neighbor in tile.neighbors() {
            if let Some(steps) = self.steps_from(neighbor)
                && best.is_none_or(|(least, _)| steps < least)
            {
                best = Some((steps, neighbor));
            }
        }
        best.map(|(_, neighbor)| neighbor)
    }
}

#[derive(Clone, Debug)]
pub(super) struct WalkMap {
    blocked: [bool; COLUMNS * ROWS],
}

impl WalkMap {
    pub fn new(objects: &[GridObject]) -> Self {
        let mut blocked = [false; COLUMNS * ROWS];
        for row in 0..ROW_COUNT {
            for column in 0..COLUMN_COUNT {
                let center = tile_center(column, row);
                let index =
                    usize::try_from(row).unwrap_or_default() * COLUMNS + usize::try_from(column).unwrap_or_default();
                blocked[index] = objects
                    .iter()
                    .any(|object| object.blocks_walking() && object.tile_bounds().contains(center));
            }
        }
        Self { blocked }
    }

    fn is_blocked(&self, stage_point: Vec2) -> bool {
        let world = crate::game::room::stage_to_world(stage_point);
        !crate::game::room::interior_bounds().contains(world) || self.blocked[Tile::containing(stage_point).index()]
    }

    /// The center of the free cell nearest `stage_point` (`ingrid()` and `outgrid()`), none if it is a
    /// wall, a doorway (`levzz()` shows doorways to enemies as solid) or blocked.
    pub fn free_spot(&self, stage_point: Vec2) -> Option<Vec2> {
        let cell = ingrid(stage_point, false);
        let value = match cell.tile() {
            Some((column, row)) => {
                let tile = Tile {
                    column: usize::try_from(column).ok()?,
                    row: usize::try_from(row).ok()?,
                };
                if self.blocked[tile.index()] {
                    BLOCKED_VALUE
                } else {
                    FLOOR_VALUE
                }
            }
            None => BLOCKED_VALUE,
        };
        (levzz(Mover::Enemy, cell, &doorway_cells(), value) < BLOCKED_VALUE).then(|| outgrid(cell))
    }

    /// `linecheck()`: nothing blocked along the line, the tile it starts in aside.
    pub fn clear_line(&self, from: Vec2, target: Vec2) -> bool {
        let start = Tile::containing(from);
        let distance = from.distance(target);
        let step = (target - from).normalize_or_zero() * SIGHT_STEP;
        let corners = [
            Vec2::new(SIGHT_CORNER, SIGHT_CORNER),
            Vec2::new(-SIGHT_CORNER, SIGHT_CORNER),
            Vec2::new(-SIGHT_CORNER, -SIGHT_CORNER),
            Vec2::new(SIGHT_CORNER, -SIGHT_CORNER),
        ];
        let mut point = from;
        let mut travelled = 0.0;
        while travelled < distance {
            point += step;
            let blocked = corners
                .iter()
                .any(|&corner| Tile::containing(point + corner) != start && self.is_blocked(point + corner));
            if blocked {
                return false;
            }
            travelled += SIGHT_STEP;
        }
        true
    }

    /// How many steps every tile is from `goal`; `open` (the walker's own tile) counts as walkable
    /// even when something stands on it.
    pub fn distances_to(&self, goal: Tile, open: Tile) -> Distances {
        let mut steps = [None; COLUMNS * ROWS];
        steps[goal.index()] = Some(0);
        let mut queue = VecDeque::from([goal]);
        while let Some(tile) = queue.pop_front() {
            let next = steps[tile.index()].map_or(0, |reached| reached + 1);
            for neighbor in tile.neighbors() {
                let walkable = !self.blocked[neighbor.index()] || neighbor == open;
                if walkable && steps[neighbor.index()].is_none() {
                    steps[neighbor.index()] = Some(next);
                    queue.push_back(neighbor);
                }
            }
        }
        Distances { steps }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::parameters::GameParameters;
    use crate::game::room::grid_kind::{GridObjectKind, TileKind};

    fn rock_on(column: u32, row: u32) -> GridObject {
        GridObject::on_tile(
            GridObjectKind::Tile(TileKind::Rock),
            column,
            row,
            GameParameters::ORIGINAL.breakables,
        )
    }

    fn tile(column: usize, row: usize) -> Tile {
        Tile { column, row }
    }

    #[test]
    fn a_rock_blocks_the_line_but_an_open_room_does_not() {
        let open = WalkMap::new(&[]);
        let walled = WalkMap::new(&[rock_on(5, 3)]);
        let from = tile(2, 3).center();
        let target = tile(9, 3).center();
        assert!(open.clear_line(from, target));
        assert!(!walled.clear_line(from, target));
    }

    #[test]
    fn distances_count_the_steps_round_what_is_blocked() {
        let wall: Vec<GridObject> = (0..6).map(|row| rock_on(6, row)).collect();
        let map = WalkMap::new(&wall);
        let distances = map.distances_to(tile(8, 0), tile(4, 0));
        assert_eq!(distances.steps_from(tile(8, 0)), Some(0));
        assert_eq!(
            distances.steps_from(tile(5, 0)),
            Some(6 + 3 + 6),
            "down to the gap, across, back up"
        );
        assert_eq!(distances.steps_from(tile(6, 2)), None, "a rock");
    }

    #[test]
    fn the_next_step_prefers_east_among_equals_and_none_when_walled_in() {
        let map = WalkMap::new(&[]);
        let distances = map.distances_to(tile(5, 3), tile(2, 2));
        assert_eq!(distances.next_step(tile(4, 4)), Some(tile(5, 4)));
        let walled = WalkMap::new(&[rock_on(1, 0), rock_on(0, 1)]);
        let trapped = walled.distances_to(tile(5, 3), tile(0, 0));
        assert_eq!(trapped.next_step(tile(0, 0)), None);
    }
}

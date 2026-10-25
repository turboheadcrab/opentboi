//! The original's grid, which is 16 cells wide and holds the room's 13×7 floor with a ring of walls
//! around it: the cell of a place (`ingrid()`, DoAction 1382–1401), the place of a cell
//! (`outgrid()`, 1375–1381) and the value a mover sees in a cell (`levzz()`, from the script's
//! first line). Places are in stage coordinates (y down), cells are numbered along the rows.

use glam::Vec2;

use crate::game::room::doors::DoorSide;
use crate::game::room::{
    COLUMN_COUNT, ORIGINAL_INTERIOR_FIRST_COLUMN, ORIGINAL_INTERIOR_FIRST_ROW, ROW_COUNT, TILE_SIZE,
};
use crate::whole_numbers::clamped_u32;

/// How many cells make a row of the grid.
pub const ROW_LENGTH: u32 = 16;

/// What a restricted lookup keeps places within (`ingrid()`'s third argument): the floor's cell
/// centers; otherwise the floor and the wall ring around it.
const FLOOR_MINIMUM: Vec2 = Vec2::new(80.0, 160.0);
const FLOOR_MAXIMUM: Vec2 = Vec2::new(560.0, 400.0);
const WALL_RING_MINIMUM: Vec2 = Vec2::new(20.0, 110.0);
const WALL_RING_MAXIMUM: Vec2 = Vec2::new(620.0, 447.0);

/// What an enemy sees in the cell in front of a door, whatever it holds.
const DOORWAY_VALUE_FOR_ENEMIES: f32 = 2.0;
/// What looks at the grid: enemies (types above 10) and everything else (the player, tears,
/// pickups and bombs).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mover {
    Enemy,
    #[cfg_attr(
        not(test),
        allow(dead_code, reason = "the player, tears and pickups look at the grid too")
    )]
    Other,
}

/// A cell of the grid: its column and row from the stage's top left.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct GridCell {
    pub column: u32,
    pub row: u32,
}

impl GridCell {
    /// The cell's number in the original (`column + 16 * row`).
    pub fn index(self) -> u32 {
        self.column + ROW_LENGTH * self.row
    }

    #[cfg(test)]
    /// The cell of the floor's tile `column`, `row` (counted from the floor's top left).
    pub fn of_tile(column: u32, row: u32) -> Self {
        Self {
            column: column + ORIGINAL_INTERIOR_FIRST_COLUMN.unsigned_abs(),
            row: row + ORIGINAL_INTERIOR_FIRST_ROW.unsigned_abs(),
        }
    }

    /// The floor's tile this cell is, if it is not in the wall ring or beyond.
    pub fn tile(self) -> Option<(u32, u32)> {
        let column = self.column.checked_sub(ORIGINAL_INTERIOR_FIRST_COLUMN.unsigned_abs())?;
        let row = self.row.checked_sub(ORIGINAL_INTERIOR_FIRST_ROW.unsigned_abs())?;
        (column < COLUMN_COUNT && row < ROW_COUNT).then_some((column, row))
    }
}

/// `ingrid()`: the cell nearest `stage_point`, held to the floor's cell centers if `restricted`
/// (DoAction 1386, DoAction 1387) and to the floor and its walls otherwise (DoAction 1391,
/// DoAction 1392).
pub fn ingrid(stage_point: Vec2, restricted: bool) -> GridCell {
    let (minimum, maximum) = if restricted {
        (FLOOR_MINIMUM, FLOOR_MAXIMUM)
    } else {
        (WALL_RING_MINIMUM, WALL_RING_MAXIMUM)
    };
    let cells = stage_point.clamp(minimum, maximum) / TILE_SIZE;
    // Flash rounds halves up.
    GridCell {
        column: clamped_u32((cells.x + 0.5).floor()).min(ROW_LENGTH - 1),
        row: clamped_u32((cells.y + 0.5).floor()),
    }
}

/// `outgrid()`: where `cell`'s center is.
pub fn outgrid(cell: GridCell) -> Vec2 {
    Vec2::new(cell.column as f32, cell.row as f32) * TILE_SIZE
}

/// `levzz()` (DoAction 3, DoAction 5, DoAction 7): the value in `cell` for a `mover`, where `value` is what the grid holds
/// there. An enemy sees the cell of a door (`door_cells`, other than cell 0) as a solid 2 whether the
/// door is open or not, which keeps it out of doorways; everything else sees what is there.
pub fn levzz(mover: Mover, cell: GridCell, door_cells: &[GridCell], value: f32) -> f32 {
    if mover == Mover::Enemy && cell.index() > 0 && door_cells.contains(&cell) {
        DOORWAY_VALUE_FOR_ENEMIES
    } else {
        value
    }
}

/// The cells of the doors, in the wall ring.
pub fn doorway_cells() -> [GridCell; 4] {
    DoorSide::ALL.map(|side| {
        let (column, row) = side.doorway_cell();
        GridCell {
            column: u32::try_from(column + ORIGINAL_INTERIOR_FIRST_COLUMN).unwrap_or_default(),
            row: u32::try_from(row + ORIGINAL_INTERIOR_FIRST_ROW).unwrap_or_default(),
        }
    })
}

#[cfg(test)]
mod tests;

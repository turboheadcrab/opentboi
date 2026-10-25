//! A room's place on the floor map. The original numbers the map's places `row * 10 + column` on an
//! 8-row, 10-column grid and keeps the floor in an array with that index; a step left or right is
//! ±1 and a step up or down ±10, so a step off the end of a row lands on the next row. Places off
//! the map (the devil room) get numbers past the grid.

use crate::game::room::doors::DoorSide;

/// Places in a row of the map, and on the whole map.
pub const GRID_COLUMNS: u8 = 10;
pub const GRID_CELLS: usize = 80;
/// Doors lead only between these columns and rows (the original's own limits, which keep
/// the outermost column and row free).
const LEFTMOST_COLUMN_WITH_A_LEFT_DOOR: u8 = 2;
const RIGHTMOST_COLUMN_WITH_A_RIGHT_DOOR: u8 = 8;
const LAST_CELL_WITH_A_BOTTOM_DOOR: u8 = 69;
const FIRST_CELL_WITH_A_TOP_DOOR: u8 = 11;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FloorCell(u8);

impl FloorCell {
    /// Where every floor starts.
    pub const START: Self = Self(35);
    /// The devil room: off the map, so no step on the grid reaches it.
    pub const DEVIL_ROOM: Self = Self(166);

    /// The place with this array index, if it is on the map.
    pub fn on_map(index: i32) -> Option<Self> {
        u8::try_from(index)
            .ok()
            .filter(|&index| usize::from(index) < GRID_CELLS)
            .map(Self)
    }

    /// Every place on the map, in index order.
    pub fn all() -> impl Iterator<Item = Self> {
        (0..GRID_CELLS).filter_map(|index| u8::try_from(index).ok().map(Self))
    }

    /// The array index, for the map's per-place tables.
    pub fn index(self) -> usize {
        usize::from(self.0)
    }

    pub fn column(self) -> u8 {
        self.0 % GRID_COLUMNS
    }

    pub fn row(self) -> u8 {
        self.0 / GRID_COLUMNS
    }

    /// The place `steps` further along the array, if it is on the map.
    pub fn offset(self, steps: i32) -> Option<Self> {
        Self::on_map(i32::from(self.0) + steps)
    }

    /// Right, left, below and above along the array, in the original's order; `None` off the map.
    pub fn array_neighbors(self) -> [Option<Self>; 4] {
        let row = i32::from(GRID_COLUMNS);
        [1, -1, row, -row].map(|steps| self.offset(steps))
    }

    /// Whether the two are side by side or one above the other (along the array).
    pub fn touches(self, other: Self) -> bool {
        let difference = self.0.abs_diff(other.0);
        difference == 1 || difference == GRID_COLUMNS
    }

    /// The place a door on `side` leads to, if the map has a door there.
    pub fn through(self, side: DoorSide) -> Option<Self> {
        let row = i32::from(GRID_COLUMNS);
        match side {
            DoorSide::Left if self.column() >= LEFTMOST_COLUMN_WITH_A_LEFT_DOOR => self.offset(-1),
            DoorSide::Right if self.column() <= RIGHTMOST_COLUMN_WITH_A_RIGHT_DOOR => self.offset(1),
            DoorSide::Bottom if self.0 <= LAST_CELL_WITH_A_BOTTOM_DOOR => self.offset(row),
            DoorSide::Top if self.0 >= FIRST_CELL_WITH_A_TOP_DOOR => self.offset(-row),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_step_off_the_end_of_a_row_lands_on_the_next_row() {
        let end_of_row = FloorCell::on_map(9).expect("on the map");
        assert_eq!(end_of_row.offset(1).map(FloorCell::row), Some(1));
        assert_eq!(FloorCell::on_map(0).expect("on the map").offset(-1), None);
        assert_eq!(FloorCell::on_map(i32::try_from(GRID_CELLS).expect("small")), None);
    }

    #[test]
    fn doors_keep_to_the_original_limits() {
        let start = FloorCell::START;
        assert_eq!(start.through(DoorSide::Left), FloorCell::on_map(34));
        assert_eq!(start.through(DoorSide::Bottom), FloorCell::on_map(45));
        assert_eq!(FloorCell::on_map(1).and_then(|cell| cell.through(DoorSide::Left)), None);
        assert_eq!(
            FloorCell::on_map(9).and_then(|cell| cell.through(DoorSide::Right)),
            None
        );
        assert_eq!(
            FloorCell::on_map(70).and_then(|cell| cell.through(DoorSide::Bottom)),
            None
        );
        assert_eq!(FloorCell::on_map(10).and_then(|cell| cell.through(DoorSide::Top)), None);
        assert_eq!(FloorCell::DEVIL_ROOM.through(DoorSide::Top), None);
    }
}

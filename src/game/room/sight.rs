//! The room's grid as the original's line checks read it (`levz`, through `levzz()`): whether a walker
//! sees along a line (`linecheck()`) and whether a blast reaches along one (`linecheckxx()`). Both
//! step from the start towards the end, the last step landing up to a step past it, and look up the
//! cell under each point (`ingrid()` held to the floor and its walls). Points are in stage
//! coordinates (y down).
//!
//! The grid holds what `golev()` writes: 1.9 on the wall ring, 2 in a doorway with a door, 1.85 in
//! a hidden wall into a secret room, and each grid object's value. Walkers mark the cell they stand
//! on (`pathfind()`), and `actions2()` fades every value between 0 and 0.99 by 0.1 each third frame,
//! the marks and what a broken object leaves alike.

use glam::Vec2;

use crate::game::original_units::Frames;
use crate::game::room::doors::DoorSide;
use crate::game::room::grid_cells::{GridCell, Mover, ROW_LENGTH, doorway_cells, ingrid, levzz};
use crate::game::room::grid_object::GridObject;
use crate::game::room::world_to_stage;
use crate::game::room::{COLUMN_COUNT, ORIGINAL_INTERIOR_FIRST_COLUMN, ORIGINAL_INTERIOR_FIRST_ROW, ROW_COUNT};

/// Rows of the grid down to the bottom wall (`ingrid()` keeps places at y 447 or above, row 11).
const ROWS: u32 = 12;
const CELL_COUNT: usize = (ROW_LENGTH * ROWS) as usize;
/// What `golev()` writes into the wall ring, a door's doorway and a hidden wall into a secret room.
const WALL_VALUE: f32 = 1.9;
pub const DOOR_VALUE: f32 = 2.0;
pub const HIDDEN_WALL_VALUE: f32 = 1.85;
/// What a walker writes into the cell it stands on, unless the cell holds 0.99 or more (`pathfind()`).
const WALKER_MARK: f32 = 0.9;
const MARKABLE_BELOW: f32 = 0.99;
/// `actions2()`: values above 0 and below 0.99 fade by 0.1 each third frame, down to 0.
pub const FADING_EVERY: Frames = Frames(3.0);
pub const FADING_STEP: f32 = 0.1;
const FADING_BELOW: f32 = 0.99;
/// `linecheck()`: the line is looked at every 6 pixels, at the corners of a 10-pixel square
/// (`pff3()` for each), with 2.5 to spend: a cell of 1 or more ends the sight, one between 0.3 and
/// 1 costs 0.1, spikes and webs (0.99) cost nothing. The sight is clear while anything is left.
const WALK_SIGHT_STEP: f32 = 6.0;
const WALK_SIGHT_CORNER: f32 = 5.0;
const SIGHT_CREDIT: f32 = 2.5;
const SIGHT_COST: f32 = 0.1;
const SIGHT_BLOCKED_FROM: f32 = 1.0;
const SIGHT_COSTS_ABOVE: f32 = 0.3;
const SIGHT_IGNORES: f32 = 0.99;
/// `linecheckxx()`: every 10 pixels, at the point itself; a cell of 1.8 or more stops the blast,
/// except a hidden wall (1.85) and a pit (3).
const BLAST_STEP: f32 = 10.0;
const BLAST_STOPPED_FROM: f32 = 1.8;
const PIT_VALUE: f32 = 3.0;
/// `enfget()`: a line of no length counts as a thousandth of a pixel long.
const SHORTEST_LINE: f32 = 0.001;

#[derive(Clone, Debug)]
pub struct SightGrid {
    values: [f32; CELL_COUNT],
    /// The walkers' marks, kept apart from what the objects hold; a cell shows the larger. Both fade
    /// alike, so this is the value a mark writing over a fading object leaves.
    marks: TileMarks,
}

/// Walkers' marks on the grid, which outlive a turn of the room.
#[derive(Clone, Debug, PartialEq)]
pub struct TileMarks([f32; CELL_COUNT]);

impl Default for TileMarks {
    fn default() -> Self {
        Self([0.0; CELL_COUNT])
    }
}

impl TileMarks {
    /// Writes `value` into the cell under `stage_point` (`levz[ingrid(x, y)] = value`).
    pub fn mark(&mut self, stage_point: Vec2, value: f32) {
        self.0[index(ingrid(stage_point, true))] = value;
    }

    /// What the cell under `stage_point` holds.
    pub fn value_at(&self, stage_point: Vec2) -> f32 {
        self.0[index(ingrid(stage_point, true))]
    }

    /// `actions2()`'s fading, on `frame` of the room.
    pub fn fade(&mut self, frame: Frames) {
        if frame.is_on_cycle(FADING_EVERY, Frames::ZERO) {
            for value in &mut self.0 {
                *value = faded(*value);
            }
        }
    }
}

/// A value after one fading step.
pub fn faded(value: f32) -> f32 {
    if value > 0.0 && value < FADING_BELOW {
        (value - FADING_STEP).max(0.0)
    } else {
        value
    }
}

impl SightGrid {
    /// The wall ring, a door in every doorway, and `objects`.
    pub fn new(objects: &[GridObject]) -> Self {
        let mut values = [0.0; CELL_COUNT];
        let first_column = ORIGINAL_INTERIOR_FIRST_COLUMN.unsigned_abs() - 1;
        let last_column = first_column + COLUMN_COUNT + 1;
        let first_row = ORIGINAL_INTERIOR_FIRST_ROW.unsigned_abs() - 1;
        let last_row = first_row + ROW_COUNT + 1;
        for row in first_row..=last_row {
            for column in first_column..=last_column {
                if row == first_row || row == last_row || column == first_column || column == last_column {
                    values[index(GridCell { column, row })] = WALL_VALUE;
                }
            }
        }
        let mut grid = Self {
            values,
            marks: TileMarks::default(),
        };
        for side in DoorSide::ALL {
            grid.set_doorway(side, DOOR_VALUE);
        }
        for object in objects {
            let cell = ingrid(world_to_stage(object.center()), true);
            grid.values[index(cell)] = object.level_value();
        }
        grid
    }

    pub fn with_marks(self, marks: TileMarks) -> Self {
        Self { marks, ..self }
    }

    pub fn marks(&self) -> &TileMarks {
        &self.marks
    }

    pub fn fade_marks(&mut self, frame: Frames) {
        self.marks.fade(frame);
    }

    /// What the doorway on `side` holds: a door, a hidden wall, or the wall ring where there is none.
    pub fn set_doorway(&mut self, side: DoorSide, value: f32) {
        let cell = doorway_cells()[DoorSide::ALL
            .iter()
            .position(|&other| other == side)
            .unwrap_or_default()];
        self.values[index(cell)] = value;
    }

    /// `levzz()`: the value `mover` sees in `cell`.
    fn value(&self, mover: Mover, cell: GridCell) -> f32 {
        let cell_index = index(cell);
        let value = self.values[cell_index].max(self.marks.0[cell_index]);
        levzz(mover, cell, &doorway_cells(), value)
    }

    /// `pathfind()`: a walker standing at `stage_point` marks its cell.
    pub fn mark_walker(&mut self, stage_point: Vec2) {
        let cell = ingrid(stage_point, false);
        if self.value(Mover::Enemy, cell) < MARKABLE_BELOW {
            self.marks.0[index(cell)] = WALKER_MARK;
        }
    }

    /// `linecheck()`, as an enemy sees the grid. The original also skips cells holding the start
    /// cell's number (`grox`), which no value can equal, so nothing is skipped.
    pub fn walk_line_clear(&self, from: Vec2, target: Vec2) -> bool {
        let corners = [
            Vec2::new(WALK_SIGHT_CORNER, WALK_SIGHT_CORNER),
            Vec2::new(WALK_SIGHT_CORNER, -WALK_SIGHT_CORNER),
            Vec2::new(-WALK_SIGHT_CORNER, -WALK_SIGHT_CORNER),
            Vec2::new(-WALK_SIGHT_CORNER, WALK_SIGHT_CORNER),
        ];
        let mut credit = SIGHT_CREDIT;
        for point in line_points(from, target, WALK_SIGHT_STEP) {
            for corner in corners {
                let value = self.value(Mover::Enemy, ingrid(point + corner, false));
                if value == SIGHT_IGNORES {
                    continue;
                }
                if value >= SIGHT_BLOCKED_FROM {
                    return false;
                }
                if value > SIGHT_COSTS_ABOVE {
                    credit -= SIGHT_COST;
                }
            }
        }
        credit > 0.0
    }

    /// `linecheckxx()` from a bomb's blast (which sees the grid as it is, doorways too). The
    /// original also lets a cell equal to `gro` through, which is never set, so none is.
    pub fn blast_line_clear(&self, from: Vec2, target: Vec2) -> bool {
        line_points(from, target, BLAST_STEP).all(|point| {
            let value = self.value(Mover::Other, ingrid(point, false));
            value < BLAST_STOPPED_FROM || value == HIDDEN_WALL_VALUE || value == PIT_VALUE
        })
    }

    /// [`Self::blast_line_clear`] between world points.
    pub fn blast_reaches(&self, blast: Vec2, target: Vec2) -> bool {
        self.blast_line_clear(world_to_stage(blast), world_to_stage(target))
    }
}

fn index(cell: GridCell) -> usize {
    usize::try_from(cell.index()).unwrap_or_default().min(CELL_COUNT - 1)
}

/// The points a line check looks at: a step at a time from `from` towards `target`, as long as the
/// distance walked before the step is short of the line's length.
fn line_points(from: Vec2, target: Vec2, step_length: f32) -> impl Iterator<Item = Vec2> {
    let length = from.distance(target).max(SHORTEST_LINE);
    let step = (target - from) / length * step_length;
    let steps = (length / step_length).ceil();
    (1..).map_while(move |count: u16| {
        let count = f32::from(count);
        (count <= steps).then(|| from + step * count)
    })
}

#[cfg(test)]
mod tests;

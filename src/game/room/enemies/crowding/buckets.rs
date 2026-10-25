//! The original's bucket grid for enemy contacts (`physix()`, DoAction 25625–25690, `gridp()`,
//! `gridput()`): a bitmap of cells 100 pixels wide, each with ten slots side by side, in which
//! every enemy is entered into the four cells around it (two across, two down). Only enemies
//! sharing a cell are tested against each other. A cell that is full gives its eleventh enemy
//! the first slot of the cell beside it, over whoever was there; slots beyond the bitmap's edge
//! are lost.

use glam::Vec2;

use crate::game::original_units::Pixels;

use crate::whole_numbers::clamped_u32;

/// `gridv`: the width of a cell; `gridmax`: its slots.
const CELL_SIZE: Pixels = Pixels(100.0);
pub(super) const SLOTS: usize = 10;
/// `gxe`, `gye`: the cells across and down, for the 640 × 480 stage.
const COLUMNS: usize = 8;
const ROWS: usize = 6;
const WIDTH: usize = COLUMNS * SLOTS;
/// The entry shifts half a cell up and left, so an enemy sits among the four cells nearest it.
const ENTRY_SHIFT: f32 = 0.5;

/// What fills the bitmap's pixels: an enemy's number in the room's list, or nothing.
#[derive(Clone, Debug)]
pub(super) struct Buckets {
    pixels: [[Option<usize>; WIDTH]; ROWS],
}

impl Buckets {
    pub fn new() -> Self {
        Self {
            pixels: [[None; WIDTH]; ROWS],
        }
    }

    /// Enters enemy `index`, standing at `stage_point`, into its four cells.
    pub fn enter(&mut self, index: usize, stage_point: Vec2) {
        let column = usize::try_from(clamped_u32(stage_point.x / CELL_SIZE.get() + ENTRY_SHIFT)).unwrap_or_default();
        let row = usize::try_from(clamped_u32(stage_point.y / CELL_SIZE.get() - ENTRY_SHIFT)).unwrap_or_default() + 1;
        let first = column * SLOTS + 1;
        for (pixel, row) in [
            (first + SLOTS, row),
            (first + SLOTS, row + 1),
            (first, row),
            (first, row + 1),
        ] {
            self.put(pixel, row, index);
        }
    }

    /// The first free slot from `pixel`, at most ten along; the last one tried is written even when
    /// taken.
    fn put(&mut self, mut pixel: usize, row: usize, index: usize) {
        let mut tried = 0;
        while self.get(pixel, row).is_some() && tried < SLOTS {
            pixel += 1;
            tried += 1;
        }
        if let Some(cell) = self.pixels.get_mut(row).and_then(|line| line.get_mut(pixel)) {
            *cell = Some(index);
        }
    }

    fn get(&self, pixel: usize, row: usize) -> Option<usize> {
        self.pixels.get(row).and_then(|line| line.get(pixel)).copied().flatten()
    }

    /// Every pair of enemies in one cell, cell by cell, in the order they were entered.
    pub fn pairs(&self) -> Vec<(usize, usize)> {
        let mut pairs = Vec::new();
        for first_pixel in (1..WIDTH).step_by(SLOTS) {
            for row in 1..ROWS {
                for slot in 0..SLOTS {
                    let Some(first) = self.get(first_pixel + slot, row) else {
                        break;
                    };
                    for other in (slot + 1)..SLOTS {
                        let Some(second) = self.get(first_pixel + other, row) else {
                            break;
                        };
                        pairs.push((first, second));
                    }
                }
            }
        }
        pairs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pair_exists(buckets: &Buckets, first: usize, second: usize) -> bool {
        buckets
            .pairs()
            .iter()
            .any(|&(one, other)| (one, other) == (first, second) || (one, other) == (second, first))
    }

    #[test]
    fn enemies_near_each_other_share_a_cell_and_far_ones_do_not() {
        let mut buckets = Buckets::new();
        buckets.enter(0, Vec2::new(300.0, 250.0));
        buckets.enter(1, Vec2::new(330.0, 260.0));
        buckets.enter(2, Vec2::new(90.0, 150.0));
        assert!(pair_exists(&buckets, 0, 1));
        assert!(!pair_exists(&buckets, 0, 2));
    }

    #[test]
    fn the_eleventh_in_a_cell_takes_the_next_cells_first_slot() {
        let mut buckets = Buckets::new();
        for index in 0..11 {
            buckets.enter(index, Vec2::new(300.0, 250.0));
        }
        assert!(pair_exists(&buckets, 0, 9), "the full cell still holds the first ten");
        assert!(
            pair_exists(&buckets, 10, 1),
            "the eleventh sits where the next cell's first was"
        );
    }
}

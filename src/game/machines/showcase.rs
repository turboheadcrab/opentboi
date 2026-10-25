//! Temporary: one machine of each kind in the shop, for checking their art and rules.

use crate::game::machines::{MachineKind, Machines};
use crate::game::room::tile_center;

/// Clear of the doorways (middle column and row) and of the wares in row 4: four along the bottom
/// row, the devil beggar and the shell game (played from below) in the upper half.
const SHOWCASE_CELLS: [(u32, u32); 6] = [(1, 6), (3, 6), (9, 6), (11, 6), (2, 2), (10, 2)];

impl Machines {
    pub fn add_one_of_each(&mut self) {
        for (kind, (column, row)) in MachineKind::ALL.into_iter().zip(SHOWCASE_CELLS) {
            self.place(kind, tile_center(column, row));
        }
    }
}

//! Temporary: pickups placed in the boss room for checking their art and rules.

use glam::Vec2;

use crate::game::pickups::Pickups;
use crate::game::pickups::card::Card;
use crate::game::pickups::item::Item;
use crate::game::pickups::kind::{HeartKind, PickupKind};
use crate::game::pickups::pill::Pill;
use crate::game::pickups::trinket::Trinket;
use crate::game::room::tile_center;

const SHOWCASE_COLUMNS: [u32; 6] = [1, 3, 5, 7, 9, 11];
const SHOWCASE_ROWS: [u32; 3] = [1, 3, 5];
const SHOWCASE_EXTRA_COLUMNS: [u32; 2] = [4, 8];
const SHOWCASE_EXTRA_ROW: u32 = 6;

impl Pickups {
    /// Until the item pools exist: every empty pedestal in the room holds `item`.
    pub fn stock_pedestals(&mut self, item: Item) {
        for kind in self.lying_kinds_mut() {
            if *kind == PickupKind::ItemPedestal {
                *kind = PickupKind::Item(item);
            }
        }
    }

    /// For checking that two eternal hearts make a heart container.
    pub fn add_eternal_hearts(&mut self) {
        for column in SHOWCASE_EXTRA_COLUMNS {
            self.spawn(
                PickupKind::Heart(HeartKind::Eternal),
                tile_center(column, SHOWCASE_EXTRA_ROW),
                Vec2::ZERO,
            );
        }
    }

    /// For checking cards and pills: all 27 cards in the top and bottom rows and either side of the
    /// middle row, clear of the doorways and the pedestal, and the pills between them.
    pub fn add_every_card(&mut self) {
        let rows = [1, 5]
            .into_iter()
            .flat_map(|row| (1..=11).map(move |column| (column, row)));
        let middle = [1, 2, 3, 9, 10].map(|column| (column, 3));
        for (card, (column, row)) in Card::ALL.into_iter().zip(rows.chain(middle)) {
            self.place(PickupKind::Card(card), tile_center(column, row));
        }
        // The six pills fill the rest of the middle row.
        for (pill, column) in Pill::ALL.into_iter().zip([4, 5, 6, 7, 8, 11]) {
            self.place(PickupKind::Pill(pill), tile_center(column, 3));
        }
    }

    /// For checking trinkets: all 34 in the rows the cards leave free, apart from the cells in front of
    /// the doorways at the top and bottom.
    pub fn add_every_trinket(&mut self) {
        const ROWS: [u32; 4] = [2, 4, 0, 6];
        const DOORWAY_COLUMN: u32 = 6;
        const EDGE_ROWS: [u32; 2] = [0, 6];
        let cells = ROWS
            .into_iter()
            .flat_map(|row| (1..=11).map(move |column| (column, row)))
            .filter(|(column, row)| !(*column == DOORWAY_COLUMN && EDGE_ROWS.contains(row)));
        for (trinket, (column, row)) in Trinket::ALL.into_iter().zip(cells) {
            self.place(PickupKind::Trinket(trinket), tile_center(column, row));
        }
    }

    /// For checking the art and rules: one of every kind in rows across the room.
    pub fn add_one_of_each(&mut self) {
        let cells = SHOWCASE_ROWS
            .iter()
            .flat_map(|&row| SHOWCASE_COLUMNS.iter().map(move |&column| (column, row)));
        for (kind, (column, row)) in PickupKind::ALL.into_iter().zip(cells) {
            self.place(kind, tile_center(column, row));
        }
    }
}

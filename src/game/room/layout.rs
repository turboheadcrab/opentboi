//! Room layouts as the original stores them: one value per grid cell (`levz`), read from the
//! marker clips placed in each frame of a chapter's room sprite. The asset pipeline supplies
//! them; this module turns values into grid objects.

use glam::Vec2;

use crate::game::parameters::BreakableParameters;
use crate::game::room::grid_kind::{BreakableKind, GridObjectKind, TileKind};
use crate::game::room::grid_object::GridObject;
use crate::game::room::{COLUMN_COUNT, ROW_COUNT};

/// One marker: a cell of the 13×7 interior and the value its clip reported (`spawnb`). Negative
/// whole values are enemy types, kept for when enemies exist.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LayoutCell {
    pub column: u32,
    pub row: u32,
    pub value: f32,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RoomLayout {
    pub cells: Vec<LayoutCell>,
    /// Figures standing in the room (`inl.shop`), each at a stage point.
    pub decorations: Vec<(Decoration, Vec2)>,
}

/// Clips a layout places to be looked at, not markers for the grid or pickups.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Decoration {
    /// The devil room's statue (3729 frame 1; frame 2 is the angel's), on a rock that blocks
    /// its cell.
    DevilStatue,
    /// The shopkeeper of `shop1` and `shop5` (6711), one of 7 looks.
    Shopkeeper,
    /// The secret room `hide0`'s shopkeeper (6718), one of 6 looks.
    SecretShopkeeper,
}

impl Decoration {
    /// How many looks the clip picks from at random.
    pub fn look_count(self) -> u8 {
        match self {
            Self::DevilStatue => 1,
            Self::Shopkeeper => 7,
            Self::SecretShopkeeper => 6,
        }
    }
}

/// Every layout a floor draws from; empty lists leave rooms empty.
#[derive(Clone, Debug, Default)]
pub struct RoomLayouts {
    /// The chapter's room sprite, one layout per frame (frame 1 first).
    pub normal: Vec<RoomLayout>,
    /// Treasure rooms `t0`–`t5`, one at random.
    pub treasure: Vec<RoomLayout>,
    /// Secret rooms `hide0`–`hide4`, one at random.
    pub secret: Vec<RoomLayout>,
    /// Shops `shop1`–`shop6`, one at random.
    pub shop: Vec<RoomLayout>,
    /// The chapter's boss rooms, `b1-0`–`b1-4` in the Basement, one at random.
    pub boss: Vec<RoomLayout>,
    /// The library (`lib`).
    pub library: Vec<RoomLayout>,
    /// The sacrifice room (`sac`).
    pub sacrifice: Vec<RoomLayout>,
    /// Curse rooms `c`, `c0`, `c1`, `c2`.
    pub curse: Vec<RoomLayout>,
    /// Super secret rooms `hi0`–`hi4`, one per style.
    pub super_secret: Vec<RoomLayout>,
    /// Mini-boss rooms `min0`–`min6`, one of the chapter's at random.
    pub miniboss: Vec<RoomLayout>,
    /// The devil room (`satan`).
    pub devil: Vec<RoomLayout>,
}

/// Grid values (`levz`) that `golev()` draws as tiles and `turd()` turns into objects.
const VALUE_KINDS: [(f32, GridObjectKind); 12] = [
    (1.0, GridObjectKind::Tile(TileKind::Rock)),
    (1.9, GridObjectKind::Tile(TileKind::Block)),
    (3.0, GridObjectKind::Tile(TileKind::Pit)),
    (1.93, GridObjectKind::Tile(TileKind::Spikes)),
    (1.94, GridObjectKind::Tile(TileKind::SpiderWeb)),
    (4.0, GridObjectKind::KeyBlock),
    (1.5, GridObjectKind::Breakable(BreakableKind::Poop)),
    (1.49, GridObjectKind::Breakable(BreakableKind::RedPoop)),
    (1.495, GridObjectKind::Breakable(BreakableKind::CornyPoop)),
    (1.4955, GridObjectKind::Breakable(BreakableKind::GoldenPoop)),
    (1.4, GridObjectKind::Breakable(BreakableKind::Fireplace)),
    (1.41, GridObjectKind::Breakable(BreakableKind::RedFireplace)),
];
/// TNT is 1.3; grid values are compared with a tolerance for float noise.
const TNT_VALUE: f32 = 1.3;
const VALUE_TOLERANCE: f32 = 0.0001;

pub fn kind_for_value(value: f32) -> Option<GridObjectKind> {
    if (value - TNT_VALUE).abs() < VALUE_TOLERANCE {
        return Some(GridObjectKind::Breakable(BreakableKind::TntBarrel));
    }
    VALUE_KINDS
        .iter()
        .find(|(kind_value, _)| (value - kind_value).abs() < VALUE_TOLERANCE)
        .map(|&(_, kind)| kind)
}

/// Enemy types start at 10, which 9 minus the value makes markers at -1 and below.
const ENEMY_MARKER_LIMIT: f32 = -1.0;

impl RoomLayout {
    /// Whether the room has enemy markers (enemies themselves do not exist yet).
    pub fn has_enemies(&self) -> bool {
        self.cells.iter().any(|cell| cell.value <= ENEMY_MARKER_LIMIT)
    }

    /// The grid objects this layout places; cells the game does not model yet are skipped. A rock
    /// on `tinted_rock` (column, row) becomes the room's tinted rock.
    pub fn grid_objects(&self, parameters: BreakableParameters, tinted_rock: Option<(u32, u32)>) -> Vec<GridObject> {
        self.cells
            .iter()
            .filter(|cell| cell.column < COLUMN_COUNT && cell.row < ROW_COUNT)
            .filter_map(|cell| {
                let kind = match kind_for_value(cell.value)? {
                    GridObjectKind::Tile(TileKind::Rock) if tinted_rock == Some((cell.column, cell.row)) => {
                        GridObjectKind::Tile(TileKind::TintedRock)
                    }
                    kind => kind,
                };
                Some(GridObject::on_tile(kind, cell.column, cell.row, parameters))
            })
            .collect()
    }
}

/// One of every grid object, kept clear of the center column and the doorways (used by tests and
/// handy for checking art): (value, column, row).
const SHOWCASE_CELLS: [(f32, u32, u32); 16] = [
    (1.0, 1, 1),
    (1.9, 3, 1),
    (3.0, 5, 1),
    (1.93, 7, 1),
    (1.94, 9, 1),
    (4.0, 11, 1),
    (1.3, 11, 3),
    (3.0, 1, 3),
    (1.0, 2, 3),
    (1.5, 1, 5),
    (1.49, 3, 5),
    (1.495, 5, 5),
    (1.4955, 7, 5),
    (1.4, 9, 5),
    (1.41, 11, 5),
    (1.0, 8, 3),
];

#[cfg_attr(not(test), expect(dead_code, reason = "used by tests"))]
pub fn showcase_layout() -> RoomLayout {
    RoomLayout {
        cells: SHOWCASE_CELLS
            .iter()
            .map(|&(value, column, row)| LayoutCell { column, row, value })
            .collect(),
        decorations: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_values_map_to_objects_and_enemies_are_skipped() {
        assert_eq!(kind_for_value(1.0), Some(GridObjectKind::Tile(TileKind::Rock)));
        assert_eq!(
            kind_for_value(1.3),
            Some(GridObjectKind::Breakable(BreakableKind::TntBarrel))
        );
        assert_eq!(kind_for_value(-12.0), None);
        let layout = RoomLayout {
            cells: vec![
                LayoutCell {
                    column: 0,
                    row: 0,
                    value: 1.5,
                },
                LayoutCell {
                    column: 3,
                    row: 2,
                    value: -12.0,
                },
                LayoutCell {
                    column: 20,
                    row: 2,
                    value: 1.0,
                },
            ],
            decorations: Vec::new(),
        };
        assert_eq!(
            layout
                .grid_objects(crate::game::GameParameters::ORIGINAL.breakables, None)
                .len(),
            1
        );
    }
}

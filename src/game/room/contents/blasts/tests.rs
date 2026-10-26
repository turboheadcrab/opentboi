use glam::Vec2;

use crate::game::parameters::GameParameters;
use crate::game::random::Random;
use crate::game::room::contents::RoomContents;
use crate::game::room::grid_kind::{BreakableKind, GridObjectKind, TileKind};
use crate::game::room::grid_object::GridObject;
use crate::game::room::layout::showcase_layout;
use crate::game::room::tile_center;

const POOP: GridObjectKind = GridObjectKind::Breakable(BreakableKind::Poop);

fn room_with(objects: &[(GridObjectKind, u32, u32)]) -> RoomContents {
    let parameters = GameParameters::ORIGINAL;
    RoomContents {
        grid_objects: objects
            .iter()
            .map(|&(kind, column, row)| GridObject::on_tile(kind, column, row, parameters.breakables))
            .collect(),
        flies: Vec::new(),
        ..RoomContents::from_layout(&showcase_layout(), &parameters, None, &mut Random::with_seed(1))
    }
}

/// Whether a bomb two tiles left of a poop breaks it, with `between` on the tile in the middle.
fn poop_survives_with(between: Option<GridObjectKind>) -> bool {
    let mut objects = vec![(POOP, 6, 3)];
    objects.extend(between.map(|kind| (kind, 5, 3)));
    let mut room = room_with(&objects);
    room.explode_at(tile_center(4, 3), &mut Random::with_seed(1), &mut Vec::new());
    room.grid_objects[0].is_intact_breakable()
}

#[test]
fn a_block_between_shields_from_a_blast_but_a_rock_does_not() {
    assert!(!poop_survives_with(None));
    assert!(poop_survives_with(Some(GridObjectKind::Tile(TileKind::Block))));
    assert!(!poop_survives_with(Some(GridObjectKind::Tile(TileKind::Rock))));
}

#[test]
fn the_blast_grid_holds_the_rooms_objects() {
    let room = room_with(&[(GridObjectKind::Tile(TileKind::Block), 5, 3)]);
    let sight = room.sight_grid();
    assert!(!sight.blast_reaches(tile_center(4, 3), tile_center(6, 3)));
    assert!(sight.blast_reaches(tile_center(4, 3), tile_center(4, 5)));
    assert!(sight.blast_reaches(Vec2::ZERO, tile_center(6, 1)));
}

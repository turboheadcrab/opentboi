use super::test_support::{HORF_MARKER, room_with_marker, run, walker_world_position};
use crate::game::original_units::{Damage, distance};
use crate::game::parameters::GameParameters;
use crate::game::room::grid_kind::{BreakableKind, GridObjectKind, TileKind};
use crate::game::room::grid_object::GridObject;
use crate::game::room::tile_center;

const BLACK_FLY_MARKER: f32 = -4.0;

fn object(kind: GridObjectKind, column: u32, row: u32) -> GridObject {
    GridObject::on_tile(kind, column, row, GameParameters::ORIGINAL.breakables)
}

#[test]
fn a_burning_fireplace_burns_a_walker_for_eight_and_pushes_it_away() {
    let fireplace = object(GridObjectKind::Breakable(BreakableKind::Fireplace), 5, 3);
    let player = tile_center(12, 6);
    let mut enemies = room_with_marker(HORF_MARKER, 5, 3);
    run(&mut enemies, 1, player, &[]);
    let before = enemies.walkers[0].hit_points();
    let start = walker_world_position(&enemies);
    run(&mut enemies, 12, player, &[fireplace]);
    assert_eq!(enemies.walkers[0].hit_points(), before - Damage(8.0));
    assert!(distance(walker_world_position(&enemies), tile_center(5, 3)) > distance(start, tile_center(5, 3)));
}

#[test]
fn spikes_hurt_a_walker_every_ten_frames_but_not_a_fly() {
    let spikes = object(GridObjectKind::Tile(TileKind::Spikes), 5, 3);
    let player = tile_center(12, 6);
    let mut walker = room_with_marker(HORF_MARKER, 5, 3);
    run(&mut walker, 1, player, &[]);
    run(&mut walker, 25, player, std::slice::from_ref(&spikes));
    assert!(!walker.any_alive(), "a hit at once and one ten frames on");
    let mut fly = room_with_marker(BLACK_FLY_MARKER, 5, 3);
    run(&mut fly, 1, player, &[]);
    run(&mut fly, 80, player, &[spikes]);
    assert!(fly.any_alive());
}

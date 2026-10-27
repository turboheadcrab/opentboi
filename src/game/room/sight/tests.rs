use super::*;
use crate::game::parameters::GameParameters;
use crate::game::room::grid_kind::{GridObjectKind, TileKind};
use crate::game::room::tile_center;

fn object(kind: GridObjectKind, column: u32, row: u32) -> GridObject {
    GridObject::on_tile(kind, column, row, GameParameters::ORIGINAL.breakables)
}

fn tile(kind: TileKind, column: u32, row: u32) -> GridObject {
    object(GridObjectKind::Tile(kind), column, row)
}

/// The center of floor tile `column`, `row` in stage coordinates.
fn spot(column: u32, row: u32) -> Vec2 {
    world_to_stage(tile_center(column, row))
}

#[test]
fn a_rock_ends_a_walkers_sight_but_spikes_and_an_open_room_do_not() {
    let (from, target) = (spot(2, 3), spot(9, 3));
    assert!(SightGrid::new(&[]).walk_line_clear(from, target));
    assert!(!SightGrid::new(&[tile(TileKind::Rock, 5, 3)]).walk_line_clear(from, target));
    let spikes: Vec<GridObject> = (3..9).map(|column| tile(TileKind::Spikes, column, 3)).collect();
    assert!(
        SightGrid::new(&spikes).walk_line_clear(from, target),
        "0.99 costs nothing"
    );
}

#[test]
fn marked_cells_cost_a_tenth_each_until_the_sight_is_spent() {
    let mut grid = SightGrid::new(&[]);
    for column in 0..13 {
        grid.mark_walker(spot(column, 3));
    }
    // Each step's four corners lie in marked cells: 0.4 a step, so six steps leave 0.1 of the 2.5.
    let from = spot(4, 3);
    assert!(grid.walk_line_clear(from, from + Vec2::new(36.0, 0.0)));
    assert!(!grid.walk_line_clear(from, from + Vec2::new(42.0, 0.0)));
}

#[test]
fn marks_fade_every_third_frame_and_cells_of_099_are_not_marked() {
    let mut grid = SightGrid::new(&[tile(TileKind::SpiderWeb, 5, 3)]);
    grid.mark_walker(spot(4, 3));
    grid.mark_walker(spot(5, 3));
    let marked = |grid: &SightGrid, column, value: f32| {
        (grid.marks.0[index(ingrid(spot(column, 3), false))] - value).abs() < f32::EPSILON
    };
    assert!(marked(&grid, 4, WALKER_MARK));
    assert!(marked(&grid, 5, 0.0), "the web's 0.99 stays");
    grid.fade_marks(Frames::from_number(4));
    assert!(marked(&grid, 4, WALKER_MARK));
    for number in 1..=27 {
        grid.fade_marks(Frames::from_number(number));
    }
    assert!(marked(&grid, 4, 0.0));
}

#[test]
fn blasts_pass_rocks_and_pits_but_not_blocks_or_closed_key_blocks() {
    let (from, target) = (spot(2, 3), spot(9, 3));
    let reaches = |objects: &[GridObject]| SightGrid::new(objects).blast_line_clear(from, target);
    assert!(reaches(&[tile(TileKind::Rock, 5, 3)]));
    assert!(reaches(&[tile(TileKind::Pit, 5, 3)]));
    assert!(!reaches(&[tile(TileKind::Block, 5, 3)]));
    assert!(!reaches(&[object(GridObjectKind::KeyBlock, 5, 3)]));
}

#[test]
fn a_blast_reaches_a_hidden_wall_but_not_a_door() {
    let mut grid = SightGrid::new(&[]);
    let (column, row) = DoorSide::Top.doorway_cell();
    let wall = world_to_stage(crate::game::room::cell_bounds(column, row).center());
    let bomb = spot(6, 1);
    assert!(!grid.blast_line_clear(bomb, wall), "a door's doorway holds 2");
    grid.set_doorway(DoorSide::Top, HIDDEN_WALL_VALUE);
    assert!(grid.blast_line_clear(bomb, wall));
}

#[test]
fn the_last_step_can_land_past_the_end() {
    let grid = SightGrid::new(&[tile(TileKind::Block, 5, 3)]);
    // The block's cell starts 20 px left of its center; the line ends 3 px short of it. Over 70 px
    // the seventh step lands on the end; over 75 px the eighth lands 5 px past it, in the block.
    let target = spot(5, 3) - Vec2::new(23.0, 0.0);
    assert!(grid.blast_line_clear(target - Vec2::new(70.0, 0.0), target));
    assert!(!grid.blast_line_clear(target - Vec2::new(75.0, 0.0), target));
}

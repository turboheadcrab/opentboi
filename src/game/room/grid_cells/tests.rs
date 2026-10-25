use super::*;

#[test]
fn places_round_to_the_nearest_cell_and_stay_in_the_grid() {
    assert_eq!(ingrid(Vec2::new(100.0, 190.0), false), GridCell { column: 3, row: 5 });
    // Halves round up, as Flash's rounding does.
    assert_eq!(ingrid(Vec2::new(100.0, 200.0), false).row, 5);
    assert_eq!(
        ingrid(Vec2::new(-500.0, 9999.0), false),
        GridCell { column: 1, row: 11 }
    );
    assert_eq!(
        ingrid(Vec2::new(9999.0, -500.0), false),
        GridCell { column: 15, row: 3 }
    );
    // Restricted, the walls are left out.
    assert_eq!(ingrid(Vec2::new(0.0, 0.0), true), GridCell { column: 2, row: 4 });
    assert_eq!(ingrid(Vec2::new(900.0, 900.0), true), GridCell { column: 14, row: 10 });
}

#[test]
fn a_cells_place_is_a_place_in_the_cell() {
    for row in 0..ROW_COUNT {
        for column in 0..COLUMN_COUNT {
            let cell = GridCell::of_tile(column, row);
            assert_eq!(ingrid(outgrid(cell), true), cell);
            assert_eq!(cell.tile(), Some((column, row)));
        }
    }
    assert_eq!(GridCell::of_tile(0, 0).index(), 2 + 16 * 4);
    assert_eq!(GridCell { column: 1, row: 7 }.tile(), None, "the left wall");
}

#[test]
fn enemies_see_doorways_as_solid_and_the_rest_see_the_grid() {
    let doorway = GridCell { column: 8, row: 3 };
    let doorways = [doorway];
    assert_eq!(levzz(Mover::Enemy, doorway, &doorways, 0.0), 2.0);
    assert_eq!(levzz(Mover::Other, doorway, &doorways, 0.0), 0.0, "a tear");
    assert_eq!(levzz(Mover::Enemy, GridCell { column: 9, row: 3 }, &doorways, 0.0), 0.0);
    let corner = GridCell { column: 0, row: 0 };
    assert_eq!(
        levzz(Mover::Enemy, corner, &[corner], 1.0),
        1.0,
        "cell 0 is not a doorway"
    );
}

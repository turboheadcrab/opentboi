use super::*;
use crate::game::GameParameters;
use crate::game::original_units::distance;
use crate::game::room::enemies::EnemyParameters;
use crate::game::room::layout::{LayoutCell, RoomLayout};
use crate::game::room::{interior_bounds, tile_center};

const ONE_FRAME: Frames = Frames(1.0);
const GAPER_MARKER: f32 = -1.0;

fn flies() -> (BlueFlyParameters, BlueFlies) {
    (GameParameters::ORIGINAL.blue_flies, BlueFlies::default())
}

fn room_with_a_gaper(column: u32, row: u32) -> Enemies {
    let layout = RoomLayout {
        cells: vec![LayoutCell {
            column,
            row,
            value: GAPER_MARKER,
        }],
        ..RoomLayout::default()
    };
    Enemies::from_layout(
        &layout,
        &EnemyParameters::from_game(&GameParameters::ORIGINAL),
        &mut Random::with_seed(1),
    )
}

fn run(
    blue_flies: &mut BlueFlies,
    parameters: &BlueFlyParameters,
    enemies: &mut Enemies,
    player: Vec2,
    frames: u32,
    random: &mut Random,
) {
    for _ in 0..frames {
        let mut world = Surroundings {
            player,
            player_busy: false,
            damage: Damage(3.5),
            interior: interior_bounds(),
            enemies,
        };
        blue_flies.advance(ONE_FRAME, parameters, &mut world, random, &mut Vec::new());
    }
}

#[test]
fn owed_flies_appear_one_a_frame_by_the_player() {
    let (parameters, mut blue_flies) = flies();
    let mut enemies = room_with_a_gaper(12, 1);
    blue_flies.owe(3);
    run(
        &mut blue_flies,
        &parameters,
        &mut enemies,
        Vec2::ZERO,
        2,
        &mut Random::with_seed(1),
    );
    assert_eq!(blue_flies.count(), 2);
    run(
        &mut blue_flies,
        &parameters,
        &mut enemies,
        Vec2::ZERO,
        5,
        &mut Random::with_seed(1),
    );
    assert_eq!(blue_flies.count(), 3);
}

#[test]
fn flies_circle_the_player_on_a_ring() {
    let (parameters, mut blue_flies) = flies();
    let mut enemies = room_with_a_gaper(12, 1);
    blue_flies.owe(3);
    let player = Vec2::new(-100.0, -20.0);
    run(
        &mut blue_flies,
        &parameters,
        &mut enemies,
        player,
        200,
        &mut Random::with_seed(1),
    );
    assert_eq!(blue_flies.count(), 3);
    for fly in &blue_flies.flies {
        let reach = distance(fly.position, player);
        assert!(reach < Pixels(60.0) && reach > Pixels(10.0), "{reach:?}");
    }
}

#[test]
fn a_fly_hurts_what_it_touches_once_in_a_while_and_may_die_of_it() {
    let (parameters, _) = flies();
    let gaper = tile_center(6, 3);
    // 7.2 a hit against ten hit points: two hits kill it, whichever way the coin falls.
    let mut deaths = 0;
    for seed in 1..40 {
        let mut enemies = room_with_a_gaper(6, 3);
        for _ in 0..60 {
            enemies.advance(
                ONE_FRAME,
                &[],
                Vec2::new(250.0, 120.0),
                interior_bounds(),
                &mut Random::with_seed(1),
                &mut Vec::new(),
            );
        }
        let mut blue_flies = BlueFlies::default();
        blue_flies.owe(6);
        run(
            &mut blue_flies,
            &parameters,
            &mut enemies,
            gaper,
            200,
            &mut Random::with_seed(seed),
        );
        deaths += usize::from(!enemies.any_alive());
    }
    assert!(deaths > 30, "six flies kill a Gaper: {deaths} of 39");
}

#[test]
fn up_to_ten_flies_follow_into_the_next_room() {
    let (parameters, mut blue_flies) = flies();
    let mut enemies = room_with_a_gaper(12, 1);
    blue_flies.owe(12);
    run(
        &mut blue_flies,
        &parameters,
        &mut enemies,
        Vec2::ZERO,
        20,
        &mut Random::with_seed(1),
    );
    assert_eq!(blue_flies.count(), 12);
    blue_flies.leave_room(&parameters);
    assert_eq!((blue_flies.count(), blue_flies.owed), (0, 10));
}

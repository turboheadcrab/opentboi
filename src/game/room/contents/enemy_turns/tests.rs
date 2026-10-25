use glam::Vec2;

use crate::game::original_units::Frames;
use crate::game::parameters::GameParameters;
use crate::game::random::Random;
use crate::game::room::contents::RoomContents;
use crate::game::room::enemies::{Enemies, EnemyParameters};
use crate::game::room::grid_kind::{BreakableKind, GridObjectKind};
use crate::game::room::grid_object::GridObject;
use crate::game::room::layout::{LayoutCell, RoomLayout, showcase_layout};

const ONE_FRAME: Frames = Frames(1.0);
/// The black fly's marker value.
const FLY_MARKER: f32 = -4.0;
/// A peck lands one frame in 2000 or so; this many frames make one all but certain.
const PATIENCE_FRAMES: u32 = 40_000;

/// A room with one poop and a black fly beside it.
fn room_with_a_fly_by_a_poop() -> RoomContents {
    let parameters = GameParameters::ORIGINAL;
    let poop = GridObject::on_tile(
        GridObjectKind::Breakable(BreakableKind::Poop),
        6,
        3,
        parameters.breakables,
    );
    let fly_layout = RoomLayout {
        cells: vec![LayoutCell {
            column: 7,
            row: 3,
            value: FLY_MARKER,
        }],
        ..RoomLayout::default()
    };
    RoomContents {
        grid_objects: vec![poop],
        flies: Vec::new(),
        enemies: Enemies::from_layout(
            &fly_layout,
            &EnemyParameters::from_game(&parameters),
            &mut Random::with_seed(1),
        ),
        ..RoomContents::from_layout(&showcase_layout(), &parameters, None, &mut Random::with_seed(1))
    }
}

#[test]
fn a_black_fly_pecking_its_poop_turns_into_a_pooter() {
    let mut room = room_with_a_fly_by_a_poop();
    let mut random = Random::with_seed(5);
    let far_away = Vec2::new(-250.0, -150.0);
    let frames_until_pooter = (0..PATIENCE_FRAMES).position(|_| {
        room.advance(ONE_FRAME, far_away, &mut random, &mut Vec::new());
        room.enemies.buzzing_count() == 1 && room.enemies.has_pooter()
    });
    assert!(frames_until_pooter.is_some(), "it turned into a Pooter");
}

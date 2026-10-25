use glam::Vec2;

use super::*;
use crate::game::parameters::GameParameters;
use crate::game::room::layout::showcase_layout;

const KEEP_CLEAR: Pixels = Pixels(150.0);
const TRIES: u32 = 40;

fn room() -> RoomContents {
    RoomContents::from_layout(
        &showcase_layout(),
        &GameParameters::ORIGINAL,
        None,
        &mut Random::with_seed(1),
    )
}

#[test]
fn a_spot_keeps_out_of_the_circles_it_must_avoid() {
    let room = room();
    let clearance = Clearance {
        center: Vec2::ZERO,
        distance: KEEP_CLEAR,
    };
    for seed in 0..TRIES {
        let spot = room.spawn_spot_clear_of(Vec2::ZERO, Pixels::ZERO, &[clearance], &mut Random::with_seed(seed));
        // The spot is a tile's center and the try inside the circle was thrown away, so the
        // tile's center can only be closer by a tile's half diagonal.
        assert!(spot.length() > KEEP_CLEAR.get() - 40.0, "seed {seed}: {spot}");
    }
}

#[test]
fn without_circles_a_spot_can_be_right_at_the_target() {
    let room = room();
    let close = (0..TRIES)
        .map(|seed| room.spawn_spot(Vec2::ZERO, Pixels::ZERO, &mut Random::with_seed(seed)))
        .any(|spot| spot.length() < KEEP_CLEAR.get());
    assert!(close);
}

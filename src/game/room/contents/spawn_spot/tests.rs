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

#[test]
/// DoAction 29095 and 29037: the tile `spaw()` gives away is marked, so the next one asked for at
/// the same place goes to another tile until the mark has faded.
fn a_tile_given_away_is_not_given_away_again_at_once() {
    let mut room = room();
    let mut random = Random::with_seed(2);
    let spots: Vec<Vec2> = (0..4)
        .map(|_| room.spaw(Vec2::ZERO, Pixels::ZERO, SpawOptions::default(), &mut random))
        .collect();
    for (index, spot) in spots.iter().enumerate() {
        assert!(!spots[index + 1..].contains(spot), "{spots:?}");
    }
}

#[test]
/// DoAction 29098 and 29099: what hatches lands up to nine pixels off its tile's center.
fn a_scattered_spawn_lands_off_the_center_of_its_tile() {
    let mut room = room();
    let options = SpawOptions {
        scatter: true,
        ..SpawOptions::default()
    };
    let tile = room.spawn_spot(Vec2::ZERO, Pixels::ZERO, &mut Random::with_seed(4));
    let spot = room.spaw(Vec2::ZERO, Pixels::ZERO, options, &mut Random::with_seed(4));
    assert!(spot.distance(tile) < 9.5, "{spot} against {tile}");
}

#[test]
/// DoAction 29047–29057: an enemy in the challenge room keeps 150 px from Isaac.
fn an_enemy_in_the_challenge_room_keeps_away_from_isaac() {
    let options = SpawOptions {
        enemy_clear_of_player: Some(Vec2::ZERO),
        ..SpawOptions::default()
    };
    for seed in 0..TRIES {
        let mut room = room();
        let spot = room.spaw(Vec2::ZERO, Pixels::ZERO, options, &mut Random::with_seed(seed));
        assert!(spot.length() > KEEP_CLEAR.get() - 40.0, "seed {seed}: {spot}");
    }
}

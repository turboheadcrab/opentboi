use glam::Vec2;

use super::*;
use crate::game::floor::DoorSide;
use crate::game::parameters::GameParameters;
use crate::game::random::Random;
use crate::game::room::layout::RoomLayouts;

#[test]
fn the_mini_boss_icon_shows_once_the_room_is_entered() {
    let found = (1..200).find_map(|seed| {
        let floor = Floor::generate(
            &RoomLayouts::default(),
            &GameParameters::ORIGINAL,
            &mut Random::with_seed(seed),
        );
        let mini_boss = floor.cell_with(RoomRole::MiniBoss)?;
        let next_door = DoorSide::ALL
            .into_iter()
            .filter_map(|side| mini_boss.through(side))
            .find(|&cell| floor.plan.roles_by_cell.get(&cell) == Some(&RoomRole::Normal))?;
        Some((floor, mini_boss, next_door))
    });
    let (mut floor, mini_boss, next_door) = found.expect("a floor with a mini-boss room");
    assert!(!floor.map_pieces().contains(&(mini_boss, MapPiece::MiniBossIcon)));
    floor.teleport_to(next_door, Vec2::ZERO);
    assert!(
        !floor.map_pieces().contains(&(mini_boss, MapPiece::MiniBossIcon)),
        "a door into it does not give it away (only item 91 would)"
    );
    floor.teleport_to(mini_boss, Vec2::ZERO);
    floor.teleport_to(next_door, Vec2::ZERO);
    assert!(floor.map_pieces().contains(&(mini_boss, MapPiece::MiniBossIcon)));
}

#[test]
fn a_secret_room_shows_every_room_around_it() {
    let found = (1..200).find_map(|seed| {
        let floor = Floor::generate(
            &RoomLayouts::default(),
            &GameParameters::ORIGINAL,
            &mut Random::with_seed(seed),
        );
        let secret = floor.cell_with(RoomRole::Secret)?;
        let around: Vec<FloorCell> = DoorSide::ALL
            .into_iter()
            .filter_map(|side| secret.through(side))
            .filter(|&cell| floor.plan.has_room(cell))
            .collect();
        (around.len() > 1).then_some((floor, secret, around))
    });
    let (mut floor, secret, around) = found.expect("a floor whose secret room has several neighbors");
    floor.teleport_to(secret, Vec2::ZERO);
    let shown: Vec<FloorCell> = floor.map_pieces().into_iter().map(|(cell, _)| cell).collect();
    for cell in around {
        assert!(shown.contains(&cell), "{cell:?} is next to the secret room");
    }
}

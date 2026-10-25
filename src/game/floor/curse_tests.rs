//! The curse room's doors next to a secret room.

use super::*;
use crate::game::original_units::Pixels;
use crate::game::sound_event::SoundEvent;

#[test]
fn a_secret_room_next_to_the_curse_room_has_a_curse_door_or_a_wall_to_bomb() {
    let found = (1..2000).find_map(|seed| {
        let floor = Floor::generate(
            &RoomLayouts::default(),
            &GameParameters::ORIGINAL,
            &mut Random::with_seed(seed),
        );
        let roles = &floor.plan.roles_by_cell;
        let (&curse, _) = roles.iter().find(|(_, role)| **role == RoomRole::Curse)?;
        let side = DoorSide::ALL.into_iter().find(|side| {
            curse
                .through(*side)
                .is_some_and(|cell| roles.get(&cell) == Some(&RoomRole::Secret))
        })?;
        Some((floor, curse, side))
    });
    let (mut floor, curse, side) = found.expect("some floor has its secret room next to the curse room");
    floor.current_cell = curse;
    assert!(floor.visible_doors().contains(&side), "original: a curse door");
    assert_eq!(
        floor.curse_doorways().len(),
        floor.visible_doors().len(),
        "every door is spiked"
    );
    floor.set_door_rules(Style::Original, Style::Rebirth);
    assert!(!floor.visible_doors().contains(&side), "Rebirth: a wall");
    floor.blast_walls(side.position() * 0.8, Pixels(60.0));
    assert!(floor.visible_doors().contains(&side));
    assert_eq!(
        floor.curse_doorways().len(),
        floor.visible_doors().len() - 1,
        "the hole does not hurt"
    );
}

#[test]
fn bombing_out_of_a_secret_room_entered_through_a_curse_door_stays_quiet() {
    let found = (1..2000).find_map(|seed| {
        let floor = Floor::generate(
            &RoomLayouts::default(),
            &GameParameters::ORIGINAL,
            &mut Random::with_seed(seed),
        );
        let roles = &floor.plan.roles_by_cell;
        let (&curse, _) = roles.iter().find(|(_, role)| **role == RoomRole::Curse)?;
        let side = DoorSide::ALL.into_iter().find(|side| {
            curse
                .through(*side)
                .is_some_and(|cell| roles.get(&cell) == Some(&RoomRole::Secret))
        })?;
        Some((floor, curse, side))
    });
    let (mut floor, curse, side) = found.expect("some floor has its secret room next to the curse room");
    floor.current_cell = curse;
    let entry = floor.go_through(side);
    assert_eq!(entry.jingle, Some(SoundEvent::SecretRoomFound), "found on entering");
    for wall in DoorSide::ALL {
        assert_eq!(floor.blast_walls(wall.position() * 0.8, Pixels(60.0)), None);
    }
}

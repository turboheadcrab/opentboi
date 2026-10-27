use super::*;
use crate::game::original_units::Pixels;
use crate::game::sound_event::SoundEvent;
use crate::game::texture_id::TextureId;

#[test]
fn walking_into_a_door_enters_the_neighbor_by_its_opposite_door() {
    let mut floor = Floor::generate(
        &RoomLayouts::default(),
        &GameParameters::ORIGINAL,
        &mut Random::with_seed(7),
    );
    let side = floor.doors().next().expect("the start room has a neighbor");
    let destination = FloorCell::START.through(side);
    assert_eq!(floor.door_reached(Vec2::ZERO), None);
    assert_eq!(floor.door_reached(side.trigger_point()), Some(side));
    let entry = floor.go_through(side).position;
    assert_eq!(Some(floor.current_cell), destination);
    assert_eq!(entry, side.entry_position());
    assert!(
        floor.doors().any(|door| door == side.opposite()),
        "the way back is open"
    );
    assert_eq!(floor.door_reached(entry), None, "no bouncing straight back");
}

#[test]
fn a_blast_by_a_hidden_wall_opens_the_secret_room_both_ways() {
    for seed in 1..50 {
        let mut floor = Floor::generate(
            &RoomLayouts::default(),
            &GameParameters::ORIGINAL,
            &mut Random::with_seed(seed),
        );
        let secret = floor
            .plan
            .roles_by_cell
            .iter()
            .find(|(_, role)| **role == RoomRole::Secret)
            .map(|(cell, _)| *cell);
        let Some(secret) = secret else { continue };
        let Some(side) = DoorSide::ALL.into_iter().find(|side| {
            secret
                .through(side.opposite())
                .is_some_and(|cell| floor.plan.roles_by_cell.get(&cell) == Some(&RoomRole::Normal))
        }) else {
            continue;
        };
        floor.current_cell = secret.through(side.opposite()).expect("found above");
        assert!(!floor.visible_doors().contains(&side), "hidden before the blast");
        assert!(
            floor
                .blast_walls(Vec2::ZERO, Pixels(60.0), &floor.blast_sight())
                .is_none(),
            "a blast in the middle reaches no wall"
        );
        assert_eq!(
            floor.blast_walls(side.position() * 0.8, Pixels(60.0), &floor.blast_sight()),
            Some(SoundEvent::SecretRoomFound)
        );
        assert!(floor.visible_doors().contains(&side));
        floor.go_through(side);
        assert_eq!(floor.current_cell, secret);
        assert!(floor.visible_doors().contains(&side.opposite()), "the way back is open");
        floor.go_through(side.opposite());
        assert!(
            !floor.visible_doors().contains(&side),
            "the original closes the wall behind you"
        );
        floor.set_door_rules(Style::Rebirth, Style::Original);
        floor.blast_walls(side.position() * 0.8, Pixels(60.0), &floor.blast_sight());
        floor.go_through(side);
        floor.go_through(side.opposite());
        assert!(floor.visible_doors().contains(&side), "Rebirth keeps it open");
        return;
    }
    panic!("no floor with a secret room next to a normal room");
}

#[test]
fn walking_into_the_shop_door_with_a_key_unlocks_it() {
    for seed in 1..50 {
        let mut floor = Floor::generate(
            &RoomLayouts::default(),
            &GameParameters::ORIGINAL,
            &mut Random::with_seed(seed),
        );
        let shop = floor
            .plan
            .roles_by_cell
            .iter()
            .find(|(_, role)| **role == RoomRole::Shop)
            .map(|(cell, _)| *cell);
        let Some(shop) = shop else { continue };
        let Some(side) = DoorSide::ALL.into_iter().find(|side| {
            shop.through(side.opposite())
                .is_some_and(|cell| floor.plan.roles_by_cell.get(&cell) == Some(&RoomRole::Normal))
        }) else {
            continue;
        };
        floor.current_cell = shop.through(side.opposite()).expect("found above");
        assert!(floor.visible_doors().contains(&side), "the shop door is shown");
        assert_eq!(floor.door_reached(side.trigger_point()), None, "but locked");
        let toward_door = side.position().normalize() * 6.0;
        assert!(!floor.unlock_doors(Vec2::ZERO, toward_door), "too far away");
        assert!(floor.unlock_doors(side.position() * 0.85, toward_door));
        assert_eq!(floor.door_reached(side.trigger_point()), Some(side));
        return;
    }
    panic!("no floor with a shop next to a normal room");
}

#[test]
fn the_minimap_shows_the_current_room_and_the_rooms_behind_its_doors() {
    let mut floor = Floor::generate(
        &RoomLayouts::default(),
        &GameParameters::ORIGINAL,
        &mut Random::with_seed(7),
    );
    let pieces = floor.map_pieces();
    assert!(pieces.contains(&(FloorCell::START, MapPiece::Current)));
    let side = floor.doors().next().expect("the start room has a neighbor");
    let neighbor = FloorCell::START.through(side).expect("a door leads somewhere");
    assert!(
        pieces
            .iter()
            .any(|&(cell, piece)| cell == neighbor && piece != MapPiece::Current)
    );
    floor.go_through(side);
    let pieces = floor.map_pieces();
    assert!(pieces.contains(&(FloorCell::START, MapPiece::Visited)));
    assert!(pieces.contains(&(neighbor, MapPiece::Current)));
}

#[test]
fn a_key_opens_the_shop_door_only_when_walking_into_it() {
    let seed_and_side = (1..100).find_map(|seed| {
        let floor = Floor::generate(
            &RoomLayouts::default(),
            &GameParameters::ORIGINAL,
            &mut Random::with_seed(seed),
        );
        let (&shop, _) = floor
            .plan
            .roles_by_cell
            .iter()
            .find(|(_, role)| **role == RoomRole::Shop)?;
        let side = DoorSide::ALL.into_iter().find(|side| {
            shop.through(side.opposite())
                .is_some_and(|cell| floor.plan.roles_by_cell.get(&cell) == Some(&RoomRole::Normal))
        })?;
        Some((seed, side, shop.through(side.opposite())?))
    });
    let (seed, side, next_to_shop) = seed_and_side.expect("some floor has a shop");
    let mut floor = Floor::generate(
        &RoomLayouts::default(),
        &GameParameters::ORIGINAL,
        &mut Random::with_seed(seed),
    );
    floor.current_cell = next_to_shop;
    let near_door = side.position() - side.outward() * 30.0;
    let sideways = side.outward().perp() * 2.0;
    assert!(!floor.unlock_doors(near_door, sideways), "walking past keeps it locked");
    assert!(floor.unlock_doors(near_door, side.outward() * 2.0));
    assert_eq!(
        floor.door_texture(side),
        TextureId::Door(crate::game::DoorTexture::ShopUnlocking(0))
    );
}

#[test]
fn a_hole_bombed_into_the_shop_is_open_without_a_key() {
    let found = (1..300).find_map(|seed| {
        let floor = Floor::generate(
            &RoomLayouts::default(),
            &GameParameters::ORIGINAL,
            &mut Random::with_seed(seed),
        );
        let roles = &floor.plan.roles_by_cell;
        let (&secret, _) = roles.iter().find(|(_, role)| **role == RoomRole::Secret)?;
        let side = DoorSide::ALL.into_iter().find(|side| {
            secret
                .through(*side)
                .is_some_and(|cell| roles.get(&cell) == Some(&RoomRole::Shop))
        })?;
        Some((floor, secret, side))
    });
    let (mut floor, secret, side) = found.expect("some floor has its secret room next to the shop");
    floor.current_cell = secret;
    floor.blast_walls(side.position() * 0.8, Pixels(60.0), &floor.blast_sight());
    assert!(floor.open_doors().any(|door| door == side));
}

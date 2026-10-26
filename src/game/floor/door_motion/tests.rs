use glam::Vec2;

use super::*;
use crate::game::floor::plan::RoomRole;
use crate::game::parameters::GameParameters;
use crate::game::random::Random;
use crate::game::room::enemies::{Enemies, EnemyParameters};
use crate::game::room::layout::{LayoutCell, RoomLayout, RoomLayouts};

const ONE_FRAME: Frames = Frames(1.0);
/// The black fly's marker value.
const FLY_MARKER: f32 = -4.0;

/// A floor standing in its boss room, which holds one fly.
fn boss_room_with_a_fly() -> Floor {
    room_with_a_fly(RoomRole::Boss)
}

/// A floor standing in a room of `role` holding one fly.
fn room_with_a_fly(role: RoomRole) -> Floor {
    let mut floor = Floor::generate(
        &RoomLayouts::default(),
        &GameParameters::ORIGINAL,
        &mut Random::with_seed(7),
    );
    let room = floor.cell_with(role).expect("a room of that kind");
    let layout = RoomLayout {
        cells: vec![LayoutCell {
            column: 2,
            row: 2,
            value: FLY_MARKER,
        }],
        ..RoomLayout::default()
    };
    floor.teleport_to(room, Vec2::ZERO);
    floor.current_room_mut().enemies = Enemies::from_layout(
        &layout,
        &EnemyParameters::from_game(&GameParameters::ORIGINAL),
        &mut Random::with_seed(1),
    );
    floor.set_doors_for_arrival();
    floor
}

fn advance(floor: &mut Floor, frames: u32, frames_in_room: &mut Frames) {
    for _ in 0..frames {
        *frames_in_room += ONE_FRAME;
        floor.update_doors(ONE_FRAME, *frames_in_room, &mut Vec::new());
    }
}

#[test]
fn boss_doors_swing_shut_then_open_ten_frames_after_the_last_enemy() {
    let mut floor = boss_room_with_a_fly();
    let mut frames_in_room = Frames::ZERO;
    assert_eq!(
        floor.moving_door_texture(DoorStyle::Boss),
        Some(TextureId::Door(DoorTexture::Closing(DoorStyle::Boss, 0)))
    );
    advance(&mut floor, 40, &mut frames_in_room);
    assert_eq!(
        floor.moving_door_texture(DoorStyle::Boss),
        Some(TextureId::Door(DoorTexture::Closing(DoorStyle::Boss, 10))),
        "held on the clip's last frame"
    );
    assert!(floor.doors_shut());
    // 260 frames in: a frame counter wrapping at 256 would show the clip's 5th frame again.
    advance(&mut floor, 220, &mut frames_in_room);
    assert_eq!(
        floor.moving_door_texture(DoorStyle::Boss),
        Some(TextureId::Door(DoorTexture::Closing(DoorStyle::Boss, 10))),
        "still held shut long after (no wrapping at 256 frames)"
    );
    floor.current_room_mut().enemies.blast(
        Vec2::ZERO,
        crate::game::original_units::Pixels(1000.0),
        |_| true,
        &mut crate::game::random::Random::with_seed(1),
        &mut Vec::new(),
    );
    assert!(!floor.doors_shut(), "usable at once");
    advance(&mut floor, 9, &mut frames_in_room);
    assert!(
        matches!(floor.door_motion.motion, DoorMotion::Closing { .. }),
        "still shut for 9 frames"
    );
    advance(&mut floor, 1, &mut frames_in_room);
    assert_eq!(
        floor.moving_door_texture(DoorStyle::Boss),
        Some(TextureId::Door(DoorTexture::Opening(DoorStyle::Boss, 0)))
    );
    advance(&mut floor, 19, &mut frames_in_room);
    assert_eq!(floor.door_motion.motion, DoorMotion::Open);
}

#[test]
fn a_blast_by_a_shut_plain_door_blows_it_open_but_not_a_boss_door() {
    let mut floor = room_with_a_fly(RoomRole::Normal);
    let plain = floor
        .doors()
        .find(|&side| floor.door_style(side) == DoorStyle::Plain)
        .expect("a plain door");
    let (column, row) = plain.doorway_cell();
    let doorway = crate::game::room::cell_bounds(column, row).center();
    let radius = crate::game::original_units::Pixels(40.0);
    assert!(floor.doors_shut());
    floor.blow_doors(doorway, radius);
    assert!(
        floor.open_doors().any(|side| side == plain),
        "usable while the fly lives"
    );
    assert_eq!(floor.door_texture(plain), TextureId::Door(DoorTexture::Blown(0)));

    let mut boss = boss_room_with_a_fly();
    let side = boss.doors().next().expect("a door");
    let (column, row) = side.doorway_cell();
    boss.blow_doors(crate::game::room::cell_bounds(column, row).center(), radius);
    assert_eq!(boss.open_doors().count(), 0, "boss doors do not blow open");
}

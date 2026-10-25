use glam::Vec2;

use crate::game::original_units::Frames;
use crate::game::parameters::GameParameters;
use crate::game::random::Random;
use crate::game::room::contents::RoomContents;
use crate::game::room::grid_kind::{BreakableKind, GridObjectKind, TileKind};
use crate::game::room::grid_object::{GridObject, HitStrength};
use crate::game::room::layout::showcase_layout;
use crate::game::room::tile_center;
use crate::game::sound_event::SoundEvent;
use crate::game::texture_id::EXPLOSION_FRAME_COUNT;

const ONE_FRAME: Frames = Frames(1.0);

fn room_with(objects: &[(GridObjectKind, u32, u32)]) -> RoomContents {
    let parameters = GameParameters::ORIGINAL;
    RoomContents {
        grid_objects: objects
            .iter()
            .map(|&(kind, column, row)| GridObject::on_tile(kind, column, row, parameters.breakables))
            .collect(),
        flies: Vec::new(),
        ..RoomContents::from_layout(&showcase_layout(), &parameters, None, &mut Random::with_seed(1))
    }
}

#[test]
fn pushing_into_a_key_block_opens_it_with_sounds() {
    let mut room = room_with(&[(GridObjectKind::KeyBlock, 6, 3)]);
    let block_center = room.grid_objects[0].center();
    let player_position = block_center - Vec2::new(30.0, 0.0);
    let mut sounds = Vec::new();
    for _ in 0..10 {
        room.push_against_key_blocks(player_position, Vec2::X, true, ONE_FRAME, &mut sounds);
    }
    assert!(!room.grid_objects[0].is_closed_key_block());
    assert_eq!(sounds, [SoundEvent::KeyUsed, SoundEvent::KeyBlockBroken]);
}

#[test]
fn a_key_block_stays_shut_without_a_key() {
    let mut room = room_with(&[(GridObjectKind::KeyBlock, 6, 3)]);
    let player_position = room.grid_objects[0].center() - Vec2::new(30.0, 0.0);
    let mut sounds = Vec::new();
    for _ in 0..10 {
        assert!(!room.push_against_key_blocks(player_position, Vec2::X, false, ONE_FRAME, &mut sounds));
    }
    assert!(room.grid_objects[0].is_closed_key_block());
}

#[test]
fn brushing_past_a_key_block_does_not_open_it() {
    let mut room = room_with(&[(GridObjectKind::KeyBlock, 6, 3)]);
    let block_center = room.grid_objects[0].center();
    let mut sounds = Vec::new();
    room.push_against_key_blocks(
        block_center - Vec2::new(30.0, 0.0),
        Vec2::X,
        true,
        ONE_FRAME,
        &mut sounds,
    );
    room.push_against_key_blocks(
        block_center - Vec2::new(30.0, 0.0),
        Vec2::X,
        true,
        ONE_FRAME,
        &mut sounds,
    );
    assert!(room.grid_objects[0].is_closed_key_block());
    assert!(sounds.is_empty(), "{sounds:?}");
}

#[test]
fn exploding_tnt_breaks_neighbors_and_leaves_a_scorch_mark() {
    let mut room = room_with(&[
        (GridObjectKind::Breakable(BreakableKind::TntBarrel), 6, 3),
        (GridObjectKind::Breakable(BreakableKind::TntBarrel), 8, 3),
        (GridObjectKind::Breakable(BreakableKind::Poop), 10, 3),
        (GridObjectKind::Breakable(BreakableKind::Poop), 6, 0),
    ]);
    let mut sounds = Vec::new();
    room.hit(0, HitStrength::Explosion, &mut Random::default(), &mut sounds);
    let intact: Vec<bool> = room.grid_objects.iter().map(GridObject::is_intact_breakable).collect();
    // The first barrel sets off the second, which reaches the poop beside it; the far poop survives.
    assert_eq!(intact, [false, false, false, true]);
    assert_eq!(room.explosions.len(), 2);
    assert_eq!(room.scorch_marks.len(), 2);
    assert_eq!(
        sounds.iter().filter(|&&sound| sound == SoundEvent::Explosion).count(),
        2
    );
    room.advance(
        Frames::from(EXPLOSION_FRAME_COUNT),
        Vec2::ZERO,
        &mut Random::with_seed(1),
        &mut Vec::new(),
    );
    assert!(room.explosions.is_empty());
    assert_eq!(room.scorch_marks.len(), 2);
}

#[test]
fn a_corny_poop_fly_leaves_with_its_poop() {
    let mut room = RoomContents::from_layout(
        &showcase_layout(),
        &GameParameters::ORIGINAL,
        None,
        &mut Random::with_seed(1),
    );
    let corny_index = room.flies[0].anchor_index;
    assert_eq!(
        room.grid_objects[corny_index].kind(),
        GridObjectKind::Breakable(BreakableKind::CornyPoop)
    );
    room.hit(
        corny_index,
        HitStrength::Explosion,
        &mut Random::default(),
        &mut Vec::new(),
    );
    room.advance(ONE_FRAME, Vec2::ZERO, &mut Random::with_seed(1), &mut Vec::new());
    assert!(room.flies.is_empty());
}

#[test]
fn a_bomb_clears_rocks_and_webs_but_not_blocks() {
    use crate::game::room::grid_kind::TileKind;
    let mut room = room_with(&[
        (GridObjectKind::Tile(TileKind::Rock), 5, 3),
        (GridObjectKind::Tile(TileKind::SpiderWeb), 7, 3),
        (GridObjectKind::Tile(TileKind::Block), 6, 2),
        (GridObjectKind::Tile(TileKind::Rock), 1, 3),
    ]);
    let center = room.grid_objects[2].center() - Vec2::new(0.0, 40.0);
    let mut sounds = Vec::new();
    room.explode_at(center, &mut Random::default(), &mut sounds);
    let near_rock = &room.grid_objects[0];
    assert!(!near_rock.blocks_walking() && !near_rock.blocks_tears());
    assert!(!room.grid_objects[1].is_spider_web());
    assert!(room.grid_objects[2].blocks_walking());
    assert!(room.grid_objects[3].blocks_walking(), "the far rock is out of reach");
    assert_eq!(sounds, [SoundEvent::Explosion, SoundEvent::RockCrumbled]);
}

#[test]
fn a_rock_blown_towards_a_pit_bridges_it() {
    use crate::game::room::grid_kind::TileKind;
    let mut room = room_with(&[
        (GridObjectKind::Tile(TileKind::Pit), 1, 3),
        (GridObjectKind::Tile(TileKind::Rock), 2, 3),
        (GridObjectKind::Tile(TileKind::Pit), 2, 4),
    ]);
    let bomb_position = room.grid_objects[1].center() + Vec2::new(40.0, 5.0);
    room.explode_at(bomb_position, &mut Random::default(), &mut Vec::new());
    assert!(
        !room.grid_objects[0].blocks_walking(),
        "the pit behind the rock is bridged"
    );
    assert!(
        room.grid_objects[2].blocks_walking(),
        "a pit beside the rock stays open"
    );
}

#[test]
fn a_rock_blown_up_between_two_pits_leaves_broken_floor() {
    let pit = GridObjectKind::Tile(TileKind::Pit);
    let rock = GridObjectKind::Tile(TileKind::Rock);
    let mut room = room_with(&[(pit, 5, 3), (rock, 6, 3), (pit, 7, 3)]);
    // From above, so the rubble does not fill either pit.
    room.explode_at(
        tile_center(6, 3) + Vec2::new(0.0, 20.0),
        &mut Random::default(),
        &mut Vec::new(),
    );
    assert_eq!(room.broken_floors, [tile_center(6, 3)]);
}

#[test]
fn rock_cleared_with_a_pit_on_one_side_only_or_far_from_the_blast_is_not_broken_floor() {
    let pit = GridObjectKind::Tile(TileKind::Pit);
    let rock = GridObjectKind::Tile(TileKind::Rock);
    let mut room = room_with(&[(pit, 5, 3), (rock, 6, 3)]);
    room.explode_at(
        tile_center(6, 3) + Vec2::new(0.0, 20.0),
        &mut Random::default(),
        &mut Vec::new(),
    );
    assert_eq!(room.broken_floors.len(), 0);
    let mut room = room_with(&[(pit, 5, 3), (rock, 6, 3), (pit, 7, 3)]);
    // Close enough to clear the rock (60) but not to break the floor (40).
    room.explode_at(
        tile_center(6, 3) + Vec2::new(0.0, 80.0),
        &mut Random::default(),
        &mut Vec::new(),
    );
    assert_eq!(room.broken_floors.len(), 0);
}

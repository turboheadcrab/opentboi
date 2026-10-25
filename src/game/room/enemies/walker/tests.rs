use crate::game::original_units::Frames;
use glam::Vec2;

use super::WalkerClip;
use super::test_support::{
    CALM_GAPER_MARKER, FRAMES_TO_WALK, GAPER_MARKER, ONE_FRAME, parameters, rock_on,
    room_with_marker as room_with_gaper, run, walker_world_position,
};
use crate::game::ball_contact::Body;
use crate::game::original_units::{Hearts, Pixels, distance};
use crate::game::parameters::GameParameters;
use crate::game::player::death::Killer;
use crate::game::random::Random;
use crate::game::room::enemies::Enemies;
use crate::game::room::grid_object::GridObject;
use crate::game::room::layout::{LayoutCell, RoomLayout};
use crate::game::room::{ROW_COUNT, interior_bounds, stage_to_world, tile_center};
use crate::game::sound_event::SoundEvent;

#[test]
fn a_plain_marker_makes_a_frantic_gaper_and_a_faint_one_a_calm_one() {
    assert!(room_with_gaper(GAPER_MARKER, 2, 3).walkers[0].frantic);
    assert!(!room_with_gaper(CALM_GAPER_MARKER, 2, 3).walkers[0].frantic);
}

#[test]
fn a_gaper_walks_to_the_player_across_an_empty_room() {
    let mut enemies = room_with_gaper(GAPER_MARKER, 2, 3);
    let player = tile_center(10, 3);
    run(&mut enemies, FRAMES_TO_WALK + 100, player, &[]);
    assert!(distance(walker_world_position(&enemies), player) < Pixels(30.0));
}

#[test]
fn a_gaper_does_not_move_while_it_appears() {
    let mut enemies = room_with_gaper(GAPER_MARKER, 2, 3);
    let start = walker_world_position(&enemies);
    run(&mut enemies, 40, tile_center(10, 3), &[]);
    assert!(distance(walker_world_position(&enemies), start) < Pixels(1.0));
}

#[test]
fn a_gaper_walks_round_a_wall_with_one_gap() {
    let wall: Vec<GridObject> = (0..6).map(|row| rock_on(6, row)).collect();
    let mut enemies = room_with_gaper(GAPER_MARKER, 2, 3);
    let player = tile_center(10, 3);
    run(&mut enemies, FRAMES_TO_WALK + 400, player, &wall);
    assert!(
        distance(walker_world_position(&enemies), player) < Pixels(40.0),
        "it ended at {:?}",
        walker_world_position(&enemies)
    );
}

#[test]
fn a_gaper_boxed_in_by_rocks_stays_put() {
    let mut boxed_in = Vec::new();
    for (column, row) in [(1, 3), (3, 3), (2, 2), (2, 4)] {
        boxed_in.push(rock_on(column, row));
    }
    let mut enemies = room_with_gaper(GAPER_MARKER, 2, 3);
    let start = walker_world_position(&enemies);
    run(&mut enemies, FRAMES_TO_WALK + 100, tile_center(10, 3), &boxed_in);
    assert!(distance(walker_world_position(&enemies), start) < Pixels(40.0));
}

#[test]
fn a_calm_gaper_is_slower_than_a_frantic_one() {
    let player = tile_center(11, 3);
    let mut frantic = room_with_gaper(GAPER_MARKER, 1, 3);
    let mut calm = room_with_gaper(CALM_GAPER_MARKER, 1, 3);
    run(&mut frantic, FRAMES_TO_WALK, player, &[]);
    run(&mut calm, FRAMES_TO_WALK, player, &[]);
    assert!(walker_world_position(&frantic).x > walker_world_position(&calm).x);
}

#[test]
fn touching_a_gaper_hurts_by_half_a_heart() {
    let mut enemies = room_with_gaper(GAPER_MARKER, 2, 3);
    let player = Body {
        position: walker_world_position(&enemies) + Vec2::new(5.0, 0.0),
        velocity: Vec2::ZERO,
        mass: GameParameters::ORIGINAL.pickups.player_mass,
    };
    let bump = enemies.bump(player, false, ONE_FRAME, &mut Random::with_seed(3), &mut Vec::new());
    assert_eq!(bump.hurt, Some((Hearts::HALF, Killer::Gaper)));
}

#[test]
fn a_frantic_gaper_never_opens_its_eyes_between_appearing_and_standing() {
    let mut enemies = room_with_gaper(GAPER_MARKER, 2, 3);
    let wall: Vec<GridObject> = (0..ROW_COUNT).map(|row| rock_on(6, row)).collect();
    let player = tile_center(12, 3);
    let appearing = parameters().gapers.appearing_frames.whole();
    for _ in 0..appearing + 20 {
        run(&mut enemies, 1, player, &wall);
        assert_eq!(enemies.walkers[0].head, 1, "its head stays on the frantic frame");
    }
}

#[test]
fn a_frantic_gaper_opens_its_eyes_on_sight_of_the_player_while_still_far_away() {
    let mut enemies = room_with_gaper(GAPER_MARKER, 1, 3);
    let player = tile_center(12, 3);
    for _ in 0..FRAMES_TO_WALK + 100 {
        run(&mut enemies, 1, player, &[]);
        if enemies.walkers[0].head > 2 {
            break;
        }
    }
    assert!(enemies.walkers[0].head > 2, "its head has gone on to the shouting");
    assert!(matches!(enemies.walkers[0].clip, WalkerClip::Walking { .. }));
    assert!(distance(walker_world_position(&enemies), player) > Pixels(250.0));
}

#[test]
fn a_calm_gaper_keeps_its_happy_face() {
    let mut enemies = room_with_gaper(CALM_GAPER_MARKER, 1, 3);
    run(&mut enemies, FRAMES_TO_WALK + 200, tile_center(12, 3), &[]);
    assert_eq!(enemies.walkers[0].head, 0);
}

#[test]
fn a_gaper_with_a_clear_line_growls_now_and_then_but_not_in_a_burst() {
    let mut enemies = room_with_gaper(GAPER_MARKER, 2, 3);
    let player = tile_center(12, 3);
    let mut random = Random::with_seed(5);
    let mut growl_frames = Vec::new();
    for frame in 0..600_u32 {
        let mut sounds = Vec::new();
        enemies.advance(ONE_FRAME, &[], player, interior_bounds(), &mut random, &mut sounds);
        if sounds.contains(&SoundEvent::GaperGrowled) {
            growl_frames.push(frame);
        }
    }
    assert!(!growl_frames.is_empty(), "it growls");
    let spacing = parameters().gapers.growl_spacing;
    assert!(
        growl_frames
            .windows(2)
            .all(|pair| Frames::from_number(pair[1] - pair[0]) >= spacing)
    );
}

#[test]
fn gapers_closing_in_on_the_same_spot_do_not_stack_into_one() {
    let layout = RoomLayout {
        cells: vec![
            LayoutCell {
                column: 2,
                row: 3,
                value: GAPER_MARKER,
            },
            LayoutCell {
                column: 10,
                row: 3,
                value: GAPER_MARKER,
            },
        ],
        ..RoomLayout::default()
    };
    let mut enemies = Enemies::from_layout(&layout, &parameters(), &mut Random::with_seed(1));
    run(&mut enemies, FRAMES_TO_WALK + 300, tile_center(6, 3), &[]);
    let apart = distance(
        stage_to_world(enemies.walkers[0].position),
        stage_to_world(enemies.walkers[1].position),
    );
    assert!(apart > Pixels(20.0), "{apart:?} apart");
}

#[test]
fn gapers_still_appearing_push_apart_too() {
    let layout = RoomLayout {
        cells: vec![LayoutCell {
            column: 6,
            row: 3,
            value: GAPER_MARKER,
        }],
        ..RoomLayout::default()
    };
    let mut enemies = Enemies::from_layout(&layout, &parameters(), &mut Random::with_seed(1));
    let mut twin = enemies.walkers[0].clone();
    twin.position += Vec2::new(4.0, 0.0);
    enemies.walkers.push(twin);
    run(&mut enemies, 1, tile_center(1, 1), &[]);
    let apart = distance(
        stage_to_world(enemies.walkers[0].position),
        stage_to_world(enemies.walkers[1].position),
    );
    assert!(apart > Pixels(4.0), "{apart:?} apart");
}

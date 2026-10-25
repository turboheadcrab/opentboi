use glam::Vec2;

use crate::game::ball_contact::Body;
use crate::game::original_units::{Frames, Hearts};
use crate::game::parameters::GameParameters;
use crate::game::player::death::Killer;
use crate::game::random::Random;
use crate::game::room::enemies::shot::EnemyShot;
use crate::game::room::enemies::{Enemies, EnemyParameters};
use crate::game::room::interior_bounds;
use crate::game::room::layout::{LayoutCell, RoomLayout};
use crate::game::room::{stage_to_world, world_to_stage};
use crate::game::texture_id::FlyKind;

const ONE_FRAME: Frames = Frames(1.0);
/// The Pooter's marker value (sprite 6603), and a half-transparent one.
const POOTER_MARKER: f32 = -5.0;
const HALF_TRANSPARENT_POOTER_MARKER: f32 = -5.1;

fn room_with(values: &[f32], seed: u32) -> Enemies {
    let layout = RoomLayout {
        cells: values
            .iter()
            .enumerate()
            .map(|(index, &value)| LayoutCell {
                column: 6 + u32::try_from(index).expect("a few markers"),
                row: 3,
                value,
            })
            .collect(),
        ..RoomLayout::default()
    };
    Enemies::from_layout(
        &layout,
        &EnemyParameters::from_game(&GameParameters::ORIGINAL),
        &mut Random::with_seed(seed),
    )
}

fn run(enemies: &mut Enemies, frames: u32, player: Vec2, random: &mut Random) {
    for _ in 0..frames {
        enemies.advance(ONE_FRAME, &[], player, interior_bounds(), random, &mut Vec::new());
        enemies.remove_finished();
    }
}

#[test]
fn pooter_markers_make_pooters_and_faint_ones_the_two_shot_kind() {
    let kinds: Vec<FlyKind> = (0..50)
        .flat_map(|seed| room_with(&[POOTER_MARKER], seed).flies)
        .map(|fly| fly.kind)
        .collect();
    assert!(kinds.contains(&FlyKind::Pooter));
    assert!(kinds.iter().all(|kind| kind.is_pooter()));
    let faint = room_with(&[HALF_TRANSPARENT_POOTER_MARKER], 1);
    assert_eq!(faint.flies[0].kind, FlyKind::SuperPooter);
}

#[test]
fn a_pooter_shoots_a_player_in_reach_and_the_shot_hurts() {
    // The two-shot Pooter's spread passes either side of a player standing this far off.
    let mut enemies = (1..=u32::MAX)
        .map(|seed| room_with(&[POOTER_MARKER], seed))
        .find(|room| room.flies[0].kind == FlyKind::Pooter)
        .expect("a plain Pooter");
    let mut random = Random::with_seed(3);
    let pooter = stage_to_world(enemies.flies[0].position);
    let player = pooter + Vec2::new(-120.0, -20.0);
    let mut hurt = None;
    for _ in 0..200 {
        run(&mut enemies, 1, player, &mut random);
        let body = Body {
            position: player,
            velocity: Vec2::ZERO,
            mass: GameParameters::ORIGINAL.pickups.player_mass,
        };
        hurt = hurt.or(enemies.bump(body, false, ONE_FRAME, &mut random, &mut Vec::new()).hurt);
    }
    assert_eq!(
        hurt,
        Some((Hearts::HALF, Killer::Enemy(FlyKind::Pooter))),
        "a shot reached the player"
    );
}

#[test]
fn a_pooter_does_not_shoot_a_player_out_of_reach() {
    let mut enemies = room_with(&[POOTER_MARKER], 2);
    let mut random = Random::with_seed(3);
    let far_corner = stage_to_world(world_to_stage(Vec2::ZERO) + Vec2::new(-250.0, 140.0));
    for _ in 0..120 {
        run(&mut enemies, 1, far_corner, &mut random);
        let pooter = stage_to_world(enemies.flies[0].position);
        if pooter.distance(far_corner) < GameParameters::ORIGINAL.pooters.sight_reach.get() {
            return;
        }
        assert!(enemies.shots.is_empty());
    }
}

#[test]
fn a_shot_created_this_frame_touches_nothing_until_the_next() {
    let mut enemies = room_with(&[], 1);
    let player = Vec2::ZERO;
    let frame = enemies.frames.floor();
    enemies.shots.push(
        EnemyShot::new(
            world_to_stage(player),
            Vec2::ZERO,
            Killer::Enemy(FlyKind::Pooter),
            GameParameters::ORIGINAL.enemy_shots,
        )
        .born_in(frame),
    );
    let body = Body {
        position: player,
        velocity: Vec2::ZERO,
        mass: GameParameters::ORIGINAL.pickups.player_mass,
    };
    let mut random = Random::with_seed(1);
    let bump =
        |enemies: &mut Enemies, random: &mut Random| enemies.bump(body, false, ONE_FRAME, random, &mut Vec::new());
    assert_eq!(bump(&mut enemies, &mut random).hurt, None, "created in this frame");
    enemies.frames += ONE_FRAME;
    assert_eq!(
        bump(&mut enemies, &mut random).hurt,
        Some((Hearts::HALF, Killer::Enemy(FlyKind::Pooter)))
    );
}

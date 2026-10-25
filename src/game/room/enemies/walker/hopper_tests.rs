use super::test_support::{ONE_FRAME, room_with_marker, run};
use crate::game::original_units::{Damage, distance};
use crate::game::player::death::Killer;
use crate::game::random::Random;
use crate::game::room::enemies::Enemies;
use crate::game::room::enemies::WalkerKind;
use crate::game::room::interior_bounds;
use crate::game::room::tile_center;
use crate::game::sound_event::SoundEvent;

/// The Hopper's marker value (sprite 6612) and the flesh mound's (6613).
const HOPPER_MARKER: f32 = -20.0;
const FLESH_MOUND_MARKER: f32 = -68.0;
/// Its appearing clip, then a few leaps.
const FRAMES_TO_APPEAR: u32 = 30;

/// One frame at a time with the same dice throughout (`run` starts its own each call).
fn step(enemies: &mut Enemies, player: glam::Vec2, random: &mut Random) -> Vec<SoundEvent> {
    let mut sounds = Vec::new();
    enemies.advance(ONE_FRAME, &[], player, interior_bounds(), random, &mut sounds);
    sounds
}

#[test]
fn markers_make_a_hopper_and_a_flesh_mound() {
    assert_eq!(
        room_with_marker(HOPPER_MARKER, 3, 3).walkers[0].kind,
        WalkerKind::Hopper
    );
    assert_eq!(
        room_with_marker(FLESH_MOUND_MARKER, 3, 3).walkers[0].kind,
        WalkerKind::FleshMound
    );
}

#[test]
fn a_hopper_hops_without_a_pause_through_the_frames_of_its_clip_and_lands_on_the_nineteenth() {
    let mut enemies = room_with_marker(HOPPER_MARKER, 6, 3);
    let far = tile_center(12, 6);
    let mut random = Random::with_seed(4);
    (0..FRAMES_TO_APPEAR).for_each(|_| drop(step(&mut enemies, far, &mut random)));
    let mut frames = Vec::new();
    let mut landings = 0;
    for _ in 0..400 {
        let sounds = step(&mut enemies, far, &mut random);
        landings += sounds
            .iter()
            .filter(|&&sound| sound == SoundEvent::HopperLanded)
            .count();
        frames.push(enemies.walkers[0].hop_frame());
    }
    assert!(
        frames.iter().flatten().all(|&frame| (1..=22).contains(&frame)),
        "its clip stops short of the end: {frames:?}"
    );
    assert!(
        frames.iter().filter(|frame| **frame == Some(2)).count() >= 10,
        "{frames:?}"
    );
    assert!(landings >= 10, "it lands again and again: {landings}");
    assert!(
        frames.windows(2).all(|pair| pair[0] != Some(19) || pair[1] == Some(20)),
        "the clip does not stall"
    );
}

#[test]
fn a_hopper_hops_at_a_player_in_reach() {
    let mut enemies = room_with_marker(HOPPER_MARKER, 6, 3);
    let start = crate::game::room::stage_to_world(enemies.walkers[0].position);
    let player = start + glam::Vec2::new(100.0, 0.0);
    run(&mut enemies, FRAMES_TO_APPEAR + 60, player, &[]);
    let now = crate::game::room::stage_to_world(enemies.walkers[0].position);
    assert!(
        distance(now, player).get() < 60.0,
        "it came to the player: {now} for {player}"
    );
}

#[test]
fn a_hopper_takes_ten_hit_points_to_kill_and_hurts_on_touch() {
    let mut enemies = room_with_marker(HOPPER_MARKER, 3, 3);
    let parameters = enemies.parameters;
    let mut random = Random::with_seed(1);
    assert_eq!(
        enemies.walkers[0].hurt(Damage(10.0), &parameters, &mut random),
        super::Hurt::Survived
    );
    assert_eq!(
        enemies.walkers[0].hurt(Damage(1.0), &parameters, &mut random),
        super::Hurt::Died
    );
    let mut enemies = room_with_marker(HOPPER_MARKER, 3, 3);
    run(&mut enemies, FRAMES_TO_APPEAR, tile_center(12, 6), &[]);
    let point = crate::game::room::stage_to_world(enemies.walkers[0].position);
    let bump = enemies.bump(
        super::test_support::player_body(point - glam::Vec2::new(8.0, 0.0)),
        false,
        ONE_FRAME,
        &mut Random::with_seed(1),
        &mut Vec::new(),
    );
    assert_eq!(
        bump.hurt,
        Some((crate::game::original_units::Hearts::HALF, Killer::Hopper))
    );
}

#[test]
fn a_flesh_mound_wanders_and_takes_twelve_hit_points() {
    let mut enemies = room_with_marker(FLESH_MOUND_MARKER, 6, 3);
    let start = enemies.walkers[0].position;
    run(&mut enemies, FRAMES_TO_APPEAR, tile_center(12, 6), &[]);
    let mut farthest = 0.0_f32;
    for _ in 0..300 {
        run(&mut enemies, 1, tile_center(12, 6), &[]);
        farthest = farthest.max(enemies.walkers[0].position.distance(start));
    }
    assert!(farthest > 20.0, "it wanders off: {farthest}");
    let parameters = enemies.parameters;
    let mut random = Random::with_seed(1);
    assert_eq!(
        enemies.walkers[0].hurt(Damage(12.0), &parameters, &mut random),
        super::Hurt::Survived
    );
    assert_eq!(
        enemies.walkers[0].hurt(Damage(1.0), &parameters, &mut random),
        super::Hurt::Died
    );
}

use crate::game::original_units::Frames;
use glam::Vec2;

use super::test_support::{
    GAPER_MARKER, HEADLESS_MARKER, HEADLESS_MARKER_WITHOUT_BLOOD, ONE_FRAME, room_with_marker, run,
    walker_world_position,
};
use crate::game::ball_contact::Body;
use crate::game::original_units::{Hearts, Pixels, distance};
use crate::game::parameters::GameParameters;
use crate::game::player::death::Killer;
use crate::game::random::Random;
use crate::game::room::enemies::WalkerKind;
use crate::game::room::{interior_bounds, tile_center};
use crate::game::sound_event::SoundEvent;

fn count(sounds: &[SoundEvent], wanted: SoundEvent) -> usize {
    sounds.iter().filter(|&&sound| sound == wanted).count()
}

#[test]
fn a_marker_makes_a_headless_body_and_a_fractional_one_does_not_spurt() {
    let plain = room_with_marker(HEADLESS_MARKER, 6, 3);
    assert_eq!(plain.walkers[0].kind, WalkerKind::Headless);
    assert!(plain.walkers[0].spurts);
    assert!(!room_with_marker(HEADLESS_MARKER_WITHOUT_BLOOD, 6, 3).walkers[0].spurts);
}

#[test]
fn a_headless_body_wanders_about_on_its_own() {
    let mut enemies = room_with_marker(HEADLESS_MARKER, 6, 3);
    let start = walker_world_position(&enemies);
    let far_away = tile_center(0, 0);
    let mut farthest = Pixels(0.0);
    for _ in 0..40 {
        run(&mut enemies, 10, far_away, &[]);
        farthest = farthest.max(distance(walker_world_position(&enemies), start));
    }
    assert!(farthest > Pixels(20.0), "it wandered {farthest:?}");
    assert!(interior_bounds().contains(walker_world_position(&enemies)));
}

#[test]
fn a_spurting_body_shoots_blood_balls_every_forty_frames_or_more() {
    let mut enemies = room_with_marker(HEADLESS_MARKER, 6, 3);
    let mut random = Random::with_seed(6);
    let mut frames_of_shots = Vec::new();
    for frame in 0..600_u32 {
        let mut sounds = Vec::new();
        enemies.advance(
            ONE_FRAME,
            &[],
            tile_center(0, 0),
            interior_bounds(),
            &mut random,
            &mut sounds,
        );
        if sounds.contains(&SoundEvent::BloodShot) {
            frames_of_shots.push(frame);
        }
    }
    assert!(frames_of_shots.len() >= 4, "it spurts: {frames_of_shots:?}");
    let spacing = GameParameters::ORIGINAL.headless.spurt_cooldown_base;
    assert!(
        frames_of_shots
            .windows(2)
            .all(|pair| Frames::from_number(pair[1] - pair[0]) >= spacing)
    );
}

#[test]
fn a_body_without_blood_never_shoots() {
    let mut enemies = room_with_marker(HEADLESS_MARKER_WITHOUT_BLOOD, 6, 3);
    let sounds = run(&mut enemies, 600, tile_center(0, 0), &[]);
    assert_eq!(count(&sounds, SoundEvent::BloodShot), 0);
}

#[test]
fn touching_a_headless_body_hurts_in_its_name() {
    let mut enemies = room_with_marker(HEADLESS_MARKER, 6, 3);
    let player = Body {
        position: walker_world_position(&enemies) + Vec2::new(5.0, 0.0),
        velocity: Vec2::ZERO,
        mass: GameParameters::ORIGINAL.pickups.player_mass,
    };
    let bump = enemies.bump(player, false, ONE_FRAME, &mut Random::with_seed(3), &mut Vec::new());
    assert_eq!(bump.hurt, Some((Hearts::HALF, Killer::Headless)));
}

#[test]
fn a_headless_body_does_not_push_a_gaper_but_a_gaper_pushes_a_gaper() {
    let mut enemies = room_with_marker(GAPER_MARKER, 6, 3);
    let mut twin = enemies.walkers[0].clone();
    twin.position += Vec2::new(4.0, 0.0);
    enemies.walkers.push(twin);
    run(&mut enemies, 1, tile_center(0, 0), &[]);
    let gapers_apart = distance(
        crate::game::room::stage_to_world(enemies.walkers[0].position),
        crate::game::room::stage_to_world(enemies.walkers[1].position),
    );
    assert!(gapers_apart > Pixels(4.0), "two Gapers part");

    let mut enemies = room_with_marker(GAPER_MARKER, 6, 3);
    let mut body = enemies.walkers[0].clone();
    body.kind = WalkerKind::Headless;
    body.position += Vec2::new(4.0, 0.0);
    enemies.walkers.push(body);
    run(&mut enemies, 1, tile_center(0, 0), &[]);
    let apart = distance(
        crate::game::room::stage_to_world(enemies.walkers[0].position),
        crate::game::room::stage_to_world(enemies.walkers[1].position),
    );
    assert!(
        (apart - Pixels(4.0)).get().abs() < 0.5,
        "a body does not push: {apart:?}"
    );
}

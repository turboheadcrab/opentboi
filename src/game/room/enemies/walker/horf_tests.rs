use crate::game::original_units::Frames;
use glam::Vec2;

use super::test_support::{
    FRAMES_TO_WALK, HORF_MARKER, ONE_FRAME, rock_on, room_with_marker, run, walker_world_position,
};
use crate::game::original_units::{Damage, Hearts, Pixels, distance};
use crate::game::parameters::GameParameters;
use crate::game::player::death::Killer;
use crate::game::random::Random;
use crate::game::room::enemies::WalkerKind;
use crate::game::room::tile_center;
use crate::game::sound_event::SoundEvent;

fn count(sounds: &[SoundEvent], wanted: SoundEvent) -> usize {
    sounds.iter().filter(|&&sound| sound == wanted).count()
}

#[test]
fn a_marker_makes_horf() {
    let enemies = room_with_marker(HORF_MARKER, 3, 3);
    assert_eq!(enemies.walkers[0].kind, WalkerKind::Horf);
}

#[test]
fn horf_shoots_a_player_within_reach_and_sight_but_never_moves() {
    let mut enemies = room_with_marker(HORF_MARKER, 3, 3);
    let start = walker_world_position(&enemies);
    let sounds = run(&mut enemies, FRAMES_TO_WALK + 200, tile_center(6, 3), &[]);
    assert!(count(&sounds, SoundEvent::HorfShot) >= 2, "it keeps shooting");
    assert!(distance(walker_world_position(&enemies), start) < Pixels(1.0));
}

#[test]
fn horfs_shots_are_at_least_a_cooldown_apart() {
    let mut enemies = room_with_marker(HORF_MARKER, 3, 3);
    let player = tile_center(6, 3);
    let mut random = Random::with_seed(4);
    let mut frames_of_shots = Vec::new();
    for frame in 0..400_u32 {
        let mut sounds = Vec::new();
        enemies.advance(
            ONE_FRAME,
            &[],
            player,
            crate::game::room::interior_bounds(),
            &mut random,
            &mut sounds,
        );
        if sounds.contains(&SoundEvent::HorfShot) {
            frames_of_shots.push(frame);
        }
    }
    let parameters = GameParameters::ORIGINAL.pooters;
    let spacing = parameters.shot_check_every * f32::from(parameters.cooldown_checks);
    assert!(
        frames_of_shots
            .windows(2)
            .all(|pair| Frames::from_number(pair[1] - pair[0]) >= spacing)
    );
}

#[test]
fn horf_ignores_a_player_too_far_off_or_behind_a_rock() {
    let mut far = room_with_marker(HORF_MARKER, 1, 3);
    let sounds = run(&mut far, FRAMES_TO_WALK + 200, tile_center(12, 3), &[]);
    assert_eq!(count(&sounds, SoundEvent::HorfShot), 0, "out of reach");
    let mut hidden = room_with_marker(HORF_MARKER, 3, 3);
    let wall: Vec<_> = (0..7).map(|row| rock_on(5, row)).collect();
    let sounds = run(&mut hidden, FRAMES_TO_WALK + 200, tile_center(7, 3), &wall);
    assert_eq!(count(&sounds, SoundEvent::HorfShot), 0, "out of sight");
}

#[test]
fn horfs_shot_hurts_the_player_it_reaches_in_horfs_name() {
    let mut enemies = room_with_marker(HORF_MARKER, 3, 3);
    let player_spot = tile_center(6, 3);
    let player = crate::game::ball_contact::Body {
        position: player_spot,
        velocity: Vec2::ZERO,
        mass: GameParameters::ORIGINAL.pickups.player_mass,
    };
    let mut random = Random::with_seed(4);
    let mut hurt = None;
    for _ in 0..300 {
        enemies.advance(
            ONE_FRAME,
            &[],
            player_spot,
            crate::game::room::interior_bounds(),
            &mut random,
            &mut Vec::new(),
        );
        hurt = hurt.or(enemies
            .bump(player, false, ONE_FRAME, &mut random, &mut Vec::new())
            .hurt);
    }
    assert_eq!(hurt, Some((Hearts::HALF, Killer::Horf)));
}

#[test]
fn three_tears_kill_horf() {
    let mut enemies = room_with_marker(HORF_MARKER, 3, 3);
    let spot = walker_world_position(&enemies);
    let mut sounds = Vec::new();
    let mut random = Random::with_seed(9);
    for _ in 0..3 {
        assert!(enemies.hit_by_tear(spot, Vec2::ZERO, Damage(3.5), ONE_FRAME, &mut random, &mut sounds));
    }
    assert!(!enemies.any_alive());
    assert_eq!(sounds, [SoundEvent::EnemyDied]);
}

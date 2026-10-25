use super::WalkerClip;
use super::test_support::{ONE_FRAME, room_with_marker, run, walker_world_position};
use crate::game::original_units::{Damage, Frames, Pixels, distance};
use crate::game::random::Random;
use crate::game::room::enemies::{WalkerKind, WalkerState};
use crate::game::room::tile_center;
use crate::game::sound_event::SoundEvent;

const MULLIGAN_MARKER: f32 = -7.0;
const FLIES_LEFT: usize = 4;

#[test]
fn a_marker_makes_a_mulligan() {
    let enemies = room_with_marker(MULLIGAN_MARKER, 3, 3);
    assert_eq!(enemies.walkers[0].kind, WalkerKind::Mulligan);
}

#[test]
fn a_mulligan_runs_from_a_player_who_comes_close() {
    let mut enemies = room_with_marker(MULLIGAN_MARKER, 6, 3);
    let player = tile_center(8, 3);
    let start = distance(walker_world_position(&enemies), player);
    run(&mut enemies, 200, player, &[]);
    let end = distance(walker_world_position(&enemies), player);
    assert!(enemies.walkers[0].is_scared() || end > start);
    assert!(end > start + Pixels(40.0), "it keeps away: {start:?} → {end:?}");
}

#[test]
fn a_mulligan_killed_leaves_four_flies() {
    let mut enemies = room_with_marker(MULLIGAN_MARKER, 6, 3);
    let mut sounds = Vec::new();
    enemies.hurt_all(Damage(20.0), &mut Random::with_seed(3), &mut sounds);
    assert_eq!(enemies.flies.len(), FLIES_LEFT);
    assert!(enemies.any_alive(), "the flies are enemies too");
}

#[test]
fn a_mulligan_blown_far_past_death_leaves_none() {
    let mut enemies = room_with_marker(MULLIGAN_MARKER, 6, 3);
    enemies.hurt_all(Damage(60.0), &mut Random::with_seed(3), &mut Vec::new());
    assert!(enemies.flies.is_empty());
}

#[test]
fn a_bursting_mulligan_fires_four_shots_leaves_flies_and_dies() {
    let mut enemies = room_with_marker(MULLIGAN_MARKER, 6, 3);
    run(&mut enemies, 60, tile_center(12, 6), &[]);
    enemies.walkers[0].clip = WalkerClip::Shooting { since: Frames(25.0) };
    let mut sounds = Vec::new();
    let mut random = Random::with_seed(2);
    enemies.advance(
        ONE_FRAME,
        &[],
        tile_center(12, 6),
        crate::game::room::interior_bounds(),
        &mut random,
        &mut sounds,
    );
    assert!(matches!(enemies.walkers[0].state, WalkerState::Dying(_)));
    assert_eq!(enemies.shots.len(), 4);
    assert_eq!(enemies.flies.len(), FLIES_LEFT);
    assert!(sounds.contains(&SoundEvent::BloodShot));
}

#[test]
fn the_death_card_kills_attack_flies_and_gapers() {
    for marker in [-9.0, -1.0, -3.0] {
        let mut enemies = room_with_marker(marker, 6, 3);
        enemies.hurt_all(Damage(60.0), &mut Random::with_seed(3), &mut Vec::new());
        assert!(!enemies.any_alive(), "marker {marker}");
    }
}

#[test]
fn touching_a_mulligan_does_not_hurt_but_a_clotty_does() {
    use crate::game::ball_contact::Body;
    use crate::game::original_units::Mass;
    for (marker, hurts) in [(MULLIGAN_MARKER, false), (-6.0, true)] {
        let mut enemies = room_with_marker(marker, 6, 3);
        let position = walker_world_position(&enemies);
        let player = Body {
            position: position + glam::Vec2::new(5.0, 0.0),
            velocity: glam::Vec2::ZERO,
            mass: Mass(5.0),
        };
        let bump = enemies.bump(player, false, ONE_FRAME, &mut Random::with_seed(1), &mut Vec::new());
        assert_eq!(bump.hurt.is_some(), hurts, "marker {marker}");
    }
}

use glam::Vec2;

use super::WalkerClip;
use super::test_support::{ONE_FRAME, room_with_marker, run};
use crate::game::original_units::{Frames, PixelsPerFrame};
use crate::game::random::Random;
use crate::game::room::enemies::WalkerKind;
use crate::game::room::enemies::shot::EnemyShot;
use crate::game::room::{interior_bounds, tile_center};
use crate::game::sound_event::SoundEvent;

const CLOTTY_MARKER: f32 = -6.0;
const SHOT_SPEED: PixelsPerFrame = PixelsPerFrame(7.0);

#[test]
fn a_marker_makes_a_clotty() {
    let enemies = room_with_marker(CLOTTY_MARKER, 3, 3);
    assert_eq!(enemies.walkers[0].kind, WalkerKind::Clotty);
}

#[test]
fn a_clotty_shoots_four_shots_along_the_axes_now_and_then() {
    let mut enemies = room_with_marker(CLOTTY_MARKER, 3, 3);
    let player = tile_center(12, 3);
    let mut random = Random::with_seed(6);
    let mut volleys = 0;
    for _ in 0..1200 {
        let before = enemies.shots.len();
        let mut sounds = Vec::new();
        enemies.advance(ONE_FRAME, &[], player, interior_bounds(), &mut random, &mut sounds);
        if sounds.contains(&SoundEvent::ClottySquished) {
            volleys += 1;
            let fresh = &enemies.shots[before..];
            assert_eq!(fresh.len(), 4, "four shots");
            let mut velocities: Vec<Vec2> = fresh.iter().map(EnemyShot::velocity).collect();
            velocities.sort_by(|first, second| first.to_array().partial_cmp(&second.to_array()).unwrap());
            let speed = SHOT_SPEED.get();
            assert_eq!(
                velocities,
                [
                    Vec2::new(-speed, 0.0),
                    Vec2::new(0.0, -speed),
                    Vec2::new(0.0, speed),
                    Vec2::new(speed, 0.0)
                ]
            );
        }
    }
    assert!(volleys >= 2, "it shoots again and again: {volleys}");
}

#[test]
fn a_clotty_takes_fifteen_hit_points_to_kill() {
    use crate::game::original_units::Damage;
    let mut enemies = room_with_marker(CLOTTY_MARKER, 3, 3);
    let parameters = enemies.parameters;
    let mut random = Random::with_seed(1);
    assert_eq!(
        enemies.walkers[0].hurt(Damage(15.0), &parameters, &mut random),
        super::Hurt::Survived
    );
    assert_eq!(
        enemies.walkers[0].hurt(Damage(1.0), &parameters, &mut random),
        super::Hurt::Died
    );
}

#[test]
fn a_clotty_starts_shooting_with_its_shot_clip() {
    let mut enemies = room_with_marker(CLOTTY_MARKER, 3, 3);
    let mut saw_the_clip = false;
    for _ in 0..600 {
        run(&mut enemies, 1, tile_center(12, 3), &[]);
        saw_the_clip |= matches!(enemies.walkers[0].clip, WalkerClip::Shooting { .. });
    }
    assert!(saw_the_clip);
    let _ = Frames::ZERO;
}

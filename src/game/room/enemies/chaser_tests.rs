//! The flies that go for the player: the red fly bursts into shots and the Moter splits in two.

use super::tests::{FAR_PLAYER, ONE_FRAME, parameters, player_at, room_with_flies, run_with_player};
use super::*;
use crate::game::original_units::Damage;
use crate::game::parameters::GameParameters;
use crate::game::texture_id::FlyKind;

/// The red fly's marker value (sprite 6619) and the Moter's (6616).
const RED_FLY_MARKER: f32 = -52.0;
const MOTER_MARKER: f32 = -71.0;

fn kill(enemies: &mut Enemies, damage: Damage) {
    let point = stage_to_world(enemies.flies[0].position);
    let mut sounds = Vec::new();
    enemies.hit_by_tear(
        point,
        Vec2::X,
        damage,
        ONE_FRAME,
        &mut Random::with_seed(1),
        &mut sounds,
    );
    assert_eq!(sounds, [SoundEvent::EnemyDied]);
}

/// Lets them appear, so a hit lands on a fly that thinks.
fn appeared(marker: f32) -> Enemies {
    let mut enemies = room_with_flies(&[marker]);
    run_with_player(&mut enemies, 40, FAR_PLAYER, &mut Random::with_seed(3));
    enemies
}

#[test]
fn the_markers_place_a_red_fly_and_a_moter() {
    let enemies = room_with_flies(&[RED_FLY_MARKER, MOTER_MARKER]);
    let kinds: Vec<FlyKind> = enemies.flies.iter().map(|fly| fly.kind).collect();
    assert_eq!(kinds, [FlyKind::Red, FlyKind::Moter]);
}

#[test]
fn both_hurt_on_touch_by_their_own_portrait() {
    for kind in [FlyKind::Red, FlyKind::Moter] {
        let mut enemies = appeared(if kind == FlyKind::Red {
            RED_FLY_MARKER
        } else {
            MOTER_MARKER
        });
        let point = stage_to_world(enemies.flies[0].position);
        let bump = enemies.bump(
            player_at(point - Vec2::new(8.0, 0.0), Vec2::ZERO),
            false,
            ONE_FRAME,
            &mut Random::with_seed(1),
            &mut Vec::new(),
        );
        assert_eq!(bump.hurt, Some((parameters().flies.attack_damage, Killer::Enemy(kind))));
    }
}

#[test]
fn a_red_fly_dies_in_a_shot_along_each_axis() {
    let mut enemies = appeared(RED_FLY_MARKER);
    kill(&mut enemies, Damage(100.0));
    run_with_player(&mut enemies, 1, FAR_PLAYER, &mut Random::with_seed(3));
    let speed = GameParameters::ORIGINAL.flies.red_burst_speed.get();
    let mut velocities: Vec<Vec2> = enemies.shots.iter().map(shot::EnemyShot::velocity).collect();
    velocities.sort_by(|first, second| first.to_array().partial_cmp(&second.to_array()).expect("numbers"));
    assert_eq!(
        velocities,
        [
            Vec2::new(-speed, 0.0),
            Vec2::new(0.0, -speed),
            Vec2::new(0.0, speed),
            Vec2::new(speed, 0.0)
        ]
    );
    assert!(
        enemies
            .shots
            .iter()
            .all(|shot| shot.shooter == Killer::Enemy(FlyKind::Red))
    );
}

#[test]
fn a_moter_splits_into_two_attack_flies_on_either_side() {
    let mut enemies = appeared(MOTER_MARKER);
    let moter = enemies.flies[0].position;
    kill(&mut enemies, Damage(11.0));
    run_with_player(&mut enemies, 1, FAR_PLAYER, &mut Random::with_seed(3));
    let born: Vec<&Fly> = enemies.flies.iter().skip(1).collect();
    assert_eq!(born.len(), 2);
    assert!(born.iter().all(|fly| fly.kind == FlyKind::Attack));
    let middle = (born[0].position + born[1].position) / 2.0;
    assert!(
        middle.distance(moter) < 1.0,
        "either side of the Moter: {middle} against {moter}"
    );
    let sent = GameParameters::ORIGINAL.flies.moter_split_offset.get();
    assert!((born[0].velocity.length() - sent).abs() < 0.5 && (born[0].velocity + born[1].velocity).length() < 0.5);
}

#[test]
fn a_moter_blown_far_past_zero_leaves_nothing() {
    let mut enemies = appeared(MOTER_MARKER);
    kill(&mut enemies, Damage(40.0));
    run_with_player(&mut enemies, 1, FAR_PLAYER, &mut Random::with_seed(3));
    assert_eq!(enemies.flies.len(), 1);
}

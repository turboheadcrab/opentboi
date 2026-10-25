use super::test_support::{
    GAPER_MARKER, ONE_FRAME, parameters, room_with_marker as room_with_gaper, run, walker_world_position,
};
use crate::game::original_units::{Damage, Pixels};
use crate::game::random::Random;
use crate::game::room::enemies::{Enemies, WalkerKind};
use crate::game::sound_event::SoundEvent;
use glam::Vec2;

/// Spreads small numbers over the seeds (a small seed's first draws are all alike).
const SEED_SPREAD: u32 = 2_654_435_761;

/// A seed whose first coin flip (`random(2)`) comes up as `beheaded` says.
fn seed_for_coin(beheaded: bool) -> u32 {
    (1..200_u32)
        .map(|number| number.wrapping_mul(SEED_SPREAD))
        .find(|&seed| (Random::with_seed(seed).below(2) == 0) == beheaded)
        .expect("a seed of each kind")
}

/// Three tears (10.5 damage against 10 hit points), the last one with the given coin flip.
fn three_tears(enemies: &mut Enemies, spot: Vec2, seed: u32, sounds: &mut Vec<SoundEvent>) {
    let mut random = Random::with_seed(seed);
    for _ in 0..3 {
        assert!(enemies.hit_by_tear(spot, Vec2::ZERO, Damage(3.5), ONE_FRAME, &mut random, sounds));
    }
}

#[test]
fn three_tears_kill_a_gaper_and_its_death_clip_then_clears() {
    let mut enemies = room_with_gaper(GAPER_MARKER, 2, 3);
    let spot = walker_world_position(&enemies);
    let mut sounds = Vec::new();
    three_tears(&mut enemies, spot, seed_for_coin(false), &mut sounds);
    assert!(!enemies.any_alive());
    assert_eq!(sounds.len(), 1, "one death");
    assert_eq!(enemies.walkers.len(), 1, "its death clip is still playing");
    run(&mut enemies, 40, spot, &[]);
    assert!(enemies.walkers.is_empty());
}

#[test]
fn a_gaper_killed_by_a_close_hit_sometimes_goes_on_as_a_headless_body() {
    let mut enemies = room_with_gaper(GAPER_MARKER, 2, 3);
    let spot = walker_world_position(&enemies);
    let mut sounds = Vec::new();
    three_tears(&mut enemies, spot, seed_for_coin(true), &mut sounds);
    assert!(enemies.any_alive(), "it is still on its feet");
    assert!(sounds.is_empty(), "silently");
    assert_eq!(enemies.walkers[0].kind, WalkerKind::Headless);
    assert!(
        (enemies.walkers[0].hit_points().get() - parameters().gapers.hit_points.get()).abs() < f32::EPSILON,
        "at full health"
    );
}

#[test]
fn a_blast_leaves_no_gaper_to_lose_its_head() {
    let mut enemies = room_with_gaper(GAPER_MARKER, 2, 3);
    let spot = walker_world_position(&enemies);
    let mut sounds = Vec::new();
    enemies.blast(
        spot,
        Pixels(100.0),
        &mut Random::with_seed(seed_for_coin(true)),
        &mut sounds,
    );
    assert!(!enemies.any_alive(), "40 damage is far past the limit");
}

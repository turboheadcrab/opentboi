use super::spawn::SPAWN_DROP;
use super::*;
use crate::game::ball_contact::Body;
use crate::game::original_units::distance;
use crate::game::parameters::GameParameters;
use crate::game::player::death::Killer;
use crate::game::room::interior_bounds;
use crate::game::room::layout::LayoutCell;
use crate::game::room::tile_center;
use crate::game::texture_id::FlyKind;

pub(super) fn parameters() -> EnemyParameters {
    EnemyParameters::from_game(&GameParameters::ORIGINAL)
}
pub(super) const ONE_FRAME: Frames = Frames(1.0);
/// The black fly's marker value (sprite 6601) and its half-transparent variant.
const FLY_MARKER: f32 = -4.0;
const FAINT_FLY_MARKER: f32 = -4.1;
/// The attack fly's marker value (sprite 6589).
const ATTACK_FLY_MARKER: f32 = -9.0;
/// Where the player stands in the tests that don't involve them: the room's bottom-left corner.
pub(super) const FAR_PLAYER: Vec2 = Vec2::new(-240.0, -120.0);

pub(super) fn room_with_flies(values: &[f32]) -> Enemies {
    let layout = RoomLayout {
        cells: values
            .iter()
            .enumerate()
            .map(|(index, &value)| LayoutCell {
                column: 2 + u32::try_from(index).expect("a few markers") * 3,
                row: 3,
                value,
            })
            .collect(),
        ..RoomLayout::default()
    };
    Enemies::from_layout(&layout, &parameters(), &mut Random::with_seed(1))
}

fn run(enemies: &mut Enemies, frames: u32, random: &mut Random) {
    run_with_player(enemies, frames, FAR_PLAYER, random);
}

pub(super) fn run_with_player(enemies: &mut Enemies, frames: u32, player: Vec2, random: &mut Random) {
    for _ in 0..frames {
        enemies.advance(ONE_FRAME, &[], player, interior_bounds(), random, &mut Vec::new());
        enemies.remove_finished();
    }
}

pub(super) fn player_at(position: Vec2, velocity: Vec2) -> Body {
    Body {
        position,
        velocity,
        mass: GameParameters::ORIGINAL.pickups.player_mass,
    }
}

#[test]
fn fly_markers_place_flies_below_their_cells() {
    let enemies = room_with_flies(&[FLY_MARKER, FAINT_FLY_MARKER, -0.01, 1.0, ATTACK_FLY_MARKER]);
    assert_eq!(enemies.flies.len(), 3, "only the fly markers");
    let kinds: Vec<FlyKind> = enemies.flies.iter().map(|fly| fly.kind).collect();
    assert_eq!(kinds, [FlyKind::Black, FlyKind::Black, FlyKind::Attack]);
    assert!(enemies.spawned_any && enemies.any_alive());
    let cell_center = world_to_stage(tile_center(2, 3));
    assert_eq!(enemies.flies[0].position, cell_center + Vec2::new(0.0, SPAWN_DROP));
}

#[test]
fn flies_appear_then_wander_inside_the_room() {
    let mut enemies = room_with_flies(&[FLY_MARKER, FLY_MARKER]);
    let mut random = Random::with_seed(7);
    run(&mut enemies, 30, &mut random);
    assert_eq!(enemies.flies[0].state, FlyState::Appearing);
    run(&mut enemies, 30, &mut random);
    assert_eq!(enemies.flies[0].state, FlyState::Flying);
    let start = enemies.flies[0].position;
    let mut moved = false;
    for _ in 0..600 {
        run(&mut enemies, 1, &mut random);
        let fly = &enemies.flies[0];
        assert!(interior_bounds().contains(stage_to_world(fly.position)));
        moved |= fly.position.distance(start) > 5.0;
    }
    assert!(moved, "it wanders");
}

#[test]
fn a_tear_kills_a_fly_whose_death_then_plays_out() {
    let mut enemies = room_with_flies(&[FLY_MARKER]);
    let mut random = Random::with_seed(3);
    let mut sounds = Vec::new();
    let fly_point = stage_to_world(enemies.flies[0].position);
    let damage = GameParameters::ORIGINAL.tear.base_damage;
    assert!(!enemies.hit_by_tear(
        fly_point + Vec2::new(60.0, 0.0),
        Vec2::X,
        damage,
        ONE_FRAME,
        &mut random,
        &mut sounds
    ));
    assert!(enemies.hit_by_tear(
        fly_point + Vec2::new(5.0, 0.0),
        Vec2::X,
        damage,
        ONE_FRAME,
        &mut random,
        &mut sounds
    ));
    assert_eq!(sounds, [SoundEvent::EnemyDied], "3.5 damage beats 3 hit points");
    assert!(!enemies.any_alive());
    run(&mut enemies, 20, &mut random);
    assert!(enemies.flies.is_empty(), "removed once its death has played");
}

#[test]
fn bumping_a_fly_pushes_both_but_does_not_hurt() {
    let mut enemies = room_with_flies(&[FLY_MARKER]);
    let fly_point = stage_to_world(enemies.flies[0].position);
    let player = player_at(fly_point - Vec2::new(20.0, 0.0), Vec2::new(3.0, 0.0));
    let bump = enemies.bump(player, false, ONE_FRAME, &mut Random::with_seed(1), &mut Vec::new());
    assert!(bump.contact.first_offset.x < 0.0, "the player is pushed back");
    assert_eq!(bump.hurt, None);
    assert!(enemies.any_alive());
}

#[test]
fn an_attack_fly_chases_the_player_and_hurts_on_contact() {
    let mut enemies = room_with_flies(&[ATTACK_FLY_MARKER]);
    let mut random = Random::with_seed(5);
    let start = stage_to_world(enemies.flies[0].position);
    let player = start + Vec2::new(200.0, -80.0);
    run_with_player(&mut enemies, 26 + 90, player, &mut random);
    let fly_point = stage_to_world(enemies.flies[0].position);
    assert!(
        distance(fly_point, player) < distance(start, player) / 2.0,
        "it closes in: {fly_point} vs {player}"
    );
    let bump = enemies.bump(
        player_at(fly_point - Vec2::new(10.0, 0.0), Vec2::ZERO),
        false,
        ONE_FRAME,
        &mut Random::with_seed(1),
        &mut Vec::new(),
    );
    assert_eq!(
        bump.hurt,
        Some((parameters().flies.attack_damage, Killer::Enemy(FlyKind::Attack)))
    );
}

#[test]
fn an_attack_fly_takes_two_tears() {
    let mut enemies = room_with_flies(&[ATTACK_FLY_MARKER]);
    let mut sounds = Vec::new();
    let damage = GameParameters::ORIGINAL.tear.base_damage;
    let hit = |enemies: &mut Enemies, sounds: &mut Vec<SoundEvent>| {
        let fly_point = stage_to_world(enemies.flies[0].position);
        enemies.hit_by_tear(fly_point, Vec2::X, damage, ONE_FRAME, &mut Random::with_seed(1), sounds)
    };
    assert!(hit(&mut enemies, &mut sounds));
    assert!(enemies.any_alive(), "5 hit points outlast one tear");
    assert!(hit(&mut enemies, &mut sounds));
    assert_eq!(sounds, [SoundEvent::EnemyDied]);
}

#[test]
fn riding_the_unicorn_kills_flies_on_contact() {
    let mut enemies = room_with_flies(&[FLY_MARKER]);
    let mut sounds = Vec::new();
    let fly_point = stage_to_world(enemies.flies[0].position);
    enemies.bump(
        player_at(fly_point - Vec2::new(10.0, 0.0), Vec2::ZERO),
        true,
        ONE_FRAME,
        &mut Random::with_seed(1),
        &mut sounds,
    );
    assert!(!enemies.any_alive());
    assert_eq!(sounds, [SoundEvent::EnemyDied]);
}

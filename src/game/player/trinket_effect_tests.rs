//! What the trinkets do when Isaac is hurt, enemies die and rooms are cleared or entered.

use glam::Vec2;

use crate::game::floor::RoomEntry;
use crate::game::floor::plan::RoomRole;
use crate::game::original_units::{Frames, Hearts};
use crate::game::parameters::GameParameters;
use crate::game::pickups::kind::{HeartKind, PickupKind};
use crate::game::pickups::trinket::Trinket;
use crate::game::player::death::Killer;
use crate::game::sound_event::SoundEvent;
use crate::game::{Game, InputState, RoomAppearance, RoomLayouts};

const ONE_FRAME: Frames = Frames(1.0);
const MANY_TRIES: u32 = 500;

fn game_with(trinket: Trinket) -> Game {
    let mut game = Game::new(
        RoomAppearance::Placeholder,
        &GameParameters::ORIGINAL,
        &RoomLayouts::default(),
        1,
    );
    for _ in 0..40 {
        game.advance(&InputState::default(), ONE_FRAME);
    }
    game.inventory.take_trinket(trinket);
    game
}

/// A hit of half a heart from full health, with the blink after the last one over.
fn hit(game: &mut Game) {
    game.inventory.heal_fully();
    game.inventory.advance(Frames(1000.0));
    game.hurt_player(Hearts::HALF, Killer::Fire);
}

#[test]
/// DoAction 4781: each hit brings a blue fly.
fn the_fish_head_gives_a_blue_fly_for_every_hit() {
    let mut game = game_with(Trinket::FishHead);
    hit(&mut game);
    hit(&mut game);
    for _ in 0..20 {
        game.advance(&InputState::default(), ONE_FRAME);
    }
    assert_eq!(game.blue_flies.count(), 2);
}

#[test]
/// DoAction 4810 and 220: hits sometimes add to the room's damage, until the room is left.
fn the_red_patch_raises_damage_for_the_room() {
    let mut game = game_with(Trinket::RedPatch);
    let bare = game.player_stats().damage;
    for _ in 0..MANY_TRIES {
        hit(&mut game);
    }
    assert!(game.cards.boosts.razor >= game.parameters.trinkets.red_patch_points);
    assert!(game.player_stats().damage > bare);
    game.end_room_boosts();
    assert_eq!(game.player_stats().damage, bare);
}

#[test]
/// DoAction 4814: now and then a hit plays the death animation and hurts every enemy.
fn a_missing_page_sometimes_plays_the_death_animation() {
    let mut game = game_with(Trinket::AMissingPage);
    let mut played = false;
    for _ in 0..MANY_TRIES {
        hit(&mut game);
        played |= game.pending_sound_events.contains(&SoundEvent::DeathCard);
    }
    assert!(played);
}

#[test]
/// DoAction 4873: a hit that leaves half a heart or less gives the Polaroid's bubble, not a hit that
/// leaves more.
fn the_polaroid_shields_isaac_at_half_a_heart() {
    let mut game = game_with(Trinket::ThePolaroid);
    hit(&mut game);
    assert!(!game.inventory.is_shielded());
    game.inventory.advance(Frames(1000.0));
    game.inventory.red_hearts = Hearts::ONE;
    game.hurt_player(Hearts::HALF, Killer::Fire);
    assert!(game.inventory.is_shielded());
    let hearts = game.inventory.red_hearts;
    // The blink after the hit is over in a few frames; the bubble lasts far longer.
    game.inventory.advance(Frames(40.0));
    game.hurt_player(Hearts::HALF, Killer::Fire);
    assert_eq!(game.inventory.red_hearts, hearts, "the bubble takes the hit");
}

#[test]
/// DoAction 4877: at half a heart with no soul hearts, the Cursed Skull teleports Isaac away.
fn the_cursed_skull_teleports_isaac_away_when_he_is_nearly_dead() {
    let mut game = game_with(Trinket::CursedSkull);
    hit(&mut game);
    assert!(game.cards.teleport.is_none());
    game.inventory.advance(Frames(1000.0));
    game.inventory.red_hearts = Hearts::ONE;
    game.hurt_player(Hearts::HALF, Killer::Fire);
    assert!(game.cards.teleport.is_some());
}

#[test]
/// DoAction 4411–4419: kills raise the rage, up to a cap.
fn samsons_lock_raises_the_rage_with_kills() {
    let mut game = game_with(Trinket::SamsonsLock);
    let bare = game.player_stats().damage;
    game.rage_for_kills(1000);
    let amounts = game.parameters.trinkets;
    assert!((game.cards.boosts.rage - amounts.rage_cap).abs() < f32::EPSILON);
    assert!(game.player_stats().damage > bare);
}

#[test]
/// DoAction 32842: a heart for entering the boss room the first time.
fn the_tick_heals_on_first_entering_the_boss_room() {
    let mut game = game_with(Trinket::Tick);
    game.inventory.red_hearts = Hearts::ONE;
    let entry = |role, first_visit| RoomEntry {
        position: Vec2::ZERO,
        role,
        first_visit,
        jingle: None,
    };
    game.trinkets_welcome(&entry(RoomRole::Normal, true));
    game.trinkets_welcome(&entry(RoomRole::Boss, false));
    assert_eq!(game.inventory.red_hearts, Hearts::ONE);
    game.trinkets_welcome(&entry(RoomRole::Boss, true));
    assert_eq!(game.inventory.red_hearts, Hearts::ONE + Hearts::ONE);
}

#[test]
/// DoAction 30873: clearing a room sometimes heals half a heart.
fn isaacs_fork_sometimes_heals_when_a_room_is_cleared() {
    let mut game = game_with(Trinket::IsaacsFork);
    game.inventory.red_hearts = Hearts::HALF;
    for _ in 0..MANY_TRIES {
        game.trinkets_react_to_clearing();
    }
    assert!(game.inventory.red_hearts > Hearts::HALF);
}

#[test]
/// DoAction 3308: a heart made is a soul heart now and then.
fn moms_pearl_turns_some_hearts_into_soul_hearts() {
    let mut game = game_with(Trinket::MomsPearl);
    let luck = game.pickup_luck();
    let room = game.floor.current_room_mut();
    room.pickups.luck = luck;
    let mut random = crate::game::random::Random::with_seed(5);
    for _ in 0..MANY_TRIES {
        room.spawn_pickup(PickupKind::Heart(HeartKind::Full), Vec2::ZERO, false, &mut random);
    }
    let souls = room
        .pickups
        .lying_untaken()
        .filter(|kind| *kind == PickupKind::Heart(HeartKind::Soul))
        .count();
    assert!(souls > 10, "{souls}");
}

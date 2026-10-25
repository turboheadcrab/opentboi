use crate::game::original_units::Hearts;
use glam::Vec2;

use super::*;
use crate::game::parameters::GameParameters;
use crate::game::room::enemies::{EnemyParameters, Walker};
use crate::game::room::layout::RoomLayouts;
use crate::game::room::world_to_stage;
use crate::game::texture_id::FlyKind;
use crate::game::{RoomAppearance, TextureId};

const ONE_FRAME: Frames = Frames(1.0);
/// Long enough for the invulnerability after a hit to wear off.
const RECOVERY_FRAMES: u32 = 100;
/// Frames of his death that the blink of the fatal hit would have hidden some of.
/// Long enough for a Gaper to finish appearing and start pressing against Isaac.
const WALKER_APPEARING_FRAMES: u32 = 90;
const DYING_FRAMES_CHECKED: u32 = 60;

fn game() -> Game {
    Game::new(
        RoomAppearance::Placeholder,
        &GameParameters::ORIGINAL,
        &RoomLayouts::default(),
        1,
    )
}

fn run(game: &mut Game, frames: u32, input: &InputState) -> Vec<SoundEvent> {
    let mut sounds = Vec::new();
    for _ in 0..frames {
        game.advance(input, ONE_FRAME);
        sounds.extend(game.drain_sound_events());
    }
    sounds
}

/// Hurts Isaac until he dies, by `killer`.
fn kill(game: &mut Game, killer: Killer) {
    while !game.is_dying() {
        game.hurt_player(Hearts::ONE, killer);
        run(game, RECOVERY_FRAMES, &InputState::default());
    }
}

fn frames_until_note(game: &Game) -> u32 {
    game.note_shows_after().whole()
}

#[test]
fn isaac_dies_slowly_then_the_note_names_his_killer() {
    let mut game = game();
    game.hurt_player(Hearts::ONE, Killer::Enemy(FlyKind::Attack));
    game.hurt_player(Hearts::ONE, Killer::Enemy(FlyKind::Attack));
    assert!(!game.is_dying(), "invulnerable right after a hit");
    kill(&mut game, Killer::Enemy(FlyKind::Attack));
    assert_eq!(
        game.death.map(|death| death.killer),
        Some(Killer::Enemy(FlyKind::Attack))
    );

    let mut game = game_dying_now(Killer::Enemy(FlyKind::Attack));
    let note_after = frames_until_note(&game);
    let sounds = run(&mut game, note_after - 1, &InputState::default());
    assert_eq!(
        sounds.iter().filter(|&&sound| sound == SoundEvent::PlayerDied).count(),
        1,
        "isaacdies.wav on the clip's 32nd frame"
    );
    assert_eq!(game.death_note(), None);
    run(&mut game, 1, &InputState::default());
    assert_eq!(game.death_note(), Some(Killer::Enemy(FlyKind::Attack)));
    let mut draw_items = Vec::new();
    game.fill_draw_items(&mut draw_items);
    assert!(
        draw_items
            .iter()
            .any(|item| item.texture == Some(TextureId::DeathNote(Killer::Enemy(FlyKind::Attack))))
    );
}

/// A game whose Isaac has just died.
fn game_dying_now(killer: Killer) -> Game {
    let mut game = game();
    game.die(killer);
    game
}

#[test]
fn dying_isaac_ignores_the_controls_and_slides_to_a_stop() {
    let mut game = game_dying_now(Killer::Spikes);
    let start = game.player.position();
    let walk_and_shoot = InputState {
        place_bomb: true,
        ..InputState::from_raw_axes(Vec2::X, Vec2::Y)
    };
    let sounds = run(&mut game, 30, &walk_and_shoot);
    assert_eq!(game.player.position(), start);
    assert!(game.tears.is_empty() && !sounds.contains(&SoundEvent::BombPlaced));
    game.hurt_player(Hearts::ONE, Killer::Fire);
    assert_eq!(
        game.death.map(|death| death.killer),
        Some(Killer::Spikes),
        "not hurt again"
    );
}

#[test]
fn the_note_stops_the_game_until_replay_is_pressed() {
    let mut game = game_dying_now(Killer::Bomb);
    let replay = InputState {
        replay: true,
        ..InputState::default()
    };
    // Held from before the note: not an answer.
    let note_after = frames_until_note(&game);
    run(&mut game, note_after + 5, &replay);
    assert!(game.death_note().is_some());
    run(&mut game, 1, &InputState::default());
    let seed = game.seed;
    run(&mut game, 1, &replay);
    assert!(!game.is_dying(), "a new run");
    assert_ne!(game.seed, seed);
}

#[test]
fn isaac_stays_shown_while_dying_despite_the_blink_of_the_fatal_hit() {
    let mut game = game();
    loop {
        game.hurt_player(Hearts::ONE, Killer::Fire);
        if game.is_dying() {
            break;
        }
        run(&mut game, RECOVERY_FRAMES, &InputState::default());
    }
    for _ in 0..DYING_FRAMES_CHECKED {
        game.advance(&InputState::default(), ONE_FRAME);
        let mut draw_items = Vec::new();
        game.fill_room_draw_items(&mut draw_items);
        assert!(
            draw_items
                .iter()
                .any(|item| matches!(item.texture, Some(TextureId::IsaacAnimation { .. }))),
            "Isaac is drawn on every frame of his death"
        );
    }
}

#[test]
fn a_dying_isaac_is_not_pushed_about_by_what_crowds_him() {
    let mut game = game_dying_now(Killer::Spikes);
    run(&mut game, DYING_FRAMES_CHECKED, &InputState::default());
    let resting_place = game.player.position();
    let parameters = EnemyParameters::from_game(&GameParameters::ORIGINAL);
    let crowding = Walker::gaper(
        world_to_stage(resting_place + Vec2::new(5.0, 0.0)),
        false,
        0,
        &parameters,
        &mut game.random,
    );
    game.floor.current_room_mut().enemies.add_walker(crowding);
    run(&mut game, WALKER_APPEARING_FRAMES, &InputState::default());
    assert_eq!(game.player.position(), resting_place);
}

/// The black cover over the stage, if there is one: its opacity.
fn black_cover(game: &Game) -> Option<f32> {
    let mut items = Vec::new();
    game.fill_draw_items(&mut items);
    items
        .iter()
        .find(|item| item.texture.is_none() && item.size == crate::game::room::STAGE_SIZE)
        .map(|item| item.color.alpha)
}

#[test]
fn a_new_run_comes_up_out_of_black() {
    let mut game = game();
    assert_eq!(black_cover(&game), None, "nothing covers the first run");
    game.restart();
    assert_eq!(black_cover(&game), Some(1.0));
    run(&mut game, 7, &InputState::default());
    let after_seven = black_cover(&game).expect("still fading");
    assert!((after_seven - 0.51).abs() < 0.01, "7 % a frame: {after_seven}");
    run(&mut game, 8, &InputState::default());
    assert_eq!(black_cover(&game), None, "gone after 15 frames");
}

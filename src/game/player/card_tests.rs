use glam::Vec2;

use crate::game::floor::FloorCell;
use crate::game::floor::plan::RoomRole;
use crate::game::original_units::{Frames, Hearts};
use crate::game::pickups::card::Card;
use crate::game::pickups::consumable::Consumable;
use crate::game::pickups::kind::{CoinKind, PickupKind};
use crate::game::{Game, GameParameters, InputState, RoomAppearance, RoomLayouts, SoundEvent, TextureId};

const ONE_FRAME: Frames = Frames(1.0);

fn game() -> Game {
    let mut game = Game::new(
        RoomAppearance::Placeholder,
        &GameParameters::ORIGINAL,
        &RoomLayouts::default(),
        1,
    );
    for _ in 0..40 {
        game.advance(&InputState::default(), ONE_FRAME);
    }
    game
}

fn use_card(game: &mut Game, card: Card) {
    game.inventory.held = Some(Consumable::Card(card));
    let use_it = InputState {
        use_card: true,
        ..InputState::default()
    };
    game.advance(&use_it, ONE_FRAME);
}

#[test]
/// `pillc()`: the card taken becomes the one held, and touching it keeps it blocked.
fn a_card_swapped_for_another_is_not_taken_back_while_isaac_stands_on_it() {
    let mut game = game();
    game.inventory.held = Some(Consumable::Card(Card::TheSun));
    let position = game.player.position();
    let pickups = &mut game.floor.current_room_mut().pickups;
    pickups.place(PickupKind::Card(Card::TheFool), position);
    for _ in 0..60 {
        game.advance(&InputState::default(), ONE_FRAME);
    }
    assert_eq!(
        game.inventory.held,
        Some(Consumable::Card(Card::TheFool)),
        "the old card is not taken back"
    );
    let lying = lying_except_the_start_items(&game);
    assert_eq!(
        lying,
        [PickupKind::Card(Card::TheSun)],
        "the same pickup, now the old card"
    );
}

#[test]
fn holding_the_button_uses_one_card() {
    let mut game = game();
    use_card(&mut game, Card::TwoOfClubs);
    let bombs = game.inventory.bombs;
    use_card(&mut game, Card::TwoOfClubs);
    assert_eq!(game.inventory.bombs, bombs, "still held: no second use");
}

#[test]
fn the_moon_leaves_a_way_out_of_the_secret_room() {
    let mut game = game();
    let secret = game.floor.cell_with(RoomRole::Secret).expect("a secret room");
    let entry = game.floor.teleport_to(secret, Vec2::ZERO);
    game.settle_into(&entry);
    assert!(game.floor.open_doors_count() > 0);
}

#[test]
fn the_two_of_clubs_doubles_bombs_and_the_card_is_gone() {
    let mut game = game();
    game.inventory.bombs = 3;
    use_card(&mut game, Card::TwoOfClubs);
    assert_eq!(game.inventory.bombs, 6);
    assert_eq!(game.inventory.held, None);
}

#[test]
fn the_lovers_put_two_hearts_beside_isaac() {
    let mut game = game();
    use_card(&mut game, Card::TheLovers);
    let hearts = lying_except_the_start_items(&game);
    assert_eq!(hearts.len(), 2, "{hearts:?}");
}

#[test]
fn the_fool_teleports_to_the_start_room_after_the_animation() {
    let mut game = game();
    game.floor.teleport_to(game.floor.cells()[1], Vec2::ZERO);
    use_card(&mut game, Card::TheFool);
    assert!(
        game.drain_sound_events()
            .any(|sound| sound == SoundEvent::TeleportStarted)
    );
    for _ in 0..20 {
        game.advance(&InputState::default(), ONE_FRAME);
    }
    assert_eq!(game.floor.current_cell(), FloorCell::START);
}

/// DoAction 4690: a boss rush refuses every teleport.
#[test]
fn a_boss_rush_refuses_to_teleport() {
    let mut game = game();
    game.floor.teleport_to(game.floor.cells()[1], Vec2::ZERO);
    game.floor.make_current_room(RoomRole::BossRush);
    use_card(&mut game, Card::TheFool);
    assert!(game.cards.teleport.is_none());
    assert!(
        !game
            .drain_sound_events()
            .any(|sound| sound == SoundEvent::TeleportStarted)
    );
}

#[test]
fn the_empress_gives_isaac_the_demonic_look() {
    let mut game = game();
    let demonic_parts = |game: &Game| {
        let mut draw_items = Vec::new();
        game.player.append_draw_items(&mut draw_items);
        draw_items
            .iter()
            .filter_map(|item| match item.texture {
                Some(TextureId::Isaac { part, .. }) => Some(part.is_demonic()),
                _ => None,
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(demonic_parts(&game), [false, false]);
    use_card(&mut game, Card::TheEmpress);
    // Let the card's animation play out so Isaac is drawn as himself again.
    for _ in 0..200 {
        game.advance(&InputState::default(), ONE_FRAME);
    }
    assert_eq!(demonic_parts(&game), [true, true], "body and head");
}

#[test]
fn the_devil_raises_damage_for_the_room() {
    let mut game = game();
    let before = game.player_stats().damage;
    use_card(&mut game, Card::TheDevil);
    let after = game.player_stats().damage;
    assert!(
        (after.get() - 3.5 * 1.9_f32.sqrt()).abs() < 1e-4 && after > before,
        "{before:?} → {after:?}"
    );
}

#[test]
fn the_devil_darkens_the_tears_for_the_room() {
    let mut game = game();
    assert!(!game.player_stats().dark_tears);
    use_card(&mut game, Card::TheDevil);
    assert!(game.player_stats().dark_tears);
}

#[test]
fn strength_gives_a_heart_container_until_isaac_leaves_the_room() {
    let mut game = game();
    let containers = game.inventory.maximum_red_hearts;
    use_card(&mut game, Card::Strength);
    assert_eq!(game.inventory.maximum_red_hearts, containers + Hearts::ONE);
    assert!(game.player_stats().scale > 1.0);
    let entry = game.floor.teleport_to(FloorCell::START, Vec2::ZERO);
    game.settle_into(&entry);
    assert_eq!(game.inventory.maximum_red_hearts, containers);
}

#[test]
fn the_hanged_man_flies() {
    let mut game = game();
    use_card(&mut game, Card::TheHangedMan);
    assert!(game.player_stats().flying);
}

#[test]
fn summoned_machines_appear_away_from_isaac() {
    let mut game = game();
    use_card(&mut game, Card::WheelOfFortune);
    let position = game.player.position();
    let machine = game
        .floor
        .current_room()
        .machines
        .positions()
        .next()
        .expect("a slot machine");
    assert!(machine.distance(position) > 30.0, "{machine:?} vs {position:?}");
}

/// What lies in the start room besides its item pedestals and the dime for its fortune teller.
fn lying_except_the_start_items(game: &Game) -> Vec<PickupKind> {
    let lying = game.floor.current_room().pickups.lying_kinds();
    lying
        .into_iter()
        .filter(|kind| !kind.is_pedestal() && *kind != PickupKind::Coin(CoinKind::Dime))
        .collect()
}

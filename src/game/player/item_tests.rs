use glam::Vec2;

use crate::game::machines::rolls::slot_result;
use crate::game::original_units::Frames;
use crate::game::pickups::item::Item;
use crate::game::pickups::kind::PickupKind;
use crate::game::random::Random;
use crate::game::room::stage_to_world;
use crate::game::{Game, GameParameters, InputState, RoomAppearance, RoomLayouts, SoundEvent};

const ONE_FRAME: Frames = Frames(1.0);
const ROLLS: u32 = 5000;
/// Where the start room's pedestals stand (stage pixels).
const LUCKY_FOOT_STAGE_POINT: Vec2 = Vec2::new(270.0, 190.0);
const UNICORN_STAGE_POINT: Vec2 = Vec2::new(370.0, 190.0);

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

#[test]
fn the_start_room_offers_the_lucky_foot_and_taking_it_leaves_the_pedestal_empty() {
    let mut game = game();
    assert!(
        game.floor
            .current_room()
            .pickups
            .lying_kinds()
            .contains(&PickupKind::Item(Item::LuckyFoot))
    );
    game.player.place_at(stage_to_world(LUCKY_FOOT_STAGE_POINT));
    game.drain_sound_events().for_each(drop);
    for _ in 0..10 {
        game.advance(&InputState::default(), ONE_FRAME);
    }
    assert!(game.inventory.has(Item::LuckyFoot));
    let lying = game.floor.current_room().pickups.lying_kinds();
    assert!(lying.contains(&PickupKind::ItemPedestal), "{lying:?}");
    assert!(!lying.contains(&PickupKind::Item(Item::LuckyFoot)), "{lying:?}");
    assert!(game.drain_sound_events().any(|sound| sound == SoundEvent::ItemTaken));
}

#[test]
fn the_unicorn_is_used_once_charged_and_rides_without_tears() {
    let mut game = game();
    game.player.place_at(stage_to_world(UNICORN_STAGE_POINT));
    for _ in 0..10 {
        game.advance(&InputState::default(), ONE_FRAME);
    }
    assert_eq!(game.inventory.active.held(), Some(Item::MyLittleUnicorn));
    assert!(!game.inventory.has(Item::LuckyFoot), "only the unicorn was taken");
    game.drain_sound_events().for_each(drop);
    let use_item = InputState {
        use_item: true,
        ..InputState::default()
    };
    game.advance(&use_item, ONE_FRAME);
    assert!(game.inventory.rides_unicorn());
    assert_eq!(game.inventory.active.charge(), Some((0, 6)));
    assert!(game.drain_sound_events().any(|sound| sound == SoundEvent::ChariotTune));
    let shooting = InputState {
        shoot_axes: Vec2::X,
        use_item: true,
        ..InputState::default()
    };
    for _ in 0..30 {
        game.advance(&shooting, ONE_FRAME);
    }
    assert!(game.tears.is_empty(), "no tears on the unicorn");
    assert!(game.player_stats().speed > GameParameters::ORIGINAL.movement.speed_stat);
    for _ in 0..160 {
        game.advance(&InputState::default(), ONE_FRAME);
    }
    assert!(!game.inventory.rides_unicorn(), "the ride lasts 180 frames");
}

#[test]
/// `pillc(3)` from the Deck of Cards: a random card is taken as if picked up, and the one held
/// lies where Isaac stands.
fn the_deck_of_cards_gives_a_card_and_puts_down_the_one_held() {
    use crate::game::pickups::card::Card;
    use crate::game::pickups::consumable::Consumable;
    use crate::game::pickups::item::ItemNumber;
    let mut game = game();
    game.inventory.take_item(Item::DeckOfCards);
    game.inventory.take(PickupKind::Card(Card::TheFool));
    game.drain_sound_events().for_each(drop);
    let use_item = InputState {
        use_item: true,
        ..InputState::default()
    };
    game.advance(&use_item, ONE_FRAME);
    assert!(matches!(game.inventory.held, Some(Consumable::Card(_))));
    assert_eq!(game.inventory.active.charge(), Some((0, 6)));
    let lying = game.floor.current_room().pickups.lying_kinds();
    assert!(lying.contains(&PickupKind::Card(Card::TheFool)), "{lying:?}");
    assert!(game.progress.has_collected(ItemNumber(61)));
    assert!(game.drain_sound_events().any(|sound| sound == SoundEvent::CardPickedUp));
}

#[test]
fn the_chariot_holds_up_the_unicorn() {
    use crate::game::pickups::card::Card;
    use crate::game::player::animation::Held;
    assert_eq!(Card::TheChariot.held_picture(), Held::Item(Item::MyLittleUnicorn));
    assert_eq!(Card::TheSun.held_picture(), Held::Card(Card::TheSun));
}

#[test]
fn the_lucky_foot_leaves_out_the_last_slot_results() {
    let mut random = Random::with_seed(5);
    let highest = |random: &mut Random, lucky_foot| {
        (0..ROLLS)
            .map(|_| slot_result(random, lucky_foot, false))
            .max()
            .unwrap_or(0)
    };
    assert_eq!(highest(&mut random, false), 24, "3 + 20, moved on by one");
    assert_eq!(highest(&mut random, true), 21, "3 + 17, moved on by one");
}

#[test]
fn the_portable_slot_always_moves_high_results_on() {
    let mut random = Random::with_seed(9);
    let results: Vec<u8> = (0..ROLLS).map(|_| slot_result(&mut random, false, true)).collect();
    assert!(!results.contains(&9), "9 always becomes 10");
}

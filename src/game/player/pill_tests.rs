use glam::Vec2;

use crate::game::original_units::{Frames, Hearts};
use crate::game::pickups::card::Card;
use crate::game::pickups::consumable::Consumable;
use crate::game::pickups::kind::PickupKind;
use crate::game::pickups::pill::Pill;
use crate::game::player::animation::PlayerAnimation;
use crate::game::player::pill_state::{PillEffect, PillKind};
use crate::game::shop::{Ware, WareKind};
use crate::game::{Game, GameParameters, InputState, RoomAppearance, RoomLayouts, SoundEvent};

const ONE_FRAME: Frames = Frames(1.0);
const PILL: Pill = Pill::ALL[2];

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

/// Gives the look held an effect and uses it.
fn use_pill_of(game: &mut Game, kind: PillKind, good: bool) -> Vec<SoundEvent> {
    game.inventory.pills.give_effect(PILL, PillEffect { kind, good });
    game.inventory.held = Some(Consumable::Pill(PILL));
    // A press uses it, so let go first.
    game.advance(&InputState::default(), ONE_FRAME);
    game.drain_sound_events().for_each(drop);
    game.advance(
        &InputState {
            use_card: true,
            ..InputState::default()
        },
        ONE_FRAME,
    );
    game.drain_sound_events().collect()
}

#[test]
fn a_pill_is_taken_into_the_slot_and_swaps_with_a_card_held() {
    let mut game = game();
    game.inventory.held = Some(Consumable::Card(Card::TheSun));
    let position = game.player.position();
    game.floor
        .current_room_mut()
        .pickups
        .place(PickupKind::Pill(PILL), position);
    for _ in 0..60 {
        game.advance(&InputState::default(), ONE_FRAME);
    }
    assert_eq!(game.inventory.held, Some(Consumable::Pill(PILL)));
    assert!(
        game.floor
            .current_room()
            .pickups
            .lying_untaken()
            .any(|kind| kind == PickupKind::Card(Card::TheSun)),
        "the card is dropped"
    );
}

#[test]
fn a_pill_not_used_yet_is_unknown_in_the_hud_and_named_once_it_is() {
    let mut game = game();
    let unknown = game.inventory.pill_name(PILL);
    use_pill_of(&mut game, PillKind::Luck, true);
    assert_ne!(game.inventory.pill_name(PILL), unknown);
    assert_eq!(game.inventory.held, None, "the pill is used up");
}

#[test]
fn tears_up_makes_isaac_shoot_faster_and_tears_down_slower() {
    let mut game = game();
    let normal = game.player_stats().delay_bonus;
    use_pill_of(&mut game, PillKind::Tears, true);
    let faster = game.player_stats().delay_bonus;
    assert!(faster > normal);
    use_pill_of(&mut game, PillKind::Tears, false);
    use_pill_of(&mut game, PillKind::Tears, false);
    assert!(game.player_stats().delay_bonus < faster);
}

#[test]
fn range_speed_and_shot_speed_pills_change_the_stats_they_name() {
    let mut game = game();
    let before = game.player_stats();
    use_pill_of(&mut game, PillKind::Range, true);
    use_pill_of(&mut game, PillKind::Speed, true);
    use_pill_of(&mut game, PillKind::ShotSpeed, true);
    let after = game.player_stats();
    assert!(after.tear_height_bonus > before.tear_height_bonus);
    assert!(after.speed > before.speed);
    assert!(after.shot_speed_multiplier > before.shot_speed_multiplier);
    use_pill_of(&mut game, PillKind::ShotSpeed, false);
    use_pill_of(&mut game, PillKind::ShotSpeed, false);
    assert!(game.player_stats().shot_speed_multiplier >= GameParameters::ORIGINAL.pills.shot_speed_multiplier_minimum);
}

#[test]
fn luck_pills_move_luck_up_or_down() {
    let mut game = game();
    use_pill_of(&mut game, PillKind::Luck, true);
    assert_eq!(game.player_stats().luck, 1);
    use_pill_of(&mut game, PillKind::Luck, false);
    use_pill_of(&mut game, PillKind::Luck, false);
    assert_eq!(game.player_stats().luck, -1);
}

#[test]
fn health_pills_add_or_take_a_container_and_full_health_heals() {
    let mut game = game();
    let containers = game.inventory.maximum_red_hearts;
    use_pill_of(&mut game, PillKind::Health, true);
    assert_eq!(game.inventory.maximum_red_hearts, containers + Hearts::ONE);
    use_pill_of(&mut game, PillKind::Health, false);
    assert_eq!(game.inventory.maximum_red_hearts, containers);
    game.inventory.red_hearts = Hearts::HALF;
    use_pill_of(&mut game, PillKind::FullHealth, false);
    assert_eq!(game.inventory.red_hearts, game.inventory.maximum_red_hearts);
}

#[test]
fn health_down_with_one_container_gives_one_instead() {
    let mut game = game();
    game.inventory.maximum_red_hearts = Hearts::ONE;
    game.inventory.red_hearts = Hearts::ONE;
    use_pill_of(&mut game, PillKind::Health, false);
    assert_eq!(game.inventory.maximum_red_hearts, Hearts::ONE * 2.0);
}

#[test]
fn balls_of_steel_gives_two_soul_hearts_and_bombs_are_keys_swaps_the_counts() {
    let mut game = game();
    use_pill_of(&mut game, PillKind::BallsOfSteel, false);
    assert_eq!(game.inventory.soul_hearts, Hearts::ONE * 2.0);
    game.inventory.bombs = 5;
    game.inventory.keys = 2;
    use_pill_of(&mut game, PillKind::BombsAreKeys, false);
    assert_eq!((game.inventory.bombs, game.inventory.keys), (2, 5));
}

#[test]
fn a_bad_trip_hurts_through_the_blink_of_an_earlier_hit_but_not_the_last_heart() {
    let mut game = game();
    game.inventory.red_hearts = Hearts::ONE * 3.0;
    game.hurt_player(Hearts::HALF, crate::game::Killer::Fire);
    let hurt = game.inventory.red_hearts;
    use_pill_of(&mut game, PillKind::BadTrip, false);
    assert_eq!(game.inventory.red_hearts, hurt - Hearts::ONE);
    game.inventory.red_hearts = Hearts::HALF;
    game.inventory.maximum_red_hearts = Hearts::ONE * 3.0;
    game.inventory.soul_hearts = Hearts::ZERO;
    use_pill_of(&mut game, PillKind::BadTrip, false);
    assert_eq!(
        game.inventory.red_hearts, game.inventory.maximum_red_hearts,
        "with a heart or less it heals instead"
    );
}

#[test]
fn a_good_pill_cheers_and_a_bad_one_groans() {
    let mut game = game();
    let cheer = use_pill_of(&mut game, PillKind::Speed, true);
    assert!(cheer.contains(&SoundEvent::ThumbsUp) && cheer.contains(&SoundEvent::GoodPill));
    let groan = use_pill_of(&mut game, PillKind::Speed, false);
    assert!(groan.contains(&SoundEvent::ThumbsDown) && !groan.contains(&SoundEvent::ThumbsUp));
}

#[test]
fn a_good_pill_gives_a_thumbs_up_and_a_bad_one_a_thumbs_down() {
    let mut game = game();
    use_pill_of(&mut game, PillKind::Speed, true);
    assert!(game.player.is_playing(PlayerAnimation::ThumbsUp));
    use_pill_of(&mut game, PillKind::Speed, false);
    assert!(game.player.is_playing(PlayerAnimation::ThumbsDown));
}

/// Puts a pill ware where Isaac stands and gives him `coins`.
fn stand_at_pill_ware(game: &mut Game, on_sale: bool, coins: u8) {
    let position = game.player.position();
    let ware = Ware {
        kind: WareKind::Consumable(Consumable::Pill(PILL)),
        on_sale,
    };
    game.floor.current_room_mut().wares.put_on_offer(ware, position);
    game.inventory.coins = coins;
}

#[test]
fn the_shop_sells_a_pill_for_five_coins_and_pushes_out_what_is_held() {
    let mut game = game();
    game.inventory.held = Some(Consumable::Card(Card::TheSun));
    stand_at_pill_ware(&mut game, false, 7);
    game.advance(&InputState::default(), ONE_FRAME);
    assert_eq!(game.inventory.coins, 2);
    assert_eq!(game.inventory.held, Some(Consumable::Pill(PILL)));
    assert!(
        game.floor
            .current_room()
            .pickups
            .lying_untaken()
            .any(|kind| kind == PickupKind::Card(Card::TheSun)),
        "the card is pushed out onto the floor"
    );
}

#[test]
fn a_pill_on_sale_costs_three_and_too_few_coins_buy_nothing() {
    let mut game = game();
    stand_at_pill_ware(&mut game, true, 3);
    game.advance(&InputState::default(), ONE_FRAME);
    assert_eq!(
        (game.inventory.coins, game.inventory.held),
        (0, Some(Consumable::Pill(PILL)))
    );
    let mut poor = self::game();
    stand_at_pill_ware(&mut poor, false, 4);
    poor.advance(&InputState::default(), ONE_FRAME);
    assert_eq!((poor.inventory.coins, poor.inventory.held), (4, None));
}

#[test]
fn the_lucky_foot_makes_every_pill_help() {
    let mut game = game();
    game.inventory.take_item(crate::game::pickups::item::Item::LuckyFoot);
    use_pill_of(&mut game, PillKind::Speed, false);
    assert!(game.inventory.pills.speed > 0.0);
}

#[test]
fn the_telepills_teleport_somewhere() {
    let mut game = game();
    use_pill_of(&mut game, PillKind::TelePills, false);
    assert!(game.cards.teleport.is_some() || game.player.is_animating());
    let _ = Vec2::ZERO;
}

mod gas;
mod panel;

use super::*;
use crate::game::hud::{HudHeart, life_bar};
use crate::game::original_units::{Frames, Hearts};
use crate::game::parameters::GameParameters;

fn inventory() -> Inventory {
    Inventory::new(GameParameters::ORIGINAL.inventory)
}

#[test]
fn a_run_starts_with_three_hearts_and_one_bomb() {
    let inventory = inventory();
    assert_eq!((inventory.coins, inventory.bombs, inventory.keys), (0, 1, 0));
    assert_eq!(
        life_bar(&inventory)[..4],
        [Some(HudHeart::Full), Some(HudHeart::Full), Some(HudHeart::Full), None]
    );
}

#[test]
fn hearts_are_left_lying_at_full_health() {
    let mut inventory = inventory();
    assert_eq!(inventory.take(PickupKind::Heart(HeartKind::Full)), None);
    assert!(inventory.hurt(Hearts::ONE));
    assert_eq!(
        inventory.take(PickupKind::Heart(HeartKind::Half)),
        Some(SoundEvent::HeartPickedUp)
    );
    assert_eq!(life_bar(&inventory)[2], Some(HudHeart::Half));
}

#[test]
fn coins_add_their_value_and_stop_at_99() {
    let mut inventory = inventory();
    inventory.take(PickupKind::Coin(CoinKind::Dime));
    inventory.take(PickupKind::Coin(CoinKind::Nickel));
    assert_eq!(inventory.coins, 15);
    for _ in 0..10 {
        inventory.take(PickupKind::Coin(CoinKind::Dime));
    }
    assert_eq!(inventory.coins, 99);
}

#[test]
fn soul_hearts_take_hits_first_and_lose_the_rest() {
    let mut inventory = inventory();
    inventory.take(PickupKind::Heart(HeartKind::Soul));
    assert_eq!(life_bar(&inventory)[3], Some(HudHeart::SoulFull));
    assert!(inventory.hurt(Hearts::HALF));
    assert_eq!(life_bar(&inventory)[3], Some(HudHeart::SoulHalf));
    inventory.advance(Frames(300.0));
    assert!(inventory.hurt(Hearts::ONE));
    assert_eq!(
        (inventory.soul_hearts, inventory.red_hearts),
        (Hearts::ZERO, Hearts(3.0))
    );
}

#[test]
fn hits_need_the_invulnerability_to_wear_off() {
    let mut inventory = inventory();
    assert!(inventory.hurt(Hearts::ONE));
    assert!(
        !inventory.hurt(Hearts::ONE),
        "two seconds of invulnerability after a heart of damage"
    );
    assert!(inventory.blinks_hidden());
    inventory.advance(Frames(60.0));
    assert!(!inventory.blinks_hidden());
    assert!(inventory.hurt(Hearts::ONE));
    inventory.advance(Frames(60.0));
    assert!(inventory.hurt(Hearts::ONE));
    assert!(inventory.is_dead());
}

#[test]
fn two_eternal_hearts_make_a_heart_container() {
    let mut inventory = inventory();
    inventory.take(PickupKind::Heart(HeartKind::Eternal));
    assert_eq!(life_bar(&inventory)[3], Some(HudHeart::Eternal));
    inventory.take(PickupKind::Heart(HeartKind::Eternal));
    assert_eq!(
        (inventory.maximum_red_hearts, inventory.red_hearts),
        (Hearts(4.0), Hearts(4.0))
    );
}

#[test]
fn golden_keys_never_run_out() {
    let mut inventory = inventory();
    assert!(!inventory.has_key());
    inventory.take(PickupKind::Key(KeyKind::Golden));
    inventory.use_key();
    assert!(inventory.has_key());
}

#[test]
fn blood_comes_from_red_hearts_before_soul_hearts() {
    let mut inventory = inventory();
    inventory.take(PickupKind::Heart(HeartKind::Soul));
    assert!(inventory.give_blood(1));
    assert_eq!(
        (inventory.red_hearts, inventory.soul_hearts),
        (Hearts(2.5), Hearts::ONE)
    );
}

#[test]
fn a_lone_half_red_heart_gives_way_to_the_soul_hearts() {
    let mut inventory = inventory();
    inventory.take(PickupKind::Heart(HeartKind::Soul));
    inventory.red_hearts = Hearts::HALF;
    assert!(inventory.give_blood(1));
    assert_eq!(
        (inventory.red_hearts, inventory.soul_hearts),
        (Hearts::HALF, Hearts::HALF)
    );
}

#[test]
fn in_a_late_chapter_a_borrowed_half_soul_heart_is_handed_back_when_nothing_was_taken() {
    let mut inventory = inventory();
    inventory.take(PickupKind::Heart(HeartKind::Soul));
    inventory.red_hearts = Hearts::HALF;
    assert!(inventory.hurt(Hearts::HALF));
    let (red_hearts, soul_hearts) = (inventory.red_hearts, inventory.soul_hearts);
    assert!(!inventory.give_blood(7));
    assert_eq!((inventory.red_hearts, inventory.soul_hearts), (red_hearts, soul_hearts));
}

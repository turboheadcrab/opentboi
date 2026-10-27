use crate::game::original_units::Frames;
use crate::game::pickups::kind::PickupKind;
use crate::game::pickups::trinket::Trinket;
use crate::game::{Game, GameParameters, InputState, RoomAppearance, RoomLayouts};

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

fn stand_on(game: &mut Game, trinket: Trinket) {
    let position = game.player.position();
    game.floor
        .current_room_mut()
        .pickups
        .place(PickupKind::Trinket(trinket), position);
    for _ in 0..60 {
        game.advance(&InputState::default(), ONE_FRAME);
    }
}

#[test]
/// `pillc()`: a trinket walked over is taken, and the one it replaces is left lying, untaken again.
fn a_trinket_is_taken_and_a_swapped_one_is_left_lying() {
    let mut game = game();
    stand_on(&mut game, Trinket::PushPin);
    assert_eq!(game.inventory.trinkets(), [Trinket::PushPin]);
    stand_on(&mut game, Trinket::FishHead);
    assert_eq!(game.inventory.trinkets(), [Trinket::FishHead]);
    let lying: Vec<PickupKind> = game.floor.current_room().pickups.lying_untaken().collect();
    assert!(lying.contains(&PickupKind::Trinket(Trinket::PushPin)), "{lying:?}");
}

#[test]
/// DoAction 28938, 241, 245 and 11062: what the stat trinkets add.
fn the_stat_trinkets_change_speed_damage_and_fire_delay() {
    let mut game = game();
    let bare = game.player_stats();
    let amounts = GameParameters::ORIGINAL.trinkets;
    game.inventory.take(PickupKind::Trinket(Trinket::GoatHoof));
    assert_eq!(game.player_stats().speed, bare.speed + amounts.goat_hoof_speed);
    game.inventory.take(PickupKind::Trinket(Trinket::CurvedHorn));
    assert_eq!(game.player_stats().damage, bare.damage + amounts.curved_horn_damage);
    game.inventory.take(PickupKind::Trinket(Trinket::GameSquid));
    assert_eq!(game.player_stats().damage, bare.damage + amounts.game_squid_damage);
    game.inventory.take(PickupKind::Trinket(Trinket::Cancer));
    assert_eq!(game.player_stats().delay_offset, amounts.cancer_delay);
    assert_eq!(game.player_stats().damage, bare.damage, "the Cancer adds no damage");
}

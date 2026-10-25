use super::*;
use crate::game::original_units::{Frames, Hearts};

/// Tests step the world one original frame at a time.
const ONE_FRAME: Frames = Frames(1.0);

#[test]
fn firing_and_tear_landing_produce_sound_events() {
    let mut game = Game::new(
        RoomAppearance::Placeholder,
        &GameParameters::ORIGINAL,
        &RoomLayouts::default(),
        1,
    );
    // Firing waits for the room's first 20 original frames, like movement.
    let shoot_up = InputState::from_raw_axes(Vec2::ZERO, Vec2::Y);
    game.advance(&shoot_up, ONE_FRAME);
    assert_eq!(game.drain_sound_events().count(), 0);
    for _ in 0..20 {
        game.advance(&InputState::default(), ONE_FRAME);
    }
    game.advance(&shoot_up, ONE_FRAME);
    assert_eq!(game.drain_sound_events().collect::<Vec<_>>(), [SoundEvent::TearFired]);

    let idle = InputState::default();
    let mut later_events = Vec::new();
    for _ in 0..60 {
        game.advance(&idle, ONE_FRAME);
        later_events.extend(game.drain_sound_events());
    }
    assert_eq!(later_events, [SoundEvent::TearImpact]);
    assert!(game.splashes.is_empty(), "the splash should have finished playing");
}

#[test]
fn movement_waits_twenty_frames_after_entering_a_room() {
    let mut game = Game::new(
        RoomAppearance::Placeholder,
        &GameParameters::ORIGINAL,
        &RoomLayouts::default(),
        1,
    );
    let start = game.player.position();
    let walk_right = InputState::from_raw_axes(Vec2::X, Vec2::ZERO);
    for _ in 0..19 {
        game.advance(&walk_right, ONE_FRAME);
    }
    assert_eq!(game.player.position(), start, "frozen while the room settles");
    for _ in 0..5 {
        game.advance(&walk_right, ONE_FRAME);
    }
    assert!(game.player.position().x > start.x);
}

#[test]
fn walking_straight_at_a_door_goes_through_it() {
    let mut game = Game::new(
        RoomAppearance::Placeholder,
        &GameParameters::ORIGINAL,
        &RoomLayouts::default(),
        1,
    );
    let start_cell = game.floor.current_cell();
    let side = game.floor.visible_doors()[0];
    let toward_door = side.position().normalize();
    // Line up with the doorway first, as a player would.
    game.player.place_at(side.position() * 0.5);
    let walk = InputState::from_raw_axes(toward_door, Vec2::ZERO);
    for _ in 0..120 {
        game.advance(&walk, ONE_FRAME);
    }
    assert_ne!(
        game.floor.current_cell(),
        start_cell,
        "should have walked through the {side:?} door"
    );
}

#[test]
fn turning_straight_back_after_entering_goes_back_through_the_door() {
    let mut game = Game::new(
        RoomAppearance::Placeholder,
        &GameParameters::ORIGINAL,
        &RoomLayouts::default(),
        1,
    );
    let side = game.floor.visible_doors()[0];
    let first_room = game.floor.current_cell();
    game.player.place_at(side.position() * 0.5);
    let toward = InputState::from_raw_axes(side.position().normalize(), Vec2::ZERO);
    for _ in 0..120 {
        game.advance(&toward, ONE_FRAME);
        if game.floor.current_cell() != first_room {
            break;
        }
    }
    assert_ne!(game.floor.current_cell(), first_room);
    let back = InputState::from_raw_axes(-side.position().normalize(), Vec2::ZERO);
    for _ in 0..120 {
        game.advance(&back, ONE_FRAME);
        if game.floor.current_cell() == first_room {
            break;
        }
    }
    assert_eq!(
        game.floor.current_cell(),
        first_room,
        "holding the way back returns through the door"
    );
}

#[test]
fn spikes_hurt_by_a_heart() {
    let mut game = Game::new(
        RoomAppearance::Placeholder,
        &GameParameters::ORIGINAL,
        &RoomLayouts::default(),
        1,
    );
    let full = game.inventory.red_hearts;
    let spike = room::grid_object::GridObject::on_tile(
        room::grid_kind::GridObjectKind::Tile(TileKind::Spikes),
        3,
        3,
        GameParameters::ORIGINAL.breakables,
    );
    game.floor.current_room_mut().add_grid_object(spike.clone());
    game.player.place_at(spike.center());
    game.advance(&InputState::default(), ONE_FRAME);
    assert_eq!(game.inventory.red_hearts, full - Hearts::ONE);
}

#[test]
fn showcases_appear_only_when_asked_for() {
    use crate::game::floor::plan::RoomRole;
    let cards_in_treasure_room = |showcase: bool| {
        let mut game = Game::new(
            RoomAppearance::Placeholder,
            &GameParameters::ORIGINAL,
            &RoomLayouts::default(),
            1,
        );
        game.apply_settings(GameSettings {
            showcase,
            ..GameSettings::default()
        });
        let treasure = game.floor.cell_with(RoomRole::Treasure).expect("a treasure room");
        let entry = game.floor.teleport_to(treasure, Vec2::ZERO);
        game.settle_into(&entry);
        let lying = game.floor.current_room().pickups.lying_kinds();
        lying.iter().filter(|kind| matches!(kind, PickupKind::Card(_))).count()
    };
    assert_eq!(cards_in_treasure_room(false), 0, "normal play");
    assert_eq!(cards_in_treasure_room(true), PickupCard::ALL.len());
}

#[test]
fn a_fly_keeps_the_doors_shut_until_it_dies() {
    use crate::game::floor::plan::RoomRole;
    use crate::game::room::enemies::Enemies;
    use crate::game::room::layout::LayoutCell;
    let mut game = Game::new(
        RoomAppearance::Placeholder,
        &GameParameters::ORIGINAL,
        &RoomLayouts::default(),
        1,
    );
    let room = game.floor.cell_with(RoomRole::Normal).expect("a normal room");
    let fly_layout = RoomLayout {
        cells: vec![LayoutCell {
            column: 2,
            row: 2,
            value: -4.0,
        }],
        ..RoomLayout::default()
    };
    let fly = Enemies::from_layout(
        &fly_layout,
        &crate::game::room::enemies::EnemyParameters::from_game(&GameParameters::ORIGINAL),
        &mut Random::with_seed(1),
    );
    let entry = game.floor.teleport_to(room, Vec2::ZERO);
    game.settle_into(&entry);
    game.floor.current_room_mut().enemies = fly;
    let entry = game.floor.teleport_to(room, Vec2::ZERO);
    game.settle_into(&entry);
    assert_eq!(game.floor.open_doors_count(), 0, "shut while the fly lives");
    // One fly buzzes at 30 % plus 30 % for it: 60 %.
    let buzz: Vec<LoopingSound> = game.looping_sounds().collect();
    assert_eq!(
        buzz,
        [LoopingSound {
            sound: SoundEvent::FlySwarm,
            volume: 0.6
        }]
    );
    game.floor.current_room_mut().enemies.blast(
        Vec2::ZERO,
        crate::game::original_units::Pixels(1000.0),
        &mut crate::game::random::Random::with_seed(1),
        &mut Vec::new(),
    );
    game.advance(&InputState::default(), ONE_FRAME);
    // `ashut = shutdoor`: usable at once, though their clips open only 10 frames later.
    assert!(game.floor.open_doors_count() > 0, "open once it is dead");
    assert_eq!(game.looping_sounds().count(), 0, "silent once it is dead");
}

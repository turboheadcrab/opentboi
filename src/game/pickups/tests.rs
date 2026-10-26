use super::*;
use crate::game::ball_contact::{Body, Contact};
use crate::game::bounds::Bounds;
use crate::game::original_units::Frames;
use crate::game::parameters::GameParameters;
use crate::game::pickups::kind::{BombKind, CoinKind, HeartKind};
use crate::game::player::inventory::Inventory;
use crate::game::room::interior_bounds;
use crate::game::room::layout::LayoutCell;
use crate::game::room::tile_center;

const ONE_FRAME: Frames = Frames(1.0);

fn pickups_with(kind: PickupKind) -> Pickups {
    Pickups {
        parameters: GameParameters::ORIGINAL.pickups,
        lying: vec![Pickup::new(kind, Vec2::ZERO, Vec2::ZERO)],
        red_chests: RedChestChance::default(),
    }
}

fn advance_frames(pickups: &mut Pickups, frames: u32) -> Vec<Vec2> {
    (0..frames)
        .flat_map(|_| pickups.advance(ONE_FRAME, quiet_room(), &mut Vec::new()))
        .collect()
}

fn quiet_room() -> PickupSurroundings<'static> {
    PickupSurroundings {
        room_interior: interior_bounds(),
        solid_tiles: &[],
        frames_in_room: Frames::ZERO,
        player_position: Vec2::new(200.0, 0.0),
        walls: GameParameters::ORIGINAL.walls,
    }
}

fn inventory() -> Inventory {
    Inventory::new(GameParameters::ORIGINAL.inventory)
}

#[test]
fn coin_markers_become_coins_below_their_cell() {
    let layout = RoomLayout {
        cells: vec![LayoutCell {
            column: 6,
            row: 3,
            value: -0.02,
        }],
        decorations: Vec::new(),
    };
    let pickups = Pickups::from_layout(&layout, GameParameters::ORIGINAL.pickups, &mut Random::with_seed(3));
    assert_eq!(pickups.lying.len(), 1);
    assert!(matches!(pickups.lying[0].kind, PickupKind::Coin(_)));
    assert_eq!(pickups.lying[0].center, tile_center(6, 3) - Vec2::new(0.0, 10.0));
}

#[test]
fn a_pickup_is_taken_once_it_has_landed_a_little() {
    let mut pickups = pickups_with(PickupKind::Coin(CoinKind::Penny));
    let mut inventory = inventory();
    let mut sounds = Vec::new();
    pickups.collect(Vec2::new(10.0, 0.0), &mut inventory, false, &mut sounds);
    assert_eq!(inventory.coins, 0, "not during its first 25 frames");
    advance_frames(&mut pickups, 26);
    pickups.collect(Vec2::new(30.0, 0.0), &mut inventory, false, &mut sounds);
    assert_eq!(inventory.coins, 0, "out of reach");
    pickups.collect(Vec2::new(10.0, 0.0), &mut inventory, false, &mut sounds);
    assert_eq!(inventory.coins, 1);
    assert_eq!(sounds, [SoundEvent::PennyPickedUp]);
    advance_frames(&mut pickups, 15);
    assert!(pickups.lying.is_empty(), "gone after its pickup animation");
}

#[test]
fn troll_bombs_cannot_be_taken_and_go_off() {
    let mut pickups = pickups_with(PickupKind::Bomb(BombKind::Troll));
    let mut inventory = inventory();
    let mut sounds = Vec::new();
    let explosions: Vec<Vec2> = (0..63)
        .flat_map(|_| {
            pickups.collect(Vec2::ZERO, &mut inventory, false, &mut sounds);
            pickups.advance(ONE_FRAME, quiet_room(), &mut Vec::new())
        })
        .collect();
    assert_eq!(explosions, [Vec2::ZERO]);
    assert_eq!(inventory.bombs, 1);
    assert!(sounds.is_empty(), "{sounds:?}");
}

#[test]
fn a_golden_chest_needs_a_key_and_stays_open() {
    let mut pickups = pickups_with(PickupKind::Chest(ChestKind::Golden));
    let mut inventory = inventory();
    let mut sounds = Vec::new();
    advance_frames(&mut pickups, 30);
    let locked = pickups.collect(Vec2::new(5.0, 0.0), &mut inventory, false, &mut sounds);
    assert!(locked.opened_chests.is_empty(), "{:?}", locked.opened_chests);
    let opened = pickups.collect(Vec2::new(5.0, 0.0), &mut inventory, true, &mut sounds);
    assert_eq!(opened.opened_chests, [(Vec2::ZERO, ChestKind::Golden)]);
    assert!(opened.used_key);
    assert_eq!(sounds, [SoundEvent::KeyUsed, SoundEvent::ChestOpened]);
    advance_frames(&mut pickups, 60);
    assert_eq!(pickups.lying.len(), 1, "the open chest stays");
}

#[test]
fn hearts_at_full_health_get_bumped_aside() {
    let mut pickups = pickups_with(PickupKind::Heart(HeartKind::Full));
    let mut inventory = inventory();
    advance_frames(&mut pickups, 30);
    pickups.collect(Vec2::new(10.0, 0.0), &mut inventory, false, &mut Vec::new());
    let player = Body {
        position: Vec2::new(10.0, 0.0),
        velocity: Vec2::new(-3.0, 0.0),
        mass: GameParameters::ORIGINAL.pickups.player_mass,
    };
    let contact = pickups.bump(player, ONE_FRAME, interior_bounds(), &[]);
    assert!(contact.first_offset.x > 0.0, "the player is pushed back");
    assert!(pickups.lying[0].center.x < 0.0, "the heart moves away");
    assert!(pickups.lying[0].velocity().x < 0.0, "and is kicked along");
}

#[test]
fn isaac_bumps_into_troll_bombs() {
    let mut pickups = pickups_with(PickupKind::Bomb(BombKind::Troll));
    advance_frames(&mut pickups, 30);
    let player = Body {
        position: Vec2::new(10.0, 0.0),
        velocity: Vec2::ZERO,
        mass: GameParameters::ORIGINAL.pickups.player_mass,
    };
    assert_ne!(
        pickups.bump(player, ONE_FRAME, interior_bounds(), &[]),
        Contact::default()
    );
}

#[test]
fn blasts_push_settled_pickups_away() {
    let mut pickups = pickups_with(PickupKind::Coin(CoinKind::Penny));
    let explosion = GameParameters::ORIGINAL.explosion;
    pickups.push_from_blast(Vec2::new(-30.0, 0.0), explosion, |_| true);
    advance_frames(&mut pickups, 1);
    assert_eq!(pickups.lying[0].center, Vec2::ZERO, "not while it is landing");
    advance_frames(&mut pickups, 30);
    pickups.push_from_blast(Vec2::new(-30.0, 0.0), explosion, |_| true);
    advance_frames(&mut pickups, 10);
    assert!(pickups.lying[0].center.x > 20.0);
}

#[test]
fn mega_troll_bombs_roll_toward_the_player() {
    let mut pickups = pickups_with(PickupKind::Bomb(BombKind::MegaTroll));
    advance_frames(&mut pickups, 40);
    assert!(pickups.lying[0].center.x > 20.0, "toward the player at x 200");
}

#[test]
fn a_heart_wedged_against_a_rock_holds_the_player_back() {
    let mut pickups = pickups_with(PickupKind::Heart(HeartKind::Full));
    advance_frames(&mut pickups, 30);
    let mut inventory = inventory();
    pickups.collect(Vec2::new(-10.0, 0.0), &mut inventory, false, &mut Vec::new());
    // A rock from x 12 to 32: the heart (radius 12) cannot get past x 0.
    let rock = Bounds::centered(Vec2::new(22.0, 0.0), Vec2::splat(20.0));
    let player = Body {
        position: Vec2::new(-10.0, 0.0),
        velocity: Vec2::new(3.0, 0.0),
        mass: GameParameters::ORIGINAL.pickups.player_mass,
    };
    let contact = pickups.bump(player, ONE_FRAME, interior_bounds(), &[rock]);
    assert!(
        pickups.lying[0].center.x <= 0.0 + 1e-3,
        "the heart cannot go into the rock"
    );
    assert!(contact.first_offset.x < -5.0, "the player takes the whole push");
}

#[test]
fn a_placed_pedestal_gives_way_slowly_and_snaps_back_from_big_shoves() {
    let mut pickups = Pickups {
        parameters: GameParameters::ORIGINAL.pickups,
        lying: vec![Pickup::placed(PickupKind::ItemPedestal, Vec2::ZERO)],
        red_chests: RedChestChance::default(),
    };
    pickups.lying[0].move_to(Vec2::new(10.0, 0.0), interior_bounds(), &[]);
    advance_frames(&mut pickups, 1);
    assert!((pickups.lying[0].center.x - 3.0).abs() < 1e-3, "keeps 30 % of the push");
    pickups.lying[0].move_to(Vec2::new(60.0, 0.0), interior_bounds(), &[]);
    advance_frames(&mut pickups, 1);
    assert!(
        (pickups.lying[0].center.x - 3.0).abs() < 1e-3,
        "a jump of 30 or more snaps back"
    );
}

/// How far a pickup lying at the origin ends up after Isaac walks into it for two seconds.
fn pushed_distance(kind: PickupKind) -> f32 {
    let mut pickups = Pickups {
        parameters: GameParameters::ORIGINAL.pickups,
        lying: vec![Pickup::placed(kind, Vec2::ZERO)],
        red_chests: RedChestChance::default(),
    };
    advance_frames(&mut pickups, 40);
    let walking = Vec2::new(4.0, 0.0);
    let mut player = Vec2::new(-30.0, 0.0);
    for _ in 0..60 {
        player += walking;
        let body = Body {
            position: player,
            velocity: walking,
            mass: GameParameters::ORIGINAL.pickups.player_mass,
        };
        player += pickups.bump(body, ONE_FRAME, interior_bounds(), &[]).first_offset;
        advance_frames(&mut pickups, 1);
    }
    pickups.lying[0].center.x
}

#[test]
fn an_empty_pedestal_gives_way_at_a_crawl_where_a_heart_is_kicked_along() {
    let pedestal = pushed_distance(PickupKind::ItemPedestal);
    let heart = pushed_distance(PickupKind::Heart(HeartKind::Full));
    assert!(pedestal < heart * 0.5, "pedestal {pedestal}, heart {heart}");
}

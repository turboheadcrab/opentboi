use glam::Vec2;

use super::*;
use crate::game::hud::name_banner::FORTUNE_COUNT;
use crate::game::parameters::GameParameters;

const PARAMETERS: PickupParameters = GameParameters::ORIGINAL.pickups;

fn player_at(position: Vec2) -> Body {
    Body {
        position,
        velocity: Vec2::ZERO,
        mass: PARAMETERS.player_mass,
    }
}

/// Runs the machines for `frames` single frames, collecting what they did.
fn run(machines: &mut Machines, frames: u32, random: &mut Random) -> Vec<MachineEvent> {
    (0..frames)
        .flat_map(|_| machines.advance(Frames(1.0), random, false))
        .map(|(_, event)| event)
        .collect()
}

#[test]
fn a_slot_machine_takes_a_coin_spins_and_shows_a_result() {
    let mut random = Random::default();
    let mut machines = Machines::new(PARAMETERS);
    machines.place(MachineKind::SlotMachine, Vec2::ZERO);
    let mut sounds = Vec::new();
    let near = player_at(Vec2::new(0.0, -10.0));
    let (contact, cost) = machines.bump(near, 1, Frames(1.0), &mut random, &mut sounds);
    assert_eq!(cost, Some(Cost::Coin));
    assert!(contact.first_offset.y < 0.0, "the player is pushed back");
    assert_eq!(sounds, [SoundEvent::CoinInserted]);
    let (_, again) = machines.bump(near, 1, Frames(1.0), &mut random, &mut sounds);
    assert_eq!(again, None, "busy while spinning");
    let events = run(&mut machines, 120, &mut random);
    assert!(events.contains(&MachineEvent::Sound(SoundEvent::ReelsStopped)));
    let machine = &machines.standing.first();
    assert!(machine.is_none_or(|machine| !machine.spinning), "the game ends");
}

#[test]
fn a_game_keeps_the_doors_shut_until_it_is_over() {
    let mut random = Random::default();
    let mut machines = Machines::new(PARAMETERS);
    machines.place(MachineKind::SlotMachine, Vec2::ZERO);
    assert!(!machines.doors_locked());
    machines.bump(
        player_at(Vec2::new(0.0, -10.0)),
        1,
        Frames(1.0),
        &mut random,
        &mut Vec::new(),
    );
    assert!(machines.doors_locked(), "`lastspin = fra + 100`");
    run(&mut machines, 300, &mut random);
    assert!(!machines.doors_locked());
}

#[test]
fn nothing_happens_without_coins_or_from_behind_the_shell_game() {
    let mut random = Random::default();
    let mut machines = Machines::new(PARAMETERS);
    machines.place(MachineKind::Beggar, Vec2::ZERO);
    machines.place(MachineKind::ShellGame, Vec2::new(200.0, 0.0));
    let mut sounds = Vec::new();
    let (_, broke) = machines.bump(
        player_at(Vec2::new(0.0, -10.0)),
        0,
        Frames(1.0),
        &mut random,
        &mut sounds,
    );
    assert_eq!(broke, None);
    let behind = player_at(Vec2::new(200.0, 10.0));
    let (_, from_behind) = machines.bump(behind, 5, Frames(1.0), &mut random, &mut sounds);
    assert_eq!(from_behind, None);
    let in_front = player_at(Vec2::new(200.0, -10.0));
    let (_, from_front) = machines.bump(in_front, 5, Frames(1.0), &mut random, &mut sounds);
    assert_eq!(from_front, Some(Cost::Coin));
}

#[test]
fn the_blood_bank_takes_blood_and_pays_coins_or_an_item() {
    let mut random = Random::with_seed(7);
    for _ in 0..20 {
        let mut machines = Machines::new(PARAMETERS);
        machines.place(MachineKind::BloodDonation, Vec2::ZERO);
        let (_, cost) = machines.bump(
            player_at(Vec2::new(0.0, -10.0)),
            0,
            Frames(1.0),
            &mut random,
            &mut Vec::new(),
        );
        assert_eq!(cost, Some(Cost::Blood));
        let events = run(&mut machines, 60, &mut random);
        let prizes: Vec<_> = events
            .iter()
            .filter_map(|event| match event {
                MachineEvent::Prize(drop, _) => Some(drop.frame),
                _ => None,
            })
            .collect();
        assert_eq!(prizes.len(), 1, "{events:?}");
        assert!(matches!(prizes[0], 2 | 10));
    }
}

#[test]
fn the_fortune_machine_tells_fortunes_from_its_list() {
    let mut random = Random::with_seed(3);
    let mut told = std::collections::HashSet::new();
    for _ in 0..200 {
        let mut machines = Machines::new(PARAMETERS);
        machines.place(MachineKind::FortuneTeller, Vec2::ZERO);
        machines.bump(
            player_at(Vec2::new(0.0, -10.0)),
            1,
            Frames(1.0),
            &mut random,
            &mut Vec::new(),
        );
        for event in run(&mut machines, 200, &mut random) {
            if let MachineEvent::Fortune(index) = event {
                assert!(index < FORTUNE_COUNT);
                told.insert(index);
            }
        }
    }
    assert!(told.len() > 8, "most of the list comes up: {told:?}");
}

#[test]
fn with_the_lucky_foot_no_fortune_is_told() {
    let mut random = Random::with_seed(3);
    for _ in 0..100 {
        let mut machines = Machines::new(PARAMETERS);
        machines.place(MachineKind::FortuneTeller, Vec2::ZERO);
        machines.bump(
            player_at(Vec2::new(0.0, -10.0)),
            1,
            Frames(1.0),
            &mut random,
            &mut Vec::new(),
        );
        let events: Vec<_> = (0..200)
            .flat_map(|_| machines.advance(Frames(1.0), &mut random, true))
            .map(|(_, event)| event)
            .collect();
        assert!(!events.iter().any(|event| matches!(event, MachineEvent::Fortune(_))));
    }
}

#[test]
fn a_beggar_given_enough_coins_leaves_an_item_or_gives_pickups() {
    let mut random = Random::with_seed(3);
    let mut machines = Machines::new(PARAMETERS);
    machines.place(MachineKind::Beggar, Vec2::ZERO);
    let mut gifts = 0;
    for _ in 0..40 {
        if machines.standing.is_empty() {
            break;
        }
        machines.bump(
            player_at(Vec2::new(0.0, -10.0)),
            1,
            Frames(1.0),
            &mut random,
            &mut Vec::new(),
        );
        let events = run(&mut machines, 150, &mut random);
        gifts += events
            .iter()
            .filter(|event| matches!(event, MachineEvent::Prize(..) | MachineEvent::PedestalBelow))
            .count();
    }
    assert!(gifts > 0);
}

#[test]
fn blasts_break_machines_open_once() {
    let mut machines = Machines::new(PARAMETERS);
    machines.place(MachineKind::SlotMachine, Vec2::ZERO);
    machines.place(MachineKind::DevilBeggar, Vec2::new(300.0, 0.0));
    let events: Vec<_> = machines
        .blow_up_near(Vec2::new(40.0, 0.0))
        .into_iter()
        .map(|(_, event)| event)
        .collect();
    assert_eq!(
        events,
        [MachineEvent::Contents(
            crate::game::pickups::rewards::RewardSource::BrownChest
        )]
    );
    let blown = machines.blow_up_near(Vec2::new(40.0, 0.0));
    assert!(blown.is_empty(), "{blown:?}");
}

#[test]
fn layout_markers_place_machines() {
    let mut random = Random::default();
    assert_eq!(
        MachineKind::for_marker(31, &mut random),
        Some(MachineKind::BloodDonation)
    );
    assert_eq!(MachineKind::for_marker(33, &mut random), Some(MachineKind::ShellGame));
    assert_eq!(MachineKind::for_marker(37, &mut random), Some(MachineKind::DevilBeggar));
    assert_eq!(MachineKind::for_marker(1, &mut random), None);
}

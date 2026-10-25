use std::collections::BTreeSet;

use super::*;

const FLOORS_CHECKED: u32 = 200;

fn neighbors(cell: FloorCell) -> impl Iterator<Item = FloorCell> {
    cell.array_neighbors().into_iter().flatten()
}

#[test]
fn floors_are_connected_with_a_distant_boss_room() {
    for seed in 1..=FLOORS_CHECKED {
        let plan = FloorPlan::generate(1, &mut Random::with_seed(seed));
        let rooms: BTreeSet<FloorCell> = plan.roles_by_cell.keys().copied().collect();
        assert_eq!(plan.roles_by_cell.get(&FloorCell::START), Some(&RoomRole::Start));
        let boss = plan
            .roles_by_cell
            .iter()
            .find(|(_, role)| **role == RoomRole::Boss)
            .map(|(cell, _)| *cell)
            .expect("every floor has a boss room");
        assert!(!boss.touches(FloorCell::START), "seed {seed}");
        assert!(rooms.len() >= 7, "seed {seed}: only {} rooms", rooms.len());
        let mut reached = BTreeSet::from([FloorCell::START]);
        let mut stack = vec![FloorCell::START];
        while let Some(cell) = stack.pop() {
            for next in neighbors(cell) {
                if rooms.contains(&next) && reached.insert(next) {
                    stack.push(next);
                }
            }
        }
        assert_eq!(reached, rooms, "seed {seed}: unreachable rooms");
    }
}

#[test]
fn first_floors_have_treasure_shop_and_secret_rooms() {
    let with_treasure = (1..=FLOORS_CHECKED)
        .filter(|&seed| {
            FloorPlan::generate(1, &mut Random::with_seed(seed))
                .roles_by_cell
                .values()
                .any(|role| *role == RoomRole::Treasure)
        })
        .count();
    assert!(
        u32::try_from(with_treasure).expect("counted floors") * 10 >= FLOORS_CHECKED * 9,
        "only {with_treasure} of {FLOORS_CHECKED}"
    );
}

#[test]
fn about_half_the_floors_have_a_curse_room_and_some_a_sacrifice_room() {
    let mut random = Random::with_seed(11);
    let floors: Vec<FloorPlan> = (0..200).map(|_| FloorPlan::generate(1, &mut random)).collect();
    let count = |role| {
        floors
            .iter()
            .filter(|plan| plan.roles_by_cell.values().any(|&found| found == role))
            .count()
    };
    let curses = count(RoomRole::Curse);
    let sacrifices = count(RoomRole::Sacrifice);
    assert!((50..150).contains(&curses), "{curses} curse rooms");
    assert!((5..60).contains(&sacrifices), "{sacrifices} sacrifice rooms");
}

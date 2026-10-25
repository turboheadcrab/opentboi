use super::*;

const ROLLS: u32 = 2000;

#[test]
fn breakables_drop_something_about_one_time_in_ten() {
    let mut random = Random::with_seed(5);
    let drops: Vec<u8> = (0..ROLLS).filter_map(|_| breakable_drop(&mut random)).collect();
    let fraction = drops.len() as f32 / ROLLS as f32;
    assert!((0.07..0.13).contains(&fraction), "{fraction}");
    assert!(drops.iter().all(|&frame| frame == HEART_FRAME || frame == COIN_FRAME));
}

#[test]
fn chests_hold_a_few_known_things() {
    let mut random = Random::with_seed(9);
    for _ in 0..ROLLS {
        let drops = contents(RewardSource::BrownChest, &mut random);
        assert!(drops.len() <= 3);
        assert!(
            drops
                .iter()
                .all(|drop| (HEART_FRAME..=GOLDEN_CHEST_FRAME).contains(&drop.frame) || drop.frame == PILL_FRAME)
        );
    }
}

#[test]
fn a_brown_chest_sometimes_holds_a_pill_and_a_golden_one_a_card_instead() {
    let mut random = Random::with_seed(11);
    let frames = |source, random: &mut Random| -> Vec<u8> {
        (0..ROLLS)
            .flat_map(|_| contents(source, random))
            .map(|drop| drop.frame)
            .collect()
    };
    let brown = frames(RewardSource::BrownChest, &mut random);
    assert!(brown.contains(&PILL_FRAME) && !brown.contains(&CARD_MARKER_FRAME));
    let golden = frames(RewardSource::GoldenChest, &mut random);
    assert!(golden.contains(&CARD_MARKER_FRAME) && !golden.contains(&PILL_FRAME));
}

#[test]
fn the_tinted_rock_gives_soul_hearts_but_no_coins() {
    let mut random = Random::with_seed(2);
    let drops: Vec<Drop> = (0..ROLLS)
        .flat_map(|_| contents(RewardSource::TintedRock, &mut random))
        .collect();
    assert!(drops.iter().all(|drop| drop.frame != COIN_FRAME));
    assert!(
        drops
            .iter()
            .filter(|drop| drop.frame == HEART_FRAME)
            .all(|drop| drop.soul_heart)
    );
}

#[test]
fn room_clear_rewards_skip_about_a_quarter() {
    let mut random = Random::with_seed(4);
    let given = (0..ROLLS)
        .filter(|_| room_clear_reward(&mut random, 0, false, false).is_some())
        .count();
    let fraction = given as f32 / ROLLS as f32;
    assert!((0.70..0.80).contains(&fraction), "{fraction}");
}

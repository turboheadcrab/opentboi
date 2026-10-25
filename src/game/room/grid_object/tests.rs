use super::*;
use crate::game::original_units::Frames;
use crate::game::parameters::GameParameters;

const ORIGINAL: BreakableParameters = GameParameters::ORIGINAL.breakables;
const TRIALS: usize = 200;

fn hits_to_break(kind: BreakableKind, random: &mut Random) -> usize {
    let mut object = GridObject::on_tile(GridObjectKind::Breakable(kind), 0, 0, ORIGINAL);
    let mut hits = 0;
    while object.blocks_tears() {
        object.take_hit(HitStrength::Tear, random);
        hits += 1;
    }
    hits
}

#[test]
fn breakables_need_the_original_number_of_hits() {
    let mut random = Random::default();
    for _ in 0..TRIALS {
        assert!((2..=4).contains(&hits_to_break(BreakableKind::Poop, &mut random)));
        assert!((2..=4).contains(&hits_to_break(BreakableKind::Fireplace, &mut random)));
        assert!((2..=3).contains(&hits_to_break(BreakableKind::TntBarrel, &mut random)));
        assert!(hits_to_break(BreakableKind::GoldenPoop, &mut random) >= 4);
    }
}

#[test]
fn red_poop_lacks_the_poop_only_extra_roll() {
    let mut random = Random::default();
    let poop_average: usize = (0..TRIALS)
        .map(|_| hits_to_break(BreakableKind::Poop, &mut random))
        .sum();
    let red_average: usize = (0..TRIALS)
        .map(|_| hits_to_break(BreakableKind::RedPoop, &mut random))
        .sum();
    assert!(red_average > poop_average);
}

#[test]
fn pits_block_walking_but_not_tears_and_webs_block_neither() {
    let pit = GridObject::on_tile(GridObjectKind::Tile(TileKind::Pit), 0, 0, ORIGINAL);
    assert!(pit.blocks_walking() && !pit.blocks_tears());
    let web = GridObject::on_tile(GridObjectKind::Tile(TileKind::SpiderWeb), 0, 0, ORIGINAL);
    assert!(!web.blocks_walking() && !web.blocks_tears() && web.is_spider_web());
    let mut rock = GridObject::on_tile(GridObjectKind::Tile(TileKind::Rock), 0, 0, ORIGINAL);
    assert!(rock.blocks_tears() && rock.take_hit(HitStrength::Tear, &mut Random::default()).is_none());
}

#[test]
fn red_poop_grows_back_after_four_frames() {
    let mut red_poop = GridObject::on_tile(GridObjectKind::Breakable(BreakableKind::RedPoop), 0, 0, ORIGINAL);
    let mut random = Random::default();
    while red_poop.blocks_tears() {
        red_poop.take_hit(HitStrength::Tear, &mut random);
    }
    red_poop.advance(Frames(119.0));
    assert!(!red_poop.blocks_walking());
    red_poop.advance(Frames(2.0));
    assert!(red_poop.blocks_walking());
}

#[test]
fn explosions_break_everything_but_golden_poop_in_one_hit() {
    let mut random = Random::default();
    for kind in [BreakableKind::Poop, BreakableKind::Fireplace, BreakableKind::TntBarrel] {
        let mut object = GridObject::on_tile(GridObjectKind::Breakable(kind), 0, 0, ORIGINAL);
        assert!(matches!(
            object.take_hit(HitStrength::Explosion, &mut random),
            Some(HitOutcome::Destroyed(_))
        ));
    }
}

#[test]
fn opened_key_block_stops_blocking() {
    let mut key_block = GridObject::on_tile(GridObjectKind::KeyBlock, 0, 0, ORIGINAL);
    assert!(key_block.blocks_walking());
    key_block.open_with_key();
    assert!(!key_block.blocks_walking() && !key_block.is_closed_key_block());
}

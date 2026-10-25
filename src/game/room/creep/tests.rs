use glam::Vec2;

use super::*;
use crate::game::parameters::GameParameters;

const PARAMETERS: CreepParameters = GameParameters::ORIGINAL.creep;
const SPOT: Vec2 = Vec2::new(100.0, 50.0);

fn puddle() -> Creep {
    let mut creep = Creep::default();
    creep.spawn(SPOT, 1.0);
    creep
}

#[test]
fn a_puddle_covers_what_is_near_its_center_only() {
    let creep = puddle();
    assert!(creep.is_under(SPOT + Vec2::new(10.0, 0.0), &PARAMETERS));
    assert!(!creep.is_under(SPOT + Vec2::new(40.0, 0.0), &PARAMETERS));
}

#[test]
fn a_puddle_holds_its_size_then_shrinks_and_is_gone_at_141_frames() {
    let mut creep = puddle();
    creep.advance(Frames(100.0), &PARAMETERS);
    assert!(creep.is_under(SPOT + Vec2::new(14.0, 0.0), &PARAMETERS));
    creep.advance(Frames(30.0), &PARAMETERS);
    assert!(!creep.is_under(SPOT + Vec2::new(14.0, 0.0), &PARAMETERS));
    assert!(creep.is_under(SPOT, &PARAMETERS));
    creep.advance(Frames(11.0), &PARAMETERS);
    assert!(!creep.is_under(SPOT, &PARAMETERS));
}

#[test]
fn a_bigger_puddle_covers_more() {
    let mut creep = Creep::default();
    creep.spawn(SPOT, 3.0);
    assert!(creep.is_under(SPOT + Vec2::new(40.0, 0.0), &PARAMETERS));
}

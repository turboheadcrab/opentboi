use super::*;
use crate::game::parameters::GameParameters;
use crate::game::room::interior_bounds;

const ORIGINAL_TEAR: TearParameters = GameParameters::ORIGINAL.tear;

#[test]
fn tear_lands_after_the_original_25_to_28_frames() {
    let mut random = Random::default();
    for _ in 0..20 {
        let mut tear = Tear::fire(Vec2::ZERO, Vec2::Y, Vec2::ZERO, 1.0, ORIGINAL_TEAR, &mut random);
        let stationary = Tear {
            velocity: Vec2::ZERO,
            ..tear.clone()
        };
        tear = stationary;
        tear.advance(Frames(24.0));
        assert_eq!(tear.status(&[]), TearStatus::Flying);
        tear.advance(Frames(4.0));
        assert_eq!(tear.status(&[]), TearStatus::Landed);
    }
}

#[test]
fn tear_flies_at_ten_pixels_per_original_frame() {
    let tear = Tear::fire(
        Vec2::ZERO,
        Vec2::X,
        Vec2::ZERO,
        1.0,
        ORIGINAL_TEAR,
        &mut Random::default(),
    );
    assert!((tear.velocity.length() - ORIGINAL_TEAR.speed_per_frame.get()).abs() < 1e-3);
    assert!(interior_bounds().contains(tear.position));
}

#[test]
fn tear_hits_the_wall_beyond_the_floor_edge() {
    let near_right_wall = Vec2::new(250.0, 0.0);
    let mut tear = Tear::fire(
        near_right_wall,
        Vec2::X,
        Vec2::ZERO,
        1.0,
        ORIGINAL_TEAR,
        &mut Random::default(),
    );
    tear.advance(Frames(4.0));
    assert_eq!(tear.status(&[]), TearStatus::Flying);
    tear.advance(Frames(2.0));
    assert_eq!(tear.status(&[]), TearStatus::HitWall);
}

#[test]
fn a_dark_tear_draws_the_dark_picture_a_little_smaller() {
    let mut random = Random::default();
    let mut drawn = |dark: bool| {
        let tear = Tear::fire(Vec2::ZERO, Vec2::Y, Vec2::ZERO, 1.0, ORIGINAL_TEAR, &mut random).with_dark_look(dark);
        let mut items = Vec::new();
        tear.append_draw_items(&mut items);
        items.pop().expect("the body is drawn last")
    };
    let (plain, dark) = (drawn(false), drawn(true));
    assert_eq!(plain.texture, Some(TextureId::Tear));
    assert_eq!(dark.texture, Some(TextureId::DarkTear));
    assert!((dark.size.x / plain.size.x - ORIGINAL_TEAR.dark_body_scale).abs() < 1e-4);
}

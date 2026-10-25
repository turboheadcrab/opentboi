use super::*;
use crate::game::parameters::GameParameters;
use crate::game::room::interior_bounds;

fn open_room() -> Bounds {
    interior_bounds()
}

const ONE_FRAME: Frames = Frames(1.0);

#[test]
fn holding_the_button_places_one_bomb_per_second() {
    let mut bombs = Bombs::new(GameParameters::ORIGINAL.bombs, GameParameters::ORIGINAL.walls);
    let placed = (0..60)
        .filter(|_| {
            let placed = bombs.try_place(true, Vec2::ZERO);
            bombs.advance(ONE_FRAME, open_room());
            placed
        })
        .count();
    assert_eq!(placed, 2);
}

#[test]
fn a_bomb_explodes_where_it_was_placed_after_its_fuse() {
    let parameters = GameParameters::ORIGINAL.bombs;
    let mut bombs = Bombs::new(parameters, GameParameters::ORIGINAL.walls);
    let position = Vec2::new(30.0, -20.0);
    assert!(bombs.try_place(true, position));
    let half_frame = Frames(0.5);
    let fuse_frames = parameters.exploding_fuse_frame - FIRST_CLIP_FRAME;
    let explosions = bombs.advance(fuse_frames - half_frame, open_room());
    assert!(explosions.is_empty(), "{explosions:?}");
    assert_eq!(bombs.advance(half_frame * 2.0, open_room()), [position]);
    assert!(bombs.lit.is_empty());
}

#[test]
fn a_bomb_placed_under_the_player_lets_them_walk_off() {
    let parameters = GameParameters::ORIGINAL;
    let mut bombs = Bombs::new(parameters.bombs, parameters.walls);
    assert!(bombs.try_place(true, Vec2::ZERO));
    let player = |x: f32| Body {
        position: Vec2::new(x, 0.0),
        velocity: Vec2::ZERO,
        mass: parameters.pickups.player_mass,
    };
    assert_eq!(
        bombs.bump(player(5.0), ONE_FRAME),
        Contact::default(),
        "still standing on it"
    );
    bombs.advance(Frames(4.0), open_room());
    let contact = bombs.bump(player(20.0), ONE_FRAME);
    assert!(contact.first_offset.x > 0.0, "solid four frames after the last touch");
}

#[test]
fn tears_push_lit_bombs_and_splash() {
    let parameters = GameParameters::ORIGINAL;
    let mut bombs = Bombs::new(parameters.bombs, parameters.walls);
    bombs.try_place(true, Vec2::ZERO);
    let tear_velocity = Vec2::new(10.0, 0.0);
    assert!(
        !bombs.hit_by_tear(Vec2::new(-10.0, 0.0), tear_velocity, ONE_FRAME),
        "just placed"
    );
    bombs.advance(Frames(4.0), open_room());
    assert!(bombs.hit_by_tear(Vec2::new(-10.0, 0.0), tear_velocity, ONE_FRAME));
    assert!(bombs.lit[0].velocity.x > 0.0, "kicked along");
    assert!(
        !bombs.hit_by_tear(Vec2::new(-40.0, 0.0), tear_velocity, ONE_FRAME),
        "out of reach"
    );
}

#[test]
fn lit_bombs_push_each_other_apart() {
    let parameters = GameParameters::ORIGINAL;
    let mut bombs = Bombs::new(parameters.bombs, parameters.walls);
    bombs.try_place(true, Vec2::ZERO);
    bombs.frames_since_placed = parameters.bombs.frames_between_placements;
    bombs.try_place(true, Vec2::new(5.0, 0.0));
    bombs.advance(Frames(1.0), open_room());
    let gap = bombs.lit[1].center.x - bombs.lit[0].center.x;
    assert!(gap > 5.0, "{gap}");
}

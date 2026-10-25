use super::*;
use crate::game::original_units::Frames;
use crate::game::room::interior_bounds;

const HALF_FRAME: Frames = Frames(0.5);

fn open_room() -> Surroundings<'static> {
    Surroundings {
        room_interior: interior_bounds(),
        solid_tiles: &[],
        spider_webs: &[],
    }
}

#[test]
fn spider_web_slows_the_player() {
    let input = InputState::from_raw_axes(Vec2::X, Vec2::ZERO);
    let web = Bounds::centered(Vec2::ZERO, Vec2::splat(1_000.0));
    let mut free_player = Player::new(Vec2::ZERO, &GameParameters::ORIGINAL);
    let mut webbed_player = Player::new(Vec2::ZERO, &GameParameters::ORIGINAL);
    let webbed_room = Surroundings {
        spider_webs: &[web],
        ..open_room()
    };
    for _ in 0..30 {
        free_player.update(&input, HALF_FRAME, &open_room(), &mut Random::default());
        webbed_player.update(&input, HALF_FRAME, &webbed_room, &mut Random::default());
    }
    assert!(webbed_player.velocity.length() < free_player.velocity.length() * 0.5);
}

#[test]
fn player_stays_inside_the_room() {
    let mut player = Player::new(Vec2::ZERO, &GameParameters::ORIGINAL);
    let input = InputState::from_raw_axes(Vec2::X, Vec2::ZERO);
    for _ in 0..600 {
        player.update(&input, HALF_FRAME, &open_room(), &mut Random::default());
    }
    assert!(interior_bounds().shrunk_by(COLLISION_RADIUS).contains(player.position));
}

#[test]
fn holding_fire_respects_fire_delay() {
    let mut player = Player::new(Vec2::ZERO, &GameParameters::ORIGINAL);
    let input = InputState::from_raw_axes(Vec2::ZERO, Vec2::Y);
    let mut random = Random::default();
    let half_frames_in_one_second = 60;
    let tears_fired = (0..half_frames_in_one_second)
        .filter_map(|_| player.update(&input, HALF_FRAME, &open_room(), &mut random))
        .count();
    let fire_delay_frames = GameParameters::ORIGINAL.firing.frames_between_shots();
    let frames_in_one_second = Frames(30.0);
    assert_eq!(
        tears_fired,
        crate::whole_numbers::clamped_usize(f64::from((frames_in_one_second / fire_delay_frames).ceil()))
    );
}

#[test]
fn holding_the_mouse_aims_at_the_cursor_and_overrides_keys() {
    let mut player = Player::new(Vec2::ZERO, &GameParameters::ORIGINAL);
    let input = InputState {
        aim_target: Some(Vec2::new(-200.0, 50.0)),
        ..InputState::from_raw_axes(Vec2::ZERO, Vec2::Y)
    };
    let tear = player.update(&input, HALF_FRAME, &open_room(), &mut Random::default());
    let tear_velocity = tear.expect("a tear should be fired").velocity();
    assert!(tear_velocity.x < 0.0 && tear_velocity.x.abs() > tear_velocity.y.abs());
}

#[test]
fn player_cannot_walk_into_a_solid_tile() {
    let tile = Bounds::centered(Vec2::new(60.0, 0.0), Vec2::splat(40.0));
    let mut player = Player::new(Vec2::ZERO, &GameParameters::ORIGINAL);
    let input = InputState::from_raw_axes(Vec2::X, Vec2::ZERO);
    for _ in 0..120 {
        let surroundings = Surroundings {
            solid_tiles: &[tile],
            ..open_room()
        };
        player.update(&input, HALF_FRAME, &surroundings, &mut Random::default());
    }
    assert!(player.position.x <= tile.minimum.x - COLLISION_RADIUS.get() + 0.01);
}

#[test]
fn pressing_into_a_wall_leaves_no_speed_into_it() {
    let mut player = Player::new(Vec2::ZERO, &GameParameters::ORIGINAL);
    let down_and_right = InputState::from_raw_axes(Vec2::new(1.0, -1.0).normalize(), Vec2::ZERO);
    for _ in 0..120 {
        player.update(&down_and_right, Frames(1.0), &open_room(), &mut Random::default());
    }
    let input_down = InputState::from_raw_axes(Vec2::NEG_Y, Vec2::ZERO);
    for _ in 0..30 {
        player.update(&input_down, Frames(1.0), &open_room(), &mut Random::default());
    }
    let top_speed = GameParameters::ORIGINAL.movement.top_speed_per_frame().get();
    assert!(player.velocity.y > -top_speed * 0.5, "{:?}", player.velocity);
}

#[test]
fn a_diagonal_into_a_wall_slides_along_it() {
    let mut player = Player::new(Vec2::ZERO, &GameParameters::ORIGINAL);
    let down_and_right = InputState::from_raw_axes(Vec2::new(1.0, -1.0), Vec2::ZERO);
    let pressed = (0..60).any(|_| {
        player.update(&down_and_right, Frames(1.0), &open_room(), &mut Random::default());
        player.slide.is_some()
    });
    assert!(pressed, "pressing into the bottom wall");
    let along = player.slide_along_wall(Vec2::new(1.0, -1.0).normalize(), Frames(1.0));
    assert!(
        along.abs_diff_eq(Vec2::X, 1e-5),
        "only the direction along the wall, at full strength: {along:?}"
    );
}

#[test]
fn a_nearly_straight_push_into_a_rock_does_not_bounce_back() {
    let rock = [Bounds::centered(Vec2::new(100.0, 10.0), Vec2::splat(40.0))];
    let room = Surroundings {
        solid_tiles: &rock,
        ..open_room()
    };
    let mut player = Player::new(Vec2::ZERO, &GameParameters::ORIGINAL);
    let nearly_straight = InputState::from_raw_axes(Vec2::new(1.0, 0.3).normalize(), Vec2::ZERO);
    let mut closest = f32::MIN;
    for _ in 0..12 {
        player.update(&nearly_straight, Frames(1.0), &room, &mut Random::default());
        closest = closest.max(player.position.x);
        assert!(
            player.position.x > closest - 2.0,
            "bounced back from {closest} to {}",
            player.position.x
        );
    }
}

//! `physix()`'s answer to a body pressing into a wall or obstacle (DoAction 26013–26112): the
//! velocity into it is turned around with a slight rebound (DoAction 26026, DoAction 26036), and a
//! hard impact slows the body by half the share of its new speed that it had pressed into the wall
//! (DoAction 26047, DoAction 26107, DoAction 26108).

use glam::Vec2;

use crate::game::parameters::WallParameters;

/// The velocity after meeting a wall: `blocked` is how far the walls pushed the body back this
/// step (zero if nothing did).
pub fn rebound(velocity: Vec2, blocked: Vec2, walls: WallParameters) -> Vec2 {
    let towards_wall = blocked.normalize_or_zero();
    let into_wall = towards_wall.dot(velocity);
    if into_wall <= 0.0 {
        return velocity;
    }
    let turned = velocity - towards_wall * (into_wall * walls.rebound + walls.push_away.get());
    let speed = turned.length();
    if speed <= 0.0 {
        return turned;
    }
    turned * (1.0 - into_wall / speed * walls.impact_damping)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::parameters::GameParameters;

    #[test]
    fn a_body_thrown_at_a_wall_bounces_back_slower() {
        let walls = GameParameters::ORIGINAL.walls;
        let velocity = rebound(Vec2::new(4.0, 1.0), Vec2::X, walls);
        assert!(velocity.x < 0.0 && velocity.x > -4.0, "{velocity:?}");
        assert!(
            velocity.y > 0.0 && velocity.y < 1.0,
            "along the wall it keeps going, damped"
        );
        assert_eq!(rebound(Vec2::new(-4.0, 0.0), Vec2::X, walls), Vec2::new(-4.0, 0.0));
    }

    #[test]
    /// DoAction 26047, 26107 and 26108: straight into a wall at 4 px/frame, the rebound is 3.3 px/frame
    /// back, and the impact slows that to `1 - 0.5 * 4 / 3.3` of it.
    fn a_hard_impact_slows_the_rebound_by_the_share_of_the_new_speed_pressed_in() {
        let walls = GameParameters::ORIGINAL.walls;
        let velocity = rebound(Vec2::new(4.0, 0.0), Vec2::X, walls);
        let expected = -3.3 * (1.0 - 0.5 * 4.0 / 3.3);
        assert!((velocity.x - expected).abs() < 1e-4, "{velocity:?} against {expected}");
    }
}

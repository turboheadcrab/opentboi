//! `physix()`'s answer to a body pressing into a wall or obstacle (DoAction 26026–26112): the
//! velocity into it is turned around with a slight rebound, and a hard impact slows the body.

use glam::Vec2;

use crate::game::original_units::Pixels;
use crate::game::parameters::WallParameters;

/// The velocity after meeting a wall: `blocked` is how far the walls pushed the body back this
/// step (zero if nothing did), `radius` the body's size (`sizes[s]`).
pub fn rebound(velocity: Vec2, blocked: Vec2, radius: Pixels, walls: WallParameters) -> Vec2 {
    let towards_wall = blocked.normalize_or_zero();
    let into_wall = towards_wall.dot(velocity);
    if into_wall <= 0.0 {
        return velocity;
    }
    let turned = velocity - towards_wall * (into_wall * walls.rebound + walls.push_away.get());
    if turned.length() <= 0.0 {
        return turned;
    }
    let reach = (radius + walls.reach_margin).get();
    turned * (1.0 - into_wall / reach * walls.impact_damping)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::parameters::GameParameters;

    #[test]
    fn a_body_thrown_at_a_wall_bounces_back_slower() {
        let walls = GameParameters::ORIGINAL.walls;
        let velocity = rebound(Vec2::new(4.0, 1.0), Vec2::X, Pixels(12.0), walls);
        assert!(velocity.x < 0.0 && velocity.x > -4.0, "{velocity:?}");
        assert!(
            velocity.y > 0.0 && velocity.y < 1.0,
            "along the wall it keeps going, damped"
        );
        assert_eq!(
            rebound(Vec2::new(-4.0, 0.0), Vec2::X, Pixels(12.0), walls),
            Vec2::new(-4.0, 0.0)
        );
    }
}

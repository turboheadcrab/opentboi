//! Which way the player shoots when two directions are held (`playc()`, DoAction 28381–28441):
//! `dirkey()` sums the arrow keys per axis, and the game remembers when each axis was last let go;
//! with both held, the axis pressed more recently wins, and keys pressed on the same frame shoot
//! sideways.

use glam::Vec2;

use crate::game::original_units::Frames;

/// An axis counts as held from half deflection: keys give 1, a stick pushed diagonally about 0.7,
/// and a stick pushed nearly straight leaves the other axis below this.
const HELD_DEFLECTION: f32 = 0.5;

#[derive(Clone, Copy, Debug, Default)]
pub struct ShootingAxes {
    /// `fra`.
    frames: Frames,
    /// `lastx`, `lasty`: when each axis was last not held.
    horizontal_released_at: Frames,
    vertical_released_at: Frames,
}

impl ShootingAxes {
    /// The direction to shoot this tick from the held `axes` (each -1 to 1), as a unit vector.
    pub fn direction(&mut self, axes: Vec2, elapsed: Frames) -> Option<Vec2> {
        self.frames += elapsed;
        let horizontal = (axes.x.abs() >= HELD_DEFLECTION).then_some(axes.x.signum());
        let vertical = (axes.y.abs() >= HELD_DEFLECTION).then_some(axes.y.signum());
        if horizontal.is_none() {
            self.horizontal_released_at = self.frames;
        }
        if vertical.is_none() {
            self.vertical_released_at = self.frames;
        }
        match (horizontal, vertical) {
            (Some(_), Some(y)) if self.vertical_released_at > self.horizontal_released_at => Some(Vec2::new(0.0, y)),
            (Some(x), _) => Some(Vec2::new(x, 0.0)),
            (None, Some(y)) => Some(Vec2::new(0.0, y)),
            // A stick nudged less than half way still shoots along its main axis.
            (None, None) => crate::game::input_state::snap_to_cardinal(axes),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ONE_FRAME: Frames = Frames(1.0);

    #[test]
    fn the_axis_pressed_last_wins() {
        let mut axes = ShootingAxes::default();
        assert_eq!(axes.direction(Vec2::Y, ONE_FRAME), Some(Vec2::Y));
        assert_eq!(axes.direction(Vec2::new(1.0, 1.0), ONE_FRAME), Some(Vec2::X));
        let mut axes = ShootingAxes::default();
        assert_eq!(axes.direction(Vec2::X, ONE_FRAME), Some(Vec2::X));
        assert_eq!(axes.direction(Vec2::new(1.0, -1.0), ONE_FRAME), Some(Vec2::NEG_Y));
    }

    #[test]
    fn keys_pressed_together_shoot_sideways() {
        let mut axes = ShootingAxes::default();
        assert_eq!(axes.direction(Vec2::new(-1.0, 1.0), ONE_FRAME), Some(Vec2::NEG_X));
    }

    #[test]
    fn nothing_held_shoots_nothing() {
        assert_eq!(ShootingAxes::default().direction(Vec2::ZERO, ONE_FRAME), None);
    }
}

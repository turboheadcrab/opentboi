//! Where flying things end: the original's `mhity()` (DoAction 5995–6004) and `bord()` (23625–23650), in stage coordinates (pixels,
//! y down), which is how the enemies keep their positions.

use glam::Vec2;

/// `mhity()`: shots and tears fly only strictly inside this area, which reaches over the walls so
/// they splash against the wall face. Anything outside it, or in something solid, ends them.
const FLIGHT_AREA_MINIMUM: Vec2 = Vec2::new(20.0, 110.0);
const FLIGHT_AREA_MAXIMUM: Vec2 = Vec2::new(620.0, 447.0);

/// Whether `stage_point` is outside the flight area.
pub fn leaves_flight_area(stage_point: Vec2) -> bool {
    !(stage_point.cmpgt(FLIGHT_AREA_MINIMUM).all() && stage_point.cmplt(FLIGHT_AREA_MAXIMUM).all())
}

/// `mhity()`'s test of the cell's value (DoAction 6003): from 1 up, other than a pit's 3, it stops
/// what flies.
pub fn stops_flight(level_value: f32) -> bool {
    const SOLID_FROM: f32 = 1.0;
    const PIT_VALUE: f32 = 3.0;
    level_value >= SOLID_FROM && level_value != PIT_VALUE
}

/// `mhity()`: whether something flying at `stage_point` is stopped, either by leaving the area or
/// by `in_solid` (looked up only inside the area), which says whether the point lies in something
/// that blocks flight: everything that blocks walking except pits.
pub fn blocks_flight(stage_point: Vec2, in_solid: impl FnOnce() -> bool) -> bool {
    leaves_flight_area(stage_point) || in_solid()
}

/// `bord()`: the rectangle (stage coordinates) an enemy is kept in.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Border {
    pub minimum: Vec2,
    pub maximum: Vec2,
}

/// Where an enemy ended up after `bord()`, and whether it was against an edge.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BorderContact {
    pub position: Vec2,
    pub touched: bool,
}

impl Border {
    /// The rectangle the original uses when none is given: x 60–575, y 150–410.
    pub const ORIGINAL: Self = Self {
        minimum: Vec2::new(60.0, 150.0),
        maximum: Vec2::new(575.0, 410.0),
    };

    /// Pulls `position` inside; it counts as touching also when it was exactly on an edge.
    pub fn keep_within(self, position: Vec2) -> BorderContact {
        let kept = position.clamp(self.minimum, self.maximum);
        let touched = kept.cmpeq(self.minimum).any() || kept.cmpeq(self.maximum).any();
        BorderContact {
            position: kept,
            touched,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flight_ends_outside_the_area_or_in_something_solid() {
        let middle = Vec2::new(320.0, 280.0);
        assert!(!blocks_flight(middle, || false));
        assert!(blocks_flight(middle, || true));
        assert!(blocks_flight(Vec2::new(20.0, 280.0), || false), "the edge is outside");
        assert!(blocks_flight(Vec2::new(320.0, 447.0), || false));
        assert!(blocks_flight(Vec2::new(320.0, 100.0), || false));
    }

    #[test]
    fn the_border_pulls_inside_and_reports_touching() {
        let inside = Border::ORIGINAL.keep_within(Vec2::new(300.0, 300.0));
        assert_eq!(inside.position, Vec2::new(300.0, 300.0));
        assert!(!inside.touched);
        let outside = Border::ORIGINAL.keep_within(Vec2::new(700.0, 300.0));
        assert_eq!(outside.position, Vec2::new(575.0, 300.0));
        assert!(outside.touched);
        assert!(
            Border::ORIGINAL.keep_within(Vec2::new(60.0, 300.0)).touched,
            "exactly on an edge"
        );
    }
}

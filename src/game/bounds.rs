use glam::Vec2;

use crate::game::original_units::Pixels;

/// Axis-aligned rectangle in world units.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Bounds {
    pub minimum: Vec2,
    pub maximum: Vec2,
}

impl Bounds {
    pub fn centered(center: Vec2, size: Vec2) -> Self {
        let half_size = size / 2.0;
        Self {
            minimum: center - half_size,
            maximum: center + half_size,
        }
    }

    /// Moves every edge inward by `margin`, e.g. to keep a circle of that radius inside.
    pub fn shrunk_by(self, margin: Pixels) -> Self {
        Self {
            minimum: self.minimum + margin.get(),
            maximum: self.maximum - margin.get(),
        }
    }

    pub fn clamp(self, point: Vec2) -> Vec2 {
        point.clamp(self.minimum, self.maximum)
    }

    pub fn center(self) -> Vec2 {
        (self.minimum + self.maximum) / 2.0
    }

    pub fn contains(self, point: Vec2) -> bool {
        point.cmpge(self.minimum).all() && point.cmple(self.maximum).all()
    }

    /// Moves a circle out of an axis-aligned tile along the shortest way.
    pub fn push_circle_out(self, center: Vec2, radius: Pixels) -> Vec2 {
        let radius = radius.get();
        let tile = self;
        let closest_point = tile.clamp(center);
        let offset = center - closest_point;
        let distance = offset.length();
        if distance >= radius {
            return center;
        }
        if distance > 0.0 {
            return closest_point + offset / distance * radius;
        }
        // Center inside the tile: leave through the nearest edge.
        let to_minimum = center - tile.minimum;
        let to_maximum = tile.maximum - center;
        let exits = [
            (to_minimum.x, Vec2::new(tile.minimum.x - radius, center.y)),
            (to_maximum.x, Vec2::new(tile.maximum.x + radius, center.y)),
            (to_minimum.y, Vec2::new(center.x, tile.minimum.y - radius)),
            (to_maximum.y, Vec2::new(center.x, tile.maximum.y + radius)),
        ];
        exits
            .into_iter()
            .min_by(|first, second| first.0.total_cmp(&second.0))
            .map_or(center, |(_, exit)| exit)
    }
}

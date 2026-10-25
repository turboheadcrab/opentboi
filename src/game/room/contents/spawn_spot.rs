//! Where `spaw()` puts what it creates (`posw()`, DoAction 29028): random points on a circle
//! around the target that grows by 5 px each try, snapped to their tile's center, until a tile
//! is inside the room, holds no grid object (`levzz(...) < 0.2`) and has no pickup within 20 px.

use glam::Vec2;

use crate::game::original_units::Pixels;
use crate::game::random::Random;
use crate::game::room::contents::RoomContents;
use crate::game::room::grid_kind::{GridObjectKind, TileKind};
use crate::game::room::{COLUMN_COUNT, INTERIOR_SIZE, ROW_COUNT, TILE_SIZE, tile_center};

/// The radius grows by 5 per try (DoAction 29044), starting over past 500 (DoAction 29040).
const RADIUS_STEP: Pixels = Pixels(5.0);
const MAXIMUM_RADIUS: Pixels = Pixels(500.0);
/// A spot is taken if a pickup lies closer than this on both axes (DoAction 29079, DoAction 29081).
const PICKUP_CLEARANCE: Pixels = Pixels(20.0);
/// Enough tries to find any free tile; the original loops until it does.
const MAXIMUM_TRIES: usize = 10_000;
/// A taken spot is still given away one time in this many: the pickup test is skipped then
/// (DoAction 29072).
const PICKUP_TEST_SKIPPED_ONE_IN: i32 = 100;

/// A circle a spawned thing must stay out of: how enemies keep clear of each other (50 px) and
/// of Isaac (150 px in the challenge room) when they appear (DoAction 29053, DoAction 29056).
#[derive(Clone, Copy, Debug)]
pub struct Clearance {
    pub center: Vec2,
    pub distance: Pixels,
}

impl Clearance {
    fn contains(self, point: Vec2) -> bool {
        let offset = point - self.center;
        offset.x.abs() < self.distance.get()
            && offset.y.abs() < self.distance.get()
            && offset.length() < self.distance.get()
    }
}

impl RoomContents {
    /// `spaw()`: a free tile's center near `target`, at least `radius + 5` away when the room
    /// allows.
    pub fn spawn_spot(&self, target: Vec2, radius: Pixels, random: &mut Random) -> Vec2 {
        self.spawn_spot_clear_of(target, radius, &[], random)
    }

    /// [`Self::spawn_spot`] for something that also keeps out of the `clearances` (a try that lands
    /// in one is thrown away, DoAction 29061, DoAction 29062).
    pub fn spawn_spot_clear_of(
        &self,
        target: Vec2,
        radius: Pixels,
        clearances: &[Clearance],
        random: &mut Random,
    ) -> Vec2 {
        let mut radius = radius;
        for _ in 0..MAXIMUM_TRIES {
            if radius > MAXIMUM_RADIUS {
                radius = Pixels::ZERO;
            }
            radius += RADIUS_STEP;
            let point = target + random.direction(radius.get());
            if clearances.iter().any(|clearance| clearance.contains(point)) {
                continue;
            }
            let skips_pickup_test = random.below(PICKUP_TEST_SKIPPED_ONE_IN) == 0;
            if let Some(center) = tile_containing(point)
                && self.is_free_for_spawning(center)
                && (skips_pickup_test || !self.pickup_near(center))
            {
                return center;
            }
        }
        self.free_spot_near(target)
    }

    fn pickup_near(&self, center: Vec2) -> bool {
        self.pickups.positions().any(|position| is_near(position, center))
    }

    fn is_free_for_spawning(&self, center: Vec2) -> bool {
        let holds_object = self.grid_objects.iter().any(|object| {
            object.center() == center
                && (object.blocks_walking()
                    || object.is_spider_web()
                    || object.kind() == GridObjectKind::Tile(TileKind::Spikes))
        });
        !holds_object && !self.machines.positions().any(|position| is_near(position, center))
    }
}

fn is_near(position: Vec2, center: Vec2) -> bool {
    (position.x - center.x).abs() < PICKUP_CLEARANCE.get() && (position.y - center.y).abs() < PICKUP_CLEARANCE.get()
}

/// The interior tile `point` lies in, as its center.
fn tile_containing(point: Vec2) -> Option<Vec2> {
    let from_corner = Vec2::new(point.x + INTERIOR_SIZE.x / 2.0, INTERIOR_SIZE.y / 2.0 - point.y);
    let column = (from_corner.x / TILE_SIZE.get()).floor();
    let row = (from_corner.y / TILE_SIZE.get()).floor();
    let inside = (0.0..COLUMN_COUNT as f32).contains(&column) && (0.0..ROW_COUNT as f32).contains(&row);
    inside.then(|| {
        tile_center(
            crate::whole_numbers::clamped_u32(column),
            crate::whole_numbers::clamped_u32(row),
        )
    })
}

#[cfg(test)]
mod tests;

//! The room as the enemies see it, in stage coordinates: what is solid, where the breakables
//! and the player are, and whether something blocks the view.

use glam::Vec2;

use crate::game::bounds::Bounds;
use crate::game::room::enemies::fly::FlySurroundings;
use crate::game::room::enemies::walk_map::WalkMap;
use crate::game::room::grid_object::GridObject;
use crate::game::room::{stage_to_world, world_to_stage};

/// `linechecky()`: the line is sampled every 10 pixels, at the corners of a 4-pixel square.
const SIGHT_STEP: f32 = 10.0;
const SIGHT_CORNER: f32 = 2.0;

/// The room's grid objects and the player (stage coordinates), and what walkers make of them.
pub(super) struct Grid<'objects> {
    pub objects: &'objects [GridObject],
    pub player: Vec2,
    /// How fast the player moves, per frame.
    pub player_velocity: Vec2,
    pub walk_map: WalkMap,
    /// What walkers cannot enter, in world coordinates.
    pub solid_tiles: Vec<Bounds>,
}

impl<'objects> Grid<'objects> {
    pub fn new(objects: &'objects [GridObject], player: Vec2) -> Self {
        Self {
            objects,
            player,
            player_velocity: Vec2::ZERO,
            walk_map: WalkMap::new(objects),
            solid_tiles: objects
                .iter()
                .filter(|object| object.blocks_walking())
                .map(GridObject::tile_bounds)
                .collect(),
        }
    }
}

impl Grid<'_> {
    pub fn with_player_velocity(self, player_velocity: Vec2) -> Self {
        Self {
            player_velocity,
            ..self
        }
    }
}

impl FlySurroundings for Grid<'_> {
    fn is_solid(&self, stage_point: Vec2) -> bool {
        let point = stage_to_world(stage_point);
        self.objects
            .iter()
            .any(|object| object.blocks_tears() && object.tile_bounds().contains(point))
    }

    fn breakable_cell(&self, stage_point: Vec2) -> Option<Vec2> {
        let point = stage_to_world(stage_point);
        self.objects
            .iter()
            .find(|object| object.is_intact_breakable() && object.tile_bounds().contains(point))
            .map(|object| world_to_stage(object.center()))
    }

    fn player(&self) -> Vec2 {
        self.player
    }

    fn in_sight(&self, from: Vec2, target: Vec2) -> bool {
        let distance = from.distance(target);
        let step = (target - from).normalize_or_zero() * SIGHT_STEP;
        let corners = [
            Vec2::new(SIGHT_CORNER, SIGHT_CORNER),
            Vec2::new(-SIGHT_CORNER, SIGHT_CORNER),
            Vec2::new(-SIGHT_CORNER, -SIGHT_CORNER),
            Vec2::new(SIGHT_CORNER, -SIGHT_CORNER),
        ];
        let mut point = from;
        let mut travelled = 0.0;
        while travelled < distance {
            point += step;
            if corners.iter().any(|&corner| self.is_solid(point + corner)) {
                return false;
            }
            travelled += SIGHT_STEP;
        }
        true
    }
}

//! How a walker moves and ages.

use super::{Walker, WalkerClip, WalkerState};
use crate::game::bounds::Bounds;
use crate::game::original_units::Frames;
use crate::game::room::enemies::EnemyParameters;
use crate::game::room::{stage_to_world, world_to_stage};

/// `sideflip()`: it turns when its sideways speed against its facing, at the clip's scale in
/// percent, passes this many percent.
const TURN_PERCENT: f32 = 100.0;
const PERCENT: f32 = 100.0;
/// Pushed this far or more by a wall or a rock, it counts as pressed against it.
const CORNERED_MARGIN: f32 = 0.01;

impl Walker {
    /// Moves, kept inside the room and out of rocks and pits, and ages.
    pub fn advance(
        &mut self,
        elapsed: Frames,
        solid_tiles: &[Bounds],
        room_interior: Bounds,
        parameters: &EnemyParameters,
    ) {
        let radius = self.kind.body(parameters).size;
        self.flash_left = (self.flash_left - elapsed).max(Frames::ZERO);
        self.previous_position = self.position;
        let wanted = stage_to_world(self.position + self.velocity * elapsed);
        let mut world = room_interior.shrunk_by(radius).clamp(wanted);
        for tile in solid_tiles {
            world = tile.push_circle_out(world, radius);
        }
        // Pressed against a wall or a rock, as far as it wanted to go or not (`gh`).
        self.blob.cornered = world.distance(wanted) > CORNERED_MARGIN;
        self.position = world_to_stage(world);
        self.age += elapsed;
        self.clip_age += elapsed;
        if let WalkerClip::Walking { since } = &mut self.clip {
            *since += elapsed;
        }
        if let WalkerClip::Shooting { since } = &mut self.clip {
            *since += elapsed;
        }
        if let Some(age) = &mut self.shooter.animation_age {
            *age += elapsed;
            if *age >= parameters.horfs.shooting_frames {
                self.shooter.animation_age = None;
            }
        }
        if let WalkerState::Dying(frames) = &mut self.state {
            *frames += elapsed;
        }
    }

    /// `sideflip()`: faces `towards`' side once that is far enough (as a percent of the clip's scale)
    /// against its facing.
    pub(super) fn face(&mut self, towards: f32) {
        let facing = if self.mirrored { -1.0 } else { 1.0 };
        if towards * facing * self.scale * PERCENT < -TURN_PERCENT {
            self.mirrored = !self.mirrored;
        }
    }
}

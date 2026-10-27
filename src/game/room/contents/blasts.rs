//! Blasts in the room: the explosion shown and the hits it deals, the rubble a blown rock throws
//! into a pit, and the blast centers handed on to whatever else they hurt.

use glam::Vec2;

use crate::game::attacks::explosion::Explosion;
use crate::game::bounds::Bounds;
use crate::game::input_state::snap_to_cardinal;
use crate::game::original_units::distance;
use crate::game::parameters::ExplosionParameters;
use crate::game::room::TILE_SIZE;
use crate::game::room::contents::{PendingHit, RoomContents};
use crate::game::room::grid_object::HitStrength;
use crate::game::room::sight::SightGrid;
use crate::game::sound_event::SoundEvent;

impl RoomContents {
    /// `bloww`: the rubble is thrown one tile away from the blast along the dominant axis, filling
    /// a pit there.
    pub(super) fn bridge_pit_behind(&mut self, rock_index: usize, blast_center: Vec2) {
        let rock_center = self.grid_objects[rock_index].center();
        let Some(away) = snap_to_cardinal(rock_center - blast_center) else {
            return;
        };
        let landing_tile = Bounds::centered(rock_center + away * TILE_SIZE, Vec2::splat((TILE_SIZE / 2.0).get()));
        if let Some(pit) = self
            .grid_objects
            .iter_mut()
            .find(|object| landing_tile.contains(object.center()))
        {
            pit.bridge_if_pit();
        }
    }

    /// Shows the blast and returns the strong hits it deals to every object in reach.
    pub(super) fn start_explosion(&mut self, center: Vec2, sounds: &mut Vec<SoundEvent>) -> Vec<PendingHit> {
        sounds.push(SoundEvent::Explosion);
        self.explosions.push(Explosion::starting_at(center));
        self.scorch_marks.push(center);
        self.blasts.push(center);
        self.objects_within(center)
            .map(|near| (near, HitStrength::Explosion, Some(center)))
            .collect()
    }

    /// Blast centers since the last call, and how blasts behave.
    pub fn take_blasts(&mut self) -> (Vec<Vec2>, ExplosionParameters) {
        (std::mem::take(&mut self.blasts), self.parameters.explosion)
    }

    /// The room's grid for the line checks, with a door in every doorway.
    pub fn sight_grid(&self) -> SightGrid {
        SightGrid::new(&self.grid_objects)
    }

    /// Objects whose tile reaches into the blast radius and whose center the blast sees
    /// (`linecheckxx()` in `gosplash()`'s look at the grid).
    fn objects_within(&self, center: Vec2) -> impl Iterator<Item = usize> {
        let radius = self.parameters.explosion.radius;
        let sight = self.sight_grid();
        (0..self.grid_objects.len()).filter(move |&object_index| {
            let object = &self.grid_objects[object_index];
            distance(object.tile_bounds().clamp(center), center) <= radius
                && sight.blast_reaches(center, object.center())
        })
    }
}

#[cfg(test)]
mod tests;

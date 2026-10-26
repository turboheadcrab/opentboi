//! The enemies' turn in the room: they act, then the breakables they pecked take the hit
//! (`killshit()`, the same weak hit a tear deals); a peck that lands turns the fly into a Pooter.

use glam::Vec2;

use crate::game::original_units::Frames;
use crate::game::random::Random;
use crate::game::room::PlayerSighting;
use crate::game::room::contents::RoomContents;
use crate::game::room::grid_object::{HitOutcome, HitStrength};
use crate::game::room::interior_bounds;
use crate::game::sound_event::SoundEvent;

impl RoomContents {
    pub(super) fn advance_enemies(
        &mut self,
        elapsed: Frames,
        player: PlayerSighting,
        random: &mut Random,
        sounds: &mut Vec<SoundEvent>,
    ) {
        self.enemies.watch_player_velocity(player.velocity);
        let pecks = self.enemies.advance(
            elapsed,
            &self.grid_objects,
            player.position,
            interior_bounds(),
            random,
            sounds,
        );
        for (fly_index, cell) in pecks {
            if self.hit_breakable_at(cell, random, sounds) {
                self.enemies.peck_landed(fly_index, random);
            }
        }
        self.spill_enemy_blood(random);
        self.enemies.remove_finished();
    }

    /// `killshit()` on the breakable at `cell` (world), from a fly's peck or the unicorn's
    /// hooves; whether it took the hit.
    pub fn hit_breakable_at(&mut self, cell: Vec2, random: &mut Random, sounds: &mut Vec<SoundEvent>) -> bool {
        let Some(index) = self
            .grid_objects
            .iter()
            .position(|object| object.is_intact_breakable() && object.tile_bounds().contains(cell))
        else {
            return false;
        };
        !matches!(
            self.hit(index, HitStrength::Tear, random, sounds),
            None | Some(HitOutcome::Resisted)
        )
    }
}

#[cfg(test)]
mod tests;

//! The Pooter's own rules, run before the flies' shared ones (its case falls through to them): it
//! stays within the room's middle (`bord()`), and shoots the player (`firemode()`, DoAction
//! 9515–9690): on every 7th frame (staggered by its place in the room), with its
//! cooldown spent and the player within 200 pixels and in sight, it plays its shot (`d` frame
//! 5), nudges its drift towards the player, and releases the shot 8 frames later. While the
//! cooldown runs it slows down.

use glam::Vec2;

use crate::game::original_units::Frames;
use crate::game::parameters::PooterParameters;
use crate::game::room::enemies::fly::{Fly, FlySurroundings};

impl Fly {
    /// One frame of a Pooter's own rules; the player's position when a shot leaves.
    pub(super) fn pooter_turn(
        &mut self,
        frame: Frames,
        surroundings: &impl FlySurroundings,
        parameters: PooterParameters,
    ) -> Option<Vec2> {
        self.position = parameters.border.keep_within(self.position).position;
        let player = surroundings.player();
        let position = self.position;
        let turn = self.shooter.turn(frame, position, player, parameters, || {
            surroundings.in_sight(position, player)
        });
        if let Some((away_from_player, distance)) = turn.began {
            self.drift -= away_from_player * (parameters.shot_drift_nudge / distance);
        }
        if self.shooter.is_cooling_down() {
            self.velocity *= parameters.firing_velocity_retention;
            self.drift *= parameters.firing_drift_retention;
        }
        turn.release
    }
}

#[cfg(test)]
mod tests;

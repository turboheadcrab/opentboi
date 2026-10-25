//! Horf: it never moves of its own accord (each frame it keeps 70 % of whatever speed a hit or a
//! bump gave it) and shoots the player the way a Pooter does (`firemode(200, 4)`, see
//! `shooter.rs`): the shot leaves 8 frames after its mouth opened.

use super::{Walker, WalkerShot, WalkerTurn};
use crate::game::original_units::Frames;
use crate::game::room::enemies::EnemyParameters;
use crate::game::room::enemies::fly::FlySurroundings;
use crate::game::room::enemies::surroundings::Grid;

impl Walker {
    /// One frame of Horf's case of `smarts()`.
    pub(super) fn shoot(&mut self, frame: Frames, grid: &Grid, parameters: &EnemyParameters) -> WalkerTurn {
        self.velocity *= parameters.horfs.velocity_retention.0;
        let (position, player) = (self.position, grid.player);
        let turn = self.shooter.turn(frame, position, player, parameters.pooters, || {
            grid.in_sight(position, player)
        });
        self.mirrored = self.shooter.faces_right;
        WalkerTurn {
            shot: turn.release.map(WalkerShot::Aimed),
            ..WalkerTurn::default()
        }
    }
}

//! `firemode()` (DoAction 9515–9690) for the enemies that shoot the player when close and in sight
//! (the Pooter, Horf): on every 7th frame (staggered by the enemy's place in the room), with its
//! cooldown spent and the player within 200 pixels and in sight, it plays its shot (`d` frame 5)
//! and releases the shot 8 frames later; then it waits 4 such checks.

use glam::Vec2;

use crate::game::original_units::Frames;
use crate::game::parameters::PooterParameters;

#[derive(Clone, Copy, Debug, Default)]
pub struct Shooter {
    /// `fire`: shot checks until it may shoot again.
    cooldown: i32,
    /// `fir`: counts up each frame; the shot leaves once it is past 0.
    release_counter: Option<Frames>,
    /// Time since the shooting clip started, while it plays.
    pub animation_age: Option<Frames>,
    /// `e`: staggers the shot checks of the room's enemies.
    pub phase: u32,
    /// Mirrored to face a player on its right (`sideflip()`).
    pub faces_right: bool,
}

/// What a frame of `firemode()` did.
#[derive(Clone, Copy, Debug, Default)]
pub struct ShooterTurn {
    /// A shot started: the vector from the player to the shooter and its length.
    pub began: Option<(Vec2, f32)>,
    /// A shot leaves, aimed at this point (the player).
    pub release: Option<Vec2>,
}

impl Shooter {
    /// Ready to shoot, at its place among the room's enemies.
    pub fn with_phase(phase: u32) -> Self {
        Self {
            phase,
            ..Self::default()
        }
    }

    /// Whether the cooldown is still running.
    pub fn is_cooling_down(&self) -> bool {
        self.cooldown > 0
    }

    /// One frame of `firemode()` for a shooter at `position` (stage coordinates), the player at
    /// `player`; `in_sight` is asked only when everything else allows a shot.
    pub fn turn(
        &mut self,
        frame: Frames,
        position: Vec2,
        player: Vec2,
        parameters: PooterParameters,
        in_sight: impl FnOnce() -> bool,
    ) -> ShooterTurn {
        let away_from_player = position - player;
        let distance = away_from_player.length();
        let mut turn = ShooterTurn::default();
        if (frame + Frames::from_number(self.phase)).is_on_cycle(parameters.shot_check_every, Frames::ZERO) {
            if self.cooldown <= 0 && distance > 0.0 && distance < parameters.sight_reach.get() && in_sight() {
                self.faces_right = away_from_player.x < 0.0;
                self.animation_age = Some(Frames::ZERO);
                self.cooldown = i32::from(parameters.cooldown_checks);
                self.release_counter = Some(Frames::ONE - parameters.shot_delay);
                turn.began = Some((away_from_player, distance));
            }
            self.cooldown -= 1;
        }
        if let Some(counter) = self.release_counter {
            if counter > Frames::ZERO {
                self.release_counter = None;
                self.faces_right = away_from_player.x < 0.0;
                turn.release = Some(player);
            } else {
                self.release_counter = Some(counter + Frames::ONE);
            }
        }
        turn
    }
}

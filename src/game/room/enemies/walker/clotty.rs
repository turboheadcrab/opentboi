//! The Clotty (type 15): the case for it in `smarts()` (DoAction 24381–24470). It starts in a random
//! one of two modes. Wandering (`randrun()`, as the headless body does) turns into shooting once it
//! arrives; shooting plays the shot clip, fires four shots along the axes when the clip is past its
//! tenth frame (`quadf()`), and goes back to wandering once the clip is over, or sooner at random.
//! The blood it drips on the floor is not drawn yet.

use super::{ClottyMode, Walker, WalkerClip, WalkerShot, WalkerTurn};
use crate::game::odds::Odds;
use crate::game::original_units::Frames;
use crate::game::parameters::ClottyParameters;
use crate::game::random::Random;
use crate::game::room::enemies::EnemyParameters;
use crate::game::room::enemies::surroundings::Grid;

impl Walker {
    /// One frame of the Clotty's case of `smarts()`.
    pub(super) fn clot(
        &mut self,
        frame: Frames,
        grid: &Grid,
        parameters: &EnemyParameters,
        random: &mut Random,
    ) -> WalkerTurn {
        let clotty = parameters.clottys;
        let mut turn = WalkerTurn::default();
        self.end_finished_shot(&clotty);
        self.velocity *= clotty.velocity_retention;
        match self.blob.mode {
            ClottyMode::Undecided => {
                self.blob.shot_done = false;
                self.blob.mode = if random.happens(Odds::EVEN) {
                    ClottyMode::Shooting
                } else {
                    ClottyMode::Wandering
                };
                self.blob.mode_frame = frame;
            }
            ClottyMode::Shooting => self.shoot_four_ways(frame, &clotty, random, &mut turn),
            ClottyMode::Wandering => {
                self.blob.mode_frame = frame;
                self.wander_to_a_stop(grid, parameters, random);
            }
        }
        turn
    }

    /// The shot clip has played out: it is standing again.
    fn end_finished_shot(&mut self, clotty: &ClottyParameters) {
        if matches!(self.clip, WalkerClip::Shooting { since } if since >= clotty.shooting_frames) {
            self.start_clip(WalkerClip::Standing);
        }
    }

    fn shoot_four_ways(
        &mut self,
        frame: Frames,
        clotty: &ClottyParameters,
        random: &mut Random,
        turn: &mut WalkerTurn,
    ) {
        let standing = matches!(self.clip, WalkerClip::Standing);
        let waited = frame - self.blob.mode_frame > clotty.mode_minimum_frames;
        if (standing && self.blob.shot_done) || (random.happens(clotty.mode_change_odds) && waited) {
            self.blob.mode = ClottyMode::Wandering;
            self.blob.shot_done = false;
            return;
        }
        if !matches!(self.clip, WalkerClip::Shooting { .. }) {
            self.start_clip(WalkerClip::Shooting { since: Frames::ZERO });
        }
        if let WalkerClip::Shooting { since } = self.clip
            && since >= clotty.shot_after
            && !self.blob.shot_done
        {
            turn.shot = Some(WalkerShot::Cross(clotty.shot_speed));
            turn.squished = true;
            self.blob.shot_done = true;
        }
    }

    /// `randrun()`: walking to a spot nearby, then it stops and shoots.
    fn wander_to_a_stop(&mut self, grid: &Grid, parameters: &EnemyParameters, random: &mut Random) {
        let arrived = self.wander_step(grid, &parameters.headless, random);
        if self.goal.is_some() && !matches!(self.clip, WalkerClip::Walking { .. }) {
            self.start_clip(WalkerClip::Walking { since: Frames::ZERO });
        }
        if arrived {
            self.blob.mode = ClottyMode::Shooting;
            self.blob.shot_done = false;
        }
    }

    /// Changing clip starts the new one over.
    pub(super) fn start_clip(&mut self, clip: WalkerClip) {
        self.clip = clip;
        self.clip_age = Frames::ZERO;
    }
}

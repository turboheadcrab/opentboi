//! The Hopper (type 29): `randruny()` (DoAction 8662–8830) for a plain one. With no leap going it
//! picks a spot: the player's, if he (two frames ahead at his speed) is within 130 pixels, else a
//! point 95 pixels off in a random direction, if the tile there is free. A leap plays the frames of
//! its hop clip: it crouches for the first four (a tenth of the pull towards the spot), pulls with
//! the full strength through the air (a tenth of the distance, at most 3, each frame), lands from
//! the clip's 21st frame on (slowing to 40 %, no pull, two more frames) and picks the next spot at
//! once, so it hops without a pause. It shakes the floor on the 19th frame (`Meat_impacts_`).
//!
//! The original also holds the clip back in mid-air when the leap is long and fast, and ends the
//! leap early when close or slow; those tests read `enfget()` of a sum with no second argument,
//! which is always the standstill speed, so only the first never happens and the second never
//! matters before the clip ends: neither is ported.

use glam::Vec2;

use super::{Walker, WalkerTurn};
use crate::game::parameters::HopperParameters;
use crate::game::random::Random;
use crate::game::room::enemies::EnemyParameters;
use crate::game::room::enemies::surroundings::Grid;

/// `enfget()` of nothing: so that dividing by a distance is safe.
const NO_DISTANCE: f32 = 0.001;

/// Where a Hopper is in its leaping.
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct Hop {
    /// `xpp`, `ypp`: the spot it leaps to.
    target: Option<Vec2>,
    /// The hop clip is playing (`d` frame 2), and its frame (`d.d._currentframe`).
    hopping: bool,
    clip_frame: u8,
}

impl Hop {
    /// The hop clip's frame while it plays, counted from 1.
    pub(super) fn playing_frame(self) -> Option<u8> {
        self.hopping.then_some(self.clip_frame)
    }
}

impl Walker {
    /// One frame of the Hopper's case of `smarts5()`.
    pub(super) fn hop(&mut self, grid: &Grid, parameters: &EnemyParameters, random: &mut Random) -> WalkerTurn {
        let hopper = parameters.hoppers;
        let turn = WalkerTurn {
            landed: self.hop.playing_frame() == Some(hopper.landing_sound_frame),
            ..WalkerTurn::default()
        };
        if self.hop.target.is_none() {
            if self.hop.hopping {
                self.hop.clip_frame = 1;
            }
            self.hop.target = pick_target(self.position, grid, &hopper, random);
        }
        if let Some(target) = self.hop.target {
            self.leap(target, &hopper);
        }
        self.velocity *= hopper.velocity_retention;
        turn
    }

    /// One frame of the leap towards `target`: the hop clip steps on (DoAction 8762), a tenth of the
    /// pull while crouching (DoAction 8790, DoAction 8791), landing at 40 % of the speed (DoAction
    /// 8778, DoAction 8779), turning to face a spot more than 8 px sideways (DoAction 8824).
    fn leap(&mut self, target: Vec2, hopper: &HopperParameters) {
        if !self.hop.hopping {
            self.hop.hopping = true;
            self.hop.clip_frame = 1;
        }
        self.hop.clip_frame = (self.hop.clip_frame + 1).min(hopper.hop_frame_count);
        let mut away = self.position - target;
        let distance = away.length().max(NO_DISTANCE);
        if self.hop.clip_frame + hopper.landing_frames > hopper.hop_frame_count {
            self.velocity *= hopper.landing_retention;
            away = Vec2::ZERO;
            self.hop.clip_frame = (self.hop.clip_frame + 1).min(hopper.hop_frame_count);
            if self.hop.clip_frame + hopper.ending_frames > hopper.hop_frame_count {
                self.hop.target = None;
            }
        } else if self.hop.clip_frame < hopper.pull_from_frame {
            away *= hopper.crouch_pull;
        }
        let pull = (distance * hopper.pull_per_distance).min(hopper.pull_maximum) / distance;
        self.velocity -= away * pull;
        if away.x.abs() > hopper.turn_beyond.get() {
            self.face(-away.x);
        }
    }
}

/// The spot for the next leap, if the tile there is free: the player's own place when he is near
/// (DoAction 8713, DoAction 8714), landing within 3 px of the tile's center (DoAction 8753, DoAction
/// 8754).
fn pick_target(from: Vec2, grid: &Grid, hopper: &HopperParameters, random: &mut Random) -> Option<Vec2> {
    let player_ahead = grid.player + grid.player_velocity * hopper.player_lead.get();
    let spot = if from.distance(player_ahead) < hopper.player_reach.get() {
        grid.player
    } else {
        from + random.direction(hopper.wander_distance.get())
    };
    let spot = spot.clamp(hopper.stage_minimum, hopper.stage_maximum);
    let center = grid.walk_map.free_spot(spot)?;
    Some(center + random.direction(hopper.scatter.get()))
}

//! The flesh mound (type 77), the last piece of the family that splits: it wanders as the headless
//! body does (`randrun()`), in the loop of its walking clip. Each frame it keeps 80 % of its speed,
//! 60 % more once the clip is past its 19th frame, and it picks a new spot each time the clip starts
//! over (`bawssmart2()`, DoAction 15677–15954). Appearing, it keeps 45 % on top of the 80 %.

use super::{Walker, WalkerTurn};
use crate::game::original_units::Frames;
use crate::game::random::Random;
use crate::game::room::enemies::EnemyParameters;
use crate::game::room::enemies::surroundings::Grid;

impl Walker {
    /// One frame of the flesh mound's case: its walking clip's frame (from 1) is the frames since it
    /// finished appearing, around the clip.
    pub(super) fn crawl(&mut self, grid: &Grid, parameters: &EnemyParameters, random: &mut Random) -> WalkerTurn {
        let mound = parameters.flesh_mounds;
        let since_appeared: Frames = self.age - mound.appearing_frames;
        let clip_frame = since_appeared.looped_frame(mound.walking_frame_count) + 1;
        self.velocity *= mound.velocity_retention;
        let _ = self.wander_step(grid, &parameters.headless, random);
        if clip_frame == 1 {
            self.goal = None;
        }
        if clip_frame > mound.late_clip_frame {
            self.velocity *= mound.late_clip_retention;
        }
        WalkerTurn::default()
    }
}

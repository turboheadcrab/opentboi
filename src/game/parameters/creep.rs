//! Blood creep, the puddles bosses and some enemies leave (`parc("bloo")`, DoAction 3935; hit by
//! `firecheck()`, DoAction 9992–10010; the clip `bloo`, 1494).

use crate::game::original_units::{Frames, Hearts, Pixels};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CreepParameters {
    /// The clip has 141 frames and ends the puddle.
    pub lifetime: Frames,
    /// It keeps its size through frame 101, then loses 3/143 of it each frame.
    pub full_size_frames: Frames,
    pub shrink_per_frame: f32,
    /// The game tests a point against the splat's drawn shape, which wobbles between four looks of
    /// 480 to 930 pixels; a circle of the mean area (about 700 pixels) stands in for it.
    pub hit_radius: Pixels,
    /// Standing in it hurts by a heart, or, in dark rooms and spider boss fights, slows for 10 frames.
    pub damage: Hearts,
    pub slow_frames: Frames,
}

pub const ORIGINAL_CREEP: CreepParameters = CreepParameters {
    lifetime: Frames(141.0),
    full_size_frames: Frames(101.0),
    shrink_per_frame: 3.0 / 143.0,
    hit_radius: Pixels(15.0),
    damage: Hearts::ONE,
    slow_frames: Frames(10.0),
};

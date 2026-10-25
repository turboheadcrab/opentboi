//! How bodies meet walls and obstacles (`physix()`, DoAction 25960–26112).

use crate::game::original_units::{Frames, Pixels, PixelsPerFrame, Retention};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WallParameters {
    /// Moving into a wall, the part of the velocity into the wall is taken away 1.8 times (a slight
    /// rebound) and 0.1 px/frame added away from it.
    pub rebound: f32,
    pub push_away: PixelsPerFrame,
    /// The reach is the body's size plus this; a hard impact also slows the body down, by half the
    /// share of that reach it pressed in.
    pub reach_margin: Pixels,
    pub impact_damping: f32,
    /// Isaac against a wall keeps 88 % of his speed every frame.
    pub player_friction: Retention,
    /// After pressing into a wall, diagonal input keeps only the axis along it (at full speed) for
    /// this long.
    pub player_slide_frames: Frames,
}

pub const ORIGINAL_WALLS: WallParameters = WallParameters {
    rebound: 1.8,
    push_away: PixelsPerFrame(0.1),
    reach_margin: Pixels(2.0),
    impact_damping: 0.5,
    player_friction: Retention(0.88),
    player_slide_frames: Frames(4.0),
};

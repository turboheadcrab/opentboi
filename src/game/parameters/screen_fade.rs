//! The black the screen flashes when it changes (`goblack()`, Timeline 1394–1409): the main timeline's
//! cover `overlay2` jumps to full strength (Timeline 1407) and fades out by 7 percentage points a
//! frame (Timeline 1543), so a new run (the restart key, "Replay?") comes up out of black over about
//! half a second.

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScreenFadeParameters {
    /// How opaque the cover starts: all of it.
    pub start_opacity: f32,
    /// How much of the opacity it loses each frame.
    pub step_per_frame: f32,
}

pub const ORIGINAL_SCREEN_FADE: ScreenFadeParameters = ScreenFadeParameters {
    start_opacity: 1.0,
    step_per_frame: 0.07,
};

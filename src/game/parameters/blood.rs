//! Blood splats on the floor (`splater()`): what leaves them, how often and how big. They are drawn
//! into the floor's `splat` bitmap with the blood tint `bloc`; the bitmap is shown at 70 % and also
//! holds the craters blasts leave.

use crate::game::odds::Odds;
use crate::game::original_units::{Frames, Pixels};

/// A splat's scale: `least` plus a random fraction of `spread`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SplatSize {
    pub least: f32,
    pub spread: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SplatParameters {
    /// The `splat` bitmap's opacity (`_alpha = 70`), for splats and craters alike.
    pub layer_opacity: f32,
    /// `bloc`: every 7 frames each color channel of the blood is made 0.85 to 1 of itself.
    pub tint_every: Frames,
    pub tint_least: f32,
    pub tint_spread: f32,
    /// `splater()` moves each splat by `crand(5)` and 5 px up.
    pub jitter: Pixels,
    pub lift: Pixels,
    /// The red blood's looks, the blood clip's first ten frames (`spl` 0).
    pub look_count: u8,
    /// A dying enemy splats on each of its death clip's first three frames, a Pooter smaller.
    pub dying_frames: Frames,
    pub dying: SplatSize,
    pub pooter_dying_factor: f32,
    /// A Gaper losing its head (`hurt()`).
    pub beheaded: SplatSize,
    /// The Clotty, each fifth frame (offset by its number).
    pub clotty_every: Frames,
    pub clotty: SplatSize,
    /// The headless body, one frame in ten (any look but the first, or the last one drawn).
    pub headless_odds: Odds,
    pub headless: SplatSize,
    /// A tear or enemy shot ending, one time in thirty (`physix()`).
    pub landing_odds: Odds,
    pub landing: SplatSize,
    /// An enemy shot in flight, one frame in sixteen (`aistuff()`).
    pub flying_shot_odds: Odds,
    pub flying_shot: SplatSize,
}

pub const ORIGINAL_SPLATS: SplatParameters = SplatParameters {
    layer_opacity: 0.7,
    tint_every: Frames(7.0),
    tint_least: 0.85,
    tint_spread: 0.15,
    jitter: Pixels(5.0),
    lift: Pixels(5.0),
    look_count: 10,
    dying_frames: Frames(3.0),
    dying: SplatSize {
        least: 0.4,
        spread: 0.7,
    },
    pooter_dying_factor: 0.7,
    beheaded: SplatSize {
        least: 0.6,
        spread: 1.0,
    },
    clotty_every: Frames(5.0),
    clotty: SplatSize {
        least: 0.5,
        spread: 0.7,
    },
    headless_odds: Odds::one_in(10),
    headless: SplatSize {
        least: 0.4,
        spread: 0.5,
    },
    landing_odds: Odds::one_in(30),
    landing: SplatSize {
        least: 0.1,
        spread: 0.8,
    },
    flying_shot_odds: Odds::one_in(16),
    flying_shot: SplatSize {
        least: 0.1,
        spread: 0.3,
    },
};

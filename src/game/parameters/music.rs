//! How the music behaves (the sound director in the main timeline's script, `soundfra()`, Timeline
//! 563–1030 (fading by 5 down to a floor of 5: Timeline 600–615; a room jingle pushing it down by 7
//! a frame to the same floor, Timeline 1058–1060, and it coming back by 1 a frame, Timeline
//! 897–903), and `soundy()`, Timeline 194–316).

use crate::game::original_units::Frames;

/// Volumes are percentages of full volume.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MusicParameters {
    /// `mvol`: music plays at 35 % (sounds at 70 %, `soundvol`).
    pub volume: f32,
    /// A fading track loses 5 % a frame and is let go once it is down to 5 %.
    pub fade_step: f32,
    pub fade_floor: f32,
    /// The sounds' volume, which music is measured against.
    pub effects_volume: f32,
    /// While a room jingle plays (until its last second), the music loses this much each frame,
    /// down to `duck_floor`, and gets back `recover_step` a frame afterwards.
    pub duck_step: f32,
    pub duck_floor: f32,
    pub recover_step: f32,
    /// The music is let back up this long before the jingle ends.
    pub jingle_tail: Frames,
}

pub const ORIGINAL_MUSIC: MusicParameters = MusicParameters {
    volume: 35.0,
    fade_step: 5.0,
    fade_floor: 5.0,
    effects_volume: 70.0,
    duck_step: 7.0,
    duck_floor: 5.0,
    recover_step: 1.0,
    jingle_tail: Frames(30.0),
};

//! Dying: `playerhurt()` at no health plays Isaac's death clip and stops the controls, then the
//! death note shows once the clip has played (DoAction 29336–29452 and 31577).

use crate::game::original_units::{Frames, Retention};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DeathParameters {
    /// Isaac slides to a stop, deaf to the controls.
    pub velocity_retention: Retention,
    /// `isaacdies.wav` plays and the game-over count starts on the death clip's 32nd frame (31
    /// from 0).
    pub sound_frame: Frames,
    /// The note shows this long after the sound.
    pub note_after_sound: Frames,
}

pub const ORIGINAL_DEATH: DeathParameters = DeathParameters {
    velocity_retention: Retention(0.8),
    sound_frame: Frames(31.0),
    note_after_sound: Frames(51.0),
};

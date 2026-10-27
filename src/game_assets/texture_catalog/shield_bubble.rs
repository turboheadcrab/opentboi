//! The bubble round a shielded Isaac (`bub`, clip 5608 placed in his clip at (-40.05, -65.65)): its
//! art lies right of and below its origin, so the texture is centered 40 px in from it each way.

use swf::CharacterId;

use super::TextureSource;
use crate::game_assets::vector_art::{FrameSelection, NO_OVERRIDES};

const BUBBLE_CLIP: CharacterId = 5608;
const BUBBLE_CENTER: f32 = 40.0;

pub(super) fn source(frame: u8) -> TextureSource {
    TextureSource {
        character_id: BUBBLE_CLIP,
        selection: FrameSelection {
            frame_number: 1,
            frames_since_frame_started: u16::from(frame),
        },
        origin_x: BUBBLE_CENTER,
        origin_y: BUBBLE_CENTER,
        overrides: &NO_OVERRIDES,
    }
}

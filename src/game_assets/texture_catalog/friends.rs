//! Isaac's familiars and the creep on the floor: the friends' clip `b3` (1788), whose frame 128 holds the blue fly's wings (`d`,
//! 1699, four frames).

use swf::CharacterId;

use super::{TextureSource, framed};
use crate::game_assets::vector_art::FrameSelection;

const FRIENDS_CLIP: CharacterId = 1788;
const BLUE_FLY_FRAME: u16 = 128;

pub(super) fn blue_fly_source(frame: u8) -> TextureSource {
    TextureSource {
        selection: FrameSelection {
            frame_number: BLUE_FLY_FRAME,
            frames_since_frame_started: u16::from(frame),
        },
        ..framed(FRIENDS_CLIP, BLUE_FLY_FRAME)
    }
}

/// The blood creep clip `bloo`: its frames 1 to 4 show the splat's four looks.
const BLOOD_CREEP_CLIP: CharacterId = 1494;

pub(super) fn blood_creep_source(look: u8) -> TextureSource {
    framed(BLOOD_CREEP_CLIP, u16::from(look) + 1)
}

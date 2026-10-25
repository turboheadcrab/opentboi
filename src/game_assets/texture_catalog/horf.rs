//! Horf `b12` (5024): `d` frame 1 idles (4994, 3 frames), 3 dies (566, 36 frames), 4 appears (5013,
//! 60 frames) and 5 shoots (5023, 20 frames).

use swf::CharacterId;

use super::{TextureSource, framed};
use crate::game::{HorfAnimation, TextureId};
use crate::game_assets::vector_art::{FrameSelection, NO_OVERRIDES, RenderOverrides, Tint};

const HORF_CLIP: CharacterId = 5024;
const IDLE_FRAME: u16 = 1;
const DYING_FRAME: u16 = 3;
const APPEARING_FRAME: u16 = 4;
const SHOOTING_FRAME: u16 = 5;

/// While a hit's flash lasts (`colorit()`): see the flies.
const HURT_TINT: Tint = Tint {
    multiply_per_mille: [400, 500, 500],
    add: [200, 0, 0],
};

static HURT_FLASH: RenderOverrides = RenderOverrides {
    children: &[],
    texts: &[],
    hidden_characters: &[],
    character_frames: &[],
    tint: Some(HURT_TINT),
};

pub(super) fn source_for(texture_id: TextureId) -> TextureSource {
    let TextureId::Horf {
        animation,
        frame,
        flashing,
    } = texture_id
    else {
        return framed(HORF_CLIP, IDLE_FRAME);
    };
    let clip_frame = match animation {
        HorfAnimation::Idle => IDLE_FRAME,
        HorfAnimation::Shooting => SHOOTING_FRAME,
        HorfAnimation::Dying => DYING_FRAME,
        HorfAnimation::Appearing => APPEARING_FRAME,
    };
    TextureSource {
        selection: FrameSelection {
            frame_number: clip_frame,
            frames_since_frame_started: u16::from(frame),
        },
        overrides: if flashing { &HURT_FLASH } else { &NO_OVERRIDES },
        ..framed(HORF_CLIP, clip_frame)
    }
}

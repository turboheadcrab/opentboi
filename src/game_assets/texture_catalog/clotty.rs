//! Clotty `b15` (4902): `d` frame 1 idles (4877, 24 frames), 2 walks (4894, 20 frames), 3 dies (566,
//! 36 frames), 4 appears (4895, 27 frames) and 5 shoots (4900, 24 frames).

use swf::CharacterId;

use super::{TextureSource, framed};
use crate::game::{ClottyAnimation, TextureId};
use crate::game_assets::vector_art::{ChildState, FrameSelection, NO_OVERRIDES, RenderOverrides, Tint};

const CLOTTY_CLIP: CharacterId = 4902;
const IDLE_FRAME: u16 = 1;
const WALKING_FRAME: u16 = 2;
const DYING_FRAME: u16 = 3;
const APPEARING_FRAME: u16 = 4;
const SHOOTING_FRAME: u16 = 5;

/// While a hit's flash lasts (`colorit()`): see the flies.
const HURT_TINT: Tint = Tint {
    multiply_per_mille: [400, 500, 500],
    add: [200, 0, 0],
};

/// Every clip holds a `hx` of three looks (the Clotty, and the stronger ones that share the clip,
/// picked by `alter`); only the first is the plain Clotty's, and the clip would cycle through them.
macro_rules! plain_look {
    ($path:literal, $tint:expr) => {
        RenderOverrides {
            children: &[($path, ChildState::Frame(PLAIN_LOOK))],
            texts: &[],
            hidden_characters: &[],
            character_frames: &[],
            tint: $tint,
        }
    };
}

const PLAIN_LOOK: u16 = 1;

/// By whether it flashes: the idle, walking and shooting clips hold it at `d.hx`, the appearing clip a
/// level deeper.
static IN_STATE_CLIP: [RenderOverrides; 2] = [plain_look!("d.hx", None), plain_look!("d.hx", Some(HURT_TINT))];
static IN_APPEARING_CLIP: [RenderOverrides; 2] = [plain_look!("d.d.hx", None), plain_look!("d.d.hx", Some(HURT_TINT))];
static HURT_FLASH: RenderOverrides = RenderOverrides {
    children: &[],
    texts: &[],
    hidden_characters: &[],
    character_frames: &[],
    tint: Some(HURT_TINT),
};

pub(super) fn source_for(texture_id: TextureId) -> TextureSource {
    let TextureId::Clotty {
        animation,
        frame,
        flashing,
    } = texture_id
    else {
        return framed(CLOTTY_CLIP, IDLE_FRAME);
    };
    let clip_frame = match animation {
        ClottyAnimation::Idle => IDLE_FRAME,
        ClottyAnimation::Walking => WALKING_FRAME,
        ClottyAnimation::Shooting => SHOOTING_FRAME,
        ClottyAnimation::Dying => DYING_FRAME,
        ClottyAnimation::Appearing => APPEARING_FRAME,
    };
    let flash = usize::from(flashing);
    let overrides = match animation {
        ClottyAnimation::Dying if flashing => &HURT_FLASH,
        ClottyAnimation::Dying => &NO_OVERRIDES,
        ClottyAnimation::Appearing => &IN_APPEARING_CLIP[flash],
        _ => &IN_STATE_CLIP[flash],
    };
    TextureSource {
        selection: FrameSelection {
            frame_number: clip_frame,
            frames_since_frame_started: u16::from(frame),
        },
        overrides,
        ..framed(CLOTTY_CLIP, clip_frame)
    }
}

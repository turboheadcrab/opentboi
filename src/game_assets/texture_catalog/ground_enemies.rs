//! The Hopper `b29` (4935): `d` frame 1 sits, 2 hops (4933, 26 frames), 3 dies (566, 36), 4 appears
//! (4934, 26). The flesh mound is the last piece of `b74` (1906): `d` frame 13 walks (1902, 24 frames),
//! 14 dies (566, 36), 15 appears (1903, 25).

use swf::CharacterId;

use super::{TextureSource, framed};
use crate::game::{EnemyTexture, FleshMoundAnimation, HopperAnimation};
use crate::game_assets::vector_art::{ChildState, FrameSelection, NO_OVERRIDES, RenderOverrides, Tint};

const HOPPER_CLIP: CharacterId = 4935;
const HOPPER_IDLE_FRAME: u16 = 1;
const HOPPER_HOPPING_FRAME: u16 = 2;
const HOPPER_DYING_FRAME: u16 = 3;
const HOPPER_APPEARING_FRAME: u16 = 4;
const FLESH_MOUND_CLIP: CharacterId = 1906;
const FLESH_MOUND_WALKING_FRAME: u16 = 13;
const FLESH_MOUND_DYING_FRAME: u16 = 14;
const FLESH_MOUND_APPEARING_FRAME: u16 = 15;

/// While a hit's flash lasts (`colorit()`): see the flies.
const HURT_TINT: Tint = Tint {
    multiply_per_mille: [400, 500, 500],
    add: [200, 0, 0],
};

/// The Hopper's clips hold an `hx` of two looks (plain, and the stronger ones that share the clip),
/// where the plain look is the first.
const PLAIN_LOOK: u16 = 1;

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

/// By whether it flashes: the sitting clip holds its look at `hx`, the others a level deeper at
/// `d.hx`.
static SITTING: [RenderOverrides; 2] = [plain_look!("hx", None), plain_look!("hx", Some(HURT_TINT))];
static IN_CLIP: [RenderOverrides; 2] = [plain_look!("d.hx", None), plain_look!("d.hx", Some(HURT_TINT))];
static HURT_FLASH: RenderOverrides = RenderOverrides {
    children: &[],
    texts: &[],
    hidden_characters: &[],
    character_frames: &[],
    tint: Some(HURT_TINT),
};

fn source(clip: CharacterId, clip_frame: u16, frame: u8, overrides: &'static RenderOverrides) -> TextureSource {
    TextureSource {
        selection: FrameSelection {
            frame_number: clip_frame,
            frames_since_frame_started: u16::from(frame),
        },
        overrides,
        ..framed(clip, clip_frame)
    }
}

pub(super) fn source_for(texture: EnemyTexture) -> TextureSource {
    match texture {
        EnemyTexture::Hopper {
            animation,
            frame,
            flashing,
        } => {
            let flash = usize::from(flashing);
            let (clip_frame, overrides) = match animation {
                HopperAnimation::Idle => (HOPPER_IDLE_FRAME, &SITTING[flash]),
                HopperAnimation::Hopping => (HOPPER_HOPPING_FRAME, &IN_CLIP[flash]),
                HopperAnimation::Appearing => (HOPPER_APPEARING_FRAME, &IN_CLIP[flash]),
                HopperAnimation::Dying => (HOPPER_DYING_FRAME, if flashing { &HURT_FLASH } else { &NO_OVERRIDES }),
            };
            source(HOPPER_CLIP, clip_frame, frame, overrides)
        }
        EnemyTexture::FleshMound {
            animation,
            frame,
            flashing,
        } => {
            let clip_frame = match animation {
                FleshMoundAnimation::Walking => FLESH_MOUND_WALKING_FRAME,
                FleshMoundAnimation::Dying => FLESH_MOUND_DYING_FRAME,
                FleshMoundAnimation::Appearing => FLESH_MOUND_APPEARING_FRAME,
            };
            let overrides = if flashing { &HURT_FLASH } else { &NO_OVERRIDES };
            source(FLESH_MOUND_CLIP, clip_frame, frame, overrides)
        }
    }
}

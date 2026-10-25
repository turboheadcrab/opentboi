//! Mulligan `b16` (4932): `d` frame 1 stands (a body of its own and a hand `hx` on frame 1 calm or 2
//! scared), 2 walks (4922, 22 frames, with a hand `hx` of its own), 3 dies (566, 36 frames), 4
//! appears (4923, 22 frames) and 5 bursts (4930, 70 frames).

use swf::CharacterId;

use super::{TextureSource, framed};
use crate::game::{MulliganAnimation, TextureId};
use crate::game_assets::vector_art::{ChildState, FrameSelection, NO_OVERRIDES, RenderOverrides, Tint};

const MULLIGAN_CLIP: CharacterId = 4932;
const STANDING_FRAME: u16 = 1;
const WALKING_FRAME: u16 = 2;
const DYING_FRAME: u16 = 3;
const APPEARING_FRAME: u16 = 4;
const BURSTING_FRAME: u16 = 5;

/// While a hit's flash lasts (`colorit()`): see the flies.
const HURT_TINT: Tint = Tint {
    multiply_per_mille: [400, 500, 500],
    add: [200, 0, 0],
};

/// The hand `hx` (at `$path`) on `$frame`, its own clip `d` (at `$face`) on the first of its three
/// looks (the others are the bomber's and the like, picked by `alter`); flashing red with `$tint`.
macro_rules! hand {
    ($path:literal, $face:literal, $frame:literal, $tint:expr) => {
        RenderOverrides {
            children: &[
                ($path, ChildState::Frame($frame)),
                ($face, ChildState::Frame(PLAIN_LOOK)),
            ],
            texts: &[],
            hidden_characters: &[],
            character_frames: &[],
            tint: $tint,
        }
    };
}

const PLAIN_LOOK: u16 = 1;

/// The burst has a second hand of its own (`hxx`) with the same looks.
macro_rules! bursting {
    ($tint:expr) => {
        RenderOverrides {
            children: &[
                ("d.hx", ChildState::Frame(PLAIN_LOOK)),
                ("d.hx.d", ChildState::Frame(PLAIN_LOOK)),
                ("d.hxx", ChildState::Frame(PLAIN_LOOK)),
            ],
            texts: &[],
            hidden_characters: &[],
            character_frames: &[],
            tint: $tint,
        }
    };
}

/// By whether it flashes: the standing one's hand calm, scared, then the walking one's, then the
/// bursting one's.
static STANDING_CALM: [RenderOverrides; 2] = [hand!("hx", "hx.d", 1, None), hand!("hx", "hx.d", 1, Some(HURT_TINT))];
static STANDING_SCARED: [RenderOverrides; 2] = [hand!("hx", "hx.d", 2, None), hand!("hx", "hx.d", 2, Some(HURT_TINT))];
static WALKING_CALM: [RenderOverrides; 2] = [
    hand!("d.hx", "d.hx.d", 1, None),
    hand!("d.hx", "d.hx.d", 1, Some(HURT_TINT)),
];
static WALKING_SCARED: [RenderOverrides; 2] = [
    hand!("d.hx", "d.hx.d", 2, None),
    hand!("d.hx", "d.hx.d", 2, Some(HURT_TINT)),
];
static BURSTING: [RenderOverrides; 2] = [bursting!(None), bursting!(Some(HURT_TINT))];
static APPEARING: [RenderOverrides; 2] = [
    hand!("d.hx", "d.hx.d", 1, None),
    hand!("d.hx", "d.hx.d", 1, Some(HURT_TINT)),
];
pub(super) fn source_for(texture_id: TextureId) -> TextureSource {
    let TextureId::Mulligan {
        animation,
        frame,
        scared,
        flashing,
    } = texture_id
    else {
        return framed(MULLIGAN_CLIP, STANDING_FRAME);
    };
    let flash = usize::from(flashing);
    let (clip_frame, chosen) = match animation {
        MulliganAnimation::Standing => (
            STANDING_FRAME,
            if scared {
                &STANDING_SCARED[flash]
            } else {
                &STANDING_CALM[flash]
            },
        ),
        MulliganAnimation::Walking => (
            WALKING_FRAME,
            if scared {
                &WALKING_SCARED[flash]
            } else {
                &WALKING_CALM[flash]
            },
        ),
        MulliganAnimation::Bursting => (BURSTING_FRAME, &BURSTING[flash]),
        MulliganAnimation::Dying => (DYING_FRAME, &NO_OVERRIDES),
        MulliganAnimation::Appearing => (APPEARING_FRAME, &APPEARING[flash]),
    };
    TextureSource {
        selection: FrameSelection {
            frame_number: clip_frame,
            frames_since_frame_started: u16::from(frame),
        },
        overrides: chosen,
        ..framed(MULLIGAN_CLIP, clip_frame)
    }
}

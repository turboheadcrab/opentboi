//! The Gaper `b10` (5051): `d` frame 1 stands (the body with its head `h` on one of 22 frames),
//! 2 walks (5048), 3 dies (566) and 4 appears (5050, with its head `d.h` on the calm or the
//! frantic frame).

use swf::CharacterId;

use super::{TextureSource, framed};
use crate::game::{GaperAnimation, TextureId};
use crate::game_assets::vector_art::{ChildState, FrameSelection, RenderOverrides, Tint};

const GAPER_CLIP: CharacterId = 5051;
const STANDING_FRAME: u16 = 1;
const WALKING_FRAME: u16 = 2;
const DYING_FRAME: u16 = 3;
const APPEARING_FRAME: u16 = 4;
/// The head `h`, the body and the floor shadow inside the standing frame.
const HEAD_CLIP: CharacterId = 5043;
const BODY_CLIP: CharacterId = 4543;
const SHADOW_CLIP: CharacterId = 342;

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

macro_rules! head_overrides {
    ($path:literal, $tint:expr, $hidden:expr; $($frame:literal)*) => {
        [$(RenderOverrides {
            children: &[($path, ChildState::Frame($frame))],
            texts: &[],
            hidden_characters: $hidden,
            character_frames: &[],
            tint: $tint,
        }),*]
    };
}

/// The standing Gaper's head `h` on each of its 22 frames, without and with the hit flash.
static STANDING_HEADS: [[RenderOverrides; 22]; 2] = [
    head_overrides!("h", None, &[]; 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22),
    head_overrides!("h", Some(HURT_TINT), &[]; 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22),
];
/// The head alone on each frame, for drawing over the walking body.
static HEADS_ALONE: [[RenderOverrides; 22]; 2] = [
    head_overrides!("h", None, &[BODY_CLIP, SHADOW_CLIP]; 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22),
    head_overrides!("h", Some(HURT_TINT), &[BODY_CLIP, SHADOW_CLIP]; 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22),
];
static WALKING_BODIES: [RenderOverrides; 2] = [
    RenderOverrides {
        children: &[],
        texts: &[],
        hidden_characters: &[HEAD_CLIP],
        character_frames: &[],
        tint: None,
    },
    RenderOverrides {
        children: &[],
        texts: &[],
        hidden_characters: &[HEAD_CLIP],
        character_frames: &[],
        tint: Some(HURT_TINT),
    },
];
/// The appearing Gaper's head `d.h` on its calm or frantic frame, without and with the hit flash.
static APPEARING_HEADS: [[RenderOverrides; 2]; 2] = [
    head_overrides!("d.h", None, &[]; 1 2),
    head_overrides!("d.h", Some(HURT_TINT), &[]; 1 2),
];

pub(super) fn source_for(texture_id: TextureId) -> TextureSource {
    let TextureId::Gaper {
        animation,
        frame,
        flashing,
    } = texture_id
    else {
        return framed(GAPER_CLIP, STANDING_FRAME);
    };
    let flash = usize::from(flashing);
    let (clip_frame, overrides) = match animation {
        GaperAnimation::Appearing { frantic } => (APPEARING_FRAME, &APPEARING_HEADS[flash][usize::from(frantic)]),
        GaperAnimation::Standing => (
            STANDING_FRAME,
            &STANDING_HEADS[flash][usize::from(frame).min(STANDING_HEADS[flash].len() - 1)],
        ),
        GaperAnimation::Walking => (WALKING_FRAME, &WALKING_BODIES[flash]),
        GaperAnimation::Head => (
            STANDING_FRAME,
            &HEADS_ALONE[flash][usize::from(frame).min(HEADS_ALONE[flash].len() - 1)],
        ),
        GaperAnimation::Dying => (DYING_FRAME, flash_overrides(false)),
    };
    // The standing frame's `frame` picks the head; the rest of the clip stands still.
    let playing_for = if matches!(animation, GaperAnimation::Standing | GaperAnimation::Head) {
        0
    } else {
        frame
    };
    TextureSource {
        selection: FrameSelection {
            frame_number: clip_frame,
            frames_since_frame_started: u16::from(playing_for),
        },
        overrides,
        ..framed(GAPER_CLIP, clip_frame)
    }
}

fn flash_overrides(flashing: bool) -> &'static RenderOverrides {
    if flashing {
        &HURT_FLASH
    } else {
        &crate::game_assets::vector_art::NO_OVERRIDES
    }
}

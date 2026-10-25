//! The headless body `b11` (3483): `d` frame 1 stands (the body 3468 with its stump's blood `bb`,
//! 10 frames), 2 walks (3481, 20 frames), 3 dies (566, 36 frames) and 4 appears (3482, 26 frames,
//! its blood `d.bb`). The blood is hidden unless the body spurts.

use swf::CharacterId;

use super::{TextureSource, framed};
use crate::game::{HeadlessAnimation, TextureId};
use crate::game_assets::vector_art::{ChildState, FrameSelection, RenderOverrides, Tint};

const HEADLESS_CLIP: CharacterId = 3483;
const STANDING_FRAME: u16 = 1;
const WALKING_FRAME: u16 = 2;
const DYING_FRAME: u16 = 3;
const APPEARING_FRAME: u16 = 4;

/// While a hit's flash lasts (`colorit()`): see the flies.
const HURT_TINT: Tint = Tint {
    multiply_per_mille: [400, 500, 500],
    add: [200, 0, 0],
};

macro_rules! overrides {
    ($children:expr, $tint:expr) => {
        RenderOverrides {
            children: $children,
            texts: &[],
            hidden_characters: &[],
            character_frames: &[],
            tint: $tint,
        }
    };
}

/// By animation's blood (hidden or shown) and flash.
static STANDING_WITHOUT_BLOOD: [RenderOverrides; 2] = [
    overrides!(&[("bb", ChildState::Hidden)], None),
    overrides!(&[("bb", ChildState::Hidden)], Some(HURT_TINT)),
];
static APPEARING_WITHOUT_BLOOD: [RenderOverrides; 2] = [
    overrides!(&[("d.bb", ChildState::Hidden)], None),
    overrides!(&[("d.bb", ChildState::Hidden)], Some(HURT_TINT)),
];
static FLASH: [RenderOverrides; 2] = [overrides!(&[], None), overrides!(&[], Some(HURT_TINT))];

pub(super) fn source_for(texture_id: TextureId) -> TextureSource {
    let TextureId::Headless {
        animation,
        frame,
        flashing,
    } = texture_id
    else {
        return framed(HEADLESS_CLIP, STANDING_FRAME);
    };
    let flash = usize::from(flashing);
    let (clip_frame, overrides) = match animation {
        HeadlessAnimation::Standing { spurting: true } => (STANDING_FRAME, &FLASH[flash]),
        HeadlessAnimation::Standing { spurting: false } => (STANDING_FRAME, &STANDING_WITHOUT_BLOOD[flash]),
        HeadlessAnimation::Walking => (WALKING_FRAME, &FLASH[flash]),
        HeadlessAnimation::Dying => (DYING_FRAME, &FLASH[0]),
        HeadlessAnimation::Appearing { spurting: true } => (APPEARING_FRAME, &FLASH[flash]),
        HeadlessAnimation::Appearing { spurting: false } => (APPEARING_FRAME, &APPEARING_WITHOUT_BLOOD[flash]),
    };
    TextureSource {
        selection: FrameSelection {
            frame_number: clip_frame,
            frames_since_frame_started: u16::from(frame),
        },
        overrides,
        ..framed(HEADLESS_CLIP, clip_frame)
    }
}

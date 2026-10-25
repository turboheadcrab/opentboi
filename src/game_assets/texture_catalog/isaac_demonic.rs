//! Isaac in the demonic look the Empress gives him (`curs()`): drawn from his whole clip `b1`
//! (5825), whose origin is his floor point, with the head's mode on 30 (his head clip 3069 holds
//! the demonic head 2912) and the body outfit `bo` (5591) on frame 8 (the body 5389: standing,
//! then the vertical and horizontal walk cycles), or 9 flying.

use swf::CharacterId;

use super::TextureSource;
use crate::game::IsaacPart;
use crate::game_assets::vector_art::{ChildState, FrameSelection, RenderOverrides};

const PLAYER_CLIP: CharacterId = 5825;
const FIRST_FRAME: u16 = 1;
/// The head sits 24 px above his floor point, the body 8 (y down in the clip's own units).
const HEAD_CENTER: (f32, f32) = (0.0, -24.0);
const BODY_CENTER: (f32, f32) = (0.0, -8.0);
const DEMONIC_HEAD_MODE: u16 = 30;
const DEMONIC_BODY_OUTFIT: u16 = 8;
const DEMONIC_FLYING_BODY_OUTFIT: u16 = 9;
const BODY_STANDING: u16 = 1;
const BODY_WALKING_VERTICALLY: u16 = 2;
const BODY_WALKING_HORIZONTALLY: u16 = 3;
/// What `b1` also holds on its first frame: the bubbles, the body, the head and the hand shadow.
pub(super) const BUBBLE_BEHIND: CharacterId = 5320;
const BODY_OUTFITS: CharacterId = 5591;
const HEAD: CharacterId = 5592;
const HAND_SHADOW: CharacterId = 5596;
pub(super) const BUBBLE_IN_FRONT: CharacterId = 5608;

/// Everything but the head hidden; the head clip's direction frames are set by `head`.
macro_rules! head_overrides {
    ($($frame:literal)*) => {
        [$(RenderOverrides {
            children: &[("d.d", ChildState::Frame(DEMONIC_HEAD_MODE)), ("d.d.d", ChildState::Frame($frame))],
            texts: &[],
            hidden_characters: &[BUBBLE_BEHIND, BODY_OUTFITS, HAND_SHADOW, BUBBLE_IN_FRONT],
            character_frames: &[],
            tint: None,
        }),*]
    };
}

/// By the head's frame (direction, then shooting).
static HEADS: [RenderOverrides; 8] = head_overrides!(1 2 3 4 5 6 7 8);

/// Only the body: `bo` on `$outfit`, its body clip on `$pose`.
macro_rules! body_overrides {
    ($outfit:expr, $pose:expr) => {
        RenderOverrides {
            children: &[("bo", ChildState::Frame($outfit)), ("bo.d", ChildState::Frame($pose))],
            texts: &[],
            hidden_characters: &[BUBBLE_BEHIND, HEAD, HAND_SHADOW, BUBBLE_IN_FRONT],
            character_frames: &[],
            tint: None,
        }
    };
}

static STANDING: RenderOverrides = body_overrides!(DEMONIC_BODY_OUTFIT, BODY_STANDING);
static FLYING: RenderOverrides = body_overrides!(DEMONIC_FLYING_BODY_OUTFIT, BODY_STANDING);
static WALKING_VERTICALLY: RenderOverrides = body_overrides!(DEMONIC_BODY_OUTFIT, BODY_WALKING_VERTICALLY);
static WALKING_HORIZONTALLY: RenderOverrides = body_overrides!(DEMONIC_BODY_OUTFIT, BODY_WALKING_HORIZONTALLY);

fn centered(overrides: &'static RenderOverrides, center: (f32, f32), cycle_frame: u8) -> TextureSource {
    TextureSource {
        character_id: PLAYER_CLIP,
        selection: FrameSelection {
            frame_number: FIRST_FRAME,
            frames_since_frame_started: u16::from(cycle_frame),
        },
        origin_x: center.0,
        origin_y: center.1,
        overrides,
    }
}

/// `part` must be one of the demonic ones.
pub(super) fn source(part: IsaacPart) -> TextureSource {
    match part {
        IsaacPart::DemonicHead(frame) => centered(&HEADS[usize::from(frame.min(7))], HEAD_CENTER, 0),
        IsaacPart::DemonicFlyingBody => centered(&FLYING, BODY_CENTER, 0),
        IsaacPart::DemonicBodyWalkVertical(frame) => centered(&WALKING_VERTICALLY, BODY_CENTER, frame),
        IsaacPart::DemonicBodyWalkHorizontal(frame) => centered(&WALKING_HORIZONTALLY, BODY_CENTER, frame),
        _ => centered(&STANDING, BODY_CENTER, 0),
    }
}

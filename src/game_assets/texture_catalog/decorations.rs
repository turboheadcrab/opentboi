//! Figures the layouts place (`inl.shop`): the devil statue, the shopkeepers.

use swf::CharacterId;

use super::{TextureSource, framed};
use crate::game::{Decoration, TextureId};
use crate::game_assets::vector_art::FrameSelection;

const DEVIL_STATUE_CLIP: CharacterId = 3729;
const SHOPKEEPER_CLIP: CharacterId = 6711;
const SECRET_SHOPKEEPER_CLIP: CharacterId = 6718;

/// A figure a layout places, in look `look` (its clip's frame from 1), `frame` frames into the
/// loops on it.
pub(super) fn source_for(texture_id: TextureId) -> TextureSource {
    let TextureId::Decoration {
        decoration,
        look,
        frame,
    } = texture_id
    else {
        return framed(DEVIL_STATUE_CLIP, 1);
    };
    let clip = match decoration {
        Decoration::DevilStatue => DEVIL_STATUE_CLIP,
        Decoration::Shopkeeper => SHOPKEEPER_CLIP,
        Decoration::SecretShopkeeper => SECRET_SHOPKEEPER_CLIP,
    };
    let clip_frame = 1 + u16::from(look);
    TextureSource {
        selection: FrameSelection {
            frame_number: clip_frame,
            frames_since_frame_started: u16::from(frame),
        },
        ..framed(clip, clip_frame)
    }
}

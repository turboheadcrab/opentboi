//! The player's tears: the body, its shadow and the splashes.

use super::{TextureSource, framed};
use crate::game::TextureId;
use swf::CharacterId;

const TEAR_BODY_SHAPE: CharacterId = 1789;
/// The tear of `b500`, the devil's: frame 1 of its clip 536 is this body and a shadow.
const DARK_TEAR_BODY_SHAPE: CharacterId = 495;
const TEAR_SHADOW_SHAPE: CharacterId = 341;
/// Player tear clip `b2`: frame 1 flying, 2–17 hit splash, 40–55 ground splash (`groundhit`).
const TEAR_CLIP: CharacterId = 1838;
const TEAR_HIT_SPLASH_FIRST_FRAME: u16 = 2;
const TEAR_GROUND_SPLASH_FIRST_FRAME: u16 = 40;

/// The tear body and its shadow `sh` (sprite 342) are separate because the shadow stays on the
/// floor while the body is lifted.
pub(super) fn source_for(texture_id: TextureId) -> TextureSource {
    match texture_id {
        TextureId::Tear => framed(TEAR_BODY_SHAPE, 1),
        TextureId::DarkTear => framed(DARK_TEAR_BODY_SHAPE, 1),
        TextureId::TearShadow => framed(TEAR_SHADOW_SHAPE, 1),
        TextureId::TearHitSplash(frame) => framed(TEAR_CLIP, TEAR_HIT_SPLASH_FIRST_FRAME + u16::from(frame)),
        TextureId::TearGroundSplash(frame) => framed(TEAR_CLIP, TEAR_GROUND_SPLASH_FIRST_FRAME + u16::from(frame)),
        _ => unreachable!("not a tear texture: {texture_id:?}"),
    }
}

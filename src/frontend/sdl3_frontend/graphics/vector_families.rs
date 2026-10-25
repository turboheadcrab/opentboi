//! Which textures the SDL3 frontend draws as triangles instead of rendered pixels.

use crate::game::{InputDevice, TextureId};
use crate::game_assets::TextureRenderer;

/// Textures drawn as vectors when they can be (a frame with a gradient or a mask keeps its
/// texture): the families that come in many animation frames of the same shapes (a texture per
/// frame costs far more video memory than the shapes' triangles), the full-screen overlays, the
/// doors, a pad's button pictures (drawn from SVG outlines, never rendered), and the banners with text (card names would otherwise render mid-game, one texture per
/// frame). Static pictures that are drawn all the time (room walls and floor, the HUD, the stage
/// surround) stay textures: one texture is cheaper to draw than hundreds of thousands of triangles.
pub fn draws_as_vectors(texture_id: TextureId) -> bool {
    matches!(
        texture_id,
        TextureId::CardAnimation { .. }
            | TextureId::EternalHeartUnion(_)
            | TextureId::Door(_)
            | TextureId::NameBanner { .. }
            | TextureId::PickupMessage { .. }
            | TextureId::Decoration { .. }
            | TextureId::IsaacAnimation { .. }
            | TextureId::Fly { .. }
            | TextureId::Gaper { .. }
            | TextureId::Headless { .. }
            | TextureId::Horf { .. }
            | TextureId::BlueFly(_)
            | TextureId::Clotty { .. }
            | TextureId::Mulligan { .. }
            | TextureId::Pickup { .. }
            | TextureId::Explosion(_)
            | TextureId::BreakableStage { .. }
            | TextureId::BreakableDestroyed { .. }
            | TextureId::BreakableRegrow { .. }
            | TextureId::KeyBlockOpening(_)
            | TextureId::BombFuse(_)
            | TextureId::TearHitSplash(_)
            | TextureId::TearGroundSplash(_)
            | TextureId::BloodSplash(_)
            | TextureId::HeldCardName(_)
            | TextureId::CardSlotAnimation { .. }
            | TextureId::DeathNote(_)
            | TextureId::DeathSpotlight(_)
            | TextureId::Machine { .. }
            | TextureId::StartRoomHint(_, InputDevice::Gamepad(_))
            | TextureId::ActiveItemLabel(InputDevice::Gamepad(_))
            | TextureId::HeldCardLabel(InputDevice::Gamepad(_))
    )
}

/// Whether the vector painter will draw `texture_id` (it is in a vector family and every shape of
/// its frame can be meshed), so no texture needs rendering for it.
pub fn is_drawn_as_vectors(renderer: &TextureRenderer, texture_id: TextureId) -> bool {
    draws_as_vectors(texture_id) && renderer.vector_picture(texture_id).is_some()
}

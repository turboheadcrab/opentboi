//! The textures the original leaves empty on purpose: the tail of a burst, a pickup before it lands.

use crate::game::{
    ClottyAnimation, EnemyTexture, FleshMoundAnimation, FlyAnimation, FlyKind, GaperAnimation, HeadlessAnimation,
    HopperAnimation, HorfAnimation, MulliganAnimation, PickupPhase, TextureId,
};

/// Clip frames the original leaves empty (kept to play the clips at their full length).
const EXPLOSION_FIRST_EMPTY_FRAME: u8 = 26;
/// `bloodsplash` (1488) shows nothing on its last (17th) frame.
const BLOOD_SPLASH_EMPTY_FRAME: u8 = 16;
const KEY_BLOCK_OPENING_FIRST_EMPTY_FRAME: u8 = 23;
/// The used card's name strip (`pilll` frame 3) holds only the button's badge on its last frames,
/// which is drawn on its own now.
const CARD_SLOT_USED_FIRST_EMPTY_FRAME: u8 = 6;
const GROUND_ENEMY_FIRST_EMPTY_DYING_FRAME: u8 = 16;
const MULLIGAN_FIRST_EMPTY_BURST_FRAME: u8 = 49;
/// Pickup landing clips (1946, ...) place nothing before their sixth frame; the 16-frame pickup
/// flashes of hearts and bombs end on an empty frame. Chests are drawn throughout.
const PICKUP_LANDING_FIRST_DRAWN_FRAME: u8 = 5;
const PICKUP_FLASH_EMPTY_FRAME: u8 = 15;

#[allow(clippy::too_many_lines, reason = "one arm for each texture that is empty on purpose")]
pub(super) fn is_empty_in_the_original(texture_id: TextureId) -> bool {
    match texture_id {
        TextureId::Explosion(frame) => frame >= EXPLOSION_FIRST_EMPTY_FRAME,
        TextureId::KeyBlockOpening(frame) => frame >= KEY_BLOCK_OPENING_FIRST_EMPTY_FRAME,
        TextureId::Pickup {
            kind,
            phase: PickupPhase::Landing,
            frame,
        } => !kind.is_chest() && frame < PICKUP_LANDING_FIRST_DRAWN_FRAME,
        TextureId::Pickup {
            kind,
            phase: PickupPhase::Collected,
            frame,
        } => !kind.is_chest() && frame == PICKUP_FLASH_EMPTY_FRAME,
        TextureId::PillPickup(_, PickupPhase::Landing, frame) => frame < PICKUP_LANDING_FIRST_DRAWN_FRAME,
        TextureId::PillPickup(_, PickupPhase::Collected, frame) => frame == PICKUP_FLASH_EMPTY_FRAME,
        // An enemy shot's splash (1488) ends on an empty frame.
        TextureId::BloodSplash(frame) => frame == BLOOD_SPLASH_EMPTY_FRAME,
        TextureId::CardSlotAnimation { used: true, frame } => frame >= CARD_SLOT_USED_FIRST_EMPTY_FRAME,
        // The darkness over dying Isaac starts fully transparent.
        TextureId::DeathSpotlight(step) => step == 0,
        // The burst (566) of a Gaper, a headless body or Horf has played out by its sixteenth frame;
        // the rest of the clip is empty.
        TextureId::Gaper {
            animation: GaperAnimation::Dying,
            frame,
            ..
        }
        | TextureId::Headless {
            animation: HeadlessAnimation::Dying,
            frame,
            ..
        }
        | TextureId::Horf {
            animation: HorfAnimation::Dying,
            frame,
            ..
        }
        | TextureId::Clotty {
            animation: ClottyAnimation::Dying,
            frame,
            ..
        }
        | TextureId::Mulligan {
            animation: MulliganAnimation::Dying,
            frame,
            ..
        }
        | TextureId::Enemy(
            EnemyTexture::Hopper {
                animation: HopperAnimation::Dying,
                frame,
                ..
            }
            | EnemyTexture::FleshMound {
                animation: FleshMoundAnimation::Dying,
                frame,
                ..
            },
        )
        | TextureId::Fly {
            kind: FlyKind::Moter,
            animation: FlyAnimation::Dying,
            frame,
            ..
        } => frame >= GROUND_ENEMY_FIRST_EMPTY_DYING_FRAME,
        // The Mulligan's burst (4930) is over by its 49th frame; Mulligans die long before the end.
        TextureId::Mulligan {
            animation: MulliganAnimation::Bursting,
            frame,
            ..
        } => frame >= MULLIGAN_FIRST_EMPTY_BURST_FRAME,
        // The flies' death (555) ends on an empty frame, then the fly is removed.
        TextureId::Fly {
            kind,
            animation: FlyAnimation::Dying,
            frame,
            ..
        } => frame + 1 == kind.frame_count(FlyAnimation::Dying),
        _ => false,
    }
}

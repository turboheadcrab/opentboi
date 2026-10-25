//! The animations of the pickup clip and the canvases they need.

use glam::Vec2;

use crate::game::original_units::Pixels;
use crate::game::pickups::kind::PickupKind;

/// Which animation of the pickup clip (`d` frames 3, 1 and 2) a texture shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PickupPhase {
    Landing,
    Idle,
    /// Taken (or, for a chest, opened).
    Collected,
}

/// Landing pickups bounce high above their spot, so that texture is centered this far up.
pub const LANDING_ART_LIFT: Pixels = Pixels(36.0);
/// Canvases per kind and phase (landing, idle, taken): bombs are big and flash widely when taken,
/// keys are tall, hearts and coins small, chests wide.
const HEART_AND_COIN_SIZES: [Vec2; 3] = [Vec2::new(48.0, 160.0), Vec2::new(40.0, 40.0), Vec2::new(96.0, 44.0)];
const KEY_SIZES: [Vec2; 3] = [Vec2::new(48.0, 160.0), Vec2::new(28.0, 56.0), Vec2::new(120.0, 68.0)];
const BOMB_SIZES: [Vec2; 3] = [Vec2::new(92.0, 160.0), Vec2::new(52.0, 66.0), Vec2::new(160.0, 80.0)];
const ITEM_PEDESTAL_SIZES: [Vec2; 3] = [Vec2::ZERO, Vec2::new(64.0, 96.0), Vec2::ZERO];
/// The item floats above its pedestal.
const HELD_ITEM_PEDESTAL_SIZES: [Vec2; 3] = [Vec2::ZERO, Vec2::new(72.0, 168.0), Vec2::ZERO];
const CARD_SIZES: [Vec2; 3] = [Vec2::new(68.0, 160.0), Vec2::new(40.0, 56.0), Vec2::new(164.0, 68.0)];
const CHEST_SIZES: [Vec2; 3] = [Vec2::new(152.0, 160.0), Vec2::new(64.0, 64.0), Vec2::new(96.0, 96.0)];

impl PickupPhase {
    pub const ALL: [Self; 3] = [Self::Landing, Self::Idle, Self::Collected];

    pub fn texture_size(self, kind: PickupKind) -> Vec2 {
        let sizes = match kind {
            PickupKind::Heart(_) | PickupKind::Coin(_) => HEART_AND_COIN_SIZES,
            PickupKind::Key(_) => KEY_SIZES,
            PickupKind::Bomb(_) => BOMB_SIZES,
            PickupKind::Chest(_) => CHEST_SIZES,
            PickupKind::ItemPedestal => ITEM_PEDESTAL_SIZES,
            PickupKind::Item(_) => HELD_ITEM_PEDESTAL_SIZES,
            PickupKind::Card(_) | PickupKind::Pill(_) => CARD_SIZES,
        };
        sizes[self as usize]
    }

    /// Where the texture's center lies above the pickup's position.
    pub fn art_lift(self) -> Pixels {
        if self == Self::Landing {
            LANDING_ART_LIFT
        } else {
            Pixels::ZERO
        }
    }

    pub fn frame_count(self, kind: PickupKind) -> u8 {
        match self {
            Self::Landing => kind.landing_frame_count(),
            Self::Idle => kind.idle_frame_count(),
            // Troll bombs are never taken.
            Self::Collected if kind.is_troll_bomb() || kind.is_pedestal() => 0,
            Self::Collected => kind.collected_frame_count(),
        }
    }
}

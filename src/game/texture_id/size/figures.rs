//! The canvases of Isaac's whole-body animations and of the figures a layout places.

use glam::Vec2;

use crate::game::PlayerAnimation;
use crate::game::room::layout::Decoration;

const ISAAC_TELEPORT_SIZE: Vec2 = Vec2::new(88.0, 420.0);
const ISAAC_HOLDING_UP_SIZE: Vec2 = Vec2::new(84.0, 180.0);
/// Dying throws blood and ends in a pool wider than he is.
const ISAAC_DYING_SIZE: Vec2 = Vec2::new(152.0, 216.0);
const DEVIL_STATUE_SIZE: Vec2 = Vec2::new(100.0, 112.0);
/// Hung from the ceiling by a string.
const SHOPKEEPER_SIZE: Vec2 = Vec2::new(48.0, 320.0);
const SECRET_SHOPKEEPER_SIZE: Vec2 = Vec2::new(52.0, 140.0);

pub(super) fn isaac_animation_size(animation: PlayerAnimation) -> Vec2 {
    match animation {
        PlayerAnimation::Teleport | PlayerAnimation::TeleportArrival => ISAAC_TELEPORT_SIZE,
        PlayerAnimation::Dying => ISAAC_DYING_SIZE,
        _ => ISAAC_HOLDING_UP_SIZE,
    }
}

pub(super) fn decoration_size(decoration: Decoration) -> Vec2 {
    match decoration {
        Decoration::DevilStatue => DEVIL_STATUE_SIZE,
        Decoration::Shopkeeper => SHOPKEEPER_SIZE,
        Decoration::SecretShopkeeper => SECRET_SHOPKEEPER_SIZE,
    }
}

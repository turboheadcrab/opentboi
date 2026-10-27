//! How large each texture is in world units: the canvas its art is rendered on (checked with
//! `cargo test texture_extents -- --ignored --nocapture`).

use glam::Vec2;

use super::{
    ClottyAnimation, DoorStyle, DoorTexture, FlyAnimation, FlyKind, GaperAnimation, HeadlessAnimation, HorfAnimation,
    MulliganAnimation, TextureId,
};
use crate::game::room::layout::Decoration;

const TEAR_SIZE: Vec2 = Vec2::new(18.0, 18.0);
const TEAR_SHADOW_SIZE: Vec2 = Vec2::new(18.0, 8.0);
const GRID_TILE_SIZE: Vec2 = Vec2::new(60.0, 60.0);
const TEAR_SPLASH_SIZE: Vec2 = Vec2::new(128.0, 128.0);
/// The opening key block bursts into a dust cloud much wider than its tile.
const KEY_BLOCK_OPENING_SIZE: Vec2 = Vec2::new(140.0, 84.0);
/// Red poop's grow-back animation splashes out past the tile before settling.
const BREAKABLE_REGROW_SIZE: Vec2 = Vec2::new(144.0, 128.0);
/// The fly clip hangs its body above and its shadow below the orbit point.
const ORBITING_FLY_SIZE: Vec2 = Vec2::new(24.0, 76.0);
const EXPLOSION_SIZE: Vec2 = Vec2::new(320.0, 224.0);
const EXPLOSION_SCORCH_SIZE: Vec2 = Vec2::new(128.0, 72.0);
/// The shop door's key swings out past the frame while it unlocks.
const DOOR_SIZE: Vec2 = Vec2::new(100.0, 96.0);
/// The boss door's horns reach past the frame.
const BOSS_DOOR_SIZE: Vec2 = Vec2::new(128.0, 132.0);
const DEVIL_DOOR_SIZE: Vec2 = Vec2::new(128.0, 108.0);
/// The fuse spark rises well above the bomb.
const BOMB_SIZE: Vec2 = Vec2::new(52.0, 72.0);
/// Covers windows from about 1:2.3 to 5:1; the black view masks lie beyond.
const STAGE_SURROUND_SIZE: Vec2 = Vec2::new(2400.0, 1440.0);
const HUD_COUNTER_SIZE: Vec2 = Vec2::new(40.0, 32.0);
const HUD_KEY_ICON_SIZE: Vec2 = Vec2::new(48.0, 48.0);
/// The heart clip's origin is off its art's center.
const HUD_HEART_SIZE: Vec2 = Vec2::new(50.0, 40.0);
/// Minimap pieces sit 25 apart horizontally and 10 vertically.
const MAP_PICKUP_MARK_SIZE: Vec2 = Vec2::new(28.0, 16.0);
const MAP_PIECE_SIZE: Vec2 = Vec2::new(40.0, 24.0);
const ISAAC_TELEPORT_SIZE: Vec2 = Vec2::new(88.0, 420.0);
const ISAAC_HOLDING_UP_SIZE: Vec2 = Vec2::new(84.0, 180.0);
/// Dying throws blood and ends in a pool wider than he is.
const ISAAC_DYING_SIZE: Vec2 = Vec2::new(152.0, 216.0);
const DEVIL_STATUE_SIZE: Vec2 = Vec2::new(100.0, 112.0);
const FLY_SIZE: Vec2 = Vec2::new(120.0, 120.0);
const ENEMY_SHOT_SIZE: Vec2 = Vec2::new(24.0, 24.0);
const BLOOD_SPLASH_SIZE: Vec2 = Vec2::new(128.0, 128.0);
/// The Pooter rises out of the floor while appearing.
const POOTER_SIZE: Vec2 = Vec2::new(120.0, 128.0);
/// The Gaper rises out of the floor while appearing, and its death clip bursts far past its body.
/// Its head stretches up while it shouts.
const GAPER_SIZE: Vec2 = Vec2::new(120.0, 128.0);
const GAPER_APPEARING_SIZE: Vec2 = Vec2::new(72.0, 164.0);
const GAPER_DYING_SIZE: Vec2 = Vec2::new(236.0, 276.0);
const HEADLESS_SIZE: Vec2 = Vec2::new(120.0, 128.0);
/// Horf rises out of the floor while appearing, and stretches while it shoots.
const HORF_SIZE: Vec2 = Vec2::new(120.0, 128.0);
const HORF_APPEARING_SIZE: Vec2 = Vec2::new(84.0, 176.0);
const HORF_SHOOTING_SIZE: Vec2 = Vec2::new(124.0, 156.0);
/// The Clotty and the Mulligan rise out of the floor while appearing; the Mulligan bursts wide.
const CLOTTY_SIZE: Vec2 = Vec2::new(120.0, 128.0);
const CLOTTY_APPEARING_SIZE: Vec2 = Vec2::new(84.0, 196.0);
/// Its shot throws blood far above it.
const CLOTTY_SHOOTING_SIZE: Vec2 = Vec2::new(160.0, 256.0);
const MULLIGAN_SIZE: Vec2 = Vec2::new(120.0, 128.0);
const MULLIGAN_APPEARING_SIZE: Vec2 = Vec2::new(84.0, 176.0);
/// Its death bursts a little wider than the Gapers'.
const MULLIGAN_DYING_SIZE: Vec2 = Vec2::new(240.0, 276.0);
const MULLIGAN_BURSTING_SIZE: Vec2 = Vec2::new(160.0, 208.0);
/// A familiar's clip is drawn around its floor point, the fly itself well above it.
const BLUE_FLY_SIZE: Vec2 = Vec2::new(64.0, 128.0);
const BLOOD_CREEP_SIZE: Vec2 = Vec2::new(120.0, 80.0);
/// The widest of the red looks needs 47×25.
const BLOOD_SPLAT_SIZE: Vec2 = Vec2::new(48.0, 28.0);
/// A plain door slamming shuts with dust and a shake past its frame.
const DOOR_MOTION_SIZE: Vec2 = Vec2::new(152.0, 128.0);
/// The boss door swinging shut flaps out sideways for a few frames.
const BOSS_DOOR_MOTION_SIZE: Vec2 = Vec2::new(268.0, 156.0);
/// Hung from the ceiling by a string.
const SHOPKEEPER_SIZE: Vec2 = Vec2::new(48.0, 320.0);
const SECRET_SHOPKEEPER_SIZE: Vec2 = Vec2::new(52.0, 140.0);
const HELD_CARD_SIZE: Vec2 = Vec2::new(32.0, 52.0);
const HELD_CARD_NAME_SIZE: Vec2 = Vec2::new(316.0, 96.0);
/// The card name banner (`hud.st1` at x 27) slides in from the stage's left edge: its texture
/// covers the stage's width from there, centered this far right of the clip's origin.
const CARD_SLOT_ANIMATION_SIZE: Vec2 = Vec2::new(140.0, 156.0);
const CARD_NAME_SIZE: Vec2 = Vec2::new(640.0, 92.0);
const SHOP_WARE_SIZE: Vec2 = Vec2::new(52.0, 80.0);
pub const CARD_NAME_TEXTURE_CENTER_X: f32 = 320.0 - 27.0;

impl TextureId {
    fn enemy_size(self) -> Vec2 {
        match self {
            Self::EnemyShot(_) => ENEMY_SHOT_SIZE,
            Self::BloodSplash(_) => BLOOD_SPLASH_SIZE,
            Self::BlueFly(_) => BLUE_FLY_SIZE,
            Self::BloodCreep(_) => BLOOD_CREEP_SIZE,
            // The Moter dies in the Gapers' burst.
            Self::Fly {
                kind: FlyKind::Moter,
                animation: FlyAnimation::Dying,
                ..
            } => GAPER_DYING_SIZE,
            Self::Fly { kind, .. } if kind.is_pooter() => POOTER_SIZE,
            _ => FLY_SIZE,
        }
    }

    /// Area in world units the texture covers at scale 1.
    fn door_size(self) -> Vec2 {
        match self {
            Self::Door(DoorTexture::Open(DoorStyle::Boss)) => BOSS_DOOR_SIZE,
            Self::Door(DoorTexture::Closing(DoorStyle::Boss, _) | DoorTexture::Opening(DoorStyle::Boss, _)) => {
                BOSS_DOOR_MOTION_SIZE
            }
            Self::Door(DoorTexture::Closing(..) | DoorTexture::Opening(..) | DoorTexture::Blown(_)) => DOOR_MOTION_SIZE,
            Self::Door(DoorTexture::Open(DoorStyle::Devil)) => DEVIL_DOOR_SIZE,
            _ => DOOR_SIZE,
        }
    }

    /// The ground enemies, whose clips differ in how far they reach.
    fn walker_size(self) -> Vec2 {
        match self {
            Self::Gaper { animation, .. } => match animation {
                GaperAnimation::Appearing { .. } => GAPER_APPEARING_SIZE,
                GaperAnimation::Dying => GAPER_DYING_SIZE,
                GaperAnimation::Standing | GaperAnimation::Walking | GaperAnimation::Head => GAPER_SIZE,
            },
            Self::Headless { animation, .. } => match animation {
                HeadlessAnimation::Dying => GAPER_DYING_SIZE,
                HeadlessAnimation::Appearing { .. } => GAPER_APPEARING_SIZE,
                _ => HEADLESS_SIZE,
            },
            Self::Horf { animation, .. } => match animation {
                HorfAnimation::Dying => GAPER_DYING_SIZE,
                HorfAnimation::Appearing => HORF_APPEARING_SIZE,
                HorfAnimation::Shooting => HORF_SHOOTING_SIZE,
                HorfAnimation::Idle => HORF_SIZE,
            },
            Self::Clotty { animation, .. } => match animation {
                ClottyAnimation::Dying => GAPER_DYING_SIZE,
                ClottyAnimation::Appearing => CLOTTY_APPEARING_SIZE,
                ClottyAnimation::Shooting => CLOTTY_SHOOTING_SIZE,
                _ => CLOTTY_SIZE,
            },
            Self::Mulligan { animation, .. } => match animation {
                MulliganAnimation::Dying => MULLIGAN_DYING_SIZE,
                MulliganAnimation::Appearing => MULLIGAN_APPEARING_SIZE,
                MulliganAnimation::Bursting => MULLIGAN_BURSTING_SIZE,
                _ => MULLIGAN_SIZE,
            },
            _ => Vec2::ZERO,
        }
    }

    /// The HUD's card, pill and trinket slots.
    fn held_size(self) -> Vec2 {
        match self {
            Self::HeldCard(_) | Self::HeldPill(_) => HELD_CARD_SIZE,
            Self::HeldCardName(_) | Self::HeldPillName(_) => HELD_CARD_NAME_SIZE,
            Self::TrinketSlot(..) => crate::game::hud::held_trinket::SLOT_SIZE,
            _ => CARD_SLOT_ANIMATION_SIZE,
        }
    }

    /// A HUD label naming a device's button.
    fn label_size(self) -> Vec2 {
        match self {
            Self::HeldCardLabel(device) => crate::game::hud::held_card::label_texture_size(device),
            Self::ActiveItemLabel(device) => crate::game::hud::active_item_label::texture_size(device),
            _ => Vec2::ZERO,
        }
    }

    #[allow(clippy::too_many_lines, reason = "one arm for each family of textures")]
    pub fn world_size(self) -> Vec2 {
        match self {
            Self::RoomBackground(_) | Self::StartRoomControls => crate::game::room::ROOM_SIZE,
            Self::StartRoomHint(control, _) => control.size(),
            Self::ActiveItemBox { .. } => crate::game::hud::item_box::TEXTURE_SIZE,
            Self::ActiveItemLabel(_) | Self::HeldCardLabel(_) => self.label_size(),
            Self::Hud | Self::DeathNote(_) | Self::EternalHeartUnion(_) | Self::CardAnimation { .. } => {
                crate::game::room::STAGE_SIZE
            }
            Self::DeathSpotlight(_) => crate::game::player::death::spotlight::BEAM_SIZE,
            Self::MapPiece(_) => MAP_PIECE_SIZE,
            Self::MapPickupMark(_) => MAP_PICKUP_MARK_SIZE,
            Self::HudCounter(_) | Self::HudGlyph(_) => HUD_COUNTER_SIZE,
            Self::HudHeart(_) => HUD_HEART_SIZE,
            Self::HudKeyIcon { .. } => HUD_KEY_ICON_SIZE,
            Self::NameBanner { .. } => CARD_NAME_SIZE,
            Self::HeldCard(_)
            | Self::HeldPill(_)
            | Self::HeldCardName(_)
            | Self::HeldPillName(_)
            | Self::CardSlotAnimation { .. }
            | Self::TrinketSlot(..) => self.held_size(),
            Self::Pickup { kind, phase, .. } => phase.texture_size(kind),
            Self::ShieldBubble(_) => crate::game::player::shield::BUBBLE_SIZE,
            Self::StageSurround => STAGE_SURROUND_SIZE,
            Self::Tear | Self::DarkTear => TEAR_SIZE,
            Self::TearShadow => TEAR_SHADOW_SIZE,
            Self::Tile(_) | Self::KeyBlock | Self::ClearedTile(_) | Self::BrokenFloor => GRID_TILE_SIZE,
            Self::BreakableStage { kind, .. } => kind.stage_size(),
            Self::BreakableDestroyed { kind, .. } => kind.destroyed_size(),
            Self::BreakableRegrow { .. } => BREAKABLE_REGROW_SIZE,
            Self::KeyBlockOpening(_) => KEY_BLOCK_OPENING_SIZE,
            Self::OrbitingFly(_) => ORBITING_FLY_SIZE,
            Self::Explosion(_) => EXPLOSION_SIZE,
            Self::ExplosionScorch => EXPLOSION_SCORCH_SIZE,
            Self::BombFuse(_) => BOMB_SIZE,
            Self::Door(_) => self.door_size(),
            Self::TearHitSplash(_) | Self::TearGroundSplash(_) => TEAR_SPLASH_SIZE,
            Self::Isaac { part, .. } => part.size(),
            Self::PickupMessage { .. } => crate::game::hud::message::MESSAGE_TEXTURE_SIZE,
            Self::ShopWare(_) | Self::ShopConsumableWare { .. } => SHOP_WARE_SIZE,
            Self::Machine { frame, .. } => crate::game::machines::clip::texture_size(frame),
            Self::Gaper { .. }
            | Self::Headless { .. }
            | Self::Horf { .. }
            | Self::Clotty { .. }
            | Self::Mulligan { .. } => self.walker_size(),
            Self::Enemy(texture) => texture.world_size(),
            Self::Fly { .. } | Self::BlueFly(_) | Self::BloodCreep(_) | Self::EnemyShot(_) | Self::BloodSplash(_) => {
                self.enemy_size()
            }
            Self::BloodSplat(_) => BLOOD_SPLAT_SIZE,
            Self::Decoration { decoration, .. } => decoration_size(decoration),
            Self::IsaacAnimation { animation, .. } => isaac_animation_size(animation),
        }
    }
}

fn isaac_animation_size(animation: crate::game::PlayerAnimation) -> Vec2 {
    use crate::game::PlayerAnimation;
    match animation {
        PlayerAnimation::Teleport | PlayerAnimation::TeleportArrival => ISAAC_TELEPORT_SIZE,
        PlayerAnimation::Dying => ISAAC_DYING_SIZE,
        _ => ISAAC_HOLDING_UP_SIZE,
    }
}

fn decoration_size(decoration: Decoration) -> Vec2 {
    match decoration {
        Decoration::DevilStatue => DEVIL_STATUE_SIZE,
        Decoration::Shopkeeper => SHOPKEEPER_SIZE,
        Decoration::SecretShopkeeper => SECRET_SHOPKEEPER_SIZE,
    }
}

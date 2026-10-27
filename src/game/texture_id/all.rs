//! The list of every texture, for rendering them all up front.

use super::{
    BLOOD_SPLASH_FRAME_COUNT, BLOWN_DOOR_FRAME_COUNT, BOMB_FUSE_FRAME_COUNT, BreakableKind, DoorStyle, DoorTexture,
    ENEMY_SHOT_BRIGHTNESS_STEPS, ETERNAL_HEART_UNION_FRAME_COUNT, EXPLOSION_FRAME_COUNT, FlyAnimation, FlyKind,
    HudHeart, InputDevice, IsaacPart, KEY_BLOCK_OPENING_FRAME_COUNT, MOVING_DOOR_STYLES, MapPiece,
    ORBITING_FLY_FRAME_COUNT, PickupKind, PickupPhase, RoomArt, SHOP_DOOR_UNLOCKING_FRAME_COUNT,
    TEAR_GROUND_SPLASH_FRAME_COUNT, TEAR_HIT_SPLASH_FRAME_COUNT, TextureId, TileKind, card_animation_frame_count,
    door_closing_frame_count, door_opening_frame_count,
};
use crate::game::hud::held_card::{SLOT_TAKEN_FRAME_COUNT, SLOT_USED_FRAME_COUNT};
use crate::game::input_state::GamepadKind;
use crate::game::pickups::card::Card;
use crate::game::room::layout::Decoration;

/// The `col` of what the shop's pill ware can show: the six pills, then nine tarot cards.
const SHOP_CONSUMABLE_COLORS: std::ops::RangeInclusive<u8> = 1..=15;
/// The minimap's pickup marks, frames 1–10.
const MAP_PICKUP_MARK_COUNT: u8 = 10;
/// The overlay's frames the cards play: The Devil, Death, The Empress, The Sun.
const CARD_ANIMATIONS: [u8; 4] = [2, 11, 13, 15];
/// Textures that have no frames or variants to count.
const SINGLE_TEXTURES: [TextureId; 6] = [
    TextureId::Hud,
    TextureId::StageSurround,
    TextureId::StartRoomControls,
    TextureId::Tear,
    TextureId::DarkTear,
    TextureId::TearShadow,
];

impl TextureId {
    /// Every texture the game can draw.
    pub fn all() -> Vec<Self> {
        let mut all: Vec<Self> = RoomArt::ALL.into_iter().map(Self::RoomBackground).collect();
        all.extend(SINGLE_TEXTURES);
        for control in crate::game::floor::StartRoomControl::ALL {
            all.push(Self::StartRoomHint(control, InputDevice::KeyboardAndMouse));
            all.extend(GamepadKind::ALL.map(|kind| Self::StartRoomHint(control, InputDevice::Gamepad(kind))));
        }
        all.push(Self::ActiveItemLabel(InputDevice::KeyboardAndMouse));
        all.extend(GamepadKind::ALL.map(|kind| Self::ActiveItemLabel(InputDevice::Gamepad(kind))));
        all.push(Self::HeldCardLabel(InputDevice::KeyboardAndMouse));
        all.extend(GamepadKind::ALL.map(|kind| Self::HeldCardLabel(InputDevice::Gamepad(kind))));
        all.extend(Self::hud_textures());
        all.extend([1, 2, 12, 13, 5, 6, 16, 17, 9, 10, 11].map(Self::ShopWare));
        all.extend(Self::shop_consumable_wares());
        all.extend(Self::pickups());
        all.extend(TileKind::ALL.map(Self::Tile));
        all.push(Self::KeyBlock);
        all.extend(TileKind::CLEARABLE.map(Self::ClearedTile));
        all.push(Self::BrokenFloor);
        for kind in BreakableKind::ALL {
            for stage in 0..kind.stage_count() {
                all.extend(
                    (0..kind.idle_frame_count(stage)).map(|idle_frame| Self::BreakableStage {
                        kind,
                        stage,
                        idle_frame,
                    }),
                );
            }
            all.extend((0..kind.destroyed_frame_count()).map(|frame| Self::BreakableDestroyed { kind, frame }));
            all.extend((0..kind.regrow_frame_count()).map(|frame| Self::BreakableRegrow { kind, frame }));
        }
        all.extend((0..KEY_BLOCK_OPENING_FRAME_COUNT).map(Self::KeyBlockOpening));
        all.extend((0..ORBITING_FLY_FRAME_COUNT).map(Self::OrbitingFly));
        all.extend((0..EXPLOSION_FRAME_COUNT).map(Self::Explosion));
        all.push(Self::ExplosionScorch);
        all.extend(DoorStyle::ALL.map(|style| Self::Door(DoorTexture::Open(style))));
        all.extend(Self::door_motions());
        all.extend((0..BOMB_FUSE_FRAME_COUNT).map(Self::BombFuse));
        all.extend((0..TEAR_HIT_SPLASH_FRAME_COUNT).map(Self::TearHitSplash));
        all.extend((0..TEAR_GROUND_SPLASH_FRAME_COUNT).map(Self::TearGroundSplash));
        all.extend(IsaacPart::all().map(|part| Self::Isaac { part, tint: None }));
        all.extend(Self::decorations());
        all.extend(Self::flies());
        all.extend(Self::walkers());
        all.extend((0..super::BLUE_FLY_FRAME_COUNT).map(Self::BlueFly));
        all.extend((0..super::BLOOD_CREEP_LOOK_COUNT).map(Self::BloodCreep));
        all.extend((0..super::BLOOD_SPLAT_LOOK_COUNT).map(Self::BloodSplat));
        all.extend((0..ENEMY_SHOT_BRIGHTNESS_STEPS).map(Self::EnemyShot));
        all.extend(Self::death());
        all
    }

    /// Every pickup's frames, with one card of each look and every item on its pedestal.
    fn pickups() -> impl Iterator<Item = Self> {
        let card_arts = [Card::WheelOfFortune, Card::TwoOfSpades].map(PickupKind::Card);
        let item_pedestals = crate::game::pickups::item::Item::ALL.map(PickupKind::Item);
        PickupKind::ALL
            .into_iter()
            .chain(card_arts)
            .chain(crate::game::pickups::pill::Pill::ALL.map(PickupKind::Pill))
            .chain(crate::game::pickups::trinket::Trinket::ALL.map(PickupKind::Trinket))
            .chain(item_pedestals)
            .flat_map(|kind| {
                PickupPhase::ALL.into_iter().flat_map(move |phase| {
                    (0..phase.frame_count(kind)).map(move |frame| Self::Pickup { kind, phase, frame })
                })
            })
    }

    /// Doors unlocking, shutting, shut and opening.
    fn door_motions() -> impl Iterator<Item = Self> {
        (0..SHOP_DOOR_UNLOCKING_FRAME_COUNT)
            .map(|frame| Self::Door(DoorTexture::ShopUnlocking(frame)))
            .chain((0..BLOWN_DOOR_FRAME_COUNT).map(|frame| Self::Door(DoorTexture::Blown(frame))))
            .chain(MOVING_DOOR_STYLES.into_iter().flat_map(|style| {
                (0..door_closing_frame_count(style))
                    .map(move |frame| Self::Door(DoorTexture::Closing(style, frame)))
                    .chain(
                        (0..door_opening_frame_count(style))
                            .map(move |frame| Self::Door(DoorTexture::Opening(style, frame))),
                    )
            }))
    }

    /// The death note for every killer, and every step of the spotlight over dying Isaac.
    fn death() -> impl Iterator<Item = Self> {
        crate::game::player::death::Killer::ALL
            .map(Self::DeathNote)
            .into_iter()
            .chain((0..crate::game::player::death::spotlight::SPOTLIGHT_FADE_STEPS).map(Self::DeathSpotlight))
    }

    /// Every frame of the flies' animations, and of the splash their shots leave.
    fn flies() -> impl Iterator<Item = Self> {
        let splashes = (0..BLOOD_SPLASH_FRAME_COUNT).map(Self::BloodSplash);
        splashes.chain(FlyKind::ALL.into_iter().flat_map(|kind| {
            kind.animations().iter().copied().flat_map(move |animation| {
                let wings = if animation == FlyAnimation::Flying {
                    kind.wing_frame_count()
                } else {
                    1
                };
                // Only the living flash red from a hit.
                let flashes: &[bool] = if animation == FlyAnimation::Dying {
                    &[false]
                } else {
                    &[false, true]
                };
                (0..kind.frame_count(animation)).flat_map(move |frame| {
                    (0..wings).flat_map(move |wing| {
                        flashes.iter().map(move |&flashing| Self::Fly {
                            kind,
                            animation,
                            frame,
                            wing,
                            flashing,
                        })
                    })
                })
            })
        }))
    }

    /// The figures layouts place, every look.
    fn decorations() -> impl Iterator<Item = Self> {
        [
            Decoration::DevilStatue,
            Decoration::Shopkeeper,
            Decoration::SecretShopkeeper,
        ]
        .into_iter()
        .flat_map(|decoration| (0..decoration.look_count()).map(move |look| (decoration, look)))
        .flat_map(|(decoration, look)| {
            (0..super::DECORATION_FRAME_COUNT).map(move |frame| Self::Decoration {
                decoration,
                look,
                frame,
            })
        })
    }

    /// The pill ware's picture for each `col` it can have, with and without the sale price.
    fn shop_consumable_wares() -> impl Iterator<Item = Self> {
        SHOP_CONSUMABLE_COLORS
            .flat_map(|color| [false, true].map(|on_sale| Self::ShopConsumableWare { color, on_sale }))
    }

    /// The HUD's parts, values and messages.
    fn hud_textures() -> Vec<Self> {
        let mut hud: Vec<Self> = MapPiece::ALL.map(Self::MapPiece).into();
        hud.extend((0..=crate::game::hud::MAXIMUM_COUNTER_VALUE).map(Self::HudCounter));
        hud.extend(HudHeart::ALL.map(Self::HudHeart));
        hud.extend((0..ETERNAL_HEART_UNION_FRAME_COUNT).map(Self::EternalHeartUnion));
        hud.extend([false, true].map(|golden| Self::HudKeyIcon { golden }));
        let box_items = std::iter::once(None).chain(crate::game::pickups::item::Item::ACTIVE.map(Some));
        for item in box_items {
            let frames = 0..crate::game::hud::item_box::ITEM_BOX_FRAME_COUNT;
            hud.extend(frames.map(|frame| Self::ActiveItemBox { item, frame }));
        }
        hud.extend(crate::game::HUD_GLYPHS.bytes().map(Self::HudGlyph));
        hud.extend((1..=MAP_PICKUP_MARK_COUNT).map(Self::MapPickupMark));
        hud.extend(Card::ALL.map(Self::HeldCard));
        hud.extend(Card::ALL.map(Self::HeldCardName));
        hud.extend(crate::game::pickups::pill::Pill::ALL.map(Self::HeldPill));
        hud.extend(
            crate::game::hud::pill_text::PillText::ALL
                .into_iter()
                .filter(|text| text.name_position().is_some())
                .map(Self::HeldPillName),
        );
        for (used, length) in [(false, SLOT_TAKEN_FRAME_COUNT), (true, SLOT_USED_FRAME_COUNT)] {
            hud.extend((0..length).map(|frame| Self::CardSlotAnimation { used, frame }));
        }
        for clip_frame in CARD_ANIMATIONS {
            hud.extend(
                (0..card_animation_frame_count(clip_frame)).map(|frame| Self::CardAnimation { clip_frame, frame }),
            );
        }
        for message in crate::game::hud::message::PickupMessage::ALL {
            let frames = crate::game::hud::message::MESSAGE_VISIBLE_FRAMES;
            hud.extend(frames.map(|frame| Self::PickupMessage { message, frame }));
        }
        hud
    }
}

//! Art sources for effects that are not tied to a grid tile: flies and explosions.

use swf::CharacterId;

use super::{TextureSource, framed};
use crate::game::{DoorStyle, DoorTexture, TextureId};
use crate::game_assets::vector_art::{ChildState, FrameSelection, RenderOverrides};

/// Object type 96 (`b96`): frame 1 is the idle fly, flapping over two frames.
const FLY_CLIP: CharacterId = 557;
const FLY_FRAME: u16 = 1;
/// The bomb clip (`b4`); `bombfail` jumps to frame 3, whose 46-frame explosion plays once.
const BOMB_CLIP: CharacterId = 3407;
const BOMB_EXPLOSION_FRAME: u16 = 3;
/// Its frame 5: the gas cloud (3402).
const BOMB_GAS_FRAME: u16 = 5;
/// The bomb clip's frame 1: the burning fuse (2039) and the shadow.
const BOMB_FUSE_FRAME: u16 = 1;
/// The door clip (`d1`–`d4`): frame 4 is the plain open door, 32 the open treasure door.
const DOOR_CLIP: CharacterId = 6550;
const OPEN_DOOR_FRAME: u16 = 4;
const OPEN_TREASURE_DOOR_FRAME: u16 = 32;
const BLOWN_OPEN_DOOR_FRAME: u16 = 17;
/// A boss door once open: frame 13 (also where it starts in a room's first second), the end of 15.
const BOSS_DOOR_FRAME: u16 = 13;
/// The devil room's doors (`sat`) once open: frame 19 (18 plays as it appears after the boss,
/// 40 and 41 are the angel room's).
const DEVIL_DOOR_FRAME: u16 = 19;
/// Curse doors start shut on frame 36 and open on frame 37 (10 frames, ending open) when the room's
/// doors open.
const OPEN_CURSE_DOOR_FRAME: u16 = 37;
const OPEN_CURSE_DOOR_LAST_FRAME: u8 = 9;
/// The shop door's frames animate: 9 closes into the lock (11 frames, ending on the keyhole), 8
/// turns the key and swings open (28 frames); their last frames are the steady states.
const LOCKED_SHOP_DOOR_FRAME: u16 = 9;
const LOCKED_SHOP_DOOR_LAST_FRAME: u8 = 10;
const UNLOCKED_SHOP_DOOR_FRAME: u16 = 8;
const UNLOCKED_SHOP_DOOR_LAST_FRAME: u8 = 27;
/// Frame 4's clip (6472) slams the plain door shut; frame 3's `p` (6471) opens it, one frame of
/// it per texture. On entry boss doors go to 16 (6494 swinging shut), treasure doors to 32 (6541,
/// on the first floor) and curse doors to 36 (standing shut); once the room
/// is clear, to 15, 33 and 37, whose `p` opens them.
const CLOSING_DOOR_FRAME: u16 = 4;
const OPENING_DOOR_FRAME: u16 = 3;
/// Frame 5: a plain door blown open while shut, in smoke (6477).
const BLOWN_DOOR_FRAME: u16 = 5;
const CLOSING_BOSS_DOOR_FRAME: u16 = 16;
const OPENING_BOSS_DOOR_FRAME: u16 = 15;
const CLOSING_TREASURE_DOOR_FRAME: u16 = 32;
const OPENING_TREASURE_DOOR_FRAME: u16 = 33;
const SHUT_CURSE_DOOR_FRAME: u16 = 36;

macro_rules! opening_door {
    ($($frame:literal)*) => {
        [$(RenderOverrides {
            children: &[("p", ChildState::Frame($frame))],
            texts: &[],
            hidden_characters: &[],
            character_frames: &[],
            tint: None,
        }),*]
    };
}

static OPENING_DOOR: [RenderOverrides; 9] = opening_door!(1 2 3 4 5 6 7 8 9);

/// The crater drawn into the floor bitmap after a blast.
const SCORCH_CLIP: CharacterId = 6474;
const SCORCH_FRAME: u16 = 1;

pub(super) fn source_for(texture_id: TextureId) -> TextureSource {
    match texture_id {
        TextureId::OrbitingFly(frame) => animated(FLY_CLIP, FLY_FRAME, frame),
        TextureId::Door(DoorTexture::Open(DoorStyle::Plain)) => framed(DOOR_CLIP, OPEN_DOOR_FRAME),
        TextureId::Door(DoorTexture::Open(DoorStyle::Treasure)) => framed(DOOR_CLIP, OPEN_TREASURE_DOOR_FRAME),
        TextureId::Door(DoorTexture::Open(DoorStyle::Boss)) => framed(DOOR_CLIP, BOSS_DOOR_FRAME),
        TextureId::Door(DoorTexture::Open(DoorStyle::Devil)) => framed(DOOR_CLIP, DEVIL_DOOR_FRAME),
        TextureId::Door(DoorTexture::Open(DoorStyle::Curse)) => {
            animated(DOOR_CLIP, OPEN_CURSE_DOOR_FRAME, OPEN_CURSE_DOOR_LAST_FRAME)
        }
        TextureId::Door(DoorTexture::Open(DoorStyle::BlownOpen)) => framed(DOOR_CLIP, BLOWN_OPEN_DOOR_FRAME),
        TextureId::Door(DoorTexture::Open(DoorStyle::ShopLocked)) => {
            animated(DOOR_CLIP, LOCKED_SHOP_DOOR_FRAME, LOCKED_SHOP_DOOR_LAST_FRAME)
        }
        TextureId::Door(DoorTexture::Open(DoorStyle::ShopOpen)) => {
            animated(DOOR_CLIP, UNLOCKED_SHOP_DOOR_FRAME, UNLOCKED_SHOP_DOOR_LAST_FRAME)
        }
        TextureId::Door(DoorTexture::ShopUnlocking(frame)) => animated(DOOR_CLIP, UNLOCKED_SHOP_DOOR_FRAME, frame),
        TextureId::Door(DoorTexture::Blown(frame)) => animated(DOOR_CLIP, BLOWN_DOOR_FRAME, frame),
        TextureId::Door(DoorTexture::Closing(style, frame)) => animated(DOOR_CLIP, closing_frame(style), frame),
        TextureId::Door(DoorTexture::Opening(DoorStyle::Plain, frame)) => TextureSource {
            overrides: &OPENING_DOOR[usize::from(frame).min(OPENING_DOOR.len() - 1)],
            ..framed(DOOR_CLIP, OPENING_DOOR_FRAME)
        },
        TextureId::Door(DoorTexture::Opening(style, frame)) => animated(DOOR_CLIP, opening_frame(style), frame),
        TextureId::BombFuse(frame) => animated(BOMB_CLIP, BOMB_FUSE_FRAME, frame),
        TextureId::Explosion(frame) => animated(BOMB_CLIP, BOMB_EXPLOSION_FRAME, frame),
        TextureId::GasCloud(frame) => animated(BOMB_CLIP, BOMB_GAS_FRAME, frame),
        _ => framed(SCORCH_CLIP, SCORCH_FRAME),
    }
}

fn closing_frame(style: DoorStyle) -> u16 {
    match style {
        DoorStyle::Boss => CLOSING_BOSS_DOOR_FRAME,
        DoorStyle::Treasure => CLOSING_TREASURE_DOOR_FRAME,
        DoorStyle::Curse => SHUT_CURSE_DOOR_FRAME,
        _ => CLOSING_DOOR_FRAME,
    }
}

fn opening_frame(style: DoorStyle) -> u16 {
    match style {
        DoorStyle::Boss => OPENING_BOSS_DOOR_FRAME,
        DoorStyle::Treasure => OPENING_TREASURE_DOOR_FRAME,
        _ => OPEN_CURSE_DOOR_FRAME,
    }
}

fn animated(character_id: CharacterId, frame_number: u16, elapsed_frames: u8) -> TextureSource {
    TextureSource {
        selection: FrameSelection {
            frame_number,
            frames_since_frame_started: u16::from(elapsed_frames),
        },
        ..framed(character_id, frame_number)
    }
}

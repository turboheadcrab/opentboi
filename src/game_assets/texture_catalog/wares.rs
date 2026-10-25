//! Shop wares (clip `b5`, 3376, frame 15): its `d` on the ware's frame; soul hearts show the
//! heart's variant `p` on frame 3 (`col = 3`), others on frame 1, and the pill ware shows the
//! picture of its `col`.

use super::shared_clips::{FUSE_SPARK_CLIP, PICKUP_CLIP};
use super::{TextureSource, framed};
use crate::game_assets::vector_art::{ChildState, RenderOverrides};

const WARE_FRAME: u16 = 15;
const WARE_FRAME_COUNT: u8 = 17;

macro_rules! ware_overrides {
    ($($frame:literal)*) => {
        [$(RenderOverrides {
            children: &[("d", ChildState::Frame($frame)), ("d.d.d.p", ChildState::Frame(1))],
            texts: &[],
            hidden_characters: &[FUSE_SPARK_CLIP],
            character_frames: &[],
            tint: None,
        }),*]
    };
}

static WARES: [RenderOverrides; WARE_FRAME_COUNT as usize] = ware_overrides!(1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17);
const CONSUMABLE_WARE_COLORS: u8 = 15;

macro_rules! consumable_ware_overrides {
    ($frame:literal; $($color:literal)*) => {
        [$(RenderOverrides {
            children: &[("d", ChildState::Frame($frame)), ("d.d", ChildState::Frame($color))],
            texts: &[],
            hidden_characters: &[FUSE_SPARK_CLIP],
            character_frames: &[],
            tint: None,
        }),*]
    };
}

/// By sale, then `col`.
static CONSUMABLE_WARES: [[RenderOverrides; CONSUMABLE_WARE_COLORS as usize]; 2] = [
    consumable_ware_overrides!(3; 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15),
    consumable_ware_overrides!(7; 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15),
];
static SOUL_HEART_WARE: RenderOverrides = RenderOverrides {
    children: &[("d", ChildState::Frame(13)), ("d.d.d.p", ChildState::Frame(3))],
    texts: &[],
    hidden_characters: &[],
    character_frames: &[],
    tint: None,
};
static SOUL_HEART_SALE_WARE: RenderOverrides = RenderOverrides {
    children: &[("d", ChildState::Frame(17)), ("d.d.d.p", ChildState::Frame(3))],
    texts: &[],
    hidden_characters: &[],
    character_frames: &[],
    tint: None,
};

/// The pill ware: frame 3 of `d` (7 on sale), whose picture clip `d.d` shows the `col`.
pub(super) fn consumable_ware_source(color: u8, on_sale: bool) -> TextureSource {
    let overrides = &CONSUMABLE_WARES[usize::from(on_sale)][usize::from(color.clamp(1, CONSUMABLE_WARE_COLORS) - 1)];
    TextureSource {
        overrides,
        ..framed(PICKUP_CLIP, WARE_FRAME)
    }
}

pub(super) fn ware_source(frame: u8) -> TextureSource {
    const SOUL_HEART: u8 = 13;
    const SOUL_HEART_ON_SALE: u8 = 17;
    let overrides = match frame {
        SOUL_HEART => &SOUL_HEART_WARE,
        SOUL_HEART_ON_SALE => &SOUL_HEART_SALE_WARE,
        _ => &WARES[usize::from(frame.clamp(1, WARE_FRAME_COUNT) - 1)],
    };
    TextureSource {
        overrides,
        ..framed(PICKUP_CLIP, WARE_FRAME)
    }
}

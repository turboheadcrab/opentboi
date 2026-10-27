//! The in-game HUD (clip 1441, linkage `hud`, attached at the stage's top left). The clip holds
//! every element at once; the game hides or sets them each frame. These overrides show the frame
//! around the live values: no boss bar or pill and card pop-ups, no active item, and no hearts or
//! counters (drawn separately, see `hud_values`).

use swf::CharacterId;

use super::{TextureSource, framed};
use crate::game::{Item, MapPiece};
use crate::game_assets::vector_art::{ChildState, FrameSelection, NO_OVERRIDES, RenderOverrides};

const HUD_CLIP: CharacterId = 1441;
pub(super) const ACTIVE_ITEM_LABEL_SHAPE: CharacterId = 786;
/// The stage is 640×480; the texture is centered on it.
const STAGE_CENTER: (f32, f32) = (320.0, 240.0);

const FLOOR_NAME: &str = "Basement 1";
const FLOOR_NAME_BOX_PER_MILLE: u16 = crate::whole_numbers::clamped_u16(FLOOR_NAME.len() as f32 * 8.6 * 10.0 + 153.0);

const RUN_START: RenderOverrides = RenderOverrides {
    children: &[
        // Boss health bar (`bar._visible`), pill and card names (`extra`…, `black`…, `plix`…),
        // the pill slot and the hard-mode marker.
        ("bar", ChildState::Hidden),
        ("extra", ChildState::Hidden),
        ("extra2", ChildState::Hidden),
        ("extra3", ChildState::Hidden),
        ("black", ChildState::Hidden),
        ("black2", ChildState::Hidden),
        ("plix", ChildState::Hidden),
        ("plix2", ChildState::Hidden),
        ("pilll", ChildState::Hidden),
        ("hardon", ChildState::Hidden),
        // The floor-name banner (`st1`) has slid away once play starts.
        ("st1", ChildState::Hidden),
        // The active item's box is drawn separately, with the item and its charge.
        ("it", ChildState::Hidden),
        // Hearts and counters are drawn separately from the inventory.
        ("lif", ChildState::Hidden),
        ("kep", ChildState::Hidden),
        // The floor name's box fits the name: `boxx._xscale = length * 8.6 + 15.3`.
        (
            "boxx",
            ChildState::HorizontalScale {
                per_mille: FLOOR_NAME_BOX_PER_MILLE,
            },
        ),
    ],
    // The counters show two digits; `chaps` is the floor's name.
    texts: &[
        ("coins", ""),
        ("bombs", ""),
        ("keys", ""),
        ("chaps", FLOOR_NAME),
        ("ups", ""),
        // Pill and card names.
        ("tex", ""),
        ("tex2", ""),
        ("tex3", ""),
    ],
    // The active item's key label (shape 786, placed by clip 794 at depth 19): drawn separately so it
    // can name the device's own button.
    hidden_characters: &[ACTIVE_ITEM_LABEL_SHAPE],
    character_frames: &[],
    tint: None,
};

/// The active item's box (`it`, clip 794) on `frame` from 0, its `d` (clip 782) on the item's
/// picture (frame 20 none, 37 the unicorn), without the label.
const ITEM_BOX_CLIP: CharacterId = 794;
const NO_ITEM_PICTURE: u16 = 20;

macro_rules! box_pictures {
    ($($picture:expr),*) => {
        [$(RenderOverrides {
            children: &[("d", ChildState::Frame($picture))],
            texts: &[],
            hidden_characters: &[ACTIVE_ITEM_LABEL_SHAPE],
            character_frames: &[],
            tint: None,
        }),*]
    };
}

/// None, then by `Item::ACTIVE`.
static BOX_PICTURES: [RenderOverrides; 4] = box_pictures!(
    NO_ITEM_PICTURE,
    Item::MyLittleUnicorn.active_picture(),
    Item::DeckOfCards.active_picture(),
    Item::MomsBottleOfPills.active_picture()
);

pub(super) fn item_box_source(item: Option<Item>, frame: u8) -> TextureSource {
    let index = item.map_or(0, |item| {
        1 + Item::ACTIVE.iter().position(|&each| each == item).unwrap_or(0)
    });
    TextureSource {
        overrides: &BOX_PICTURES[index],
        ..framed(ITEM_BOX_CLIP, u16::from(frame) + 1)
    }
}

/// The label's shape on its own, centered on its bounds (-20.1..20.0, -41.0..-24.6) in the item
/// box's clip.
const ACTIVE_ITEM_LABEL_CENTER: (f32, f32) = (-0.05, -32.8);

pub(super) fn active_item_label_source() -> TextureSource {
    TextureSource {
        character_id: ACTIVE_ITEM_LABEL_SHAPE,
        selection: FrameSelection::start_of(1),
        origin_x: ACTIVE_ITEM_LABEL_CENTER.0,
        origin_y: ACTIVE_ITEM_LABEL_CENTER.1,
        overrides: &NO_OVERRIDES,
    }
}

pub(super) fn source() -> TextureSource {
    TextureSource {
        character_id: HUD_CLIP,
        selection: FrameSelection::start_of(1),
        origin_x: STAGE_CENTER.0,
        origin_y: STAGE_CENTER.1,
        overrides: &RUN_START,
    }
}

/// The minimap piece clip `mpiece` (6580): frame 1 seen, 2 visited, 3 current room; 4 shop, 5
/// treasure, 6 boss, 7 secret, 8 mini-boss and 16 super secret icons.
const MAP_PIECE_CLIP: CharacterId = 6580;
/// The mark's `empty` frame: no pickup color marked on the room.
const NO_PICKUP_MARK: RenderOverrides = RenderOverrides {
    children: &[("itmc", ChildState::Frame(11))],
    texts: &[],
    hidden_characters: &[],
    character_frames: &[],
    tint: None,
};

pub(super) fn map_piece_source(piece: MapPiece) -> TextureSource {
    let frame = match piece {
        MapPiece::Seen => 1,
        MapPiece::Visited => 2,
        MapPiece::Current => 3,
        MapPiece::ShopIcon => 4,
        MapPiece::TreasureIcon => 5,
        MapPiece::BossIcon => 6,
        MapPiece::SecretIcon => 7,
        MapPiece::SuperSecretIcon => 16,
        MapPiece::CurseIcon => 14,
        MapPiece::SacrificeIcon => 13,
        MapPiece::LibraryIcon => 12,
        MapPiece::MiniBossIcon => 8,
    };
    TextureSource {
        overrides: &NO_PICKUP_MARK,
        ..framed(MAP_PIECE_CLIP, frame)
    }
}

/// The root timeline's `overlay` (sprite 6221, kept above the game): one
/// 3000×3000 bitmap, one pixel per stage unit, with a hole for the stage. It sits at (320.1, 240.1),
/// so the stage center (320, 240) is (-0.1, -0.1) in its coordinates.
const STAGE_SURROUND_SPRITE: CharacterId = 6221;
const STAGE_SURROUND_CENTER: (f32, f32) = (-0.1, -0.1);
pub(super) const STAGE_SURROUND_PIXELS_PER_UNIT: f32 = 1.0;

pub(super) fn stage_surround_source() -> TextureSource {
    TextureSource {
        origin_x: STAGE_SURROUND_CENTER.0,
        origin_y: STAGE_SURROUND_CENTER.1,
        ..framed(STAGE_SURROUND_SPRITE, 1)
    }
}

//! The HUD's trinket slot: the HUD clip (1441) cropped to its top left corner, with what the game
//! shows there when a trinket is held and nothing else. The picture is `extra2` (clip 1432 frame 11,
//! its `p` on the trinket's frame), the strip behind the name `black` (as wide as the name asks, see
//! `cards`), the name the text `tex2`, and the ring `plix` (frame 1; the taking animation replaces
//! it, frame 2).

use swf::CharacterId;

use super::TextureSource;
use super::cards::{card_texts, name_strip_per_mille};
use crate::game::{SlotRing, TRINKET_SLOT_CENTER, Trinket};
use crate::game_assets::vector_art::{ChildState, FrameSelection, NO_OVERRIDES, RenderOverrides};

const HUD_CLIP: CharacterId = 1441;
/// The HUD's other named parts, which the game hides or leaves empty in the corner.
const OTHER_PARTS: [&str; 13] = [
    "hardon", "bar", "boxx", "lif", "weap", "it", "kep", "cacher", "paus", "pilll", "extra", "extra3", "st1",
];
const SECOND_SLOT_PARTS: [&str; 2] = ["black2", "plix2"];
/// The shapes the HUD clip places without a name: its frame, boxes and labels.
const FRAME_SHAPES: [CharacterId; 9] = [587, 590, 602, 795, 796, 797, 798, 799, 810];
const NAME_TEXT_VARIABLE: &str = "tex2";
const PICTURE_FRAME: u16 = 11;
const RING_FRAME: u16 = 1;

/// The slot holding `trinket`, the name in `text`, with or without the ring.
pub(super) fn slot_overrides(trinket: Trinket, text: &'static str, ring: SlotRing) -> RenderOverrides {
    let mut children: Vec<(&'static str, ChildState)> = OTHER_PARTS
        .into_iter()
        .chain(SECOND_SLOT_PARTS)
        .map(|part| (part, ChildState::Hidden))
        .collect();
    children.push(("extra2.d", ChildState::Frame(PICTURE_FRAME)));
    children.push(("extra2.d.p", ChildState::Frame(u16::from(trinket.color_variant()))));
    children.push((
        "black",
        ChildState::HorizontalScale {
            per_mille: name_strip_per_mille(text),
        },
    ));
    children.push((
        "plix",
        match ring {
            SlotRing::Shown => ChildState::Frame(RING_FRAME),
            SlotRing::Hidden => ChildState::Hidden,
        },
    ));
    RenderOverrides {
        children: Box::leak(children.into_boxed_slice()),
        texts: Box::leak(Box::new([
            (NAME_TEXT_VARIABLE, text),
            ("tex", ""),
            ("tex3", ""),
            ("chaps", ""),
            ("ups", ""),
        ])),
        hidden_characters: &FRAME_SHAPES,
        character_frames: &[],
        tint: None,
    }
}

pub(super) fn source(trinket: Trinket, ring: SlotRing) -> TextureSource {
    let overrides = card_texts()
        .trinket_slots
        .get(trinket.index())
        .map_or(&NO_OVERRIDES, |slots| &slots[ring.index()]);
    TextureSource {
        character_id: HUD_CLIP,
        selection: FrameSelection::start_of(1),
        origin_x: TRINKET_SLOT_CENTER.x,
        origin_y: TRINKET_SLOT_CENTER.y,
        overrides,
    }
}

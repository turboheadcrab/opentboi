//! The HUD's trinket slots: the HUD clip (1441) cropped to the corner of one slot, with what the
//! game shows there when a trinket is held and nothing else. The first slot is the picture `extra2`
//! (clip 1432 frame 11, its `p` on the trinket's frame), the strip behind the name `black` (as wide as
//! the name asks, see `cards`), the name the text `tex2` and the ring `plix` (frame 1; the taking
//! animation replaces it, frame 2); the second is `extra3`, `black2`, `tex3` and `plix2`.

use swf::CharacterId;

use super::TextureSource;
use super::cards::{card_texts, name_strip_per_mille};
use crate::game::{SlotView, Trinket};
use crate::game_assets::vector_art::{ChildState, FrameSelection, NO_OVERRIDES, RenderOverrides};

const HUD_CLIP: CharacterId = 1441;
/// The HUD's other named parts, which the game hides or leaves empty in the corner.
const OTHER_PARTS: [&str; 12] = [
    "hardon", "bar", "boxx", "lif", "weap", "it", "kep", "cacher", "paus", "pilll", "extra", "st1",
];
/// The shapes the HUD clip places without a name: its frame, boxes and labels.
const FRAME_SHAPES: [CharacterId; 9] = [587, 590, 602, 795, 796, 797, 798, 799, 810];
const PICTURE_FRAME: u16 = 11;
const RING_FRAME: u16 = 1;
/// Text fields the game leaves empty here.
const EMPTY_TEXTS: [&str; 3] = ["tex", "chaps", "ups"];

/// The parts of one slot and the text variable of its name.
struct SlotParts {
    picture: &'static str,
    picture_clip: &'static str,
    picture_color: &'static str,
    strip: &'static str,
    ring: &'static str,
    name: &'static str,
}

const FIRST_SLOT: SlotParts = SlotParts {
    picture: "extra2",
    picture_clip: "extra2.d",
    picture_color: "extra2.d.p",
    strip: "black",
    ring: "plix",
    name: "tex2",
};
const SECOND_SLOT: SlotParts = SlotParts {
    picture: "extra3",
    picture_clip: "extra3.d",
    picture_color: "extra3.d.p",
    strip: "black2",
    ring: "plix2",
    name: "tex3",
};

impl SlotParts {
    fn hidden(&self) -> [&'static str; 3] {
        [self.picture, self.strip, self.ring]
    }
}

/// The slot `view` shows holding `trinket`, with the name in `text`.
pub(super) fn slot_overrides(trinket: Trinket, text: &'static str, view: SlotView) -> RenderOverrides {
    let (shown, other) = match view {
        SlotView::First | SlotView::FirstBeingTaken => (&FIRST_SLOT, &SECOND_SLOT),
        SlotView::Second => (&SECOND_SLOT, &FIRST_SLOT),
    };
    let mut children: Vec<(&'static str, ChildState)> = OTHER_PARTS
        .into_iter()
        .chain(other.hidden())
        .map(|part| (part, ChildState::Hidden))
        .collect();
    children.push((shown.picture_clip, ChildState::Frame(PICTURE_FRAME)));
    children.push((
        shown.picture_color,
        ChildState::Frame(u16::from(trinket.color_variant())),
    ));
    children.push((
        shown.strip,
        ChildState::HorizontalScale {
            per_mille: name_strip_per_mille(text),
        },
    ));
    children.push((
        shown.ring,
        if view == SlotView::FirstBeingTaken {
            ChildState::Hidden
        } else {
            ChildState::Frame(RING_FRAME)
        },
    ));
    let mut texts = vec![(shown.name, text), (other.name, "")];
    texts.extend(EMPTY_TEXTS.map(|variable| (variable, "")));
    RenderOverrides {
        children: Box::leak(children.into_boxed_slice()),
        texts: Box::leak(texts.into_boxed_slice()),
        hidden_characters: &FRAME_SHAPES,
        character_frames: &[],
        tint: None,
    }
}

pub(super) fn source(trinket: Trinket, view: SlotView) -> TextureSource {
    let overrides = card_texts()
        .trinket_slots
        .get(trinket.index())
        .map_or(&NO_OVERRIDES, |slots| &slots[view.index()]);
    let center = view.center();
    TextureSource {
        character_id: HUD_CLIP,
        selection: FrameSelection::start_of(1),
        origin_x: center.x,
        origin_y: center.y,
        overrides,
    }
}

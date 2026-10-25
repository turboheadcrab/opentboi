//! The pills' pictures and names in the HUD: the card clips on a pill's own frame.

use super::cards::{HELD_NAME_CLIP, HELD_PICTURE_CLIP, card_texts};
use super::{TextureSource, framed};
use crate::game::{PickupPill, PillText};
use crate::game_assets::vector_art::{ChildState, NO_OVERRIDES, RenderOverrides};

/// The pill's picture: the same clip and frame as a card's, on the pill's look.
macro_rules! held_pills {
    ($($color:literal)*) => {
        [$(RenderOverrides {
            children: &[("d", ChildState::Frame(11)), ("d.p", ChildState::Frame($color))],
            texts: &[],
            hidden_characters: &[],
            character_frames: &[],
            tint: None,
        }),*]
    };
}

static HELD_PILLS: [RenderOverrides; 6] = held_pills!(1 2 3 4 5 6);

pub(super) fn held_picture_source(pill: PickupPill) -> TextureSource {
    TextureSource {
        overrides: &HELD_PILLS[pill.index()],
        ..framed(HELD_PICTURE_CLIP, 1)
    }
}

pub(super) fn held_name_source(text: PillText) -> TextureSource {
    let overrides = card_texts().pill_names.get(text_index(text)).unwrap_or(&NO_OVERRIDES);
    TextureSource {
        overrides,
        ..framed(HELD_NAME_CLIP, 1)
    }
}

/// A pill text's place in `PillText::ALL`.
pub(super) fn text_index(text: PillText) -> usize {
    PillText::ALL.iter().position(|&each| each == text).unwrap_or(0)
}

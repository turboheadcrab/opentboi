//! Single characters of the HUD font (Upheaval, font 601) in the counter field (795), for text the
//! game composes itself, such as the stats panel.

use super::shared_clips::{COUNTER_CENTER, COUNTER_FIELD, COUNTER_VARIABLE};

use super::TextureSource;
use crate::game::HUD_GLYPHS;
use crate::game_assets::vector_art::{FrameSelection, RenderOverrides};

macro_rules! glyph_texts {
    ($($text:literal)*) => {
        [$(RenderOverrides { children: &[], texts: &[(COUNTER_VARIABLE, $text)], hidden_characters: &[], character_frames: &[], tint: None }),*]
    };
}

/// In the order of `HUD_GLYPHS`.
static GLYPHS: [RenderOverrides; 39] = glyph_texts!("-" "." "0" "1" "2" "3" "4" "5" "6" "7" "8" "9" ":" "A" "B" "C" "D" "E" "F" "G" "H" "I" "J" "K" "L" "M" "N" "O" "P" "Q" "R" "S" "T" "U" "V" "W" "X" "Y" "Z");

pub(super) fn source(glyph: u8) -> TextureSource {
    let index = HUD_GLYPHS.bytes().position(|each| each == glyph).unwrap_or(0);
    TextureSource {
        character_id: COUNTER_FIELD,
        selection: FrameSelection::start_of(1),
        origin_x: COUNTER_CENTER.0,
        origin_y: COUNTER_CENTER.1,
        overrides: &GLYPHS[index],
    }
}

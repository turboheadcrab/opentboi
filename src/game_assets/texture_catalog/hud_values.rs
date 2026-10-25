//! Textures for the HUD's live values: the coin, bomb and key counters and the life bar's hearts.

use super::shared_clips::{COUNTER_CENTER, COUNTER_FIELD, COUNTER_VARIABLE};
use swf::CharacterId;

use super::{TextureSource, framed};
use std::sync::OnceLock;

use crate::game::{HudHeart, MAXIMUM_COUNTER_VALUE, PickupMessage, texture_center_in_clip};
use crate::game_assets::script::GameScript;
use crate::game_assets::vector_art::CharacterLibrary;
use crate::game_assets::vector_art::{FrameSelection, NO_OVERRIDES, RenderOverrides};

/// The heart clip in `lif`: frame 1 full, 2 half, 3 empty, 5 and 6 soul hearts, 8 eternal.
const HEART_CLIP: CharacterId = 609;

macro_rules! counter_texts {
    ($($text:literal)*) => {
        [$(RenderOverrides { children: &[], texts: &[(COUNTER_VARIABLE, $text)], hidden_characters: &[], character_frames: &[], tint: None }),*]
    };
}

/// Counters below 10 get a leading zero.
static COUNTER_TEXTS: [RenderOverrides; MAXIMUM_COUNTER_VALUE as usize + 1] = counter_texts!(
        "00" "01" "02" "03" "04" "05" "06" "07" "08" "09"
        "10" "11" "12" "13" "14" "15" "16" "17" "18" "19"
        "20" "21" "22" "23" "24" "25" "26" "27" "28" "29"
        "30" "31" "32" "33" "34" "35" "36" "37" "38" "39"
        "40" "41" "42" "43" "44" "45" "46" "47" "48" "49"
        "50" "51" "52" "53" "54" "55" "56" "57" "58" "59"
        "60" "61" "62" "63" "64" "65" "66" "67" "68" "69"
        "70" "71" "72" "73" "74" "75" "76" "77" "78" "79"
        "80" "81" "82" "83" "84" "85" "86" "87" "88" "89"
        "90" "91" "92" "93" "94" "95" "96" "97" "98" "99"
);

pub(super) fn counter_source(value: u8) -> TextureSource {
    TextureSource {
        character_id: COUNTER_FIELD,
        selection: FrameSelection::start_of(1),
        origin_x: COUNTER_CENTER.0,
        origin_y: COUNTER_CENTER.1,
        overrides: &COUNTER_TEXTS[usize::from(value.min(MAXIMUM_COUNTER_VALUE))],
    }
}

/// `over` (6832) sits at (-2.75, -9.55) on the root timeline, over everything; its frame 16
/// plays the eternal hearts' union. The texture covers the stage, centered on (320, 240).
pub(super) const OVERLAY_CLIP: CharacterId = 6832;
const ETERNAL_HEART_UNION_FRAME: u16 = 16;
pub(super) const STAGE_CENTER_IN_OVERLAY: (f32, f32) = (320.0 + 2.75, 240.0 + 9.55);

pub(super) fn eternal_heart_union_source(frame: u8) -> TextureSource {
    TextureSource {
        character_id: OVERLAY_CLIP,
        selection: FrameSelection {
            frame_number: ETERNAL_HEART_UNION_FRAME,
            frames_since_frame_started: u16::from(frame),
        },
        origin_x: STAGE_CENTER_IN_OVERLAY.0,
        origin_y: STAGE_CENTER_IN_OVERLAY.1,
        overrides: &NO_OVERRIDES,
    }
}

/// The HUD key icon `kep`: frame 1 the key, frame 2 the golden key.
const KEY_ICON_CLIP: CharacterId = 802;
const GOLDEN_KEY_FRAME: u16 = 2;

pub(super) fn key_icon_source(golden: bool) -> TextureSource {
    framed(KEY_ICON_CLIP, if golden { GOLDEN_KEY_FRAME } else { 1 })
}

pub(super) fn heart_source(heart: HudHeart) -> TextureSource {
    let frame = match heart {
        HudHeart::Full => 1,
        HudHeart::Half => 2,
        HudHeart::Empty => 3,
        HudHeart::SoulFull => 5,
        HudHeart::SoulHalf => 6,
        HudHeart::Eternal => 8,
    };
    framed(HEART_CLIP, frame)
}

/// `st2` (6834) with its text `tex`; frames from 1.
const MESSAGE_CLIP: CharacterId = 6834;
const MESSAGE_TEXT_VARIABLE: &str = "tex";

/// The messages' texts, read once from the script (see [`PickupMessage`]), with their overrides.
static MESSAGES: OnceLock<Vec<(usize, RenderOverrides)>> = OnceLock::new();

/// Reads the pickup messages from `library`'s script, once per run; call before rendering one.
pub(super) fn read_messages(library: &CharacterLibrary) {
    MESSAGES.get_or_init(|| {
        let texts = GameScript::from_movie(library.movie()).message_texts();
        if texts.len() < PickupMessage::ALL.len() {
            println!(
                "Pickup messages found in the game: {} of {}",
                texts.len(),
                PickupMessage::ALL.len()
            );
        }
        texts
            .into_iter()
            .map(|text| {
                let text: &'static str = Box::leak(text.into_boxed_str());
                let overrides = RenderOverrides {
                    children: &[],
                    texts: Box::leak(Box::new([(MESSAGE_TEXT_VARIABLE, text)])),
                    hidden_characters: &[],
                    character_frames: &[],
                    tint: None,
                };
                (text.chars().count(), overrides)
            })
            .collect()
    });
}

pub(super) fn message_source(message: PickupMessage, frame: u8) -> TextureSource {
    let Some((characters, overrides)) = MESSAGES
        .get()
        .and_then(|messages| messages.get(message.script_position()))
    else {
        return framed(MESSAGE_CLIP, 1 + u16::from(frame));
    };
    TextureSource {
        overrides,
        origin_x: texture_center_in_clip(*characters),
        ..framed(MESSAGE_CLIP, 1 + u16::from(frame))
    }
}

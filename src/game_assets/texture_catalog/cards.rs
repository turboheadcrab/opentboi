//! Cards' textures beyond the pickup: the banner with a card's name (`hud.st1`, 1440, text `tex`)
//! the full-screen animations of `over` (6832), and the HUD's card held: its picture
//! (`extra`, 1432: clip 782 frame 11, `p` on the card's frame) and its name (`pilll`, 1431, whose
//! black strip is `length * 7 + 20` percent wide).

use swf::CharacterId;

use super::hud_values::{OVERLAY_CLIP, STAGE_CENTER_IN_OVERLAY};
use super::{TextureSource, framed};
use std::sync::OnceLock;

use crate::game::{BannerName, CARD_NAME_TEXTURE_CENTER_X, Item, PickupCard, TextureId};
use crate::game_assets::script::GameScript;
use crate::game_assets::vector_art::CharacterLibrary;
use crate::game_assets::vector_art::{ChildState, FrameSelection, NO_OVERRIDES, RenderOverrides};

const BANNER_CLIP: CharacterId = 1440;
const BANNER_TEXT_VARIABLE: &str = "tex";

pub(super) fn source_for(texture_id: TextureId) -> TextureSource {
    match texture_id {
        TextureId::NameBanner { name, frame } => name_source(name, frame),
        TextureId::HeldCard(card) => held_picture_source(card),
        TextureId::HeldCardName(card) => held_name_source(card),
        TextureId::HeldPill(pill) => super::pills::held_picture_source(pill),
        TextureId::HeldPillName(text) => super::pills::held_name_source(text),
        TextureId::CardSlotAnimation { used, frame } => {
            let clip_frame = if used { SLOT_USED_FRAME } else { SLOT_TAKEN_FRAME };
            TextureSource {
                selection: FrameSelection {
                    frame_number: clip_frame,
                    frames_since_frame_started: u16::from(frame),
                },
                overrides: &WITHOUT_LABEL,
                ..framed(HELD_NAME_CLIP, clip_frame)
            }
        }
        TextureId::CardAnimation { clip_frame, frame } => animation_source(clip_frame, frame),
        _ => held_picture_source(PickupCard::WheelOfFortune),
    }
}

/// The strip's other frames keep the badge placed on its first, so they hide it too.
static WITHOUT_LABEL: RenderOverrides = RenderOverrides {
    hidden_characters: &[CARD_LABEL_SHAPE],
    ..NO_OVERRIDES
};

fn name_source(name: BannerName, frame: u8) -> TextureSource {
    let texts = card_texts();
    let overrides = match name {
        BannerName::Card(card) => texts.banners.get(card_index(card)),
        BannerName::Item(item) => texts.item_banners.get(item_index(item)),
        BannerName::Fortune(index) => texts.fortune_banners.get(usize::from(index)),
        BannerName::Pill(text) => texts.pill_banners.get(super::pills::text_index(text)),
    }
    .unwrap_or(&NO_OVERRIDES);
    TextureSource {
        overrides,
        origin_x: CARD_NAME_TEXTURE_CENTER_X,
        ..framed(BANNER_CLIP, 1 + u16::from(frame))
    }
}

fn animation_source(clip_frame: u8, frame: u8) -> TextureSource {
    TextureSource {
        selection: FrameSelection {
            frame_number: u16::from(clip_frame),
            frames_since_frame_started: u16::from(frame),
        },
        origin_x: STAGE_CENTER_IN_OVERLAY.0,
        origin_y: STAGE_CENTER_IN_OVERLAY.1,
        ..framed(OVERLAY_CLIP, u16::from(clip_frame))
    }
}

pub(super) const HELD_PICTURE_CLIP: CharacterId = 1432;
pub(super) const HELD_NAME_CLIP: CharacterId = 1431;
/// The "Q" badge on the name strip, a shape of its own; bounds (-1.15..17.7, -4.65..12.15) in the
/// strip's clip.
const CARD_LABEL_SHAPE: CharacterId = 1428;
const CARD_LABEL_CENTER: (f32, f32) = (8.275, 3.75);
/// `pilll` frames: 2 a card taken (1429), 3 a card used (1430).
const SLOT_TAKEN_FRAME: u16 = 2;
const SLOT_USED_FRAME: u16 = 3;
const NAME_STRIP_PER_CHARACTER: u16 = 7;
const NAME_STRIP_BASE: u16 = 20;
const PER_MILLE_PER_PERCENT: u16 = 10;

/// The picture clip on frame 11 (cards and pills), its `p` on the card's own frame (`col`).
macro_rules! held_pictures {
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

/// By index into `Card::ALL`.
static HELD_PICTURES: [RenderOverrides; 27] = held_pictures!(
    7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28 70 71 72 73 74
);

fn held_picture_source(card: PickupCard) -> TextureSource {
    let index = PickupCard::ALL.iter().position(|&each| each == card).unwrap_or(0);
    TextureSource {
        overrides: &HELD_PICTURES[index],
        ..framed(HELD_PICTURE_CLIP, 1)
    }
}

fn held_name_source(card: PickupCard) -> TextureSource {
    TextureSource {
        overrides: &card_texts().held_names[card_index(card)],
        ..framed(HELD_NAME_CLIP, 1)
    }
}

/// The badge's shape on its own, centered on its bounds.
pub(super) fn label_source() -> TextureSource {
    TextureSource {
        character_id: CARD_LABEL_SHAPE,
        selection: FrameSelection::start_of(1),
        origin_x: CARD_LABEL_CENTER.0,
        origin_y: CARD_LABEL_CENTER.1,
        overrides: &NO_OVERRIDES,
    }
}

/// The names' textures, built once from the names the movie's script lists: the cards' in
/// `tart()`, the items' in the main timeline's list of item names (by item number).
pub(super) struct CardTexts {
    banners: Vec<RenderOverrides>,
    held_names: Vec<RenderOverrides>,
    item_banners: Vec<RenderOverrides>,
    /// The fortune machine's, in the order of the script's list.
    fortune_banners: Vec<RenderOverrides>,
    /// By place in `PillText::ALL`.
    pub(super) pill_banners: Vec<RenderOverrides>,
    pub(super) pill_names: Vec<RenderOverrides>,
}

static CARD_TEXTS: OnceLock<CardTexts> = OnceLock::new();

fn leaked(text: String) -> &'static str {
    Box::leak(text.into_boxed_str())
}

/// The banner showing `text`.
fn banner_overrides(text: &'static str) -> RenderOverrides {
    RenderOverrides {
        children: &[],
        texts: Box::leak(Box::new([(BANNER_TEXT_VARIABLE, text)])),
        hidden_characters: &[],
        character_frames: &[],
        tint: None,
    }
}

/// The name strip showing `text`, wide enough for it, without the badge that names the button.
fn held_name_overrides(text: &'static str) -> RenderOverrides {
    let characters = u16::try_from(text.len()).unwrap_or(u16::MAX);
    let per_mille = (characters * NAME_STRIP_PER_CHARACTER + NAME_STRIP_BASE) * PER_MILLE_PER_PERCENT;
    RenderOverrides {
        children: Box::leak(Box::new([("black", ChildState::HorizontalScale { per_mille })])),
        hidden_characters: &[CARD_LABEL_SHAPE],
        ..banner_overrides(text)
    }
}

/// Reads the cards' names from `library`'s movie, once per run; call before rendering a card's
/// name. Cards the movie does not name (playing cards before Wrath of the Lamb) show no name.
pub(super) fn read_names(library: &CharacterLibrary) {
    CARD_TEXTS.get_or_init(|| {
        let script = GameScript::from_movie(library.movie());
        let names = script.card_names().unwrap_or_default();
        let item_names = script.item_names();
        let fortunes = script.fortune_messages();
        let pill_texts: Vec<&'static str> = script.pill_texts().into_iter().map(leaked).collect();
        if names.len() < PickupCard::ALL.len() {
            println!(
                "Card names found in the game: {} of {}",
                names.len(),
                PickupCard::ALL.len()
            );
        }
        let name = |index: usize| leaked(names.get(index).cloned().unwrap_or_default());
        let names: Vec<&'static str> = (0..PickupCard::ALL.len()).map(name).collect();
        let item_name = |item: Item| leaked(item_names.get(usize::from(item.number())).cloned().unwrap_or_default());
        CardTexts {
            banners: names.iter().map(|&text| banner_overrides(text)).collect(),
            held_names: names.iter().map(|&text| held_name_overrides(text)).collect(),
            item_banners: Item::ALL.map(|item| banner_overrides(item_name(item))).into(),
            fortune_banners: fortunes
                .into_iter()
                .map(|fortune| banner_overrides(leaked(fortune)))
                .collect(),
            pill_banners: pill_texts.iter().map(|&text| banner_overrides(text)).collect(),
            pill_names: pill_texts.iter().map(|&text| held_name_overrides(text)).collect(),
        }
    });
}

pub(super) fn card_texts() -> &'static CardTexts {
    CARD_TEXTS.get_or_init(|| CardTexts {
        banners: Vec::new(),
        held_names: Vec::new(),
        item_banners: Vec::new(),
        fortune_banners: Vec::new(),
        pill_banners: Vec::new(),
        pill_names: Vec::new(),
    })
}

fn item_index(item: Item) -> usize {
    Item::ALL.iter().position(|&each| each == item).unwrap_or(0)
}

/// A card's place in `Card::ALL` (and in `tart()`'s list, by `col`).
fn card_index(card: PickupCard) -> usize {
    PickupCard::ALL.iter().position(|&each| each == card).unwrap_or(0)
}

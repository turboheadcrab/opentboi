//! Isaac's whole-body animations from his clip `b1` (5825), whose origin is his floor point:
//! frame 7 the teleport (5713), frame 8 the arrival (5714), frame 11 holding up an item (5824 on
//! the skin's frame, then 5816) and frame 4 picking one up (5652, then 5639), frame 9 the thumbs up (5786, then 5726) and frame 10 the
//! thumbs down (5815, then 5787). Both hold the item
//! clip `it` (3355), whose `d` (3354) shows `player.it`: 43 a card (its `p` on the card's frame),
//! 200–203 a heart, bomb, key or soul heart.

use super::isaac_demonic::{BUBBLE_BEHIND, BUBBLE_IN_FRONT};
use super::shared_clips::FUSE_SPARK_CLIP;
use swf::CharacterId;

use super::{TextureSource, framed};
use crate::game::{Held, Item, PickupCard, PlayerAnimation, Trinket};
use crate::game_assets::vector_art::{ChildState, FrameSelection, RenderOverrides};
use std::sync::OnceLock;

const PLAYER_CLIP: CharacterId = 5825;
const TELEPORT_FRAME: u16 = 7;
const HOLDING_UP_FRAME: u16 = 11;
const TELEPORT_ARRIVAL_FRAME: u16 = 8;
const PICKING_UP_FRAME: u16 = 4;
/// Dying is his clip's frame 6, its own child on the skin's frame (Isaac's is 1).
const DYING_FRAME: u16 = 6;

const THUMBS_UP_FRAME: u16 = 9;
const THUMBS_DOWN_FRAME: u16 = 10;

/// The death clip (5683) inside it: Isaac falling (5680) on his spot (5677), under the
/// darkness and beam (5682), which `DeathSpotlight` draws apart (see `death/spotlight.rs`).
const DEATH_CLIP: CharacterId = 5683;
const DEATH_SPOT: CharacterId = 5677;
const DEATH_FALL: CharacterId = 5680;
const DEATH_DARKNESS: CharacterId = 5682;
/// Isaac's floor point is (-20.8, -44.3) in the death clip (`d` at (-20.6, -44.3), its `d` at
/// (-0.2, 0)); the beam's strip is centered 303 above it.
const BEAM_CENTER_IN_DEATH_CLIP: (f32, f32) = (20.8, 44.3 - 303.0);

/// His clip's two colored bubbles, which cycle through the frames of the other animations, are not
/// part of dying.
static DYING: RenderOverrides = RenderOverrides {
    children: &[("d", ChildState::Frame(1))],
    texts: &[],
    hidden_characters: &[DEATH_DARKNESS, BUBBLE_BEHIND, BUBBLE_IN_FRONT],
    character_frames: &[],
    tint: None,
};

/// The thumbs on the skin's first frame (Isaac's own).
static REACTION: RenderOverrides = RenderOverrides {
    children: &[("d", ChildState::Frame(1))],
    texts: &[],
    hidden_characters: &[],
    character_frames: &[],
    tint: None,
};

static SPOTLIGHT: RenderOverrides = RenderOverrides {
    children: &[],
    texts: &[],
    hidden_characters: &[DEATH_SPOT, DEATH_FALL],
    character_frames: &[],
    tint: None,
};

/// The beam's strip of the death clip's frame `step + 1`.
pub(super) fn spotlight_source(step: u8) -> TextureSource {
    TextureSource {
        character_id: DEATH_CLIP,
        selection: FrameSelection::start_of(u16::from(step) + 1),
        origin_x: BEAM_CENTER_IN_DEATH_CLIP.0,
        origin_y: BEAM_CENTER_IN_DEATH_CLIP.1,
        overrides: &SPOTLIGHT,
    }
}

macro_rules! holding_up_wares {
    ($($item:literal)*) => {
        [$(RenderOverrides {
            children: &[("d", ChildState::Frame(1)), ("d.d.it.d", ChildState::Frame($item))],
            texts: &[],
            hidden_characters: &[FUSE_SPARK_CLIP],
            character_frames: &[],
            tint: None,
        }),*]
    };
}

/// By index into `Held::WARES`, `player.it`.
static HOLDING_UP_WARES: [RenderOverrides; 4] = holding_up_wares!(200 201 202 203);
/// By index into `Item::ALL`, each item's number.
static HOLDING_UP_ITEMS: [RenderOverrides; 2] = holding_up_wares!(46 77);

macro_rules! holding_up_cards {
    ($($color:literal)*) => {
        [$(RenderOverrides {
            children: &[
                ("d", ChildState::Frame(1)),
                ("d.d.it.d", ChildState::Frame(43)),
                ("d.d.it.d.p", ChildState::Frame($color)),
            ],
            texts: &[],
            hidden_characters: &[],
            character_frames: &[],
            tint: None,
        }),*]
    };
}

/// By index into `Card::ALL`, each card's `col`.
static HOLDING_UP: [RenderOverrides; 27] = holding_up_cards!(
    7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28 70 71 72 73 74
);

macro_rules! holding_up_pills {
    ($($color:literal)*) => {
        [$(RenderOverrides {
            children: &[
                ("d", ChildState::Frame(1)),
                ("d.d.it.d", ChildState::Frame(43)),
                ("d.d.it.d.p", ChildState::Frame($color)),
            ],
            texts: &[],
            hidden_characters: &[],
            character_frames: &[],
            tint: None,
        }),*]
    };
}

/// By the pill's look, its `col`.
static HOLDING_UP_PILLS: [RenderOverrides; 6] = holding_up_pills!(1 2 3 4 5 6);

/// What Isaac holds up for a trinket: the picture the cards use (43), on the trinket's own frame.
/// Built once, by the trinket's place in `Trinket::ALL`.
fn holding_up_trinket(trinket: Trinket) -> &'static RenderOverrides {
    static HOLDING_UP_TRINKETS: OnceLock<Vec<RenderOverrides>> = OnceLock::new();
    &HOLDING_UP_TRINKETS.get_or_init(|| {
        Trinket::ALL
            .into_iter()
            .map(|each| RenderOverrides {
                children: Box::leak(Box::new([
                    ("d", ChildState::Frame(1)),
                    ("d.d.it.d", ChildState::Frame(43)),
                    ("d.d.it.d.p", ChildState::Frame(u16::from(each.color_variant()))),
                ])),
                texts: &[],
                hidden_characters: &[],
                character_frames: &[],
                tint: None,
            })
            .collect()
    })[trinket.index()]
}

pub(super) fn source(animation: PlayerAnimation, frame: u8) -> TextureSource {
    let clip_frame = match animation {
        PlayerAnimation::Teleport => TELEPORT_FRAME,
        PlayerAnimation::TeleportArrival => TELEPORT_ARRIVAL_FRAME,
        PlayerAnimation::HoldingUp(_) => HOLDING_UP_FRAME,
        PlayerAnimation::PickingUp(_) => PICKING_UP_FRAME,
        PlayerAnimation::Dying => DYING_FRAME,
        PlayerAnimation::ThumbsUp => THUMBS_UP_FRAME,
        PlayerAnimation::ThumbsDown => THUMBS_DOWN_FRAME,
    };
    let base = TextureSource {
        selection: FrameSelection {
            frame_number: clip_frame,
            frames_since_frame_started: u16::from(frame),
        },
        ..framed(PLAYER_CLIP, clip_frame)
    };
    match animation {
        PlayerAnimation::Teleport | PlayerAnimation::TeleportArrival => base,
        PlayerAnimation::Dying => TextureSource {
            overrides: &DYING,
            ..base
        },
        PlayerAnimation::ThumbsUp | PlayerAnimation::ThumbsDown => TextureSource {
            overrides: &REACTION,
            ..base
        },
        PlayerAnimation::HoldingUp(held) | PlayerAnimation::PickingUp(held) => TextureSource {
            overrides: held_overrides(held),
            ..base
        },
    }
}

/// What the held picture of a holding-up or picking-up animation is set to.
fn held_overrides(held: Held) -> &'static RenderOverrides {
    match held {
        Held::Card(card) => &HOLDING_UP[PickupCard::ALL.iter().position(|&each| each == card).unwrap_or(0)],
        Held::Pill(pill) => &HOLDING_UP_PILLS[pill.index()],
        Held::Trinket(trinket) => holding_up_trinket(trinket),
        Held::Item(item) => &HOLDING_UP_ITEMS[Item::ALL.iter().position(|&each| each == item).unwrap_or(0)],
        ware => &HOLDING_UP_WARES[Held::WARES.iter().position(|&each| each == ware).unwrap_or(0)],
    }
}

//! Pickups (clip `b5`, 3376): its frame picks the kind, its child `d` the animation (1 idle,
//! 2 taken, 3 landing) and every nested `p` the variant (`col`), as the original's depth update
//! sets `d.d.p`, `d.d.d.p`, ... each frame.

use super::shared_clips::{FUSE_SPARK_CLIP, PICKUP_CLIP};
use swf::CharacterId;

use super::{TextureSource, framed};
use crate::game::{ChestKind, Item, PickupKind, PickupPhase, Trinket};
use crate::game_assets::vector_art::{ChildState, FrameSelection, RenderOverrides};
use std::sync::OnceLock;

const VARIANT_COUNT: u8 = 5;

macro_rules! pickup_overrides {
    ($phase:literal, $variant:literal) => {
        RenderOverrides {
            children: &[
                ("d", ChildState::Frame($phase)),
                ("d.p", ChildState::Frame($variant)),
                ("d.d.p", ChildState::Frame($variant)),
                ("d.d.d.p", ChildState::Frame($variant)),
                ("d.d.d.d.p", ChildState::Frame($variant)),
            ],
            texts: &[],
            hidden_characters: &[FUSE_SPARK_CLIP],
            character_frames: &[],
            tint: None,
        }
    };
    ($phase:literal) => {
        [
            pickup_overrides!($phase, 1),
            pickup_overrides!($phase, 2),
            pickup_overrides!($phase, 3),
            pickup_overrides!($phase, 4),
            pickup_overrides!($phase, 5),
        ]
    };
}

static IDLE: [RenderOverrides; VARIANT_COUNT as usize] = pickup_overrides!(1);
static COLLECTED: [RenderOverrides; VARIANT_COUNT as usize] = pickup_overrides!(2);
static LANDING: [RenderOverrides; VARIANT_COUNT as usize] = pickup_overrides!(3);
/// The brown chest's `d` (2090) has the red chest's animations four frames on: 5 idle, 6 opened,
/// 7 landing.
static RED_CHEST: [RenderOverrides; 3] = [
    pickup_overrides!(7, 2),
    pickup_overrides!(5, 2),
    pickup_overrides!(6, 2),
];

/// The pedestal clip (3356) puffs in over its first 25 frames and ends standing on frame 45: the
/// altar `al` and the item clip `d`, hidden while there are no items.
static EMPTY_PEDESTAL: RenderOverrides = RenderOverrides {
    children: &[("d", ChildState::Frame(45)), ("d.d", ChildState::Hidden)],
    texts: &[],
    hidden_characters: &[],
    character_frames: &[],
    tint: None,
};

/// A pedestal holding an item: the item's picture (`d.d.d`, 3354) on the item's number, bobbing with
/// its clip `d.d` (3355), whose frames 1–20 loop (frame 21 is the emptied pedestal).
macro_rules! bobbing_item {
    ($item:literal; $($bob:literal)*) => {
        [$(RenderOverrides {
            children: &[
                ("d", ChildState::Frame(45)),
                ("d.d", ChildState::Frame($bob)),
                ("d.d.d", ChildState::Frame($item)),
            ],
            texts: &[],
            hidden_characters: &[],
            character_frames: &[],
            tint: None,
        }),*]
    };
}

/// By index into `Item::ALL`, then the bob's frame.
static ITEM_PEDESTALS: [[RenderOverrides; ITEM_BOB_FRAMES]; 3] = [
    bobbing_item!(46; 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20),
    bobbing_item!(77; 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20),
    bobbing_item!(139; 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20),
];
const ITEM_BOB_FRAMES: usize = 20;

/// Cards: the variant clip `p` (2139, sometimes placed without its name) shows frame `col`; every
/// tarot card shares frame 7's picture and every playing card frame 70's.
const CARD_VARIANT_CLIP: CharacterId = 2139;

macro_rules! card_overrides {
    ($phase:literal, $art:literal) => {
        RenderOverrides {
            children: &[("d", ChildState::Frame($phase))],
            texts: &[],
            hidden_characters: &[],
            character_frames: &[(CARD_VARIANT_CLIP, $art)],
            tint: None,
        }
    };
}

/// By phase (landing, idle, collected), then tarot or playing card.
static CARDS: [[RenderOverrides; 2]; 3] = [
    [card_overrides!(3, 7), card_overrides!(3, 70)],
    [card_overrides!(1, 7), card_overrides!(1, 70)],
    [card_overrides!(2, 7), card_overrides!(2, 70)],
];

/// By phase, then the pill's look.
static PILLS: [[RenderOverrides; 6]; 3] = [
    [
        card_overrides!(3, 1),
        card_overrides!(3, 2),
        card_overrides!(3, 3),
        card_overrides!(3, 4),
        card_overrides!(3, 5),
        card_overrides!(3, 6),
    ],
    [
        card_overrides!(1, 1),
        card_overrides!(1, 2),
        card_overrides!(1, 3),
        card_overrides!(1, 4),
        card_overrides!(1, 5),
        card_overrides!(1, 6),
    ],
    [
        card_overrides!(2, 1),
        card_overrides!(2, 2),
        card_overrides!(2, 3),
        card_overrides!(2, 4),
        card_overrides!(2, 5),
        card_overrides!(2, 6),
    ],
];

/// By phase, then the trinket's place in `Trinket::ALL`: the same clip frame as a card, its variant
/// clip on the trinket's own frame. Built once (the frames are not literals).
fn trinket_overrides(phase: PickupPhase, trinket: Trinket) -> &'static RenderOverrides {
    static TRINKETS: OnceLock<Vec<RenderOverrides>> = OnceLock::new();
    let overrides = TRINKETS.get_or_init(|| {
        let phase_frames: [u16; 3] = [3, 1, 2];
        phase_frames
            .into_iter()
            .flat_map(|phase_frame| {
                Trinket::ALL.into_iter().map(move |each| RenderOverrides {
                    children: Box::leak(Box::new([("d", ChildState::Frame(phase_frame))])),
                    texts: &[],
                    hidden_characters: &[],
                    character_frames: Box::leak(Box::new([(CARD_VARIANT_CLIP, u16::from(each.color_variant()))])),
                    tint: None,
                })
            })
            .collect()
    });
    &overrides[phase as usize * Trinket::ALL.len() + trinket.index()]
}

pub(super) fn source(kind: PickupKind, phase: PickupPhase, frame: u8) -> TextureSource {
    let animated = match kind {
        PickupKind::Trinket(trinket) => Some(trinket_overrides(phase, trinket)),
        PickupKind::Pill(pill) => Some(&PILLS[phase as usize][pill.index()]),
        PickupKind::Card(card) => Some(&CARDS[phase as usize][usize::from(!card.is_tarot())]),
        PickupKind::Chest(ChestKind::Red) => Some(&RED_CHEST[phase as usize]),
        _ => None,
    };
    if let Some(overrides) = animated {
        return TextureSource {
            selection: FrameSelection {
                frame_number: u16::from(kind.clip_frame()),
                frames_since_frame_started: u16::from(frame),
            },
            overrides,
            origin_y: -phase.art_lift().get(),
            ..framed(PICKUP_CLIP, u16::from(kind.clip_frame()))
        };
    }
    if let PickupKind::Item(item) = kind {
        let index = Item::ALL.iter().position(|&each| each == item).unwrap_or(0);
        return TextureSource {
            overrides: &ITEM_PEDESTALS[index][usize::from(frame) % ITEM_BOB_FRAMES],
            ..framed(PICKUP_CLIP, u16::from(kind.clip_frame()))
        };
    }
    if kind == PickupKind::ItemPedestal {
        return TextureSource {
            overrides: &EMPTY_PEDESTAL,
            ..framed(PICKUP_CLIP, u16::from(kind.clip_frame()))
        };
    }
    let by_variant = match phase {
        PickupPhase::Idle => &IDLE,
        PickupPhase::Collected => &COLLECTED,
        PickupPhase::Landing => &LANDING,
    };
    let variant_index = usize::from(kind.color_variant().clamp(1, VARIANT_COUNT) - 1);
    TextureSource {
        selection: FrameSelection {
            frame_number: u16::from(kind.clip_frame()),
            frames_since_frame_started: u16::from(frame),
        },
        overrides: &by_variant[variant_index],
        // Flash's y axis points down: the lifted center is above the clip's origin.
        origin_y: -phase.art_lift().get(),
        ..framed(PICKUP_CLIP, u16::from(kind.clip_frame()))
    }
}

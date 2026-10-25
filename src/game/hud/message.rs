//! The message at the bottom right when a pickup is special (`st22()` on `st2`, clip 6834):
//! it plays for 124 frames, right-aligned by its length (`st2._x = 620 - length * 10`).

use glam::Vec2;

use crate::game::hud::pill_text::PillText;
use crate::game::pickups::kind::{BombKind, CoinKind, PickupKind};
use crate::game::room::stage_to_world;

pub const MESSAGE_FRAME_COUNT: u8 = 124;
/// The box slides in from past the stage's edge on frames 3–4 and out on 122–123; frame 124
/// hides the clip.
pub const MESSAGE_VISIBLE_FRAMES: std::ops::Range<u8> = 4..MESSAGE_FRAME_COUNT - 3;
/// `st2._x = 620 - length * 10`: the clip's origin moves left by 10 px per character.
const RIGHT_EDGE: f32 = 620.0;
const WIDTH_PER_CHARACTER: f32 = 10.0;
/// `st2` on the main timeline: y 455.7.
const STAGE_Y: f32 = 455.7;
/// Flash shows nothing past the stage's right edge, so the box slides in from there; the texture
/// covers from a little left of the origin of the longest message to that edge, whatever the
/// message (its text comes from the game, see `PickupMessage`).
const STAGE_RIGHT_EDGE: f32 = 640.0;
const ART_LEFT_OF_ORIGIN: f32 = 40.0;
/// Longer than any message `st22()` shows ("Friends till the end!" has 21 characters).
const LONGEST_MESSAGE: f32 = 24.0;
const TEXTURE_LEFT: f32 = RIGHT_EDGE - LONGEST_MESSAGE * WIDTH_PER_CHARACTER - ART_LEFT_OF_ORIGIN;
const TEXTURE_HEIGHT: f32 = 84.0;
pub const MESSAGE_TEXTURE_SIZE: Vec2 = Vec2::new(STAGE_RIGHT_EDGE - TEXTURE_LEFT, TEXTURE_HEIGHT);

/// The messages pickups show, in the order the original's script has them (its first four
/// `st22()` texts, read from the game: "a Nickel", "a Dime", then the troll bomb's and the double
/// bomb's).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PickupMessage {
    Nickel,
    Dime,
    /// A troll bomb touched for the first time.
    TrollBomb,
    DoubleBomb,
    /// A pill's effect (see [`PillText::message_position`]).
    Pill(PillText),
}

impl PickupMessage {
    pub const ALL: [Self; 4] = [Self::Nickel, Self::Dime, Self::TrollBomb, Self::DoubleBomb];

    /// The messages pills show.
    #[cfg(test)]
    pub fn pill_messages() -> impl Iterator<Item = Self> {
        PillText::ALL
            .into_iter()
            .filter(|text| text.message_position().is_some())
            .map(Self::Pill)
    }

    /// Its place among the script's `st22()` texts, which `ALL` lists the first four of.
    pub fn script_position(self) -> usize {
        match self {
            Self::Nickel => 0,
            Self::Dime => 1,
            Self::TrollBomb => 2,
            Self::DoubleBomb => 3,
            Self::Pill(text) => text.message_position().unwrap_or_default(),
        }
    }

    /// What taking (or, for troll bombs, first touching) `kind` says.
    pub fn for_pickup(kind: PickupKind) -> Option<Self> {
        match kind {
            PickupKind::Coin(CoinKind::Nickel) => Some(Self::Nickel),
            PickupKind::Coin(CoinKind::Dime) => Some(Self::Dime),
            PickupKind::Bomb(BombKind::Troll | BombKind::MegaTroll) => Some(Self::TrollBomb),
            PickupKind::Bomb(BombKind::Double) => Some(Self::DoubleBomb),
            _ => None,
        }
    }

    /// Where the texture's center is drawn.
    pub fn position() -> Vec2 {
        stage_to_world(Vec2::new(TEXTURE_LEFT + MESSAGE_TEXTURE_SIZE.x / 2.0, STAGE_Y))
    }
}

/// The texture's center in the message clip's own coordinates, for a text of `characters`: the
/// clip stands at `620 - length * 10`.
pub fn texture_center_in_clip(characters: usize) -> f32 {
    TEXTURE_LEFT + MESSAGE_TEXTURE_SIZE.x / 2.0 - (RIGHT_EDGE - characters as f32 * WIDTH_PER_CHARACTER)
}

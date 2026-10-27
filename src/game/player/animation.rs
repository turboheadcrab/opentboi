//! Isaac's whole-body animations, drawn instead of his body and head: the teleport (his clip's
//! frame 7, `teller()`), arriving (frame 8), holding up a card he uses (frame 11), holding up what
//! he picked up or bought (frame 4, in `pillc()`, `balljunk()` and `powerlevel()`), and dying
//! (frame 6), and the thumbs up (frame 9) or down (frame 10) of `emo()`. The held picture is a frame of the item clip, by the number of what he holds
//! (DoAction 31588); each plays until its clip's last frame (`trgnextd()`).

use glam::Vec2;

use crate::game::draw::{DrawItem, Rgba, actor_depth};
use crate::game::original_units::Frames;
use crate::game::pickups::card::Card;
use crate::game::pickups::consumable::Consumable;
use crate::game::pickups::item::Item;
use crate::game::pickups::pill::{PillLook, PillLooks};
use crate::game::pickups::trinket::Trinket;
use crate::game::texture_id::TextureId;

/// The teleport clip (5713) has 19 frames; holding up an item (5816) 20.
pub const TELEPORT_FRAME_COUNT: u8 = 19;
pub const HOLDING_UP_FRAME_COUNT: u8 = 20;
/// The arrival clip (5714) has 9.
pub const TELEPORT_ARRIVAL_FRAME_COUNT: u8 = 9;
/// The pickup animation (`player.d` frame 4: 5652, then 5639 on the skin's frame) has 42.
pub const PICKING_UP_FRAME_COUNT: u8 = 42;
/// Dying (`player.d` frame 6: 5710, then 5683 on the skin's frame) has 78, stopping on the last.
pub const DYING_FRAME_COUNT: u8 = 78;
/// The thumbs up (`player.d` frame 9: 5786, then 5726 on the skin's frame) has 24, the thumbs down
/// (frame 10: 5815, then 5787) 27.
pub const THUMBS_UP_FRAME_COUNT: u8 = 24;
pub const THUMBS_DOWN_FRAME_COUNT: u8 = 27;
const FALLBACK_COLOR: Rgba = Rgba::opaque(0.96, 0.82, 0.76);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PlayerAnimation {
    Teleport,
    /// Arriving after a teleport (his clip's frame 8).
    TeleportArrival,
    /// Holding up a card or item he uses.
    HoldingUp(Held),
    PickingUp(Held),
    Dying,
    /// Isaac's reaction to a good pill (`emo()`).
    ThumbsUp,
    /// His reaction to a bad one.
    ThumbsDown,
}

/// What Isaac holds up after picking it up (`player.it`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Held {
    /// A card: item picture 43, showing the card's face.
    Card(Card),
    /// A pill: the same picture on its look's frame.
    Pill(PillLook),
    /// A trinket: the same picture on its own frame.
    Trinket(Trinket),
    /// Bought from the shop: item pictures 200 a heart, 201 a bomb, 202 a key, 203 a soul heart.
    Heart,
    Bomb,
    Key,
    SoulHeart,
    /// An item from a pedestal (`player.it` its number).
    Item(Item),
}

impl Card {
    /// What Isaac holds up using the card: the card, but the unicorn for the Chariot (its ride).
    pub fn held_picture(self) -> Held {
        match self {
            Self::TheChariot => Held::Item(Item::MyLittleUnicorn),
            card => Held::Card(card),
        }
    }
}

impl Held {
    /// What Isaac holds up for `consumable`, a pill in the look this run gives it.
    pub fn holding(consumable: Consumable, looks: PillLooks) -> Self {
        match consumable {
            Consumable::Card(card) => Self::Card(card),
            Consumable::Pill(pill) => Self::Pill(looks.look_of(pill)),
        }
    }
}

impl Held {
    pub const WARES: [Self; 4] = [Self::Heart, Self::Bomb, Self::Key, Self::SoulHeart];
}

impl PlayerAnimation {
    pub fn frame_count(self) -> u8 {
        match self {
            Self::Teleport => TELEPORT_FRAME_COUNT,
            Self::TeleportArrival => TELEPORT_ARRIVAL_FRAME_COUNT,
            Self::HoldingUp(_) => HOLDING_UP_FRAME_COUNT,
            Self::PickingUp(_) => PICKING_UP_FRAME_COUNT,
            Self::Dying => DYING_FRAME_COUNT,
            Self::ThumbsUp => THUMBS_UP_FRAME_COUNT,
            Self::ThumbsDown => THUMBS_DOWN_FRAME_COUNT,
        }
    }
}

/// An animation playing, and for how long.
#[derive(Clone, Copy, Debug)]
pub struct PlayingAnimation {
    pub animation: PlayerAnimation,
    pub age: Frames,
}

impl PlayingAnimation {
    pub fn start(animation: PlayerAnimation) -> Self {
        Self {
            animation,
            age: Frames::ZERO,
        }
    }

    /// `false` once it has played (the teleport holds its last frame until Isaac arrives, dying
    /// for good).
    pub fn advance(&mut self, elapsed: Frames) -> bool {
        self.age += elapsed;
        let length = Frames::from(self.animation.frame_count());
        matches!(self.animation, PlayerAnimation::Teleport | PlayerAnimation::Dying) || self.age < length
    }

    pub fn draw_item(self, floor_point: Vec2, scale: f32) -> DrawItem {
        let texture = TextureId::IsaacAnimation {
            animation: self.animation,
            frame: self.age.held_frame(self.animation.frame_count()),
        };
        DrawItem::textured(texture, floor_point, scale, FALLBACK_COLOR, actor_depth(floor_point.y))
    }
}

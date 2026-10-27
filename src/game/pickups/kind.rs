//! Kinds of pickup (object type 5, clip `b5`): the clip frame (`d`) picks hearts, coins, keys or
//! bombs, and `col` the variant shown by the nested `p` clips. Variants are rolled as in
//! `create()` (with no luck or trinkets).

use crate::game::pickups::card::Card;
use crate::game::pickups::consumable::Consumable;
use crate::game::pickups::item::Item;
use crate::game::pickups::pill::Pill;
use crate::game::pickups::trinket::Trinket;
use crate::game::random::Random;

mod rolls;

use rolls::{GOLDEN_KEY_ODDS, roll_bomb, roll_coin, roll_heart};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum HeartKind {
    Full,
    Half,
    Soul,
    Eternal,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CoinKind {
    Penny,
    Nickel,
    Dime,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum KeyKind {
    Normal,
    /// `kep`: keys never run out for the rest of the run.
    Golden,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BombKind {
    Single,
    /// "1 + 1 free".
    Double,
    /// Cannot be picked up ("Trolololol") and explodes on its own.
    Troll,
    MegaTroll,
}

/// Chests stay where they are once opened; their contents spill out.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ChestKind {
    Brown,
    /// Locked: opening it takes a key.
    Golden,
    /// A brown chest turned red (`c2`, `col = 2`): it holds devil things (see `red_chest`).
    Red,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PickupKind {
    Heart(HeartKind),
    Coin(CoinKind),
    Key(KeyKind),
    Bomb(BombKind),
    Chest(ChestKind),
    /// An empty item pedestal (frame 10): what is left once its item is taken, and what the
    /// original's random items become for now.
    ItemPedestal,
    /// An item pedestal holding `Item`.
    Item(Item),
    Card(Card),
    /// A pill (frame 7 with `col` below 7).
    Pill(Pill),
    /// A trinket (frame 7 with `col` 29–62).
    Trinket(Trinket),
}

/// `b5` frames.
pub(crate) const HEART_FRAME: u8 = 1;
pub(crate) const COIN_FRAME: u8 = 2;
pub(crate) const KEY_FRAME: u8 = 3;
pub(crate) const BOMB_FRAME: u8 = 4;
pub(crate) const BROWN_CHEST_FRAME: u8 = 5;
pub(crate) const GOLDEN_CHEST_FRAME: u8 = 6;
pub(crate) const ITEM_PEDESTAL_FRAME: u8 = 10;
/// Pills and cards share frame 7: a marker for it becomes a pill, and one for frame 30 a card
/// (`tater()`).
const CARD_FRAME: u8 = 7;
pub(crate) const PILL_FRAME: u8 = CARD_FRAME;
pub(crate) const CARD_MARKER_FRAME: u8 = 30;
/// A marker for frame 35 becomes a trinket.
pub(crate) const TRINKET_MARKER_FRAME: u8 = 35;
/// A marker for frame 36 becomes a brown chest that is always red.
const RED_CHEST_MARKER_FRAME: u8 = 36;
impl PickupKind {
    pub const ALL: [Self; 17] = [
        Self::Heart(HeartKind::Full),
        Self::Heart(HeartKind::Half),
        Self::Heart(HeartKind::Soul),
        Self::Heart(HeartKind::Eternal),
        Self::Coin(CoinKind::Penny),
        Self::Coin(CoinKind::Nickel),
        Self::Coin(CoinKind::Dime),
        Self::Key(KeyKind::Normal),
        Self::Key(KeyKind::Golden),
        Self::Bomb(BombKind::Single),
        Self::Bomb(BombKind::Double),
        Self::Bomb(BombKind::Troll),
        Self::Bomb(BombKind::MegaTroll),
        Self::Chest(ChestKind::Brown),
        Self::Chest(ChestKind::Golden),
        Self::Chest(ChestKind::Red),
        Self::ItemPedestal,
    ];

    /// The pickup a marker for `b5` frame `clip_frame` becomes, or `None` for kinds not supported
    /// yet (pills, cards, machines, ...).
    pub fn roll(clip_frame: u8, random: &mut Random) -> Option<Self> {
        let kind = match clip_frame {
            HEART_FRAME => Self::Heart(roll_heart(random)),
            COIN_FRAME => Self::Coin(roll_coin(random)),
            KEY_FRAME => Self::Key(if random.happens(GOLDEN_KEY_ODDS) {
                KeyKind::Golden
            } else {
                KeyKind::Normal
            }),
            BOMB_FRAME => Self::Bomb(roll_bomb(random)),
            BROWN_CHEST_FRAME => Self::Chest(ChestKind::Brown),
            GOLDEN_CHEST_FRAME => Self::Chest(ChestKind::Golden),
            RED_CHEST_MARKER_FRAME => Self::Chest(ChestKind::Red),
            ITEM_PEDESTAL_FRAME => Self::ItemPedestal,
            CARD_FRAME => Self::Pill(Pill::roll(random)),
            CARD_MARKER_FRAME => Self::Card(Card::roll(random)),
            // `junx()`: a trinket not given out yet, else the card `tater()` rolls.
            TRINKET_MARKER_FRAME => match random.trinket(false) {
                Some(trinket) => Self::Trinket(trinket),
                None => Self::Card(Card::roll(random)),
            },
            _ => return None,
        };
        Some(kind)
    }

    /// The `b5` frame showing this kind.
    pub fn clip_frame(self) -> u8 {
        match self {
            Self::Heart(_) => HEART_FRAME,
            Self::Coin(_) => COIN_FRAME,
            Self::Key(_) => KEY_FRAME,
            Self::Bomb(_) => BOMB_FRAME,
            Self::Chest(ChestKind::Brown | ChestKind::Red) => BROWN_CHEST_FRAME,
            Self::Chest(ChestKind::Golden) => GOLDEN_CHEST_FRAME,
            Self::ItemPedestal | Self::Item(_) => ITEM_PEDESTAL_FRAME,
            Self::Card(_) | Self::Pill(_) | Self::Trinket(_) => CARD_FRAME,
        }
    }

    /// `col`, the frame of the nested `p` clips.
    pub fn color_variant(self) -> u8 {
        match self {
            Self::Heart(HeartKind::Full)
            | Self::Coin(CoinKind::Penny)
            | Self::Key(KeyKind::Normal)
            | Self::Bomb(BombKind::Single)
            | Self::Chest(ChestKind::Brown | ChestKind::Golden)
            | Self::ItemPedestal
            | Self::Item(_) => 1,
            Self::Heart(HeartKind::Half)
            | Self::Coin(CoinKind::Nickel)
            | Self::Key(KeyKind::Golden)
            | Self::Bomb(BombKind::Double)
            | Self::Chest(ChestKind::Red) => 2,
            Self::Heart(HeartKind::Soul) | Self::Coin(CoinKind::Dime) | Self::Bomb(BombKind::Troll) => 3,
            Self::Heart(HeartKind::Eternal) => 4,
            Self::Bomb(BombKind::MegaTroll) => 5,
            Self::Card(card) => card.color_variant(),
            Self::Pill(pill) => pill.color_variant(),
            Self::Trinket(trinket) => trinket.color_variant(),
        }
    }

    /// The kind whose picture this kind shows (cards share theirs).
    pub fn art(self) -> Self {
        match self {
            Self::Card(card) => Self::Card(card.art()),
            other => other,
        }
    }

    /// The card or pill this is, for the slot that holds one.
    pub fn consumable(self) -> Option<Consumable> {
        match self {
            Self::Card(card) => Some(Consumable::Card(card)),
            Self::Pill(pill) => Some(Consumable::Pill(pill)),
            _ => None,
        }
    }

    /// Troll bombs lie there until they go off.
    pub fn is_troll_bomb(self) -> bool {
        matches!(self, Self::Bomb(BombKind::Troll | BombKind::MegaTroll))
    }

    /// An item pedestal, with or without its item.
    pub fn is_pedestal(self) -> bool {
        matches!(self, Self::ItemPedestal | Self::Item(_))
    }

    pub fn is_chest(self) -> bool {
        matches!(self, Self::Chest(_))
    }

    /// Frames of the landing animation (`d` frame 3): 35 for small pickups, 22 and 19 for chests.
    pub fn landing_frame_count(self) -> u8 {
        match self {
            Self::Chest(ChestKind::Brown | ChestKind::Red) => 22,
            Self::Chest(ChestKind::Golden) => 19,
            // It stands there from the start.
            Self::ItemPedestal | Self::Item(_) => 0,
            _ => 35,
        }
    }

    /// Frames of the idle loop (`d` frame 1): hearts beat, coins shine, the rest is still.
    pub fn idle_frame_count(self) -> u8 {
        match self {
            Self::Heart(_) => 33,
            Self::Coin(_) => 30,
            // The item on a pedestal bobs.
            Self::Item(_) => 20,
            Self::Key(_)
            | Self::Bomb(_)
            | Self::Chest(_)
            | Self::ItemPedestal
            | Self::Card(_)
            | Self::Pill(_)
            | Self::Trinket(_) => 1,
        }
    }

    /// Frames of the pickup animation (`d` frame 2); a chest's opening stops on its last frame.
    pub fn collected_frame_count(self) -> u8 {
        match self {
            Self::Heart(_) | Self::Bomb(_) | Self::Card(_) | Self::Pill(_) | Self::Trinket(_) => 16,
            Self::Coin(_) | Self::Key(_) => 15,
            Self::Chest(_) => 20,
            Self::ItemPedestal | Self::Item(_) => 0,
        }
    }
}

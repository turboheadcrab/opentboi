//! What the floor-name banner (`st11()`) can name: a card used or taken, an item taken, or a fortune
//! the fortune machine tells.

use crate::game::hud::pill_text::PillText;
use crate::game::pickups::card::Card;
use crate::game::pickups::item::Item;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BannerName {
    Card(Card),
    Item(Item),
    /// One of the pill banners (`st11()`): see [`PillText::banner_position`].
    Pill(PillText),
    /// By place in the machine's list of fortunes, below [`FORTUNE_COUNT`].
    Fortune(u8),
}

/// How many fortunes the machine's list holds.
pub const FORTUNE_COUNT: u8 = 16;

impl BannerName {
    /// Every name the banner can show.
    #[cfg(test)]
    pub fn all() -> impl Iterator<Item = Self> {
        Card::ALL
            .map(Self::Card)
            .into_iter()
            .chain(Item::ALL.map(Self::Item))
            .chain((0..FORTUNE_COUNT).map(Self::Fortune))
            .chain(
                PillText::ALL
                    .into_iter()
                    .filter(|text| text.banner_position().is_some())
                    .map(Self::Pill),
            )
    }
}

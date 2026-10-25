//! What Isaac can hold in the card slot: a card or a pill (`pilc`).

use crate::game::pickups::card::Card;
use crate::game::pickups::kind::PickupKind;
use crate::game::pickups::pill::Pill;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Consumable {
    Card(Card),
    Pill(Pill),
}

impl Consumable {
    /// The one with this `col`: 1–6 a pill, 7 on a tarot card, 70 on a playing card.
    pub fn from_color(color: u8) -> Option<Self> {
        Pill::ALL
            .into_iter()
            .find(|pill| pill.color_variant() == color)
            .map(Self::Pill)
            .or_else(|| {
                Card::ALL
                    .into_iter()
                    .find(|card| card.color_variant() == color)
                    .map(Self::Card)
            })
    }

    /// `col`, as the pickup and the shop's picture show it.
    pub fn color_variant(self) -> u8 {
        match self {
            Self::Card(card) => card.color_variant(),
            Self::Pill(pill) => pill.color_variant(),
        }
    }

    /// The pickup that lies on the floor when it is dropped.
    pub fn pickup_kind(self) -> PickupKind {
        match self {
            Self::Card(card) => PickupKind::Card(card),
            Self::Pill(pill) => PickupKind::Pill(pill),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_color_names_the_pill_or_card_that_has_it() {
        assert_eq!(Consumable::from_color(3), Some(Consumable::Pill(Pill::ALL[2])));
        assert_eq!(Consumable::from_color(7), Some(Consumable::Card(Card::WheelOfFortune)));
        assert_eq!(Consumable::from_color(70), Some(Consumable::Card(Card::TwoOfSpades)));
        assert_eq!(Consumable::from_color(40), None);
        for consumable in Pill::ALL
            .map(Consumable::Pill)
            .into_iter()
            .chain(Card::ALL.map(Consumable::Card))
        {
            assert_eq!(Consumable::from_color(consumable.color_variant()), Some(consumable));
        }
    }
}

//! Cards (pickup frame 7): the 22 tarot cards (`col` 7–28) and the playing cards (`col` 70–74),
//! in `tart()`'s order; their names are read from the game (see `game_assets::script`). Isaac
//! holds one card or pill at a time (`pilc`).

use crate::game::odds::Odds;
use crate::game::random::Random;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Card {
    WheelOfFortune,
    TheDevil,
    Death,
    Strength,
    TheFool,
    TheMagician,
    TheHermit,
    TheMoon,
    TheHangedMan,
    Judgement,
    TheChariot,
    TheLovers,
    TheWorld,
    Justice,
    Temperance,
    TheEmperor,
    TheTower,
    TheHierophant,
    TheHighPriestess,
    TheEmpress,
    TheSun,
    TheStars,
    TwoOfSpades,
    TwoOfClubs,
    TwoOfHearts,
    TwoOfDiamonds,
    TheJoker,
}

const FIRST_TAROT_COLOR: u8 = 7;
const FIRST_PLAYING_CARD_COLOR: u8 = 70;
pub const TAROT_COUNT: u8 = 22;
/// `tater()` (DoAction 7045–7052): a tarot card four times in five (DoAction 7047, DoAction 7049), else
/// a playing card (DoAction 7051), each of its group equally.
const PLAYING_CARD_ODDS: Odds = Odds::one_in(5);

impl Card {
    pub const ALL: [Self; 27] = [
        Self::WheelOfFortune,
        Self::TheDevil,
        Self::Death,
        Self::Strength,
        Self::TheFool,
        Self::TheMagician,
        Self::TheHermit,
        Self::TheMoon,
        Self::TheHangedMan,
        Self::Judgement,
        Self::TheChariot,
        Self::TheLovers,
        Self::TheWorld,
        Self::Justice,
        Self::Temperance,
        Self::TheEmperor,
        Self::TheTower,
        Self::TheHierophant,
        Self::TheHighPriestess,
        Self::TheEmpress,
        Self::TheSun,
        Self::TheStars,
        Self::TwoOfSpades,
        Self::TwoOfClubs,
        Self::TwoOfHearts,
        Self::TwoOfDiamonds,
        Self::TheJoker,
    ];

    fn index(self) -> u8 {
        (0..)
            .zip(Self::ALL)
            .find(|&(_, card)| card == self)
            .map_or(0, |(index, _)| index)
    }

    /// `col`: also the frame of the pickup's and the HUD's `p` clip.
    pub fn color_variant(self) -> u8 {
        let index = self.index();
        if index < TAROT_COUNT {
            FIRST_TAROT_COLOR + index
        } else {
            FIRST_PLAYING_CARD_COLOR + (index - TAROT_COUNT)
        }
    }

    pub fn is_tarot(self) -> bool {
        self.index() < TAROT_COUNT
    }

    /// All tarot cards share one picture, and so do all playing cards: the card drawn for both.
    pub fn art(self) -> Self {
        if self.is_tarot() {
            Self::WheelOfFortune
        } else {
            Self::TwoOfSpades
        }
    }

    /// `tater()`.
    pub fn roll(random: &mut Random) -> Self {
        if random.happens(PLAYING_CARD_ODDS) {
            Self::ALL[usize::from(TAROT_COUNT) + random.index(Self::ALL.len() - usize::from(TAROT_COUNT))]
        } else {
            Self::ALL[random.index(usize::from(TAROT_COUNT))]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cards_have_the_original_colors() {
        assert_eq!(Card::WheelOfFortune.color_variant(), 7);
        assert_eq!(Card::TheStars.color_variant(), 28);
        assert_eq!(Card::TwoOfSpades.color_variant(), 70);
        assert_eq!(Card::TheJoker.color_variant(), 74);
    }
}

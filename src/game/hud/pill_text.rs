//! The words pills put on screen: the name of the pill held (`hud.pilll`), the banner (`st11()`)
//! and the message (`st22()`) shown when one is taken or used. The game's own script holds the
//! texts; each is found by its place there, which this lists for every text.

use crate::game::player::pill_state::{PillEffect, PillKind};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PillText {
    /// The name of a pill whose effect is not known yet.
    Unknown,
    /// The banner on taking one, and the two the pill that is `Puberty` shows.
    PillsHere,
    Puberty,
    IFoundPills,
    AndAteThem,
    /// What using the diarrhea pill says; its name in the HUD is [`Self::Diarrhea`].
    OhNo,
    Diarrhea,
    LuckUp,
    LuckDown,
    Friends,
    BadTrip,
    FullHealth,
    BallsOfSteel,
    PrettyFly,
    HealthUp,
    HealthDown,
    BombsAreKeys,
    TearsUp,
    TearsDown,
    RangeUp,
    RangeDown,
    SpeedUp,
    SpeedDown,
    ShotSpeedUp,
    ShotSpeedDown,
    BadGas,
    TelePills,
}

/// How many names the held pill's strip can show in the script: the unknown one and each effect's
/// (one of them for the hard mode's full health, which does not exist here).
pub const HUD_NAME_COUNT: usize = 24;
/// The pill messages come after this many `st22()` texts of other things.
pub const MESSAGES_BEFORE_PILLS: usize = 5;

impl PillText {
    pub const ALL: [Self; 27] = [
        Self::Unknown,
        Self::PillsHere,
        Self::Puberty,
        Self::IFoundPills,
        Self::AndAteThem,
        Self::OhNo,
        Self::Diarrhea,
        Self::LuckUp,
        Self::LuckDown,
        Self::Friends,
        Self::BadTrip,
        Self::FullHealth,
        Self::BallsOfSteel,
        Self::PrettyFly,
        Self::HealthUp,
        Self::HealthDown,
        Self::BombsAreKeys,
        Self::TearsUp,
        Self::TearsDown,
        Self::RangeUp,
        Self::RangeDown,
        Self::SpeedUp,
        Self::SpeedDown,
        Self::ShotSpeedUp,
        Self::ShotSpeedDown,
        Self::BadGas,
        Self::TelePills,
    ];

    /// The name of the pill with `effect` as the HUD shows it.
    pub fn name_of(effect: PillEffect, helps: bool) -> Self {
        let pair = |good, bad| if helps { good } else { bad };
        match effect.kind {
            PillKind::FullHealth => Self::FullHealth,
            PillKind::Puberty => Self::IFoundPills,
            PillKind::Diarrhea => Self::Diarrhea,
            PillKind::BallsOfSteel => Self::BallsOfSteel,
            PillKind::PrettyFly => Self::PrettyFly,
            PillKind::BadTrip => Self::BadTrip,
            PillKind::Health => pair(Self::HealthUp, Self::HealthDown),
            PillKind::BombsAreKeys => Self::BombsAreKeys,
            PillKind::Tears => pair(Self::TearsUp, Self::TearsDown),
            PillKind::Range => pair(Self::RangeUp, Self::RangeDown),
            PillKind::Speed => pair(Self::SpeedUp, Self::SpeedDown),
            PillKind::ShotSpeed => pair(Self::ShotSpeedUp, Self::ShotSpeedDown),
            PillKind::BadGas => Self::BadGas,
            PillKind::Friends => Self::Friends,
            PillKind::Luck => pair(Self::LuckUp, Self::LuckDown),
            PillKind::TelePills => Self::TelePills,
        }
    }

    /// Its place among the names of the held pill, if it is one of them.
    pub fn name_position(self) -> Option<usize> {
        const NAMES: [(PillText, usize); 22] = [
            (PillText::Unknown, 0),
            (PillText::LuckUp, 1),
            (PillText::LuckDown, 2),
            (PillText::Friends, 3),
            (PillText::BadTrip, 4),
            (PillText::FullHealth, 6),
            (PillText::IFoundPills, 7),
            (PillText::Diarrhea, 8),
            (PillText::BallsOfSteel, 9),
            (PillText::PrettyFly, 10),
            (PillText::HealthUp, 11),
            (PillText::HealthDown, 12),
            (PillText::BombsAreKeys, 13),
            (PillText::TearsUp, 14),
            (PillText::TearsDown, 15),
            (PillText::RangeUp, 16),
            (PillText::RangeDown, 17),
            (PillText::SpeedUp, 18),
            (PillText::SpeedDown, 19),
            (PillText::ShotSpeedUp, 20),
            (PillText::ShotSpeedDown, 21),
            (PillText::BadGas, 22),
        ];
        match self {
            Self::TelePills => Some(23),
            other => NAMES
                .iter()
                .find(|(text, _)| *text == other)
                .map(|&(_, position)| position),
        }
    }

    /// Its place among the pill messages of `st22()` (after [`MESSAGES_BEFORE_PILLS`] others), if
    /// it is one.
    pub fn message_position(self) -> Option<usize> {
        const MESSAGES: [(PillText, usize); 22] = [
            (PillText::LuckUp, 0),
            (PillText::LuckDown, 1),
            (PillText::Friends, 2),
            (PillText::BadTrip, 3),
            (PillText::FullHealth, 5),
            (PillText::AndAteThem, 6),
            (PillText::OhNo, 7),
            (PillText::BallsOfSteel, 8),
            (PillText::PrettyFly, 9),
            (PillText::HealthUp, 10),
            (PillText::HealthDown, 11),
            (PillText::BombsAreKeys, 12),
            (PillText::TearsUp, 13),
            (PillText::TearsDown, 14),
            (PillText::RangeUp, 15),
            (PillText::RangeDown, 16),
            (PillText::SpeedUp, 17),
            (PillText::SpeedDown, 18),
            (PillText::ShotSpeedUp, 19),
            (PillText::ShotSpeedDown, 20),
            (PillText::BadGas, 21),
            (PillText::TelePills, 22),
        ];
        MESSAGES
            .iter()
            .find(|(text, _)| *text == self)
            .map(|&(_, position)| MESSAGES_BEFORE_PILLS + position)
    }

    /// Its place among the fixed texts of `st11()`, if it is one.
    pub fn banner_position(self) -> Option<usize> {
        match self {
            Self::PillsHere => Some(0),
            Self::Puberty => Some(1),
            Self::IFoundPills => Some(2),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_text_has_a_place_to_be_read_from() {
        for text in PillText::ALL {
            assert!(
                text.name_position().is_some() || text.message_position().is_some() || text.banner_position().is_some(),
                "{text:?}"
            );
        }
    }

    #[test]
    fn places_are_not_shared() {
        for (index, text) in PillText::ALL.iter().enumerate() {
            for other in &PillText::ALL[index + 1..] {
                assert!(text.name_position().is_none() || text.name_position() != other.name_position());
                assert!(text.message_position().is_none() || text.message_position() != other.message_position());
            }
        }
        assert!(
            PillText::TelePills
                .name_position()
                .is_some_and(|position| position < HUD_NAME_COUNT)
        );
    }
}

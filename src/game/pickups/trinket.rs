//! Trinkets (pickup frame 7 with `col` 29–62; `pillc()` takes any `col` above 28 and below 69 for
//! one, DoAction 7073, but the game has none past 62): small passive charms, held in a slot of their
//! own beside the card or pill (`trix`). Their names are read from the game (see
//! `game_assets::script`), as `pillc()` does (DoAction 7079, DoAction 7080).
//! A run hands out each trinket at most once, from a pool that starts with the ones every player
//! has (`junx()`, DoAction 306–344).

use crate::game::random::Random;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Trinket {
    FishHead,
    PinkyEye,
    PushPin,
    LibertyCap,
    UmbilicalCord,
    ChildsHeart,
    CurvedHorn,
    RustedKey,
    GoatHoof,
    MomsPearl,
    Cancer,
    RedPatch,
    MatchStick,
    LuckyToe,
    CursedSkull,
    SafetyCap,
    AceOfSpades,
    IsaacsFork,
    ThePolaroid,
    AMissingPage,
    BloodyPenny,
    BurntPenny,
    FlatPenny,
    CounterfeitPenny,
    Tick,
    IsaacsHead,
    MaggysFaith,
    JudasTongue,
    BlueBabysSoul,
    SamsonsLock,
    CainsEye,
    EvesBirdFoot,
    TheLeftHand,
    GameSquid,
}

/// `col` of the first trinket.
const FIRST_COLOR: u8 = 29;
/// The trinkets in `junxx` from the start (`col` values): the rest wait for an unlock.
const STARTING_POOL: [u8; 18] = [30, 31, 32, 33, 34, 36, 37, 38, 39, 40, 41, 43, 44, 45, 46, 48, 51, 53];
/// What a gold-tinted pickup (a golden chest's or the tinted rock's) always gives.
const GOLD_PICKUP_COLOR: u8 = 52;

impl Trinket {
    pub const ALL: [Self; 34] = [
        Self::FishHead,
        Self::PinkyEye,
        Self::PushPin,
        Self::LibertyCap,
        Self::UmbilicalCord,
        Self::ChildsHeart,
        Self::CurvedHorn,
        Self::RustedKey,
        Self::GoatHoof,
        Self::MomsPearl,
        Self::Cancer,
        Self::RedPatch,
        Self::MatchStick,
        Self::LuckyToe,
        Self::CursedSkull,
        Self::SafetyCap,
        Self::AceOfSpades,
        Self::IsaacsFork,
        Self::ThePolaroid,
        Self::AMissingPage,
        Self::BloodyPenny,
        Self::BurntPenny,
        Self::FlatPenny,
        Self::CounterfeitPenny,
        Self::Tick,
        Self::IsaacsHead,
        Self::MaggysFaith,
        Self::JudasTongue,
        Self::BlueBabysSoul,
        Self::SamsonsLock,
        Self::CainsEye,
        Self::EvesBirdFoot,
        Self::TheLeftHand,
        Self::GameSquid,
    ];

    /// Its place among [`Self::ALL`].
    pub fn index(self) -> usize {
        Self::ALL.iter().position(|&each| each == self).unwrap_or(0)
    }

    /// `col`: also the frame of the pickup's and the HUD's `p` clip.
    pub fn color_variant(self) -> u8 {
        FIRST_COLOR + u8::try_from(self.index()).unwrap_or(0)
    }

    /// The trinket with this `col`, if there is one.
    pub fn from_color(color: u8) -> Option<Self> {
        Self::ALL.into_iter().find(|trinket| trinket.color_variant() == color)
    }
}

/// The trinkets still to be found this run (`junxx`): each is handed out once.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TrinketPool {
    remaining: Vec<Trinket>,
}

impl Default for TrinketPool {
    fn default() -> Self {
        Self {
            remaining: STARTING_POOL.into_iter().filter_map(Trinket::from_color).collect(),
        }
    }
}

impl TrinketPool {
    /// A pool with nothing in it.
    pub fn empty() -> Self {
        Self { remaining: Vec::new() }
    }

    /// `junx()`: a trinket not found yet, or `None` once the pool is empty (the original then
    /// rolls a card instead). A pickup that shines gold is always the Counterfeit Penny.
    pub fn take(&mut self, random: &mut Random, golden: bool) -> Option<Trinket> {
        if self.remaining.is_empty() {
            return None;
        }
        if golden {
            return Trinket::from_color(GOLD_PICKUP_COLOR);
        }
        let index = random.index(self.remaining.len());
        Some(self.remaining.remove(index))
    }

    #[cfg(test)]
    pub fn len(&self) -> usize {
        self.remaining.len()
    }

    #[cfg(test)]
    pub fn is_empty(&self) -> bool {
        self.remaining.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trinkets_have_the_originals_colors_29_to_62() {
        assert_eq!(Trinket::FishHead.color_variant(), 29);
        assert_eq!(Trinket::Tick.color_variant(), 53);
        assert_eq!(Trinket::GameSquid.color_variant(), 62);
        for trinket in Trinket::ALL {
            assert_eq!(Trinket::from_color(trinket.color_variant()), Some(trinket));
        }
        assert_eq!(Trinket::from_color(28), None);
        assert_eq!(Trinket::from_color(63), None);
    }

    #[test]
    fn a_run_starts_with_eighteen_trinkets_and_hands_each_out_once() {
        let mut pool = TrinketPool::default();
        assert_eq!(pool.len(), STARTING_POOL.len());
        let mut random = Random::default();
        let mut seen = Vec::new();
        while let Some(trinket) = pool.take(&mut random, false) {
            assert!(!seen.contains(&trinket), "{trinket:?} came twice");
            seen.push(trinket);
        }
        assert_eq!(seen.len(), STARTING_POOL.len());
        assert!(pool.is_empty());
    }

    #[test]
    fn a_gold_pickup_is_the_counterfeit_penny() {
        let mut pool = TrinketPool::default();
        assert_eq!(pool.take(&mut Random::default(), true), Some(Trinket::CounterfeitPenny));
    }
}

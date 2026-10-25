//! Pills (pickup frame 7 with `col` 1–6): six looks, each of which has an effect assigned at
//! random the first time it is used (see `player::pill_effects`). A pill is held in the same slot
//! as a card (`pilc`, below 7).

use crate::game::random::Random;

/// One of the six looks, by `col`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Pill(u8);

impl Pill {
    pub const COUNT: u8 = 6;
    pub const ALL: [Self; 6] = [Self(1), Self(2), Self(3), Self(4), Self(5), Self(6)];

    /// A pill marker for frame 7 becomes `col = random(6) + 1`.
    pub fn roll(random: &mut Random) -> Self {
        Self(1 + random.below_u8(Self::COUNT))
    }

    /// `col`: also the frame of the pickup's and the HUD's `p` clip.
    pub fn color_variant(self) -> u8 {
        self.0
    }

    /// Its place among [`Self::ALL`].
    pub fn index(self) -> usize {
        usize::from(self.0 - 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pills_have_the_originals_colors_one_to_six() {
        let colors: Vec<u8> = Pill::ALL.iter().map(|pill| pill.color_variant()).collect();
        assert_eq!(colors, [1, 2, 3, 4, 5, 6]);
        assert_eq!(Pill::ALL[3].index(), 3);
    }

    #[test]
    fn rolled_pills_cover_all_six_looks() {
        let mut random = Random::default();
        let mut seen = [false; 6];
        for _ in 0..200 {
            seen[Pill::roll(&mut random).index()] = true;
        }
        assert!(seen.iter().all(|&each| each));
    }
}

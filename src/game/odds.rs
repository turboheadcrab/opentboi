//! "One in N" chances, as the original rolls them (`random(N) == 0`).

/// A chance of one in this many.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Odds(u32);

impl Odds {
    /// A coin flip.
    pub const EVEN: Self = Self(2);

    pub const fn one_in(chances: u32) -> Self {
        Self(chances)
    }

    /// How many chances one of them wins.
    pub const fn chances(self) -> u32 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::random::Random;

    #[test]
    fn one_in_n_comes_up_about_one_time_in_n() {
        let mut random = Random::with_seed(7);
        let hits = (0..10_000).filter(|_| random.happens(Odds::one_in(5))).count();
        assert!((1_700..2_300).contains(&hits), "{hits}");
        assert!((0..100).all(|_| random.happens(Odds::one_in(1))));
    }
}

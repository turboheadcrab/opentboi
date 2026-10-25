//! Which recording of a sound event to play: the original picks one at random each time, by
//! building the file name from a random number.

/// A small xorshift generator: sound choices need no particular sequence, only variety.
pub struct VariantChooser {
    state: u64,
}

/// Any odd, non-zero start; mixed with the clock so runs differ.
const SEED_MIX: u64 = 0x9E37_79B9_7F4A_7C15;
const SHIFT_LEFT_FIRST: u32 = 13;
const SHIFT_RIGHT: u32 = 7;
const SHIFT_LEFT_SECOND: u32 = 17;

impl Default for VariantChooser {
    fn default() -> Self {
        let nanoseconds = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |elapsed| elapsed.subsec_nanos());
        Self {
            state: SEED_MIX ^ u64::from(nanoseconds) | 1,
        }
    }
}

impl VariantChooser {
    /// An index below `variant_count` (0 when there is at most one).
    pub fn pick(&mut self, variant_count: usize) -> usize {
        self.state ^= self.state << SHIFT_LEFT_FIRST;
        self.state ^= self.state >> SHIFT_RIGHT;
        self.state ^= self.state << SHIFT_LEFT_SECOND;
        let variants = u64::try_from(variant_count.max(1)).unwrap_or(u64::MAX);
        usize::try_from(self.state % variants).unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn picks_stay_in_range_and_vary() {
        let mut chooser = VariantChooser::default();
        let picks: Vec<usize> = (0..100).map(|_| chooser.pick(3)).collect();
        assert!(picks.iter().all(|&pick| pick < 3));
        assert!((0..3).all(|variant| picks.contains(&variant)));
        assert_eq!(chooser.pick(0), 0);
    }
}

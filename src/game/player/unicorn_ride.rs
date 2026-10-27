//! The unicorn ride (`unic`), from the Chariot or My Little Unicorn: while more than its last
//! second is left Isaac flashes a new color every fourth frame (`playcol()`: DoAction 4639,
//! DoAction 4641), then blinks every other frame until it ends.

use crate::game::original_units::Frames;
use crate::game::random::Random;

/// A new color every fourth frame, until the last 30 frames, which blink.
const FRAMES_PER_COLOR: Frames = Frames(4.0);
const BLINKING_FRAMES: Frames = Frames(30.0);
const FRAMES_PER_BLINK: Frames = Frames(2.0);

/// The colors Isaac flashes, each picked in turn with odds of one in 5, 4, 3, 2 and 1 (the
/// original's sixth, magenta, is never reached): DoAction 4643, DoAction 4644, DoAction 4645.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RideColor {
    Red,
    Green,
    Blue,
    Yellow,
    Cyan,
}

impl RideColor {
    pub const ALL: [Self; 5] = [Self::Red, Self::Green, Self::Blue, Self::Yellow, Self::Cyan];

    fn roll(random: &mut Random) -> Self {
        let mut colors_left = Self::ALL.len();
        for color in Self::ALL {
            if random.below(i32::try_from(colors_left).unwrap_or(1)) == 0 {
                return color;
            }
            colors_left -= 1;
        }
        Self::Cyan
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct UnicornRide {
    frames_left: Frames,
    color: Option<RideColor>,
}

impl UnicornRide {
    /// Rides for `frames`, or on for longer if already riding.
    pub fn start(&mut self, frames: Frames) {
        self.frames_left = self.frames_left.max(frames);
    }

    /// Entering a room ends the ride (`unic = -1`, DoAction 32967).
    pub fn end(&mut self) {
        self.frames_left = Frames::ZERO;
    }

    pub fn is_riding(self) -> bool {
        self.frames_left > Frames::ZERO
    }

    /// Counts down; on every fourth whole frame before the blinking, a new color, but only while
    /// Isaac is drawn as himself (`recolor`), not holding something up.
    pub fn advance(&mut self, elapsed: Frames, recolor: bool, random: &mut Random) {
        let before = self.frames_left;
        self.frames_left = (before - elapsed).max(Frames::ZERO);
        let crossed = (self.frames_left.whole()..before.whole())
            .map(Frames::from_number)
            .any(|frame| frame.is_on_cycle(FRAMES_PER_COLOR, Frames::ZERO) && frame > BLINKING_FRAMES);
        if crossed && recolor {
            self.color = Some(RideColor::roll(random));
        }
        if self.frames_left.floor() <= BLINKING_FRAMES {
            self.color = None;
        }
    }

    /// The color Isaac flashes, if any.
    pub fn color(self) -> Option<RideColor> {
        self.color
    }

    /// Hidden on every other frame of the ride's last second.
    pub fn blinks_hidden(self) -> bool {
        self.is_riding()
            && self.frames_left.floor() <= BLINKING_FRAMES
            && self.frames_left.is_on_cycle(FRAMES_PER_BLINK, Frames::ZERO)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_ride_flashes_colors_then_blinks_then_ends() {
        let mut random = Random::with_seed(1);
        let mut ride = UnicornRide::default();
        ride.start(Frames(180.0));
        let mut colors = Vec::new();
        let mut hidden_frames = 0;
        while ride.is_riding() {
            ride.advance(Frames(1.0), true, &mut random);
            colors.extend(ride.color());
            hidden_frames += u32::from(ride.blinks_hidden());
        }
        assert!(colors.len() > 100, "a color from the first fourth frame on");
        assert!(RideColor::ALL.iter().all(|color| colors.contains(color)));
        assert_eq!(hidden_frames, 15, "every other frame of the last 30");
        assert_eq!(ride.color(), None);
    }

    #[test]
    fn no_new_color_while_holding_something_up() {
        let mut random = Random::with_seed(1);
        let mut ride = UnicornRide::default();
        ride.start(Frames(180.0));
        ride.advance(Frames(20.0), false, &mut random);
        assert_eq!(ride.color(), None);
    }
}

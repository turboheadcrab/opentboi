//! The music going down under a room jingle (`mmus`): while one plays, apart from its last second,
//! the music loses a few percent every frame down to a floor (Timeline 1058–1060); afterwards it
//! climbs back, slower (Timeline 897–903).

use crate::game::original_units::Frames;
use crate::game::parameters::MusicParameters;

#[derive(Clone, Copy, Debug)]
pub(super) struct MusicDucking {
    parameters: MusicParameters,
    /// How long until the jingle lets the music go (`mmus` clears a second before its end).
    jingle_left: Frames,
    /// The music's volume, in percent of full.
    level: f32,
}

impl MusicDucking {
    pub fn new(parameters: MusicParameters) -> Self {
        Self {
            parameters,
            jingle_left: Frames::ZERO,
            level: parameters.volume,
        }
    }

    /// A jingle of `length` begins; a newer one replaces the older.
    pub fn start_jingle(&mut self, length: Frames) {
        self.jingle_left = (length - self.parameters.jingle_tail).max(Frames::ZERO);
    }

    /// Moves the volume for `elapsed`; whether it changed.
    pub fn advance(&mut self, elapsed: Frames) -> bool {
        let before = self.level;
        if self.jingle_left > Frames::ZERO {
            self.level = (self.level - self.parameters.duck_step * elapsed.get()).max(self.parameters.duck_floor);
            self.jingle_left = (self.jingle_left - elapsed).max(Frames::ZERO);
        } else {
            self.level = (self.level + self.parameters.recover_step * elapsed.get()).min(self.parameters.volume);
        }
        self.level.total_cmp(&before).is_ne()
    }

    /// What is left of the music's volume: 1 when nothing plays over it.
    pub fn factor(&self) -> f32 {
        self.level / self.parameters.volume
    }
}

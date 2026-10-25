//! Opening key blocks by walking into them with a key (`laps()`).

use glam::Vec2;

use crate::game::original_units::Frames;
use crate::game::room::contents::RoomContents;
use crate::game::sound_event::SoundEvent;

impl RoomContents {
    /// `laps()`: pushing into a key block for a few frames opens it with a key (if `has_key`);
    /// returns whether one opened, so the key is spent.
    pub fn push_against_key_blocks(
        &mut self,
        player_position: Vec2,
        player_velocity: Vec2,
        has_key: bool,
        elapsed: Frames,
        sounds: &mut Vec<SoundEvent>,
    ) -> bool {
        let key_parameters = self.parameters.keys;
        let reach_point = player_position + player_velocity.normalize_or_zero() * key_parameters.reach;
        let target = (player_velocity != Vec2::ZERO && has_key)
            .then(|| {
                self.grid_objects
                    .iter()
                    .position(|object| object.is_closed_key_block() && object.tile_bounds().contains(reach_point))
            })
            .flatten();
        // The push wears off by 0.2 each frame; pushing the same block adds one, and past 2.8
        // (compared before adding) it opens.
        if self.key_push_frames > Frames::ZERO {
            self.key_push_frames -= elapsed * key_parameters.push_decay_per_frame;
        }
        let opens = match target {
            Some(index) if self.pushed_key_block == Some(index) => {
                let before = self.key_push_frames;
                self.key_push_frames += elapsed;
                before > key_parameters.push_frames_to_open
            }
            Some(index) => {
                self.pushed_key_block = Some(index);
                self.key_push_frames = Frames::ZERO;
                false
            }
            None => false,
        };
        if let Some(index) = target
            && opens
        {
            self.grid_objects[index].open_with_key();
            self.pushed_key_block = None;
            self.key_push_frames = Frames::ZERO;
            sounds.extend([SoundEvent::KeyUsed, SoundEvent::KeyBlockBroken]);
            return true;
        }
        false
    }
}

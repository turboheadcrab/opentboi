//! What changes on the floor by itself as frames pass: what broken objects leave fades away
//! (`actions2()`), and the blood's tint is drawn again (`bloc`, in the main loop).

use crate::game::original_units::Frames;
use crate::game::random::Random;
use crate::game::room::contents::RoomContents;
use crate::game::room::grid_object::GridObject;
use crate::game::room::sight::FADING_EVERY;

impl RoomContents {
    /// Advances the room's clock by `elapsed`, doing each frame's work for the frames crossed.
    pub(super) fn pass_floor_frames(&mut self, elapsed: Frames, random: &mut Random) {
        let first_frame = self.age_frames.whole() + 1;
        self.age_frames += elapsed;
        for number in first_frame..=self.age_frames.whole() {
            let frame = Frames::from_number(number);
            if frame.is_on_cycle(FADING_EVERY, Frames::ZERO) {
                self.grid_objects.iter_mut().for_each(GridObject::fade_level_value);
            }
            self.blood.retint(frame, &self.parameters.splats, random);
        }
    }

    /// The blood the enemies spilled this turn lands on the floor.
    pub(super) fn spill_enemy_blood(&mut self, random: &mut Random) {
        for splatter in self.enemies.take_splatters() {
            self.blood.splatter(splatter, &self.parameters.splats, random);
        }
    }
}

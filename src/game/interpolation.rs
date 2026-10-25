//! Smooth drawing between ticks: moving things remember where they were a tick ago, and each frame
//! is drawn that far between the last two ticks (one tick behind the simulation). Off with
//! `--no-interpolation`, which draws the latest tick as it is.

use crate::game::attacks::splash::Splash;
use crate::game::draw::DrawItem;
use crate::game::{Game, room};

impl Game {
    /// Called at the start of every tick, before anything moves.
    pub(crate) fn remember_positions(&mut self) {
        self.player.remember_position();
        self.bombs.remember_positions();
        self.blue_flies.remember_positions();
        self.floor.current_room_mut().pickups.remember_positions();
        if let Some(transition) = &mut self.room_transition {
            transition.remember_offset();
        }
    }

    /// How far between the previous tick and the latest one to draw: `1.0` is the latest.
    fn blend(&self) -> f32 {
        if self.settings.interpolation {
            self.clock.progress()
        } else {
            1.0
        }
    }

    /// The room with everything in it, between ticks, slid during a room change.
    pub(crate) fn fill_blended_room_draw_items(&self, draw_items: &mut Vec<DrawItem>) {
        let first_item = draw_items.len();
        let blend = self.blend();
        self.fill_room_draw_items(draw_items);
        for item in &mut draw_items[first_item..] {
            item.center -= item.motion * (1.0 - blend);
        }
        if let Some(transition) = &self.room_transition {
            transition.apply(draw_items, first_item, blend);
        }
    }

    /// Everything inside the room this tick: the room, Isaac, what lies about and flies.
    pub(crate) fn fill_room_draw_items(&self, draw_items: &mut Vec<DrawItem>) {
        room::append_draw_items(self.room_appearance, self.floor.current_art(), draw_items);
        // The blink of the fatal hit would make the death flicker, so a dying Isaac stays shown.
        let blinking = self.inventory.blinks_hidden() && !self.is_dying();
        if !blinking && !self.inventory.ride.blinks_hidden() {
            self.player.append_draw_items(draw_items);
        }
        self.floor.append_draw_items(self.last_input_device, draw_items);
        self.bombs.append_draw_items(draw_items);
        self.blue_flies.append_draw_items(draw_items);
        for tear in &self.tears {
            tear.append_draw_items(draw_items);
        }
        draw_items.extend(self.splashes.iter().map(Splash::draw_item));
    }
}

/// Marks `draw_items` as having moved by `motion` during the last tick.
pub fn set_motion(draw_items: &mut [DrawItem], motion: glam::Vec2) {
    for item in draw_items {
        item.motion = motion;
    }
}

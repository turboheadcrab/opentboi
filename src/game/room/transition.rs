//! The slide between rooms, as the original's main loop does it. Entering a room (DoAction
//! 32871–32888) puts the new room 640 px off to the side or 440 px above or below, by the door
//! taken. Each frame (DoAction 30279–30302) it moves home: within 2 px it is there and the old
//! room's snapshot goes, else its offset keeps 0.5 plus 0.004 per pixel of it (at most 0.22 more)
//! of itself, and the old room's snapshot follows at the same distance behind, 40 px further apart
//! vertically. The HUD is outside the moving clip and stays put.

use glam::Vec2;

use crate::game::draw::DrawItem;
use crate::game::original_units::{Frames, Pixels, Retention, length};
use crate::game::room::doors::DoorSide;

/// Where the new room starts: off to the side, or above or below.
const HORIZONTAL_START: Pixels = Pixels(640.0);
const VERTICAL_START: Pixels = Pixels(440.0);
/// The old room's snapshot keeps this much further away vertically.
const VERTICAL_SNAPSHOT_GAP: Pixels = Pixels(40.0);
/// What the offset keeps of itself each frame: this, plus a share by its length, capped.
const BASE_RETENTION_PER_FRAME: Retention = Retention(0.5);
const RETENTION_PER_PIXEL: f32 = 0.004;
const MAXIMUM_EXTRA_RETENTION: f32 = 0.22;
/// Closer than this the slide is over.
const FINISHED_DISTANCE: Pixels = Pixels(2.0);

pub struct RoomTransition {
    old_room: Vec<DrawItem>,
    start_offset: Vec2,
    /// Where the new room is drawn relative to its place; shrinks to zero.
    new_room_offset: Vec2,
    /// The offset when the current tick began.
    previous_offset: Vec2,
}

impl RoomTransition {
    /// `old_room` is everything that was drawn in the room left through `side`.
    pub fn start(side: DoorSide, mut old_room: Vec<DrawItem>) -> Self {
        // The snapshot stands still.
        for item in &mut old_room {
            item.motion = Vec2::ZERO;
        }
        let start_offset = match side {
            DoorSide::Left => Vec2::new(-HORIZONTAL_START.get(), 0.0),
            DoorSide::Right => Vec2::new(HORIZONTAL_START.get(), 0.0),
            DoorSide::Bottom => Vec2::new(0.0, -VERTICAL_START.get()),
            DoorSide::Top => Vec2::new(0.0, VERTICAL_START.get()),
        };
        Self {
            old_room,
            start_offset,
            new_room_offset: start_offset,
            previous_offset: start_offset,
        }
    }

    pub fn remember_offset(&mut self) {
        self.previous_offset = self.new_room_offset;
    }

    /// Advances the slide; `true` once it is over.
    pub fn advance(&mut self, elapsed: Frames) -> bool {
        let distance = length(self.new_room_offset);
        if distance < FINISHED_DISTANCE {
            return true;
        }
        let extra = (distance.get() * RETENTION_PER_PIXEL).min(MAXIMUM_EXTRA_RETENTION);
        self.new_room_offset *= Retention(BASE_RETENTION_PER_FRAME.0 + extra).over(elapsed);
        false
    }

    /// Shifts the new room's items (from `first_new_item` on) and adds the old room's snapshot,
    /// `blend` of the way from the previous tick's offset to the latest.
    pub fn apply(&self, draw_items: &mut Vec<DrawItem>, first_new_item: usize, blend: f32) {
        let new_room_offset = self.previous_offset.lerp(self.new_room_offset, blend);
        for item in &mut draw_items[first_new_item..] {
            item.center += new_room_offset;
        }
        let gap = Vec2::new(0.0, VERTICAL_SNAPSHOT_GAP.get() * self.start_offset.y.signum());
        let old_room_offset = new_room_offset - self.start_offset + gap * f32::from(self.start_offset.y != 0.0);
        draw_items.extend(self.old_room.iter().map(|item| DrawItem {
            center: item.center + old_room_offset,
            ..*item
        }));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slide_settles_in_about_half_a_second() {
        let mut transition = RoomTransition::start(DoorSide::Right, Vec::new());
        // Drawn at 144 Hz: each step covers 30 / 144 of an original frame.
        let step_frames = Frames(30.0 / 144.0);
        let steps = (0..1000).take_while(|_| !transition.advance(step_frames)).count();
        let seconds = steps as f32 / 144.0;
        assert!((0.3..0.8).contains(&seconds), "took {seconds} s");
    }
}

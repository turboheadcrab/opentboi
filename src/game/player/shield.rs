//! The bubble round Isaac while a trinket shields him (`player.d.bub`, drawn while `playsave` is
//! above zero, DoAction 29487–29500): its clip loops, and in the last 50 frames it flickers, on or
//! off every 5 frames.

use glam::Vec2;

use crate::game::draw::{ACTOR_PART_DEPTH_STEP, DrawItem, Rgba, actor_depth};
use crate::game::original_units::Frames;
use crate::game::texture_id::{BUBBLE_FRAME_COUNT, TextureId};

/// The bubble's picture is this big, around its own middle.
pub const BUBBLE_SIZE: Vec2 = Vec2::new(96.0, 96.0);
/// Where that middle lies above Isaac's floor point: the clip is placed at (-40.05, -65.65) from it
/// and its art spreads 40 px right of and below its own origin.
const BUBBLE_CENTER_ABOVE_FLOOR: Vec2 = Vec2::new(-0.05, 25.65);
/// It flickers when this little is left of the shield, changing every 5 frames.
const FLICKER_BELOW: Frames = Frames(50.0);
const FLICKER_PERIOD: Frames = Frames(5.0);
const NO_FALLBACK: Rgba = Rgba {
    red: 0.0,
    green: 0.0,
    blue: 0.0,
    alpha: 0.0,
};

/// The bubble for Isaac standing at `position` (his floor point, world), with `frames_left` of the
/// shield remaining, `now` into the room; `None` when it is gone or flickered off.
pub fn draw_item(position: Vec2, frames_left: Frames, now: Frames) -> Option<DrawItem> {
    if frames_left <= Frames::ZERO {
        return None;
    }
    let flickered_off = frames_left < FLICKER_BELOW && (frames_left / FLICKER_PERIOD).floor() % 2.0 == 0.0;
    if flickered_off {
        return None;
    }
    let texture = TextureId::ShieldBubble(now.looped_frame(BUBBLE_FRAME_COUNT));
    Some(DrawItem::textured(
        texture,
        position + BUBBLE_CENTER_ABOVE_FLOOR,
        1.0,
        NO_FALLBACK,
        actor_depth(position.y) + 2.0 * ACTOR_PART_DEPTH_STEP,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_bubble_is_there_while_the_shield_is_and_flickers_at_the_end() {
        let drawn = |frames_left| draw_item(Vec2::ZERO, Frames(frames_left), Frames::ZERO).is_some();
        assert!(drawn(150.0));
        assert!(!drawn(0.0));
        let flickering: Vec<bool> = [49.0, 44.0, 39.0, 34.0].into_iter().map(drawn).collect();
        assert_eq!(flickering, [true, false, true, false]);
    }
}

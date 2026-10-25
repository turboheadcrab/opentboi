//! The darkness and the beam of light over dying Isaac: the death clip (5683) places a shape
//! (5682) above him that fades in over its first 17 frames, a sixteenth of full opacity each
//! frame. It is 1,622 by 1,158 units of 70% black (fill alpha 179) with a pale beam (alpha 77)
//! falling on him. Only the beam's strip is a texture; the rest is drawn as plain translucent
//! black. Its spot on the floor (5677) fades in with it as part of Isaac's own animation.

use glam::Vec2;

use crate::game::draw::{ACTOR_PART_DEPTH_STEP, DrawItem, Rgba, actor_depth};
use crate::game::original_units::Frames;
use crate::game::texture_id::TextureId;

/// Steps of the fade: frame 1 shows none of it, frame 17 all.
pub const SPOTLIGHT_FADE_STEPS: u8 = 17;
/// The strip holding the beam, centered this far above Isaac's floor point.
pub const BEAM_SIZE: Vec2 = Vec2::new(104.0, 654.0);
const BEAM_CENTER_ABOVE_FEET: f32 = 303.0;
/// The shape's extent around Isaac's floor point (up is +y): 811 to each side, 629 above and
/// 529 below.
const OVERLAY_HALF_WIDTH: f32 = 811.0;
const OVERLAY_TOP: f32 = 629.0;
const OVERLAY_BOTTOM: f32 = -529.0;
const DARKNESS_ALPHA: f32 = 179.0 / 255.0;
/// Above Isaac's body and head, as depth 7 is above his clip's depth 3.
const ABOVE_ISAAC: f32 = 8.0 * ACTOR_PART_DEPTH_STEP;

/// Appends the overlay `frames` into dying for Isaac standing at `floor_point`, `scale` times
/// his size.
pub fn append(floor_point: Vec2, scale: f32, frames: Frames, draw_items: &mut Vec<DrawItem>) {
    let step = frames.held_frame(SPOTLIGHT_FADE_STEPS);
    let opacity = f32::from(step) / f32::from(SPOTLIGHT_FADE_STEPS - 1);
    let depth = actor_depth(floor_point.y) + ABOVE_ISAAC;
    let darkness = Rgba {
        alpha: DARKNESS_ALPHA * opacity,
        ..Rgba::opaque(0.0, 0.0, 0.0)
    };
    let beam_center = floor_point + Vec2::new(0.0, BEAM_CENTER_ABOVE_FEET) * scale;
    let mut beam = DrawItem::textured(TextureId::DeathSpotlight(step), beam_center, scale, darkness, depth);
    beam.size = BEAM_SIZE * scale;
    draw_items.push(beam);
    let half_beam = BEAM_SIZE.x / 2.0;
    let beam_bottom = BEAM_CENTER_ABOVE_FEET - BEAM_SIZE.y / 2.0;
    // Left and right of the strip, then below it: (left, bottom) and (right, top) corners.
    let rectangles = [
        (
            Vec2::new(-OVERLAY_HALF_WIDTH, OVERLAY_BOTTOM),
            Vec2::new(-half_beam, OVERLAY_TOP),
        ),
        (
            Vec2::new(half_beam, OVERLAY_BOTTOM),
            Vec2::new(OVERLAY_HALF_WIDTH, OVERLAY_TOP),
        ),
        (Vec2::new(-half_beam, OVERLAY_BOTTOM), Vec2::new(half_beam, beam_bottom)),
    ];
    draw_items.extend(rectangles.map(|(minimum, maximum)| {
        DrawItem::solid(
            floor_point + (minimum + maximum) / 2.0 * scale,
            (maximum - minimum) * scale,
            darkness,
            depth,
        )
    }));
}

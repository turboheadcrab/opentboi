//! Where a room layout places pickups and what each marker creates.

use glam::Vec2;

use crate::game::parameters::PickupParameters;
use crate::game::room::layout::RoomLayout;
use crate::game::room::tile_center;

/// Markers between -1 and 0 create object type 5 (a pickup) with the fraction as its variant: its
/// clip frame is the fraction in hundredths, rounded down, at least 1.
const PICKUP_MARKER_VALUES: std::ops::Range<f32> = -1.0..0.0;
const MARKER_FRACTION_SCALE: f32 = 100.0;
const MARKER_FRAME_ROUNDING: f32 = 0.499;

/// The pickup markers in a layout: the `b5` frame each creates, and where.
pub fn markers(layout: &RoomLayout, parameters: PickupParameters) -> impl Iterator<Item = (u8, Vec2)> + '_ {
    layout
        .cells
        .iter()
        .filter(|cell| PICKUP_MARKER_VALUES.contains(&cell.value))
        .map(move |cell| {
            let frame = crate::whole_numbers::clamped_u8(
                (-cell.value * MARKER_FRACTION_SCALE - MARKER_FRAME_ROUNDING)
                    .round()
                    .max(1.0),
            );
            let center = tile_center(cell.column, cell.row) - Vec2::new(0.0, parameters.spawn_drop.get());
            (frame, center)
        })
}

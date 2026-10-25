//! Characters several parts of the catalog draw from.

use swf::CharacterId;

/// Pickups (clip `b5`, 3376): its frame picks the kind; frame 15 holds shop wares and the
/// machines' frames hold the prizes they show.
pub(super) const PICKUP_CLIP: CharacterId = 3376;
/// The bomb's fuse spark (2011) hides itself unless its owner is a live bomb: hidden wherever a
/// bomb is only shown (wares, prizes, held up).
pub(super) const FUSE_SPARK_CLIP: CharacterId = 2011;
/// The HUD's `coins` field (795); `bombs` (796) and `keys` (797) are identical apart from their
/// variable, so one set of textures serves all three, and its glyphs too.
pub(super) const COUNTER_FIELD: CharacterId = 795;
pub(super) const COUNTER_VARIABLE: &str = "coins";
/// Center of the field's bounds (59.1–84.9, -2–22.1) in its own coordinates.
pub(super) const COUNTER_CENTER: (f32, f32) = (72.0, 10.05);

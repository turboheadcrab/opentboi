//! Frames of the machine clip (2246, the original's `d.d` for pickup frame 8) and how long the
//! animation each one shows (`d.d.d`) runs.

use glam::Vec2;

/// Slot and fortune machines standing; the fortune machine's parts (`col` 41, DoAction 32) show
/// their second frame (`sloter()`, DoAction 34), and only a pickup's own clip counts (DoAction 28).
pub const SLOT_IDLE: u8 = 1;
pub const SPINNING: u8 = 2;
/// `slotf()` picks the result shown after a spin.
pub const FIRST_RESULT: u8 = 3;
pub const LAST_RESULT: u8 = 24;
pub const SLOT_BROKEN: u8 = 30;
pub const BLOOD_IDLE: u8 = 32;
pub const DONATING: u8 = 33;
pub const BLOOD_BROKEN: u8 = 34;
pub const BEGGAR_IDLE: u8 = 36;
pub const BEGGAR_PAID: u8 = 37;
pub const BEGGAR_GIVES: u8 = 38;
pub const BEGGAR_BLOWN: u8 = 39;
pub const BEGGAR_LEAVES: u8 = 40;
pub const SHELL_IDLE: u8 = 41;
pub const SHUFFLING: u8 = 42;
/// The left, middle and right shells lifted.
pub const LEFT_SHELL: u8 = 43;
pub const MIDDLE_SHELL: u8 = 44;
pub const RIGHT_SHELL: u8 = 45;

/// Frames of the animation shown on `frame` (1 for a still), from the SWF.
pub fn animation_length(frame: u8) -> u8 {
    match frame {
        SPINNING => 52,
        FIRST_RESULT..=LAST_RESULT => 46,
        SLOT_BROKEN | BEGGAR_PAID => 41,
        DONATING => 55,
        BLOOD_BROKEN => 38,
        BEGGAR_GIVES => 61,
        BEGGAR_BLOWN | LEFT_SHELL..=RIGHT_SHELL => 42,
        BEGGAR_LEAVES => 62,
        SHUFFLING => 117,
        _ => 1,
    }
}

/// Canvas holding all of a frame's art around the machine's position (measured with
/// `cargo test texture_extents -- --ignored --nocapture`).
pub fn texture_size(frame: u8) -> Vec2 {
    match frame {
        SLOT_BROKEN | BLOOD_BROKEN => Vec2::new(272.0, 244.0),
        BLOOD_IDLE | DONATING => Vec2::new(172.0, 232.0),
        BEGGAR_IDLE..=BEGGAR_GIVES => Vec2::new(56.0, 104.0),
        BEGGAR_BLOWN => Vec2::new(148.0, 148.0),
        BEGGAR_LEAVES => Vec2::new(64.0, 368.0),
        SHELL_IDLE..=RIGHT_SHELL => Vec2::new(116.0, 136.0),
        _ => Vec2::new(84.0, 208.0),
    }
}

/// A result pays out once its animation is past this frame (the blood bank's past 36).
pub const PAYOUT_AFTER_FRAME: u8 = 15;
pub const BLOOD_PAYOUT_AFTER_FRAME: u8 = 36;
/// The shells can be picked near the end of the shuffle, past this frame.
pub const PICKABLE_AFTER_FRAME: u8 = 110;
/// Animation frames that play a sound: the reels stopping, the shuffle, the shells lifted, the
/// beggar leaving.
pub const REELS_STOP_FRAME: u8 = 17;
pub const SHUFFLE_SOUND_FRAME: u8 = 35;
pub const SHELLS_LIFTED_FRAME: u8 = 65;
pub const BEGGAR_LEAVES_SOUND_FRAME: u8 = 20;
/// `_parent._parent._parent.done = true` on frame 61 of the leaving beggar (2236).
pub const BEGGAR_GONE_FRAME: u8 = 61;

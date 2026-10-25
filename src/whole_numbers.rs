//! Floats to whole numbers. Rust's `as` from a float clamps to the target type's range and turns
//! NaN into 0, which is what pixel sizes, sample counts and frame numbers need; these functions
//! name that once instead of at every cast.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "float-to-integer `as` clamps to the target's range"
)]

pub const fn clamped_u8(value: f32) -> u8 {
    value as u8
}

pub const fn clamped_u16(value: f32) -> u16 {
    value as u16
}

pub const fn clamped_u32(value: f32) -> u32 {
    value as u32
}

pub const fn clamped_i32(value: f32) -> i32 {
    value as i32
}

pub const fn clamped_usize(value: f64) -> usize {
    value as usize
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn values_out_of_range_are_clamped() {
        assert_eq!(clamped_u8(300.0), u8::MAX);
        assert_eq!(clamped_u32(-4.0), 0);
        assert_eq!(clamped_i32(f32::NAN), 0);
        assert_eq!(clamped_usize(2.9), 2);
    }
}

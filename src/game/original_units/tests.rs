use glam::Vec2;

use super::*;

#[test]
fn units_combine_as_physics_does() {
    assert_eq!(PixelsPerFrame(4.0) * Frames(2.0), Pixels(8.0));
    assert_eq!(PixelsPerFrameSquared(1.5) * Frames(2.0), PixelsPerFrame(3.0));
    assert!((Retention(0.5).over(Frames(2.0)) - 0.25).abs() < 1e-6);
    assert_eq!(Vec2::X * Frames(3.0), Vec2::new(3.0, 0.0));
    assert!(Frames(2.0) < Frames(3.0));
    assert_eq!(frames_per_tick(60), Frames(0.5));
}

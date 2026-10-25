use super::*;

#[test]
fn original_values_reproduce_known_results() {
    let original = TranscribedOriginal.parameters();
    assert!((original.movement.top_speed_per_frame().get() - 8.267).abs() < 0.01);
    assert_eq!(original.firing.delay_counter(), Frames(10.0));
    assert_eq!(original.firing.frames_between_shots(), Frames(11.0));
    assert!((original.firing.face_timer_after_shot().get() - 11.5).abs() < 1e-6);
    assert!((original.tear.scale_for_damage(original.tear.base_damage) - 1.1206).abs() < 1e-3);
}

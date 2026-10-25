use super::*;

#[test]
fn keys_and_pads_add_up() {
    let pad = GamepadReading {
        kind: GamepadKind::Xbox,
        left_stick: Vec2::ZERO,
        right_stick: Vec2::ZERO,
        pressed: |button| button == PadButton::North || button == BOMB_BUTTON,
    };
    let input = input_state(|key| key == Key::KeyD, [pad], None);
    assert_eq!(input.movement, Vec2::X);
    assert_eq!(input.shoot_axes, Vec2::Y);
    assert!(input.place_bomb);
}

fn resting_pad(pressed: impl Fn(PadButton) -> bool) -> GamepadReading<impl Fn(PadButton) -> bool> {
    GamepadReading {
        kind: GamepadKind::Xbox,
        left_stick: Vec2::new(0.1, 0.0),
        right_stick: Vec2::ZERO,
        pressed,
    }
}

#[test]
fn nothing_touched_reports_no_device() {
    let input = input_state(|_| false, [resting_pad(|_| false)], None);
    assert_eq!(input.device_in_use, None, "a stick inside its deadzone is not input");
}

#[test]
fn a_pad_button_reports_the_gamepad() {
    let input = input_state(|_| false, [resting_pad(|button| button == PadButton::Back)], None);
    assert_eq!(input.device_in_use, Some(InputDevice::Gamepad(GamepadKind::Xbox)));
}

#[test]
fn a_pushed_stick_reports_the_gamepad() {
    let pad = GamepadReading {
        kind: GamepadKind::Xbox,
        left_stick: Vec2::new(0.9, 0.0),
        right_stick: Vec2::ZERO,
        pressed: |_| false,
    };
    let input = input_state(|_| false, [pad], None);
    assert_eq!(input.device_in_use, Some(InputDevice::Gamepad(GamepadKind::Xbox)));
}

#[test]
fn keys_and_the_mouse_report_the_keyboard() {
    let keyboard = input_state(|key| key == Key::F11, [resting_pad(|_| false)], None);
    assert_eq!(keyboard.device_in_use, Some(InputDevice::KeyboardAndMouse));
    let mouse = input_state(|_| false, [resting_pad(|_| false)], Some(Vec2::ONE));
    assert_eq!(mouse.device_in_use, Some(InputDevice::KeyboardAndMouse));
}

#[test]
fn the_keyboard_wins_when_both_are_used() {
    let input = input_state(
        |key| key == Key::KeyE,
        [resting_pad(|button| button == BOMB_BUTTON)],
        None,
    );
    assert_eq!(input.device_in_use, Some(InputDevice::KeyboardAndMouse));
}

#[test]
fn the_pad_that_was_touched_names_its_make() {
    let pad = |kind, touched: bool| GamepadReading {
        kind,
        left_stick: Vec2::ZERO,
        right_stick: Vec2::ZERO,
        pressed: move |button| touched && button == PadButton::South,
    };
    let input = input_state(
        |_| false,
        [pad(GamepadKind::Xbox, false), pad(GamepadKind::PlayStation, true)],
        None,
    );
    assert_eq!(
        input.device_in_use,
        Some(InputDevice::Gamepad(GamepadKind::PlayStation))
    );
}

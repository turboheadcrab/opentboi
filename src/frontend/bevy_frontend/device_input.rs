//! Keyboard, mouse and gamepads through Bevy; the bindings and how they combine are shared with
//! the other frontend (`frontend::controls`).

use bevy::prelude::*;
use bevy::window::{CursorOptions, PrimaryWindow};

use super::CurrentInput;
use crate::frontend::controls::{self, GamepadReading, Key, PadButton};
use crate::frontend::cursor_visibility::CursorVisibility;
use crate::frontend::gamepad_kind::{Identity, ReportedMake, gamepad_kind};

/// Flash's `onMouseDown` only reports the primary button.
const MOUSE_SHOOT_BUTTON: MouseButton = MouseButton::Left;

pub fn key_code(key: Key) -> KeyCode {
    match key {
        Key::KeyW => KeyCode::KeyW,
        Key::KeyA => KeyCode::KeyA,
        Key::KeyS => KeyCode::KeyS,
        Key::KeyD => KeyCode::KeyD,
        Key::KeyE => KeyCode::KeyE,
        Key::KeyR => KeyCode::KeyR,
        Key::KeyQ => KeyCode::KeyQ,
        Key::ArrowUp => KeyCode::ArrowUp,
        Key::ArrowDown => KeyCode::ArrowDown,
        Key::ArrowLeft => KeyCode::ArrowLeft,
        Key::ArrowRight => KeyCode::ArrowRight,
        Key::Space => KeyCode::Space,
        Key::Escape => KeyCode::Escape,
        Key::F11 => KeyCode::F11,
    }
}

/// Bevy calls the bumpers `LeftTrigger`/`RightTrigger` and the analog triggers `LeftTrigger2`/
/// `RightTrigger2`, and Back `Select`.
pub fn gamepad_button(button: PadButton) -> GamepadButton {
    match button {
        PadButton::North => GamepadButton::North,
        PadButton::South => GamepadButton::South,
        PadButton::East => GamepadButton::East,
        PadButton::West => GamepadButton::West,
        PadButton::DPadUp => GamepadButton::DPadUp,
        PadButton::DPadDown => GamepadButton::DPadDown,
        PadButton::DPadLeft => GamepadButton::DPadLeft,
        PadButton::DPadRight => GamepadButton::DPadRight,
        PadButton::LeftShoulder => GamepadButton::LeftTrigger,
        PadButton::RightShoulder => GamepadButton::RightTrigger,
        PadButton::Start => GamepadButton::Start,
        PadButton::Back => GamepadButton::Select,
        PadButton::LeftStick => GamepadButton::LeftThumb,
        PadButton::RightStick => GamepadButton::RightThumb,
        PadButton::LeftTrigger => GamepadButton::LeftTrigger2,
        PadButton::RightTrigger => GamepadButton::RightTrigger2,
    }
}

pub fn plugin(app: &mut App) {
    app.add_systems(
        PreUpdate,
        (read_devices, update_cursor).chain().after(bevy::input::InputSystems),
    );
}

/// Hides the cursor over the window while a gamepad is the device in use, and shows it again when
/// the mouse moves or the keyboard is used.
fn update_cursor(
    current_input: Res<CurrentInput>,
    window: Single<&Window, With<PrimaryWindow>>,
    mut cursor_options: Single<&mut CursorOptions, With<PrimaryWindow>>,
    mut visibility: Local<CursorVisibility>,
    mut previous_position: Local<Option<Vec2>>,
) {
    let position = window.cursor_position();
    let mouse_moved = position.is_some() && position != *previous_position;
    *previous_position = position;
    let hidden = visibility.update(mouse_moved, current_input.0.device_in_use);
    if cursor_options.visible == hidden {
        cursor_options.visible = !hidden;
    }
}

fn read_devices(
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    gamepads: Query<&Gamepad>,
    window: Single<&Window, With<PrimaryWindow>>,
    camera: Single<(&Camera, &GlobalTransform)>,
    mut current_input: ResMut<CurrentInput>,
) {
    let readings = gamepads.iter().map(|gamepad| GamepadReading {
        kind: gamepad_kind(Identity {
            reported_make: ReportedMake::Other,
            vendor_id: gamepad.vendor_id(),
            product_id: gamepad.product_id(),
        }),
        left_stick: gamepad.left_stick(),
        right_stick: gamepad.right_stick(),
        pressed: move |button| gamepad.pressed(gamepad_button(button)),
    });
    let (camera, camera_transform) = *camera;
    let aim_target = mouse_buttons
        .pressed(MOUSE_SHOOT_BUTTON)
        .then(|| cursor_world_position(&window, camera, camera_transform))
        .flatten();
    current_input.0 = controls::input_state(|key| keyboard.pressed(key_code(key)), readings, aim_target);
}

fn cursor_world_position(window: &Window, camera: &Camera, camera_transform: &GlobalTransform) -> Option<Vec2> {
    let cursor = window.cursor_position()?;
    camera.viewport_to_world_2d(camera_transform, cursor).ok()
}

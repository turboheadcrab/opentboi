//! Keyboard, mouse and gamepads through SDL; the bindings and how they combine are shared with the
//! other frontend (`frontend::controls`).

use glam::Vec2;
use sdl3::event::{Event, WindowEvent};
use sdl3::gamepad::{Axis, Button, Gamepad, GamepadType};
use sdl3::keyboard::Scancode;
use sdl3::{EventPump, GamepadSubsystem, Sdl};

use crate::frontend::controls::{self, GamepadReading, Key, PadButton, WindowCommand};
use crate::frontend::cursor_visibility::CursorVisibility;
use crate::frontend::gamepad_kind::{Identity, ReportedMake, gamepad_kind};
use crate::frontend::sdl3_frontend::graphics::sprite_renderer::ViewMapping;
use crate::game::{GamepadKind, InputState};

/// SDL reports stick axes as `i16`; full deflection is `i16::MAX`.
const AXIS_FULL_DEFLECTION: f32 = i16::MAX as f32;

fn scancode(key: Key) -> Scancode {
    match key {
        Key::KeyW => Scancode::W,
        Key::KeyA => Scancode::A,
        Key::KeyS => Scancode::S,
        Key::KeyD => Scancode::D,
        Key::KeyE => Scancode::E,
        Key::KeyR => Scancode::R,
        Key::KeyQ => Scancode::Q,
        Key::ArrowUp => Scancode::Up,
        Key::ArrowDown => Scancode::Down,
        Key::ArrowLeft => Scancode::Left,
        Key::ArrowRight => Scancode::Right,
        Key::Space => Scancode::Space,
        Key::Escape => Scancode::Escape,
        Key::F11 => Scancode::F11,
    }
}

/// The SDL button for `button`; the triggers are axes in SDL, so they have none.
fn sdl_button(button: PadButton) -> Option<Button> {
    let sdl = match button {
        PadButton::North => Button::North,
        PadButton::South => Button::South,
        PadButton::East => Button::East,
        PadButton::West => Button::West,
        PadButton::DPadUp => Button::DPadUp,
        PadButton::DPadDown => Button::DPadDown,
        PadButton::DPadLeft => Button::DPadLeft,
        PadButton::DPadRight => Button::DPadRight,
        PadButton::LeftShoulder => Button::LeftShoulder,
        PadButton::RightShoulder => Button::RightShoulder,
        PadButton::Start => Button::Start,
        PadButton::Back => Button::Back,
        PadButton::LeftStick => Button::LeftStick,
        PadButton::RightStick => Button::RightStick,
        PadButton::LeftTrigger | PadButton::RightTrigger => return None,
    };
    Some(sdl)
}

/// What the window should do after this frame's events.
#[derive(Default)]
#[allow(
    clippy::struct_excessive_bools,
    reason = "independent requests from one frame of input"
)]
pub struct WindowRequests {
    pub quit: bool,
    pub toggle_fullscreen: bool,
    pub skip_intro: bool,
    /// The window moved to another display, which may have another scale.
    pub display_changed: bool,
}

impl WindowRequests {
    fn apply(&mut self, command: Option<WindowCommand>) {
        match command {
            Some(WindowCommand::Quit) => self.quit = true,
            Some(WindowCommand::ToggleFullscreen) => self.toggle_fullscreen = !self.toggle_fullscreen,
            Some(WindowCommand::SkipIntro) => self.skip_intro = true,
            None => {}
        }
    }
}

#[derive(Default)]
pub struct DeviceInput {
    /// Started after the first frame: SDL's joystick layer takes ~250 ms to come up on Linux.
    gamepad_subsystem: Option<GamepadSubsystem>,
    gamepads: Vec<Gamepad>,
    cursor: CursorVisibility,
    /// The mouse moved since the last frame's input was read.
    mouse_moved: bool,
    /// Whether SDL has been told to hide the cursor.
    cursor_hidden_in_window: bool,
}

impl DeviceInput {
    /// Starts gamepad support; pads already plugged in then arrive as "added" events.
    pub fn enable_gamepads(&mut self, context: &Sdl) {
        match context.gamepad() {
            Ok(subsystem) => self.gamepad_subsystem = Some(subsystem),
            Err(error) => println!("Playing without gamepads: {error}"),
        }
    }

    pub fn gamepads_enabled(&self) -> bool {
        self.gamepad_subsystem.is_some()
    }

    /// Drains SDL's event queue.
    pub fn handle_events(&mut self, event_pump: &mut EventPump) -> WindowRequests {
        let mut requests = WindowRequests::default();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => requests.quit = true,
                Event::KeyDown {
                    scancode: Some(code),
                    repeat: false,
                    ..
                } => {
                    let key = Key::ALL.into_iter().find(|&key| scancode(key) == code);
                    requests.apply(key.and_then(controls::window_command_for_key));
                }
                Event::GamepadButtonDown { button, .. } => {
                    let pressed = PadButton::ALL.into_iter().find(|&pad| sdl_button(pad) == Some(button));
                    requests.apply(pressed.and_then(controls::window_command_for_button));
                }
                Event::MouseMotion { .. } => self.mouse_moved = true,
                Event::Window {
                    win_event: WindowEvent::DisplayChanged(_),
                    ..
                } => requests.display_changed = true,
                other => self.handle_gamepad_connection(&other),
            }
        }
        requests
    }

    /// Opens gamepads as they are plugged in and forgets them when they are removed.
    fn handle_gamepad_connection(&mut self, event: &Event) {
        match *event {
            Event::GamepadAdded { which, .. } => {
                let opened = self.gamepad_subsystem.as_ref().map(|subsystem| subsystem.open(which));
                match opened {
                    Some(Ok(gamepad)) => {
                        println!(
                            "Gamepad {:?}: SDL type {:?}, vendor {:04x?}, product {:04x?}; hints show {:?}",
                            gamepad.name().unwrap_or_default(),
                            gamepad.r#type(),
                            gamepad.vendor_id(),
                            gamepad.product_id(),
                            kind_of(&gamepad)
                        );
                        self.gamepads.push(gamepad);
                    }
                    Some(Err(error)) => println!("Could not open a gamepad: {error}"),
                    None => {}
                }
            }
            Event::GamepadRemoved { which, .. } => {
                self.gamepads
                    .retain(|gamepad| gamepad.id().is_ok_and(|gamepad_id| gamepad_id != which));
            }
            _ => {}
        }
    }

    pub fn current_input(&mut self, event_pump: &EventPump, view: &ViewMapping) -> InputState {
        let keyboard = event_pump.keyboard_state();
        let gamepads = self.gamepads.iter().map(|gamepad| GamepadReading {
            kind: kind_of(gamepad),
            left_stick: stick(gamepad, Axis::LeftX, Axis::LeftY),
            right_stick: stick(gamepad, Axis::RightX, Axis::RightY),
            pressed: move |button| pad_pressed(gamepad, button),
        });
        let mouse = event_pump.mouse_state();
        let aim_target = mouse
            .left()
            .then(|| view.window_point_to_world(Vec2::new(mouse.x(), mouse.y())));
        let input = controls::input_state(|key| keyboard.is_scancode_pressed(scancode(key)), gamepads, aim_target);
        self.cursor
            .update(std::mem::take(&mut self.mouse_moved), input.device_in_use);
        input
    }

    /// Hides or shows the window's cursor as the device in use calls for (only on a change).
    pub fn apply_cursor(&mut self, context: &Sdl) {
        let hidden = self.cursor.is_hidden();
        if hidden != self.cursor_hidden_in_window {
            context.mouse().show_cursor(!hidden);
            self.cursor_hidden_in_window = hidden;
        }
    }
}

/// The make SDL reports; under Steam Input it is the real controller's, not the virtual pad's.
fn kind_of(gamepad: &Gamepad) -> GamepadKind {
    let reported_make = match gamepad.r#type() {
        GamepadType::Xbox360 | GamepadType::XboxOne => ReportedMake::Xbox,
        GamepadType::PS3 | GamepadType::PS4 | GamepadType::PS5 => ReportedMake::PlayStation,
        GamepadType::NintendoSwitchPro
        | GamepadType::NintendoSwitchJoyconLeft
        | GamepadType::NintendoSwitchJoyconRight
        | GamepadType::NintendoSwitchJoyconPair => ReportedMake::Switch,
        GamepadType::Unknown | GamepadType::Standard => ReportedMake::Other,
    };
    gamepad_kind(Identity {
        reported_make,
        vendor_id: gamepad.vendor_id(),
        product_id: gamepad.product_id(),
    })
}

/// SDL reports the triggers as axes from 0 to `i16::MAX`; past halfway counts as pressed.
const TRIGGER_PRESSED_FRACTION: f32 = 0.5;

fn pad_pressed(gamepad: &Gamepad, button: PadButton) -> bool {
    match button {
        PadButton::LeftTrigger => {
            f32::from(gamepad.axis(Axis::TriggerLeft)) / AXIS_FULL_DEFLECTION > TRIGGER_PRESSED_FRACTION
        }
        PadButton::RightTrigger => {
            f32::from(gamepad.axis(Axis::TriggerRight)) / AXIS_FULL_DEFLECTION > TRIGGER_PRESSED_FRACTION
        }
        other => sdl_button(other).is_some_and(|sdl| gamepad.button(sdl)),
    }
}

/// SDL's stick y axis points down; the game's points up.
fn stick(gamepad: &Gamepad, x_axis: Axis, y_axis: Axis) -> Vec2 {
    Vec2::new(
        f32::from(gamepad.axis(x_axis)) / AXIS_FULL_DEFLECTION,
        -f32::from(gamepad.axis(y_axis)) / AXIS_FULL_DEFLECTION,
    )
    .clamp_length_max(1.0)
}

#[cfg(test)]
mod tests;

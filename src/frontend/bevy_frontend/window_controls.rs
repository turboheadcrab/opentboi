use bevy::prelude::*;
use bevy::window::{MonitorSelection, PrimaryWindow, WindowMode};

use super::device_input::{gamepad_button, key_code};
use crate::frontend::controls::{self, Key, PadButton, WindowCommand};

pub fn plugin(app: &mut App) {
    app.add_systems(Update, (handle_window_keys, report_window_size));
}

/// Prints the window's real pixel size once it is known (it should match `--window-size`).
fn report_window_size(window: Single<&Window, With<PrimaryWindow>>, mut reported: Local<bool>) {
    let (width, height) = (window.physical_width(), window.physical_height());
    if !*reported && width > 0 {
        *reported = true;
        println!("Window is {width}×{height} pixels");
    }
}

fn toggle_fullscreen(window: &mut Window) {
    window.mode = match window.mode {
        WindowMode::Windowed => WindowMode::BorderlessFullscreen(MonitorSelection::Current),
        _ => WindowMode::Windowed,
    };
}

fn handle_window_keys(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    gamepads: Query<&Gamepad>,
    mut window: Single<&mut Window, With<PrimaryWindow>>,
) {
    let key_commands = Key::ALL
        .into_iter()
        .filter(|&key| keyboard.just_pressed(key_code(key)))
        .filter_map(controls::window_command_for_key);
    let button_commands = gamepads.iter().flat_map(|gamepad| {
        PadButton::ALL
            .into_iter()
            .filter(|&button| gamepad.just_pressed(gamepad_button(button)))
            .filter_map(controls::window_command_for_button)
    });
    for command in key_commands.chain(button_commands).collect::<Vec<_>>() {
        match command {
            WindowCommand::Quit => {
                commands.write_message(AppExit::Success);
            }
            WindowCommand::ToggleFullscreen => toggle_fullscreen(&mut window),
            WindowCommand::SkipIntro => {}
        }
    }
}

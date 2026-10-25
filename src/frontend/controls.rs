//! The controls every frontend shares: which keys and buttons do what, and how one frame's device
//! readings become the game's [`InputState`]. Frontends only translate these engine-neutral names
//! to their own key and button codes and read the devices.

use glam::Vec2;

use crate::game::{
    GamepadKind, InputDevice, InputState, LEFT_STICK_DEADZONE, RIGHT_STICK_DEADZONE, apply_radial_deadzone,
};

/// Keyboard keys the game uses, by position on a US layout.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Key {
    KeyW,
    KeyA,
    KeyS,
    KeyD,
    KeyE,
    KeyR,
    KeyQ,
    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    Space,
    Escape,
    F11,
}

impl Key {
    pub const ALL: [Self; 14] = [
        Self::KeyW,
        Self::KeyA,
        Self::KeyS,
        Self::KeyD,
        Self::KeyE,
        Self::KeyR,
        Self::KeyQ,
        Self::ArrowUp,
        Self::ArrowDown,
        Self::ArrowLeft,
        Self::ArrowRight,
        Self::Space,
        Self::Escape,
        Self::F11,
    ];
}

/// Gamepad buttons the game uses, by position (South is A on an Xbox pad).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PadButton {
    North,
    South,
    East,
    West,
    DPadUp,
    DPadDown,
    DPadLeft,
    DPadRight,
    LeftShoulder,
    RightShoulder,
    Start,
    Back,
    /// Pressing a stick in (L3, R3).
    LeftStick,
    RightStick,
    /// The analog triggers, pressed past halfway.
    LeftTrigger,
    RightTrigger,
}

impl PadButton {
    pub const ALL: [Self; 16] = [
        Self::RightShoulder,
        Self::LeftTrigger,
        Self::North,
        Self::South,
        Self::East,
        Self::West,
        Self::DPadUp,
        Self::DPadDown,
        Self::DPadLeft,
        Self::DPadRight,
        Self::LeftShoulder,
        Self::Start,
        Self::Back,
        Self::LeftStick,
        Self::RightStick,
        Self::RightTrigger,
    ];
}

/// Directions in the order left, right, down, up.
pub type DirectionBindings<T> = [T; 4];

/// The original's WASD to move and arrows to shoot (Flash key codes 65, 68, 83, 87 and 37–40).
pub const MOVEMENT_KEYS: DirectionBindings<Key> = [Key::KeyA, Key::KeyD, Key::KeyS, Key::KeyW];
pub const SHOOT_KEYS: DirectionBindings<Key> = [Key::ArrowLeft, Key::ArrowRight, Key::ArrowDown, Key::ArrowUp];
pub const MOVEMENT_BUTTONS: DirectionBindings<PadButton> = [
    PadButton::DPadLeft,
    PadButton::DPadRight,
    PadButton::DPadDown,
    PadButton::DPadUp,
];
/// Face buttons shoot like Rebirth: North/West/East/South = up/left/right/down.
pub const SHOOT_BUTTONS: DirectionBindings<PadButton> =
    [PadButton::West, PadButton::East, PadButton::South, PadButton::North];
/// The original drops bombs with E; Rebirth's pads use LB (the left trigger is
/// the active item).
pub const BOMB_KEY: Key = Key::KeyE;
pub const BOMB_BUTTON: PadButton = PadButton::LeftShoulder;
/// The card (or pill): Q in the original, RB on a gamepad as in Rebirth.
pub const USE_CARD_KEY: Key = Key::KeyQ;
pub const USE_CARD_BUTTON: PadButton = PadButton::RightShoulder;
/// The active item: Space in the original, LT on a gamepad as in Rebirth.
pub const USE_ITEM_KEY: Key = Key::Space;
pub const USE_ITEM_BUTTON: PadButton = PadButton::LeftTrigger;
pub const QUIT_KEY: Key = Key::Escape;
pub const QUIT_BUTTON: PadButton = PadButton::Start;
pub const TOGGLE_FULLSCREEN_KEY: Key = Key::F11;
/// Temporary: Back (Select/View) toggles fullscreen too, until there is an options menu.
pub const TOGGLE_FULLSCREEN_BUTTON: PadButton = PadButton::Back;
/// Rebirth restarts the run while R is held, or the right stick pressed in together with the right
/// trigger; the game times the hold.
pub const RESTART_KEY: Key = Key::KeyR;
pub const RESTART_BUTTONS: [PadButton; 2] = [PadButton::RightStick, PadButton::RightTrigger];
/// The original skips its intro with Space; South does the same on a gamepad.
pub const SKIP_INTRO_KEY: Key = Key::Space;
pub const SKIP_INTRO_BUTTON: PadButton = PadButton::South;
/// The death note's "Replay?" answers Space too.
pub const REPLAY_KEY: Key = Key::Space;
pub const REPLAY_BUTTON: PadButton = PadButton::South;

/// What the window should do when a key or button goes down.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WindowCommand {
    Quit,
    ToggleFullscreen,
    SkipIntro,
}

pub fn window_command_for_key(key: Key) -> Option<WindowCommand> {
    match key {
        QUIT_KEY => Some(WindowCommand::Quit),
        TOGGLE_FULLSCREEN_KEY => Some(WindowCommand::ToggleFullscreen),
        SKIP_INTRO_KEY => Some(WindowCommand::SkipIntro),
        _ => None,
    }
}

pub fn window_command_for_button(button: PadButton) -> Option<WindowCommand> {
    match button {
        QUIT_BUTTON => Some(WindowCommand::Quit),
        TOGGLE_FULLSCREEN_BUTTON => Some(WindowCommand::ToggleFullscreen),
        SKIP_INTRO_BUTTON => Some(WindowCommand::SkipIntro),
        _ => None,
    }
}

/// One gamepad's state this frame; sticks with y pointing up and length up to 1.
pub struct GamepadReading<Pressed: Fn(PadButton) -> bool> {
    pub kind: GamepadKind,
    pub left_stick: Vec2,
    pub right_stick: Vec2,
    pub pressed: Pressed,
}

/// Builds the game's input from the keyboard (`key_pressed`), every gamepad, and the cursor's
/// world position while the primary mouse button is held (Flash's `onMouseDown` only reports that
/// one).
pub fn input_state<Pressed: Fn(PadButton) -> bool>(
    key_pressed: impl Fn(Key) -> bool,
    gamepads: impl IntoIterator<Item = GamepadReading<Pressed>>,
    mouse_aim_target: Option<Vec2>,
) -> InputState {
    let mut movement_axes = direction_axes(MOVEMENT_KEYS, &key_pressed);
    let mut shoot_axes = direction_axes(SHOOT_KEYS, &key_pressed);
    let mut place_bomb = key_pressed(BOMB_KEY);
    let mut restart_held = key_pressed(RESTART_KEY);
    let mut use_card = key_pressed(USE_CARD_KEY);
    let mut use_item = key_pressed(USE_ITEM_KEY);
    let mut replay = key_pressed(REPLAY_KEY);
    let keyboard_or_mouse_used = mouse_aim_target.is_some() || Key::ALL.into_iter().any(&key_pressed);
    let mut gamepad_used = None;
    for gamepad in gamepads {
        let left_stick = apply_radial_deadzone(gamepad.left_stick, LEFT_STICK_DEADZONE);
        let right_stick = apply_radial_deadzone(gamepad.right_stick, RIGHT_STICK_DEADZONE);
        let touched =
            left_stick != Vec2::ZERO || right_stick != Vec2::ZERO || PadButton::ALL.into_iter().any(&gamepad.pressed);
        if touched {
            gamepad_used = gamepad_used.or(Some(gamepad.kind));
        }
        movement_axes += left_stick + direction_axes(MOVEMENT_BUTTONS, &gamepad.pressed);
        shoot_axes += right_stick + direction_axes(SHOOT_BUTTONS, &gamepad.pressed);
        place_bomb |= (gamepad.pressed)(BOMB_BUTTON);
        use_card |= (gamepad.pressed)(USE_CARD_BUTTON);
        use_item |= (gamepad.pressed)(USE_ITEM_BUTTON);
        replay |= (gamepad.pressed)(REPLAY_BUTTON);
        restart_held |= RESTART_BUTTONS.iter().all(|&button| (gamepad.pressed)(button));
    }
    InputState {
        aim_target: mouse_aim_target,
        place_bomb,
        restart_held,
        use_card,
        use_item,
        replay,
        device_in_use: device_in_use(keyboard_or_mouse_used, gamepad_used),
        ..InputState::from_raw_axes(movement_axes, shoot_axes)
    }
}

/// Which device gave input this frame, given the make of the first pad that did. The keyboard wins
/// when both did: a key press is always deliberate, while a stick resting just past its deadzone
/// can read as input by itself.
fn device_in_use(keyboard_or_mouse_used: bool, gamepad_used: Option<GamepadKind>) -> Option<InputDevice> {
    if keyboard_or_mouse_used {
        return Some(InputDevice::KeyboardAndMouse);
    }
    gamepad_used.map(InputDevice::Gamepad)
}

fn direction_axes<T: Copy>(bindings: DirectionBindings<T>, pressed: impl Fn(T) -> bool) -> Vec2 {
    let [left, right, down, upward] = bindings.map(|binding| if pressed(binding) { 1.0 } else { 0.0 });
    Vec2::new(right - left, upward - down)
}

#[cfg(test)]
mod tests;

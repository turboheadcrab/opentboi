//! When the mouse cursor shows. A player on a gamepad has no use for it over the game, so it hides
//! once a gamepad is the device in use and comes back when the mouse moves or the keyboard is used.
//! Both frontends feed this each frame and apply the answer to their window.

use crate::game::InputDevice;

#[derive(Clone, Copy, Debug, Default)]
pub struct CursorVisibility {
    hidden: bool,
}

impl CursorVisibility {
    #[cfg_attr(
        not(feature = "sdl3-frontend"),
        expect(
            dead_code,
            reason = "only the SDL3 frontend asks, the Bevy one applies the answer of update"
        )
    )]
    pub fn is_hidden(self) -> bool {
        self.hidden
    }

    /// Takes in one frame: whether the mouse moved, and which device gave input (if any). Returns
    /// whether the cursor should now be hidden. The mouse moving wins over a gamepad used in the
    /// same frame; with no input at all, nothing changes.
    pub fn update(&mut self, mouse_moved: bool, device_in_use: Option<InputDevice>) -> bool {
        self.hidden = match (mouse_moved, device_in_use) {
            (true, _) | (false, Some(InputDevice::KeyboardAndMouse)) => false,
            (false, Some(InputDevice::Gamepad(_))) => true,
            (false, None) => self.hidden,
        };
        self.hidden
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::GamepadKind;

    const GAMEPAD: Option<InputDevice> = Some(InputDevice::Gamepad(GamepadKind::Xbox));

    #[test]
    fn shows_at_first_and_hides_when_a_gamepad_is_used() {
        let mut cursor = CursorVisibility::default();
        assert!(!cursor.update(false, None), "no input yet");
        assert!(cursor.update(false, GAMEPAD));
    }

    #[test]
    fn stays_hidden_while_nothing_is_used() {
        let mut cursor = CursorVisibility::default();
        cursor.update(false, GAMEPAD);
        assert!(cursor.update(false, None));
    }

    #[test]
    fn comes_back_with_the_mouse_or_the_keyboard() {
        let mut cursor = CursorVisibility::default();
        cursor.update(false, GAMEPAD);
        assert!(!cursor.update(true, None), "the mouse moved");
        cursor.update(false, GAMEPAD);
        assert!(!cursor.update(false, Some(InputDevice::KeyboardAndMouse)));
    }

    #[test]
    fn the_mouse_moving_beats_a_gamepad_in_the_same_frame() {
        let mut cursor = CursorVisibility::default();
        assert!(!cursor.update(true, GAMEPAD));
    }
}

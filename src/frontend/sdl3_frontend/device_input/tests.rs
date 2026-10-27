//! Checks the make detection against SDL's own virtual gamepads, which stand in for pads we have no
//! hardware for. They only prove our reading of SDL's answer; a real pad can still differ.

use sdl3::sys::joystick::{
    SDL_AttachVirtualJoystick, SDL_DetachVirtualJoystick, SDL_JOYSTICK_TYPE_GAMEPAD, SDL_VirtualJoystickDesc,
};

use super::kind_of;
use crate::game::GamepadKind;

const VIRTUAL_PAD_NAME: &std::ffi::CStr = c"Virtual test pad";
const VIRTUAL_AXES: u16 = 6;
const VIRTUAL_BUTTONS: u16 = 15;
const ALL_BITS: u32 = u32::MAX;
const SONY_DUALSENSE: (u16, u16) = (0x054c, 0x0ce6);
const SONY_DUALSHOCK_4: (u16, u16) = (0x054c, 0x09cc);
const NINTENDO_SWITCH_PRO: (u16, u16) = (0x057e, 0x2009);
const MICROSOFT_XBOX_ONE: (u16, u16) = (0x045e, 0x02ea);
const VALVE_STEAM_DECK: (u16, u16) = (0x28de, 0x1205);
const UNKNOWN_MAKER: (u16, u16) = (0x1234, 0x5678);

#[allow(unsafe_code, reason = "SDL's safe wrapper cannot attach a virtual gamepad")]
fn kind_of_virtual_pad(vendor_and_product: (u16, u16)) -> GamepadKind {
    let context = sdl3::init().expect("SDL should start");
    let gamepads = context.gamepad().expect("the gamepad subsystem should start");
    let mut description = SDL_VirtualJoystickDesc::new();
    description.r#type = u16::try_from(SDL_JOYSTICK_TYPE_GAMEPAD.0).expect("a joystick type fits 16 bits");
    (description.vendor_id, description.product_id) = vendor_and_product;
    description.naxes = VIRTUAL_AXES;
    description.nbuttons = VIRTUAL_BUTTONS;
    description.axis_mask = ALL_BITS;
    description.button_mask = ALL_BITS;
    description.name = VIRTUAL_PAD_NAME.as_ptr();
    // SAFETY: the description and its name outlive the call.
    let identifier = unsafe { SDL_AttachVirtualJoystick(&raw const description) };
    assert_ne!(identifier.0, 0, "SDL should attach the virtual pad");
    let kind = kind_of(
        &gamepads
            .open(identifier.into())
            .expect("the virtual pad should open as a gamepad"),
    );
    // SAFETY: the pad was attached above and its handle is dropped.
    assert!(unsafe { SDL_DetachVirtualJoystick(identifier) });
    kind
}

#[test]
#[ignore = "starts SDL's joystick subsystem; run with --ignored"]
fn virtual_pads_get_the_hints_of_their_make() {
    for (vendor_and_product, expected) in [
        (SONY_DUALSENSE, GamepadKind::PlayStation),
        (SONY_DUALSHOCK_4, GamepadKind::PlayStation),
        (NINTENDO_SWITCH_PRO, GamepadKind::Switch),
        (MICROSOFT_XBOX_ONE, GamepadKind::Xbox),
        (VALVE_STEAM_DECK, GamepadKind::SteamDeck),
        (UNKNOWN_MAKER, GamepadKind::Xbox),
    ] {
        assert_eq!(
            kind_of_virtual_pad(vendor_and_product),
            expected,
            "{vendor_and_product:x?}"
        );
    }
}

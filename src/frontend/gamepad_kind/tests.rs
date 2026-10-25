use super::*;

const VENDOR_MICROSOFT: u16 = 0x045e;

fn identity(reported_make: ReportedMake, vendor_id: Option<u16>, product_id: Option<u16>) -> Identity {
    Identity {
        reported_make,
        vendor_id,
        product_id,
    }
}

#[test]
fn the_reported_make_decides() {
    assert_eq!(
        gamepad_kind(identity(ReportedMake::Xbox, None, None)),
        GamepadKind::Xbox
    );
    assert_eq!(
        gamepad_kind(identity(ReportedMake::PlayStation, None, None)),
        GamepadKind::PlayStation
    );
    assert_eq!(
        gamepad_kind(identity(ReportedMake::Switch, None, None)),
        GamepadKind::Switch
    );
}

#[test]
fn the_vendor_decides_when_the_make_is_unknown() {
    let by_vendor = |vendor| gamepad_kind(identity(ReportedMake::Other, Some(vendor), None));
    assert_eq!(by_vendor(VENDOR_SONY), GamepadKind::PlayStation);
    assert_eq!(by_vendor(VENDOR_NINTENDO), GamepadKind::Switch);
    assert_eq!(by_vendor(VENDOR_MICROSOFT), GamepadKind::Xbox);
}

#[test]
fn an_unidentifiable_pad_gets_the_standard_layout_names() {
    assert_eq!(
        gamepad_kind(identity(ReportedMake::Other, Some(0x1234), Some(0x5678))),
        GamepadKind::Xbox
    );
    assert_eq!(
        gamepad_kind(identity(ReportedMake::Other, None, None)),
        GamepadKind::Xbox
    );
}

#[test]
fn the_steam_decks_own_controls_are_recognised_even_when_called_a_standard_pad() {
    for make in [ReportedMake::Other, ReportedMake::Xbox] {
        let deck = identity(make, Some(VENDOR_VALVE), Some(PRODUCT_STEAM_DECK));
        assert_eq!(gamepad_kind(deck), GamepadKind::SteamDeck);
    }
}

#[test]
fn other_valve_devices_are_not_a_deck() {
    // The virtual pad Steam makes when it has no details about the real one.
    let virtual_pad = identity(ReportedMake::Xbox, Some(VENDOR_VALVE), Some(0x11ff));
    assert_eq!(gamepad_kind(virtual_pad), GamepadKind::Xbox);
}

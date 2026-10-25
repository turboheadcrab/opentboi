//! Which make a gamepad is, so the on-screen hints can show the names printed on its buttons.
//! Frontends read what the platform reports and translate it into [`Identity`].
//!
//! Under Steam Input the game sees a virtual pad, but SDL fills in the real controller's make,
//! vendor and product from Steam, so the same rules tell a Steam Deck's own controls from a
//! PlayStation pad plugged into it.

use crate::game::GamepadKind;

const VENDOR_SONY: u16 = 0x054c;
const VENDOR_NINTENDO: u16 = 0x057e;
const VENDOR_VALVE: u16 = 0x28de;
/// The Steam Deck's built-in controls (SDL calls them the Neptune controller).
const PRODUCT_STEAM_DECK: u16 = 0x1205;

/// The make a platform's gamepad layer claims for a pad; every make this game has pictures for,
/// and the rest.
#[cfg_attr(
    all(not(feature = "sdl3-frontend"), not(test)),
    expect(dead_code, reason = "only SDL reports a make; the Bevy frontend goes by the ids")
)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReportedMake {
    Xbox,
    PlayStation,
    Switch,
    /// Unknown, or a plain "standard" pad.
    Other,
}

/// Everything a platform reports that says what a pad is; ids are `None` when it does not say.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Identity {
    pub reported_make: ReportedMake,
    pub vendor_id: Option<u16>,
    pub product_id: Option<u16>,
}

/// The ids first for the Deck, which platforms call a standard or Xbox-like pad; then the make the
/// platform reports; then the vendor; Xbox when nothing says otherwise (SDL's standard layout is
/// named after it).
pub fn gamepad_kind(identity: Identity) -> GamepadKind {
    if identity.vendor_id == Some(VENDOR_VALVE) && identity.product_id == Some(PRODUCT_STEAM_DECK) {
        return GamepadKind::SteamDeck;
    }
    match (identity.reported_make, identity.vendor_id) {
        (ReportedMake::PlayStation, _) | (ReportedMake::Other, Some(VENDOR_SONY)) => GamepadKind::PlayStation,
        (ReportedMake::Switch, _) | (ReportedMake::Other, Some(VENDOR_NINTENDO)) => GamepadKind::Switch,
        (ReportedMake::Xbox | ReportedMake::Other, _) => GamepadKind::Xbox,
    }
}

#[cfg(test)]
mod tests;

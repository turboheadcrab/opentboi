//! Parsing option values.

use super::{
    Duration, FrontendKind, GAMEPAD_HINT_NAMES, GamepadKind, InputDevice, InputSampling, KEYBOARD_HINTS, LATCHED_INPUT,
    MINIMUM_SPLASH, ORIGINAL_SPLASH, ORIGINAL_STYLE, REBIRTH_STYLE, SIZE_SEPARATOR, SplashTime, Style,
    WINDOW_SIZE_PRESETS, WindowSize,
};

pub(super) fn value_of(flag: &str, arguments: &mut impl Iterator<Item = String>) -> Result<String, String> {
    arguments.next().ok_or_else(|| format!("{flag} needs a value"))
}

/// `original` or `rebirth`.
pub(super) fn parse_style(flag: &str, arguments: &mut impl Iterator<Item = String>) -> Result<Style, String> {
    match value_of(flag, arguments)?.as_str() {
        ORIGINAL_STYLE => Ok(Style::Original),
        REBIRTH_STYLE => Ok(Style::Rebirth),
        other => Err(format!(
            "unknown style {other}; use {ORIGINAL_STYLE} or {REBIRTH_STYLE}"
        )),
    }
}

pub(super) fn parse_whole_number(flag: &str, arguments: &mut impl Iterator<Item = String>) -> Result<u32, String> {
    let value = value_of(flag, arguments)?;
    value
        .parse()
        .map_err(|_| format!("invalid {flag} {value}; give a whole number"))
}

pub(super) fn parse_frontend(value: &str) -> Result<FrontendKind, String> {
    FrontendKind::NAMES
        .iter()
        .find(|(name, _)| *name == value)
        .map(|(_, kind)| *kind)
        .ok_or_else(|| format!("unknown frontend {value}; use sdl3 or bevy"))
}

pub(super) fn parse_duration(value: &str) -> Result<Duration, String> {
    value
        .parse::<f64>()
        .ok()
        .and_then(|seconds| Duration::try_from_secs_f64(seconds).ok())
        .ok_or_else(|| format!("invalid duration {value}; give seconds, e.g. 10 or 2.5"))
}

pub(super) fn parse_window_size(value: &str) -> Result<WindowSize, String> {
    if let Some((_, preset)) = WINDOW_SIZE_PRESETS.iter().find(|(name, _)| *name == value) {
        return Ok(*preset);
    }
    let invalid = || format!("invalid window size {value}; use small, medium, large or WIDTH{SIZE_SEPARATOR}HEIGHT");
    let (width, height) = value.split_once(SIZE_SEPARATOR).ok_or_else(invalid)?;
    let width: u32 = width.parse().map_err(|_| invalid())?;
    let height: u32 = height.parse().map_err(|_| invalid())?;
    if width == 0 || height == 0 {
        return Err(invalid());
    }
    Ok(WindowSize::new(width, height))
}

/// `original` or `minimum`.
pub(super) fn parse_splash(value: &str) -> Result<SplashTime, String> {
    match value {
        ORIGINAL_SPLASH => Ok(SplashTime::Original),
        MINIMUM_SPLASH => Ok(SplashTime::Minimum),
        other => Err(format!(
            "unknown splash time {other}; use {ORIGINAL_SPLASH} or {MINIMUM_SPLASH}"
        )),
    }
}

/// `keyboard`, or the make of a gamepad (`xbox`, `playstation`, `switch`, `steam-deck`).
pub(super) fn parse_input_hints(value: &str) -> Result<InputDevice, String> {
    if value == KEYBOARD_HINTS {
        return Ok(InputDevice::KeyboardAndMouse);
    }
    GAMEPAD_HINT_NAMES
        .iter()
        .zip(GamepadKind::ALL)
        .find(|(name, _)| **name == value)
        .map(|(_, kind)| InputDevice::Gamepad(kind))
        .ok_or_else(|| {
            format!(
                "unknown hints {value}; use {KEYBOARD_HINTS}, {}",
                GAMEPAD_HINT_NAMES.join(", ")
            )
        })
}

/// `original` or `latched`.
pub(super) fn parse_input_sampling(value: &str) -> Result<InputSampling, String> {
    match value {
        ORIGINAL_STYLE => Ok(InputSampling::Original),
        LATCHED_INPUT => Ok(InputSampling::Latched),
        other => Err(format!(
            "unknown input sampling {other}; use {ORIGINAL_STYLE} or {LATCHED_INPUT}"
        )),
    }
}

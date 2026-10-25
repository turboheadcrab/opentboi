//! Command-line options. Engine-independent; the frontend receives the parsed [`DisplaySettings`].

use std::fmt;
use std::time::Duration;

use crate::game::{DEFAULT_TICK_RATE, GamepadKind, InputDevice, InputSampling, Style};

const HELP_FLAG: &str = "--help";
const WINDOW_SIZE_FLAG: &str = "--window-size";
const FULLSCREEN_FLAG: &str = "--fullscreen";
const FRONTEND_FLAG: &str = "--frontend";
const NO_VSYNC_FLAG: &str = "--no-vsync";
const EXIT_AFTER_FLAG: &str = "--exit-after";
const INTRO_FLAG: &str = "--intro";
const SEED_FLAG: &str = "--seed";
const INSTANT_ROOMS_FLAG: &str = "--instant-room-changes";
const NO_INTERPOLATION_FLAG: &str = "--no-interpolation";
const NO_STATS_FLAG: &str = "--no-stats";
const SHOWCASE_FLAG: &str = "--showcase";
const INPUT_FLAG: &str = "--input";
const LATCHED_INPUT: &str = "latched";
const HINTS_FLAG: &str = "--hints";
const KEYBOARD_HINTS: &str = "keyboard";
/// The `--hints` names of the gamepad makes, in the order of `GamepadKind::ALL`.
const GAMEPAD_HINT_NAMES: [&str; 4] = ["xbox", "playstation", "switch", "steam-deck"];
const SCREENSHOT_FLAG: &str = "--screenshot";
const SECRET_ENTRANCES_FLAG: &str = "--secret-entrances";
const TICK_RATE_FLAG: &str = "--tick-rate";
const CURSE_SECRET_DOOR_FLAG: &str = "--curse-secret-door";
const HUD_FLAG: &str = "--hud";
const SPLASH_FLAG: &str = "--splash";
const ORIGINAL_SPLASH: &str = "original";
const MINIMUM_SPLASH: &str = "minimum";
const ORIGINAL_STYLE: &str = "original";
const REBIRTH_STYLE: &str = "rebirth";
const SIZE_SEPARATOR: char = 'x';

/// The original's window sizes (client area; its windows added a 30–32 px title bar).
const WINDOW_SIZE_PRESETS: [(&str, WindowSize); 3] = [
    ("small", WindowSize::new(640, 480)),
    ("medium", WindowSize::new(800, 600)),
    ("large", WindowSize::new(1024, 768)),
];
/// The original selected "large" by default.
const DEFAULT_WINDOW_SIZE: WindowSize = WindowSize::new(1024, 768);
/// Resolutions the original offered for fullscreen; listed in `--help` as known-good window sizes.
pub const ORIGINAL_FULLSCREEN_RESOLUTIONS: [WindowSize; 6] = [
    WindowSize::new(800, 600),
    WindowSize::new(720, 480),
    WindowSize::new(640, 480),
    WindowSize::new(1280, 800),
    WindowSize::new(1280, 720),
    WindowSize::new(1024, 768),
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WindowSize {
    pub width: u32,
    pub height: u32,
}

impl WindowSize {
    pub const fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }
}

impl fmt::Display for WindowSize {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}{SIZE_SEPARATOR}{}", self.width, self.height)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DisplaySettings {
    pub window_size: WindowSize,
    pub start_fullscreen: bool,
    /// Wait for the display's refresh; off draws as fast as possible (for benchmarks).
    pub vsync: bool,
    pub splash: SplashTime,
}

/// How long the splash stays up while loading.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SplashTime {
    /// At least the 3 s Zinc's projector showed it.
    Original,
    /// Only until loading is done.
    Minimum,
}

/// The engine layer that opens the window, draws and plays sound.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrontendKind {
    Sdl3,
    Bevy,
}

impl FrontendKind {
    const NAMES: [(&str, Self); 2] = [("sdl3", Self::Sdl3), ("bevy", Self::Bevy)];

    pub fn name(self) -> &'static str {
        match self {
            Self::Sdl3 => Self::NAMES[0].0,
            Self::Bevy => Self::NAMES[1].0,
        }
    }
}

impl fmt::Display for FrontendKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.name())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[allow(clippy::struct_excessive_bools, reason = "independent command-line switches")]
pub struct LaunchSettings {
    pub display: DisplaySettings,
    /// Play the original's intro (author and title cards) before the game; Space skips it.
    pub play_intro: bool,
    /// `None` uses the first frontend compiled in, preferring SDL3.
    pub frontend: Option<FrontendKind>,
    /// Switch rooms at once instead of sliding like the original (handy while debugging).
    pub instant_room_changes: bool,
    /// Draw between ticks for smooth motion; `--no-interpolation` turns it off.
    pub interpolation: bool,
    /// Show Rebirth's stats panel, for debugging (on for now; `--no-stats`).
    pub show_stats: bool,
    /// Put every pickup, machine and card into the floor for checking them (`--showcase`).
    pub showcase: bool,
    /// How presses between ticks count (`--input original|latched`).
    pub input_sampling: InputSampling,
    /// Whose button names the hints show until real input arrives (`--hints`).
    pub input_hints: InputDevice,
    /// Bombed-open secret entrances close behind the player (original) or stay open (Rebirth).
    pub secret_entrances: Style,
    /// A secret room next to the curse room has a spiked curse door (original) or a wall to bomb
    /// (Rebirth).
    pub curse_secret_door: Style,
    /// The HUD's look (only the original's exists so far).
    pub hud: Style,
    /// Save a frame (BMP) a second into the game, for visual comparisons (SDL3 frontend).
    pub screenshot_path: Option<std::path::PathBuf>,
    /// Fixes the generated floor (otherwise a new one every run).
    pub seed: Option<u32>,
    /// Simulation ticks per second; the original ran 30.
    pub tick_rate: u32,
    /// Quit by itself after this long (for benchmarks).
    pub exit_after: Option<Duration>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum LaunchRequest {
    Run(LaunchSettings),
    ShowHelp,
}

impl Default for LaunchSettings {
    fn default() -> Self {
        Self {
            display: DisplaySettings {
                window_size: DEFAULT_WINDOW_SIZE,
                start_fullscreen: false,
                vsync: true,
                splash: SplashTime::Minimum,
            },
            play_intro: false,
            frontend: None,
            instant_room_changes: false,
            interpolation: true,
            show_stats: true,
            showcase: false,
            input_sampling: InputSampling::Original,
            input_hints: InputDevice::KeyboardAndMouse,
            screenshot_path: None,
            secret_entrances: Style::Original,
            curse_secret_door: Style::Original,
            hud: Style::Original,
            seed: None,
            tick_rate: DEFAULT_TICK_RATE,
            exit_after: None,
        }
    }
}

pub fn parse_arguments(arguments: impl IntoIterator<Item = String>) -> Result<LaunchRequest, String> {
    let mut settings = LaunchSettings::default();
    let mut arguments = arguments.into_iter();
    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            HELP_FLAG => return Ok(LaunchRequest::ShowHelp),
            FULLSCREEN_FLAG => settings.display.start_fullscreen = true,
            NO_VSYNC_FLAG => settings.display.vsync = false,
            INTRO_FLAG => settings.play_intro = true,
            INSTANT_ROOMS_FLAG => settings.instant_room_changes = true,
            NO_INTERPOLATION_FLAG => settings.interpolation = false,
            NO_STATS_FLAG => settings.show_stats = false,
            SHOWCASE_FLAG => settings.showcase = true,
            INPUT_FLAG => settings.input_sampling = parse_input_sampling(&value_of(INPUT_FLAG, &mut arguments)?)?,
            HINTS_FLAG => settings.input_hints = parse_input_hints(&value_of(HINTS_FLAG, &mut arguments)?)?,
            SECRET_ENTRANCES_FLAG => settings.secret_entrances = parse_style(SECRET_ENTRANCES_FLAG, &mut arguments)?,
            CURSE_SECRET_DOOR_FLAG => settings.curse_secret_door = parse_style(CURSE_SECRET_DOOR_FLAG, &mut arguments)?,
            HUD_FLAG => settings.hud = parse_style(HUD_FLAG, &mut arguments)?,
            SPLASH_FLAG => settings.display.splash = parse_splash(&value_of(SPLASH_FLAG, &mut arguments)?)?,
            SCREENSHOT_FLAG => settings.screenshot_path = Some(value_of(SCREENSHOT_FLAG, &mut arguments)?.into()),
            SEED_FLAG => settings.seed = Some(parse_whole_number(SEED_FLAG, &mut arguments)?),
            TICK_RATE_FLAG => settings.tick_rate = parse_whole_number(TICK_RATE_FLAG, &mut arguments)?.max(1),
            WINDOW_SIZE_FLAG => {
                settings.display.window_size = parse_window_size(&value_of(WINDOW_SIZE_FLAG, &mut arguments)?)?;
            }
            FRONTEND_FLAG => settings.frontend = Some(parse_frontend(&value_of(FRONTEND_FLAG, &mut arguments)?)?),
            EXIT_AFTER_FLAG => settings.exit_after = Some(parse_duration(&value_of(EXIT_AFTER_FLAG, &mut arguments)?)?),
            unknown => return Err(format!("unknown option {unknown}; see {HELP_FLAG}")),
        }
    }
    Ok(LaunchRequest::Run(settings))
}

mod help;
mod values;

pub use help::help_text;
use values::{
    parse_duration, parse_frontend, parse_input_hints, parse_input_sampling, parse_splash, parse_style,
    parse_whole_number, parse_window_size, value_of,
};

#[cfg(test)]
mod tests;

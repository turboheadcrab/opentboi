//! Frontends: the only code that depends on an engine or platform layer. Each one does the same
//! job (see `crate::game` for the contract): open the window, turn devices into input, step the
//! game, draw its list and play its sounds. They are compiled in by Cargo features, so a build
//! with one frontend carries none of the other's dependencies.

#[cfg(feature = "bevy-frontend")]
mod bevy_frontend;
mod controls;
mod cursor_visibility;
mod frame_statistics;
mod gamepad_kind;
#[cfg(feature = "sdl3-frontend")]
mod sdl3_frontend;
mod sound_variants;

#[cfg(not(any(feature = "bevy-frontend", feature = "sdl3-frontend")))]
compile_error!("enable the `sdl3-frontend` or `bevy-frontend` feature");

use std::process::ExitCode;
use std::time::{Duration, Instant};

use crate::game::GameParameters;
use crate::launch_options::{DisplaySettings, FrontendKind};

pub mod texture_prefetch;
pub use frame_statistics::FrameStatistics;

pub const WINDOW_TITLE: &str = "OpenTBOI";
/// Wayland `app_id` / X11 `WM_CLASS` of each frontend's window; each must equal `StartupWMClass`
/// in its desktop launcher, so the desktop shows the right launcher's icon and name.
#[cfg(feature = "sdl3-frontend")]
pub const SDL3_APPLICATION_ID: &str = "opentboi";
#[cfg(feature = "bevy-frontend")]
pub const BEVY_APPLICATION_ID: &str = "opentboi-bevy";

/// Everything a frontend needs besides the game and its assets.
pub struct FrontendLaunch {
    pub display: DisplaySettings,
    pub exit_after: Option<Duration>,
    /// `--screenshot PATH` (SDL3 frontend): save a frame a second into the game.
    #[cfg_attr(
        not(feature = "sdl3-frontend"),
        expect(dead_code, reason = "only the SDL3 frontend takes screenshots")
    )]
    pub screenshot_path: Option<std::path::PathBuf>,
    pub game_settings: crate::game::GameSettings,
    /// Picks the floor layout (`--seed`, otherwise the clock).
    pub seed: u32,
    /// When `main` started, for reporting time to the first frame.
    pub process_started: Instant,
    /// Frontends with a loading screen play the original's intro after the splash (`--intro`).
    #[cfg_attr(
        not(feature = "sdl3-frontend"),
        expect(dead_code, reason = "only the SDL3 frontend plays the intro")
    )]
    pub play_intro: bool,
}

/// The frontend used when none is requested: SDL3 if it is compiled in.
pub const fn default_kind() -> FrontendKind {
    if cfg!(feature = "sdl3-frontend") {
        FrontendKind::Sdl3
    } else {
        FrontendKind::Bevy
    }
}

/// Each frontend loads the game's assets itself (see `crate::loading`), so it decides what to show
/// meanwhile.
#[cfg_attr(
    not(feature = "bevy-frontend"),
    allow(
        clippy::needless_pass_by_value,
        reason = "only the Bevy frontend keeps the launch settings"
    )
)]
pub fn run(kind: FrontendKind, parameters: &GameParameters, launch: FrontendLaunch) -> ExitCode {
    match kind {
        #[cfg(feature = "sdl3-frontend")]
        FrontendKind::Sdl3 => sdl3_frontend::run(parameters, &launch),
        #[cfg(feature = "bevy-frontend")]
        FrontendKind::Bevy => bevy_frontend::run(parameters, launch),
        #[allow(unreachable_patterns, reason = "reachable when only one frontend is compiled in")]
        missing => {
            eprintln!("The {missing} frontend is not compiled in; rebuild with --features {missing}-frontend");
            ExitCode::FAILURE
        }
    }
}

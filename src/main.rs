mod frontend;
mod game;
mod game_assets;
mod launch_options;
mod loading;
mod whole_numbers;

use std::process::ExitCode;
use std::time::Instant;

use game::{ParameterProvider, TranscribedOriginal};
use launch_options::LaunchRequest;

fn main() -> ExitCode {
    let process_started = Instant::now();
    let launch_settings = match launch_options::parse_arguments(std::env::args().skip(1)) {
        Ok(LaunchRequest::Run(launch_settings)) => launch_settings,
        Ok(LaunchRequest::ShowHelp) => {
            println!("{}", launch_options::help_text());
            return ExitCode::SUCCESS;
        }
        Err(message) => {
            eprintln!("{message}");
            return ExitCode::FAILURE;
        }
    };
    let parameter_provider = TranscribedOriginal;
    println!("Gameplay values: {:?}", parameter_provider.source());
    if launch_settings.hud == game::Style::Rebirth {
        println!("The Rebirth HUD is not made yet; showing the original's.");
    }
    let frontend_kind = launch_settings.frontend.unwrap_or_else(frontend::default_kind);
    println!("Frontend: {frontend_kind}");
    let launch = frontend::FrontendLaunch {
        display: launch_settings.display,
        exit_after: launch_settings.exit_after,
        process_started,
        screenshot_path: launch_settings.screenshot_path.clone(),
        game_settings: game::GameSettings {
            slide_between_rooms: !launch_settings.instant_room_changes,
            secret_entrances: launch_settings.secret_entrances,
            curse_secret_door: launch_settings.curse_secret_door,
            hud: launch_settings.hud,
            ticks_per_second: launch_settings.tick_rate,
            interpolation: launch_settings.interpolation,
            show_stats: launch_settings.show_stats,
            showcase: launch_settings.showcase,
            input_sampling: launch_settings.input_sampling,
            initial_input_device: launch_settings.input_hints,
        },
        seed: launch_settings.seed.unwrap_or_else(clock_seed),
        play_intro: launch_settings.play_intro,
    };
    frontend::run(frontend_kind, &parameter_provider.parameters(), launch)
}

/// A different floor every run unless `--seed` asks for a specific one.
fn clock_seed() -> u32 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(1, |since_epoch| {
            // Only the low bits matter for a seed.
            since_epoch.subsec_nanos() ^ u32::try_from(since_epoch.as_secs() & u64::from(u32::MAX)).unwrap_or_default()
        })
}

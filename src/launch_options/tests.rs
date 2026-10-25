use super::*;

fn parse(arguments: &[&str]) -> Result<LaunchRequest, String> {
    parse_arguments(arguments.iter().map(ToString::to_string))
}

#[test]
fn defaults_to_the_original_large_window() {
    assert_eq!(
        parse(&[]),
        Ok(LaunchRequest::Run(LaunchSettings {
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
        }))
    );
}

#[test]
fn the_tick_rate_can_be_chosen() {
    let Ok(LaunchRequest::Run(settings)) = parse(&["--tick-rate", "30"]) else {
        panic!("should parse");
    };
    assert_eq!(settings.tick_rate, 30);
    assert!(parse(&["--tick-rate", "fast"]).is_err());
}

#[test]
fn accepts_presets_and_every_original_resolution() {
    for (name, size) in WINDOW_SIZE_PRESETS {
        assert_eq!(parse_window_size(name), Ok(size));
    }
    for resolution in ORIGINAL_FULLSCREEN_RESOLUTIONS {
        assert_eq!(parse_window_size(&resolution.to_string()), Ok(resolution));
    }
}

#[test]
fn rejects_malformed_sizes_and_unknown_options() {
    assert!(parse_window_size("0x600").is_err());
    assert!(parse_window_size("huge").is_err());
    assert!(parse(&[WINDOW_SIZE_FLAG]).is_err());
    assert!(parse(&["--bogus"]).is_err());
}

#[test]
fn fullscreen_and_help_flags() {
    let Ok(LaunchRequest::Run(settings)) = parse(&[FULLSCREEN_FLAG]) else {
        panic!("fullscreen flag should parse");
    };
    assert!(settings.display.start_fullscreen);
    assert_eq!(parse(&[HELP_FLAG]), Ok(LaunchRequest::ShowHelp));
}

#[test]
fn benchmark_options() {
    let Ok(LaunchRequest::Run(settings)) = parse(&[FRONTEND_FLAG, "bevy", EXIT_AFTER_FLAG, "2.5", NO_VSYNC_FLAG])
    else {
        panic!("benchmark options should parse");
    };
    assert_eq!(settings.frontend, Some(FrontendKind::Bevy));
    assert_eq!(settings.exit_after, Some(Duration::from_millis(2500)));
    assert!(!settings.display.vsync);
    assert!(parse(&[FRONTEND_FLAG, "flash"]).is_err());
    assert!(parse(&[EXIT_AFTER_FLAG, "-1"]).is_err());
}

#[test]
fn hints_name_a_device() {
    let hints = |value: &str| match parse(&[HINTS_FLAG, value]) {
        Ok(LaunchRequest::Run(settings)) => Ok(settings.input_hints),
        Ok(LaunchRequest::ShowHelp) => panic!("not the help"),
        Err(message) => Err(message),
    };
    assert_eq!(hints(KEYBOARD_HINTS), Ok(InputDevice::KeyboardAndMouse));
    for (name, kind) in GAMEPAD_HINT_NAMES.into_iter().zip(GamepadKind::ALL) {
        assert_eq!(hints(name), Ok(InputDevice::Gamepad(kind)));
    }
    assert_eq!(GAMEPAD_HINT_NAMES.len(), GamepadKind::ALL.len());
    assert!(hints("atari").is_err());
}

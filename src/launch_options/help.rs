//! The `--help` text.

use super::{
    CURSE_SECRET_DOOR_FLAG, DEFAULT_TICK_RATE, DEFAULT_WINDOW_SIZE, EXIT_AFTER_FLAG, FRONTEND_FLAG, FULLSCREEN_FLAG,
    GAMEPAD_HINT_NAMES, HELP_FLAG, HINTS_FLAG, HUD_FLAG, INPUT_FLAG, INSTANT_ROOMS_FLAG, INTRO_FLAG, KEYBOARD_HINTS,
    LATCHED_INPUT, MINIMUM_SPLASH, NO_INTERPOLATION_FLAG, NO_STATS_FLAG, NO_VSYNC_FLAG,
    ORIGINAL_FULLSCREEN_RESOLUTIONS, ORIGINAL_SPLASH, ORIGINAL_STYLE, REBIRTH_STYLE, SCREENSHOT_FLAG,
    SECRET_ENTRANCES_FLAG, SEED_FLAG, SHOWCASE_FLAG, SIZE_SEPARATOR, SPLASH_FLAG, TICK_RATE_FLAG, WINDOW_SIZE_FLAG,
    WINDOW_SIZE_PRESETS,
};

pub fn help_text() -> String {
    let presets: Vec<String> = WINDOW_SIZE_PRESETS
        .iter()
        .map(|(name, size)| format!("{name} ({size})"))
        .collect();
    let originals: Vec<String> = ORIGINAL_FULLSCREEN_RESOLUTIONS
        .iter()
        .map(ToString::to_string)
        .collect();
    format!(
        "Options:\n  {WINDOW_SIZE_FLAG} SIZE   {} or WIDTH{SIZE_SEPARATOR}HEIGHT (default {DEFAULT_WINDOW_SIZE})\n  \
         {FULLSCREEN_FLAG}         start in fullscreen (F11 toggles)\n  \
         {FRONTEND_FLAG} NAME     sdl3 or bevy, if both are compiled in (default sdl3)\n  \
         {NO_VSYNC_FLAG}           draw as fast as possible (benchmarks)\n  \
         {INTRO_FLAG}              play the original's intro first (Space skips it)\n  \
         {SEED_FLAG} NUMBER        play the floor this number generates (default: a new one)\n  \
         {TICK_RATE_FLAG} NUMBER   game logic steps per second (default {DEFAULT_TICK_RATE}; the original ran 30)\n  \
         {INSTANT_ROOMS_FLAG} switch rooms at once instead of sliding\n  \
         {NO_INTERPOLATION_FLAG}  draw whole ticks only, without smoothing between them\n  \
         {NO_STATS_FLAG}          hide the stats panel on the left (debugging)\n  \
         {SHOWCASE_FLAG}          every pickup, machine and card in the boss room, shop and treasure room (debugging)\n  \
         {INPUT_FLAG} SAMPLING  {ORIGINAL_STYLE} (default: buttons as they are at each step, like the original) or {LATCHED_INPUT} (a tap between steps still counts)\n  \
         {HINTS_FLAG} DEVICE     {KEYBOARD_HINTS} (default) or a gamepad make ({}): whose button names the hints show until real input arrives (checking pictures)\n  \
         {SCREENSHOT_FLAG} PATH    save a BMP screenshot a second into the game\n  \
         {SECRET_ENTRANCES_FLAG} STYLE  {ORIGINAL_STYLE} (default: close after leaving) or {REBIRTH_STYLE} (stay open)\n  \
         {CURSE_SECRET_DOOR_FLAG} STYLE  {ORIGINAL_STYLE} (default: a spiked curse door into a secret room next to it) or {REBIRTH_STYLE} (bomb it; the hole does not hurt)\n  \
         {HUD_FLAG} STYLE      {ORIGINAL_STYLE} (default) or {REBIRTH_STYLE} (not made yet: shows the original's)\n  \
         {SPLASH_FLAG} TIME     {MINIMUM_SPLASH} (default: until loaded) or {ORIGINAL_SPLASH} (at least 3 s, like the original)\n  \
         {EXIT_AFTER_FLAG} SECONDS quit after this long and print frame statistics\n  \
         {HELP_FLAG}               show this text\n\
         The original's fullscreen resolutions: {}",
        presets.join(", "),
        GAMEPAD_HINT_NAMES.join(", "),
        originals.join(", ")
    )
}

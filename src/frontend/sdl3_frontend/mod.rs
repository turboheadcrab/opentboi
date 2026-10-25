//! SDL3 frontend (`sdl3-frontend` feature): a plain loop over SDL's window, 2D renderer, gamepad
//! and audio-stream APIs. The only module that depends on SDL. It shows the original's splash
//! while the game loads on other threads, so play starts with every texture ready.

mod audio;
mod device_input;
mod graphics;
mod intro_player;
mod session;
mod window;

use crate::frontend::sdl3_frontend::graphics::vector_families::is_drawn_as_vectors;
use std::process::ExitCode;

use sdl3::VideoSubsystem;
use sdl3::render::WindowCanvas;

use crate::frontend::{FrontendLaunch, SDL3_APPLICATION_ID, WINDOW_TITLE};
use glam::Vec2;

use crate::game::{self, GameParameters};
use crate::launch_options::DisplaySettings;
use crate::loading;
use audio::sound_player::SoundPlayer;
use session::Session;
use window::pixel_size::points_for_pixels;
use window::splash_window::SplashWindow;

/// Sets the Wayland `app_id` / X11 `WM_CLASS` before SDL creates a window.
const APPLICATION_ID_HINT: &str = "SDL_APP_ID";
/// Renderer preference, first available wins; the `SDL_RENDER_DRIVER` environment variable still
/// overrides it. Vulkan paced frames evenly in our measurements, while SDL's default (OpenGL)
/// occasionally dropped to ~100 fps on Wayland (docs/frontend-comparison.md).
const RENDER_DRIVER_HINT: &str = "SDL_RENDER_DRIVER";
const PREFERRED_RENDER_DRIVERS: &str = "vulkan,gpu,opengl";

pub fn run(parameters: &GameParameters, launch: &FrontendLaunch) -> ExitCode {
    match run_until_quit(parameters, launch) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("SDL3 frontend failed: {message}");
            // Started from a launcher, there is no terminal to read: say it in a window too.
            let _ = sdl3::messagebox::show_simple_message_box(
                sdl3::messagebox::MessageBoxFlag::ERROR,
                crate::frontend::WINDOW_TITLE,
                &message,
                None,
            );
            ExitCode::FAILURE
        }
    }
}

fn run_until_quit(parameters: &GameParameters, launch: &FrontendLaunch) -> Result<(), String> {
    // Loading starts first, so the SWF is parsed while SDL opens the window.
    let (load_sender, load_events) = std::sync::mpsc::channel();
    let requested_pixels = Vec2::new(
        launch.display.window_size.width as f32,
        launch.display.window_size.height as f32,
    );
    // The requested window's density; corrected later if the window ends up elsewhere.
    let pixels_per_world_unit = game::pixels_per_world_unit(requested_pixels);
    loading::load_in_background(pixels_per_world_unit, is_drawn_as_vectors, load_sender);
    sdl3::hint::set(APPLICATION_ID_HINT, SDL3_APPLICATION_ID);
    sdl3::hint::set(RENDER_DRIVER_HINT, PREFERRED_RENDER_DRIVERS);
    let context = sdl3::init().map_err(|error| error.to_string())?;
    let video = context.video().map_err(|error| error.to_string())?;
    let splash = loading::splash_bitmap_path()
        .and_then(|bitmap_path| SplashWindow::show(&video, &bitmap_path, launch.display.splash));
    if splash.is_some() {
        println!("Splash shown after {} ms", launch.process_started.elapsed().as_millis());
    }
    // Behind a splash the game window is prepared hidden, so it appears ready to play.
    let mut canvas = open_window(&video, launch.display, splash.is_some())?;
    set_vsync(&canvas, launch.display.vsync);
    println!("SDL3 renderer: {}", canvas.renderer_name);
    let texture_creator = canvas.texture_creator();
    let mut session = Session::new(
        parameters,
        launch,
        &texture_creator,
        SoundPlayer::open(&context),
        context.event_pump().map_err(|error| error.to_string())?,
        load_events,
        pixels_per_world_unit,
    );
    if let Some(splash) = splash {
        let keep_running = session.wait_behind_splash(&context, &splash)?;
        drop(splash);
        if !keep_running {
            println!("{}", session.report());
            return Ok(());
        }
        canvas.window_mut().show();
    }
    session.run_frames(&context, &mut canvas)?;
    println!("{}", session.report());
    Ok(())
}

/// Opens the window at the requested size in real pixels (see `pixel_size`), rendering at the
/// display's real pixel count; without high pixel density a scaled desktop would upscale a
/// smaller image and blur it.
fn open_window(video: &VideoSubsystem, display: DisplaySettings, hidden: bool) -> Result<WindowCanvas, String> {
    let mut builder = video.window(
        WINDOW_TITLE,
        points_for_pixels(video, display.window_size.width),
        points_for_pixels(video, display.window_size.height),
    );
    builder.position_centered().resizable().high_pixel_density();
    if hidden {
        builder.hidden();
    }
    let mut window = builder.build().map_err(|error| error.to_string())?;
    if display.start_fullscreen {
        window.set_fullscreen(true).map_err(|error| error.to_string())?;
    }
    Ok(window.into_canvas())
}

/// `SDL_SetRenderVSync`: 1 waits for every refresh, 0 presents immediately.
fn set_vsync(canvas: &WindowCanvas, vsync: bool) {
    let interval = i32::from(vsync);
    // SAFETY: the renderer pointer comes from a live canvas.
    let applied = unsafe { sdl3::sys::render::SDL_SetRenderVSync(canvas.raw(), interval) };
    if !applied {
        println!("Could not set vsync to {vsync}: {}", sdl3::get_error());
    }
}

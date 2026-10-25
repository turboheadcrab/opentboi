//! Bevy frontend (`bevy-frontend` feature): the only module that depends on Bevy.
// Bevy hands a system its parameters (`Res`, `Query`, `Single`) by value; borrowing them is not
// an option.
#![allow(clippy::needless_pass_by_value)]

mod device_input;
mod music_player;
mod sound_player;
mod sprite_renderer;
mod texture_library;
mod window_controls;

use std::process::ExitCode;
use std::sync::OnceLock;
use std::time::Instant;

use bevy::camera::ScalingMode;
use bevy::prelude::*;
use bevy::render::{Render, RenderApp, RenderSystems};
use bevy::window::{MonitorSelection, PresentMode, WindowMode, WindowResolution};

use super::{BEVY_APPLICATION_ID, FrameStatistics, FrontendLaunch, WINDOW_TITLE};
use crate::game::{self, Game, GameParameters, InputState};
use crate::launch_options::DisplaySettings;
use crate::loading;

#[derive(Resource)]
struct GameSimulation(Game);

#[derive(Resource, Default)]
struct CurrentInput(InputState);

/// Frame timing, printed when the app ends; `exit_after` ends it early for benchmarks.
#[derive(Resource)]
struct FrameReport {
    statistics: FrameStatistics,
    exit_after: Option<std::time::Duration>,
}

impl Drop for FrameReport {
    fn drop(&mut self) {
        if let Some(&presented_at) = FIRST_PRESENT.get() {
            self.statistics.record_first_present(presented_at);
        }
        println!("{}", self.statistics.report());
    }
}

/// When the render world finished its first frame. Bevy renders frame N while the game already
/// runs frame N + 1, so the main world cannot see this itself.
static FIRST_PRESENT: OnceLock<Instant> = OnceLock::new();

fn mark_first_present() {
    FIRST_PRESENT.get_or_init(Instant::now);
}

pub fn run(parameters: &GameParameters, launch: FrontendLaunch) -> ExitCode {
    let window_pixels = Vec2::new(
        launch.display.window_size.width as f32,
        launch.display.window_size.height as f32,
    );
    // Rendered for the requested window; resizing does not re-render yet.
    let (loaded, texture_supply) = loading::load_blocking(game::pixels_per_world_unit(window_pixels));
    let mut game = Game::new(loaded.room_appearance, parameters, &loaded.room_layouts, launch.seed);
    game.apply_settings(launch.game_settings);
    game.set_jingle_lengths(loaded.sound_bank.jingle_lengths());
    let mut app = App::new();
    app.insert_resource(ClearColor(Color::BLACK))
        .insert_resource(GameSimulation(game))
        .init_resource::<CurrentInput>()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(primary_window(launch.display)),
            ..default()
        }))
        .add_plugins((device_input::plugin, sprite_renderer::plugin, window_controls::plugin))
        .add_plugins(sound_player::SoundPlayerPlugin::new(&loaded.sound_bank))
        .add_plugins(music_player::MusicPlayerPlugin::new(&loaded.sound_bank))
        .insert_resource(FrameReport {
            statistics: FrameStatistics::new("bevy", launch.process_started),
            exit_after: launch.exit_after,
        })
        .add_systems(Last, record_frame)
        .add_systems(Startup, spawn_camera)
        .add_systems(Update, step_simulation.before(sprite_renderer::SyncSprites));
    app.sub_app_mut(RenderApp)
        .add_systems(Render, mark_first_present.in_set(RenderSystems::Cleanup));
    let texture_library = {
        let mut images = app.world_mut().resource_mut::<Assets<Image>>();
        texture_library::TextureLibrary::new(texture_supply, &mut images)
    };
    let exit = app.insert_resource(texture_library).run();
    if exit.is_success() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

/// The window size is in real pixels: overriding the scale factor to 1 makes Bevy ignore the
/// desktop's scale (a 1024×768 window stays 1024×768 pixels at 125 %), and it renders at the
/// window's pixel size either way.
fn primary_window(display: DisplaySettings) -> Window {
    Window {
        title: WINDOW_TITLE.into(),
        name: Some(BEVY_APPLICATION_ID.into()),
        resolution: WindowResolution::new(display.window_size.width, display.window_size.height)
            .with_scale_factor_override(1.0),
        mode: if display.start_fullscreen {
            WindowMode::BorderlessFullscreen(MonitorSelection::Current)
        } else {
            WindowMode::Windowed
        },
        present_mode: if display.vsync {
            PresentMode::AutoVsync
        } else {
            PresentMode::AutoNoVsync
        },
        ..default()
    }
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Transform::from_translation(game::VIEW_CENTER.extend(0.0)),
        Projection::from(OrthographicProjection {
            scaling_mode: ScalingMode::AutoMin {
                min_width: game::VIEW_SIZE.x,
                min_height: game::VIEW_SIZE.y,
            },
            ..OrthographicProjection::default_2d()
        }),
    ));
}

fn step_simulation(
    time: Res<Time>,
    input: Res<CurrentInput>,
    windows: Query<&Window>,
    mut simulation: ResMut<GameSimulation>,
) {
    simulation.0.update(&input.0, time.delta_secs());
    if let Ok(window) = windows.single() {
        let resolution = &window.resolution;
        let output_pixels = glam::Vec2::new(resolution.physical_width() as f32, resolution.physical_height() as f32);
        simulation.0.set_output_size(output_pixels);
    }
}

fn record_frame(mut commands: Commands, mut report: ResMut<FrameReport>) {
    report.statistics.record_frame();
    if report
        .exit_after
        .is_some_and(|duration| report.statistics.has_run_for(duration))
    {
        commands.write_message(AppExit::Success);
    }
}

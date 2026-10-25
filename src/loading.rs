//! Finds the game, extracts and parses its SWF, and prepares sounds and textures. Engine-
//! independent and thread-friendly: progress arrives as [`LoadEvent`]s, so a frontend can play the
//! intro while textures render on the other cores.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::mpsc::Sender;
use std::time::Instant;

use crate::game::{RoomAppearance, RoomArt, RoomLayouts, TextureId};
use crate::game_assets::{self, LoadedAssets, RgbaImage, SoundBank, TextureRenderer, TextureSupply};

/// Rendered first: whether it renders decides between artwork and placeholders.
const BASEMENT_BACKGROUND: TextureId = TextureId::RoomBackground(RoomArt::Basement);
/// Cores left free while textures render: one for the frontend's loop, one for the intro.
const CORES_KEPT_FREE: usize = 2;

pub enum LoadEvent {
    /// The SWF is parsed: the intro can be drawn and its music played.
    ArtReady {
        renderer: &'static TextureRenderer,
        #[cfg_attr(
            not(feature = "sdl3-frontend"),
            expect(dead_code, reason = "only the SDL3 frontend plays the intro")
        )]
        intro_music: Option<Arc<[u8]>>,
    },
    /// A texture rendered ahead of time.
    Texture(TextureId, RgbaImage),
    /// Loading is over; without a game install everything falls back to placeholders.
    Finished(Box<LoadedGame>),
    /// Loading broke off (the loader panicked); the message says why.
    #[cfg_attr(
        not(feature = "sdl3-frontend"),
        expect(dead_code, reason = "only the SDL3 frontend loads in the background")
    )]
    Failed(String),
}

/// What the game needs besides textures.
#[derive(Default)]
pub struct LoadedGame {
    pub sound_bank: SoundBank,
    pub room_appearance: RoomAppearance,
    /// The Basement's room layouts; empty rooms without them.
    pub room_layouts: RoomLayouts,
}

/// Runs the whole load, sending events as parts become ready; `Finished` always comes last.
/// Every texture renders up front (spread over the free cores) at `pixels_per_world_unit`, the
/// screen's density, so nothing renders mid-game; except what `drawn_as_vectors` says the
/// frontend draws as triangles, which needs no texture.
pub fn load(
    pixels_per_world_unit: f32,
    drawn_as_vectors: fn(&TextureRenderer, TextureId) -> bool,
    events: &Sender<LoadEvent>,
) {
    let started = Instant::now();
    let loaded = extracted_swf_path().and_then(|swf_path| {
        game_assets::load_assets(&swf_path)
            .inspect_err(|read_error| println!("Using placeholders, no sound: {read_error}"))
            .ok()
    });
    let Some(LoadedAssets {
        sound_bank,
        texture_renderer,
        intro_music,
    }) = loaded
    else {
        send(events, finished(SoundBank::default(), false, RoomLayouts::default()));
        return;
    };
    println!("Parsed the game SWF in {} ms", started.elapsed().as_millis());
    let listening = send(
        events,
        LoadEvent::ArtReady {
            renderer: texture_renderer,
            intro_music,
        },
    );
    if !listening {
        return;
    }
    let has_room_artwork = match texture_renderer.render(BASEMENT_BACKGROUND, pixels_per_world_unit) {
        Ok((image, _)) => {
            send(events, LoadEvent::Texture(BASEMENT_BACKGROUND, image));
            true
        }
        Err(render_error) => {
            println!("Room artwork unavailable, using placeholders: {render_error}");
            false
        }
    };
    let others: Vec<TextureId> = TextureId::all()
        .into_iter()
        .filter(|&texture_id| texture_id != BASEMENT_BACKGROUND && !drawn_as_vectors(texture_renderer, texture_id))
        .collect();
    render_textures(texture_renderer, &others, pixels_per_world_unit, events);
    println!("Loaded assets in {} ms", started.elapsed().as_millis());
    let sound_bank = sound_bank.unwrap_or_else(|missing_sounds| {
        println!("Playing without sound: {missing_sounds}");
        SoundBank::default()
    });
    send(
        events,
        finished(sound_bank, has_room_artwork, texture_renderer.basement_room_layouts()),
    );
}

/// Loads on the calling thread and gathers the results, for frontends without a loading screen.
#[cfg_attr(
    not(feature = "bevy-frontend"),
    expect(dead_code, reason = "only the Bevy frontend loads up front")
)]
pub fn load_blocking(pixels_per_world_unit: f32) -> (LoadedGame, TextureSupply) {
    let (sender, receiver) = std::sync::mpsc::channel();
    load(pixels_per_world_unit, |_, _| false, &sender);
    drop(sender);
    let mut supply = TextureSupply {
        pixels_per_world_unit,
        ..TextureSupply::default()
    };
    let mut loaded = LoadedGame::default();
    for event in receiver {
        match event {
            LoadEvent::ArtReady { renderer, .. } => supply.renderer = Some(renderer),
            LoadEvent::Texture(texture_id, image) => supply.prerendered.push((texture_id, image)),
            LoadEvent::Finished(loaded_game) => loaded = *loaded_game,
            LoadEvent::Failed(reason) => eprintln!("{reason}"),
        }
    }
    (loaded, supply)
}

fn finished(sound_bank: SoundBank, has_room_artwork: bool, room_layouts: RoomLayouts) -> LoadEvent {
    let clip_count: usize = sound_bank.events().map(|event| sound_bank.variants(event).len()).sum();
    println!("Loaded {clip_count} sound clips");
    println!(
        "Loaded {} room layouts and {} treasure rooms",
        room_layouts.normal.len(),
        room_layouts.treasure.len()
    );
    LoadEvent::Finished(Box::new(LoadedGame {
        sound_bank,
        room_appearance: if has_room_artwork {
            RoomAppearance::Artwork
        } else {
            RoomAppearance::Placeholder
        },
        room_layouts,
    }))
}

/// `false` once nobody listens (the frontend quit early): the remaining work can stop.
pub(crate) fn send(events: &Sender<LoadEvent>, event: LoadEvent) -> bool {
    events.send(event).is_ok()
}

/// The original's splash image, extracted from the projector on first use; `None` (quietly) when
/// the game or the image is not there, since the loader reports those problems.
#[cfg_attr(
    not(feature = "sdl3-frontend"),
    expect(dead_code, reason = "only the SDL3 frontend shows the splash")
)]
pub fn splash_bitmap_path() -> Option<PathBuf> {
    let game_directory = game_assets::locate_game_directory().ok()?;
    let cache_directory = game_assets::cache_directory()?;
    game_assets::ensure_extracted_splash(&game_directory, &cache_directory)
        .inspect_err(|extract_error| println!("No splash screen: {extract_error}"))
        .ok()
}

/// Locates the user's Steam copy and extracts the game SWF; the game still runs without it.
fn extracted_swf_path() -> Option<PathBuf> {
    let game_directory = game_assets::locate_game_directory()
        .inspect_err(|locate_error| {
            println!(
                "{locate_error}. Using placeholder graphics; set {} to point at the game directory.",
                game_assets::GAME_DIRECTORY_OVERRIDE_VARIABLE
            );
        })
        .ok()?;
    println!("Found game at {}", game_directory.display());
    let Some(cache_directory) = game_assets::cache_directory() else {
        println!("Could not extract the game SWF: no per-user cache directory");
        return None;
    };
    extract(&game_directory, &cache_directory)
}

fn extract(game_directory: &Path, cache_directory: &Path) -> Option<PathBuf> {
    let extraction_started = Instant::now();
    game_assets::ensure_extracted_swf(game_directory, cache_directory)
        .inspect(|swf_path| {
            println!(
                "Game SWF ready at {} ({} ms)",
                swf_path.display(),
                extraction_started.elapsed().as_millis()
            );
        })
        .inspect_err(|extract_error| println!("Could not extract the game SWF: {extract_error}"))
        .ok()
}

mod workers;

#[cfg(feature = "sdl3-frontend")]
pub use workers::load_in_background;
pub use workers::render_textures;

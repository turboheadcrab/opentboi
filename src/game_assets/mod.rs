//! Finds the user's own copy of the game and derives our working files from it.
//! Nothing produced here is shipped; it is rebuilt from the Steam install on each machine.

mod asset_loader;
mod input_glyphs;
mod intro;
mod room_layouts;
mod script;
mod sound_bank;
#[cfg(test)]
mod split_measurement;
mod steam_locator;
mod texture_catalog;
mod vector_art;
// Gradient and bitmap fills are meshed but not drawn by any frontend yet.
#[cfg_attr(not(test), allow(dead_code))]
mod vector_mesh;
#[cfg_attr(not(feature = "sdl3-frontend"), allow(dead_code))]
mod vector_picture;
mod zinc_extractor;

use std::path::{Path, PathBuf};

pub use asset_loader::{LoadedAssets, TextureRenderer, TextureSupply, load_assets};
#[cfg(feature = "sdl3-frontend")]
pub use intro::{INTRO_FRAME_COUNT, STAGE_HEIGHT, STAGE_WIDTH};
pub use sound_bank::{MusicClips, SoundBank, SoundClip};
pub use steam_locator::{GAME_DIRECTORY_OVERRIDE_VARIABLE, locate_game_directory};
pub use vector_art::RgbaImage;
#[cfg(feature = "sdl3-frontend")]
pub use vector_mesh::{MeshDraw, MeshFill};
#[cfg(feature = "sdl3-frontend")]
pub use vector_picture::{VectorPart, VectorPicture};
pub use zinc_extractor::ExtractError;

const CACHE_DIRECTORY_NAME: &str = "opentboi";
/// The cache directory before the project was named OpenTBOI; moved over on first use.
const FORMER_CACHE_DIRECTORY_NAME: &str = "tboi-remake";
pub(crate) const EXTRACTED_SWF_FILE_NAME: &str = "isaac.swf";
/// The name the original build used for it.
const EXTRACTED_SPLASH_FILE_NAME: &str = "isaacload.bmp";
const PARTIAL_FILE_EXTENSION: &str = "part";

const WINDOWS_CACHE_ROOT_VARIABLE: &str = "LOCALAPPDATA";
const HOME_VARIABLE: &str = "HOME";
const MACOS_CACHE_SUBDIRECTORY: &str = "Library/Caches";
const XDG_CACHE_ROOT_VARIABLE: &str = "XDG_CACHE_HOME";
const XDG_DEFAULT_CACHE_SUBDIRECTORY: &str = ".cache";

/// Per-user cache directory following each platform's convention.
pub fn cache_directory() -> Option<PathBuf> {
    let environment_path = |name| {
        std::env::var_os(name)
            .filter(|value| !value.is_empty())
            .map(PathBuf::from)
    };
    let cache_root = if cfg!(windows) {
        environment_path(WINDOWS_CACHE_ROOT_VARIABLE)?
    } else if cfg!(target_os = "macos") {
        environment_path(HOME_VARIABLE)?.join(MACOS_CACHE_SUBDIRECTORY)
    } else {
        environment_path(XDG_CACHE_ROOT_VARIABLE)
            .or_else(|| environment_path(HOME_VARIABLE).map(|home| home.join(XDG_DEFAULT_CACHE_SUBDIRECTORY)))?
    };
    let directory = cache_root.join(CACHE_DIRECTORY_NAME);
    let former = cache_root.join(FORMER_CACHE_DIRECTORY_NAME);
    if !directory.exists() && former.is_dir() {
        // Keeps what was already extracted; if the move fails, extraction simply runs again.
        let _ = std::fs::rename(&former, &directory);
    }
    Some(directory)
}

/// Returns the uncompressed (`FWS`) game SWF, extracting it into `cache_directory` on first use.
pub fn ensure_extracted_swf(game_directory: &Path, cache_directory: &Path) -> Result<PathBuf, ExtractError> {
    ensure_extracted(
        game_directory,
        &cache_directory.join(EXTRACTED_SWF_FILE_NAME),
        zinc_extractor::extract_swf,
    )
}

/// Returns the projector's splash bitmap (`isaacload.bmp`), extracting it on first use.
pub fn ensure_extracted_splash(game_directory: &Path, cache_directory: &Path) -> Result<PathBuf, ExtractError> {
    ensure_extracted(
        game_directory,
        &cache_directory.join(EXTRACTED_SPLASH_FILE_NAME),
        zinc_extractor::extract_splash_bitmap,
    )
}

fn ensure_extracted(
    game_directory: &Path,
    cached_path: &Path,
    extract: fn(&[u8]) -> Result<Vec<u8>, ExtractError>,
) -> Result<PathBuf, ExtractError> {
    if cached_path.is_file() {
        return Ok(cached_path.to_path_buf());
    }
    let executable_bytes = std::fs::read(game_directory.join(zinc_extractor::GAME_EXECUTABLE_NAME))?;
    let extracted = extract(&executable_bytes)?;
    if let Some(cache_directory) = cached_path.parent() {
        std::fs::create_dir_all(cache_directory)?;
    }
    // Write then rename so an interrupted extraction never leaves a truncated file behind.
    let partial_path = cached_path.with_extension(PARTIAL_FILE_EXTENSION);
    std::fs::write(&partial_path, &extracted)?;
    std::fs::rename(&partial_path, cached_path)?;
    Ok(cached_path.to_path_buf())
}

/// Test helper: writes an image as PNG for visual checks.
#[cfg(test)]
pub(crate) fn write_png(path: &Path, image: &RgbaImage) {
    let file = std::io::BufWriter::new(std::fs::File::create(path).expect("cannot create the PNG"));
    let mut encoder = png::Encoder::new(file, image.width, image.height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().expect("cannot write the PNG header");
    writer
        .write_image_data(&image.pixels)
        .expect("cannot write the PNG pixels");
}

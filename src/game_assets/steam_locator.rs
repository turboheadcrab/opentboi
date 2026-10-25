use std::path::PathBuf;

/// Steam AppID of the original Flash release (Rebirth is 250900).
pub const ORIGINAL_GAME_APP_ID: u32 = 113_200;
/// Points at a game directory directly, bypassing Steam discovery.
pub const GAME_DIRECTORY_OVERRIDE_VARIABLE: &str = "TBOI_GAME_DIR";

#[derive(Debug)]
pub enum LocateError {
    OverrideIsNotADirectory(PathBuf),
    SteamNotFound(String),
    GameNotInstalled,
}

impl std::fmt::Display for LocateError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OverrideIsNotADirectory(path) => {
                write!(
                    formatter,
                    "{GAME_DIRECTORY_OVERRIDE_VARIABLE}={} is not a directory",
                    path.display()
                )
            }
            Self::SteamNotFound(reason) => write!(formatter, "no Steam installation found: {reason}"),
            Self::GameNotInstalled => write!(
                formatter,
                "The Binding of Isaac (AppID {ORIGINAL_GAME_APP_ID}) is not installed in any Steam library"
            ),
        }
    }
}

pub fn locate_game_directory() -> Result<PathBuf, LocateError> {
    match std::env::var_os(GAME_DIRECTORY_OVERRIDE_VARIABLE) {
        Some(override_path) => {
            let override_path = PathBuf::from(override_path);
            if override_path.is_dir() {
                Ok(override_path)
            } else {
                Err(LocateError::OverrideIsNotADirectory(override_path))
            }
        }
        None => find_in_steam_libraries(),
    }
}

fn find_in_steam_libraries() -> Result<PathBuf, LocateError> {
    // locate_all covers native, Flatpak and Snap installs side by side. It reads each library's
    // appmanifest files, which stay current even when libraryfolders.vdf's app list is stale.
    let steam_installations =
        steamlocate::locate_all().map_err(|error| LocateError::SteamNotFound(error.to_string()))?;
    for steam_installation in steam_installations {
        if let Ok(Some((app, library))) = steam_installation.find_app(ORIGINAL_GAME_APP_ID) {
            return Ok(library.resolve_app_dir(&app));
        }
    }
    Err(LocateError::GameNotInstalled)
}

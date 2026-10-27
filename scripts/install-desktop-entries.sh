#!/usr/bin/env bash
# Builds the game with both frontends and installs one launcher per frontend for the current
# user. The binary is copied to target/desktop/ so later builds with other features do not break
# the launchers. Remove with: rm ~/.local/share/applications/opentboi{,-bevy}.desktop
set -euo pipefail

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
readonly PROJECT_ROOT
readonly LAUNCHER_BINARY="${PROJECT_ROOT}/target/desktop/opentboi"
readonly APPLICATIONS_DIRECTORY="${XDG_DATA_HOME:-${HOME}/.local/share}/applications"
# The project's own icon, copied into the user's icon directory; the launchers name it by its full
# path and as a PNG: GNOME Shell shows neither an SVG nor a name it has to look up in the user's
# "hicolor" (which has no index.theme). The release package installs the SVG into the theme.
readonly ICON_NAME="opentboi"
readonly ICON_SOURCE="${PROJECT_ROOT}/packaging/linux/opentboi.png"
readonly ICON_DIRECTORY="${XDG_DATA_HOME:-${HOME}/.local/share}/icons/hicolor/256x256/apps"

cd "${PROJECT_ROOT}"
cargo build --release --all-features
mkdir -p "$(dirname "${LAUNCHER_BINARY}")" "${APPLICATIONS_DIRECTORY}" "${ICON_DIRECTORY}"
cp "${ICON_SOURCE}" "${ICON_DIRECTORY}/${ICON_NAME}.png"
# Linux refuses to write into a running executable ("Text file busy"), so copy beside it and
# rename over it: a running game keeps its old file, the next launch gets the new one.
cp target/release/opentboi "${LAUNCHER_BINARY}.new"
mv -f "${LAUNCHER_BINARY}.new" "${LAUNCHER_BINARY}"
# Launchers from before the project was named OpenTBOI.
rm -f "${APPLICATIONS_DIRECTORY}"/tboi-remake{,-bevy}.desktop "${PROJECT_ROOT}/target/desktop/tboi-remake"

# write_entry FILE_NAME DISPLAY_NAME FRONTEND WINDOW_CLASS
# WINDOW_CLASS must equal the frontend's application id (src/frontend/mod.rs).
write_entry() {
    cat > "${APPLICATIONS_DIRECTORY}/$1" <<ENTRY
[Desktop Entry]
Type=Application
Name=$2
Comment=Open-source engine for The Binding of Isaac (Flash), using your own copy (dev build, $3 frontend)
Exec=${LAUNCHER_BINARY} --frontend $3
Path=${PROJECT_ROOT}
Icon=${ICON_DIRECTORY}/${ICON_NAME}.png
Terminal=false
Categories=Game;
StartupWMClass=$4
ENTRY
}

write_entry opentboi.desktop "OpenTBOI" sdl3 opentboi
write_entry opentboi-bevy.desktop "OpenTBOI (Bevy)" bevy opentboi-bevy
update-desktop-database "${APPLICATIONS_DIRECTORY}"
# A cache of this icon theme is not written: the user's copy of "hicolor" has no index.theme, so a
# cache forced onto it hides icons that Steam and other programs add later. A stale one is removed.
rm -f "${ICON_DIRECTORY}/../../icon-theme.cache"
echo "Installed launchers for ${LAUNCHER_BINARY}"

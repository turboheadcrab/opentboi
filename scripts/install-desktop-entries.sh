#!/usr/bin/env bash
# Builds the game with both frontends and installs one launcher per frontend for the current
# user. The binary is copied to target/desktop/ so later builds with other features do not break
# the launchers. Remove with: rm ~/.local/share/applications/opentboi{,-bevy}.desktop
set -euo pipefail

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
readonly PROJECT_ROOT
readonly LAUNCHER_BINARY="${PROJECT_ROOT}/target/desktop/opentboi"
readonly APPLICATIONS_DIRECTORY="${XDG_DATA_HOME:-${HOME}/.local/share}/applications"
# The project's own icon, installed into the user's icon theme (as the release package does), so the
# launchers show it on every system, whatever Steam has or has not put there.
readonly ICON_NAME="opentboi"
readonly ICON_SOURCE="${PROJECT_ROOT}/packaging/linux/opentboi.svg"
readonly ICON_DIRECTORY="${XDG_DATA_HOME:-${HOME}/.local/share}/icons/hicolor/scalable/apps"

cd "${PROJECT_ROOT}"
cargo build --release --all-features
mkdir -p "$(dirname "${LAUNCHER_BINARY}")" "${APPLICATIONS_DIRECTORY}" "${ICON_DIRECTORY}"
cp "${ICON_SOURCE}" "${ICON_DIRECTORY}/${ICON_NAME}.svg"
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
Icon=${ICON_NAME}
Terminal=false
Categories=Game;
StartupWMClass=$4
ENTRY
}

write_entry opentboi.desktop "OpenTBOI" sdl3 opentboi
write_entry opentboi-bevy.desktop "OpenTBOI (Bevy)" bevy opentboi-bevy
update-desktop-database "${APPLICATIONS_DIRECTORY}"
# Some desktops read the icon theme's cache; the tool is optional.
if command -v gtk-update-icon-cache >/dev/null 2>&1; then
    gtk-update-icon-cache --quiet --force --ignore-theme-index "${ICON_DIRECTORY}/../.." || true
fi
echo "Installed launchers for ${LAUNCHER_BINARY}"

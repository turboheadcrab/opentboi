#!/bin/sh
# Installs OpenTBOI from this unpacked tarball. Needs no root: the default prefix is ~/.local, so it
# works on immutable systems too (SteamOS, Bazzite, Silverblue). Touches only files it lists below.
#
#   ./install.sh                  install into ~/.local
#   ./install.sh --prefix /opt/x  install under another prefix (DESTDIR is honoured for packagers)
#   ./install.sh --uninstall      remove what a previous run installed
set -eu

NAME="opentboi"
here=$(cd "$(dirname "$0")" && pwd)
prefix="${HOME}/.local"
uninstall=0

while [ $# -gt 0 ]; do
    case "$1" in
        --prefix) prefix="$2"; shift 2 ;;
        --uninstall) uninstall=1; shift ;;
        -h|--help) sed -n '2,8p' "$0" | sed 's/^# \{0,1\}//'; exit 0 ;;
        *) echo "Unknown option: $1 (try --help)" >&2; exit 2 ;;
    esac
done

root="${DESTDIR:-}"
binary_directory="${root}${prefix}/bin"
data_directory="${root}${prefix}/share"
applications_directory="${data_directory}/applications"
icon_directory="${data_directory}/icons/hicolor/scalable/apps"
license_directory="${data_directory}/licenses/${NAME}"
installed_binary="${prefix}/bin/${NAME}"

refresh_desktop_caches() {
    if command -v update-desktop-database > /dev/null 2>&1; then
        update-desktop-database -q "${applications_directory}" 2> /dev/null || true
    fi
    if command -v gtk-update-icon-cache > /dev/null 2>&1; then
        gtk-update-icon-cache -q -t -f "${data_directory}/icons/hicolor" 2> /dev/null || true
    fi
}

if [ "${uninstall}" -eq 1 ]; then
    rm -f "${binary_directory}/${NAME}" "${binary_directory}/${NAME}-diagnose" "${applications_directory}/${NAME}.desktop" "${icon_directory}/${NAME}.svg"
    rm -f "${license_directory}/LICENSE" "${license_directory}/THIRD-PARTY-LICENSES" "${license_directory}/SOURCE.txt"
    rmdir "${license_directory}" 2> /dev/null || true
    refresh_desktop_caches
    echo "OpenTBOI removed from ${prefix}. Extracted game data stays in ~/.cache/opentboi (delete it if you like)."
    exit 0
fi

mkdir -p "${binary_directory}" "${applications_directory}" "${icon_directory}" "${license_directory}"
# Write beside and rename, so a running game keeps its old file ("Text file busy" otherwise).
cp "${here}/bin/${NAME}" "${binary_directory}/${NAME}.new"
chmod 755 "${binary_directory}/${NAME}.new"
mv -f "${binary_directory}/${NAME}.new" "${binary_directory}/${NAME}"
install -m 755 "${here}/bin/${NAME}-diagnose" "${binary_directory}/${NAME}-diagnose"
sed "s|@BINARY@|${installed_binary}|" "${here}/share/applications/${NAME}.desktop" > "${applications_directory}/${NAME}.desktop"
cp "${here}/share/icons/hicolor/scalable/apps/${NAME}.svg" "${icon_directory}/${NAME}.svg"
for file in LICENSE THIRD-PARTY-LICENSES SOURCE.txt; do
    if [ -f "${here}/${file}" ]; then
        cp "${here}/${file}" "${license_directory}/${file}"
    fi
done
refresh_desktop_caches

echo "Installed OpenTBOI to ${prefix} (launcher: ${applications_directory}/${NAME}.desktop)."

if command -v ldd > /dev/null 2>&1; then
    missing=$(ldd "${binary_directory}/${NAME}" 2> /dev/null | grep 'not found' || true)
    if [ -n "${missing}" ]; then
        echo "Missing system libraries (install them with your distribution's package manager):" >&2
        echo "${missing}" >&2
    fi
fi
case ":${PATH}:" in
    *":${prefix}/bin:"*) ;;
    *) echo "Note: ${prefix}/bin is not on your PATH; the launcher entry still works, or run ${installed_binary}." ;;
esac
echo "You need The Binding of Isaac (Steam AppID 113200) installed; set TBOI_GAME_DIR if it is elsewhere."

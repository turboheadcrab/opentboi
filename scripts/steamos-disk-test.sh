#!/usr/bin/env bash
# Tests OpenTBOI against a real SteamOS install whose disk is plugged into this PC (for example a
# Steam Deck's eMMC in an M.2 slot). The disk's partitions must be mounted (a desktop file manager
# does it, under /run/media/$USER/<label>); nothing here needs root.
#
#   scripts/steamos-disk-test.sh stage [TARBALL]   install OpenTBOI, a copy of the game, launchers
#                                                  and a Game Mode shortcut into the disk's /home/deck
#   scripts/steamos-disk-test.sh run [ARGS...]     run the staged game on THIS PC, inside SteamOS's
#                                                  own userland (bubblewrap); ARGS go to opentboi
#                                                  (default: --showcase --exit-after 3)
#   scripts/steamos-disk-test.sh unstage           remove everything `stage` wrote
#
# `run` checks that the release binary starts on SteamOS's libraries (glibc, loader, audio). It does
# not run SteamOS's kernel, Steam, gamescope or Game Mode; for those boot the disk.
#
# Environment:
#   STEAMOS_MOUNTS   where the partitions are mounted (default /run/media/$USER)
#   STEAMOS_ROOT     root partition label, rootfs or rootfs1 (default: the one with the newer build)
#   GAME_SOURCE      the game's folder to copy (default: the Steam library's "The Binding Of Isaac")
set -euo pipefail

SCRIPT_DIRECTORY="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly SCRIPT_DIRECTORY
readonly PROJECT_ROOT="${SCRIPT_DIRECTORY}/.."
readonly MOUNTS="${STEAMOS_MOUNTS:-/run/media/${USER}}"
readonly DECK_HOME="/home/deck"
readonly TEST_DIRECTORY_NAME="OpenTBOI-test"
readonly GAME_SOURCE_DEFAULT="${HOME}/.local/share/Steam/steamapps/common/The Binding Of Isaac"
readonly SHORTCUT_NAME="OpenTBOI test"
readonly DEFAULT_RUN_ARGUMENTS=(--showcase --exit-after 3)

die() {
    echo "$*" >&2
    exit 1
}

build_id() {
    sed -n 's/^BUILD_ID=//p' "${MOUNTS}/$1/etc/os-release" 2> /dev/null || true
}

# The root partition with the newer SteamOS build; SteamOS keeps two (A and B).
steamos_root() {
    if [ -n "${STEAMOS_ROOT:-}" ]; then
        echo "${STEAMOS_ROOT}"
        return
    fi
    local newest="" newest_build="" label
    for label in rootfs rootfs1; do
        local build
        build="$(build_id "${label}")"
        if [ -n "${build}" ] && [[ "${build}" > "${newest_build}" ]]; then
            newest="${label}"
            newest_build="${build}"
        fi
    done
    [ -n "${newest}" ] || die "No SteamOS root partition found under ${MOUNTS} (expected rootfs or rootfs1)."
    echo "${newest}"
}

require_home() {
    [ -d "${MOUNTS}/home/deck" ] || die "No SteamOS home partition at ${MOUNTS}/home (expected /deck inside it)."
}

write_launcher() {
    local path="$1" command="$2"
    printf '#!/bin/sh\nexport TBOI_GAME_DIR=%s/%s/Isaac\nexec %s\n' \
        "${DECK_HOME}" "${TEST_DIRECTORY_NAME}" "${command}" > "${path}"
    chmod 755 "${path}"
}

install_binaries() {
    local tarball="$1" work installer
    work="$(mktemp -d)"
    # The tarball's installer writes under DESTDIR; a symlink makes /home land on the SteamOS disk.
    ln -s "${MOUNTS}/home" "${work}/home"
    tar --zstd -xf "${tarball}" -C "${work}"
    for installer in "${work}"/opentboi-*/install.sh; do
        DESTDIR="${work}" "${installer}" --prefix "${DECK_HOME}/.local"
    done
    rm -rf "${work}"
}

stage() {
    require_home
    local tarball="${1:-}" candidate
    if [ -z "${tarball}" ]; then
        for candidate in "${PROJECT_ROOT}"/target/container-dist/*.tar.zst; do
            if [ -z "${tarball}" ] || [ "${candidate}" -nt "${tarball}" ]; then
                tarball="${candidate}"
            fi
        done
    fi
    [ -f "${tarball}" ] || die "No tarball: build one with scripts/build-tarball-in-container.sh, or pass its path."
    local game_source="${GAME_SOURCE:-${GAME_SOURCE_DEFAULT}}"
    [ -f "${game_source}/Isaac.exe" ] || die "No game at ${game_source} (set GAME_SOURCE)."

    local test_directory="${MOUNTS}/home/deck/${TEST_DIRECTORY_NAME}"
    install_binaries "${tarball}"
    mkdir -p "${test_directory}"
    if [ ! -d "${test_directory}/Isaac" ]; then
        cp -r --reflink=auto "${game_source}" "${test_directory}/Isaac"
    fi
    write_launcher "${test_directory}/opentboi-test.sh" "${DECK_HOME}/.local/bin/opentboi --fullscreen \"\$@\""
    write_launcher "${test_directory}/opentboi-diagnose-test.sh" "${DECK_HOME}/.local/bin/opentboi-diagnose 20"
    "${SCRIPT_DIRECTORY}/steam-shortcuts.py" add "${MOUNTS}/home/deck/.local/share/Steam" "${SHORTCUT_NAME}" \
        "${DECK_HOME}/${TEST_DIRECTORY_NAME}/opentboi-test.sh" || echo "No Game Mode shortcut added (see above)."
    echo "Staged. Boot the disk, or try it here: $0 run"
}

unstage() {
    require_home
    local work
    work="$(mktemp -d)"
    ln -s "${MOUNTS}/home" "${work}/home"
    DESTDIR="${work}" "${PROJECT_ROOT}/packaging/linux/install.sh" --uninstall --prefix "${DECK_HOME}/.local"
    rm -rf "${work}" "${MOUNTS}/home/deck/${TEST_DIRECTORY_NAME}" "${MOUNTS}/home/deck/.cache/opentboi"
    "${SCRIPT_DIRECTORY}/steam-shortcuts.py" remove "${MOUNTS}/home/deck/.local/share/Steam" "${SHORTCUT_NAME}" || true
}

run_in_userland() {
    require_home
    command -v bwrap > /dev/null || die "bubblewrap (bwrap) is needed."
    local root socket runtime="${XDG_RUNTIME_DIR:-/run/user/$(id -u)}"
    root="$(steamos_root)"
    echo "SteamOS $(sed -n 's/^VERSION_ID=//p' "${MOUNTS}/${root}/etc/os-release") (${root}), build $(build_id "${root}")"
    local binds=(--bind "${MOUNTS}/home" /home --bind "${runtime}/${WAYLAND_DISPLAY:-wayland-0}" "${runtime}/${WAYLAND_DISPLAY:-wayland-0}")
    for socket in pulse pipewire-0; do
        if [ -e "${runtime}/${socket}" ]; then
            binds+=(--bind "${runtime}/${socket}" "${runtime}/${socket}")
        fi
    done
    if [ -d /tmp/.X11-unix ]; then
        binds+=(--ro-bind /tmp/.X11-unix /tmp/.X11-unix)
    fi
    local arguments=("$@")
    if [ "$#" -eq 0 ]; then
        arguments=("${DEFAULT_RUN_ARGUMENTS[@]}")
    fi
    bwrap --ro-bind "${MOUNTS}/${root}" / --dev-bind /dev /dev --proc /proc --ro-bind /sys /sys \
        --tmpfs /tmp --tmpfs /run "${binds[@]}" \
        --setenv XDG_RUNTIME_DIR "${runtime}" --setenv WAYLAND_DISPLAY "${WAYLAND_DISPLAY:-wayland-0}" \
        --setenv DISPLAY "${DISPLAY:-}" --setenv HOME "${DECK_HOME}" \
        --setenv TBOI_GAME_DIR "${DECK_HOME}/${TEST_DIRECTORY_NAME}/Isaac" --chdir "${DECK_HOME}" \
        "${DECK_HOME}/.local/bin/opentboi" "${arguments[@]}"
}

case "${1:-}" in
    stage) stage "${2:-}" ;;
    run) shift && run_in_userland "$@" ;;
    unstage) unstage ;;
    *) sed -n '2,19p' "$0" | sed 's/^# \{0,1\}//' ;;
esac

#!/usr/bin/env bash
# Runs a command where the build packages are: in the development container (`distrobox.ini`) when
# it exists and this is not already inside a container, otherwise right here (a workstation with
# the packages installed, or the container itself). The rustup toolchain in ~/.cargo is put first
# on the path either way, since a container's shell does not load the host's version managers; in
# the container the rest of the path is the container's own, so tools the host keeps in the shared
# home (Homebrew's `pkg-config`, say) cannot stand in for Fedora's.
#
# Usage: scripts/dev-run.sh COMMAND [ARGUMENT...]   (the `mise run` tasks call it)

set -euo pipefail

readonly CONTAINER_NAME="opentboi-dev"
readonly CARGO_BIN="${HOME}/.cargo/bin"
readonly CONTAINER_SYSTEM_PATH="/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin"

if [[ $# -eq 0 ]]; then
    echo "usage: $0 COMMAND [ARGUMENT...]" >&2
    exit 2
fi

container_exists() {
    command -v distrobox >/dev/null 2>&1 || return 1
    # `distrobox list` prints a table: ID | NAME | STATUS | IMAGE.
    distrobox list --no-color 2>/dev/null | awk -F'|' 'NR > 1 { gsub(/ /, "", $2); print $2 }' |
        grep --quiet --line-regexp --fixed-strings "${CONTAINER_NAME}"
}

# The project pins its Rust version (rust-toolchain.toml) and rustup does not fetch it on its own.
if [[ -x "${CARGO_BIN}/rustup" ]] && ! "${CARGO_BIN}/rustup" show active-toolchain >/dev/null 2>&1; then
    "${CARGO_BIN}/rustup" toolchain install
fi

if [[ -z "${CONTAINER_ID:-}" ]] && container_exists; then
    exec distrobox enter --name "${CONTAINER_NAME}" -- env "PATH=${CARGO_BIN}:${CONTAINER_SYSTEM_PATH}" "$@"
fi
export PATH="${CARGO_BIN}:${PATH}"
exec "$@"

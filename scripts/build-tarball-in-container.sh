#!/usr/bin/env bash
# Builds the release tarball the way .github/workflows/release.yml does (Ubuntu 22.04, static SDL3),
# but locally with podman or docker, from the working tree (uncommitted changes included). Use it to
# get a tarball that runs on the Steam Deck / Bazzite without waiting for CI.
#
# Output: target/container-dist/ (the .tar.zst and its .sha256). The build itself happens in a
# throw-away copy inside the container, so the host's target/ directory is not touched.
#
# Environment:
#   CONTAINER_ENGINE   podman (default) or docker
#   BASE_IMAGE         default ubuntu:22.04 (keep equal to the one in release.yml)
#   CARGO_FEATURES     default "--features bundled-sdl3"
set -euo pipefail

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
readonly PROJECT_ROOT
readonly OUTPUT_DIRECTORY="${PROJECT_ROOT}/target/container-dist"
readonly ENGINE="${CONTAINER_ENGINE:-podman}"
readonly BASE_IMAGE="${BASE_IMAGE:-ubuntu:22.04}"
readonly FEATURES="${CARGO_FEATURES:---features bundled-sdl3}"

mkdir -p "${OUTPUT_DIRECTORY}"
# The script below must reach the container unexpanded, hence the single quotes.
# shellcheck disable=SC2016
# The ":Z" relabels the mounts for SELinux hosts (Fedora, Bazzite); docker ignores it harmlessly.
"${ENGINE}" run --rm \
    -v "${PROJECT_ROOT}:/source:ro,Z" \
    -v "${OUTPUT_DIRECTORY}:/output:Z" \
    -e "CARGO_FEATURES=${FEATURES}" \
    "${BASE_IMAGE}" bash -euo pipefail -c '
        apt-get update
        DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
            build-essential cmake pkg-config git curl ca-certificates zstd binutils libasound2-dev libudev-dev \
            libx11-dev libxext-dev libxcursor-dev libxi-dev libxrandr-dev libxfixes-dev libxss-dev libxtst-dev \
            libxkbcommon-dev libwayland-dev wayland-protocols libdecor-0-dev libdrm-dev libgbm-dev \
            libgl-dev libegl-dev libvulkan-dev libpulse-dev libpipewire-0.3-dev libdbus-1-dev
        curl --proto "=https" --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal
        . "$HOME/.cargo/env"
        cargo install cargo-about --locked --features cli
        mkdir /work
        tar -C /source --exclude=./target --exclude=./downloads -cf - . | tar -C /work -xf -
        cd /work
        scripts/package-linux-tarball.sh
        cp target/dist/* /output/
    '
chmod 644 "${OUTPUT_DIRECTORY}"/*
ls -l "${OUTPUT_DIRECTORY}"

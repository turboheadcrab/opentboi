#!/usr/bin/env bash
# Builds a self-contained Linux tarball in target/dist/: the release binary, its own launcher and
# icon (packaging/linux/, independent of any desktop entry on the build machine), the licences, a
# source pointer and install.sh. Unpack anywhere and run ./install.sh (no root needed).
#
# Environment:
#   CARGO_FEATURES   extra cargo arguments, e.g. "--features bundled-sdl3" (default: none)
#   RUSTFLAGS        e.g. "-C target-cpu=x86-64-v2" (default: rustc's baseline, which runs on any x86-64 CPU)
set -euo pipefail

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
readonly PROJECT_ROOT
readonly PACKAGING_DIRECTORY="${PROJECT_ROOT}/packaging/linux"
readonly DISTRIBUTION_DIRECTORY="${PROJECT_ROOT}/target/dist"
readonly NAME="opentboi"

cd "${PROJECT_ROOT}"
version=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -n 1)
architecture=$(uname -m)
package_name="${NAME}-${version}-linux-${architecture}"
staging="${DISTRIBUTION_DIRECTORY}/${package_name}"

# Whether the working tree differs from the commit. A release stamps its version into Cargo.toml and
# Cargo.lock (scripts/stamp-version.sh, OPENTBOI_STAMPED_VERSION set), which is not a difference.
is_dirty() {
    if [[ -n "${OPENTBOI_STAMPED_VERSION:-}" ]]; then
        ! git diff --quiet HEAD -- . ':!Cargo.toml' ':!Cargo.lock' 2> /dev/null
    else
        ! git diff --quiet HEAD 2> /dev/null
    fi
}

# Release builds must carry up-to-date third-party licence texts, so fail rather than warn.
export OPENTBOI_REQUIRE_LICENSES=1
# shellcheck disable=SC2086
cargo build --release --locked ${CARGO_FEATURES:-}

rm -rf "${staging}"
mkdir -p "${staging}/bin" "${staging}/share/applications" "${staging}/share/icons/hicolor/scalable/apps"
install -m 755 "target/release/${NAME}" "${staging}/bin/${NAME}"
strip "${staging}/bin/${NAME}"
install -m 755 "${PACKAGING_DIRECTORY}/${NAME}-diagnose" "${staging}/bin/${NAME}-diagnose"
install -m 644 "${PACKAGING_DIRECTORY}/${NAME}.desktop" "${staging}/share/applications/${NAME}.desktop"
install -m 644 "${PACKAGING_DIRECTORY}/${NAME}.svg" "${staging}/share/icons/hicolor/scalable/apps/${NAME}.svg"
install -m 755 "${PACKAGING_DIRECTORY}/install.sh" "${staging}/install.sh"
install -m 644 LICENSE THIRD-PARTY-LICENSES README.md "${staging}/"

# GPLv3 section 6: whoever gets the binary must be able to get the matching source.
{
    echo "OpenTBOI ${version} is free software (GPL-3.0-or-later); this binary was built from:"
    echo "  commit:  $(git rev-parse HEAD 2> /dev/null || echo unknown)$(is_dirty && echo ' (plus uncommitted changes)')"
    echo "  source:  $(git remote get-url origin 2> /dev/null || echo 'the project repository')"
    echo "Build it yourself with: cargo build --release --locked (see README.md)."
} > "${staging}/SOURCE.txt"

# Reproducible archive: sorted names, no owner, fixed time (the commit's, else now).
source_date=$(git log -1 --format=%ct 2> /dev/null || date +%s)
archive="${DISTRIBUTION_DIRECTORY}/${package_name}.tar.zst"
tar --sort=name --owner=0 --group=0 --numeric-owner --mtime="@${source_date}" \
    -C "${DISTRIBUTION_DIRECTORY}" -cf - "${package_name}" | zstd -q -19 -f -o "${archive}"
(cd "${DISTRIBUTION_DIRECTORY}" && sha256sum "${package_name}.tar.zst" > "${package_name}.tar.zst.sha256")
rm -rf "${staging}"
echo "${archive}"

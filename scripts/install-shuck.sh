#!/usr/bin/env bash
# Installs the shell linter shuck (`.shuck.toml`) at the one version the project uses, checking the
# download against the checksum recorded here, so every machine and CI lint with the same tool:
#
#   scripts/install-shuck.sh [DIRECTORY]      (default: ~/.local/bin)
#
# To move to a newer release: change SHUCK_VERSION and the checksum (from the release page's
# `.sha256` file and the GitHub digest of the asset; check that they agree), and fix what the newer
# rules find (`shuck check scripts packaging .github`).
set -euo pipefail

readonly SHUCK_VERSION="0.2.7"
readonly SHUCK_ASSET="shuck-cli-x86_64-unknown-linux-gnu.tar.xz"
readonly SHUCK_SHA256="66db3daa642210657876c9f57bc69b76c4215d3e80cbee591ed729bfeb910215"
readonly SHUCK_URL="https://github.com/ewhauser/shuck/releases/download/v${SHUCK_VERSION}/${SHUCK_ASSET}"

install_directory="${1:-${HOME}/.local/bin}"

if [[ "$(uname -s)-$(uname -m)" != "Linux-x86_64" ]]; then
    echo "No pinned download for $(uname -s) $(uname -m); build it with:" >&2
    echo "  cargo install --locked shuck-cli --version ${SHUCK_VERSION}" >&2
    exit 1
fi

work_directory="$(mktemp -d)"
trap 'rm -rf "${work_directory}"' EXIT

curl --proto '=https' --tlsv1.2 --fail --silent --show-error --location --output "${work_directory}/${SHUCK_ASSET}" "${SHUCK_URL}"
echo "${SHUCK_SHA256}  ${work_directory}/${SHUCK_ASSET}" | sha256sum --check --quiet
tar -C "${work_directory}" -xJf "${work_directory}/${SHUCK_ASSET}"
mkdir -p "${install_directory}"
install -m 0755 "${work_directory}/${SHUCK_ASSET%.tar.xz}/shuck" "${install_directory}/shuck"
echo "Installed shuck ${SHUCK_VERSION} in ${install_directory}"

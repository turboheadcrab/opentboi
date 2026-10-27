#!/usr/bin/env bash
# Writes a release version into Cargo.toml and into the lock file's entry for this package, in the
# working tree only, so the binary and the tarball carry the version the release is made under
# (the repository's own Cargo.toml version is not kept in step with the tags).
#
#   scripts/stamp-version.sh X.Y.Z
set -euo pipefail

if [[ $# -ne 1 || ! "$1" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    echo "usage: $0 MAJOR.MINOR.PATCH" >&2
    exit 2
fi
version="$1"
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

# The package's own version is the first `version = ` line of Cargo.toml.
sed -i "0,/^version = \".*\"/s//version = \"${version}\"/" "${root}/Cargo.toml"
# In Cargo.lock it is the line after `name = "opentboi"`.
sed -i "/^name = \"opentboi\"\$/{n;s/^version = \".*\"/version = \"${version}\"/}" "${root}/Cargo.lock"
echo "Stamped ${version}"

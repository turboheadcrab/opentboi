#!/usr/bin/env bash
# Prints the next release version, worked out from the pull request titles merged since the last
# release tag (`vX.Y.Z`). Run it in the repository, with the tags fetched:
#
#   scripts/next-version.sh
#
# Every squash-merged commit is titled `TYPE: what changed (#N)`, TYPE being one of
#   breaking  a change that breaks what players or users relied on: the next major version (while the
#             major version is 0, the next minor one)
#   feature   something new: the next minor version
#   fix       a bug fixed: the next patch version
#   chore     upkeep (dependencies, docs, CI, refactoring): no release by itself
# The biggest bump among the commits wins. Without any tag yet, the first release is the version
# Cargo.toml names. Exit status 3 (and no output) means nothing since the last tag calls for a release.
set -euo pipefail

readonly NO_RELEASE_STATUS=3

root="$(git rev-parse --show-toplevel)"
last_tag="$(git describe --tags --abbrev=0 --match 'v[0-9]*.[0-9]*.[0-9]*' 2> /dev/null || true)"

if [[ -z "${last_tag}" ]]; then
    sed -n 's/^version = "\(.*\)"/\1/p' "${root}/Cargo.toml" | head -n 1
    exit 0
fi

IFS=. read -r major minor patch <<< "${last_tag#v}"

# How big a change the commit subjects on standard input call for: 3 breaking, 2 feature, 1 fix, 0 none.
biggest_bump() {
    local subject rank
    local best=0
    while IFS= read -r subject; do
        case "${subject}" in
            breaking:* | breaking\(*\):*) rank=3 ;;
            feature:* | feature\(*\):*) rank=2 ;;
            fix:* | fix\(*\):*) rank=1 ;;
            *) rank=0 ;;
        esac
        if [[ "${rank}" -gt "${best}" ]]; then
            best="${rank}"
        fi
    done
    echo "${best}"
}

case "$(git log --format=%s "${last_tag}..HEAD" | biggest_bump)" in
    3)
        if [[ "${major}" -eq 0 ]]; then
            echo "0.$((minor + 1)).0"
        else
            echo "$((major + 1)).0.0"
        fi
        ;;
    2) echo "${major}.$((minor + 1)).0" ;;
    1) echo "${major}.${minor}.$((patch + 1))" ;;
    *) exit "${NO_RELEASE_STATUS}" ;;
esac

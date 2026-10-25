#!/usr/bin/env bash
# Measures clean and incremental build times in a separate target directory and appends a
# Markdown table row per measurement to the given results file (default: stdout only).
# FRONTEND=sdl3 (default) or FRONTEND=bevy picks which frontend is compiled in.
# TARGET_CPU=x86-64-v3 (any rustc `-C target-cpu` value; default: rustc's baseline) compares CPU levels.
set -euo pipefail

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
readonly PROJECT_ROOT
readonly FRONTEND="${FRONTEND:-sdl3}"
readonly TARGET_CPU="${TARGET_CPU:-}"
readonly CPU_SUFFIX="${TARGET_CPU:+-${TARGET_CPU}}"
readonly BENCHMARK_TARGET_DIRECTORY="${PROJECT_ROOT}/target/build-benchmark-${FRONTEND}${CPU_SUFFIX}"
readonly CARGO_FEATURES=(--no-default-features --features "${FRONTEND}-frontend")
readonly TOUCHED_SOURCE_FILE="${PROJECT_ROOT}/src/main.rs"
readonly RESULTS_FILE="${1:-}"

measure() {
    local label="${FRONTEND}${TARGET_CPU:+, ${TARGET_CPU}}: $1"
    shift
    local started finished
    started=$(date +%s.%N)
    "$@" > /dev/null 2>&1
    finished=$(date +%s.%N)
    local seconds
    seconds=$(echo "${finished} - ${started}" | bc)
    local row
    row=$(printf '| %s | %s | %.1f s |' "$(date +%F)" "${label}" "${seconds}")
    echo "${row}"
    if [[ -n "${RESULTS_FILE}" ]]; then
        echo "${row}" >> "${RESULTS_FILE}"
    fi
}

cd "${PROJECT_ROOT}"
export CARGO_TARGET_DIR="${BENCHMARK_TARGET_DIRECTORY}"
if [[ -n "${TARGET_CPU}" ]]; then
    export RUSTFLAGS="-C target-cpu=${TARGET_CPU}"
fi

rm -rf "${BENCHMARK_TARGET_DIRECTORY}"
measure "debug, clean" cargo build "${CARGO_FEATURES[@]}"
measure "debug, no changes" cargo build "${CARGO_FEATURES[@]}"
touch "${TOUCHED_SOURCE_FILE}"
measure "debug, touched main.rs" cargo build "${CARGO_FEATURES[@]}"
measure "test build (debug), after debug build" cargo test --no-run "${CARGO_FEATURES[@]}"
measure "release, clean" cargo build --release "${CARGO_FEATURES[@]}"
touch "${TOUCHED_SOURCE_FILE}"
measure "release, touched main.rs" cargo build --release "${CARGO_FEATURES[@]}"

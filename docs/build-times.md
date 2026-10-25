# Build times

Measured with `scripts/benchmark-build.sh` (separate target directory, nothing else compiling).

Machine: a current AMD desktop CPU (8 cores / 16 threads), 32 GB RAM, Fedora 44, rustc 1.99.0,
Bevy 0.20, default linker (GNU ld via cc). Dev profile builds dependencies with `opt-level = 3`.

| Date | Build | Time |
|---|---|---|
| 2026-10-11 | debug, clean | 194.6 s |
| 2026-10-11 | debug, no changes | 0.2 s |
| 2026-10-11 | debug, after touching `src/main.rs` | 2.2 s |
| 2026-10-11 | test build (debug), after a debug build | 2.7 s |
| 2026-10-11 | release, clean | 118.9 s |
| 2026-10-11 | release, after touching `src/main.rs` | 2.2 s |

After replacing Bevy's `2d` feature bundle with only the features the game uses (407 → 338 Linux
dependency crates):

| Date | Build | Time |
|---|---|---|
| 2026-10-11 | debug, clean | 156.3 s |
| 2026-10-11 | debug, no changes | 0.2 s |
| 2026-10-11 | debug, after touching `src/main.rs` | 1.6 s |
| 2026-10-11 | test build (debug), after a debug build | 5.2 s |
| 2026-10-11 | release, clean | 93.6 s |
| 2026-10-11 | release, after touching `src/main.rs` | 2.0 s |

## Reading the numbers
- Day-to-day edits rebuild in about 2 seconds, so iteration speed is fine today.
- Clean builds are dominated by Bevy's dependency crates (338 compiled on Linux; `Cargo.lock`
  lists ~580 because it also records Windows/macOS-only crates that never build here). The debug clean build is slower than
  release because dependencies are optimized at `opt-level = 3` in both, while debug also emits
  debug info.

## Options if it gets slow
- Faster linker: `mold` or `lld` (both packaged in Fedora 44), set via `.cargo/config.toml`.
  Mostly helps incremental builds as the binary grows.
- Bevy's `dynamic_linking` feature for dev builds (links Bevy as one shared library).
- Lower dependency optimization in dev (`opt-level = 1`) to speed clean builds at some runtime cost.
- A build cache (`sccache`) to share compiled dependencies between clean builds.
- Leaving Bevy (see the SDL3 discussion) would remove most of the clean-build time.

## How the build uses the CPU (2026-10-11, clean release build with `cargo build --timings`)
- Cargo ran up to 16 compile jobs at once (`jobs=16 ncpu=16`, peak 18 units including build
  scripts). Wall time 90.3 s, CPU time 1226 s user + 59 s system → about 14 of 16 hardware threads
  busy on average; Cargo's own sampling shows 96% average CPU use.
- Within a crate, code generation is split into codegen units (16 in release, 256 in debug) that
  LLVM compiles on separate threads. Type checking and borrow checking for one crate run on one
  thread (stable rustc; a parallel front end exists only on nightly as `-Z threads=N`).
- The last ~17 s (73.5 s → 90.3 s) run with at most two units active: the dependency chain
  `bevy_render` → … → our crate, then linking, which GNU ld does on one thread.
- Slowest crates: wesl 38.5 s, naga 34.8 s, bevy_ecs 34.7 s, wgsl-parse 31.1 s, x11rb-protocol
  28.3 s, wgpu-core 28.0 s, bevy_reflect 27.7 s.

## CPU instruction sets (measured 2026-10-15)
`scripts/benchmark-build.sh` with `TARGET_CPU=...` (SDL3 frontend only, so no Bevy; same machine as
above; each level in its own target directory, nothing else running):

| Build | `x86-64` (baseline, v1) | `x86-64-v2` | `x86-64-v3` |
|---|---|---|---|
| debug, clean | 45.1 s | 48.0 s | 46.5 s |
| debug, touched `main.rs` | 4.5 s | 4.5 s | 4.7 s |
| test build (debug), after a debug build | 10.6 s | 11.1 s | 10.9 s |
| release, clean | 29.1 s | 29.3 s | 29.1 s |
| release, touched `main.rs` | 3.7 s | 3.7 s | 3.9 s |

- **The target level does not change build time**: differences are within run-to-run noise (about
  ±3 s). The compiler does the same work whichever instructions it emits. So v3 cannot be used
  "to build faster and ship v1". What does speed up builds is unrelated to the target: `sccache` or
  CI caching of `target/`, `mold`/`lld`, fewer dependencies.
- Build *on* any machine for any level: `-C target-cpu` is a code-generation choice, not a
  requirement on the build host (cross-level builds need no emulation).
- **Run time** (`scripts/benchmark-runtime.py`, the same desktop CPU, 144 Hz display, release binaries,
  standing in the showcase room). A first round with vsync on found the same frame times for v1
  and v3 (6.99 against 6.98 ms, 143 fps). **That proves nothing about speed**: with vsync a frame
  takes one refresh (6.94 ms) whenever the CPU needs less than that, so any build that is fast
  enough shows the same number. It only shows that neither build misses a refresh on this PC. The
  useful measurements are the ones that remove the cap, `--no-vsync` (4 runs of 6 s, two rounds,
  alternating order):

   Build | first frame | mean frame, uncapped | CPU use |
  ---|---|---|---
   v1 `x86-64` | 1430 / 1416 ms | 0.36 / 0.37 ms (2752 / 2732 fps) | 250 / 248 % |
   v3 `x86-64-v3` | 1359 / 1342 ms | 0.34 / 0.36 ms (2940 / 2813 fps) | 238 / 236 % |

  v3 starts about 70 ms (5 %) sooner (the first frame follows the loader that rasterises the
  vector art on 14 threads) and draws 0 to 6 % more uncapped frames per second; the second round's
  frame difference is inside run-to-run noise. Uncapped frame time here is mostly the GPU and the
  presentation path, not our CPU code. (The first round measured 1513 against 1324 ms at start-up
  with an older build; the spread between sessions is larger than that gap, so call it "a
  few percent, 0.1 to 0.2 s".) Memory is the same. The real question, how a *weak* CPU
  behaves, needs a weak CPU; this PC cannot answer it.
- Why only the start-up: `tiny-skia` (the rasteriser) chooses its SIMD code with
  `#[cfg(target_feature = "...")]`, i.e. **at compile time**, not by detecting the CPU. A v3 build
  has about 12,600 AVX2 instructions in 537 functions (`tiny_skia::pipeline`, our image
  decoding, the SWF reader); the baseline build has 276, all in `memchr`, `simdutf8` and
  `simd-adler32`, which detect AVX2 at run time and so run fine on old CPUs
  (`objdump -d | grep ymm`, grouped by function).
- Therefore the baseline binary uses SSE2 only and runs on every x86-64 CPU (a Core 2 Quad Q6600
  is v1: it has SSSE3 but no SSE4). Shipping a faster binary for newer CPUs would mean a second
  build and a choice at install time (`install.sh` could read `/proc/cpuinfo`), worth doing only
  if a later measurement on weaker hardware shows more than this 0.2 s.
- `-C target-cpu=native` is fine for local builds but makes binaries that may crash on older CPUs.

# OpenTBOI

Open-source Rust engine for the original *The Binding of Isaac* (Steam AppID 113200) that derives its
assets from the user's own Steam install at runtime. Never commit or ship game assets or the SWF.

## Layout
- `src/game/` — engine-independent rules; the API every frontend uses (see its module docs).
  Grouped into `player/`, `pickups/`, `room/` (grid, contents, layouts, doors, room changes),
  `floor/` (generation, traversal), `attacks/` (tears, bombs) and `hud/`, plus shared basics
  (units and ticks, drawing, input, sounds, textures, parameters). State is kept in the
  original's units, typed: `Frames`, `Pixels`, `PixelsPerFrame`, `PixelsPerFrameSquared`,
  `Retention`, `Hearts` (`original_units.rs`); a room's place on the floor map is a `FloorCell`; positions and velocities are `Vec2` in pixels and pixels per
  frame. A tick advances `30 / tick rate` frames (default 30 ticks, the original's rate). Must not depend on Bevy or any other engine crate; math comes from `glam`.
- `src/frontend/` — the only engine code: `sdl3_frontend/` (default, feature `sdl3-frontend`) and
  `bevy_frontend/` (feature `bevy-frontend`), each doing the same job behind `frontend::run`.
  What they share lives beside them: `controls.rs` (bindings, input combination),
  `sound_variants.rs`, `frame_statistics.rs`.
  Build one with `--no-default-features --features bevy-frontend`, or both with
  `--all-features` and pick with `--frontend`. Building SDL3 needs `SDL3-devel`.
- `src/game_assets/` — Steam discovery, reading the SWF from the installed game's files,
  and conversion into engine-neutral sounds (`SoundBank`) and images (`TextureBank`, via the
  `vector_art` renderer). `vector_mesh` turns shapes into triangles (Ruffle's tessellator behind
  our `ShapeMesher`); `script` reads the original's AVM1 bytecode into statements.

## Other files
- `docs/` — design notes and measurements (`display-modes.md`, `build-times.md`,
  `texture-streaming.md`, `frontend-comparison.md`, `parameter-extraction.md`,
  `vector-rendering.md`, `door-frames.md`, `enemies.md`, `original-coverage.md`, `steam-deck-testing.md`).
  `distribution-options.md` holds the publishing decisions.
- `README.md`, `LICENSE` (GPLv3) and `THIRD-PARTY-LICENSES`; the last is generated (git-ignored) by
  `build.rs` via `cargo about` (`about.toml`, `about.hbs`), so don't edit it by hand. Any SWF is
  git-ignored.
- `packaging/linux/` (launcher, icon, `install.sh` that go into the tarball) and
  `scripts/package-linux-tarball.sh` (builds `target/dist/opentboi-VERSION-linux-ARCH.tar.zst`,
  independent of the dev launchers). CI lives in `.github/workflows/`.
- `scripts/steamos-disk-test.sh stage|run|unstage` (with `scripts/steam-shortcuts.py`) — puts the
  tarball, a copy of the game and a Game Mode shortcut on a SteamOS disk mounted in this PC and runs the
  binary in that disk's userland; see `docs/steam-deck-testing.md`.
- `.shuck.toml` — shell lint with every shuck rule on (minus a few style policies); check scripts with
  `shuck check scripts packaging .github`.
- `scripts/build-tarball-in-container.sh` — the release tarball built locally in an Ubuntu 22.04
  container (as CI does), into `target/container-dist/`; for testing on a Deck without CI.
- `scripts/benchmark-build.sh [results.md]` — clean and incremental build timings in a separate
  target directory (`target/build-benchmark-$FRONTEND[-$TARGET_CPU]`, `FRONTEND=sdl3|bevy`,
  optional `TARGET_CPU=x86-64-v3`); don't edit sources while it runs.
- `mise.toml`, `distrobox.ini`, `scripts/dev-run.sh` — the development tasks (`mise tasks` lists
  them) and the Fedora container they run in on immutable systems.
- `scripts/install-desktop-entries.sh` — builds both frontends, copies the binary to
  `target/desktop/` and installs a launcher per frontend.
- `scripts/original-coverage.py [--top N] [--function NAME] [--suggest]` — how much of the
  original's scripts our code covers (the game's, cited as `DoAction N`, and the main timeline's,
  cited as `Timeline N`: sounds, music, saving, the main loop; a timeline function sharing a name
  with a game one is `timeline_NAME()`), per gameplay line, a strict count and an upper bound,
  fully covered functions, blind spots. Its text output is for local use; don't paste it into files.
  `--svg docs/original-coverage-map.svg` writes the tracked coverage map (one square per gameplay
  line, only function names and counts, none of the original's code); regenerate it when coverage
  moves noticeably.
- `scripts/benchmark-runtime.py BINARY LABEL` — frame times, CPU, memory, GPU memory over timed runs
  (uses the game's `--exit-after`).

## Reference material (derived from the user's copy, never committed)
- `downloads/` (git-ignored) — the browser demo SWF; the game should work from it to the latest
  release.
- `~/.cache/opentboi/preview/` — FFDec renders used as visual references.
- Gameplay values and formula coefficients belong in `src/game/parameters.rs`
  (`GameParameters`), not as constants scattered through the game code.
- Never put the original's code in a committed file (source, comments, docs, tests). Describe
  behaviour in your own words.

## Code conventions
- Change files with the Edit and Write tools, not with scripts fed to an interpreter through a heredoc
  (`python3 - <<EOF`): the chat shows the diff of the former only. A project hook
  (`.claude/hooks/no-script-edits.sh`, local) refuses the latter.
- Source files stay at or below 250 lines (enforced by `tests/source_file_length.rs`); split modules
  before they grow past it. Functions stay at or below 50 lines (`clippy::too_many_lines`).
- Full, descriptive names for variables, parameters and fields; no abbreviations like `dt`, `pos`,
  `e`. Only `x`, `y`, `z` may be single letters (`clippy::min_ident_chars`, see `clippy.toml`).
- No magic strings or meaningful magic numbers: put them in named constants.
- Clippy's `pedantic` group is on (`Cargo.toml`), with the few exceptions listed there. Float-to-integer
  conversions go through `src/whole_numbers.rs`; integers convert with `try_from`, not `as`. An
  `#[allow]` names its `reason`.
- Quantities get their dedicated type, not a bare `f32`/`u8`: durations, ages and counters of
  original frames are `Frames` (never a plain number of frames or seconds), distances `Pixels`,
  speeds `PixelsPerFrame`, accelerations `PixelsPerFrameSquared`, per-frame decay `Retention`, health
  `Hearts`, a room's place on the floor map `FloorCell`; positions and velocities are `Vec2`. Things
  with a closed set of values get an enum or newtype (`Card`, `Pill`, `Consumable`, `Item`), never a
  raw number from the original (`col`, a frame index). Convert at the edge (`Frames::from`,
  `.get()`), and when a needed unit type is missing, add it to `original_units.rs` first. Random
  draws go through `Random` (`below`, `index`, `centered`), not raw floats.
- Versions are pinned and moved on purpose: `Cargo.lock`, the Rust toolchain (`rust-toolchain.toml`,
  an exact version; bump it with the fixes the newer Clippy asks for, in one commit), the shell
  linter (its version in `.github/workflows/ci.yml`) and the GitHub Actions (by commit).
  Dependabot proposes the crates' and Actions' updates a week after release; the weekly "Latest Rust"
  workflow tells when a newer compiler would break the build.
- Before finishing: `cargo fmt`, `cargo clippy --all-targets --all-features` with no warnings (also
  once per single frontend), `cargo test`, and `shuck check scripts packaging .github` when a shell
  script changed.
  Then regenerate the desktop launchers with `scripts/install-desktop-entries.sh` (a release build of
  both frontends copied to `target/desktop/`), so the shortcuts run the latest code.
  `mise run check` does all of that, in the development container where there is one (below).
- Build and test through the `mise run` tasks (`mise.toml`): on machines without the build packages
  (Bazzite and other immutable systems) they run in the distrobox from `distrobox.ini`
  (`mise run dev-box` creates it; `scripts/dev-run.sh` picks the container or the host). Never
  layer packages onto an immutable host (`rpm-ostree install`). The Rust version is the one
  `rust-toolchain.toml` pins; `scripts/dev-run.sh` has rustup install it when it is missing.
  Tests that need the real extracted SWF are `#[ignore]`d; run them with `cargo test -- --ignored`.
  After adding or resizing a texture, run `cargo test texture_extents -- --ignored --nocapture` to
  check its canvas holds all of its art.

## Local notes (untracked)
- `PROGRESS.md` — running log of actions and findings.
- `HOST_INSTALLS.md` — everything installed on the host and how to remove it; update it whenever
  something is installed. Prefer distro packages; verify checksums for anything downloaded manually.
- `TODO.md` (tracked) — deferred work.
- `VERIFIED.md` (tracked) — checklist of what is confirmed working, by the user or automated;
  update it whenever something is verified.

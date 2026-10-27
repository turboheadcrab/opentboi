# TODO (later, not now)

## Next priority: a `Marker` enum
- The room-clear and machine rolls name what to spawn by a raw frame of the original's pickup clip
  (`CARD_MARKER_FRAME`, `TRINKET_MARKER_FRAME` and the other `*_FRAME` constants, plain `u8`s in
  `pickups/kind.rs`, `rewards.rs`, `machines/payout.rs` and their tests, about 40 places). Replace them
  with a `Marker` enum (the closed set of kinds a marker can ask for), converted to and from the
  frame at the edge only; do it as a refactor of its own before more kinds are added.

## Enemies: after the flies
- Knockback from blasts (half the speed kept, then pushed away); the blast already has to see them.
- Other enemy markers are still skipped: rooms holding only those keep the timed clear-reward
  stand-in and open at once.
- The unicorn's contact damage ignores its 30-frame cooldown per enemy (`aidsdone`): 40 damage
  kills any fly anyway.

## Gapers: follow-ups
- Walkers' tile marks (0.9, fading) are made and cost `linecheck()` its sight; not yet: a mark ahead
  slowing a walker to 0.7 (`pathfind()` every third frame) and marks steering the flood fill,
  champions; all listed in `docs/enemies.md`.
- The fly types are still named for flies (`FlyKind`, `Enemies::flies`) though the Pooter shoots;
  the walkers are `Walker`s now (Gaper, headless body, Horf).

## Stats panel
- Show the optional rooms (LIB, SAC, CUR, MIN) with their minimap icons instead of text.

## Enemies: next
- See `docs/enemies.md` for every Basement enemy, its marker, clip and how often rooms place it;
  only the chain worm (19, three linked segments in one room) is still missing; the Hopper, the red
  fly, Moter and the flesh mound are in. Clotty and Mulligan are in; not ported for them: the Mulligan's
  refuge cell memory (`checked`), the Mulligan bumping bodies as well as walls (`gh`; only walls and
  rocks now), champions. `deathscripts()` is ported only for the Mulligan's flies; the rest is boss
  bars, drops of special rooms and bosses' deaths.
- The Devil card also gives Isaac's head and body a devil look for the room (look mode 8); only
  the dark tears are done.
- Champions (`special`), the red hit flash, knockback from blasts.
- Line checks still approximated: the Mulligan looks with `linechecky()` (`in_sight`) where
  `angstfind()` uses `linecheckx()` (one point every 10 px, `pff3()`), and `in_sight` itself only
  asks whether a corner is in something solid, not `pff3x1()`'s 2.5 credit (pits ignored, values
  between 0.3 and 1 costing 0.1). Both can use `room::sight::SightGrid`.
- Blood (`splater()`, see `docs/enemies.md`): the other colors (`spl` 10, 20, 30), dark rooms'
  tint and their blast splats, bomb tears' 20 splats, other characters' and items' blood, bosses'.
  Splats and craters share one bitmap in the original, so a crater can lie under later blood; ours
  always draws craters above. The blood clip's last frame is kept per room, not for the whole game.
- The two-shot Pooter's spread shots pass either side of a player standing still at range; that
  is the original's aim, not a bug.

## Death follow-ups
- The death note's "Back to main menu" does nothing (there is no menu); Space replays.
- Not yet: the death count (`so.data.deads`, its unlock at 100), the music fading out
  (`dmusic`), items on the note (`i0`–`i17`, once items exist), other characters' names
  and floors (`nam`, `loco`), killers we do not have yet (a red fireplace gives 5).
- While dying the original still runs `playcol()` (pickups and the like); we leave Isaac's touch
  as it is. Isaac's death clip (78 frames) renders on first use, like his other animations.

## Input between ticks (assessment R03): a choice
The original polls every key once per frame (`Key.isDown(...)`; only the mouse button has a
listener), so a tap shorter than a frame is lost there too; `--input original` (default) does the
same. `--input latched` keeps presses seen between ticks for the next tick. Both work on what the
frontend samples once per drawn frame; a tap shorter than a drawn frame would need the frontends
to latch key events too. The card button deliberately differs from the original either way: we use
a card on a new press, the original whenever Q is down.

## Display server fallback
winit 0.30 picks Wayland only if `WAYLAND_DISPLAY`/`WAYLAND_SOCKET` is non-empty, then X11 via
`DISPLAY`, else exits with "neither WAYLAND_DISPLAY nor WAYLAND_SOCKET nor DISPLAY is set".
libwayland itself defaults to `wayland-0`, so on a Wayland-only system launched without the
variable (ssh, systemd service, some sandboxes) the game fails although the socket exists.
- Option: at the top of `main`, if neither variable is set and `$XDG_RUNTIME_DIR/wayland-0`
  exists, set `WAYLAND_DISPLAY=wayland-0` before anything spawns threads.
- Decide whether that is worth it, and test a real X11-only session.

## Simple installation everywhere
Decisions and status are in `docs/distribution-options.md`.
- Done: Linux tarball (`scripts/package-linux-tarball.sh`, `packaging/linux/`, `install.sh`), CI
  and release workflows (never run yet), `bundled-sdl3` feature (untested here).
- Next: run the workflows and fix what breaks; test the tarball on a Deck, on Bazzite and on old
  hardware (Core 2 Quad, Sandy Bridge, Radeon HD 5770, GTX 660); Flatpak (own repo first; make the
  window `app_id` configurable); `.deb`/`.rpm` on the release page; AUR; an optional Decky plugin
  as a convenience wrapper; Windows and macOS.
- Support only distributions their vendors still patch (policy table in the distribution doc).
- AppImage (when we make one): it must run on systems **without FUSE 2** (`libfuse.so.2`), i.e.
  Fedora and Ubuntu 24.04 by default. Build it with the static type-2 runtime
  (`AppImage/type2-runtime`, no `libfuse2` needed; check the tool actually embeds it, e.g. by
  running the result in a Fedora container that has only FUSE 3), keep `--appimage-extract-and-run`
  working as the fallback, and add a CI job that starts the AppImage in a FUSE-3-only container.
- If we ever ship a v2/v3 build: choose the binary at install time from `/proc/cpuinfo` (a v3
  binary on a v1 CPU dies with "Illegal instruction" and no message; see
  `docs/build-times.md`). Also worth building: log CPU time per frame and our own
  update-plus-draw time, so CPU-level comparisons don't depend on vsync.

## Vector art pipeline
Room floors, walls and most sprites are vector shapes, so the game needs its own rasterizer;
FFDec is only a dev tool and cannot be assumed on players' machines.
- Chosen: Ruffle's `swf` crate + rasterizing directly with `tiny-skia` (resvg's backend), skipping
  the SVG text step. Alternative kept in mind: tessellate with `lyon` for GPU, resolution-independent.
- Main difficulty in all options: Flash shapes store edges with left/right fill styles, which must
  be rebuilt into closed fill paths (Ruffle's renderer does this; study it).
- Done for the room background (`src/game_assets/vector_art`: own edge-to-contour rebuild +
  `tiny-skia`). Still missing: bitmap fills, other frames (chapter styles), color transforms, masks,
  morph shapes, text. Compare against FFDec renders in `~/.cache/opentboi/preview/`.
- Graphics settings (edge quality, texture cap, frame cap): deferred; best settings for now (user).
- GPU meshes: the SDL3 frontend draws every frame family whose shapes have no gradient as meshes
  (overlays, doors, Isaac's animations, pickups, effects, machines, card banners; solid and bitmap
  fills, 2× supersampled): 314 textures instead of 2096 at start-up, 134 MiB of video memory
  instead of 383 (re-measured 2026-10-15; `vector_families.rs`). What is left, measured with
  `cargo test split_measurement -- --ignored --nocapture` and `vector_drawable_share`:
  - Only gradients block frames now (no masks): 754 of Isaac's 1948 on-demand frames and 18 of 148
    door frames. Their gradient shapes are wholly gradient (no shape mixes fills), so the whole
    remaining cost is "draw a gradient fill". About 113 MiB of Isaac frames (if every one were
    resident) and 7 MiB of doors. All 772 gradients are the same kind: one two-stop, pad, radial
    gradient without a focal point per frame (a soft blob), and there are exactly two distinct ones
    (doors: dark red, alpha 128 to 0; Isaac: dark blue, alpha 130 to 3, first stop at ratio 124).
    Cheapest fix: bake one small radial ramp texture per distinct gradient and draw those shapes
    through the bitmap-fill path (texture coordinates from the fill matrix); about a day.
    A hybrid with a cropped texture per gradient
    shape (estimated 33 MiB and 0.8 MiB) is more work (it splits the painter's layers) and saves
    less.
  - Static pictures drawn all the time stay textures on purpose: room backgrounds (12, about 32
    MiB), the stage surround (about 34 MiB), the HUD. Drawing the surround as meshes was tried
    (2026-10-15, not re-run) and is worse: 156 instead of 134 MiB of video memory, 99th-percentile frame 13 ms
    instead of 7.3 ms (earlier run), uncapped 0.55 instead of 0.37 ms per frame. Crop it or draw it in slices
    instead.
  - The active-item box's "SPACE" label is a shape baked into the HUD clip (one shape above the
    box on the clip's first frame), so it can't follow rebinding or show a gamepad button:
    skip that shape and draw our own label or glyph (by position in the clip, not by id).
  - Still open: text, 4×4 quality like Flash HIGH without 16× memory (e.g. SDL3 GPU API with MSAA),
    and the Bevy frontend (still uses textures for everything).

## Fullscreen key
The original toggles fullscreen with F. Decide whether to bind F as well as (or instead of) F11.
F does not clash with the current bindings (WASD move, arrows shoot). Consider making it rebindable.

## Soundtrack music
The game's own music plays from the SWF (`game/music.rs`). Still to do: boss music (`bossintro`, `m-b0`/`m-b1`, the win jingles `isaacbosswin` and `ambushwin`) with the boss intro screen, mini-boss and challenge room music (`m6`), the jingles of the challenge room, the holy room and a devil item taken (they duck the music like the others), the other chapters' tracks, the credits and title music, and the music's mute setting. SDL decodes a whole track when it starts (about 70 ms for the Basement's loop): decode the loop apart from the intro if that hitch is felt.
Alternatively use the user's soundtrack: Steam installs the soundtrack DLC inside the game's own folder:
`steamapps/common/The Binding Of Isaac/The Binding of Isaac Soundtrack/` holds 39 MP3s named `Danny
Baranowsky - The Binding of Isaac - NN Title.mp3` (01 "Those Responsible" … 39 "The Wages of Faith") and
`cover.jpg` (found on the user's PC, 2026-10-17). Map track names to the music events before preferring
them, and never ship any of it.

## Primitive matching
`original_arithmetic_in_our_code` (see `docs/original-coverage.md`) judges each original function against
its counterparts (named in our doc comments) since 2026-10-17, calibrated on `ender()`, `crand()`
(complete), `spawnb()` (10 of 11) and `slotf()` (8 of 8 since the Lucky Foot, 2026-10-18). Still to do: name the
branch a counterpart ports, so a one-branch port of a big function is judged against that branch; see
per-axis vector maths and array index arithmetic; record the result as an evidence level in the coverage
tags; resolve more of our names (method results, `parameters.x.y` chains, enum tables); run it against
other builds. Sound names and health in the original's units (`Hearts`) are done (2026-10-18).
- A doc comment that marks a statement of ours as an equivalent (`spawnb()`'s lazy grid, `crand()`'s
  pair), so the matcher counts it instead of the code having to copy it.
- `scripts/original-coverage.py`'s lower bound counted lines our comments quoted; with the quotes gone
  (2026-10-18) it fell from 6.1 % to 3.7 %. Base it on line citations (`DoAction 26277`) and the
  matcher's findings instead.
- The line checks and blood (2026-10-26: `room/sight.rs`, `room/blood.rs`, `floor/blast_sight.rs`,
  `linecheckxx()`, `linecheck()`, `pff3()`, `splater()`, `actions2()`'s fading, `pathfind()`'s marks)
  were read from the bytecode on a machine without FFDec's dump, so they name functions but cite no
  `DoAction` lines yet; add the citations and regenerate `docs/original-coverage-map.svg`.

## Items
The item model exists (`pickups::item::Item`, a pedestal holding one, taking it with the choir, the
name banner and Isaac holding it up); the Lucky Foot and My Little Unicorn are in it, on pedestals in
the start room (not in the original: a way to try items until the treasure rooms give them). The
active slot (`player::active_item`), its use (Space or the left trigger), the charge per cleared room
and the HUD box are made. Still open:
- Swapping active items: the old one goes back on the pedestal and cannot be retaken until Isaac
  steps off (a stand-in for the pedestal's swap animation, frame 10); untested in play, as only one
  active item exists.
- The ride's colour flashes are not applied while Isaac holds something up (the original tints his
  whole clip); the active item's charge is kept per item, as the original does, but there is only one.
- The Gamekid (item 93): the same ride with the Pac-Man head and fleeing enemies.
- The other items, and the random ones the treasure room, golden chests and machines give
  (`giveit()`); their effects; the pickup jingle by room (boss, treasure, shop) and the subtitle
  (`st22()`) for items that have one.
- The Portable Slot (active item 137, the arcade flag of `slotf()`): `slot_result` already takes it.
- The Lucky Foot's other uses: pills (none yet), and the beggar's counter step the original sets and
  overwrites at once (no effect, left out).

## Sound polish
- Flash MP3s carry ~1,677 samples (~38 ms) of encoder delay (the SWF "seek samples" field); trim it
  if shots feel late.
- Original likely picks variants at random; we cycle through them.

## Gameplay values still approximated
Movement, fire delay, tear speed/range/size and mouse aiming now follow the original's script.
Still approximated: the player's collision radius and placeholder body; the small momentum
bonus in movement (`v1 = 3 + momentum / speed`). Keyboard shooting priority now follows `lastx`/`lasty`.

## Controls hint: remaining work
Done (2026-10-15): the start room's bomb hint shows `E` or `LB` by the last-used device. Left: the HUD's `SPACE` label (baked into the active-item clip)
has no pad variant; the name should come from `SDL_GetGamepadType()` (`L1`, `L`) instead of a fixed
`LB`; try a real PlayStation/Switch pad or SDL's virtual joystick; touchscreen input is untested for
the device rule; the Bevy frontend's pad path was not run.

## SDL's KMSDRM backend (no desktop, straight to the display)
Never run. The release binary's static SDL3 is built with it (`libgbm` and `libdrm` are in the build
container), so there is nothing to build, only to try and decide on:
- Try: from a text console (Ctrl+Alt+F3, logged in, in the `video`/`render` groups or on a logind seat)
  `SDL_VIDEODRIVER=kmsdrm opentboi --fullscreen`; note the picture size, whether Vulkan or only
  OpenGL ES works, whether pads and keys work without a compositor, and how it exits (Escape).
- Check what the frontend assumes about a window: display-mode and scale queries
  (`docs/display-modes.md`), the cursor, `--window-size`.
- Decide: support it (document, add a test line to `docs/steam-deck-testing.md`) or leave it as an
  untested bonus for console-style setups (Batocera/Lakka-like, kiosks). Not needed for SteamOS or
  Bazzite: Game Mode is gamescope, a Wayland compositor.

## Controller and key glyphs
Done for the bomb hint on the start screen (`src/game_assets/input_glyphs.rs`): the left-shoulder picture
of Kenney's CC0 "Input Prompts" 1.5a, per make (Xbox `LB`, PlayStation `L1`, Switch `L`, Steam Deck `L1`),
chosen from the pad's make at run time, ink-coloured like the drawing. Still open:
- Done since: the item label and all four start-room hints (D-pad, face buttons by position, LB, LT).
- Compare with Rebirth's own glyphs for PlayStation, Switch and the Deck; only its Xbox and keyboard
  hints have been seen (no other pads here). Ways to get them to show, easiest first:
  1. The console releases draw their own pad's buttons: screenshots or videos of the PS4/PS5 and
     Switch versions' first room and HUD.
  2. A virtual pad with another make's USB ids: on Linux a uinput device (python-evdev) that copies the
     Flydigi's input under a DualShock 4, DualSense or Switch Pro vendor and product id, with Steam Input
     turned off for Rebirth so the game sees the device itself. Check first whether Rebirth (Repentance)
     ships PlayStation or Switch glyphs on PC at all.
  3. Borrow a pad.
- Check the glyphs on real PlayStation and Switch pads and on the Deck's own controls; the checks so far
  use SDL's virtual pads (`cargo test virtual_pads -- --ignored`) and `--hints NAME`.
- Under Steam Input the make comes from what SDL reports; the new log line "Gamepad ..." in a diagnostic
  report shows what the Deck sends.

## Not tried
- Start in fullscreen automatically under gamescope (`docs/steam-deck-testing.md`).
- A CI lint step for shell scripts. `.shuck.toml` now selects every shuck rule bar seven style policies; still to do: run ShellCheck on the same files for a side-by-side
  comparison, then add the step (`cargo install shuck-cli` or the `ewhauser/shuck-action` action).
- The start-room hint starting as `LB` when a pad is already open.
- Booting the Deck's SteamOS disk on a PC with a current GPU, or in a virtual machine.

## Several versions of the original (browser demo, Steam release, others)
Coverage must not mean "coverage of one build". The full process (statuses `port`, `equivalent`,
`partial`, evidence levels, tags, lock file, tool) is `docs/original-coverage.md`. Earlier plan:
- `docs/original-builds.md` lists every build we support: name, where it comes from, SHA-256,
  whether its names are obfuscated (the demo's are), and what differs.
- Citations (`function()#n:hash`) refer to one *reference build* (the Steam release). The demo's
  function names are mangled, so names cannot be the key across builds: a local tool matches a
  cited statement to other builds by a structure hash (ignoring names and constants) and reports
  per build: same structure and values, same structure with other values (a parameter table, as in
  `docs/parameter-extraction.md`), or no match (a real behaviour difference).
- First comparison (2026-10-18, by hand on FFDec's text, notes in the git-ignored `local/`): 133
  functions share a name in both builds, 86 of them use different numbers; small ones with real
  behaviour changes include the room-marker alpha rule, the line-of-sight step and partial-cell
  cost, teleport distances, the slowing floor, and a laser sound's cooldown. One function kept its
  name but lost its body to a renamed copy, so builds must be paired by body, not only by name.
- Our decoder returns the demo's functions empty: they are defined inside an obfuscated wrapper
  function. Teaching it that wrapper is the first step of the tool above.
- Committed output is aggregate only (a table of builds and percentages, no statements).
- Code adapts by detecting data (a linkage name or frame label exists), not by version numbers.
- CI has no game files: test the decoder and the matching on SWFs the tests build themselves
  (the `swf` crate can write them) and keep the real-build golden tests `#[ignore]`d.

## Citing the original without quoting it (to build)
Line numbers of the script dump are only reproducible with the same tool version and settings, and
committed files should not quote the original's expressions. Projects that track progress against an
original binary key everything to identifiers from the binary itself: matching decompilations (sm64,
OoT) by function address in a hash-verified release; translation layers by system-function IDs and per-game playability lists.
For us:
- Cite the original as function name, statement position as our own bytecode reader decodes it
  and a short hash of that statement (`balljunk()#412:9f3a`), from a fingerprinted SWF (Steam build
  and SHA-256 recorded once in a doc). The position is our decoder's index, not a text line, so
  anyone with the same build gets the same one; a golden test guards the numbering.
- A local-only tool resolves a citation to the statement for the developer's terminal. Rewrite `scripts/original-coverage.py` to count these
  citations instead of quoted expressions and FFDec text.
- Replace the quoted expressions in committed comments and docs with prose plus citations; add a
  test that rejects ActionScript tokens in comments.
- For behaviour, compare traces against the original running in Ruffle (logged per frame) rather
  than writing a Flash runtime.

## Behaviour written by observation, to replace with the original (audit 2026-10-13)
- Shop hearts: the original sells a red heart at full health (case 15 has no health check; floor
  hearts are refused at 7728). Ours matches; Rebirth refuses. Decide whether to add a choice.
- `balljunk()` parts not ported: case 34 (trapdoor/heaven), case 9, case 10 (pedestal items,
  `powerlevel()`), hard mode's bomb-to-troll roll, the devil deals of case 15 (pay heart
  containers), and everything with enemies.

## In-RAM extraction on low-end systems
Startup currently extracts (~0.2 s, first launch only while the disk cache exists) and loads
(~0.2 s) on a current desktop CPU. Measure on weak hardware (Steam Deck, old laptops) before deciding
whether to drop the disk cache: SWF read + inflate (46 MB), full SWF parse, vector rendering.

## Fullscreen at the original's resolutions
The original switched the display mode (DirectX) to 800x600, 720x480, 640x480, 1280x800, 1280x720
or 1024x768. Windowed sizes are supported now (`--window-size`), and fullscreen uses the
monitor's native mode with the room scaled up. Wayland cannot change display modes, so options:
exclusive video modes where the platform allows (X11, Windows, macOS), or render internally at
the chosen resolution and scale it to the screen (works everywhere, including the Steam Deck).

## Steam achievements
The original unlocks achievements by launching its bundled helper `FlashAchievements.exe <ids>`
(`mdm.Process.create` in `scripts/frame_1/DoAction.as`, around line 1217), which talks to Steam
as AppID 113200. Options to evaluate:
- Run the user's own `FlashAchievements.exe` (Windows binary; Wine/Proton on Linux and macOS) with
  the same arguments, so unlocks go through the original's own Steam integration.
- Use the Steamworks API directly (e.g. the `steamworks` crate) while identifying as AppID 113200.
  Works technically when Steam runs and the user owns the game.
- Either way, map our events to the original's achievement IDs (its `locker` list in the code).

## Control schemes: original and Rebirth
Let the player choose between the original's controls and Rebirth's. Document both first (the
original: keyboard and mouse only; Rebirth: keyboard plus controller, its own bomb/item/card
buttons), then make bindings data-driven so a scheme is a table, not code.

## Controller deadzones from Rebirth
Sticks use Microsoft's documented XInput deadzones (left 7849, right 8689 of 32767) for now.
Rebirth (Repentance+ `isaac-ng.exe`) reads controllers itself via XInput/DirectInput with
SDL-style mappings and exposes no deadzone setting; its value is not known.

## Resolution-independent art
Vector art is rasterized once at a fixed scale (room 2×, sprites 4×, effects 3× the original's
pixels), so very large windows upscale those images and lose some sharpness. Options: rasterize at
the actual window scale at startup and again after a resize, or tessellate the vectors and draw
them on the GPU (`lyon`) for true resolution independence.

## Fixed simulation tick
The original advanced its logic once per Flash frame at 30 fps (`onEnterFrame`, counting frames), so it
slowed down when the machine could not keep up. The remake integrates in real time with exact
continuous equivalents, which is frame-rate independent. A fixed 30 Hz tick with interpolated
rendering would let more logic be ported verbatim (counters, per-frame random rolls); decide
before porting enemies.

## Gameplay values read from the SWF
`GameParameters` (src/game/parameters.rs) holds every gameplay value and formula coefficient, and
`ParameterProvider` supplies a full set; today only `TranscribedOriginal` exists. A SWF-backed
provider could parse the AVM1 bytecode of `DefineSprite_6765` frame 1 (Ruffle's `swf` crate has an
AVM1 reader) and pick the constants out of known statements (e.g. a fire-rate value set to 10 in
`newstats`). Brittle against other game versions, so it should fall back to the transcribed values
and report which ones it could not find.

## Grid object follow-ups
- Explosions should also hurt the player and enemies (within `85 + size` px) and shake the
  screen; today they damage breakables and clear rocks and webs.
- Blast details not modeled: the 150 random sample points (our blast takes every tile in reach,
  each one it sees with `linecheckxx()`), the radius growing to 90 near blocks, `breakfloor` within
  40 px, rock gibs.
  The pit bridge uses the rock's direction from the blast, not the hitting sample point's.
- The tinted rock (`rarer`) should give a chest when blown up; there are no pickups yet.
- Bombs are physics objects in the original (Isaac and tears push them); ours stay put.
- Blown rocks always show rubble frame 60 (the original picks 60–64).
- The corny poop's fly should turn hostile when its poop breaks; today it just disappears.
- Spikes hurt and fireplaces burn on contact; needs player health.
- Keys are infinite (`KeyParameters::infinite_keys`); real key counts need pickups and a HUD.
- Rocks break only from bombs (`bloww`), with `Rock_crumble` sounds.
- Red fireplace: the wrapper `fireblock2` (5846, 84 frames) adds rising smoke puffs we don't draw.
- Destroyed fireplaces jump to frame 6 (charred logs) after the smoke; we hold the smoke's last frame.
- Rocks, blocks and webs pick a random variant frame; we always show the first.

## Texture memory and load time
Animation frames are now rendered lazily on first use (see docs/texture-streaming.md). Remaining
options: pack frames into atlases, evict frames not drawn for a while, share identical frames
between kinds. Measure on low-end hardware (see the in-RAM item).

## Empty clip frames
Some clips end in empty frames that the original still plays: the explosion (sprite 2040, frames
27–46) and the key block opening (frames 24–26). We keep them for faithful timing. They could be
skipped when rendering (an empty texture needs no upload or draw), keeping the timing.

## Optimizations from the original worth adopting
The original rendered vectors on the CPU every frame, so most of its tricks don't apply to a GPU
renderer. Worth adopting, roughly in order:
- **Persistent floor layer.** The original draws stains, splats, scorch marks, rubble and bridges
  into one floor bitmap (`splat`, `gut`, `dblock`) instead of keeping objects. Do the
  same with one render-target texture per room: the per-frame cost stays flat however many
  decals pile up. Today every scorch mark, blood splat and rubble tile is its own sprite (blood is
  dropped when the room is left, as the original's bitmap is). The SDL3 frontend already draws the
  splats and craters into one off-screen picture each frame (`DrawLayer`, shown at 70 %); keeping
  that picture between frames would be this. The Bevy frontend has no layers yet: it applies a
  layer's opacity to each item, so overlapping splats darken there.
- **Static tiles baked once.** `dblock` holds every rock, block, pit and web; only changed tiles
  are redrawn (`fillRect` + `draw` in `bloww`). We could bake static tiles into the floor layer
  too and redraw one tile when a blast changes it, leaving only animated objects as sprites.
- **Texture resolution as the quality setting.** The original's `_quality` (HIGH/MED/LOW/AUTO,
  AUTO dropping quality below ~14 fps) traded CPU time. Our equivalent knob is the pixels per
  world unit textures are rendered at, which saves memory on weak devices (Steam Deck, old iGPUs).
- **Don't render empty frames.** See "Empty clip frames".
Not worth adopting: hiding instead of deleting objects (a Flash object-cost workaround), and
frame-locked logic (replaced by time-based updates).

## SDL3 frontend follow-ups
- Gamepad start-up costs ~250 ms on machines with many input devices (SDL opens each one); look
  for a cheaper SDL setting or accept the post-first-frame stall.
- Decide whether to drop the Bevy frontend once SDL3 is verified (the comparison is in
  docs/frontend-comparison.md); keeping both costs a second clippy run per change.
- Measure input latency for both frontends.
- Ship SDL3: rely on the system/Steam runtime library or bundle it (the `sdl3` crate can also
  build it from source).

## Shipping without bundling libraries
- **Linux (Steam):** run under the Steam Linux Runtime instead of bundling SDL3. Pick **4.0**
  (`steamrt4`, Debian 13): Valve recommends it for new native games, it ships SDL3 (plus
  SDL3_image/SDL3_mixer) and is 64-bit only, which suits us. 3.0 (`sniper`) also ships SDL3 but
  is the older base. Build against the runtime's SDK (or at least an older glibc) and select the
  runtime in the Steamworks partner settings. Outside Steam (Arch, a distro package, Flatpak)
  the system or Flatpak runtime provides SDL3.
- **Windows and macOS** have no equivalent runtime: ship `SDL3.dll` next to the executable, or
  `SDL3.framework` inside the `.app` (zlib licence, ~2–3 MB), or link SDL3 statically (the `sdl3`
  crate's `build-from-source-static`).

## Startup follow-ups
- Re-run docs/frontend-comparison.md runtime numbers with the screen on: the last runs happened
  while the GNOME screensaver had blanked the display, which throttles every window.
- The splash waits the original's 3 s minimum (Zinc setting); loading itself takes ~0.4 s.
  Decide whether to keep the original timing or close it as soon as loading is done.
- Wayland does not let apps position windows, so the compositor decides where the splash appears
  (GNOME usually centers it); check it is centered on X11, Windows and macOS.
- The Bevy frontend has no splash; it loads in ~0.4 s before opening its window.

## Rooms and floors follow-ups
- Minimap and HUD: the original draws them in the strip above the room (`mapd()`, `mpiece`
  6580); our view only covers the room, so the frontends need the 800×600 stage layout.
- Special rooms: the boss room is empty, the shop has no items or shopkeeper (items and pickups
  do not exist yet; shop layouts mark them with `shop` markers); the treasure room has its layout and
  gold doors but no item pedestal (items do not exist yet); secret rooms (`hide`), sacrifice, library, arcade and
  challenge rooms are not generated yet.
- Door variants: the shop and library doors in rooms with enemies (frame 9 stays as it is),
  challenge and arcade doors (clip 6550).
- Enemies from layouts: only the fly markers (6601, 6589) are read; the others (6597, …) are
  skipped. Pits and spikes named `a<n>` report `-n / 100` variants, not read yet.
- Later chapters: other room sprites (`levs2`…, alt chapters `levs10`…) and `llev` ranges.
- Bombs left in a room are dropped on leaving; the original keeps per-room state (`levsav`).
- The Bevy frontend renders textures for the requested window size only; it does not re-render
  them when the window is resized or goes fullscreen (the SDL3 frontend does).
- Per-room floor and wall variety: the original picks `bg.d`/`w*.d` frames from a hash of the
  room number (`rax00 + lev + …`); every room shows the same art here.
- Secret rooms open to bombs and use `hide0`–`hide4` layouts, but have no pickups (none exist);
  the super secret room (`hide2`) is empty.
- Treasure doors from chapter 2 on start locked (frame 30) and open with a key (frame 31, then 32).
- Supersampled rendering briefly needs up to ~1.1 GB when re-rendering everything for a
  2560×1440 fullscreen (14 workers × supersampling buffers); render the two room-sized textures
  one at a time or tile them to cut the peak.
- Door animations: on entering a room the original plays the closing animation (frame 4,
  24 frames), then 10 frames later opens the doors of a cleared room, with
  `Door_Heavy_Open.mp` after the room's first 20 frames; ours are static open doors. Part of the "feel".
- Blast push and damage follow the original's line of sight (`linecheckxx()`: blocks, closed key
  blocks, walls and doors stop a blast, rocks and pits do not) for Isaac, enemies, pickups,
  breakables and hidden walls; machines break whether seen or not, as in the original. Enemies are
  not pushed yet.
- The intro timing test (`intro_frames_render_and_music_decodes`, `--ignored`) fails at ~55 ms
  per frame against its budget, on `main` too; check whether the CPU was throttled or the budget
  needs revisiting.
- The other super secret room styles (`[10, 26, 19, 23, 21]`). Boss rooms have their art, doors
  and layouts but no boss (no enemies yet), and their doors stay open.

## Pickups and health
- Temporary: one of every pickup kind, plus two eternal hearts, waits in each floor's boss room
  (`add_one_of_each`, `add_eternal_hearts` in `Floor::generate`); remove once checked.
- Machines and beggars (pickup frame 8, clip 2246; `col`: default slot machine, 41 fortune
  machine, 1 blood donation, 2 beggar, 31 devil beggar, 10 shell game) and the arcade room
  (`gamb`, `gam0`–`gam5`, coin door). Research notes in PROGRESS.md (2026-10-12, machines).
  Their animations need about 2000 frames: decide on lazy or streamed textures first.
- Room-clear rewards are a stand-in until enemies exist: rooms whose layout has enemy markers
  give theirs 10 frames after the first entry (the original rolls when the doors open after the
  fight).
- Not supported yet, so nothing appears where the original gives one: pills and cards (7, 30,
  35), trinkets, items (8, `5.1`), shop items for sale (15–19), the special chest variant (`c2`,
  1 in 20 brown chests) and other markers.
- Pickups and bombs only bump into the player; the original's `ballhit` also has them bump into
  each other, exchange velocity, and be pushed by blasts.
- Mega troll bombs (col 5) explode like normal troll bombs; the original's may be bigger.
- The bomb landing sound in clip 2038 (`Fetus_Land_1.wav` at frame 26) and `Chest_Drop.mp` are not
  played.
- Getting hurt: no hurt animation (`player.d` frame 2) or red flash; death starts a new run at
  once, without the death animation (`player.d` frame 6) or the death screen.
- The life bar shows up to 12 hearts (`lif` frame 1); more switch it to smaller layouts (frames
  2 and 3).
- The golden key indicator (`hud.kep`) is not shown.
- Ticks are fixed (30 per second by default, the original's rate) and drawn as they are: no
  interpolation between ticks, so on faster displays motion steps at 30 per second. Interpolating
  positions for drawing would smooth it without changing the rules.
- Minibosses are rolled (keeping the original's floor odds) but their dead ends stay normal rooms.
- Item pedestals stand empty (no items yet): treasure rooms, the library, golden chests. Shop items
  for sale and boss rewards are not placed yet.
- Load per floor instead of everything up front; draw what the next floor needs during the floor
  change and free the rest. Candidates:
  - room backgrounds (~5.5 MB each at 1080p): only the floor's super secret style, and curse,
    sacrifice and library art when the floor has those rooms;
  - later chapters' room art, enemies, bosses and boss "versus" screens;
  - title, character select, game over and ending cutscenes, only on their screens;
  - the chosen character's sprites only;
  - item pictures (203 in the pedestal clip) only for items actually placed;
  - the eternal hearts' full-screen animation, drawn when the first eternal heart is taken.
- A Rebirth-style HUD for `--hud rebirth` (the original's is shown meanwhile).
- Teleporting (once it exists) into the shop, library, or a locked treasure room (from chapter 2)
  must unlock its door like walking in does (`shopa`, `liba`, `bona` are set by being in the
  room), so it stays open on the way out.
- Curse room `c2` holds pickup marker `a10` (an unsupported kind); curse rooms are otherwise empty
  of their special contents.
- The eternal hearts' animation is drawn where `over` sits; check its position against the
  original (it looks about 50 px right of center).
- The HUD key icon leaves out `kep`'s 94 % vertical scale.
- A bomb placed under the player ignores them until they step off: this follows what the user saw
  in the original; the rule was not found in the original's script.
- Temporary: the gamepad's Back button toggles fullscreen.

## HUD follow-ups
- The HUD is one static texture showing a run's start (0 coins, 1 bomb, 0 keys, 3 hearts, no
  item): wire counters, hearts, the active item and its charge, pill and card names to the game.
- Minimap: rooms are never marked with the color of the pickups left in them (`levcol` →
  `mpiece.itmc`); only the shop, treasure, boss and secret icons exist so far (no sacrifice,
  library, challenge… rooms yet), and items that reveal the map (items 21 and 54) are missing.
- The floor-name banner (`st1`) slides in at the start of each floor; hidden for now.
- Text: only single-line fields and the HTML subset the HUD uses; no kerning or letter spacing.

## Gambling follow-ups
- Machines give nothing where the original gives a pill, card, trinket,
  "Pretty Fly", attack fly (now possible) or the devil beggar's spiders (`boil()`); items get empty pedestals.
- `lastspin`: after paying, the original keeps the doors from working for a moment.
- Broken machines can be pushed in the original (`busted` lifts `xp = xpp`); ours stay put.
- The arcade (`gamb`, layouts `gam0`–`gam5`, coin door) from chapter 2; the shell game only
  appears there.
- Isaac's thumbs up/down after a pill is in; other `emo()` callers (cards, items) still do not play it.
- Temporary: one machine of each kind waits in the shop.
- Shop: pill and item wares leave their places empty; buying does not show Isaac holding the
  ware up (`player.d` frame 4); the devil room's heart prices (`satan`) are not done.
- Take gameplay values and formulas from the player's SWF (any version): plan in
  `docs/parameter-extraction.md`.
- Stats panel: values are the starting ones until items and pills change them; luck is always 0;
  "range" shows the tear's start height. Glyphs use a fixed advance.
- Textures rendered ahead are never evicted; machine break animations are large (up to 272×244
  world units, ~40 frames each).
- `scripts/original-coverage.py` is a heuristic: common small numbers can mark a line covered
  that was not ported; quote original expressions in comments to make it exact.
- VRAM (in order of payoff): crop each rendered frame to its art and store the offset; reuse
  identical frames (slot results share sprites); pack small frames into atlases; evict textures
  not drawn for a while; prefetch only animations that can start soon. Long term: draw the
  vector shapes on the GPU as meshes (Ruffle's approach) instead of rasterizing every frame.
- Coverage by execution instead of comments: run original functions in the AVM1 interpreter
  (`docs/parameter-extraction.md`) side by side with our ports in differential tests, and record
  which original lines ran in passing tests.
- Vector meshes: draw them in the frontends (docs/vector-rendering.md steps 2–4).
- Bytecode reader: follow jumps (blocks, if/else), then anchors and a `SwfParameters` provider;
  structure-based anchors for obfuscated SWFs (the demo).
- Support every version from the browser demo to the latest release: find characters by linkage
  name or structure instead of hardcoded ids.
- Save data and unlocks (`so.data`, `locker[]`), e.g. `killshit()`'s 50-poop unlock; `Progress` (`progress.rs`) holds the blood donation count and its unlock in memory only, and the item it unlocks is not in the game.
- Blood creep is ported (`room/creep.rs`) but nothing leaves it yet: bosses, the spider boss, the dark rooms' slowing and the creep's long stay and colours (`longstay`, `colorit`) come with them. `playerhurt()` also raises every hit to a full heart past chapter 6.
- `physix()`: enemy and enemy-shot parts with enemies; `slow`/`sloww` with items; `turdb`
  (standing over a broken poop's tile) and ladders.
- Cards: The Magician (homing, range), The High Priestess (Mom's foot) wait for items; The Empress's demonic look is done (head mode 30, body outfit 8 or 9) except holding up item 122 and its look while riding the unicorn (the plain body shows) and its tear look (tear frame 30);
  The Devil's look (red tears `fmode 8`, face `hmode 8`); the 15 % more damage per tear that The Devil
  and Strength give (DoAction 3732–3735); Death's use counter, which unlocks something after five uses
  (DoAction 27967–27971); card names in the demo (follow jumps). `docs`-less status of every card:
  22 complete, 3 partial (The Devil, Strength, The Empress), 2 not done (The Magician, The High
  Priestess).
- Pills are in (`player::pill_effects`); the shop sells them too (a pill, or a tarot card); still missing: the random look swaps (`picc`), Isaac's colour flash, blue flies (`Friends`), the poison cloud (`Bad Gas`), the bombs of `Explosive Diarrhea`, puberty's hat and face, the fart sound (not an MP3 in the bank), Strength counting in the tear's first rise like range pills do.
- `--showcase` puts every card in the treasure room (and every pickup kind in the boss room,
  every machine in the shop); off in normal play.
- Devil room: open only after the boss and only sometimes (as the original); deals for heart
  containers once items exist; `krampus`/`sata` variants.
- Rooms not ported: arcade, challenge room, I AM ERROR room (its shopkeeper is clip 6726), and
  the later-chapter rooms.
- Red chests: spiders (`boil`) and three blue flies give nothing yet; the
  item pedestal is empty and does not push the player away (`enfcheck(..., 70)`).

## Raw numbers that should be typed (audit 2026-10-24)
Done (same day): every duration, counter and timer of frames is `Frames` (`*_every`/`*_phase` cycles via `Frames::is_on_cycle`, the room's frame number as `Frames`, timers, cooldowns, delays, flashes); retentions are `Retention`; heights, distances, reaches and radii are `Pixels` (also `TILE_SIZE`, `Bounds::shrunk_by`, `push_circle_out`, `spawn_spot`); speeds are `PixelsPerFrame`, gravity and gains `PixelsPerFrameSquared`; the pill code's hearts are `Hearts`; bounds that were `(f32, f32)` are `Vec2`.
Also done: `HitPoints`, `Damage` and `Mass` (`original_units/combat.rs`) and `Odds` (`odds.rs`, rolled with `Random::happens`). Left: enemy numbers (`phase`) are ordinals and stay `u32`; clip frame counts (`*_FRAME_COUNT: u8`) index the original's clips, not durations; HUD layout offsets and texture sizes are `Vec2` in the stage's own pixels.
- Still raw: the grid's value (`levz`) is a bare `f32` in about 20 files (layout cells, `levzz()`, `GridObject::level_value`, `room/sight.rs`, spawn spots), with its levels repeated as constants (the pit's 3 in three files; walls 1.9, doors 2, hidden walls 1.85, spikes and webs 0.99, rocks 1). Make it a `GridValue` with named levels; the exact comparisons the tests now allow (`float_cmp`) would then compare the type.

## Rebirth-style local multiplayer
Up to four players on one screen, as *The Binding of Isaac: Rebirth* does it (the Flash original has
one player only, so none of this is in the script to port; it is our own design):
- Players 2–4 join by pressing a button on another controller or the keyboard's second half; each is a
  character (Isaac to start with) with their own hearts, bombs, keys, coins, active item, card or pill
  and passive items; the pickups, rooms and floor are shared.
- A shared camera: the room is one screen already, so no scrolling; all players must be through a door
  for the room to change (or the room changes when the first one leaves, and the others are carried
  along; decide by playing).
- A dead player turns into a ghost that cannot be hurt and can still shoot; they come back in the next
  room with half a heart or on a revive item; the run ends when everyone is dead.
- Enemies pick the nearest player as their target (the attack fly, the Gapers' pathfinding, the Mulligan's
  fear, the Pooters' shots); blue flies and other familiars follow the player who owns them.
- Code that has to change first: `Game` holds one `Player` and one `Inventory` (and `PlayerLook`, the
  death note, the HUD's hearts and held card, the stats panel); they become a list of players with
  a per-player input (`InputState` per controller, `controls.rs` bindings per device); `Enemies::advance`
  and the walkers take `player: Vec2`, the shots hurt `PLAYER_SIZE` at one position.
- HUD: one row of hearts, counters and item boxes per player at the screen's corners; the minimap and
  floor stay shared.

## Trinkets (`pillc()`'s trinket branch)
- Done: the 34 trinkets (`Trinket`, ids 29–62) as a pickup kind, with their art; the pool every run
  starts with (18 of them, each handed out once); the room-clear roll, chests, the fortune machine and
  the devil beggar giving them; taking one (the sound, Isaac holding it up, the name banner read from the
  game, the held one swapping out and lying on, the already-held and Tick refusals); every trinket in
  the treasure room for checking (`--showcase`); both HUD slots (picture, name strip and ring, with
  the taking animation); Mom's Purse (item 139, on a pedestal in the start room) and the second
  slot it opens; the Tick's hat; and the effects of the Goat Hoof, Curved Horn, Game Squid, Cancer,
  Fish Head, Red Patch, A Missing Page, the Polaroid (its bubble), the Cursed Skull, Samson's Lock,
  the Tick (a heart in the boss room), Isaac's Fork, the Lucky Toe and Mom's Pearl.
- Left, blocked by what the game lacks: the Umbilical Cord (at half a heart it grants the familiar
  item 100) and Eve's Bird Foot (a kill sometimes calls the Dead Bird familiar): no familiars yet;
  the Tick's 15 % off the health of enemies above 60 hit points (none that strong yet); the Polaroid's
  part in the heaven door and in an unlock (no later chapters); Samson's Lock's red tint and its size
  change; the Fork's rising heart (`hearter`).
- Left, not looked at yet: the tear effects (Pinky Eye, Push Pin, Game Squid's dark tear), the pennies
  (extra pickups from coins), the pickup swaps (Ace of Spades, Safety Cap, Match Stick, Rusted Key,
  Child's Heart), the Left Hand and Judas' Tongue, Blue Baby's Soul and Isaac's Head (ghost
  familiars), Maggy's Faith and Cain's Eye, and Liberty Cap. Which trinkets join the pool later
  (`junx()`) and a gold pickup being always the Counterfeit Penny. Look each one up with the script's
  check for it (`trixx(ID)`) before porting.

## Items: `itz()` and what feeds it
- Done: `item_roll` (`itz()` and `itzz()`): the test each candidate of an item pool goes through
  (already held, found in an earlier run, no fourth familiar, the power level's odds for a major
  item), the power level (`wiptz`) kept in the inventory and raised by major items taken, and the
  save's memory of items found (`Progress`), which taking a pill or a card also feeds (items 43 and
  61).
- Left: the pools themselves (`ittt`, ... each room kind's list of candidates) and what draws from
  them: treasure room, shop, boss, devil, angel and secret room pedestals; the devil room's pedestal
  looks that `itz()` picks (DoAction 1459–1470); the characters' starting power level (DoAction
  877–885) and the other power-level changes (DoAction 5143, `giveit()`); hard mode. The power
  level is not saved yet.

## Original Steam Controller outside Steam Input
- With Steam running, the 2015 Steam Controller (wireless dongle 28de:1142) reaches a program started
  outside Steam only as a keyboard and a mouse ("lizard mode"); SDL lists no gamepad for it, even
  with `SDL_JOYSTICK_HIDAPI_STEAM=1`. Added to Steam as a non-Steam game it should show up as Steam
  Input's virtual Xbox 360 pad, which the game handles; untested. Whether SDL's own driver would
  take it with Steam closed is untested too (needs `/dev/hidraw` access, which the `60-steam-input`
  udev rule grants to the logged-in user).

## Before publishing
- Drop every mention of the old name `tboi-remake`; the project is OpenTBOI only. Where it is now:
  `FORMER_CACHE_DIRECTORY_NAME` in `src/game_assets/mod.rs` (moves an old cache directory to the new
  name; nobody else has one), the `rm -f` of the old launchers in `scripts/install-desktop-entries.sh`,
  and the `app_id` line in `VERIFIED.md`. The project folder's own name is local and doesn't matter.
- Test the GitHub plumbing in the private staging repository first (done once: CI, a dry-run
  release, branch protection), then publish the work in a separate, fresh repository.

## Steam Controller: how other games do it
- Games rarely do anything special: they accept an XInput/SDL gamepad and count on Steam Input to turn
  the Steam Controller into one, which only happens when the game is started from Steam. Games that
  want more use the Steam Input API (action sets, glyphs, gyro) and also need the Steam client. The
  only route that works without Steam is reading the device directly, which is what SDL's HIDAPI
  drivers are for. The Last of Us Part I's controller support was named as a model: look at it
  (the layout, whether it uses the Steam Input API, how it behaves launched outside Steam) once it
  is installed; not looked at yet.

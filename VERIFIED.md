# Confirmed Working

What has actually been checked, by whom, and on what. "User" means tried by hand; "automated"
means a test or a scripted run by the assistant. Anything not listed here is unverified.

Test machine: Fedora 44, GNOME on Wayland (XWayland available), a current AMD GPU (RADV/Vulkan),
native Steam in `~/.local/share/Steam`.

## Gameplay MVP
- [x] Keyboard: WASD moves, arrow keys shoot — user, 2026-10-10 (before the engine-boundary refactor)
- [x] Gamepad controls — user, 2026-10-11 (after the refactor)
- [x] Keyboard re-checked after the engine-boundary refactor — user, 2026-10-11
- [x] Player stays inside the room; holding fire respects the fire delay — automated (unit tests)
- [x] Input: diagonal keyboard movement normalized; shooting snaps to four directions — automated
- [x] F11 fullscreen toggle, Esc to quit — user, 2026-10-11
- [x] Isaac's original art: body (standing, vertical and mirrored horizontal walk cycles) and head (4 directions + shooting face) assembled from clips 4308/4307/1620 and 2261; head timing from the face timer `plox` — automated (unit tests, render test)
- [x] Isaac's look and animation checked in game — user, 2026-10-11 ("looks, walks and shoots great")
- [x] Room grid objects: rock, block, pit, spikes, spider web, key block, poop, red/corny/golden poop, fireplace, red fireplace, TNT barrel — rendered (stages, idle loops, destroyed animations, no canvas clipping) and collision/damage rules unit-tested — automated
- [ ] Room grid objects checked in game (one of each placed; shoot the breakables, walk into the solid ones, tears over the pit, tears slowed by the web)
- [x] Movement top speed (≈248 px/s), fire delay (11 frames), tear speed (10 px/frame + 0.6 × player velocity) and tear range (lands after 25–27 frames) follow the original's code — automated (unit tests)
- [x] Start button quits like Esc — user, 2026-10-11
- [x] Mouse shooting: holding the left button shoots toward the cursor (snapped to four directions) — user, 2026-10-11
- [x] Tears drawn with the original tear art and floor shadow — user, 2026-10-11 ("look great")
- [x] Poop shown on its tile — user, 2026-10-11 ("looks fine")
- [x] Poop blocks Isaac and tears, breaks after 2–4 tears through its damage frames, plays plop.wav and leaves a stain — automated (unit tests, render test)
- [x] Poop behavior checked in game — user, 2026-10-11 ("works great")
- [x] Tear splashes: wall/obstacle splash (b2 frames 2–17) and ground splash (frames 40–55) rendered with fades, randomly rotated — automated (render test)
- [x] Splashes checked in game — user, 2026-10-11 ("work great")
- [x] Stick deadzones (XInput defaults) unit-tested — automated
- [x] Stick deadzones feel right on a real controller — user, 2026-10-11
- [x] `--window-size small|medium|large|WxH`, `--fullscreen`, `--help`; every original resolution accepted — automated (unit tests)

## Platform and windowing
- [x] Native Wayland window (`xdg_toplevel`, app_id `tboi-remake`) — automated (`WAYLAND_DEBUG`)
- [x] X11 via XWayland when `WAYLAND_DISPLAY` is unset — automated
- [x] Binary links no Wayland/X11 libraries (loaded at runtime); only libasound and libudev — automated (`ldd`)
- [x] GNOME launcher shows in the app grid and starts the game — user, 2026-10-11
- [x] Source-only build: a copy of the tree without `cargo-about`, `THIRD-PARTY-LICENSES` or git history builds and passes `cargo test --locked` (163 passed, 12 ignored; the build only warns) — automated, 2026-10-15
- [x] Linux tarball (`scripts/package-linux-tarball.sh`): unpacks, `install.sh` installs binary, launcher (`desktop-file-validate` clean), icon and licences into a scratch prefix, the installed binary starts (`--help`), `--uninstall` removes every file it wrote, `DESTDIR` staging works — automated, 2026-10-15
- [x] Baseline (`x86-64`) release binary: AVX2 instructions only in `memchr`, `simdutf8` and `simd-adler32` (run-time detected) — automated (`objdump`), 2026-10-15
- [x] `--features bundled-sdl3` (static SDL3) builds in a clean Fedora 44 container (podman) with `cmake make gcc` plus the window-system and audio `-devel` packages (without them SDL's configure fails, it does not silently drop Wayland/X11); the 7.6 MB binary links only libc, libm and libgcc_s (X11, Wayland, ALSA, PulseAudio, PipeWire, udev are `dlopen`ed) and runs on the host under Wayland and XWayland (first frame and 143 fps) — automated/manual, 2026-10-15. The Ubuntu package list in the workflows was missing `libxtst-dev` (found through this build; fixed, Ubuntu itself not run)
- [x] OpenGL renderer holds the refresh as evenly as Vulkan on the test PC (Wayland and XWayland; 2026-10-15, `docs/frontend-comparison.md`)
- [x] Start room bomb hint (2026-10-15): the keyboard shows `E`, a pad shows `LB` — automated (unit tests of the device rule and of the choice; a screenshot of the keyboard case). Not tried with a physical pad.
- [x] Release tarball built in an Ubuntu 22.04 container with static SDL3 (2026-10-15, `scripts/build-tarball-in-container.sh`): 2.3 MB, needs glibc 2.35 at most, `--help`, a 3 s showcase run and `opentboi-diagnose` work on the Fedora 44 PC — automated. Not run on a Deck or Bazzite (`docs/steam-deck-testing.md`).
- [x] Release tarball binary on SteamOS 3.6.24's userland (2026-10-16, the Deck's eMMC mounted in the PC, `scripts/steamos-disk-test.sh run`, bubblewrap): starts, extracts the SWF, loads layouts, plays sound through the PipeWire/Pulse socket, exits cleanly — automated. Software rendering only (that Mesa has no driver for the development PC's GPU); not SteamOS's kernel, Steam, gamescope or Game Mode, and not a Deck
- [x] `shuck check scripts packaging .github` clean with every shuck rule on except seven house-style policies (`.shuck.toml`, 2026-10-16) — automated; ShellCheck itself not run
- [x] Start-room hint flipping to `LB` with a physical pad in the rebuilt launcher binary (2026-10-16) — user
- [x] Under nested gamescope (2026-10-15, GNOME/Wayland, current AMD GPU, `--showcase --exit-after 6`): starts, draws 806 frames and exits cleanly — automated. Frame delivery looked uneven (median 0.79 ms, mean 6.34 ms) and gamescope logged "Compositor released us but we were not acquired"; not understood, not a Deck or Game Mode test
- [x] Vector drawing (2026-10-15): 314 textures at start-up instead of 2096, 134 MiB GPU memory instead of 383, same vsync frame times (6.96 ms mean, 7.7 ms 99th percentile, re-run) — automated measurement; that it looks the same on screen is not yet confirmed by the user. Drawing the stage surround as meshes was measured earlier as worse (156 MiB, 99th percentile 13 ms; not re-run) and not kept
- [ ] The GitHub Actions workflows (never run); an old CPU (Q6600, i5-2500K) or old GPU (HD 5770, GTX 660); OpenGL pacing on NVIDIA, Intel or the Deck
- [x] SteamOS 3.8.28 (kernel 6.18, Mesa 26.1.2) on the development PC with an AMD GPU, Desktop Mode (KDE Wayland), tarball binary: Vulkan renderer, 144 fps, 6.94 ms mean, 99th percentile 7.7-7.9 ms, runs clean for 15-20 s, finds the game both through `TBOI_GAME_DIR` and in the Steam library (`~/.local/share/Steam/steamapps/common`, report 2026-10-17 18:35 and 18:37, `TBOI_GAME_DIR` unset) (2026-10-17, user; reports read)
- [x] Controller glyphs: all five hint pictures (`E`, `LB`, `L1`, `L`, Deck `L1`) drawn in the start room (`--hints`); SDL's virtual pads map to the right make (`cargo test virtual_pads -- --ignored`) (2026-10-17, automated)
- [x] SteamOS in Desktop Mode only, with the same Flydigi controller (user, 2026-10-17); Game Mode was not tried
- [x] Controller glyphs are drawn as vector meshes in the SDL3 frontend (start-room hints, HUD item label), rasterised only for the Bevy frontend (screenshots, automated)
- [x] The item label as `LT`/`L2`/`ZL` in a black outline on no background (2026-10-17) — user, 2026-10-18 ("seems to work fine")
- [ ] The start room's pad hints: Move as the D-pad's four directions (solid, the pressed arm hollow, like Rebirth), Attack as the four face buttons (Rebirth's order, by position), Bomb as the left bumper, Item as the left trigger (2026-10-18, screenshot-checked for Xbox, PlayStation and Switch) — user
- [x] Items: the Lucky Foot on a pedestal in the start room (screenshot), taken with the choir, the empty pedestal left behind (test), the name banner and Isaac holding it up (textures rendered); `slotf()` complete: the Lucky Foot's roll and the Portable Slot's flag (tests), and the Lucky Foot at the devil beggar, the fortune teller, the shell game and room rewards (2026-10-18, automated)
- [x] Taking the Lucky Foot in play — user, 2026-10-18
- [x] Held card's button badge: the keyboard's Q as the original draws it, a pad's right bumper per make in its place (tests; the strip without the baked Q) (2026-10-18, automated)
- [ ] Gapers in rooms (the marker was missing; now read), the card badge over the ring, no badge during the use animation, and the secret room's neighbours on the minimap — user
- [ ] Gapers pushing each other apart; the Devil card's dark tears — user
- [ ] The Gaper's growl on sighting you — user
- [ ] The card's `RB` hint in play with a pad, and a fortune shown on the banner when the fortune teller is paid — user
- [ ] The Lucky Foot's effect at the start room's fortune teller (a dime beside it to play with) — user
- [x] My Little Unicorn: taken into the active slot from its start-room pedestal, used with the use
  button (charge to nothing, the ride with its tune, no tears, faster, over after 180 frames), the
  Chariot holding up the unicorn (tests); the ride's colours and blink, the HUD box's frames
  (unit tests); unicorn head, tinted parts and the item box render and fit their canvases
  (2026-10-18, automated)
- [x] Trampling poop on the unicorn — user, 2026-10-18
- [x] The active item's label drawn over its box (it was under it); the box's dark fill matches the
  original HUD's (same colour as FFDec's render); chests, cards and pedestals lose the extra 20 % of
  their speed each frame the original gives every pickup from the chest frame on, so a pushed empty
  pedestal crawls (test) (2026-10-18, automated)
- [ ] My Little Unicorn in play: HUD box flashes, the ride's look, recharge beeps after cleared
  rooms; empty pedestals now harder to push — user
- [x] No quote of the original's code in comments or docs (`tests/no_original_code_in_comments.rs`), and none in the git history after the rewrite (every blob scanned; code identical in every commit) (2026-10-18, automated)
- [x] Clippy `pedantic` with no warnings in all three builds; health in `Hearts`, floor places as `FloorCell`, random directions and list picks through `Random`, with every test passing (2026-10-18, automated)
- [x] `spawnb()`'s alpha rules agree with the original's order for every marker value and opacity (`alpha_rules_agree_with_the_originals_order_for_every_opacity`, 2026-10-18, automated)
- [x] Primitive matching of the original's arithmetic against our code runs (`cargo test original_arithmetic_in_our_code -- --ignored --nocapture`; unit tests for its normalisation) (2026-10-17, automated)
- [x] Primitive matching judges each original function against its counterparts: functions read by hand
  come out as they are (`ender()`, `crand()` complete; `spawnb()` 10 of 11, the missing one the grid's
  lazy creation; `slotf()` 7 of 8, the missing one the Lucky Foot's roll); coincidental best fits are
  gone; the decoder keeps register locals as names (unit tests for each rule) (2026-10-17, automated)
- [ ] Controller glyphs on real PlayStation, Switch and Steam Deck controls (not available; the Deck is to be tried later); Game Mode (gamescope) and Steam Input
- [ ] Real X11-only session; the Deck itself; Windows; macOS

## Game files
- [x] Steam discovery finds AppID 113200 on native Steam (reads appmanifest, not the stale `libraryfolders.vdf` app list) — automated
- [ ] Flatpak or Snap Steam installs
- [x] The SWF is read from the installed game's files and matches an independent reading; cached in `~/.cache/opentboi/isaac.swf` — automated
- [x] Reader unit tests (synthetic inputs) — automated

## Assets
- [x] Tear shot sound `Tears_Fire_0.mp3` (and `TearImpacts0.mp3`): raw MP3 from `DefineSound`, valid 44.1 kHz mono — automated (ffprobe)
- [x] Room art identified and rendered with FFDec: basement floor (sprite 6281, 4 variants), room walls (sprite 7796) — automated, previews in `~/.cache/opentboi/preview/`
- [ ] Previews checked by the user
- [x] Tear sounds (3 fire + 3 impact variants) load from the SWF via Ruffle's `swf` crate at startup — automated ("Loaded 6 sound clips")
- [x] All 6 clips decode with rodio (Bevy's decoder) as 44.1 kHz mono, 0.1–1 s — automated (`cargo test -- --ignored`)
- [x] Firing emits a TearFired event, a landing tear a TearImpact event — automated (unit test)
- [x] Tear sounds audible in game when shooting and when tears land — user, 2026-10-11
- [x] Room background (sprite 6468: walls, door frames, Basement floor) rendered by our own code (Ruffle `swf` parser + `tiny-skia`), no unsupported features, matches FFDec's render by eye — automated (`cargo test -- --ignored`, writes `target/room_background.png`)
- [x] Room background shown in game, aligned with the walkable area (player stops at the inner wall edges) — user, 2026-10-11
- [x] Tear (shape 1789), tear shadow (shape 341) and poop (shape 1460, uses a bitmap fill) rendered by our code; bitmap fills (lossless 8/15/32-bit) supported — automated (`cargo test -- --ignored`, writes `target/textures/`)

## Room objects (automated, awaiting the user)
- [x] Red poop grows back 4 s after breaking, with its grow-back animation — unit test
- [x] Spider webs slow the player (70 % of the speed kept for 4 frames after leaving) — unit test
- [x] A TNT barrel explodes: blast animation (26 frames), scorch mark, `boss1_explosions` sound; the blast breaks breakables within 60 px and sets off other barrels — unit test
- [x] A white fly circles the corny poop and leaves with it — unit test
- [x] Pushing into the key block for about 5 frames (0.17 s) opens it with a key (`Unlock00` + `metal_blockbreak`) — unit test
- [x] Red fireplace recolored like the original (bitmap fills take color transforms) — automated render
- [x] Regrow, key block opening, fly, explosion and scorch art render without clipping or unsupported features — automated (`cargo test -- --ignored`)

- [x] Key block collision, white fly, red fireplace, TNT explosion, web slowdown — user, 2026-10-11
- [x] Damaged fireplaces keep their (shrinking) fire animation — user, 2026-10-11
- [x] Bombs break the web and set off TNT; red poops grow back — user, 2026-10-11
- [x] The dark rock (tile frames 11–14, level 1.9) survives bombs, as in the original — the original's script
- [x] Red poop regrows silently in the original (no sound in the script or the clips) — the original's script and SWF tags
- [x] A rock blown towards a neighboring pit bridges it (tile frame 66); the tinted rock (frame 15)
  breaks like a rock — unit tests
- [x] Bombs: E (or LB) drops one per second; it explodes 58 frames later with the full
  46-frame blast; rocks crumble to rubble (`Rock_crumble`), webs tear, blocks survive — unit tests
- [x] Bomb fuse colors match the original's timeline (FFDec's export cycles the bomb variants
  because it ignores `stop()`) — automated render, compared by eye

## Frontends
- [x] SDL3 frontend runs the showcase room with sound decoding and frame reports; Vulkan, OpenGL
  and SDL_GPU renderers all work — automated (timed runs)
- [x] SDL3 and Bevy draw the same frame (mean difference 0.77/255) at native resolution — automated
  screenshot comparison
- [ ] SDL3 frontend played by the user: keyboard, mouse aim, gamepad, sounds, F11, Esc/Start
- [x] Window size is in real pixels regardless of desktop scale: `large` gives 1024×768 pixels at
  GNOME 125 % in both frontends — automated (frontends report the size)
- [x] The original splash (`isaacload.bmp`, 800×455) is found in the installed game's files
  and shown 111 ms after launch while loading; all 701
  textures render behind it in ~120 ms on 14 threads — automated (`cargo test -- --ignored`, logs)
- [x] Splash looks right and is centered — user, 2026-10-11
- [x] The original intro (`--intro`, sprite 7317 + its stream music) renders without unsupported
  features, under 33 ms per frame at 1200 px, music decodes to ~12 s — automated
- [x] Separate launchers for SDL3 and Bevy, valid desktop entries, both start — automated

## Floors and rooms
- [x] Basement room layouts read from sprite 6622 (186 frames, 2276 markers, all values known,
  all inside the 13×7 interior) — automated (`cargo test -- --ignored`)
- [x] Floor generation port: 200 seeds give connected floors of ≥ 7 rooms with the boss room not
  next to the start — unit test
- [x] Walking into a door enters the neighbor by the opposite door, no bouncing back — unit test
- [x] Doors drawn on all four walls, facing inward — automated composite of the rendered art
- [ ] Exploring a floor in game (doors, rooms keep their state) — user
- [x] Room slide ported (offset ×(0.5 + min(d·0.004, 0.22)) per frame from 640/440 px) settles in
  0.3–0.8 s; `--instant-room-changes` switches at once — unit test, smoke run
- [x] Start room shows the controls drawing (shape 6313 from floor frame 50) — automated render
- [x] Textures render at the window's pixel density (1.58 px/unit at 1024×768; 3.43 after going
  fullscreen on 2560×1440, re-rendered in 0.24 s) — automated (logs)
- [x] Start room drawing and room slide — user, 2026-10-11
- [x] Shapes are anti-aliased like Flash's HIGH quality (aliased 4×4 supersampling, averaged):
  no more seams around floor specks; 703 textures in ~0.35 s — automated (screenshot comparison)
- [ ] Room art looks as smooth as the original — user
- [x] Entering a room freezes movement for 20 frames and doors for 30 (`fra`) — unit test
- [x] Every first floor (200 seeds) has a treasure room, shop, secret and super secret room —
  unit test
- [x] Treasure room: one of six layouts `t0`–`t5`, gold doors (clip 6550 frame 32), jingle on
  the first visit — automated (asset tests), not yet seen in game
- [x] Treasure room in game, jingle plays — user, 2026-10-11
- [x] Walls are solid cells with open doorways; the door trigger sits 28 px out in the doorway
  (28 px out from the door art); walking straight at a door goes through — unit test
- [x] A bomb by a secret room's wall opens it (blown hole, frame 17) both ways — unit test
- [x] Door feel (better) and wall bombing in game — user, 2026-10-11
- [x] Secret, super secret and shop rooms use their own room art (wall and floor clips on frames
  20, 10 and 2; the renderer now honours chosen child frames and draws buttons) — automated render
- [x] Bombing open a secret wall plays `secret room find.mp3` (super secret: its reveal jingle),
  once per floor — unit test
- [x] `--secret-entrances original` closes the wall behind you on leaving the secret room;
  `rebirth` keeps it open — unit test
- [x] Shops: six layouts, door locked (keyhole) until walked into with a key (`Unlock00`) —
  unit test, automated render
- [x] Secret rooms, jingles and shops in game — user, 2026-10-11
- [x] "Original" secret entrances close behind you, confirmed in the original game by the user;
  the code agrees (the door setup resets the wall to 1.85 after the blown-cell replay)
- [x] Turning straight back after entering a room goes back through the door (doors re-arm at 35
  px every frame, like `nod`) — unit test
- [x] Static HUD (clip 1441) over a 640×480 stage like the original: counters, weapon and item
  boxes, hearts, floor name; text fields and static text drawn with the embedded fonts —
  automated render and screenshot
- [ ] HUD in game — user
- [x] Minimap (`mapd()`): current, visited and seen rooms, shop/treasure/boss icons once a door
  to them was seen, secret icons once found — unit test, screenshot
- [x] Minimap in game — user, 2026-10-12
- [x] Shop door unlocks only when walking into it (`keyhole`), never through a hidden secret wall,
  and plays its key animation (frame 8) — unit test
- [ ] Shop door unlocking in game — user
- [x] Blasts push Isaac away (by the reach left over, at most 15 px per frame, after halving his velocity) —
  unit test
- [x] Blast push in game — user, 2026-10-12
- [x] Wide or tall windows show the original's surround pattern (overlay sprite 6221) instead of
  black bars — screenshot at 1600×600
- [x] Fullscreen surround and the re-render pause with progress bar — user, 2026-10-12
- [x] Boss rooms: blood-red art (frame 3), boss doors (frame 16) both ways, layouts `b1-0`–`b1-4`,
  secret walls next to them unbombable — automated render
- [x] Boss rooms in game — user, 2026-10-12
- [x] Pickups from room markers (hearts, coins, keys, bombs with the original's variant odds),
  taken within 22 px after 15 frames, troll bombs explode; live HUD counters and life bar;
  blasts hurt (1 heart, 60 invulnerable frames, blinking); soul and eternal hearts — unit tests,
  automated render
- [x] HUD counters and blast damage with grunt — user, 2026-10-12
- [x] Shop door stays shut without a key — user, 2026-10-12
- [ ] Pickups in game (boss room has one of each) — user
- [x] Firing waits 20 frames after entering a room (`nomove`); fireplaces hurt and push; chests
  (brown, golden with key), drops from poop/fireplaces, tinted rock and golden poop contents,
  room-clear rewards; player and bombs bump apart; bomb pickups without a burning fuse; a
  bombed hole into the shop is open — unit tests, automated render
- [x] Fireplaces hurt, Back toggles fullscreen, chests open, poop drops, entering the shop from the
  secret room — user, 2026-10-12
- [x] Fixed tick rate (`--tick-rate`, default 60): game state in the original's units, original
  counters written as in the original (`fail++ > 60`, `keypoww++ > 2.8`); pickups pushed by blasts,
  friction 0.965, settle after 25 frames; landing sounds; five super secret room styles; entering
  the shop unlocks its door; no blast push when standing right on the bomb — unit tests
- [x] Two eternal hearts make a heart container, landing sounds, dying, fire hiss matches the
  original, entering the shop from the secret room, super secret room styles, Back toggles
  fullscreen — user, 2026-10-12
- [x] Default tick rate 30; collision trades momentum (`xb` impulses), bombs slide and ignore the
  player until stepped off, troll bombs passable, mega troll bombs chase; golden key HUD icon;
  eternal heart animation; curse rooms (art 19, spiked doors 36, half-heart doorways, `c`
  layouts), sacrifice rooms (art 24, spikes may give a chest once a floor); spikes hurt a heart;
  shared frontend controls and random sound variants; Bevy prerenders everything — unit tests,
  automated render
- [x] Golden key HUD, eternal heart animation, curse and sacrifice rooms appear, Bevy explosions
  smoother — user, 2026-10-12
- [x] Unit types (`Frames`, `Pixels`, ...); open curse doors (frame 37); curse/secret door rule
  (`--curse-secret-door`); empty item pedestals (immovable); library (art 25, key-locked door,
  icon 12, `lib` layout); pickups and bombs keep their radius from walls — unit tests, automated
  render
- [x] `Style` (original/Rebirth) for secret entrances, curse–secret doors and the HUD; bumped
  pickups stay out of rocks and walls (the player takes the push instead); pedestals movable;
  entering a secret room finds it (jingle), so bombing out stays quiet; fixed: bombing from a
  secret room into a shop or treasure room no longer played a jingle; hold R / Back to restart, tap
  Back for fullscreen; renamed to OpenTBOI (cache moved) — unit tests, timed run
- [x] Restart on held R — user, 2026-10-12; secret room exit stays quiet — user, 2026-10-12
- [x] Gamepad restart on right stick + right trigger (Rebirth, confirmed by the user), Back toggles
  fullscreen again; placed pedestals cling to their spot (30 % of a push per frame, snap back
  from 30 px or more, only onto free floor), chest pedestals move freely — unit tests
- [ ] Those in game — user
- [x] Restart takes holding R or RS+RT for 2 seconds while the screen fades to black, then needs a
  release; interpolation between ticks (on by default, `--no-interpolation`); gambling machines
  (slot, fortune, blood bank, beggar, devil beggar, shell game) with payouts, breaking by blasts
  and art rendered on demand — unit tests, real-asset tests, rendered contact sheet
- [x] Restart fade — user, 2026-10-12
- [x] Holding restart restarts again every 2 s (Rebirth, per the user); the devil beggar's look
  (parts `p`, `p0`–`p4` on frame 2); machines moved to the shop; fixed: bumping a machine pushed
  Isaac through walls, and every bump reset his interpolation (walking looked choppy) — unit tests
- [x] Pickups spawn inside the room; devil beggar look in every frame (parts by character);
  broken fortune machine keeps its colors (`sloter` on 2175, 2206); the shell game shows only
  the prize played; the floor name's box fits the name; pickup messages ("a Nickel", "a Dime",
  "1 + 1 free", "Trolololol"); shop wares (heart, bomb, key, soul heart, with one sale) —
  unit tests, real-asset tests, rendered textures
- [x] Walking with interpolation, shell game, fortune machine — user, 2026-10-12
- [x] Wares do not block; no lit fuse on shop and shell-game bombs; the pickup message is cut at
  the stage edge; machine textures rendered ahead (`Game::upcoming_textures`, 3 ms a frame);
  `--stats` panel; `scripts/original-coverage.py` — unit tests, real-asset tests
- [x] Smooth machine explosions — user, 2026-10-12
- [x] No velocity into walls (tears fired against a wall go straight) — unit test
- [x] Shots against walls go straight — user, 2026-10-12
- [x] Wall rebound for pickups and bombs, bombs bump each other, Isaac's wall friction and sliding
  along walls — unit tests
- [x] Pickups move better — user, 2026-10-13
- [x] No bounce back pushing a stick nearly straight into a rock; cards: pickups, slot, use,
  effects (see PROGRESS.md), art and sounds — unit tests, real-asset tests, rendered textures
- [x] RB uses cards; HUD card picture; card spawns away from Isaac (posw); Devil/Strength/Hanged
  Man boosts with live stats; teleport and card-use animations; card names read from the SWF —
  unit tests, real-asset tests, rendered textures
- [x] Card spawns placed like the original — user, 2026-10-13
- [x] No card swap loop, one card per press, The Moon's way out, teleport arrival, The Empress,
  HUD slot animations — unit tests, real-asset tests
- [x] Devil room door from the boss room and back, off the map; minimap marks; teleport arrival
  spot; Chariot and sacrifice; mini-boss rooms — unit tests, real-asset tests, rendered textures
- [x] Minimap marks — user, 2026-10-13
- [x] `lastl` (secret room way out, devil room way back), devil door frame 19, devil statue —
  unit tests, rendered textures
- [ ] Gambling, shop, messages, stats panel (LIB, SEED), cards, devil and mini-boss rooms — user
- [x] A SWF shape (Isaac's head, 2253) meshed with Ruffle's tessellator draws correctly through SDL3 `render_geometry` — automated screenshot
- [x] Red chest art (idle, landing, opened) renders from clip 2090 frames 5–7; turning-red odds and contents covered by tests — automated
- [x] Red chests in play — user
- [x] Card animation overlays drawn as vectors match the texture path (frozen-frame screenshots of The Sun and The Devil); all 158 overlay frames convert (`full_screen_overlays_draw_as_vectors`) — automated
- [x] Overlays drawn as vectors in play: crisp, "better than Rebirth" — user
- [x] Doors drawn as vectors match the texture path (screenshots, incl. a rotated side door) — automated
- [ ] Doors as vectors in play (all door kinds, shop unlocking animation) — user
- [x] Card-name banner and pickup message drawn as vectors match the texture path (screenshots) — automated
- [x] Re-render progress bar advances (logged 0 → 0.94 over the pass, frames keep coming) — automated
- [x] No stutter when a card's name slides in; progress bar works on fullscreen toggle — user
- [x] Devil/boss/mini-boss first-visit sounds; Isaac bumps troll bombs; shopkeeper flies flap — user
- [x] Pickup messages come from the game's script (test) and stay right-aligned by length (renders) — automated
- [x] Isaac's pickup animation holds the bought heart/bomb/key/soul heart or the card (renders) — automated
- [x] Kicking bombs, tears pushing bombs, the buying (pickup) animation — user
- [ ] In play: card swap like the original, shooting with two keys, doors shut while a machine runs — user
- [x] A loader crash ends in an error message and a message box instead of a splash that never closes (forced panic) — automated
- [x] Showcases only with `--showcase` (test) — automated
- [x] Damaged executables, bitmaps and bytecode fail with errors, not panics or huge allocations (synthetic tests); the real game files still load — automated
- [x] Black flies spawn from layouts, wander inside the room, die to a tear, bump without hurting; doors stay shut until the last dies (tests); flies and shut doors drawn in a screenshot — automated
- [ ] Black flies and door slamming/opening in play — user
- [x] Attack flies spawn from marker −9, chase the player, hurt half a heart on contact, take two
  tears; riding the unicorn kills any fly touched (tests); all 46 attack fly frames render — automated
- [x] Attack flies in play; a fly in the boss room keeps the doors shut until it dies — user, 2026-10-14
- [x] Doors open 10 frames after the last enemy dies and are usable at once; boss doors swing shut
  (16) and fade their glow in (15), treasure and curse doors play their clips (tests, renders) — automated
- [ ] Boss, treasure and curse doors closing and opening in play — user
- [x] Doors stay shut however long enemies live (a frame counter wrapped at 256 frames and
  replayed the clip every 8.5 s; test) — automated
- [x] The mini-boss room's icon (`mpiece` frame 8) shows once a door into it is seen (test) — automated
- [x] The mini-boss room's icon shows once the room is entered, not when its door is seen (only
  item 91 reveals it early) (test) — automated
- [ ] Mini-boss icon on the minimap in play — user
- [x] Flies buzz: the swarm loop plays at 30 % plus 30 % per fly (at most 150 %) while flies live (test) — automated
- [x] Buzzing loops without a gap (clips trimmed to `SeekSamples`/`SampleCount`) — user, 2026-10-14 (SDL3)
- [x] A blast at a shut plain door blows it open (usable, smoke frame); boss doors stay shut (test) — automated
- [x] Bombing out of a shut room — user, 2026-10-14
- [x] Enemies flash red for a few frames on a hit they survive (`hurtcol`); Pooters keep their
  variant while appearing (no flicker) — automated (build, render)
- [x] Red hit flash; Pooters spawn without flickering — user, 2026-10-14
- [x] Pooters: spawn from marker −5 (half-transparent: two-shot), shoot a player within 200 px
  in sight, shots hurt half a heart, land and splash; a black fly pecking its poop turns into a
  Pooter (about 2,000 frames on average); all Pooter, shot and splash frames render (tests) — automated
- [ ] Pooters and a fly turning into one in play — user
- [x] Stats panel lists LIB, SAC, CUR and MIN for the floor — automated (build)
- [x] Dying: no control, slides to a stop, `isaacdies.wav` on frame 32, darkness and beam fade in,
  the death note names the killer 51 frames later and stops the game, Space replays (tests, two
  screenshots with a temporary probe) — automated
- [ ] Dying and the death note in play — user
- [x] Gapers: spawn from marker −1 (a faint marker makes a calm one), appear for 45 frames, walk after
  the player, round rocks and pits by the shortest way (tests), hurt half a heart on touch and name
  themselves on the death note, die to three tears, the unicorn tramples them; every Gaper frame
  renders inside its canvas (`texture_extents`) and the game starts with them — automated
- [ ] Gapers in play (speed, going round rocks, the shouting head, the death burst) — user
- [x] Headless body and Horf: layouts' markers make them (read from the real layouts since 2026-10-24); a Gaper's lethal hit turns it into a headless
  body on a coin flip (not for blasts or the unicorn), the body wanders, spurts blood balls (one in
  three), hurts on touch under its own portrait; Horf stays put and shoots like a Pooter, under its own
  portrait; the bucket grid pushes enemies (flies, Pooters, Gapers, Horf) apart — unit-tested — automated
- [ ] Headless bodies, Horf and the crowding pushes in play — user
- [x] A dying Isaac stays drawn through the fatal hit's blink; a shot created in a frame cannot hurt in it — unit-tested
- [ ] Isaac's death no longer flickers in play, even with walkers around — user

## Tooling
- [x] `cargo fmt`, `cargo clippy --all-targets` clean; 28 tests pass (+3 opt-in real-asset tests); gameplay values behind the `GameParameters` API with a test reproducing the known results, including the 250-line file limit — automated

## Pills (2026-10-24)
- [x] Pill effect rolls give each look its own effect and only some kinds have a bad version; the Lucky Foot removes bad ones (automated).
- [x] Using a pill changes tears, range, speed, shot speed, luck, hearts, bombs and keys as described (automated).
- [x] The game's own texts for all 27 pill words are found in its script (automated, real SWF).
- [x] Full health pill works in play — user (2026-10-24).
- [ ] Pills look right in play: pickup, HUD strip, held-up picture, messages, banners — user
- [ ] The shop sells a pill ware that can be bought, and a held card or pill is pushed out onto the floor; Isaac gives a thumbs up or down after a good or bad pill — user
- [ ] Isaac's death animation no longer flickers or shows a black patch underneath — user

## Blasts between pits (2026-10-24)
- [x] A rock cleared between two pits leaves a broken floor; with a pit on one side only, or far from the blast, it does not (automated).
- [ ] The crack looks right in play after bombing a rock between two pits — user

## Clotty, Mulligan, Death card, Empress look (2026-10-24)
- [x] Clotty shoots four shots along the axes; Mulligan keeps away, bursts, leaves four flies, none when blown far past death; the Death card kills attack flies and Gapers; frantic Gapers keep their eyes shut after appearing (automated).
- [ ] Clotty and Mulligan look and move right in play (Basement rooms with them) — user
- [ ] The Death card kills attack flies in a room — user
- [ ] Dormant Gapers no longer open their eyes a second or two after entering — user
- [ ] The Empress gives Isaac the demonic look (black body, horns; wings when flying) — user

## Blue flies, enemies in fire (2026-10-25)
- [x] Owed blue flies appear one a frame, circle the player, hurt what they touch (once in 14 frames each) and follow up to ten into the next room; fire and spikes hurt walkers, fire pushes them (automated).
- [ ] Blue flies look right and behave in play (Friends pill, red chest) — user
- [ ] Clotty keeps its red look through its appearing and shooting clips — user

## Stats panel, grid lookups (2026-10-25)
- [x] The stats panel shows the tears pill's faster fire rate, shot speed and luck (automated).
- [x] Cells and places round-trip through `ingrid`/`outgrid`; enemies see doorways as solid, tears and other movers do not (automated).
- [ ] The Tears Up pill visibly lowers TRS on the left panel — user
- [ ] Clotty's shot and appearing animations are no longer cut off; Isaac's death animation neither — user

## Range pill, Isaac's death (2026-10-25)
- [x] Speed, range, tears, shot speed and luck pills change the stats panel (automated)
- [ ] The range pill changes RNG on the stats panel in play (run the rebuilt `target/desktop` launcher) — user
- [ ] Isaac's death shows only him, his spot on the floor, the blood and the beam: no colored bubbles or flicker — user
- [x] Blood donation: half a heart from red hearts first; counted towards the unlock (automated)

## Gapers, creep, music (2026-10-25)
- [ ] Frantic Gapers walk with closed eyes and open their hollow sockets and shout as soon as they see you, far away — user
- [ ] Calm Gapers (faint markers) keep the happy head — user
- [x] Blood creep hurts by a heart, slows in dark rooms, ignores fliers; `firecheck` is green (automated)
- [ ] The Basement's music plays from the start of a run (opening, intro, then loop) at a comfortable volume, in both frontends — user
- [ ] The shop, library, secret rooms and devil room change the music (fade, then their own track) and returning to the floor goes back to its loop — user
- [ ] Dying plays the death track; restarting starts the Basement's again — user

## Cursor hiding (user to check, both frontends)
- [x] (user, 2026-10-25) Using a gamepad hides the mouse cursor over the window; moving the mouse (or pressing a key) brings it back; holding the left button to aim still works.

## Enemies and music of the latest round (user to check)
- [ ] The music goes down while the treasure room, mini-boss room, devil room and secret room jingles play, then comes back up over about a second (both frontends). Automated so far: the game side (a jingle's sound takes the music's factor down frame by frame, `music/tests.rs`). The console prints "Music goes down under a jingle" and "Music is back after a jingle" when the frontend applies it, so a run from a terminal shows whether it happens.
- [ ] Hoppers hop at you without stopping, landing with a thud; flesh mounds wander; red flies chase and burst into four shots; Moters chase and split into two attack flies; the death note names them.

## Fade-in on a new run (user to check)
- [ ] Restarting (holding the restart key, or answering "Replay?") brings the new run up out of black over about half a second.

## Line checks and blood (2026-10-26)
- [x] `linecheck()`: rocks, walls and doorways end a walker's sight, spikes and webs cost nothing, marked cells cost 0.1 of 2.5; Gapers mark their cell and the marks fade every third frame, as does what a broken poop leaves — unit tests
- [x] `linecheckxx()`: blocks, closed key blocks and doors stop a blast, rocks and pits do not, a hidden wall can be reached, the last step can land past the end; a block between a bomb and a poop shields it — unit tests
- [x] Blasts push and hurt Isaac, enemies and pickups and open hidden walls only where they see them; machines break regardless — wired in `apply_blasts`, floor tests pass with the real grid
- [x] `splater()`: splats land `crand(5)` off and 5 up, upside down one time in two, tinted every seventh frame, drawn solid into one layer with the craters, shown at 70 % (SDL3: overlaps do not darken, probe screenshot 2026-10-26; Bevy applies the 70 % per item); a look of 0 keeps the last; dying flies bleed on three frames, Pooters smaller, the Clotty every fifth frame, a beheaded Gaper once — unit tests
- [x] The blood clip's ten red looks render inside a 48×28 canvas without unsupported features (`texture_extents`, `every_texture_renders_without_unsupported_features`); a temporary probe showed tinted, see-through splats under the actors in the SDL3 frontend — automated, screenshot
- [ ] Blood in play (dying enemies, the Clotty's trail, shots, tears now and then), gone when a room is entered again; bombs no longer reaching through metal blocks — user

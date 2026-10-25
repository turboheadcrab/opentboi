# Testing on a Steam Deck (Bazzite)

Status (2026-10-15): the tarball and the diagnostic tool exist and were tried **on a desktop PC
only**. Nothing here has been run on a Deck yet; the checklist below is what to run first.

## What the release already carries for this
- `opentboi-0.1.0-linux-x86_64.tar.zst`: static SDL3, built on Ubuntu 22.04, so the binary needs only
  glibc 2.35 or newer (checked: highest symbol `GLIBC_2.35`; links only `libc`, `libm`, `libgcc_s`).
  Bazzite (Fedora based) and SteamOS are both newer. Nothing has to be installed on the Deck.
- `install.sh`: copies into `~/.local` only, so it works on the immutable root of Bazzite and SteamOS
  with no `rpm-ostree`, no `pacman`, no root. `./install.sh --uninstall` removes it.
- `bin/opentboi-diagnose [SECONDS]`: collects system, graphics and controller names and the game's own
  log into `~/opentboi-report-DATE.txt` (no game files), runs the game for a few seconds. It works from a
  terminal **and as a non-Steam shortcut in Game Mode**, where there is no terminal.
- `scripts/build-tarball-in-container.sh`: builds that same tarball locally with podman or docker
  (about 10 minutes the first time), from the working tree, into `target/container-dist/`. CI does
  the same on a tag (`release.yml`); use the script until the repository is on GitHub.

## One-time preparation on the Deck
1. Install *The Binding of Isaac* from Steam on the Deck (Steam Play/Proton is on by default for games
   without a native Linux build; the game never has to be started). Games on the SD card live under
   `/run/media/...`; the finder reads Steam's library list, so that is fine. If it is not found,
   `TBOI_GAME_DIR=/path/to/game opentboi` points at it.
2. Get the tarball onto the Deck: `scp target/container-dist/*.tar.zst deck@HOST:` (turn on SSH in Bazzite
   with `sudo systemctl enable --now sshd`), or a USB stick, or the release page. Compare the `.sha256`.
3. Desktop Mode, Konsole: `tar --zstd -xf opentboi-*.tar.zst && cd opentboi-*/ && ./install.sh`.

## Checklist (in this order; stop and send the report at the first failure)
1. `opentboi --fullscreen`, from Konsole in Desktop Mode: starts, first-run extraction message, room draws.
   *Records:* start-up time (the log prints it), frame statistics line on exit (`--exit-after 20`).
2. `opentboi-diagnose 20` in Konsole: read the report, then keep it for comparison with Game Mode.
3. Add `opentboi` to Steam (Desktop Mode, "Add a Non-Steam Game", pick OpenTBOI from the list; set the
   launch option `--fullscreen`). Also add `opentboi-diagnose` the same way.
4. Game Mode: start the diagnostic shortcut first (no terminal there), come back to Desktop Mode and read
   `~/opentboi-report-*.txt`. Then start OpenTBOI itself.
   *Look at:* does it start at all; picture size and aspect (the panel is 1280×800, 16:10; the room is 4:3);
   sound; the performance overlay (Steam button, then X) frame rate and frame time graph.
5. Controls in Game Mode with the default Steam Input layout for a non-Steam game (unverified which
   template Steam picks): can you move, shoot with the right stick, drop a bomb on LB, quit?
   **Bomb hint:** on the first room the key under "BOMB" must read `E` until you touch a pad button or stick,
   then `LB`; pressing a key (or holding the mouse button) flips it back. The Deck's own buttons count as
   a gamepad. Touchscreen taps are not tested for this.
6. Sleep and wake in the middle of a run; leave it running 10 minutes (battery, temperature, no crash).
7. Refresh rate: the Deck's quick menu can set 40 to 60 Hz and 90 Hz on the OLED model. Note any stutter.

## What to send back
The report file(s) plus: Deck model (LCD or OLED), Bazzite image (`bazzite` Deck variant or desktop
variant, `rpm-ostree status` first lines), Game Mode or Desktop Mode, and what looked wrong. A screen
photo of the performance overlay is enough for frame times.

## Known unknowns (why this is a test, not a promise)
- Whether SDL3 inside gamescope's Game Mode picks Wayland or XWayland and whether either has pacing issues
  (the nested gamescope run on a PC showed uneven frame delivery, `VERIFIED.md`).
- The Deck's integrated GPU memory is shared with the system; the showcase room uses about 134 MiB of
  video memory on the PC (`docs/texture-streaming.md`), expected fine.
- Which Steam Input template a non-Steam shortcut gets, and whether it hides the pad's real type
  from SDL (affects the future glyph work).
- Start-up time: first launch extracts the SWF and decodes sounds (about 1 s on the PC).

## Testing with the Deck's disk in a PC (2026-10-16)
The original eMMC (SteamOS 3.6.24 and 3.6.22 from early 2025: kernel 6.5, Mesa 24.0.5, gamescope 3.15.16,
glibc 2.39) is in the development PC and mounted under `/run/media/$USER/`. Everything is done by
`scripts/steamos-disk-test.sh` without root.
- `scripts/steamos-disk-test.sh stage [TARBALL]`: installs the tarball (default: the newest in
  `target/container-dist/`) into the disk's `/home/deck/.local`, copies your game folder to
  `/home/deck/OpenTBOI-test/Isaac`, writes `opentboi-test.sh` (sets `TBOI_GAME_DIR`, starts
  `--fullscreen`) and `opentboi-diagnose-test.sh` beside it, and adds a Game Mode shortcut
  "OpenTBOI test" for the most recent Steam account (`scripts/steam-shortcuts.py`; the old
  `shortcuts.vdf` is saved as `shortcuts.vdf.before-opentboi`). Only while Steam is not running there.
- `scripts/steamos-disk-test.sh run [ARGS]`: runs the staged binary on this PC with the disk's root as `/`
  (bubblewrap), default `--showcase --exit-after 3`. It proves the binary, its glibc needs and the audio
  path work on SteamOS's userland. It cannot prove anything about the GPU (the development PC's GPU has no
  driver in that old Mesa, so it renders in software), SteamOS's kernel, Steam, gamescope or Game Mode.
- `scripts/steamos-disk-test.sh unstage` removes it all again.
- **Booting the disk here is not expected to give Game Mode** (RDNA4 needs a newer kernel and Mesa; not tried).
  For the real run, put the eMMC back in the Deck and use the checklist above from step 4. The staged
  game copy bypasses Steam, so also install Isaac through Steam once and run `opentboi` without
  `TBOI_GAME_DIR` to test the library finder.

## Possible later additions (not done)
- Start in fullscreen automatically under gamescope / on `SteamDeck=1`, so no launch option is needed.
- A `.desktop` action or a Decky plugin that does steps 1 to 3 for the user (`docs/distribution-options.md`).

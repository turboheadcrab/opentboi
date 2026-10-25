# Bevy vs SDL3 frontend (2026-10-11)

> Measured before the SDL3 frontend gained the original's splash screen. Since then it shows the
> splash 0.11 s after launch, loads and renders all textures behind it, and opens the game window
> after the original's 3 s splash time, so "first frame" below means the first game frame of the
> earlier design. Use `--exit-after` runs longer than the splash when re-measuring.

a current AMD desktop CPU (16 threads) and GPU (RADV), Fedora 44 Wayland, 144 Hz display. Same game code;
only `src/frontend/` differs. Runtime: release builds, standing still in the showcase room,
`scripts/benchmark-runtime.py` (10 s runs, averaged over 3, or 5 where noted).

## Build
`scripts/benchmark-build.sh` with `FRONTEND=sdl3|bevy` (separate target directories).

| | SDL3 | Bevy |
|---|---|---|
| Debug, clean | 16.7 s | 167.1 s |
| Debug, touched main.rs | 0.4 s | 1.9 s |
| Test build after debug build | 7.2 s | 6.2 s |
| Release, clean | 9.2 s | 100.3 s |
| Release, touched main.rs | 1.2 s | 2.4 s |
| Rust crates (normal + build deps) | 73 | 348 |
| Release binary (stripped) | 1.7 MiB + system libSDL3 3.2 MiB | 55 MiB |
| Shared libraries linked | libSDL3 | libudev, libasound |
| Frontend code | 653 lines | 526 lines |

## Runtime
Both render at the display's physical resolution (1280×960 for a 1024×768 window at GNOME's 125 %
scale).

| Configuration | First frame | Mean frame | 99th pct | Slowest | CPU | Peak RSS | GPU memory | Threads |
|---|---|---|---|---|---|---|---|---|
| SDL3, Vulkan renderer (our default, 5 runs) | 440 ms | 7.12 ms (140 fps) | 8.30 ms | 250 ms* | 10.2 % | 248 MiB | 73 MiB | 20 |
| SDL3, OpenGL renderer (SDL's default, 5 runs) | 417 ms | 7.13 ms (140 fps) | 7.30 ms | 237 ms* | 10.1 % | 271 MiB | 56 MiB | 26 |
| SDL3, SDL_GPU renderer | 463 ms | 7.13 ms (140 fps) | 7.27 ms | 242 ms* | 10.5 % | 245 MiB | 124 MiB | 20 |
| Bevy (wgpu, Vulkan) | 359 ms | 6.96 ms (144 fps) | 10.49 ms | 32.6 ms | 31.6 % | 283 MiB | 238 MiB | 27 |
| SDL3 Vulkan, no vsync | 427 ms | 0.10 ms (~10 000 fps) | 0.17 ms | 245 ms* | 103 % | 250 MiB | 73 MiB | 20 |
| Bevy, no vsync | 370 ms | 0.68 ms (1476 fps) | 1.12 ms | 41.5 ms | 232 % | 283 MiB | 243 MiB | 27 |

\* One stall right after the first frame: SDL's joystick layer starts there (~250 ms on this PC,
which has 35 `/dev/input` devices; SDL opens each one). It alone explains SDL's 140 vs 144 fps
mean: both have a 6.94 ms median. Starting gamepads before the window instead delays the window
by ~280 ms. Bevy's first frame is measured when its render world finished that frame.

## What the numbers say
- **Build:** SDL3 builds ~10× faster from clean and links ~2–5× faster after an edit, with a fifth
  of the crates. Most of Bevy's time is wgpu, naga and the ECS.
- **Running:** both hold the display's refresh. SDL3 uses a third of the CPU and a third or less
  of the GPU memory, and renders ~7× more frames per second when uncapped (the game is tiny, so
  this measures engine overhead). Bevy has fewer late frames at startup but more slightly late
  frames afterwards (99th percentile 10.5 ms vs 7.3–8.3 ms).
- **Startup:** about equal; SDL3 pays for its gamepad scan, Bevy for its renderer setup.
- **Looks:** screenshots of the same moment differ by 0.77/255 per channel on average; 3 % of
  pixels differ visibly, nearly all of them animations caught at slightly different times, the rest
  faint edges (SDL blends in sRGB space like Flash; Bevy blends in linear space).

## Notes
- OpenGL had runs that fell to ~100 fps in an earlier session (not in the final 5 runs; rechecked on
  2026-10-15 below, no repeat); Vulkan stayed even and uses fewer threads, hence the `SDL_RENDER_DRIVER` preference
  `vulkan,gpu,opengl` (the environment variable overrides it).
- The SDL3 window must ask for high pixel density; without it a scaled desktop upscales a smaller
  image and blurs it (found by comparing screenshots).
- SDL has no MP3 decoder; clips are decoded at startup with Symphonia (~6 ms for all 19).
- Not measured: input latency, battery use, other GPUs and the Steam Deck.

## Recheck of the OpenGL renderer (2026-10-15)
The `SDL_RENDER_DRIVER` choice was made from an old report ("OpenGL had runs that fell to ~100 fps").
Same PC (Wayland, 144 Hz, Mesa/RADV), current SDL3 frontend, x86-64-v3 release build, standing in
the showcase room, 20 s runs averaged over 3 (60 s once), vsync on, `scripts/benchmark-runtime.py`:

| Renderer | Mean frame | 99th pct | Slowest | GPU memory | Threads |
|---|---|---|---|---|---|
| Vulkan (run 1 / run 2) | 6.97 / 6.97 ms (143 / 144 fps) | 7.75 / 7.07 ms | 26 / 29 ms | 383 MiB | 22 |
| OpenGL (run 1 / run 2) | 6.94 / 6.94 ms (144 / 144 fps) | 7.28 / 7.28 ms | 25 / 24 ms | 425 MiB | 28 |
| Vulkan, 60 s | 6.96 ms | 6.99 ms | 28 ms | 383 MiB | 21 |
| OpenGL, 60 s | 6.97 ms | 7.25 ms | 37 ms | 426 MiB | 27 |
| XWayland, Vulkan (2 runs) | 7.12 ms (141 fps) | 10.3 ms | 42 ms | 383 MiB | 14 |
| XWayland, OpenGL (2 runs) | 6.99 ms (143 fps) | 8.6 ms | 29 ms | 461 MiB | 18 |

- No frame-pacing problem reproduced with OpenGL here: it held the refresh as evenly as Vulkan
  (its 99th percentile was even slightly better). The "slowest" column is the one start-up stall
  (gamepad scan, see above), not steady-state.
- Under XWayland both renderers are worse (99th percentile 9 to 10 ms, CPU use over 100 %); that is
  the X11 compatibility layer, not the renderer.
- This is one GPU, one driver and one compositor. The old report may have been another one, so
  this does not clear NVIDIA, Intel, older Mesa, or the Steam Deck. It does mean the fallback
  order `vulkan,gpu,opengl` has no known pacing penalty for OpenGL on this PC.
- The GPU memory column (383 MiB for Vulkan) was measured before most frame families moved to
  vector drawing. With the current SDL3 frontend the showcase room starts with 314 textures
  instead of 2096 (395 ms on 14 threads instead of 805 ms) and uses 134 MiB of GPU memory on
  Vulkan; only static pictures (room floor and walls, the stage surround, the HUD) and the
  frames with gradients are textures (`TODO.md`, "Vector art pipeline"). The frame-time columns
  above are unaffected (a later vsync run: 6.96 ms mean, 7.7 ms 99th percentile).

# Display modes: Wayland, X11, Windows and macOS

Why the remake cannot switch the monitor's resolution on Wayland the way the original did with
DirectX, how the other platforms differ, and what that means for this game.

## Terms

- **Display mode**: the resolution, refresh rate and timings the GPU actually sends to the
  monitor (for example 1280×720 at 60 Hz).
- **Mode switch**: changing that signal. The monitor resynchronizes (typically one to a few
  seconds of black screen) and, if the new mode is not its native one, its own scaler stretches
  the picture to the panel.
- **Exclusive fullscreen**: the game takes over an output, usually together with a mode switch,
  and presents frames to it directly.
- **Borderless fullscreen**: an ordinary window covering the whole screen at the desktop's mode.
  If the game renders fewer pixels, the GPU or compositor scales them up.

## What each platform allows

### X11 (Xorg)
Any client may change any output's mode through the XRandR extension (or the older XF86VidMode).
The change is global: every application and the whole desktop see the new resolution. SDL,
Wine and many older Linux games relied on this. Typical problems: desktop icons and windows get
rearranged, multi-monitor layouts shift, and if the game crashes the screen stays at the wrong
resolution until the user fixes it. Fullscreen windows can bypass the compositor ("unredirect"),
which used to matter for latency.

### Wayland
The compositor (GNOME Shell, KWin, Sway, gamescope, ...) owns the outputs. The core protocol
and `xdg-shell` give a client no request to change an output's mode. A game can only ask for its
window to be fullscreen on an output (`xdg_toplevel.set_fullscreen`); the compositor decides how
to show it. If the game's buffer is smaller than the screen, the compositor scales it (clients can
describe the desired scaling with `wp_viewporter`). Modern compositors can still put a fullscreen
window directly on screen without an extra copy ("direct scanout"), so this costs no latency.

The old `wl_shell` interface had a fullscreen "method" hint that included asking for a driver
mode switch; it is deprecated and gone from current compositors. Resolution changes are the
user's decision, made in the system's display settings.

Two consequences for games:
- **X11 games under XWayland** that try to switch modes are given an emulated mode change: XWayland
  reports the requested resolution to the game and scales its window, while the physical mode
  stays as it is.
- **Steam Deck (Game Mode)** runs games inside gamescope, a compositor built for this: a game may
  render at any resolution and gamescope upscales it to the panel (FSR, integer, nearest, linear).

This is a design choice, not a missing feature: one program cannot leave the user's screens in a
broken state, rearrange other applications or take outputs away from the compositor.

### Windows
Applications may switch modes with `ChangeDisplaySettingsEx`, and DirectX exclusive fullscreen
(`IDXGISwapChain::SetFullscreenState`) can take over an output with its own mode. The original
used this through Zinc's DirectX full-screen call. Windows 10 and 11 apply
"fullscreen optimizations" to many exclusive-fullscreen games, presenting them like a borderless
window with comparable latency. Alt+Tab out of true exclusive mode is slow and can minimize the
game or flash the screen.

### macOS
Core Graphics can still capture a display and set its mode (`CGDisplayCapture`,
`CGDisplaySetDisplayMode`), but Apple steers applications toward native fullscreen Spaces at the
desktop's mode. Retina displays already scale everything, and Apple recommends rendering at a
lower resolution and letting the system or Metal scale it, rather than changing modes.

## Switching modes: pros and cons

| | Switching the display mode | Rendering at a chosen size and scaling it |
|---|---|---|
| GPU cost | Fewer pixels rendered and scanned out | Fewer pixels rendered; scaling is nearly free on modern GPUs |
| Picture | On LCD/OLED panels, non-native modes are blurred by the monitor's scaler, sometimes with the wrong aspect ratio | Scaling filter is chosen by the game (sharp nearest, linear, FSR), aspect ratio preserved |
| CRT monitors | Real low resolution, the authentic look | Only an approximation |
| Switching speed | Seconds of black screen on every switch and Alt+Tab | Instant |
| Robustness | A crash can leave the desktop at the wrong resolution; other windows and monitors get rearranged | Nothing outside the game changes |
| Overlays, recording, streaming | Often break or show black frames | Work normally |
| Availability | X11, Windows, macOS (discouraged); not native Wayland | Everywhere, including Wayland and the Steam Deck |

Mode switching made sense when software renderers (like the Flash Player the original ran in)
struggled to fill large screens and monitors were CRTs. On today's flat panels its main effect is
a blurrier picture and a more fragile desktop.

## What this means for the remake

- **Performance is not a reason to switch modes.** The remake draws a few dozen textured
  rectangles per frame; filling a 4K screen is trivial for any GPU from the last decade.
- **Current behavior**: fullscreen is borderless at the monitor's own resolution, and the room is
  scaled to fit with letterboxing. This works identically on every platform.
- **Matching the original's resolution list** can be done portably with an *internal resolution*
  setting: render the game into an off-screen image of exactly 640×480, 800×600, 1280×720 and so
  on, then scale that image to the screen (optionally with sharp, nearest-neighbor scaling). The
  player sees exactly the original's pixel count on Wayland, X11, Windows, macOS and the Deck,
  without touching the monitor's mode.
- **True mode switching** could be offered as an optional extra where the platform allows it
  (X11, Windows, macOS; Bevy exposes it as `WindowMode::Fullscreen` with a specific video mode). It
  would be ignored or emulated on Wayland, so it should never be the only way to get a resolution.

# Distributing OpenTBOI on Linux: the options

Written 2026-10-15 (revised the same day after follow-up questions) for someone who has published Java/Groovy/Gradle artifacts (Maven Central, the
Gradle plugin portal) but not for Linux desktops.

## The mental model

Maven Central is one registry with one format that every build tool understands. Linux has no
equivalent. What you publish depends on **who installs it and how**:

| Java world | Linux world |
|---|---|
| Maven Central coordinates (`group:artifact:version`) | there is no single registry; each distro and each sandbox format has its own |
| a JAR runs anywhere with a JVM | a native binary runs only where its libraries (glibc, SDL3, ALSA, udev) are compatible |
| the JVM is the shared runtime | a Flatpak *runtime* plays that part; for a tarball it is the user's distro |
| signing keys for Central | GPG keys for a Flatpak repo or an AUR/COPR/PPA account; none for a tarball |

Our binary is a Rust program that links the system **SDL3**, **libudev** and **libasound**
(`ldd target/release/opentboi`); Wayland, X11 and Vulkan/GL libraries are loaded at run time.
Two things follow: a binary built on a new distro (Fedora 44 here) may refuse to start on an older
one (it needs a newer glibc), and SDL3 is too new to be installed everywhere. Every option below is
a different answer to those two problems.

Immutable distros (Bazzite, SteamOS, Fedora Silverblue/Kinoite, Aurora/Bluefin) matter here:
their root file system is read-only or image-based, so the right installs live in the user's home
or in a sandbox. Flatpak is their native format; Bazzite also ships Homebrew and Distrobox;
SteamOS wipes changes to the system partition on updates.

## Support policy

- **Supported systems only.** We target distributions their vendors still patch, as of 2026-10-15
  (dates from endoflife.date and the vendors; recheck before each release). Bugs that only
  happen on an end-of-life system are closed as "won't fix".

  | Supported | Until | End of life (not supported) |
  |---|---|---|
  | Ubuntu 22.04 LTS | spring 2027 (then drop) | Ubuntu 20.04, 25.10 |
  | Ubuntu 24.04 LTS, 26.04 LTS | 2029, 2031 | |
  | Debian 12 | LTS to June 2028 | Debian 11 (LTS ended August 2026) |
  | Debian 13 | 2028 security, LTS 2030 | |
  | Fedora 43, 44 | Dec 2026, Jun 2027 | Fedora 41, 42 |
  | RHEL, Alma, Rocky 9 and 10 | 2032, 2035 | (8 runs to 2029 but its glibc 2.28 is below our floor) |
  | openSUSE Leap 16.0, Tumbleweed | Oct 2027, rolling | Leap 15.6 |
  | Linux Mint 22 (21 until April 2027) | 2029 | |
  | SteamOS 3.x, Bazzite (Fedora 44 based) | rolling | |
  | Arch, CachyOS, Nobara | rolling | |

- **The glibc floor.** The tarball is built on the *oldest* supported system, so its glibc
  requirement is the lowest any supported user has: Ubuntu 22.04 (glibc 2.35). Debian 12 (2.36),
  Ubuntu 24.04 (2.39), Fedora and SteamOS are newer, so they run it. RHEL-family 8 (2.28) is below
  the floor, and so excluded. When Ubuntu 22.04 expires, move the release container to Debian 12 or
  Ubuntu 24.04 and update this table.
- **SDL3 availability** is why the tarball links SDL3 statically (feature `bundled-sdl3`): Ubuntu
  22.04/24.04 and Debian 12 ship no SDL3 package; Debian 13 has 3.2.x (the crate expects 3.4);
  Fedora 43 and 44 have 3.4.16 and 3.4.18. (Package indexes read 2026-10-15; not re-verified.)
- **Hardware floor:** baseline x86-64 (SSE2) CPU, and a GPU with Vulkan or OpenGL. The renderer
  list is `vulkan,gpu,opengl` (`PREFERRED_RENDER_DRIVERS`), so a Radeon HD 5770 (no Vulkan) falls
  back to OpenGL, and a GTX 660 uses Vulkan on the 470-series legacy driver or OpenGL. **Not
  tested on any of it**: the CPU level is verified statically, the GPUs are untested (see
  `VERIFIED.md`).
- **Source builds stay first-class.** CI (`.github/workflows/ci.yml`) builds and tests from the
  repository with nothing but rustup and the distribution's packages, on Fedora (system SDL3) and
  Ubuntu 24.04 (`--features bundled-sdl3`), and does *not* install `cargo-about`, so a contributor's
  path is exercised on every push. A scratch copy of the tree without git or cargo-about builds and
  passes the tests locally (`VERIFIED.md`).

## Options

### 1. Source only (`git clone` plus `cargo run`)

What the README does today.

- Pros: no packaging work; the GPL's "corresponding source" duty is trivially met.
- Cons: needs a Rust toolchain and SDL3-devel; slow first build (minutes); the audience is
  developers, not players. On SteamOS the root file system is read-only, so installing `SDL3-devel`
  means unlocking it, and the change is lost on the next update.
- Verdict: keep it for contributors only.

### 2. `cargo install` / crates.io

- Pros: familiar to Rust users; one command.
- Cons: still compiles on the user's machine with the same system libraries; no desktop entry or
  icon; crates.io is for libraries and tools, and a game that needs a Steam copy is an odd fit.
  We set `publish = false` in `Cargo.toml` (it also keeps `cargo about` from listing us as a
  third-party crate). `build.rs` writes `THIRD-PARTY-LICENSES` into the source tree, which
  `cargo publish` verification dislikes.
- Verdict: skip.

### 3. Prebuilt tarball on GitHub Releases, with an `install.sh`

A `.tar.zst` or `.tar.gz` with the binary, a `.desktop` file, an icon, `LICENSE`,
`THIRD-PARTY-LICENSES` and a script that copies them under `~/.local/{bin,share}`.

- Pros: easiest to produce (a GitHub Actions job); needs no root, so it works on immutable distros;
  the same shape serves Windows (`.zip`) and macOS (`.app` in a `.dmg`) later; no gatekeeper
  between you and users. This is the closest thing to "upload a JAR to a GitHub release".
- Cons: you decide what the binary links. To run on any distro it must be built on an old one (a
  container such as Debian oldstable or Ubuntu 22.04) with SDL3 linked **statically**
  (`sdl3-sys`'s `build-from-source-static` feature). No updates unless you build them
  (or the program checks for them). Users must trust a shell script. No sandbox.
- Verdict: **do first. Implemented**: `packaging/linux/` (own launcher `opentboi.desktop`, own
  icon `opentboi.svg`, `install.sh`) and `scripts/package-linux-tarball.sh`, which builds
  `target/dist/opentboi-VERSION-linux-ARCH.tar.zst` plus a `.sha256`, a reproducible archive
  (sorted, no owner, the commit's timestamp), with `LICENSE`, `THIRD-PARTY-LICENSES` and
  `SOURCE.txt` (commit and repository, for GPLv3 section 6). It does not use, read or depend on
  any desktop entry already on the build machine; `install.sh` writes the launcher with the
  absolute path of the installed binary and only touches files under the chosen prefix
  (default `~/.local`). `.github/workflows/release.yml` builds it on Ubuntu 22.04 with static SDL3.

### 4. Flatpak

A sandboxed application format with its own runtime (`org.freedesktop.Platform`), distributed
through a **remote** (a repository). Three ways to ship it:

| | Flathub | Your own repo (GitHub Pages) | A `.flatpak` bundle on a release |
|---|---|---|---|
| Install | `flatpak install flathub <id>`, or the software centre | `flatpak remote-add` once, then as usual | `flatpak install ./file.flatpak` |
| Updates | automatic | automatic | none |
| Review | yes, manual, and strict about naming and sandbox permissions | none | none |
| Discoverability | very high (Discover, GNOME Software, Bazzite's store, Steam Deck Desktop Mode) | low | none |
| Effort | highest (build from source offline, metainfo, review) | medium (you sign and host an OSTree repo) | lowest |

- Pros: the native format on SteamOS and Bazzite, and available on almost every desktop distro;
  runs on any distro with the same libraries, which removes the SDL3 and glibc problem; installs
  without root; sandboxed; automatic updates; works from Desktop Mode and can then be added to
  Steam so it appears in Game Mode.
- Difficulty of **Flathub** (researched 2026-10-15, flathub.org docs): about 7/10: the mechanics
  are a pull request against the `new-pr` branch of `flathub/flathub` with the manifest, reviewed
  by volunteers (no fixed turnaround), but the work is in the details. Builds are offline, so
  Rust crates go through `flatpak-cargo-generator` (and SDL3 must be a module, or static as in the
  tarball). The metainfo needs a valid SPDX licence, release entries, screenshots (which would show
  the original's art, an issue of its own) and an OARS rating, and `flatpak-builder-lint` must be
  clean. The app ID must be `io.github.<account>.OpenTBOI` (or a domain you control), and our
  window `app_id` is currently the hard-coded `opentboi`, so the ID must be configurable first
  (SDL's `SDL_APP_ID` hint). Naming needs care: Flathub does not accept names or icons that
  suggest affiliation, and the README says OpenTBOI is independent. There is precedent for engines that need the user's own game data (ScummVM,
  OpenMW), and permissions are scrutinised (`--filesystem` to the Steam library will be questioned;
  a file-chooser portal fallback is the safer ask). Flathub also requires disclosing
  AI-generated application code and forbids it in the manifest itself. Approval is not
  guaranteed, which is why the own-repo route comes first.
- Cons: more packaging work (a manifest, offline Rust sources generated by
  `flatpak-cargo-generator` from `Cargo.lock`, an AppStream `metainfo.xml`, an icon of our own);
  the sandbox hides the Steam library, so you must grant read access or let the user choose the
  folder through the file-chooser portal (see the questions document); Flathub reviewers may
  question a game that needs somebody else's game (engines such as OpenMW and ScummVM are on
  Flathub, so there is precedent); the app ID needs a domain or `io.github.<account>.OpenTBOI`.
- Verdict: **do second**; publish in your own repo and as a bundle first, apply to Flathub once
  the game is more complete.

### 5. AppImage

A single executable file that bundles the libraries and mounts itself.

- Pros: one file, no root, no installation (download, `chmod +x`, run); works on any distro;
  SteamOS Desktop Mode runs it fine.
- Cons: bundles libraries yourself (the same glibc-age problem as the tarball); no sandbox, no
  automatic updates unless you add AppImageUpdate; no central store; no desktop integration unless
  the user adds a helper. On Wayland and with gamepads it works as well as a plain binary.
- **Why FUSE 2 in 2026?** An AppImage is a small program (the *runtime*) glued to a SquashFS
  image; to run it, the runtime mounts the image through FUSE and then starts the app inside it.
  The original type-2 runtime was written against the FUSE **2** library (`libfuse.so.2`), and
  FUSE 3 is a different, incompatible ABI, so the old runtime cannot use it. Distributions moved
  to FUSE 3 (Fedora ships only FUSE 3 by default; Ubuntu 24.04 renamed the old library
  `libfuse2t64` and does not install it), so old AppImages say "dlopen(): error loading libfuse.so.2".
  The AppImage project has since built a statically linked type-2 runtime that needs no
  `libfuse2` on the host (it still uses the kernel's FUSE, not the library), but whether an
  image has it depends on the tool that made it (older `appimagetool`/`linuxdeploy` releases
  embed the old runtime; not verified for the current ones). Fallbacks that always work:
  `./App.AppImage --appimage-extract-and-run` (unpacks to a temporary folder), or install
  `libfuse2`/`fuse-libs`. Sources: docs.appimage.org (FUSE troubleshooting) and
  github.com/AppImage/type2-runtime.
- Verdict: optional; cheap once the tarball exists, and only with the static runtime.
- **Requirement (2026-10-15):** the AppImage must start on systems **without FUSE 2** (Fedora, Ubuntu 24.04
  by default): static type-2 runtime, verified by running it in a FUSE-3-only container in CI. See `TODO.md`.

### 6. Native distro packages

| Format | Where | Pros | Cons |
|---|---|---|---|
| `.deb` (`cargo-deb`) | a download, or an APT repo you host | familiar on Debian, Ubuntu, Mint; declares SDL3 as a dependency | needs SDL3 to exist in that release; repo hosting; you need one build per distro version |
| `.rpm` (`cargo-generate-rpm`) | a download, or Fedora **COPR** | easy on Fedora, where SDL3 is current | immutable Fedora (Bazzite, Silverblue) has to *layer* it with `rpm-ostree`, which slows updates and is discouraged |
| AUR (`PKGBUILD`) | Arch, CachyOS, Manjaro (via helpers) | very easy to write; builds from source against the user's SDL3; popular with Linux gamers | users run `yay`/`paru`; not for SteamOS because pacman changes are lost on update |
| Nix / Homebrew formula | `nixpkgs`, a Homebrew tap | reproducible; Bazzite ships Homebrew, which installs into the home directory | small audience; each has its own review and format |
| Official distro repositories | Debian, Fedora, Arch `extra` | the best result: automatic updates and trust | needs a maintainer and a stable project; a game that needs a copy of another game and borrows its name is a hard sell for now |

- Verdict: AUR and a COPR are cheap and reach enthusiasts; the others can wait. With CI the
  *building* is cheap (`cargo-deb`, `cargo-generate-rpm` after the tarball job), so publishing
  `.deb` and `.rpm` files on the release page is fine. The downsides are not the build:

  | Downside | Detail |
  |---|---|
  | One build per distribution release | A package links the distribution's libraries, so Fedora 44, Ubuntu 24.04 and Debian 13 each need their own build and CI container; a package built on a new release fails on an old one. A fully static binary packaged as `.deb`/`.rpm` avoids that, at the cost of "not how distributions do it". |
  | SDL3 is a hard dependency | Where the release has no SDL3 package (Ubuntu 22.04/24.04, Debian 12) the package must bundle it or it is uninstallable. |
  | Root and the system database | Installs need `sudo`, write to `/usr`, and fail on immutable systems (SteamOS) or need layering (`rpm-ostree`, which Bazzite discourages). A GitHub-hosted file never updates itself: no repository, no updates. |
  | Hosting a repository is a commitment | An APT or DNF repo needs a signing key, hosting, retention, and users who add it (with a key they must trust). COPR/PPA outsource the hosting but not the per-release builds. |
  | Support surface | Every package is another "it broke on my distro" channel; maintainers of the official repositories may also ship their own, diverging package. |
  | Policies | Official Debian/Fedora inclusion needs no bundled libraries (so no static SDL3), reproducible source, and a maintainer; a game that needs another publisher's game is a hard sell. |

  Recommendation: `.deb`/`.rpm` on the release page *after* the tarball and Flatpak, built from the
  same CI, clearly labelled as the "native" option. No repository until there is demand.

### 7. Snap

- Pros: a store on Ubuntu; auto-updates.
- Cons: Canonical-controlled store; ignored by Fedora, Arch and SteamOS; gamepad and Steam-library
  access is awkward; the audience overlaps with Flatpak.
- Verdict: skip.

### 8. Tools that automate this

- **cargo-dist**: builds release archives and installer scripts (shell, PowerShell, MSI, Homebrew
  formula) for Linux, macOS and Windows from GitHub Actions. Good for option 3 and the other
  platforms; it does not make Flatpaks. It will not link SDL3 statically for you.
- **flatpak-builder** plus `flatpak-cargo-generator`: option 4.
- **GitHub Actions** with a matrix job: one runner per platform; the Windows and macOS builds need
  their own runners (and macOS needs a code-signing identity and notarization to avoid warnings).

## Comparison

| | Works on SteamOS/Bazzite | No root | Any distro | Auto-update | Effort | Reach |
|---|---|---|---|---|---|---|
| Source | needs devel packages | yes | yes | no | none | developers |
| Tarball + `install.sh` | yes | yes | yes, if built on an old distro | no | low | anyone who reads a README |
| Flatpak (own repo or bundle) | **yes, natively** | yes | yes | repo only | medium | Linux gamers |
| Flathub | **yes, natively** | yes | yes | yes | high | everyone on Linux |
| AppImage | yes | yes | yes | optional | low | enthusiasts |
| AUR | no for SteamOS | no | Arch only | via helper | low | Arch users |
| `.rpm` / COPR | layering on atomic Fedora | no | Fedora family | via COPR | low | Fedora users |
| `.deb` | no | no | Debian family | via repo | medium | Ubuntu users |

## Recommendation

1. **Prebuilt tarball** from CI with static SDL3 and an `install.sh` (also the template for the
   Windows and macOS packages).
2. **Flatpak**: a self-hosted repo plus a bundle on the release page, then Flathub.
3. An **AUR package** for Arch users, an **AppImage** if people ask.
4. `.deb` and `.rpm` files on the release page (built by the same CI), once 1 and 2 exist.
5. The **Game Mode** story (non-Steam shortcut or a Decky plugin) is a convenience on top of 1 to 3,
   never the only way in.

## Flatpak sketch

Only what the sandbox needs. Details to verify when the manifest is written.

```yaml
app-id: io.github.<account>.OpenTBOI
runtime: org.freedesktop.Platform
runtime-version: '25.08'
sdk: org.freedesktop.Sdk
sdk-extensions: [org.freedesktop.Sdk.Extension.rust-stable]
command: opentboi
finish-args:
  - --socket=wayland
  - --socket=fallback-x11
  - --device=dri
  - --socket=pulseaudio                 # still the only audio socket; PipeWire answers it
  - --device=input
  - --device-if=all:!has-input-device   # old Flatpaks that do not know "input" keep working
  - --filesystem=xdg-data/Steam:ro
  - --filesystem=~/.var/app/com.valvesoftware.Steam/.local/share/Steam:ro
  - --filesystem=/run/media:ro          # Steam Deck SD card
```

SDL3 must be built as a module if the runtime does not ship it. Rust crates are vendored offline
through `cargo-sources.json`; the git dependency on Ruffle works with the generator.

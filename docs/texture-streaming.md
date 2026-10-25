# Drawing textures on demand

## Baseline (2026-10-11, everything pre-rendered at startup)
A current AMD desktop CPU and GPU, release build, `cargo test --release asset_load_timings -- --ignored --nocapture`:

| Step | Time |
|---|---|
| Read the extracted SWF (46 MB) | 31 ms |
| Parse it (Ruffle's `swf` crate) | 153 ms |
| Index characters | 2 ms |
| Render 382 textures (92.7 MiB of RGBA pixels) | 329 ms |
| Collect sounds | < 1 ms |

Game: "Loaded assets in 580 ms", peak resident memory 422 MB, GPU memory +~600 MiB while running
(whole-system VRAM reading, so other programs add noise).

Where memory goes: every texture existed three times for a moment — our `RgbaImage`, a cloned Bevy
`Image` (the plugin copied each one), and the GPU upload — and all 382 were uploaded although a
room shows a few dozen at a time.

## Plan
1. **Keep the parsed SWF for the whole run.** The parse result borrows the decompressed buffer, so
   both are leaked once at startup (`Box::leak`) and live as long as the game. This makes a
   long-lived `TextureRenderer` possible without self-referential structs.
2. **Render on first use.** The frontend asks the renderer for a `TextureId` the first time a draw
   item needs it, uploads the image, and keeps only the GPU copy (`RenderAssetUsages::RENDER_WORLD`).
   Failures are remembered so a broken texture is attempted once and then drawn with its fallback
   color.
3. **Render the room background up front**, because it decides between artwork and placeholders
   and is needed on the first frame anyway.
4. **Measure again**: startup time, peak memory, GPU memory, and the worst single-frame cost of a
   first-use render (the largest textures are the fireplace smoke and poop bursts).

## Later, if first-use hitches show up
- Render on a worker thread and show the fallback (or the previous frame) until it is ready.
- Prefetch: when an animation starts, queue all of its frames.
- Evict textures not drawn for a while (an LRU budget), since everything can be re-rendered.
- Pack small frames into atlases to cut GPU allocations.

## Result (2026-10-11, implemented)
Renderer kept alive (`TextureRenderer`), room background rendered at startup, everything else on
first use (`frontend/texture_library.rs`); `--prerender-textures` restores the old behavior for
comparison. Measured 4 s after launch, standing still in the showcase room:

| | Lazy (default) | Eager (`--prerender-textures`) | Before (eager + extra CPU copy) |
|---|---|---|---|
| "Loaded assets in" | 199 ms | 560 ms | 580 ms |
| Peak resident memory | 282 MiB | 347 MiB | 422 MiB |
| GPU memory increase | +242 MiB | +548 MiB | ~+600 MiB |

On-demand rendering in those 4 s: 99 textures (room objects, idle loops, Isaac), 104 ms in total,
slowest single texture 2.0 ms (TNT barrel) — no visible hitch; the first frame pays for ~20 at once.
The game prints these statistics when it exits.

Keeping the parsed SWF costs memory, but lazy rendering still uses 65 MiB less than eager at peak.
The worker-thread, prefetch and eviction ideas above stay on hold until a hitch is measured.

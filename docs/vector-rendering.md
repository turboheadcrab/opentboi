# Drawing the vectors instead of rasterizing them

Today every texture is rasterized on the CPU (`vector_art`) at screen density, one image per
animation frame, which costs a lot of video memory (see PROGRESS.md: ~190 MiB for the shop's
machine explosions at 1080p). Flash itself drew shapes as vectors every frame.

## Approach
1. **Meshes** (`src/game_assets/vector_mesh/`): each `DefineShape` becomes triangles once,
   cached by character id. Our own types (`ShapeMesh`, `MeshDraw`, `MeshFill`) behind the
   `ShapeMesher` trait; today `RuffleMesher` wraps Ruffle's tessellator (`ruffle_render`, lyon),
   pinned to one Ruffle commit together with `swf`. A lyon-based mesher of our own can replace
   it without touching anything else.
   Status: every shape of the Steam SWF (5,231 shapes, 4.8 M triangles) and of the demo (3,413)
   tessellates, about a second each in a release build (ignored tests).
2. **Display lists**: per frame, walk the clips like `vector_art::render_pass` does, but emit
   (mesh, transform, color effect) instead of pixels.
3. **Frontends**: solid fills first (SDL3 `SDL_RenderGeometry`; Bevy `Mesh2d` with vertex
   colors), then gradients (a shader, or a 256×1 texture per gradient), bitmap fills (textures),
   masks (stencil). Batch draws sharing a fill.
4. Keep rasterized textures as the fallback until the mesh path draws everything correctly.

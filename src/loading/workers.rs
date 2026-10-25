//! Loading work on other threads: the whole load in the background, and textures on many cores.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::Sender;
use std::time::Instant;

use super::{CORES_KEPT_FREE, LoadEvent, load, send};
use crate::game::TextureId;
use crate::game_assets::TextureRenderer;

/// [`load`] on its own thread; a panic there reaches the frontend as [`LoadEvent::Failed`]
/// instead of a channel that simply goes quiet.
#[cfg_attr(
    not(feature = "sdl3-frontend"),
    expect(dead_code, reason = "only the SDL3 frontend loads in the background")
)]
pub fn load_in_background(
    pixels_per_world_unit: f32,
    drawn_as_vectors: fn(&TextureRenderer, TextureId) -> bool,
    events: Sender<LoadEvent>,
) {
    std::thread::spawn(move || {
        let loaded = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            load(pixels_per_world_unit, drawn_as_vectors, &events);
        }));
        if let Err(panic) = loaded {
            let reason = panic
                .downcast_ref::<&str>()
                .map(ToString::to_string)
                .or_else(|| panic.downcast_ref::<String>().cloned())
                .unwrap_or_else(|| "unknown error".to_string());
            send(&events, LoadEvent::Failed(format!("Loading the game failed: {reason}")));
        }
    });
}

/// Renders `texture_ids` on all cores but [`CORES_KEPT_FREE`], sending each as it is done. Also
/// used to redo every texture when the window's pixel density changes.
pub fn render_textures(
    renderer: &TextureRenderer,
    texture_ids: &[TextureId],
    pixels_per_world_unit: f32,
    events: &Sender<LoadEvent>,
) {
    let started = Instant::now();
    let available_cores = std::thread::available_parallelism().map_or(1, usize::from);
    let worker_count = available_cores.saturating_sub(CORES_KEPT_FREE).max(1);
    let next_index = AtomicUsize::new(0);
    std::thread::scope(|scope| {
        for _ in 0..worker_count {
            let worker_events = events.clone();
            let next_index = &next_index;
            scope.spawn(move || {
                while let Some(&texture_id) = texture_ids.get(next_index.fetch_add(1, Ordering::Relaxed)) {
                    match renderer.render(texture_id, pixels_per_world_unit) {
                        Ok((image, _)) => {
                            if !send(&worker_events, LoadEvent::Texture(texture_id, image)) {
                                return;
                            }
                        }
                        Err(render_error) => println!("{texture_id:?} artwork unavailable: {render_error}"),
                    }
                }
            });
        }
    });
    println!(
        "Rendered {} textures at {pixels_per_world_unit:.2} px per unit in {} ms on {worker_count} threads",
        texture_ids.len(),
        started.elapsed().as_millis()
    );
}

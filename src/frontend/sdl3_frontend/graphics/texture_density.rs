//! Keeps textures rendered at the window's pixel density, so art is drawn 1:1 instead of being
//! shrunk or stretched by the GPU. When the density changes (resize, fullscreen), every texture
//! is rendered again on the worker threads and swapped in as it arrives.

use std::sync::mpsc::{Receiver, TryRecvError};
use std::time::{Duration, Instant};

use glam::Vec2;
use sdl3::render::WindowCanvas;

use crate::frontend::sdl3_frontend::graphics::texture_library::TextureLibrary;
use crate::frontend::sdl3_frontend::graphics::vector_families::is_drawn_as_vectors;
use crate::game::{self, TextureId};
use crate::loading::{self, LoadEvent};

/// Time per frame spent uploading re-rendered textures, so frames (and the progress bar) keep
/// coming while the workers render faster than one frame can upload.
const UPLOAD_BUDGET_PER_FRAME: Duration = Duration::from_millis(8);
/// Density changes smaller than this keep the current textures (avoids work while dragging).
const RELATIVE_TOLERANCE: f32 = 0.05;

pub struct TextureDensity {
    pixels_per_world_unit: f32,
    rerender: Option<Receiver<LoadEvent>>,
    /// Textures re-rendered so far in the current pass, out of `total_count`.
    received_count: usize,
    total_count: usize,
}

impl TextureDensity {
    pub fn new(pixels_per_world_unit: f32) -> Self {
        Self {
            pixels_per_world_unit,
            rerender: None,
            received_count: 0,
            total_count: 0,
        }
    }

    /// While textures are being re-rendered, the fraction done (the game pauses meanwhile).
    pub fn rerender_progress(&self) -> Option<f32> {
        self.rerender
            .as_ref()
            .map(|_| self.received_count as f32 / self.total_count.max(1) as f32)
    }

    /// Uploads re-rendered textures and starts a new pass if the window's density moved. Call only
    /// once loading has finished, so the loader and a re-render never overlap.
    pub fn follow_window(&mut self, canvas: &WindowCanvas, textures: &mut TextureLibrary) {
        if let Some(receiver) = &self.rerender {
            let started = Instant::now();
            loop {
                if started.elapsed() > UPLOAD_BUDGET_PER_FRAME {
                    return;
                }
                match receiver.try_recv() {
                    Ok(LoadEvent::Texture(texture_id, image)) => {
                        textures.insert(texture_id, &image);
                        self.received_count += 1;
                    }
                    Ok(_) => {}
                    Err(TryRecvError::Empty) => return,
                    Err(TryRecvError::Disconnected) => break,
                }
            }
            self.rerender = None;
        }
        let Ok((width, height)) = canvas.output_size() else {
            return;
        };
        let wanted = game::pixels_per_world_unit(Vec2::new(width as f32, height as f32));
        let Some(renderer) = textures.renderer() else { return };
        if (wanted / self.pixels_per_world_unit - 1.0).abs() < RELATIVE_TOLERANCE {
            return;
        }
        self.pixels_per_world_unit = wanted;
        textures.set_pixels_per_world_unit(wanted);
        let (sender, receiver) = std::sync::mpsc::channel();
        let mut texture_ids = TextureId::all();
        texture_ids.retain(|&texture_id| !is_drawn_as_vectors(renderer, texture_id));
        self.total_count = texture_ids.len();
        self.received_count = 0;
        std::thread::spawn(move || loading::render_textures(renderer, &texture_ids, wanted, &sender));
        self.rerender = Some(receiver);
    }
}

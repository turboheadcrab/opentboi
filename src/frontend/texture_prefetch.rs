//! Renders textures the game will soon need (`Game::upcoming_textures`) on background threads, so
//! the frame that first shows them only uploads them. Shared by both frontends.

use std::collections::HashSet;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Mutex};
use std::thread;

use crate::game::TextureId;
use crate::game_assets::{RgbaImage, TextureRenderer};

/// Background renderers; the rest of the cores stay free for the game and the frontend.
const WORKER_COUNT: usize = 2;
/// How many finished textures are taken (and uploaded) per frame, to spread the uploads.
const MAXIMUM_UPLOADS_PER_FRAME: usize = 8;

type Request = (TextureId, f32);

pub struct TexturePrefetcher {
    requests: Sender<Request>,
    /// Behind a lock only so the prefetcher can live in a Bevy resource (which must be `Sync`).
    finished: Mutex<Receiver<(TextureId, f32, Option<RgbaImage>)>>,
    /// Asked for already (finished or not), so nothing is rendered twice.
    requested: HashSet<(TextureId, u32)>,
}

impl TexturePrefetcher {
    pub fn new(renderer: &'static TextureRenderer) -> Self {
        let (requests, request_queue) = channel::<Request>();
        let (finished_sender, finished) = channel();
        let request_queue = Arc::new(Mutex::new(request_queue));
        for _ in 0..WORKER_COUNT {
            let request_queue = Arc::clone(&request_queue);
            let finished_sender = finished_sender.clone();
            thread::spawn(move || {
                loop {
                    let Ok(request) = request_queue.lock().map(|queue| queue.recv()) else {
                        return;
                    };
                    let Ok((texture_id, pixels_per_world_unit)) = request else {
                        return;
                    };
                    let image = renderer
                        .render(texture_id, pixels_per_world_unit)
                        .ok()
                        .map(|(image, _)| image);
                    if finished_sender
                        .send((texture_id, pixels_per_world_unit, image))
                        .is_err()
                    {
                        return;
                    }
                }
            });
        }
        Self {
            requests,
            finished: Mutex::new(finished),
            requested: HashSet::new(),
        }
    }

    /// Queues what is not known yet (`is_known`) at this density, most urgent first.
    pub fn request(
        &mut self,
        upcoming: &[TextureId],
        pixels_per_world_unit: f32,
        is_known: impl Fn(TextureId) -> bool,
    ) {
        for &texture_id in upcoming {
            let key = (texture_id, pixels_per_world_unit.to_bits());
            if !is_known(texture_id) && self.requested.insert(key) {
                let _ = self.requests.send((texture_id, pixels_per_world_unit));
            }
        }
    }

    /// Textures finished at `pixels_per_world_unit` since the last call (a few per frame);
    /// ones rendered for an older density are dropped.
    pub fn take_finished(&mut self, pixels_per_world_unit: f32) -> Vec<(TextureId, RgbaImage)> {
        let Ok(finished) = self.finished.get_mut() else {
            return Vec::new();
        };
        finished
            .try_iter()
            .take(MAXIMUM_UPLOADS_PER_FRAME)
            .filter(|(_, density, _)| density.to_bits() == pixels_per_world_unit.to_bits())
            .filter_map(|(texture_id, _, image)| Some((texture_id, image?)))
            .collect()
    }
}

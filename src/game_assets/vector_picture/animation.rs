//! Which animations are drawn as meshes throughout.

use crate::game::TextureId;
use crate::game_assets::TextureRenderer;

impl TextureRenderer {
    /// Whether every frame of the animation `texture_id` belongs to can be drawn as meshes; always
    /// so for textures that are not a frame of Isaac's animations.
    pub(in crate::game_assets) fn animation_is_drawable(&self, texture_id: TextureId) -> bool {
        let TextureId::IsaacAnimation { animation, .. } = texture_id else {
            return true;
        };
        let known = self
            .animations_drawable
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(&animation)
            .copied();
        known.unwrap_or_else(|| {
            let drawable = (0..animation.frame_count()).all(|frame| {
                self.frame_picture(TextureId::IsaacAnimation { animation, frame })
                    .is_some()
            });
            self.animations_drawable
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .insert(animation, drawable);
            drawable
        })
    }
}

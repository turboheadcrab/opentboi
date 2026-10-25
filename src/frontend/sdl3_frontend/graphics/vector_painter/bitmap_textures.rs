//! The SWF's bitmaps on the GPU for bitmap fills, uploaded on first use: each as it is, and as a
//! white silhouette for the additive pass of a color effect's offset.

use crate::game_assets::RgbaImage;
use std::collections::HashMap;

use sdl3::pixels::PixelFormat;
use sdl3::render::{ScaleMode, Texture, TextureCreator};
use sdl3::sys::blendmode::{SDL_BLENDMODE_ADD_PREMULTIPLIED, SDL_BLENDMODE_BLEND_PREMULTIPLIED, SDL_BlendMode};
use sdl3::video::WindowContext;

use crate::game_assets::TextureRenderer;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BitmapKey {
    pub bitmap_id: u16,
    /// The bitmap's coverage in white, blended additively.
    pub silhouette: bool,
}

pub struct BitmapTextures<'creator> {
    creator: &'creator TextureCreator<WindowContext>,
    /// `None` for bitmaps that failed to decode or upload.
    textures: HashMap<BitmapKey, Option<Texture<'creator>>>,
}

impl<'creator> BitmapTextures<'creator> {
    pub fn new(creator: &'creator TextureCreator<WindowContext>) -> Self {
        Self {
            creator,
            textures: HashMap::new(),
        }
    }

    pub fn get(&self, key: BitmapKey) -> Option<&Texture<'creator>> {
        self.textures.get(&key)?.as_ref()
    }

    /// Uploads whichever of `keys` are not on the GPU yet.
    pub fn upload_missing(&mut self, renderer: &TextureRenderer, keys: impl Iterator<Item = BitmapKey>) {
        for key in keys {
            if !self.textures.contains_key(&key) {
                let texture = self.upload(renderer, key);
                self.textures.insert(key, texture);
            }
        }
    }

    fn upload(&self, renderer: &TextureRenderer, key: BitmapKey) -> Option<Texture<'creator>> {
        let mut image = renderer.bitmap(key.bitmap_id)?;
        if key.silhouette {
            for pixel in image.pixels.as_chunks_mut::<{ RgbaImage::BYTES_PER_PIXEL }>().0 {
                let alpha = pixel[RgbaImage::ALPHA_CHANNEL];
                pixel.fill(alpha);
            }
        }
        let mut texture = self
            .creator
            .create_texture_static(PixelFormat::RGBA32, image.width, image.height)
            .ok()?;
        texture
            .update(None, &image.pixels, image.width as usize * RgbaImage::BYTES_PER_PIXEL)
            .ok()?;
        texture.set_scale_mode(ScaleMode::Linear);
        let blend_mode = if key.silhouette {
            SDL_BLENDMODE_ADD_PREMULTIPLIED
        } else {
            SDL_BLENDMODE_BLEND_PREMULTIPLIED
        };
        set_texture_blend_mode(&texture, blend_mode);
        Some(texture)
    }
}

/// The safe API has no premultiplied blend modes.
pub fn set_texture_blend_mode(texture: &Texture, blend_mode: SDL_BlendMode) {
    // SAFETY: the texture is alive and the mode is a valid SDL blend mode.
    unsafe { sdl3::sys::render::SDL_SetTextureBlendMode(texture.raw(), blend_mode) };
}

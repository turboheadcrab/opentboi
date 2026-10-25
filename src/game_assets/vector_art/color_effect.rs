//! Flash color transforms: `channel * multiply + add`, applied per channel including alpha.

const CHANNEL_MAXIMUM: f32 = 255.0;
const CHANNEL_COUNT: usize = 4;
const ALPHA_CHANNEL: usize = 3;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ColorEffect {
    /// Red, green, blue, alpha multipliers.
    multiply: [f32; CHANNEL_COUNT],
    /// Red, green, blue, alpha offsets in `0.0..=1.0` units.
    add: [f32; CHANNEL_COUNT],
}

impl ColorEffect {
    /// Flash's `_alpha` divided by 100.
    pub fn alpha_multiplier(&self) -> f32 {
        self.multiply[ALPHA_CHANNEL]
    }

    pub const IDENTITY: Self = Self {
        multiply: [1.0; CHANNEL_COUNT],
        add: [0.0; CHANNEL_COUNT],
    };

    /// A script's tint on a whole clip.
    pub fn from_tint(tint: super::Tint) -> Self {
        let [red, green, blue] = tint.multiply_per_mille.map(|per_mille| f32::from(per_mille) / 1000.0);
        let [red_add, green_add, blue_add] = tint.add.map(|offset| f32::from(offset) / CHANNEL_MAXIMUM);
        Self {
            multiply: [red, green, blue, 1.0],
            add: [red_add, green_add, blue_add, 0.0],
        }
    }

    /// Red, green and blue multiplied by `factor`, alpha kept.
    pub fn brightness(factor: f32) -> Self {
        Self {
            multiply: [factor, factor, factor, 1.0],
            add: [0.0; CHANNEL_COUNT],
        }
    }

    pub fn from_swf(transform: &swf::ColorTransform) -> Self {
        Self {
            multiply: [
                transform.r_multiply.to_f32(),
                transform.g_multiply.to_f32(),
                transform.b_multiply.to_f32(),
                transform.a_multiply.to_f32(),
            ],
            add: [
                f32::from(transform.r_add) / CHANNEL_MAXIMUM,
                f32::from(transform.g_add) / CHANNEL_MAXIMUM,
                f32::from(transform.b_add) / CHANNEL_MAXIMUM,
                f32::from(transform.a_add) / CHANNEL_MAXIMUM,
            ],
        }
    }

    /// The effect of applying `child` first and then `self`, as nesting does in Flash.
    pub fn then_child(self, child: Self) -> Self {
        let mut combined = Self::IDENTITY;
        for channel in 0..CHANNEL_COUNT {
            combined.multiply[channel] = self.multiply[channel] * child.multiply[channel];
            combined.add[channel] = self.multiply[channel] * child.add[channel] + self.add[channel];
        }
        combined
    }

    /// A straight RGBA color (0–255) through the effect, premultiplied, in `0.0..=1.0`.
    pub fn apply_premultiplied(self, rgba: [u8; CHANNEL_COUNT]) -> [f32; CHANNEL_COUNT] {
        let [red, green, blue, alpha]: [f32; CHANNEL_COUNT] = std::array::from_fn(|channel| {
            (f32::from(rgba[channel]) / CHANNEL_MAXIMUM * self.multiply[channel] + self.add[channel]).clamp(0.0, 1.0)
        });
        [red * alpha, green * alpha, blue * alpha, alpha]
    }

    /// For tinting a premultiplied bitmap on the GPU, where texels can only be multiplied: the
    /// multipliers (`[r·a, g·a, b·a, a]`, `a` the alpha multiplier) and what a second, additive pass
    /// adds where the bitmap is opaque (`add·a`). Flash's alpha offset is left out.
    pub fn bitmap_tint(self) -> ([f32; CHANNEL_COUNT], [f32; CHANNEL_COUNT]) {
        let alpha = self.multiply[ALPHA_CHANNEL].clamp(0.0, 1.0);
        let multiply = std::array::from_fn(|channel| {
            if channel == ALPHA_CHANNEL {
                alpha
            } else {
                self.multiply[channel].max(0.0) * alpha
            }
        });
        let add = std::array::from_fn(|channel| {
            if channel == ALPHA_CHANNEL {
                0.0
            } else {
                self.add[channel].clamp(0.0, 1.0) * alpha
            }
        });
        (multiply, add)
    }

    pub fn apply(self, color: swf::Color) -> tiny_skia::Color {
        let channels = [color.r, color.g, color.b, color.a];
        let transformed: [f32; CHANNEL_COUNT] = std::array::from_fn(|channel| {
            (f32::from(channels[channel]) / CHANNEL_MAXIMUM * self.multiply[channel] + self.add[channel])
                .clamp(0.0, 1.0)
        });
        tiny_skia::Color::from_rgba(transformed[0], transformed[1], transformed[2], transformed[3])
            .unwrap_or(tiny_skia::Color::TRANSPARENT)
    }

    /// Exact bit pattern of the effect, for caching recolored bitmaps per effect.
    pub fn cache_key(self) -> [u32; 2 * CHANNEL_COUNT] {
        std::array::from_fn(|index| {
            if index < CHANNEL_COUNT {
                self.multiply[index].to_bits()
            } else {
                self.add[index - CHANNEL_COUNT].to_bits()
            }
        })
    }

    /// Applies the effect to one premultiplied RGBA pixel, as Flash does to bitmaps.
    pub fn apply_to_premultiplied_pixel(self, pixel: [u8; CHANNEL_COUNT]) -> [u8; CHANNEL_COUNT] {
        let alpha = f32::from(pixel[CHANNEL_COUNT - 1]) / CHANNEL_MAXIMUM;
        let straight: [f32; CHANNEL_COUNT] = std::array::from_fn(|channel| {
            if channel == CHANNEL_COUNT - 1 {
                alpha
            } else if alpha > 0.0 {
                f32::from(pixel[channel]) / CHANNEL_MAXIMUM / alpha
            } else {
                0.0
            }
        });
        let transformed: [f32; CHANNEL_COUNT] = std::array::from_fn(|channel| {
            (straight[channel] * self.multiply[channel] + self.add[channel]).clamp(0.0, 1.0)
        });
        let new_alpha = transformed[CHANNEL_COUNT - 1];
        std::array::from_fn(|channel| {
            let value = if channel == CHANNEL_COUNT - 1 {
                new_alpha
            } else {
                transformed[channel] * new_alpha
            };
            crate::whole_numbers::clamped_u8((value * CHANNEL_MAXIMUM).round())
        })
    }
}

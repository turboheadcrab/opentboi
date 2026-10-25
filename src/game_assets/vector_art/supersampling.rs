//! Flash's HIGH quality anti-aliasing: rasterize without anti-aliasing at 4×4 the resolution and
//! average each 4×4 block. Unlike anti-aliasing every path on its own, this leaves no faint seams
//! where two fills of a shape meet.

use tiny_skia::Pixmap;

/// Samples per output pixel along each axis (HIGH quality).
pub const SUPERSAMPLING_FACTOR: u32 = 4;
/// Large textures (the room at fullscreen) use fewer samples to bound the temporary buffer.
const MAXIMUM_SUPERSAMPLED_PIXELS: u64 = 16 * 1024 * 1024;
const CHANNELS: usize = 4;

/// The factor to use for an output of `width` × `height`: 4, or less for huge canvases.
pub fn factor_for(width: u32, height: u32) -> u32 {
    let output_pixels = u64::from(width.max(1)) * u64::from(height.max(1));
    (1..=SUPERSAMPLING_FACTOR)
        .rev()
        .find(|&factor| output_pixels * u64::from(factor * factor) <= MAXIMUM_SUPERSAMPLED_PIXELS)
        .unwrap_or(1)
}

/// Averages `factor`×`factor` blocks of premultiplied pixels into a pixmap `factor` times smaller.
pub fn downsample(source: &Pixmap, factor: u32) -> Option<Pixmap> {
    if factor == 1 {
        return Some(source.clone());
    }
    let (width, height) = (source.width() / factor, source.height() / factor);
    let mut output = Pixmap::new(width, height)?;
    let source_data = source.data();
    let source_stride = source.width() as usize * CHANNELS;
    let samples = factor * factor;
    let factor = factor as usize;
    for (output_index, output_pixel) in output.data_mut().as_chunks_mut::<CHANNELS>().0.iter_mut().enumerate() {
        let (column, row) = (output_index % width as usize, output_index / width as usize);
        let mut sums = [0u32; CHANNELS];
        for sample_row in row * factor..(row + 1) * factor {
            let row_start = sample_row * source_stride + column * factor * CHANNELS;
            for sample in source_data[row_start..row_start + factor * CHANNELS]
                .as_chunks::<CHANNELS>()
                .0
            {
                for (sum, &value) in sums.iter_mut().zip(sample) {
                    *sum += u32::from(value);
                }
            }
        }
        for (target, sum) in output_pixel.iter_mut().zip(sums) {
            *target = u8::try_from((sum + samples / 2) / samples).unwrap_or(u8::MAX);
        }
    }
    Some(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn averages_blocks_and_limits_huge_canvases() {
        let mut source = Pixmap::new(4, 4).unwrap();
        source.data_mut()[..CHANNELS].copy_from_slice(&[255, 255, 255, 255]);
        let output = downsample(&source, 4).unwrap();
        assert_eq!(output.data(), &[16, 16, 16, 16]);
        assert_eq!(factor_for(100, 100), SUPERSAMPLING_FACTOR);
        assert!(factor_for(2560, 1700) < SUPERSAMPLING_FACTOR);
    }
}

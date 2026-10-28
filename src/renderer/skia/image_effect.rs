//! Source-pixel binary effects from the committed Skia branch.

use crate::model::image::ImageEffect;
use std::io::Cursor;

const ORDERED_DITHER_8X8: [u8; 64] = [
    0, 48, 12, 60, 3, 51, 15, 63, 32, 16, 44, 28, 35, 19, 47, 31, 8, 56, 4, 52, 11, 59, 7, 55, 40,
    24, 36, 20, 43, 27, 39, 23, 2, 50, 14, 62, 1, 49, 13, 61, 34, 18, 46, 30, 33, 17, 45, 29, 10,
    58, 6, 54, 9, 57, 5, 53, 42, 26, 38, 22, 41, 25, 37, 21,
];

pub(super) fn binary_effect_png(bytes: &[u8], effect: ImageEffect) -> Option<Vec<u8>> {
    if !matches!(effect, ImageEffect::BlackWhite | ImageEffect::Pattern8x8) {
        return None;
    }
    let mut reader = image::ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .ok()?;
    let mut limits = image::Limits::default();
    limits.max_alloc = Some(64 * 1024 * 1024);
    limits.max_image_width = Some(4096);
    limits.max_image_height = Some(4096);
    reader.limits(limits);
    let mut image = reader.decode().ok()?.to_rgba8();
    for (x, y, pixel) in image.enumerate_pixels_mut() {
        let luma = (f32::from(pixel[0]) * 0.299
            + f32::from(pixel[1]) * 0.587
            + f32::from(pixel[2]) * 0.114)
            .round() as u8;
        let white = if effect == ImageEffect::BlackWhite {
            luma >= 128
        } else {
            let cell = u16::from(ORDERED_DITHER_8X8[((y & 7) * 8 + (x & 7)) as usize]);
            luma > (((cell * 2 + 1) * 255) / 128) as u8
        };
        let value = if white { 255 } else { 0 };
        *pixel = image::Rgba([value, value, value, pixel[3]]);
    }
    let mut output = Cursor::new(Vec::new());
    image.write_to(&mut output, image::ImageFormat::Png).ok()?;
    Some(output.into_inner())
}

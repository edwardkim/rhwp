// Print 그림의 공통 장치 표본과 별도 색·마스크 출력 경로.

#[path = "print_image_sampling_core.rs"]
mod core;

pub(crate) use core::{rasterize, DeviceRaster};

/// encoded PNG의 straight RGB를 먼저 보존한 뒤 색·마스크를 독립 표본화한다.
/// Skia/Canvas 디코딩의 premultiplication과 결합 RGBA 필터를 거치지 않는다.
pub(crate) fn rasterize_masked(
    encoded: &[u8],
    rectangle: (f64, f64, f64, f64),
    page: (f64, f64),
    scale: f64,
) -> Option<DeviceRaster> {
    if !super::image_resolver::png_sampling::print_separate_alpha(encoded) {
        return None;
    }
    let mut colors =
        super::image_resolver::decode_image_with_format_limited(encoded, image::ImageFormat::Png)?
            .into_rgba8();
    let size = colors.dimensions();
    let mut mask = Vec::with_capacity(colors.as_raw().len());
    for pixel in colors.pixels_mut() {
        mask.extend_from_slice(&[pixel[3], pixel[3], pixel[3], 255]);
        pixel[3] = 255;
    }
    let mut colors = rasterize(colors.as_raw(), size, rectangle, page, scale)?;
    let mask = rasterize(&mask, size, rectangle, page, scale)?;
    for (color, alpha) in colors
        .rgba
        .chunks_exact_mut(4)
        .zip(mask.rgba.chunks_exact(4))
    {
        color[3] = alpha[0];
    }
    Some(colors)
}

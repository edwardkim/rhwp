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
    let colors =
        super::image_resolver::decode_image_with_format_limited(encoded, image::ImageFormat::Png)?
            .into_rgba8();
    if super::image_resolver::png_sampling::print_alpha_device_bands(encoded) {
        return rasterize_device_bands(&colors, rectangle, page, scale);
    }
    rasterize_planes(colors, rectangle, page, scale)
}

fn rasterize_planes(
    mut colors: image::RgbaImage,
    rectangle: (f64, f64, f64, f64),
    page: (f64, f64),
    scale: f64,
) -> Option<DeviceRaster> {
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

fn rasterize_device_bands(
    source: &image::RgbaImage,
    (x, y, width, height): (f64, f64, f64, f64),
    page: (f64, f64),
    scale: f64,
) -> Option<DeviceRaster> {
    if ![x, y, width, height].into_iter().all(f64::is_finite) || width <= 0.0 || height <= 0.0 {
        return None;
    }
    let edges = [
        x.floor(),
        y.floor(),
        (x + width).ceil(),
        (y + height).ceil(),
    ];
    if !edges
        .iter()
        .all(|v| (f64::from(i32::MIN)..=f64::from(i32::MAX)).contains(v))
    {
        return None;
    }
    let [left, top, right, bottom] = edges.map(|v| v as i32);
    let dw = u32::try_from(i64::from(right) - i64::from(left)).ok()?;
    let dh = u32::try_from(i64::from(bottom) - i64::from(top)).ok()?;
    if dw == 0 || dh == 0 || u64::from(dw) * u64::from(dh) > 4_194_304 {
        return None;
    }
    let (sw, sh) = source.dimensions();
    // Hancom divides the color surface into enough 32,000-pixel bands, then
    // rounds the common row count up. The last band gets the remaining rows.
    let count = (u64::from(sw) * u64::from(sh)).div_ceil(32_000);
    let band_height = u64::from(sh).div_ceil(count.max(1)) as u32;
    if band_height == 0 {
        return None;
    }
    let mut result = DeviceRaster {
        left,
        top,
        width: dw,
        height: dh,
        rgba: vec![0; (dw as usize) * (dh as usize) * 4],
    };
    for first in (0..sh).step_by(band_height as usize) {
        let rows = band_height.min(sh - first);
        let band = image::imageops::crop_imm(source, 0, first, sw, rows).to_image();
        if band.pixels().all(|pixel| pixel[3] == 0) {
            continue;
        }
        let band_y = y + height * f64::from(first) / f64::from(sh);
        let band_end = y + height * f64::from(first + rows) / f64::from(sh);
        let pass = rasterize_planes(band, (x, band_y, width, band_end - band_y), page, scale)?;
        let x0 = u32::try_from(pass.left.checked_sub(left)?).ok()?;
        let y0 = u32::try_from(pass.top.checked_sub(top)?).ok()?;
        if x0.checked_add(pass.width)? > dw || y0.checked_add(pass.height)? > dh {
            return None;
        }
        for (i, pixel) in pass.rgba.chunks_exact(4).enumerate() {
            let row = y0 as usize + i / pass.width as usize;
            let col = x0 as usize + i % pass.width as usize;
            let offset = (row * dw as usize + col) * 4;
            let out = &mut result.rgba[offset..offset + 4];
            let fg_alpha = u32::from(pixel[3]);
            let bg_alpha = u32::from(out[3]) * (255 - fg_alpha);
            let total_alpha = fg_alpha * 255 + bg_alpha;
            if total_alpha == 0 {
                continue;
            }
            for c in 0..3 {
                let mixed = u32::from(pixel[c]) * fg_alpha * 255 + u32::from(out[c]) * bg_alpha;
                out[c] = ((mixed + total_alpha / 2) / total_alpha) as u8;
            }
            out[3] = ((total_alpha + 127) / 255) as u8;
        }
    }
    Some(result)
}

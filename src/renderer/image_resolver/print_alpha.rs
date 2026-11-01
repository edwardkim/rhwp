//! 연속 알파 비트맵의 한컴 2020 Print 전달.
//!
//! 원본 픽셀의 정수 확대 격자와 전역 장치 격자를 구별한다. RGB 운반체의
//! 시작점은 전자에 올림하고, 알파 마스크는 원래 프레임에서 보간해 후자에
//! 디더링한다. 원본·조판 상자는 보존하며 두 출력 backend가 같은 PNG를 소비한다.

use std::io::Cursor;

use super::{decode_image_with_format_limited, memoized, Conversion};
use crate::model::{image::ImageEffect, style::ImageFillMode};
use crate::paint::{ResolvedImageKind, ResolvedImagePayload};
use crate::renderer::print_raster::PrintRasterFrame;
use crate::renderer::render_tree::{BoundingBox, ImageNode};

const MAX_PIXELS: u64 = 4_194_304;

pub(crate) fn print_alpha_payload(
    image: &ImageNode,
    source_frame: PrintRasterFrame,
    dpi: f64,
) -> Option<(BoundingBox, ResolvedImagePayload)> {
    if image.hancom2024_print_bitmap
        || image.effect != ImageEffect::RealPic
        || image.brightness != 0
        || image.contrast != 0
        || image.opacity != 1.0
        || image.content_inset.is_some()
        || image.transform.has_transform()
        || !matches!(
            image.fill_mode,
            None | Some(ImageFillMode::FitToSize | ImageFillMode::Total)
        )
        || !dpi.is_finite()
        || dpi <= 0.0
    {
        return None;
    }
    let data = image.data.as_deref()?;
    if !data.starts_with(b"\x89PNG\r\n\x1a\n") || super::png_sampling::pdf_source(data) {
        return None;
    }
    let header = crate::renderer::image_header::canvaskit_encoded_image_header(data)
        .filter(|header| header.is_within_decode_limits())?;
    if let Some(crop) = image.crop {
        let source = crate::renderer::svg::compute_image_crop_src(
            crop,
            super::image_crop_reference(image),
            f64::from(header.width),
            f64::from(header.height),
        );
        if source != (0.0, 0.0, f64::from(header.width), f64::from(header.height)) {
            return None;
        }
    }
    let values = source_frame.values();
    let [pw, ph, l, t, mw, mh, r, b] = values;
    // 축소·회전·부분 crop은 서로 다른 인쇄 전달 계약이다.
    if mw < f64::from(header.width) || mh < f64::from(header.height) {
        return None;
    }
    let step_x = (mw / f64::from(header.width)).round();
    let step_y = (mh / f64::from(header.height)).round();
    let left = (l / step_x).ceil() * step_x;
    let top = (t / step_y).ceil() * step_y;
    let columns = ((r - left) / step_x).ceil().min(f64::from(header.width));
    let rows = ((b - top) / step_y).ceil().min(f64::from(header.height));
    let width = columns * step_x;
    let height = rows * step_y;
    if width <= 0.0 || height <= 0.0 || width * height > MAX_PIXELS as f64 {
        return None;
    }
    // 반올림한 정수 표본 간격으로 원본 전체를 펼쳐도 저장 출력 프레임을
    // 덮지 못하면 마지막 행이나 열이 잘린다. 예를 들어 86행을 268 장치
    // 픽셀에 놓을 때 간격을 3으로 반올림하면 258픽셀만 생성된다.
    // 프레임을 다 채우지 못하는 경우 기존 이미지 샘플링 경로를 사용한다.
    if left + width < r - 0.5 || top + height < b - 0.5 {
        return None;
    }
    let output = memoized(
        Conversion::PrintAlpha {
            frame: values.map(f64::to_bits),
        },
        data,
        || {
            let source =
                decode_image_with_format_limited(data, image::ImageFormat::Png)?.to_rgba8();
            // 이진 투명도는 한컴이 RGB 행 조각으로 출력한다. 연속 알파와
            // 같은 마스크 경로로 추정하지 않고 기존 전달을 유지한다.
            if !source.pixels().any(|pixel| pixel[3] > 0 && pixel[3] < 255) {
                return None;
            }
            let (width, height) = (width as u32, height as u32);
            let xs = samples(left - l, mw, source.width(), width);
            let ys = samples(top - t, mh, source.height(), height);
            let colors = carrier_colors(
                &source,
                columns as u32,
                rows as u32,
                left - l,
                top - t,
                mw,
                mh,
                step_x,
                step_y,
            );
            let mut output = image::RgbaImage::new(width, height);
            for (y, samples_y) in ys.iter().enumerate() {
                for (x, samples_x) in xs.iter().enumerate() {
                    let px = left + x as f64;
                    let py = top + y as f64;
                    if px >= r || py >= b {
                        continue;
                    }
                    let mut alpha = 0.0;
                    for &(sy, wy) in samples_y {
                        for &(sx, wx) in samples_x {
                            alpha += f64::from(source.get_pixel(sx, sy)[3]) * wx * wy;
                        }
                    }
                    let alpha = alpha.round().clamp(0.0, 255.0) as u8;
                    if alpha != 255 && (alpha == 0 || alpha <= 255 - bayer(px as i32, py as i32)) {
                        continue;
                    }
                    let color =
                        colors.get_pixel(x as u32 / step_x as u32, y as u32 / step_y as u32);
                    output.put_pixel(
                        x as u32,
                        y as u32,
                        image::Rgba([color[0], color[1], color[2], 255]),
                    );
                }
            }
            let mut bytes = Vec::new();
            image::DynamicImage::ImageRgba8(output)
                .write_to(&mut Cursor::new(&mut bytes), image::ImageFormat::Png)
                .ok()?;
            Some(bytes)
        },
    )?;
    let sx = (pw / 100.0 + 1e-9).floor() / (pw / 12.0).round() * dpi / 72.0;
    let sy = (ph / 100.0 + 1e-9).floor() / (ph / 12.0).round() * dpi / 72.0;
    Some((
        BoundingBox::new(left * sx, top * sy, width * sx, height * sy),
        ResolvedImagePayload {
            data: output,
            mime: "image/png",
            kind: ResolvedImageKind::OutputDevice,
            suppress_effects: false,
        },
    ))
}

fn samples(origin: f64, extent: f64, source: u32, count: u32) -> Vec<[(u32, f64); 4]> {
    (0..count)
        .map(|i| {
            // 600dpi 마스크의 픽셀 중심을 원래 그림 프레임으로 역투영한다.
            let position = (origin + f64::from(i) + 0.5) * f64::from(source) / extent;
            let base = position.floor();
            std::array::from_fn(|j| {
                let index = base + j as f64 - 1.0;
                (
                    index.clamp(0.0, f64::from(source - 1)) as u32,
                    crate::renderer::cubic_sampling::weight(position - index),
                )
            })
        })
        .collect()
}

// 인쇄 DIB의 아래에서 위로 가는 행 순서와 전역 16×16 ordered threshold.
// 원본/운반체의 크기마다 디더 원점을 재설정하지 않는다.
fn bayer(x: i32, y: i32) -> u8 {
    let mut x = x.rem_euclid(16) as u8;
    let mut y = (-y - 2).rem_euclid(16) as u8;
    let mut rank = 0;
    for _ in 0..4 {
        rank = rank * 4 + [[0, 2], [3, 1]][(y & 1) as usize][(x & 1) as usize];
        x >>= 1;
        y >>= 1;
    }
    rank
}

#[allow(clippy::too_many_arguments)]
fn carrier_colors(
    source: &image::RgbaImage,
    width: u32,
    height: u32,
    left: f64,
    top: f64,
    mw: f64,
    mh: f64,
    step_x: f64,
    step_y: f64,
) -> image::RgbImage {
    let mut colors = image::RgbImage::new(width, height);
    for y in 0..height {
        for x in 0..width {
            let u = (left + f64::from(x) * step_x) * f64::from(source.width()) / mw;
            let v = (top + f64::from(y) * step_y) * f64::from(source.height()) / mh;
            let mut rgb = [0.0; 3];
            for j in 0..2 {
                for i in 0..2 {
                    let sx = (u.floor() as u32 + i).min(source.width() - 1);
                    let sy = (v.floor() as u32 + j).min(source.height() - 1);
                    let p = source.get_pixel(sx, sy);
                    let weight = (1.0 - (u - u.floor() - f64::from(i)).abs())
                        * (1.0 - (v - v.floor() - f64::from(j)).abs());
                    for c in 0..3 {
                        rgb[c] += f64::from(if p[3] == 0 { 255 } else { p[c] }) * weight;
                    }
                }
            }
            colors.put_pixel(
                x,
                y,
                image::Rgb(rgb.map(|v| v.round().clamp(0.0, 255.0) as u8)),
            );
        }
    }
    colors
}

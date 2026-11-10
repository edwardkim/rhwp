//! 그림 효과의 alpha 준비. 모델·조판은 보존하고 공통 paint의 복제본만 준비한다.
//!
//! 독립 단색·crop·투명 PNG Print 대조군은 crop 뒤의 alpha 침식과 분리 가능한
//! Gaussian 감쇠를 지지한다. RGB·전역 opacity는 이 연산에 섞지 않는다.

use std::collections::VecDeque;
use std::io::Cursor;

use super::{decode_image_with_format_limited, memoized, Conversion};
use crate::paint::{ResolvedImageKind, ResolvedImagePayload};
use crate::renderer::render_tree::{BoundingBox, ImageNode};

const WORKING_DPI: f64 = 96.0;
// 독립 한컴 단색 반경 대조군의 작업 표본 단위. 실제 사양의 HU 반경은 유지한다.
const RADIUS_HU_PER_SAMPLE: f64 = 120.0;

pub(crate) fn prepare_picture_soft_edge(
    image: &mut ImageNode,
    frame: BoundingBox,
    logical_frame: BoundingBox,
    dpi: f64,
    page_aligned: bool,
) -> Option<ResolvedImagePayload> {
    let radius = image
        .soft_edge_radius_hu
        .filter(|r| r.is_finite() && *r > 0.0)?;
    if !dpi.is_finite() || dpi <= 0.0 || image.fill_mode.is_some() {
        return None;
    }
    let source = image.data.as_deref()?;
    let format = image::guess_format(source).ok()?;
    let header =
        super::canvaskit_encoded_image_header(source).filter(|h| h.is_within_decode_limits())?;
    // 2024 PNG 효과는 RGB와 1bit 마스크를 각각 장치에 보낸다. JPEG 압축·
    // 묶음/회전의 별도 계약을 이 근거만으로 바꾸지 않는다.
    let print_planes = page_aligned
        && image.hancom2024_print_bitmap
        && format == image::ImageFormat::Png
        && image.effect == crate::model::image::ImageEffect::RealPic
        && image.brightness == 0
        && image.contrast == 0
        && image.opacity == 1.0
        && !image.transform.has_transform()
        && image.source_rect.is_none();
    let bbox = image.transformed_paint_bbox(&if print_planes { logical_frame } else { frame });
    let size = [bbox.width, bbox.height].map(|v| v * WORKING_DPI / dpi);
    if size.iter().any(|v| !v.is_finite() || *v <= 0.0) {
        return None;
    }
    let crop = image.crop.map_or(
        (0.0, 0.0, f64::from(header.width), f64::from(header.height)),
        |c| {
            crate::renderer::svg::compute_image_crop_src(
                c,
                super::image_crop_reference(image),
                f64::from(header.width),
                f64::from(header.height),
            )
        },
    );
    if [crop.0, crop.1, crop.2, crop.3]
        .iter()
        .any(|v| !v.is_finite())
        || crop.2 <= 0.0
        || crop.3 <= 0.0
    {
        return None;
    }
    // 효과 커널은 장치 확대와 분리한다. 큰 효과도 GDI+의 최대 작업 반경 안에서
    // 같은 물리 크기로 표본화하며 기존 이미지 디코드 한도를 늘리지 않는다.
    let scale = (255.0 / (radius / RADIUS_HU_PER_SAMPLE)).min(1.0);
    let width = (size[0] * scale).round().max(1.0) as u32;
    let height = (size[1] * scale).round().max(1.0) as u32;
    // 색 자원은 효과용 작업 표면으로 축소하지 않는다. alpha만 작업 격자에서
    // 계산하고, 원본 가시 창 이상의 색 해상도로 되돌려 사진 세부를 유지한다.
    let color_width = if print_planes {
        (size[0] * 600.0 / WORKING_DPI).round()
    } else {
        f64::from(width).max(crop.2.ceil())
    } as u32;
    let color_height = if print_planes {
        (size[1] * 600.0 / WORKING_DPI).round()
    } else {
        f64::from(height).max(crop.3.ceil())
    } as u32;
    if width > super::CANVASKIT_MAX_IMAGE_DIMENSION
        || height > super::CANVASKIT_MAX_IMAGE_DIMENSION
        || u64::from(width) * u64::from(height) > super::CANVASKIT_MAX_IMAGE_PIXELS
        || color_width > super::CANVASKIT_MAX_IMAGE_DIMENSION
        || color_height > super::CANVASKIT_MAX_IMAGE_DIMENSION
        || color_width == 0
        || color_height == 0
        || u64::from(color_width) * u64::from(color_height) > super::CANVASKIT_MAX_IMAGE_PIXELS
    {
        return None;
    }
    let key = Conversion::PictureSoftEdge {
        radius: radius.to_bits(),
        width,
        height,
        color_width,
        color_height,
        print_planes,
        crop: [crop.0, crop.1, crop.2, crop.3].map(f64::to_bits),
    };
    let bytes = memoized(key, source, || {
        let source = decode_image_with_format_limited(source, format)?.to_rgba8();
        let working = sample_window(&source, crop, width, height);
        let r = (radius / RADIUS_HU_PER_SAMPLE * scale).round().max(1.0) as usize;
        let alpha: Vec<f64> = working.pixels().map(|p| f64::from(p[3])).collect();
        let mask = feather(&alpha, width as usize, height as usize, r);
        let mask = image::GrayImage::from_raw(
            width,
            height,
            mask.into_iter()
                .map(|a| a.round().clamp(0.0, 255.0) as u8)
                .collect(),
        )?;
        let mask = image::imageops::resize(
            &mask,
            color_width,
            color_height,
            image::imageops::FilterType::Triangle,
        );
        let mut pixels = sample_window(&source, crop, color_width, color_height);
        if print_planes {
            return super::print_alpha_2024::encode_effect_planes(pixels, &mask);
        }
        for (pixel, alpha) in pixels.pixels_mut().zip(mask.pixels()) {
            // 재표본이 내부의 완전 투명 영역을 다시 채우지 않는다.
            pixel[3] = pixel[3].min(alpha[0]);
        }
        let mut bytes = Cursor::new(Vec::new());
        pixels.write_to(&mut bytes, image::ImageFormat::Png).ok()?;
        Some(bytes.into_inner())
    })?;
    image.data = Some(bytes.clone());
    image.crop = None;
    image.original_size_hu = None;
    Some(ResolvedImagePayload {
        data: bytes,
        mime: "image/png",
        kind: if print_planes {
            ResolvedImageKind::OutputDeviceWindow
        } else {
            ResolvedImageKind::SourceWindow
        },
        suppress_effects: false,
    })
}

fn sample_window(
    source: &image::RgbaImage,
    crop: (f64, f64, f64, f64),
    width: u32,
    height: u32,
) -> image::RgbaImage {
    image::RgbaImage::from_fn(width, height, |x, y| {
        let sx = crop.0 + (f64::from(x) + 0.5) * crop.2 / f64::from(width) - 0.5;
        let sy = crop.1 + (f64::from(y) + 0.5) * crop.3 / f64::from(height) - 0.5;
        let ix = sx.floor();
        let iy = sy.floor();
        let fx = sx - ix;
        let fy = sy - iy;
        let mut result = [0.0; 4];
        for (dy, wy) in [(0.0, 1.0 - fy), (1.0, fy)] {
            for (dx, wx) in [(0.0, 1.0 - fx), (1.0, fx)] {
                let px = (ix + dx).clamp(0.0, f64::from(source.width() - 1)) as u32;
                let py = (iy + dy).clamp(0.0, f64::from(source.height() - 1)) as u32;
                for (out, channel) in result.iter_mut().zip(source.get_pixel(px, py).0) {
                    *out += f64::from(channel) * wx * wy;
                }
            }
        }
        image::Rgba(result.map(|c| c.round().clamp(0.0, 255.0) as u8))
    })
}

// 외부는 투명하다. 각 이웃의 침식 창은 원 표본을 포함하므로 Gaussian 합성 뒤에도
// 원 alpha보다 불투명해지지 않는다. alpha를 다시 곱하면 반투명 중앙이 제곱된다.
fn feather(alpha: &[f64], width: usize, height: usize, radius: usize) -> Vec<f64> {
    if radius * 2 >= width.min(height) {
        return vec![0.0; alpha.len()];
    }
    let mut horizontal = vec![0.0; alpha.len()];
    for y in 0..height {
        erode_line(
            &alpha[y * width..(y + 1) * width],
            &mut horizontal[y * width..(y + 1) * width],
            radius,
        );
    }
    let mut eroded = vec![0.0; alpha.len()];
    let mut input = vec![0.0; height];
    let mut output = input.clone();
    for x in 0..width {
        for y in 0..height {
            input[y] = horizontal[y * width + x];
        }
        erode_line(&input, &mut output, radius);
        for y in 0..height {
            eroded[y * width + x] = output[y];
        }
    }
    let sigma = radius as f64 / 2.0;
    let mut kernel: Vec<f64> = (0..=radius * 2)
        .map(|i| {
            let x = i as f64 - radius as f64;
            (-x * x / (2.0 * sigma * sigma)).exp()
        })
        .collect();
    let sum: f64 = kernel.iter().sum();
    for k in &mut kernel {
        *k /= sum;
    }
    horizontal.fill(0.0);
    for y in 0..height {
        for x in 0..width {
            for (i, k) in kernel.iter().enumerate() {
                let sx = x as isize + i as isize - radius as isize;
                if sx >= 0 && sx < width as isize {
                    horizontal[y * width + x] += eroded[y * width + sx as usize] * k;
                }
            }
        }
    }
    let mut result = vec![0.0; alpha.len()];
    for y in 0..height {
        for x in 0..width {
            for (i, k) in kernel.iter().enumerate() {
                let sy = y as isize + i as isize - radius as isize;
                if sy >= 0 && sy < height as isize {
                    result[y * width + x] += horizontal[sy as usize * width + x] * k;
                }
            }
        }
    }
    result
}

fn erode_line(input: &[f64], output: &mut [f64], radius: usize) {
    let mut queue: VecDeque<usize> = VecDeque::new();
    for end in 0..input.len() {
        while queue.back().is_some_and(|&i| input[i] >= input[end]) {
            queue.pop_back();
        }
        queue.push_back(end);
        if end >= radius * 2 {
            let start = end - radius * 2;
            while queue.front().is_some_and(|&i| i < start) {
                queue.pop_front();
            }
            output[end - radius] = input[*queue.front().unwrap()];
        }
    }
}

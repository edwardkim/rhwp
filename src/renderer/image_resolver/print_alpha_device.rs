//! 장치의 색·알파 격자가 같은 Print 그림의 공통 전달.

use std::io::Cursor;

use super::{memoized, print_alpha_core::bayer_rank, Conversion};
use crate::paint::{ResolvedImageKind, ResolvedImagePayload};
use crate::renderer::render_tree::{BoundingBox, ImageNode};

const MAX_DEVICE_PIXELS: u64 = 4_194_304;

/// 불투명 그림은 순번을 소비하지 않으며, 이진 투명 PNG도 순번에 포함된다.
pub(crate) fn print_transparent_png(image: &ImageNode) -> bool {
    let Some(data) = image.data.as_deref().filter(|data| {
        data.starts_with(b"\x89PNG\r\n\x1a\n") && !super::png_sampling::pdf_source(data)
    }) else {
        return false;
    };
    png_may_have_alpha(data)
        && memoized(Conversion::PrintTransparentPng, data, || {
            let rgba = super::decode_image_with_format_limited(data, image::ImageFormat::Png)?
                .into_rgba8();
            rgba.pixels().any(|pixel| pixel[3] != 255).then(Vec::new)
        })
        .is_some()
}

fn png_may_have_alpha(data: &[u8]) -> bool {
    if matches!(data.get(25), Some(4 | 6)) {
        return true;
    }
    let mut chunks = data.get(33..).unwrap_or_default();
    while let Some(header) = chunks.get(..8) {
        match &header[4..] {
            b"tRNS" => return true,
            b"IDAT" | b"IEND" => break,
            _ => {}
        }
        let len = u32::from_be_bytes([header[0], header[1], header[2], header[3]]) as usize;
        let Some(remaining) = len.checked_add(12).and_then(|n| chunks.get(n..)) else {
            break;
        };
        chunks = remaining;
    }
    false
}

/// 마스크의 한 셀은 출력 DPI가 아닌 600dpi Print 장치의 한 픽셀이다.
#[derive(Clone, Copy, Hash)]
pub(crate) struct PrintAlphaDeviceGrid {
    size: (u32, u32),
    phase: (u32, u32),
}

impl PrintAlphaDeviceGrid {
    pub(crate) fn new(frame: BoundingBox, device_unit: (f64, f64)) -> Option<Self> {
        let (sx, sy) = device_unit;
        let [x, y, width, height] = [
            frame.x / sx,
            frame.y / sy,
            frame.width / sx,
            frame.height / sy,
        ]
        .map(f64::round);
        let limit = crate::renderer::image_header::CANVASKIT_MAX_IMAGE_DIMENSION;
        if ![x, y, width, height].into_iter().all(f64::is_finite)
            || !(1.0..=f64::from(limit)).contains(&width)
            || !(1.0..=f64::from(limit)).contains(&height)
            || width * height > MAX_DEVICE_PIXELS as f64
        {
            return None;
        }
        Some(Self {
            size: (width as u32, height as u32),
            phase: (x.rem_euclid(16.0) as u32, y.rem_euclid(16.0) as u32),
        })
    }
}

impl PrintAlphaDeviceGrid {
    pub(crate) fn payload(
        mut self,
        image: &ImageNode,
        alpha_order: u8,
    ) -> Option<ResolvedImagePayload> {
        if !super::print_alpha_2024::eligible_picture(image)
            || image.content_inset.is_some()
            || image.source_rect.is_some()
        {
            return None;
        }
        let source = image.data.as_deref()?;
        let header = super::canvaskit_encoded_image_header(source)?;
        if !header.is_within_decode_limits() {
            return None;
        }
        // 축별 원본 간격이 장치 한 픽셀로 반올림되고 두 축 모두 500dpi
        // 자원 재표본화 경계 안에 있으면 색과 마스크의 장치 격자가 같다.
        let unit_step = |source: u32, device: u32| {
            let (source, device) = (u64::from(source), u64::from(device));
            source < device && 2 * device < 3 * source && 6 * source <= 5 * device
        };
        if !unit_step(header.width, self.size.0) || !unit_step(header.height, self.size.1) {
            return None;
        }
        self.phase.1 = (self.phase.1 + u32::from(alpha_order)) % 16;
        let data = memoized(Conversion::PrintAlphaDevice { grid: self }, source, || {
            let rgba = super::decode_image_with_format_limited(source, image::ImageFormat::Png)?
                .into_rgba8();
            if !rgba.pixels().any(|pixel| (1..255).contains(&pixel[3])) {
                return None;
            }
            let mut scaled = super::print_prefilter::resize_alpha(rgba, self.size)?;
            for (x, y, pixel) in scaled.enumerate_pixels_mut() {
                let alpha = pixel[3];
                let opaque = alpha == 255 || alpha > bayer_rank(x + self.phase.0, y + self.phase.1);
                pixel[3] = if opaque { 255 } else { 0 };
            }
            let mut bytes = Cursor::new(Vec::new());
            scaled.write_to(&mut bytes, image::ImageFormat::Png).ok()?;
            let mut bytes = bytes.into_inner();
            super::png_sampling::mark_print_alpha_device_bands(&mut bytes);
            Some(bytes)
        })?;
        Some(ResolvedImagePayload {
            data,
            mime: "image/png",
            kind: ResolvedImageKind::OutputDevice,
            suppress_effects: false,
        })
    }
}

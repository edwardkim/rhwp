//! Print 그림의 공통 장치 표본. 조판 좌표와 원본 픽셀은 변경하지 않는다.
//!
//! 공개 PDFium chromium/7999 CStretchEngine의 16.16 면적 계약과 독립 Print를
//! 대조했다. 확대는 같은 공개 경로의 표본 중심 보간을 소비한다. 각 축에서 채널을
//! 절삭하며, 한 번에 2차원 평균을 내는 식과 다르다.
//
// Arithmetic adapted from PDFium CStretchEngine (BSD-style license).
// Copyright 2017 The PDFium Authors.
// Original code copyright 2014 Foxit Software Inc. http://www.foxitsoftware.com
// The source license is preserved in licenses/pdfium/LICENSE.

const ONE: u32 = 1 << 16;
const MAX_PIXELS: usize = 32 * 1024 * 1024;
const MAX_DIMENSION: u32 = 8192;

pub(crate) struct DeviceRaster {
    pub left: i32,
    pub top: i32,
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

struct Weight {
    first: usize,
    values: Vec<u32>,
}

fn weights(source: u32, destination: u32, bilinear: bool) -> Option<Vec<Weight>> {
    if source == 0 || destination == 0 {
        return None;
    }
    let scale = f64::from(source) / f64::from(destination);
    let mut result = Vec::with_capacity(destination as usize);
    for pixel in 0..destination {
        if scale < 1.0 {
            let center = (f64::from(pixel) + 0.5) * scale;
            if bilinear {
                let first = ((center - 0.5).floor() as i64).max(0) as usize;
                let last = ((center + 0.5).floor() as usize).min(source as usize - 1);
                let values = if first >= last {
                    vec![ONE]
                } else {
                    let second = ((center - first as f64 - 0.5) * f64::from(ONE)).round() as u32;
                    vec![ONE - second, second]
                };
                result.push(Weight { first, values });
            } else {
                result.push(Weight {
                    first: (center.floor() as usize).min(source as usize - 1),
                    values: vec![ONE],
                });
            }
            continue;
        }
        let start = f64::from(pixel) * scale;
        let end = start + scale;
        let first = start.floor() as usize;
        let last = (end.floor() as usize).min(source as usize - 1);
        let mut remaining = ONE;
        let mut error = 0.0;
        let mut values = Vec::with_capacity(last - first + 1);
        for position in first..last {
            let left = (position as f64 / scale).max(f64::from(pixel));
            let right = ((position + 1) as f64 / scale).min(f64::from(pixel + 1));
            let area = (right - left).max(0.0);
            let fixed = ((area + error) * f64::from(ONE)).round() as u32;
            values.push(fixed);
            remaining = remaining.checked_sub(fixed)?;
            error = area - f64::from(fixed) / f64::from(ONE);
        }
        values.push(remaining);
        result.push(Weight { first, values });
    }
    Some(result)
}

/// 이미 최종 장치 변환을 적용한 사각형을 실제 페이지 bitmap 격자에 대응시킨다.
/// 출력 버퍼의 크기 올림은 물리 배율이 아니다. 변환된 좌표를 다시 늘리지 않는다.
/// 부분 창 재생에서도 같은 장치 격자를 유지한다.
pub(crate) fn rasterize(
    rgba: &[u8],
    source: (u32, u32),
    rectangle: (f64, f64, f64, f64),
    page: (f64, f64),
    scale: f64,
) -> Option<DeviceRaster> {
    let (x, y, width, height) = rectangle;
    if ![x, y, width, height, page.0, page.1, scale]
        .into_iter()
        .all(f64::is_finite)
        || [width, height, page.0, page.1, scale]
            .into_iter()
            .any(|v| v <= 0.0)
    {
        return None;
    }
    let edges = [
        x.floor(),
        y.floor(),
        (x + width).ceil(),
        (y + height).ceil(),
    ];
    if edges
        .iter()
        .any(|v| !v.is_finite() || *v < f64::from(i32::MIN) || *v > f64::from(i32::MAX))
    {
        return None;
    }
    let dw = u32::try_from(edges[2] as i64 - edges[0] as i64).ok()?;
    let dh = u32::try_from(edges[3] as i64 - edges[1] as i64).ok()?;
    Some(DeviceRaster {
        left: edges[0] as i32,
        top: edges[1] as i32,
        width: dw,
        height: dh,
        rgba: resample_opaque_rgba(rgba, source, (dw, dh))?,
    })
}

/// 불투명 전체 원본의 장치 표본. alpha·crop·회전은 별도 계약이다.
pub(crate) fn resample_opaque_rgba(
    source: &[u8],
    (width, height): (u32, u32),
    (dw, dh): (u32, u32),
) -> Option<Vec<u8>> {
    let (w, h, dx, dy) = (width as usize, height as usize, dw as usize, dh as usize);
    let pixels = w.checked_mul(h)?;
    if width == 0
        || height == 0
        || dw == 0
        || dh == 0
        || width > MAX_DIMENSION
        || height > MAX_DIMENSION
        || dw > MAX_DIMENSION
        || dh > MAX_DIMENSION
        || pixels > MAX_PIXELS
        || dx.checked_mul(h)? > MAX_PIXELS
        || dx.checked_mul(dy)? > MAX_PIXELS
        || pixels.checked_mul(4)? != source.len()
        || source.chunks_exact(4).any(|p| p[3] != 255)
    {
        return None;
    }
    // 공개 CStretchEngine의 기본 smoothing 선택이다. 각 축의 확대 가중치는
    // 표본 중심을 사용하고 축소는 기존 면적 가중치를 그대로 소비한다.
    let bilinear = u64::from(dh) / 8 < u64::from(width) * u64::from(height) / u64::from(dw);
    let horizontal = weights(width, dw, bilinear)?;
    let vertical = weights(height, dh, bilinear)?;
    let mut intermediate = vec![0u8; dx.checked_mul(h)?.checked_mul(3)?];
    for y in 0..h {
        for (x, weight) in horizontal.iter().enumerate() {
            for channel in 0..3 {
                let mut sum = 0u32;
                for (offset, value) in weight.values.iter().enumerate() {
                    sum += u32::from(source[(y * w + weight.first + offset) * 4 + channel]) * value;
                }
                intermediate[(y * dx + x) * 3 + channel] = (sum >> 16) as u8;
            }
        }
    }
    let mut output = vec![255u8; dx.checked_mul(dy)?.checked_mul(4)?];
    for (y, weight) in vertical.iter().enumerate() {
        for x in 0..dx {
            for channel in 0..3 {
                let mut sum = 0u32;
                for (offset, value) in weight.values.iter().enumerate() {
                    sum +=
                        u32::from(intermediate[((weight.first + offset) * dx + x) * 3 + channel])
                            * value;
                }
                output[(y * dx + x) * 4 + channel] = (sum >> 16) as u8;
            }
        }
    }
    Some(output)
}

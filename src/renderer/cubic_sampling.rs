//! 장치 좌표에서 한 레벨 이내로 축소하는 그림의 재구성 계약.
//!
//! 이 범위에서 선형 필터의 두 표본은 얇은 선을 흰 배경에 희석한다.
//! Native Skia와 CanvasKit은 Catmull–Rom을 직접 사용하고 Canvas2D는
//! 같은 네 표본 식을 장치 격자에 계산한다. 더 큰 축소는 기존 저역 통과를 쓴다.
//! 호출자는 실제 한컴 2024 색 확장 payload 표식을 확인한 뒤 이 경로를 선택한다.

pub(crate) fn modest_reduction(source: (f64, f64), device: (f64, f64)) -> bool {
    [source.0, source.1, device.0, device.1]
        .iter()
        .all(|v| v.is_finite() && *v > 0.0)
        && device.0 <= source.0
        && device.1 <= source.1
        && (device.0 < source.0 || device.1 < source.1)
        && source.0 <= 2.0 * device.0
        && source.1 <= 2.0 * device.1
}

#[cfg(target_arch = "wasm32")]
pub(crate) struct DeviceRaster {
    pub rgba: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

#[cfg(target_arch = "wasm32")]
fn weight(distance: f64) -> f64 {
    let d = distance.abs();
    if d < 1.0 {
        ((1.5 * d - 2.5) * d) * d + 1.0
    } else if d < 2.0 {
        ((-0.5 * d + 2.5) * d - 4.0) * d + 2.0
    } else {
        0.0
    }
}

#[cfg(target_arch = "wasm32")]
fn axis_samples(
    origin: f64,
    extent: f64,
    source: u32,
    start: f64,
    count: u32,
) -> Vec<[(usize, f64); 4]> {
    (0..count)
        .map(|i| {
            let position = (start + f64::from(i) + 0.5 - origin) * f64::from(source) / extent - 0.5;
            let base = position.floor();
            std::array::from_fn(|j| {
                let index = base + j as f64 - 1.0;
                (
                    index.clamp(0.0, f64::from(source - 1)) as usize,
                    weight(position - index),
                )
            })
        })
        .collect()
}

/// 사각형·원점은 배치 좌표가 아니라 현재 CTM을 적용한 장치 좌표다.
/// 알파를 먼저 곱해 보간하므로 투명 픽셀의 숨은 RGB가 가장자리에 새지 않는다.
#[cfg(target_arch = "wasm32")]
pub(crate) fn rasterize(
    rgba: &[u8],
    source: (u32, u32),
    destination: (f64, f64, f64, f64),
) -> Option<DeviceRaster> {
    let (x, y, w, h) = destination;
    if !x.is_finite()
        || !y.is_finite()
        || !modest_reduction((f64::from(source.0), f64::from(source.1)), (w, h))
        || rgba.len()
            != (source.0 as usize)
                .checked_mul(source.1 as usize)?
                .checked_mul(4)?
    {
        return None;
    }
    let (left, top) = (x.floor(), y.floor());
    let width = (x + w).ceil() - left;
    let height = (y + h).ceil() - top;
    // 브라우저 보조 표본 한 장도 기존 변환 메모의 16 MiB 예산 안에 둔다.
    if width * height > 4_194_304.0 || width <= 0.0 || height <= 0.0 {
        return None;
    }
    let (width, height) = (width as u32, height as u32);
    let xs = axis_samples(x, w, source.0, left, width);
    let ys = axis_samples(y, h, source.1, top, height);
    let mut output = vec![0u8; width as usize * height as usize * 4];
    for (dy, samples_y) in ys.iter().enumerate() {
        let py = top + dy as f64;
        let coverage_y = (py + 1.0).min(y + h) - py.max(y);
        for (dx, samples_x) in xs.iter().enumerate() {
            let mut color = [0.0; 4];
            for &(sy, wy) in samples_y {
                for &(sx, wx) in samples_x {
                    let pixel = &rgba[(sy * source.0 as usize + sx) * 4..][..4];
                    let alpha = f64::from(pixel[3]) / 255.0;
                    for (channel, &value) in color[..3].iter_mut().zip(&pixel[..3]) {
                        *channel += f64::from(value) * alpha * wx * wy;
                    }
                    color[3] += alpha * wx * wy;
                }
            }
            let alpha = color[3].clamp(0.0, 1.0);
            let pixel = &mut output[(dy * width as usize + dx) * 4..][..4];
            if alpha > 0.0 {
                for (channel, &value) in pixel[..3].iter_mut().zip(&color[..3]) {
                    *channel = (value / alpha).round().clamp(0.0, 255.0) as u8;
                }
            }
            let px = left + dx as f64;
            let coverage_x = (px + 1.0).min(x + w) - px.max(x);
            pixel[3] = (alpha * coverage_x * coverage_y * 255.0).round() as u8;
        }
    }
    Some(DeviceRaster {
        rgba: output,
        width,
        height,
    })
}

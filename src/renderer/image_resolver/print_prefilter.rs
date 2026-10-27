//! Print 그림 자원의 면적 적분 cubic 재표본화.
//!
//! 원본 중심은 정수 격자이며 영상 밖은 투명하다. 축소 시 지지 구간을 넓힌다.
//! Keys cubic(a=-1)의 누적 적분을 계산하며 문서별 계수나 이진 테이블을 쓰지 않는다.

use crate::renderer::image_header::{CANVASKIT_MAX_IMAGE_DIMENSION, CANVASKIT_MAX_IMAGE_PIXELS};

const ONE: i64 = 1 << 16;

fn cumulative(x: f64) -> f64 {
    if x <= -2.0 {
        return 0.0;
    }
    if x >= 2.0 {
        return 1.0;
    }
    if x < 0.0 {
        return 1.0 - cumulative(-x);
    }
    if x <= 1.0 {
        return 0.5 + x - 2.0 * x.powi(3) / 3.0 + x.powi(4) / 4.0;
    }
    // [1,2] 구간은 -x³+5x²-8x+4이며 F(1)=13/12다.
    let primitive = |x: f64| -x.powi(4) / 4.0 + 5.0 * x.powi(3) / 3.0 - 4.0 * x * x + 4.0 * x;
    13.0 / 12.0 + primitive(x) - primitive(1.0)
}

fn integrated_weight(x: f64) -> i64 {
    if x <= -2.0 {
        return 0;
    }
    if x >= 2.0 {
        return ONE;
    }
    // 누적 적분의 1/128 셀 중점을 16.16으로 양자화한다.
    let midpoint = ((x * 128.0).floor() + 0.5) / 128.0;
    (cumulative(midpoint) * ONE as f64).round() as i64
}

fn weights(source: u32, destination: u32) -> Vec<Vec<(usize, i64)>> {
    let step = f64::from(source) / f64::from(destination);
    let support = step.max(1.0);
    (0..destination)
        .map(|pixel| {
            let centre = f64::from(pixel) * step;
            // 범위 밖의 표본은 투명하다. 남은 계수 합을 다시 1로 정규화하지 않는다.
            let first = ((centre - 2.0 * support - 0.5).ceil() as i64).max(0);
            let last = ((centre + 2.0 * support + 0.5).floor() as i64).min(i64::from(source) - 1);
            (first..=last)
                .filter_map(|index| {
                    let start = (index as f64 - 0.5 - centre) / support;
                    let end = (index as f64 + 0.5 - centre) / support;
                    let weight = integrated_weight(end) - integrated_weight(start);
                    (weight != 0).then_some((index as usize, weight))
                })
                .collect()
        })
        .collect()
}

fn length(width: u32, height: u32) -> Option<usize> {
    (width > 0
        && height > 0
        && width <= CANVASKIT_MAX_IMAGE_DIMENSION
        && height <= CANVASKIT_MAX_IMAGE_DIMENSION
        && u64::from(width) * u64::from(height) <= CANVASKIT_MAX_IMAGE_PIXELS)
        .then(|| {
            (width as usize)
                .checked_mul(height as usize)?
                .checked_mul(4)
        })
        .flatten()
}

fn channel(sum: i64) -> u8 {
    ((sum + ONE / 2) >> 16).clamp(0, 255) as u8
}

/// 불투명 원본의 RGB와 투명 경계의 coverage를 함께 생성한다.
/// 원본 alpha의 별도 재구성 계약은 이 불투명 JPEG 경로로 대체하지 않는다.
pub(super) fn resize_opaque(
    source: &image::RgbaImage,
    (dw, dh): (u32, u32),
) -> Option<image::RgbaImage> {
    let (width, height) = source.dimensions();
    length(width, height)?;
    let temporary_len = length(dw, height)?;
    let output_len = length(dw, dh)?;
    if source.pixels().any(|p| p[3] != 255) {
        return None;
    }
    let horizontal = weights(width, dw);
    let vertical = weights(height, dh);
    let mut temporary = vec![0_u8; temporary_len];
    for y in 0..height as usize {
        for (x, taps) in horizontal.iter().enumerate() {
            let mut sums = [0_i64; 4];
            for &(sx, weight) in taps {
                let pixel = &source.as_raw()[(y * width as usize + sx) * 4..][..4];
                for c in 0..4 {
                    sums[c] += i64::from(pixel[c]) * weight;
                }
            }
            let pixel = &mut temporary[(y * dw as usize + x) * 4..][..4];
            pixel[3] = channel(sums[3]);
            for c in 0..3 {
                pixel[c] = channel(sums[c]).min(pixel[3]);
            }
        }
    }
    let mut output = vec![0_u8; output_len];
    for (y, taps) in vertical.iter().enumerate() {
        for x in 0..dw as usize {
            let mut sums = [0_i64; 4];
            for &(sy, weight) in taps {
                let pixel = &temporary[(sy * dw as usize + x) * 4..][..4];
                for c in 0..4 {
                    sums[c] += i64::from(pixel[c]) * weight;
                }
            }
            let alpha = channel(sums[3]);
            let pixel = &mut output[(y * dw as usize + x) * 4..][..4];
            pixel[3] = alpha;
            if alpha == 0 {
                pixel[..3].fill(255);
                continue;
            }
            for c in 0..3 {
                let value = channel(sums[c]).min(alpha);
                pixel[c] = ((u32::from(value) * 255 + u32::from(alpha) / 2) / u32::from(alpha))
                    .min(255) as u8;
            }
        }
    }
    image::RgbaImage::from_raw(dw, dh, output)
}

#[cfg(test)]
mod tests {
    use super::resize_opaque;

    #[test]
    fn independent_integer_centre_line_and_transparent_edge() {
        // 공개 GDI+ HQ bicubic의 별도 32×32 선 자극에서 관측한 값이다.
        // 문서/PDF 픽셀이나 현재 구현 계산값으로 기대 응답을 만들지 않는다.
        let source = image::RgbaImage::from_fn(32, 32, |x, y| {
            image::Rgba([
                if x == 16 { 192 } else { 128 },
                if y == 16 { 192 } else { 128 },
                128,
                255,
            ])
        });
        let output = resize_opaque(&source, (64, 16)).unwrap();
        for (x, expected) in [
            (28, 126),
            (29, 123),
            (30, 134),
            (31, 165),
            (32, 183),
            (33, 166),
            (34, 134),
            (35, 123),
            (36, 126),
        ] {
            assert!((i16::from(output.get_pixel(x, 8)[0]) - expected).abs() <= 1);
        }
        assert!((i16::from(output.get_pixel(32, 0)[3]) - 188).abs() <= 1);
        assert!((i16::from(output.get_pixel(0, 8)[3]) - 237).abs() <= 1);
    }

    #[test]
    fn allocation_and_opaque_source_contracts_are_checked() {
        let source = image::RgbaImage::from_pixel(2, 2, image::Rgba([0, 0, 0, 255]));
        assert!(resize_opaque(&source, (0, 2)).is_none());
        assert!(resize_opaque(&source, (8193, 2)).is_none());
        let alpha = image::RgbaImage::from_pixel(2, 2, image::Rgba([0, 0, 0, 128]));
        assert!(resize_opaque(&alpha, (2, 2)).is_none());
    }
}

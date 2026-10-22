//! 축소 비트맵은 목표 장치 격자에서 한 번만 저역 통과시킨다.
//!
//! source/layout 좌표는 유지한다. 인쇄 paint의 장치 경계에서 준비한
//! 픽셀을 fractional destination에서 다시 보간하지 않는다.

use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::io::Cursor;
use std::sync::Arc;

use crate::paint::RenderProfile;

use super::render_tree::BoundingBox;

const MAX_PIXELS: u64 = 67_108_864;
const CACHE_BYTES: usize = 16 * 1024 * 1024;
const CACHE_ENTRIES: usize = 64;

#[derive(Hash, PartialEq, Eq)]
struct CacheKey {
    source_hash: u64,
    source_len: usize,
    width: u32,
    height: u32,
    print: bool,
    device_rect: [u64; 4],
}

#[derive(Default)]
struct RasterCache {
    entries: HashMap<CacheKey, Arc<Vec<u8>>>,
    bytes: usize,
}

thread_local! {
    static CACHE: RefCell<RasterCache> = RefCell::new(RasterCache::default());
}

pub(crate) struct PreparedBitmap {
    pub bytes: Arc<Vec<u8>>,
    pub bbox: BoundingBox,
}

/// `bbox`는 crop/채움/안여백을 해석한 최종 paint 사각형이다. 실제로 잘린
/// source 사각형은 이 함수에 넘기지 않는다. 실패/비대상은 기존 paint로 돌아간다.
pub(crate) fn prepare_bitmap(
    bytes: &[u8],
    bbox: BoundingBox,
    matrix: [f64; 6],
    profile: RenderProfile,
) -> Option<PreparedBitmap> {
    let [a, b, c, d, e, f] = matrix;
    if !matches!(profile, RenderProfile::Print | RenderProfile::HighQuality)
        || b != 0.0
        || c != 0.0
        || a <= 0.0
        || d <= 0.0
        || bbox.width <= 0.0
        || bbox.height <= 0.0
        || !matrix
            .into_iter()
            .chain([bbox.x, bbox.y, bbox.width, bbox.height])
            .all(f64::is_finite)
    {
        return None;
    }
    let print = profile == RenderProfile::Print;
    // Print 그림은 장치 픽셀의 덮이는 구간 전체를 소유한다. 최근접 경계로
    // 먼저 줄이면 소수 원점에 따라 가는 선의 source 구간까지 달라진다.
    let lower = |x: f64| if print { x.floor() } else { x.round() };
    let upper = |x: f64| if print { x.floor() + 1.0 } else { x.round() };
    let left = lower(a * bbox.x + e);
    let right = upper(a * (bbox.x + bbox.width) + e);
    let top = lower(d * bbox.y + f);
    let bottom = upper(d * (bbox.y + bbox.height) + f);
    let (width, height) = (right - left, bottom - top);
    if ![left, right, top, bottom, width, height]
        .into_iter()
        .all(f64::is_finite)
        || width < 1.0
        || height < 1.0
        || width > u32::MAX as f64
        || height > u32::MAX as f64
        || width * height > MAX_PIXELS as f64
    {
        return None;
    }
    let (width, height) = (width as u32, height as u32);
    let dimensions = image::ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .ok()?
        .into_dimensions()
        .ok()?;
    // 얕은 축소는 선언된 Print 재구성 경로가 표본을 소유한다.
    // 더 큰 축소는 기존 장치 구간 평균을 유지한다.
    if super::image_resolver::png_sampling::print_bitmap_reconstruction(bytes)
        && super::cubic_sampling::modest_reduction(
            (f64::from(dimensions.0), f64::from(dimensions.1)),
            (a * bbox.width, d * bbox.height),
        )
    {
        return None;
    }
    if width >= dimensions.0
        || height >= dimensions.1
        || u64::from(dimensions.0) * u64::from(dimensions.1) > MAX_PIXELS
    {
        return None;
    }
    let mut hash = std::collections::hash_map::DefaultHasher::new();
    bytes.hash(&mut hash);
    // 투명 그림의 coverage는 source의 소수 원점과 실제 장치 배율에도
    // 의존한다. 같은 정수 출력 크기라는 이유로 다른 원점의 표본을 재사용하지 않는다.
    let device_rect = [
        a * bbox.x + e - left,
        d * bbox.y + f - top,
        a * bbox.width,
        d * bbox.height,
    ];
    let key = CacheKey {
        source_hash: hash.finish(),
        source_len: bytes.len(),
        width,
        height,
        print,
        device_rect: device_rect.map(f64::to_bits),
    };
    let png = CACHE.with(|cache| cache.borrow().entries.get(&key).cloned());
    let png = match png {
        Some(png) => png,
        None => {
            let mut reader = image::ImageReader::new(Cursor::new(bytes))
                .with_guessed_format()
                .ok()?;
            let mut limits = image::Limits::default();
            limits.max_alloc = Some(MAX_PIXELS * 4);
            reader.limits(limits);
            let source = reader.decode().ok()?;
            let transparent = source.color().has_alpha()
                && source.to_rgba8().pixels().any(|pixel| pixel.0[3] != 255);
            // 독립 Print 표본 C352/C354/C356의 구간 평균을 보존한다.
            // 각 source 픽셀은 정확히 한 장치 구간에 들어간다. Lanczos의
            // 음의 lobe나 새 보간으로 옅은 선의 밝기를 다시 바꾸지 않는다.
            let resized = if transparent {
                // Print의 투명 표본은 실제 소수 장치 사각형에서 coverage를
                // 적분한다. 숨은 RGB를 먼저 섞거나 투명 가장자리를 정수 box에
                // 늘리지 않는다. 다른 프로필의 기존 합성은 그대로 둔다.
                if !print {
                    return None;
                }
                image::DynamicImage::ImageRgba8(transparent_area_downsample(
                    &source.to_rgba8(),
                    width,
                    height,
                    device_rect,
                )?)
            } else if print {
                image::DynamicImage::ImageRgb8(integer_area_downsample(
                    &source.to_rgb8(),
                    width,
                    height,
                ))
            } else {
                source.resize_exact(width, height, image::imageops::FilterType::Lanczos3)
            };
            let mut output = Cursor::new(Vec::new());
            resized
                .write_to(&mut output, image::ImageFormat::Png)
                .ok()?;
            let png = Arc::new(output.into_inner());
            if png.len() <= CACHE_BYTES {
                CACHE.with(|cache| {
                    let mut cache = cache.borrow_mut();
                    if cache.entries.len() >= CACHE_ENTRIES || cache.bytes + png.len() > CACHE_BYTES
                    {
                        cache.entries.clear();
                        cache.bytes = 0;
                    }
                    cache.bytes += png.len();
                    cache.entries.insert(key, png.clone());
                });
            }
            png
        }
    };
    Some(PreparedBitmap {
        bytes: png,
        bbox: BoundingBox::new(
            (left - e) / a,
            (top - f) / d,
            f64::from(width) / a,
            f64::from(height) / d,
        ),
    })
}

fn integer_area_downsample(source: &image::RgbImage, width: u32, height: u32) -> image::RgbImage {
    let mut result = image::RgbImage::new(width, height);
    let boundary = |index: u32, source_extent: u32, target_extent: u32| {
        (u64::from(index) * u64::from(source_extent) / u64::from(target_extent)) as u32
    };
    for y in 0..height {
        let top = boundary(y, source.height(), height);
        let bottom = boundary(y + 1, source.height(), height);
        for x in 0..width {
            let left = boundary(x, source.width(), width);
            let right = boundary(x + 1, source.width(), width);
            let count = u64::from(bottom - top) * u64::from(right - left);
            let mut sum = [0u64; 3];
            for source_y in top..bottom {
                for source_x in left..right {
                    let pixel = source.get_pixel(source_x, source_y);
                    for channel in 0..3 {
                        sum[channel] += u64::from(pixel[channel]);
                    }
                }
            }
            result.put_pixel(x, y, image::Rgb(sum.map(|value| (value / count) as u8)));
        }
    }
    result
}

fn transparent_area_downsample(
    source: &image::RgbaImage,
    width: u32,
    height: u32,
    device_rect: [f64; 4],
) -> Option<image::RgbaImage> {
    let [x, y, w, h] = device_rect;
    let sx = f64::from(source.width()) / w;
    let sy = f64::from(source.height()) / h;
    let area = sx * sy;
    if !device_rect
        .into_iter()
        .chain([sx, sy, area])
        .all(f64::is_finite)
        || w <= 0.0
        || h <= 0.0
        || area <= 0.0
    {
        return None;
    }
    let spans = |count: u32, source: u32, origin: f64, scale: f64| {
        (0..count)
            .map(|i| {
                let first = (f64::from(i) - origin) * scale;
                let end = first + scale;
                (
                    first.floor().clamp(0.0, f64::from(source)) as u32,
                    end.ceil().clamp(0.0, f64::from(source)) as u32,
                    first,
                    end,
                )
            })
            .collect::<Vec<_>>()
    };
    let xs = spans(width, source.width(), x, sx);
    let ys = spans(height, source.height(), y, sy);
    let mut output = image::RgbaImage::new(width, height);
    for (y, &(top, bottom, y0, y1)) in ys.iter().enumerate() {
        for (x, &(left, right, x0, x1)) in xs.iter().enumerate() {
            let mut alpha = 0.0;
            let mut color = [0.0; 3];
            for iy in top..bottom {
                let wy = (f64::from(iy + 1).min(y1) - f64::from(iy).max(y0)).max(0.0);
                for ix in left..right {
                    let wx = (f64::from(ix + 1).min(x1) - f64::from(ix).max(x0)).max(0.0);
                    let pixel = source.get_pixel(ix, iy);
                    let weight = wx * wy * f64::from(pixel[3]);
                    alpha += weight;
                    for channel in 0..3 {
                        color[channel] += f64::from(pixel[channel]) * weight;
                    }
                }
            }
            let rgb = color.map(|value| {
                if alpha > 0.0 {
                    (value / alpha).round().clamp(0.0, 255.0) as u8
                } else {
                    0
                }
            });
            output.put_pixel(
                x as u32,
                y as u32,
                image::Rgba([
                    rgb[0],
                    rgb[1],
                    rgb[2],
                    (alpha / area).round().clamp(0.0, 255.0) as u8,
                ]),
            );
        }
    }
    Some(output)
}

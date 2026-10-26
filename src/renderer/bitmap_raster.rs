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

/// 논리 종이를 실제 정수 raster viewport에 투영하는 공통 장치 격자.
/// 배치 bbox는 바꾸지 않으며 쪽 바깥의 정수 이동도 같은 원점에서 보존한다.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct PrintPaperGrid {
    factors: [u64; 2],
    origin: [u64; 2],
}

impl PrintPaperGrid {
    pub(crate) fn new(paper: [f64; 2], matrix: [f64; 6]) -> Option<Self> {
        let [a, b, c, d, e, f] = matrix;
        if b != 0.0
            || c != 0.0
            || a <= 0.0
            || d <= 0.0
            || e.fract() != 0.0
            || f.fract() != 0.0
            || !paper.into_iter().chain(matrix).all(f64::is_finite)
            || paper.into_iter().any(|size| size <= 0.0)
        {
            return None;
        }
        let extents = [paper[0] * a, paper[1] * d];
        if extents.iter().any(|size| !size.is_finite() || *size <= 0.0) {
            return None;
        }
        let factors = extents.map(|size| size.ceil() / size);
        if factors.iter().any(|factor| !factor.is_finite()) {
            return None;
        }
        Some(Self {
            factors: factors.map(f64::to_bits),
            origin: [e.to_bits(), f.to_bits()],
        })
    }

    fn project(self, [x, y, w, h]: [f64; 4]) -> [f64; 4] {
        let [sx, sy] = self.factors.map(f64::from_bits);
        let [ox, oy] = self.origin.map(f64::from_bits);
        [(x - ox) * sx + ox, (y - oy) * sy + oy, w * sx, h * sy]
    }
}

/// `bbox`는 crop/채움/안여백을 해석한 최종 paint 사각형이다. 실제로 잘린
/// source 사각형은 이 함수에 넘기지 않는다. 실패/비대상은 기존 paint로 돌아간다.
pub(crate) fn prepare_bitmap(
    bytes: &[u8],
    bbox: BoundingBox,
    matrix: [f64; 6],
    profile: RenderProfile,
    paper_grid: Option<PrintPaperGrid>,
) -> Option<PreparedBitmap> {
    // SVG와 마찬가지로 메타파일 표본은 최종 장치에서 이미 합성했다.
    // 원본 비트맵용 경계 확장/저역 통과를 다시 적용하지 않는다.
    if super::image_resolver::png_sampling::metafile_surface(bytes) {
        return None;
    }
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
    let rectangle = [
        a * bbox.x + e,
        d * bbox.y + f,
        a * bbox.width,
        d * bbox.height,
    ];
    let rectangle = if print {
        paper_grid.map_or(rectangle, |grid| grid.project(rectangle))
    } else {
        rectangle
    };
    let [x, y, w, h] = rectangle;
    let left = lower(x);
    let right = upper(x + w);
    let top = lower(y);
    let bottom = upper(y + h);
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
            (w, h),
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
    let device_rect = [x - left, y - top, w, h];
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
            // 장치 픽셀이 덮는 원본 구간의 면적을 적분한다. 원본 픽셀을
            // 정수 경계 한쪽에만 배정하면 비정수 축소에서 가는 선의 밝기가
            // 구간 위상에 따라 달라진다. 두 축의 겹치는 비율을 함께 소비한다.
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
                image::DynamicImage::ImageRgb8(opaque_area_downsample(
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

/// 공통 정수 격자로 구간의 겹침을 표현해 부동소수점 경계 오차를 피한다.
fn area_span(index: u32, source: u32, target: u32) -> impl Iterator<Item = (u32, u64)> + Clone {
    let source = u64::from(source);
    let target = u64::from(target);
    let start = u64::from(index) * source;
    let end = start + source;
    (start / target..end.div_ceil(target)).map(move |pixel| {
        let overlap = end.min((pixel + 1) * target) - start.max(pixel * target);
        (pixel as u32, overlap)
    })
}

fn opaque_area_downsample(source: &image::RgbImage, width: u32, height: u32) -> image::RgbImage {
    let mut horizontal = image::RgbImage::new(width, source.height());
    for y in 0..source.height() {
        for x in 0..width {
            let mut sum = [0u64; 3];
            for (source_x, weight) in area_span(x, source.width(), width) {
                let pixel = source.get_pixel(source_x, y);
                for channel in 0..3 {
                    sum[channel] += u64::from(pixel[channel]) * weight;
                }
            }
            horizontal.put_pixel(
                x,
                y,
                image::Rgb(sum.map(|value| (value / u64::from(source.width())) as u8)),
            );
        }
    }
    let mut result = image::RgbImage::new(width, height);
    for y in 0..height {
        let span = area_span(y, source.height(), height);
        for x in 0..width {
            let mut sum = [0u64; 3];
            for (source_y, weight) in span.clone() {
                let pixel = horizontal.get_pixel(x, source_y);
                for channel in 0..3 {
                    sum[channel] += u64::from(pixel[channel]) * weight;
                }
            }
            result.put_pixel(
                x,
                y,
                image::Rgb(sum.map(|value| (value / u64::from(source.height())) as u8)),
            );
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

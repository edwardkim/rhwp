// 원본 창의 면적 기여와 전체/부분 bitmap 공용 캐시. 출력 backend에 의존하지 않는다.
use std::{
    cell::RefCell,
    collections::HashMap,
    hash::{Hash, Hasher},
    io::Cursor,
    sync::Arc,
};
pub(super) const MAX_PIXELS: u64 = 67_108_864;
pub(super) const CACHE_BYTES: usize = 16 * 1024 * 1024;
pub(super) const CACHE_ENTRIES: usize = 64;
#[derive(Hash, PartialEq, Eq)]
pub(super) struct CacheKey {
    pub(super) source_hash: u64,
    pub(super) source_len: usize,
    pub(super) width: u32,
    pub(super) height: u32,
    pub(super) print: bool,
    pub(super) device_rect: [u64; 4],
    pub(super) source_window: Option<[u64; 4]>,
}

#[derive(Default)]
pub(super) struct RasterCache {
    pub(super) entries: HashMap<CacheKey, Arc<Vec<u8>>>,
    pub(super) bytes: usize,
}

thread_local! {
    pub(super) static CACHE: RefCell<RasterCache> = RefCell::new(RasterCache::default());
}

pub(super) fn prepare(
    bytes: &[u8],
    width: u32,
    height: u32,
    device_rect: [f64; 4],
    source_window: [f64; 4],
) -> Option<Arc<Vec<u8>>> {
    let [sx, sy, sw, sh] = source_window;
    if width == 0
        || height == 0
        || u64::from(width) * u64::from(height) > MAX_PIXELS
        || !source_window
            .into_iter()
            .chain(device_rect)
            .all(f64::is_finite)
        || sx < 0.0
        || sy < 0.0
        || sw <= 0.0
        || sh <= 0.0
    {
        return None;
    }
    let dimensions = image::ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .ok()?
        .into_dimensions()
        .ok()?;
    if u64::from(dimensions.0) * u64::from(dimensions.1) > MAX_PIXELS
        || sx + sw > f64::from(dimensions.0)
        || sy + sh > f64::from(dimensions.1)
    {
        return None;
    }
    let mut hash = std::collections::hash_map::DefaultHasher::new();
    bytes.hash(&mut hash);
    let key = CacheKey {
        source_hash: hash.finish(),
        source_len: bytes.len(),
        width,
        height,
        print: true,
        device_rect: device_rect.map(f64::to_bits),
        source_window: Some(source_window.map(f64::to_bits)),
    };
    let cached = CACHE.with(|cache| cache.borrow().entries.get(&key).cloned());
    let png = match cached {
        Some(png) => png,
        None => {
            let mut reader = image::ImageReader::new(Cursor::new(bytes))
                .with_guessed_format()
                .ok()?;
            let mut limits = image::Limits::default();
            limits.max_alloc = Some(MAX_PIXELS * 4);
            reader.limits(limits);
            let source = reader.decode().ok()?;
            if source.color().has_alpha() && source.to_rgba8().pixels().any(|p| p[3] != 255) {
                return None;
            }
            let result = opaque_window_downsample(&source.to_rgb8(), width, height, source_window);
            let mut output = Cursor::new(Vec::new());
            image::DynamicImage::ImageRgb8(result)
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
    Some(png)
}

/// 원본 창의 소수 경계를 두 축에 그대로 투영한다. 창 밖 픽셀은 평균에 넣지 않는다.
pub(super) fn opaque_window_downsample(
    source: &image::RgbImage,
    width: u32,
    height: u32,
    window: [f64; 4],
) -> image::RgbImage {
    let [sx, sy, sw, sh] = window;
    let spans = |origin: f64, extent: f64, count: u32, limit: u32| -> Vec<Vec<(u32, f64)>> {
        (0..count)
            .map(|i| {
                let first = origin + f64::from(i) * extent / f64::from(count);
                let end = origin + f64::from(i + 1) * extent / f64::from(count);
                (first.floor().clamp(0.0, f64::from(limit)) as u32
                    ..end.ceil().clamp(0.0, f64::from(limit)) as u32)
                    .filter_map(|pixel| {
                        let overlap =
                            (end.min(f64::from(pixel + 1)) - first.max(f64::from(pixel))).max(0.0);
                        (overlap > 0.0).then_some((pixel, overlap))
                    })
                    .collect()
            })
            .collect()
    };
    let xs = spans(sx, sw, width, source.width());
    let ys = spans(sy, sh, height, source.height());
    let first_y = sy.floor() as u32;
    let last_y = (sy + sh).ceil() as u32;
    let mut horizontal = image::RgbImage::new(width, last_y - first_y);
    for y in first_y..last_y {
        for (x, span) in xs.iter().enumerate() {
            let mut sum = [0.0; 3];
            for &(ix, weight) in span {
                let pixel = source.get_pixel(ix, y);
                for c in 0..3 {
                    sum[c] += f64::from(pixel[c]) * weight;
                }
            }
            horizontal.put_pixel(
                x as u32,
                y - first_y,
                image::Rgb(
                    sum.map(|v| (v * f64::from(width) / sw).round().clamp(0.0, 255.0) as u8),
                ),
            );
        }
    }
    let mut result = image::RgbImage::new(width, height);
    for (y, span) in ys.iter().enumerate() {
        for x in 0..width {
            let mut sum = [0.0; 3];
            for &(iy, weight) in span {
                let pixel = horizontal.get_pixel(x, iy - first_y);
                for c in 0..3 {
                    sum[c] += f64::from(pixel[c]) * weight;
                }
            }
            result.put_pixel(
                x,
                y as u32,
                image::Rgb(
                    sum.map(|v| (v * f64::from(height) / sh).round().clamp(0.0, 255.0) as u8),
                ),
            );
        }
    }
    result
}

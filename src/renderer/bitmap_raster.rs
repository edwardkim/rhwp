//! 축소 비트맵은 목표 장치 격자에서 한 번만 저역 통과시킨다.
//!
//! source/layout 좌표는 유지한다. 인쇄 paint에서만 최근접 장치 경계를
//! 사용하여, 준비한 픽셀을 fractional destination에서 다시 보간하지 않는다.

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
    let left = (a * bbox.x + e).round();
    let right = (a * (bbox.x + bbox.width) + e).round();
    let top = (d * bbox.y + f).round();
    let bottom = (d * (bbox.y + bbox.height) + f).round();
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
    if width >= dimensions.0
        || height >= dimensions.1
        || u64::from(dimensions.0) * u64::from(dimensions.1) > MAX_PIXELS
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
            // 투명 픽셀을 straight-alpha로 필터링하면 테두리에 색이 번진다.
            // 이 경로는 불투명 자료만 다루고 투명 자료의 기존 합성을 유지한다.
            if source.color().has_alpha()
                && source.to_rgba8().pixels().any(|pixel| pixel.0[3] != 255)
            {
                return None;
            }
            let resized = source.resize_exact(width, height, image::imageops::FilterType::Lanczos3);
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

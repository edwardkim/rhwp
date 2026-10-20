//! WMF의 장치 크기·글꼴·clip 합성 뒤 전달색을 Native/WASM에서 함께 만든다.

use std::collections::{HashSet, VecDeque};

use resvg::{tiny_skia, usvg};

use super::metafile_fonts::{requests, MetafileFonts};
use crate::model::{image::ImageEffect, style::ImageFillMode};
use crate::paint::{ResolvedImageKind, ResolvedImagePayload};
use crate::renderer::render_tree::{BoundingBox, ImageNode};

const MAX_SVG_BYTES: usize = 32 * 1024 * 1024;
const MAX_SURFACE_PIXELS: u64 = 67_108_864;
const MAX_CACHE_BYTES: usize = 32 * 1024 * 1024;
const MAX_CACHE_ENTRIES: usize = 32;

#[derive(Default)]
pub(super) struct SurfaceCache {
    entries: VecDeque<([u8; 32], Vec<u8>)>,
    bytes: usize,
}

impl SurfaceCache {
    fn get(&self, key: &[u8; 32]) -> Option<Vec<u8>> {
        self.entries
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, data)| data.clone())
    }

    fn insert(&mut self, key: [u8; 32], data: &[u8]) {
        if data.len() > MAX_CACHE_BYTES {
            return;
        }
        while self.entries.len() >= MAX_CACHE_ENTRIES || self.bytes + data.len() > MAX_CACHE_BYTES {
            if let Some((_, old)) = self.entries.pop_front() {
                self.bytes -= old.len();
            } else {
                break;
            }
        }
        self.bytes += data.len();
        self.entries.push_back((key, data.to_vec()));
    }
}

fn device_size(
    image: &ImageNode,
    bbox: BoundingBox,
    natural: (f64, f64),
    pixel: f64,
) -> Option<(u32, u32)> {
    let (natural_w, natural_h) = natural;
    let crop = image.crop.map(|crop| {
        crate::renderer::svg::compute_image_crop_src(
            crop,
            crate::renderer::image_resolver::image_crop_reference(image),
            natural_w,
            natural_h,
        )
    });
    let (_, _, crop_w, crop_h) = crop.unwrap_or((0.0, 0.0, natural_w, natural_h));
    if crop_w <= 0.0 || crop_h <= 0.0 {
        return None;
    }
    let target = image
        .transform
        .effective_image_bbox(&image.paint_bbox(&bbox));
    let (w, h) = match image.fill_mode.unwrap_or(ImageFillMode::FitToSize) {
        ImageFillMode::FitToSize | ImageFillMode::Total => (
            target.width * natural_w / crop_w,
            target.height * natural_h / crop_h,
        ),
        ImageFillMode::None | ImageFillMode::Zoom => {
            let scale = (target.width / crop_w).min(target.height / crop_h);
            (natural_w * scale, natural_h * scale)
        }
        _ => {
            let (w, h) = image.original_size.unwrap_or(natural);
            (w * natural_w / crop_w, h * natural_h / crop_h)
        }
    };
    if !w.is_finite()
        || !h.is_finite()
        || w <= 0.0
        || h <= 0.0
        || !pixel.is_finite()
        || pixel <= 0.0
    {
        return None;
    }
    // 독립 Print는 비균등 배치에서도 원본 비율의 자원을 만든 뒤 최종 사각형에 늘린다.
    // 두 축의 장치 표본 수를 만족하는 단일 배율을 쓰며 crop·배치 좌표는 바꾸지 않는다.
    let scale = (w / natural_w).max(h / natural_h) / pixel;
    let w = (natural_w * scale).round();
    let h = (natural_h * scale).round();
    if !w.is_finite()
        || !h.is_finite()
        || w < 1.0
        || h < 1.0
        || w > f64::from(u32::MAX)
        || h > f64::from(u32::MAX)
    {
        return None;
    }
    let (w, h) = (w as u32, h as u32);
    (u64::from(w).checked_mul(u64::from(h))? <= MAX_SURFACE_PIXELS).then_some((w, h))
}

fn transmission(pixmap: tiny_skia::Pixmap) -> Option<Vec<u8>> {
    // tiny-skia는 premultiplied RGBA다. 원본 alpha를 바꾸거나 색에 두 번 곱하지 않는다.
    let (width, height) = (pixmap.width(), pixmap.height());
    let mut pixels = pixmap.take_demultiplied();
    let mut palette = HashSet::from([[0u8; 3], [255u8; 3]]);
    for pixel in pixels.chunks_exact(4) {
        if pixel[3] == 0 {
            continue;
        }
        palette.insert([pixel[0], pixel[1], pixel[2]]);
        if palette.len() > 256 {
            break;
        }
    }
    let expand = palette.len() > 256;
    if expand {
        for pixel in pixels.chunks_exact_mut(4) {
            for channel in &mut pixel[..3] {
                *channel = channel.saturating_add(*channel >> 5);
            }
        }
    }
    let image = image::RgbaImage::from_raw(width, height, pixels)?;
    let mut png = std::io::Cursor::new(Vec::new());
    image::DynamicImage::ImageRgba8(image)
        .write_to(&mut png, image::ImageFormat::Png)
        .ok()?;
    Some(png.into_inner())
}

pub(super) fn payload(
    image: &ImageNode,
    bbox: BoundingBox,
    pixel: f64,
    fonts: &MetafileFonts,
) -> Option<ResolvedImagePayload> {
    if image.effect != ImageEffect::RealPic || image.brightness != 0 || image.contrast != 0 {
        return None;
    }
    let source = image.data.as_deref()?;
    // 표준 WMFC의 enhanced content는 기존 EMF/EMF+ 경로가 소유한다.
    if crate::parser::ole_container::emf_from_wmf_comment_chunks(source).is_some() {
        return None;
    }
    let svg = crate::renderer::svg::convert_wmf_to_svg(source)?;
    if svg.len() > MAX_SVG_BYTES {
        return None;
    }
    let natural = crate::renderer::metafile_geometry::svg_viewport(&svg)?;
    let (width, height) = device_size(image, bbox, natural, pixel)?;
    let font_requests = requests(&svg)?;
    let font_sources = fonts.coverage_fingerprint(&font_requests)?;
    let mut hash = blake3::Hasher::new();
    hash.update(source);
    hash.update(&width.to_le_bytes());
    hash.update(&height.to_le_bytes());
    hash.update(&fonts.fingerprint);
    hash.update(&font_sources);
    let key = *hash.finalize().as_bytes();
    let cached = fonts.surfaces.lock().ok()?.get(&key);
    let data = if let Some(data) = cached {
        data
    } else {
        // SVG 안에 글꼴이 없으면 빈 glyph를 완료 PNG로 캐시하지 않는다.
        let options = usvg::Options {
            resources_dir: None,
            fontdb: fonts.database(),
            shape_rendering: usvg::ShapeRendering::CrispEdges,
            text_rendering: usvg::TextRendering::OptimizeSpeed,
            image_href_resolver: usvg::ImageHrefResolver {
                resolve_data: usvg::ImageHrefResolver::default_data_resolver(),
                resolve_string: Box::new(|_, _| None),
            },
            ..Default::default()
        };
        let tree = usvg::Tree::from_data(&svg, &options).ok()?;
        let mut pixmap = tiny_skia::Pixmap::new(width, height)?;
        let scale = tiny_skia::Transform::from_scale(
            width as f32 / tree.size().width(),
            height as f32 / tree.size().height(),
        );
        resvg::render(&tree, scale, &mut pixmap.as_mut());
        let data = transmission(pixmap)?;
        fonts.surfaces.lock().ok()?.insert(key, &data);
        data
    };
    Some(ResolvedImagePayload {
        data,
        mime: "image/png",
        kind: ResolvedImageKind::OutputDevice,
        suppress_effects: false,
    })
}

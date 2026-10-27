//! WMF의 장치 크기·글꼴·clip 합성 뒤 전달색을 Native/WASM에서 함께 만든다.

use std::collections::{HashSet, VecDeque};

use resvg::usvg as style_usvg;
use resvg::{tiny_skia, usvg};

// PDF(usvg 0.45)와 메타파일(resvg의 usvg 0.47)은 타입이 다르다.
// 판정 구현은 한 소스를 공유하되 각 소비자의 실제 선택 글꼴로 실행한다.
#[cfg_attr(
    not(target_arch = "wasm32"),
    expect(
        clippy::duplicate_mod,
        reason = "PDF는 usvg 0.45, 메타파일은 usvg 0.47의 서로 다른 타입으로 같은 판정 소스를 인스턴스화한다"
    )
)]
#[path = "../renderer/pdf_synthetic_bold.rs"]
mod synthetic_bold;
#[cfg_attr(
    not(target_arch = "wasm32"),
    expect(
        clippy::duplicate_mod,
        reason = "PDF는 usvg 0.45, 메타파일은 usvg 0.47의 서로 다른 타입으로 같은 판정 소스를 인스턴스화한다"
    )
)]
#[path = "../renderer/pdf_synthetic_italic.rs"]
mod synthetic_italic;

use super::metafile_fonts::{requests, MetafileFonts};
use crate::model::{image::ImageEffect, style::ImageFillMode};
use crate::paint::{ResolvedImageKind, ResolvedImagePayload};
use crate::renderer::render_tree::{BoundingBox, ImageNode};

#[path = "metafile_masks.rs"]
mod masks;

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
    pub(super) fn get(&self, key: &[u8; 32]) -> Option<Vec<u8>> {
        self.entries
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, data)| data.clone())
    }

    pub(super) fn insert(&mut self, key: [u8; 32], data: &[u8]) {
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
    let target = image.transformed_paint_bbox(&bbox);
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
    device_size_for_extent((w, h), natural, pixel)
}

fn device_size_for_extent(
    (w, h): (f64, f64),
    (natural_w, natural_h): (f64, f64),
    pixel: f64,
) -> Option<(u32, u32)> {
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
    let mut png = png.into_inner();
    crate::renderer::image_resolver::png_sampling::mark_metafile_surface(&mut png);
    Some(png)
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
    let svg = crate::wmf::converter::WMFConverter::new(
        source,
        crate::wmf::converter::SVGPlayer::new().with_source_contract(),
    )
    .run()
    .ok()?;
    render_payload(image, bbox, pixel, fonts, svg, false)
}

/// 색 보정 SVG도 같은 Print 장치에서 재생한다. painter마다 낮은 해상도에서
/// 다시 축소하거나 이미 적용한 색 전달을 두 번 처리하지 않는다.
pub(super) fn picture_payload(
    image: &ImageNode,
    bbox: BoundingBox,
    pixel: f64,
    fonts: &MetafileFonts,
) -> Option<ResolvedImagePayload> {
    if !super::metafile_picture::eligible(image) {
        return None;
    }
    let source = image.data.as_deref()?;
    if crate::parser::ole_container::emf_from_wmf_comment_chunks(source).is_some() {
        return None;
    }
    super::metafile_picture::geometry(image, bbox)?;
    // 이 producer는 현재 변환기가 확인한 단일 bitmap만 받는다.
    // 벡터·ROP 또는 효과를 굽지 않은 일반 SVG는 종전 경로를 유지한다.
    let resolved = crate::renderer::image_resolver::resolve_image_payload(image)?;
    if !resolved.suppress_effects || resolved.mime != "image/svg+xml" {
        return None;
    }
    render_payload(image, bbox, pixel, fonts, resolved.data, true)
}

fn render_payload(
    image: &ImageNode,
    bbox: BoundingBox,
    pixel: f64,
    fonts: &MetafileFonts,
    svg: Vec<u8>,
    tone_baked: bool,
) -> Option<ResolvedImagePayload> {
    let source = image.data.as_deref()?;
    if svg.len() > MAX_SVG_BYTES {
        return None;
    }
    let natural = crate::renderer::metafile_geometry::svg_viewport(&svg)?;
    let (width, height) = if tone_baked {
        let (full, _) = super::metafile_picture::geometry(image, bbox)?;
        device_size_for_extent((full.width, full.height), natural, pixel)?
    } else {
        device_size(image, bbox, natural, pixel)?
    };
    // 색·mask 공통 표본기가 수용하는 한도를 게시 전에 확인한다.
    if tone_baked
        && (width > 8192
            || height > 8192
            || u64::from(width) * u64::from(height) > 32 * 1024 * 1024)
    {
        return None;
    }
    let font_requests = requests(&svg)?;
    let font_sources = fonts.coverage_fingerprint(&font_requests)?;
    let mut hash = blake3::Hasher::new();
    hash.update(source);
    if tone_baked {
        hash.update(b"picture-tone-separate-mask");
        hash.update(&svg);
    }
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
        // 요청 weight와 실제 선택된 SFNT를 같은 공통 판정으로 대조한다.
        // Bold face가 없는 글자는 기존 0.02em 인쇄 획을 사용하며 실제 Bold나
        // 기존 stroke는 다시 굵히지 않는다. 처리 불가한 혼합 스타일은 캐시하지 않는다.
        // ROP의 비트맵 패턴 마스크는 원본 정수 격자에서 먼저 합성한다.
        // 비트 연산 도중 축소 보간하면 흑백 마스크가 회색으로 바뀌어 배경과
        // 전경의 소속이 달라진다. 텍스트·벡터 형제는 그대로 최종 장치로 전달한다.
        // 한 논리 단위가 한 장치 픽셀인 MM_TEXT만 원본 정수 격자를 소유한다.
        // MM_ANISOTROPIC은 기존 2024 경로와 같은 최종 장치에서 합성한다.
        let source_pixel_grid = roxmltree::Document::parse(std::str::from_utf8(&svg).ok()?)
            .ok()?
            .root_element()
            .attribute("data-rhwp-wmf-pixel-grid")
            == Some("1");
        let mask_grid = source_pixel_grid
            .then(|| masks::flatten_bitmap_masks(&svg))
            .flatten();
        let source = std::str::from_utf8(mask_grid.as_deref().unwrap_or(&svg)).ok()?;
        let (prepared, report) =
            synthetic_bold::prepare_svg_with_synthetic_bold(source, &options).ok()?;
        if report.unsupported_texts > 0 {
            return None;
        }
        let tree = usvg::Tree::from_str(&prepared, &options).ok()?;
        let mut pixmap = tiny_skia::Pixmap::new(width, height)?;
        let scale = tiny_skia::Transform::from_scale(
            width as f32 / tree.size().width(),
            height as f32 / tree.size().height(),
        );
        resvg::render(&tree, scale, &mut pixmap.as_mut());
        let data = if tone_baked {
            picture_transmission(pixmap)?
        } else {
            transmission(pixmap)?
        };
        fonts.surfaces.lock().ok()?.insert(key, &data);
        data
    };
    Some(ResolvedImagePayload {
        data,
        mime: "image/png",
        kind: ResolvedImageKind::OutputDevice,
        suppress_effects: tone_baked,
    })
}

fn picture_transmission(pixmap: tiny_skia::Pixmap) -> Option<Vec<u8>> {
    let (width, height) = (pixmap.width(), pixmap.height());
    let mut pixels = pixmap.take_demultiplied();
    // WMF 창의 비점유 부분은 흰색 평면과 별도의 coverage다.
    // 이미 구운 색에는 기존 transmission의 추가 확장을 적용하지 않는다.
    for pixel in pixels.chunks_exact_mut(4) {
        if pixel[3] == 0 {
            pixel[..3].fill(255);
        }
    }
    let image = image::RgbaImage::from_raw(width, height, pixels)?;
    let mut encoded = std::io::Cursor::new(Vec::new());
    image::DynamicImage::ImageRgba8(image)
        .write_to(&mut encoded, image::ImageFormat::Png)
        .ok()?;
    let mut encoded = encoded.into_inner();
    crate::renderer::image_resolver::png_sampling::mark_print_separate_alpha(&mut encoded);
    Some(encoded)
}

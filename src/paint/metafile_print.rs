//! 2020 Print의 source 계약을 공통 장치 재생 자료로 내린다.

mod catalog;
mod frame;
mod images;
mod masks;
mod metrics;
#[cfg(any(feature = "native-skia", target_arch = "wasm32"))]
mod replay;
mod text;
mod xml;

pub(crate) use catalog::{Catalogue, SourceUse};
#[cfg(any(feature = "native-skia", target_arch = "wasm32"))]
pub(crate) use replay::{is_prepared, rasterize};

use super::metafile_fonts::{requests, MetafileFonts};
use crate::model::{image::ImageEffect, style::ImageFillMode};
use crate::paint::{ResolvedImageKind, ResolvedImagePayload};
use crate::renderer::render_tree::{BoundingBox, ImageNode};

pub(super) fn payload(
    image: &ImageNode,
    bbox: BoundingBox,
    page: (f64, f64),
    origin: (f64, f64),
    dpi: f64,
    fonts: &MetafileFonts,
    catalogue: &Catalogue,
) -> Option<ResolvedImagePayload> {
    if image.effect != ImageEffect::RealPic
        || image.brightness != 0
        || image.contrast != 0
        || image.opacity != 1.0
        || image.content_inset.is_some()
        || image.transform.has_transform()
        || !matches!(
            image.fill_mode,
            None | Some(ImageFillMode::FitToSize | ImageFillMode::Total)
        )
    {
        return None;
    }
    let source = catalogue.source(image.data.as_deref()?)?;
    let viewport = crate::renderer::metafile_geometry::svg_viewport(&source.svg)?;
    let document = roxmltree::Document::parse(std::str::from_utf8(&source.svg).ok()?).ok()?;
    let window = xml::numbers::<8>(document.root_element().attribute("data-rhwp-wmf-window")?)?;
    if (window[2], window[3]) != viewport || window[4..] != [1.0, 1.0, 0.0, 0.0] {
        return None;
    }
    if image.crop.is_some_and(|crop| {
        crate::renderer::svg::compute_image_crop_src(
            crop,
            crate::renderer::image_resolver::image_crop_reference(image),
            viewport.0,
            viewport.1,
        ) != (0.0, 0.0, viewport.0, viewport.1)
    }) {
        return None;
    }
    let frame = frame::Frame::new(page, origin, bbox, dpi)?;
    let coverage = fonts.coverage_fingerprint(&requests(&source.svg)?)?;
    let mut hash = blake3::Hasher::new();
    hash.update(b"wmf-2020-source-replay-v1");
    hash.update(&catalogue.fingerprint);
    hash.update(&fonts.fingerprint);
    hash.update(&coverage);
    hash.update(&source.svg);
    hash.update(frame.encode().as_bytes());
    let key = *hash.finalize().as_bytes();
    let cached = fonts.surfaces.lock().ok()?.get(&key);
    let data = if let Some(bytes) = cached {
        bytes
    } else {
        // ROP 마스크의 작은 조각은 사진 자원이 아니다. 원본 격자로 합성한
        // 결과에는 표식을 남겨 JPEG 전달로 다시 압축하지 않는다.
        let pictures =
            masks::flatten_bitmap_masks(&source.svg).unwrap_or_else(|| source.svg.as_ref().clone());
        let pictures = images::prepare(&pictures, frame, viewport, catalogue)?;
        let outlines = text::prepare(&pictures, frame, source.preceding_polygon?, fonts)?;
        let source = std::str::from_utf8(&outlines).ok()?;
        let document = roxmltree::Document::parse(source).ok()?;
        let data = xml::apply(
            source,
            xml::attributes(
                source,
                document.root_element(),
                &[(frame::ATTRIBUTE, frame.encode())],
            )?,
        )?
        .into_bytes();
        fonts.surfaces.lock().ok()?.insert(key, &data);
        data
    };
    Some(ResolvedImagePayload {
        data,
        mime: "image/svg+xml",
        kind: ResolvedImageKind::OutputDevice,
        suppress_effects: false,
    })
}

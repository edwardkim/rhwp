//! 한컴 2020 Print에서 1비트 DIB 브러시를 출력 장치 크기로 배치한다.

use super::xml;
use crate::model::{image::ImageEffect, style::ImageFillMode};
use crate::paint::{ResolvedImageKind, ResolvedImagePayload};
use crate::renderer::{
    image_resolver, metafile_geometry,
    render_tree::{BoundingBox, ImageNode},
};

const BRUSH_DEVICE_DPI: f64 = 100.0;
pub(super) const MONOCHROME_TILE: &str = "data-rhwp-wmf-pattern-bpp";

pub(in crate::paint) fn place_monochrome_tiles(
    image: &ImageNode,
    frame: BoundingBox,
    dpi: f64,
    payload: &mut ResolvedImagePayload,
) {
    if payload.mime != "image/svg+xml"
        || payload.kind != ResolvedImageKind::FormatConverted
        || image.effect != ImageEffect::RealPic
        || image.brightness != 0
        || image.contrast != 0
        || image.transform.has_transform()
        || !matches!(image.fill_mode, None | Some(ImageFillMode::FitToSize))
    {
        return;
    }
    let Some(source) = image.data.as_deref() else {
        return;
    };
    if image_resolver::detect_image_mime_type(source) != "image/x-wmf" {
        return;
    }
    if let Some(svg) = scaled_tiles(image, frame, dpi, &payload.data) {
        payload.data = svg.into_bytes();
        payload.kind = ResolvedImageKind::OutputDevice;
    }
}

fn scaled_tiles(image: &ImageNode, frame: BoundingBox, dpi: f64, data: &[u8]) -> Option<String> {
    let svg = std::str::from_utf8(data).ok()?;
    if !svg.contains(MONOCHROME_TILE) || data.len() > xml::MAX_SVG_BYTES {
        return None;
    }
    let (width, height) = metafile_geometry::svg_viewport(data)?;
    if image.crop.is_some_and(|crop| {
        crate::renderer::svg::compute_image_crop_src(
            crop,
            metafile_geometry::wmf_crop_reference(
                image.original_size_hu,
                image.data.as_deref(),
                image.crop,
            ),
            width,
            height,
        ) != (0.0, 0.0, width, height)
    }) {
        return None;
    }
    let device_pixel = dpi / BRUSH_DEVICE_DPI;
    let scale = [
        width / frame.width * device_pixel,
        height / frame.height * device_pixel,
    ];
    if scale
        .iter()
        .any(|value| !value.is_finite() || *value <= 0.0)
    {
        return None;
    }
    let parsed = roxmltree::Document::parse(svg).ok()?;
    let mut edits = Vec::new();
    for tile in parsed.descendants().filter(|node| {
        node.has_tag_name("pattern")
            && node.attribute(MONOCHROME_TILE) == Some("1")
            && node.attribute("width") == Some("8")
            && node.attribute("height") == Some("8")
            && node.attribute("patternTransform").is_none()
    }) {
        // 펜과 도형은 WMF 논리 창을 사용한다. 단색 DIB 브러시의 원본 픽셀은
        // 장치 격자를 사용하므로 타일 자체에만 역배율을 적용한다.
        let transform = format!("scale({} {})", scale[0], scale[1]);
        edits.extend(xml::attributes(
            svg,
            tile,
            &[("patternTransform", transform)],
        )?);
    }
    if edits.is_empty() {
        return None;
    }
    xml::apply(svg, edits)
}

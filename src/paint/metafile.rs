//! 레이어 출력의 장치와 그림 배치를 메타파일 재생에 전달한다.

use crate::model::style::ImageFillMode;
use crate::paint::{ResolvedImageKind, ResolvedImagePayload};
use crate::renderer::render_tree::{BoundingBox, ImageNode, RawSvgNode};

/// 한컴 2024 Print의 EMF 래스터 장치. 독립 Print의 1515×1500 자원에서
/// 1픽셀 cosmetic 선을 확인했다. 한컴 2020의 벡터 경로에는 적용하지 않는다.
pub(crate) const HANCOM_2024_PRINT_DEVICE_DPI: f64 = 600.0;

pub(crate) fn has_wmf(node: &crate::renderer::render_tree::RenderNode) -> bool {
    if !node.visible {
        return false;
    }
    matches!(&node.node_type, crate::renderer::render_tree::RenderNodeType::Image(image)
        if image.data.as_deref().is_some_and(|data|
            crate::renderer::image_resolver::detect_image_mime_type(data) == "image/x-wmf"))
        || node.children.iter().any(has_wmf)
}

pub(super) fn image_payload(
    image: &ImageNode,
    bbox: BoundingBox,
    device_pixel_width: Option<f64>,
    fonts: &super::metafile_fonts::MetafileFonts,
) -> Option<ResolvedImagePayload> {
    let Some(pixel) = device_pixel_width else {
        return crate::renderer::image_resolver::resolve_image_payload(image);
    };
    let data = image.data.as_deref()?;
    if image.hancom2024_print_bitmap
        && crate::renderer::image_resolver::detect_image_mime_type(data) == "image/x-wmf"
    {
        if let Some(payload) = super::metafile_surface::payload(image, bbox, pixel, fonts) {
            return Some(payload);
        }
    }
    if crate::renderer::image_resolver::detect_image_mime_type(data) != "image/x-emf" {
        return crate::renderer::image_resolver::resolve_image_payload(image);
    }
    let (natural_w, natural_h) = crate::emf::standalone_size(data)?;
    let crop = image.crop.map(|crop| {
        crate::renderer::svg::compute_image_crop_src(
            crop,
            image.original_size_hu,
            natural_w,
            natural_h,
        )
    });
    let (_, _, crop_w, crop_h) = crop.unwrap_or((0.0, 0.0, natural_w, natural_h));
    if crop_w <= 0.0 || crop_h <= 0.0 {
        return crate::renderer::image_resolver::resolve_image_payload(image);
    }
    let target = image
        .transform
        .effective_image_bbox(&image.paint_bbox(&bbox));
    let mode = image.fill_mode.unwrap_or(ImageFillMode::FitToSize);
    let (width, height) = match mode {
        ImageFillMode::FitToSize | ImageFillMode::Total => (
            target.width * natural_w / crop_w,
            target.height * natural_h / crop_h,
        ),
        ImageFillMode::None | ImageFillMode::Zoom => {
            let scale = (target.width / crop_w).min(target.height / crop_h);
            (natural_w * scale, natural_h * scale)
        }
        _ => {
            let (w, h) = image.original_size.unwrap_or((natural_w, natural_h));
            (w * natural_w / crop_w, h * natural_h / crop_h)
        }
    };
    let data = crate::emf::convert_to_sized_svg(data, width, height, pixel)?;
    let data = super::metafile_print_coverage::apply(&data).unwrap_or(data);
    Some(ResolvedImagePayload {
        data,
        mime: "image/svg+xml",
        kind: ResolvedImageKind::OutputDevice,
        suppress_effects: false,
    })
}

pub(super) fn raw_svg(raw: &RawSvgNode, device_pixel_width: Option<f64>) -> RawSvgNode {
    let mut output = raw.clone();
    if let (Some(pixel), Some(source)) = (device_pixel_width, raw.emf_source.as_ref()) {
        if let Ok(svg) =
            crate::emf::convert_to_svg_with_device(&source.data, source.render_rect, Some(pixel))
        {
            output.svg = super::metafile_print_coverage::apply(svg.as_bytes())
                .and_then(|bytes| String::from_utf8(bytes).ok())
                .unwrap_or(svg);
        }
    }
    output
}

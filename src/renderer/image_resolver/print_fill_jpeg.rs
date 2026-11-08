//! 전체 JPEG 채움의 원본 RGB와 배치별 Print 압축을 공통 자원으로 발행한다.

use super::{memoized, print_codec, Conversion};
use crate::paint::{ResolvedImageKind, ResolvedImagePayload};
use crate::renderer::render_tree::ImageNode;

pub(super) fn payload(image: &ImageNode, width_points: f64) -> Option<ResolvedImagePayload> {
    if !crate::paint::device_image::eligible(image)
        || !width_points.is_finite()
        || width_points <= 0.0
    {
        return None;
    }
    let source = image.data.as_deref()?;
    if super::detect_image_mime_type(source) != "image/jpeg"
        || super::png_sampling::pdf_source(source)
    {
        return None;
    }
    let header = super::canvaskit_encoded_image_header(source)?;
    if !header.is_within_decode_limits() {
        return None;
    }
    let quality = print_codec::requested_quality(width_points, header.width);
    // 같은 JPEG를 Picture와 채움으로 사용할 때도 두 색 계약의 캐시를 섞지 않는다.
    let data = memoized(
        Conversion::Hancom2024FillJpegRgb { quality },
        source,
        || {
            let rgb = super::print_tone::decode_jpeg_fill_rgb(source)?;
            let quality = print_codec::quality_for_palette(quality, print_codec::palette_len(&rgb));
            print_codec::encode_rgb(rgb, quality)
        },
    )?;
    Some(ResolvedImagePayload {
        data,
        mime: "image/jpeg",
        kind: ResolvedImageKind::OutputDevice,
        suppress_effects: false,
    })
}

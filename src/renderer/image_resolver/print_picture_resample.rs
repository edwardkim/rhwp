//! 한컴 2024 Print profile의 JPEG 그림 자원 크기와 공통 재표본화.
//!
//! 선언 크기는 논리 point이며 종이 mapping 뒤의 최종 bbox와 구분한다.
//! 소수 DPI 비교 대신 정수 장치 크기를 먼저 확정한다. 한 축 축소가 필요하면
//! 다른 축 확대도 같은 자원에 반영한다. 편집 원본과 최종 그림 위치는 유지한다.

use super::{
    detect_image_mime_type, memoized, print_alpha_2024 as print_alpha, print_codec, print_tone,
    Conversion,
};
use crate::model::image::ImageEffect;
use crate::paint::{ResolvedImageKind, ResolvedImagePayload};
use crate::renderer::image_header::{CANVASKIT_MAX_IMAGE_DIMENSION, CANVASKIT_MAX_IMAGE_PIXELS};
use crate::renderer::render_tree::ImageNode;

/// 독립 한컴 2024 Print와 두 축 정수 경계 대조에서 확인한 자원 profile이다.
/// HWP 파일 형식의 해상도나 화면/최종 출력 DPI가 아니다.
const PRINT_RESOURCE_DPI: f64 = 500.0;

fn dimensions(source: (u32, u32), points: (f64, f64)) -> Option<(u32, u32)> {
    if ![points.0, points.1]
        .into_iter()
        .all(|n| n.is_finite() && n > 0.0)
    {
        return None;
    }
    let width = (points.0 * PRINT_RESOURCE_DPI / 72.0).round();
    let height = (points.1 * PRINT_RESOURCE_DPI / 72.0).round();
    if width < 1.0
        || height < 1.0
        || width > f64::from(CANVASKIT_MAX_IMAGE_DIMENSION)
        || height > f64::from(CANVASKIT_MAX_IMAGE_DIMENSION)
    {
        return None;
    }
    let size = (width as u32, height as u32);
    (u64::from(size.0) * u64::from(size.1) <= CANVASKIT_MAX_IMAGE_PIXELS
        && (size.0 < source.0 || size.1 < source.1))
        .then_some(size)
}

fn eligible(image: &ImageNode) -> bool {
    image.hancom2024_print_bitmap
        && image.effect == ImageEffect::RealPic
        && image.brightness == 0
        && image.contrast == 0
        && image.opacity == 1.0
        && image.fill_mode.is_none()
        && image.content_inset.is_none()
        && !image.transform.has_transform()
        && image.crop.is_none_or(|(l, t, r, b)| {
            super::image_crop_reference(image).is_some_and(|(w, h)| {
                l == 0 && t == 0 && i64::from(r) == i64::from(w) && i64::from(b) == i64::from(h)
            })
        })
        && image
            .data
            .as_deref()
            .is_some_and(|data| detect_image_mime_type(data) == "image/jpeg")
}

pub(super) fn is_device_picture(image: &ImageNode, payload: Option<&ResolvedImagePayload>) -> bool {
    eligible(image)
        && payload.is_some_and(|p| {
            p.kind == ResolvedImageKind::OutputDevice
                && super::png_sampling::print_separate_alpha(&p.data)
        })
}

pub(super) fn payload(
    image: &ImageNode,
    points: (f64, f64),
    width_points: f64,
) -> Option<ResolvedImagePayload> {
    if !eligible(image) || !width_points.is_finite() || width_points <= 0.0 {
        return None;
    }
    let source = image.data.as_deref()?;
    let header =
        super::canvaskit_encoded_image_header(source).filter(|h| h.is_within_decode_limits())?;
    let (width, height) = dimensions((header.width, header.height), points)?;
    let quality = print_codec::requested_quality(width_points, width);
    let data = memoized(
        Conversion::Hancom2024PrintResampled {
            width,
            height,
            quality,
        },
        source,
        || {
            let source =
                print_tone::decode_print_bitmap(source, image::ImageFormat::Jpeg)?.to_rgba8();
            let rgba = super::print_prefilter::resize_opaque(&source, (width, height))?;
            // 보간 결과는 이미 straight RGB다. 원본 PNG의 PArgb 전달을 다시 적용하지 않는다.
            print_alpha::encode_planes(rgba, quality, |channel, _alpha| channel)
        },
    )?;
    Some(ResolvedImagePayload {
        data,
        mime: "image/png",
        kind: ResolvedImageKind::OutputDevice,
        suppress_effects: false,
    })
}

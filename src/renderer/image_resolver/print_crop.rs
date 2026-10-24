//! JPEG Print 자원의 source 창. 팔레트 색 수를 세는 완전 내포 창과 구분한다.

use crate::renderer::render_tree::ImageNode;

// source를 CPU에서 잘라 실제 출력 자원을 만들 때만 쓰는 제한이다.
// 최종 PNG 창은 기존8192px·32Mpix 검사도 별도로 통과해야 한다.
// Native의 source raster 준비와 같은64Mpix 안에서만 디코드한다.
const MAX_SOURCE_PIXELS: u64 = crate::renderer::image_header::CANVASKIT_MAX_IMAGE_PIXELS * 2;

fn source_header(
    bytes: &[u8],
) -> Option<crate::renderer::image_header::CanvasKitEncodedImageHeader> {
    crate::renderer::image_header::canvaskit_encoded_image_header(bytes).filter(|header| {
        header.width <= crate::renderer::image_header::CANVASKIT_MAX_IMAGE_DIMENSION
            && header.height <= crate::renderer::image_header::CANVASKIT_MAX_IMAGE_DIMENSION
            && u64::from(header.width) * u64::from(header.height) <= MAX_SOURCE_PIXELS
    })
}

/// 독립 한컴 2024 Print의 JPEG 자원은 원점과 길이를 각각 정수화한다.
/// 끝점 ceil/floor로 완전히 포함된 픽셀만 고르면 원점이 한 픽셀 밀리고,
/// 원점의 소수부에 따라 길이도 줄어든다. source IR과 BinData를 유지하고
/// 복제된 Print paint에만 픽셀 좌표 창을 발행한다.
pub(crate) fn publish_print_jpeg_crop(image: &mut ImageNode) {
    if !image.hancom2024_print_bitmap {
        return;
    }
    let Some((left, top, right, bottom)) = image.crop else {
        return;
    };
    let Some((ref_width, ref_height)) = image.original_size_hu.filter(|&(w, h)| w > 0 && h > 0)
    else {
        return;
    };
    if left < 0
        || top < 0
        || right <= left
        || bottom <= top
        || i64::from(right) > i64::from(ref_width)
        || i64::from(bottom) > i64::from(ref_height)
    {
        return;
    }
    let Some(data) = image
        .data
        .as_deref()
        .filter(|bytes| super::detect_image_mime_type(bytes) == "image/jpeg")
    else {
        return;
    };
    let Some(header) = source_header(data) else {
        return;
    };
    let quantize = |value: i32, pixels: u32, reference: u32| {
        (i64::from(value) * i64::from(pixels) / i64::from(reference)) as i32
    };
    let x = quantize(left, header.width, ref_width);
    let y = quantize(top, header.height, ref_height);
    let width = quantize(right - left, header.width, ref_width);
    let height = quantize(bottom - top, header.height, ref_height);
    if width == 0 || height == 0 {
        return;
    }
    image.crop = Some((x, y, x + width, y + height));
    image.original_size_hu = Some((header.width, header.height));
}

/// 같은 정수 source 창은 실제 crop 여부에 따라 축소 필터가 갈라지면 안 된다.
/// 창을 한 번 발행하고 Native·Canvas·SVG는 일반 source raster로 재생한다.
/// 최종 장치 표본(OutputDevice)이 아니므로 기존 저역 통과는 유지한다.
pub(crate) fn resolve_print_jpeg_window(
    image: &mut ImageNode,
    resolved: Option<&crate::paint::ResolvedImagePayload>,
) -> Option<crate::paint::ResolvedImagePayload> {
    use crate::model::style::ImageFillMode;
    use crate::paint::{ResolvedImageKind, ResolvedImagePayload};
    if !image.hancom2024_print_bitmap
        || !matches!(
            image.fill_mode,
            None | Some(ImageFillMode::FitToSize | ImageFillMode::Total)
        )
        || resolved.is_some_and(|p| {
            matches!(
                p.kind,
                ResolvedImageKind::OutputDevice | ResolvedImageKind::BakedWatermark
            )
        })
    {
        return None;
    }
    let original = image.data.as_deref()?;
    if super::detect_image_mime_type(original) != "image/jpeg" {
        return None;
    }
    let data = resolved.map_or(original, |payload| payload.data.as_slice());
    let header = source_header(data)?;
    if image.original_size_hu != Some((header.width, header.height)) {
        return None;
    }
    let (left, top, right, bottom) = image.crop?;
    if left < 0
        || top < 0
        || right <= left
        || bottom <= top
        || i64::from(right) > i64::from(header.width)
        || i64::from(bottom) > i64::from(header.height)
    {
        return None;
    }
    let window_header = crate::renderer::image_header::CanvasKitEncodedImageHeader {
        width: (right - left) as u32,
        height: (bottom - top) as u32,
        ..header
    };
    if !window_header.is_within_decode_limits() {
        return None;
    }
    let bounds = [left as u32, top as u32, right as u32, bottom as u32];
    let bytes = super::memoized(super::Conversion::PrintJpegWindow { bounds }, data, || {
        let format = image::ImageFormat::from_mime_type(super::detect_image_mime_type(data))?;
        let mut reader = image::ImageReader::with_format(std::io::Cursor::new(data), format);
        let mut limits = image::Limits::default();
        limits.max_image_width = Some(crate::renderer::image_header::CANVASKIT_MAX_IMAGE_DIMENSION);
        limits.max_image_height =
            Some(crate::renderer::image_header::CANVASKIT_MAX_IMAGE_DIMENSION);
        limits.max_alloc = Some(MAX_SOURCE_PIXELS.saturating_mul(4));
        reader.limits(limits);
        let decoded = reader.decode().ok()?;
        let window = decoded.crop_imm(
            bounds[0],
            bounds[1],
            bounds[2] - bounds[0],
            bounds[3] - bounds[1],
        );
        let mut output = std::io::Cursor::new(Vec::new());
        window.write_to(&mut output, image::ImageFormat::Png).ok()?;
        Some(output.into_inner())
    })?;
    let suppress_effects = resolved.is_some_and(|payload| payload.suppress_effects);
    image.crop = None;
    image.original_size_hu = None;
    Some(ResolvedImagePayload {
        data: bytes,
        mime: "image/png",
        kind: ResolvedImageKind::SourceWindow,
        suppress_effects,
    })
}

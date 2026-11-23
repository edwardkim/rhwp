//! 한컴 Print의 Picture 원본 코덱 지원. 편집 디코딩과 조판 점유는 별개다.

use crate::renderer::render_tree::ImageNode;

/// 한컴 2020/2024 Print는 PCX Picture를 싣지 않는다. 같은 픽셀을 PNG로
/// 공급한 대조군은 같은 프레임에 인쇄하며 96/300 DPI에 따라 달라지지 않는다.
/// 1-bit/8-bit palette PCX 양쪽의 독립 Print 근거는 task_m100_7754_stage2에 있다.
/// 채우기는 별도 전달 경로이므로 Picture의 코덱 계약을 적용하지 않는다.
pub(crate) fn unsupported_picture(image: &ImageNode) -> bool {
    image.fill_mode.is_none()
        && image
            .data
            .as_deref()
            .is_some_and(|bytes| super::detect_image_mime_type(bytes) == "image/x-pcx")
}

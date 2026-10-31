//! NORMAL 사각형의 단색 Print 채움 중첩을 공통 paint 프레임으로 확정한다.
//!
//! 원본 폭 0/60/425/1600 HU의 독립 한컴 2024 Print와 실제 600dpi 그리기
//! 호출을 대조했다. 선을 표시하지 않아도 저장된 폭의 중첩은 유지된다.
//! 근거: mydocs/plans/issue_8153_rectangle_fill_coverage_20261024.md.

use crate::renderer::render_tree::{BoundingBox, RectangleNode};
use crate::renderer::HWPUNIT_PER_INCH;

pub(super) fn frame(bounds: BoundingBox, rect: &RectangleNode, pixel: f64) -> Option<BoundingBox> {
    let width_hu = rect.drawing_border_width_hu?;
    if width_hu <= 0
        || !pixel.is_finite()
        || pixel <= 0.0
        || rect.corner_radius != 0.0
        || rect.transform.has_transform()
        || rect.image_frame_owner.is_some()
        || rect.gradient.is_some()
        || rect.style.fill_color.is_none()
        || rect.style.pattern.is_some()
        || rect.style.shadow.is_some()
        || rect.style.opacity != 1.0
        || ![bounds.x, bounds.y, bounds.width, bounds.height]
            .into_iter()
            .all(f64::is_finite)
        || bounds.width <= 0.0
        || bounds.height <= 0.0
    {
        return None;
    }
    // 선폭의 장치 양자화는 출력 PNG 배율과 독립적이다. pixel에는 실제
    // 문서 dpi가 반영되어 있으므로 96dpi로 환산한 폭을 다시 나누지 않는다.
    let pen = (f64::from(width_hu) * super::metafile::HANCOM_2024_PRINT_DEVICE_DPI
        / HWPUNIT_PER_INCH)
        .round();
    let overlap = (pen * 0.4).trunc() * pixel;
    if overlap <= 0.0 || !overlap.is_finite() {
        return None;
    }
    Some(BoundingBox::new(
        bounds.x - overlap,
        bounds.y - overlap,
        bounds.width + 2.0 * overlap,
        bounds.height + 2.0 * overlap,
    ))
}

/// 테두리와 자식의 원래 프레임은 유지하고 단색 채움만 별도 op로 옮긴다.
pub(super) fn take_fill(rect: &mut RectangleNode) -> RectangleNode {
    let mut fill = rect.clone();
    fill.style.stroke_color = None;
    fill.style.stroke_width = 0.0;
    fill.drawing_border_width_hu = None;
    rect.style.fill_color = None;
    fill
}

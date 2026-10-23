//! Print 사각형 clip을 소유 종류와 무관한 장치 픽셀 포함 경계에 대응한다.

use crate::renderer::render_tree::BoundingBox;

/// 축 정렬 raster에서만 바깥 정수 경계를 구하고 원래 좌표계로 반환한다.
/// 원본의 물리 clip과 벡터 출력은 바꾸지 않는다. 호출자는 실제 장치 행렬을 전달한다.
pub(crate) fn rectangle(clip: BoundingBox, [a, b, c, d, e, f]: [f64; 6]) -> Option<BoundingBox> {
    if ![clip.x, clip.y, clip.width, clip.height, a, b, c, d, e, f]
        .into_iter()
        .all(f64::is_finite)
        || clip.width <= 0.0
        || clip.height <= 0.0
        || a <= 0.0
        || d <= 0.0
        || b != 0.0
        || c != 0.0
    {
        return None;
    }
    let left = (a * clip.x + e).floor();
    let top = (d * clip.y + f).floor();
    let right = (a * (clip.x + clip.width) + e).ceil();
    let bottom = (d * (clip.y + clip.height) + f).ceil();
    let mapped = BoundingBox::new(
        (left - e) / a,
        (top - f) / d,
        (right - left) / a,
        (bottom - top) / d,
    );
    [mapped.x, mapped.y, mapped.width, mapped.height]
        .into_iter()
        .all(f64::is_finite)
        .then_some(mapped)
}

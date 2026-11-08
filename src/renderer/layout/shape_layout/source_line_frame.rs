//! 저장 아핀의 단일 축과 공통 물리 상자가 이미 정한 직선 방향.

use crate::model::shape::LineShape;

/// 축을 접은 원본 행렬의 실제 선이 저장 물리 상자와 일치하는데,
/// 회전 메타데이터를 다시 적용하면 다른 축의 상자가 되는 경우다.
/// 일반 회전·대각선·행렬 없는 입력은 이 계약으로 인증하지 않는다.
pub(super) fn owns_axis_rotation(line: &LineShape) -> bool {
    let sa = &line.drawing.shape_attr;
    if sa.raw_rendering.is_empty()
        || sa.rotation_angle == 0
        || line.common.width >= 0x8000_0000
        || line.common.height >= 0x8000_0000
        || (line.common.width == 0 && line.common.height == 0)
        || ![
            sa.render_sx,
            sa.render_b,
            sa.render_tx,
            sa.render_c,
            sa.render_sy,
            sa.render_ty,
        ]
        .iter()
        .all(|value| value.is_finite())
    {
        return false;
    }
    // 단순히 작은 폭인 선을 고르지 않는다. 원본 행렬이 출력 축 하나를
    // 정확히 접었다는 계약이 있어야 메타데이터와 실제 점의 소유를 구별한다.
    if !((sa.render_sx == 0.0 && sa.render_b == 0.0) || (sa.render_c == 0.0 && sa.render_sy == 0.0))
    {
        return false;
    }
    let dx = f64::from(line.end.x) - f64::from(line.start.x);
    let dy = f64::from(line.end.y) - f64::from(line.start.y);
    let width = (sa.render_sx * dx + sa.render_b * dy).abs();
    let height = (sa.render_c * dx + sa.render_sy * dy).abs();
    // 선의 0축은 공통 상자에서 1HU sentinel로 보존되기도 한다.
    // 정수 HWPUNIT의 한 단위만 허용하며 화면 픽셀 허용치로 판정하지 않는다.
    let closes = |w: f64, h: f64| {
        (w - f64::from(line.common.width)).abs() <= 1.0
            && (h - f64::from(line.common.height)).abs() <= 1.0
    };
    let angle = f64::from(sa.rotation_angle).to_radians();
    let (sin, cos) = angle.sin_cos();
    width.is_finite()
        && height.is_finite()
        && closes(width, height)
        && !closes(
            cos.abs() * width + sin.abs() * height,
            sin.abs() * width + cos.abs() * height,
        )
}

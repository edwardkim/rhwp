//! 묶음 경계와 자식 저장 행렬이 함께 사용하는 원본 → 현재 프레임 변환.

use crate::model::shape::ShapeObject;

/// 최상위 도형의 표시 중심은 유지하고, 저장 프레임만 회전 후 외곽 크기로 맞춘다.
/// 원본 꼭짓점과 current 크기는 회전 전 로컬 좌표계이므로 변경하지 않는다.
pub(super) fn refresh_top_level_frame(shape: &mut ShapeObject) {
    use crate::model::shape::{HorzAlign, VertAlign};

    let sa = shape.shape_attr();
    let (w, h) = (sa.current_width, sa.current_height);
    let (bw, bh) =
        crate::document_core::DocumentCore::picture_rotated_bounds(w, h, sa.rotation_angle);
    if w == 0 || h == 0 {
        return;
    }
    let common = shape.common_mut();
    // 가운데 정렬은 프레임 크기 변화가 이미 정렬 원점에 반영된다.
    if common.horz_align != HorzAlign::Center {
        let delta = (i64::from(common.width) - i64::from(bw)) / 2;
        common.horizontal_offset =
            (i64::from(common.horizontal_offset as i32) + delta) as i32 as u32;
    }
    if common.vert_align != VertAlign::Center {
        let delta = (i64::from(common.height) - i64::from(bh)) / 2;
        common.vertical_offset = (i64::from(common.vertical_offset as i32) + delta) as i32 as u32;
    }
    common.width = bw;
    common.height = bh;
    let sa = shape.shape_attr_mut();
    sa.rotation_center.x = (bw / 2) as i32;
    sa.rotation_center.y = (bh / 2) as i32;
}

/// 실제 변환 변경에서만 저장 캐시를 비우며, 무변경 속성 되먹임은 원본 행렬을 보존한다.
pub(super) fn finish_top_level_edit(
    shape: &mut ShapeObject,
    before: (u32, u32, u32, u32, u32, u32, i16, bool, bool),
) {
    if matches!(shape, ShapeObject::Group(_)) {
        return;
    }
    if shape.shape_attr().rotation_angle != before.6 {
        refresh_top_level_frame(shape);
    }
    if super::super::common::shape_transform_fingerprint(shape.common(), shape.shape_attr())
        != before
    {
        let sa = shape.shape_attr_mut();
        sa.raw_rendering.clear();
        sa.render_tx = 0.0;
        sa.render_ty = 0.0;
        sa.render_sx = 1.0;
        sa.render_sy = 1.0;
        sa.render_b = 0.0;
        sa.render_c = 0.0;
    }
}

pub(super) struct Placement {
    pub translation: (f64, f64),
    pub scale: [f64; 6],
    pub rotation: [f64; 6],
    original_size: (f64, f64),
}

impl Placement {
    pub(super) fn bounds(&self) -> (f64, f64, f64, f64) {
        let (w, h) = self.original_size;
        let [sx, _, _, _, sy, _] = self.scale;
        let [a, b, _, c, d, _] = self.rotation;
        [(0.0, 0.0), (w, 0.0), (w, h), (0.0, h)]
            .into_iter()
            .map(|(x, y)| {
                (
                    a * sx * x + b * sy * y + self.translation.0,
                    c * sx * x + d * sy * y + self.translation.1,
                )
            })
            .fold(
                (
                    f64::INFINITY,
                    f64::INFINITY,
                    f64::NEG_INFINITY,
                    f64::NEG_INFINITY,
                ),
                |(left, top, right, bottom), (x, y)| {
                    (left.min(x), top.min(y), right.max(x), bottom.max(y))
                },
            )
    }
}

/// 최상위 프레임 중심을 유지하며 원본 좌표에 크기·대칭·회전을 차례로 적용한다.
pub(super) fn placement(child: &ShapeObject, x: f64, y: f64) -> Placement {
    let common = child.common();
    let sa = child.shape_attr();
    let (frame_w, frame_h) = (common.width as f64, common.height as f64);
    let positive = |value: u32, fallback: f64| if value > 0 { value as f64 } else { fallback };
    let cur_w = positive(sa.current_width, frame_w);
    let cur_h = positive(sa.current_height, frame_h);
    let original_w = positive(sa.original_width, cur_w);
    let original_h = positive(sa.original_height, cur_h);
    // 수평·수직 선의 길이가 없는 축은 단위 배율로 보존한다.
    let scale_x = if original_w > 0.0 {
        cur_w / original_w
    } else {
        1.0
    };
    let scale_y = if original_h > 0.0 {
        cur_h / original_h
    } else {
        1.0
    };
    let fx = if sa.horz_flip { -1.0 } else { 1.0 };
    let fy = if sa.vert_flip { -1.0 } else { 1.0 };
    let (sin, cos) = (sa.rotation_angle as f64).to_radians().sin_cos();
    let (hx, hy) = (fx * cur_w / 2.0, fy * cur_h / 2.0);
    Placement {
        translation: (
            x + frame_w / 2.0 - (cos * hx - sin * hy),
            y + frame_h / 2.0 - (sin * hx + cos * hy),
        ),
        scale: [fx * scale_x, 0.0, 0.0, 0.0, fy * scale_y, 0.0],
        rotation: [cos, -sin, 0.0, sin, cos, 0.0],
        original_size: (original_w, original_h),
    }
}

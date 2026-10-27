//! HWP/HWPX가 공유하는 도형 저장 변환. 원본 좌표 → 크기 → 회전 → 이동 순서다.

use crate::model::shape::ShapeComponentAttr;

pub(super) type Matrix = [f64; 6];
pub(super) type RenderMatrices = (Matrix, Vec<(Matrix, Matrix)>);
const IDENTITY: Matrix = [1.0, 0.0, 0.0, 0.0, 1.0, 0.0];

/// 원본 행렬이 없는 편집 개체의 변환을 만든다. 기존 HWP 저장 계약을 양 형식에 적용한다.
pub(super) fn generated(attr: &ShapeComponentAttr) -> RenderMatrices {
    let (trans, scale, rotation) = if attr.rotation_angle.rem_euclid(360) != 0 {
        (
            [
                1.0,
                0.0,
                attr.offset_x as f64,
                0.0,
                1.0,
                attr.offset_y as f64,
            ],
            shape_scale_matrix(attr),
            shape_rotation_matrix(attr),
        )
    } else if has_explicit_rendering_matrix(attr) {
        (
            [1.0, 0.0, attr.render_tx, 0.0, 1.0, attr.render_ty],
            [
                attr.render_sx,
                attr.render_b,
                0.0,
                attr.render_c,
                attr.render_sy,
                0.0,
            ],
            IDENTITY,
        )
    } else {
        // 위치의 권위는 명시적 render_tx/ty다. offset을 임의로 이동 행렬로 승격하지 않는다.
        // 크기는 current/original로 표현하므로 회전 없는 폴백에 배율을 중복 기록하지 않는다.
        (IDENTITY, attr.local_reflection_matrix(), IDENTITY)
    };
    let mut pairs = vec![(scale, rotation)];
    // 한컴 HWP5 저장본의 행렬 쌍 개수는 그룹 깊이 + 1이다.
    pairs.extend((1..attr.group_level.saturating_add(1)).map(|_| (IDENTITY, IDENTITY)));
    (trans, pairs)
}

fn has_explicit_rendering_matrix(attr: &ShapeComponentAttr) -> bool {
    const EPSILON: f64 = 0.000_001;

    (attr.render_sx - 1.0).abs() > EPSILON
        || attr.render_b.abs() > EPSILON
        || attr.render_tx.abs() > EPSILON
        || attr.render_c.abs() > EPSILON
        || (attr.render_sy - 1.0).abs() > EPSILON
        || attr.render_ty.abs() > EPSILON
}

fn shape_scale_matrix(attr: &ShapeComponentAttr) -> [f64; 6] {
    let sx = if attr.original_width > 0 && attr.current_width > 0 {
        attr.current_width as f64 / attr.original_width as f64
    } else {
        1.0
    };
    let sy = if attr.original_height > 0 && attr.current_height > 0 {
        attr.current_height as f64 / attr.original_height as f64
    } else {
        1.0
    };
    let reflection = attr.local_reflection_matrix();
    [
        sx * reflection[0],
        0.0,
        sx * reflection[2],
        0.0,
        sy * reflection[4],
        sy * reflection[5],
    ]
}

fn shape_rotation_positive_correction(attr: &ShapeComponentAttr) -> (f64, f64) {
    let width = attr.current_width as f64;
    let height = attr.current_height as f64;
    if width <= 0.0 || height <= 0.0 {
        return (0.0, 0.0);
    }

    let theta = (attr.rotation_angle as f64).to_radians();
    let cos = theta.cos();
    let sin = theta.sin();
    let corners = [
        (0.0, 0.0),
        (width * cos, width * sin),
        (-height * sin, height * cos),
        (width * cos - height * sin, width * sin + height * cos),
    ];
    let min_x = corners
        .iter()
        .map(|(x, _)| *x)
        .fold(f64::INFINITY, f64::min);
    let min_y = corners
        .iter()
        .map(|(_, y)| *y)
        .fold(f64::INFINITY, f64::min);
    (-min_x, -min_y)
}

fn shape_rotation_matrix(attr: &ShapeComponentAttr) -> [f64; 6] {
    if attr.rotation_angle.rem_euclid(360) == 0 {
        return [1.0, 0.0, 0.0, 0.0, 1.0, 0.0];
    }

    let theta = (attr.rotation_angle as f64).to_radians();
    let cos = theta.cos();
    let sin = theta.sin();
    let (tx, ty) = shape_rotation_positive_correction(attr);

    [
        cos,
        -sin,
        tx - attr.offset_x as f64,
        sin,
        cos,
        ty - attr.offset_y as f64,
    ]
}

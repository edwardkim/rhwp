//! 그림의 외접 프레임과 회전 전 출력 크기를 구분한다.
use super::render_tree::{BoundingBox, ShapeTransform};
use crate::model::image::Picture;

/// 그림 좌표계의 viewport와 그 안에서 시작하는 bitmap 원점. 둘은 다른 값이다.
#[derive(Debug, Clone, Copy, serde::Serialize)]
pub struct PictureSourceRect {
    min_x: f64,
    min_y: f64,
    width: f64,
    height: f64,
    origin_x: f64,
    origin_y: f64,
}

pub(crate) fn picture_source_rect(picture: &Picture) -> Option<PictureSourceRect> {
    let shape = &picture.shape_attr;
    if shape.group_level != 0
        || shape.original_width == 0
        || shape.original_height == 0
        || picture
            .border_x
            .iter()
            .chain(&picture.border_y)
            .all(|x| *x == 0)
    {
        return None;
    }
    let [x0, y0, x1, y1] = picture.border_x.map(f64::from);
    let [x2, y2, x3, y3] = picture.border_y.map(f64::from);
    let w = f64::from(shape.original_width);
    let h = f64::from(shape.original_height);
    let min_x = x0.min(x1).min(x2).min(x3);
    let min_y = y0.min(y1).min(y2).min(y3);
    let width = x0.max(x1).max(x2).max(x3) - min_x;
    let height = y0.max(y1).max(y2).max(y3) - min_y;
    if x0 == 0.0 && y0 == 0.0 && min_x == 0.0 && min_y == 0.0 && width == w && height == h {
        return None;
    }
    Some(PictureSourceRect {
        min_x: min_x / w,
        min_y: min_y / h,
        width: width / w,
        height: height / h,
        origin_x: x0 / w,
        origin_y: y0 / h,
    })
}

pub(crate) struct PicturePaintGeometry {
    pub bitmap: BoundingBox,
    pub viewport: BoundingBox,
    pub clip: Option<[[f64; 2]; 4]>,
}

/// 이미 여백과 회전 전 크기를 해소한 사각형에서 실제 그리기와 viewport를 구한다.
/// 논리 개체 틀은 수정하지 않으며, viewport의 원점과 bitmap의 자체 원점을 각각 소비한다.
pub(crate) fn source_geometry(
    base: BoundingBox,
    transform: ShapeTransform,
    source: Option<PictureSourceRect>,
) -> PicturePaintGeometry {
    let Some(s) = source else {
        return PicturePaintGeometry {
            bitmap: base,
            viewport: base,
            clip: None,
        };
    };
    let (sin, cos) = transform.rotation.to_radians().sin_cos();
    let offset = |x: f64, y: f64| {
        let x = x * base.width * if transform.horz_flip { -1.0 } else { 1.0 };
        let y = y * base.height * if transform.vert_flip { -1.0 } else { 1.0 };
        [cos * x - sin * y, sin * x + cos * y]
    };
    let w = base.width * s.width;
    let h = base.height * s.height;
    let [dx, dy] = offset(
        s.min_x + s.origin_x + (s.width - 1.0) / 2.0,
        s.min_y + s.origin_y + (s.height - 1.0) / 2.0,
    );
    let bitmap = BoundingBox::new(
        base.x + (base.width - w) / 2.0 + dx,
        base.y + (base.height - h) / 2.0 + dy,
        w,
        h,
    );
    let [vx, vy] = offset(
        s.min_x + (s.width - 1.0) / 2.0,
        s.min_y + (s.height - 1.0) / 2.0,
    );
    let cx = base.x + base.width / 2.0 + vx;
    let cy = base.y + base.height / 2.0 + vy;
    let viewport = BoundingBox::new(cx - w / 2.0, cy - h / 2.0, w, h);
    // Print의 clip은 회전 영역의 계단형 스캔 변환이다. 외접 AABB로 대체하면
    // viewport 모서리 바깥의 bitmap까지 보이므로 실제 네 꼭짓점을 전달한다.
    let clip = [
        [-w / 2.0, -h / 2.0],
        [w / 2.0, -h / 2.0],
        [w / 2.0, h / 2.0],
        [-w / 2.0, h / 2.0],
    ]
    .map(|[x, y]| [cx + cos * x - sin * y, cy + sin * x + cos * y]);
    PicturePaintGeometry {
        bitmap,
        viewport,
        clip: Some(clip),
    }
}

/// 저장본은 실제 합성 행렬이 그림의 방향을 정한다.
/// 편집 중에는 아직 저장 행렬이 없으므로 편집 플래그를 사용한다.
pub(crate) fn picture_transform(picture: &Picture) -> ShapeTransform {
    let shape = &picture.shape_attr;
    let mut transform = crate::renderer::layout::extract_picture_transform(picture);
    if shape.raw_rendering.is_empty() || shape.group_level != 0 {
        return transform;
    }
    // imgRect는 그림의 저장 경계다. 꼭짓점 순서를 뒤집은 입력을 이미지
    // 반전으로 추정하지 않고, 실제 행렬이 변환한 두 기저축을 사용한다.
    let u = [shape.render_sx, shape.render_c];
    let v = [shape.render_b, shape.render_sy];
    let (sin, cos) = transform.rotation.to_radians().sin_cos();
    // 직교·축 정렬 계약이 성립하는 사각형만 분해한다. 전단·묶음 행렬을
    // 두 개의 flip 플래그로 근사하지 않으며 기존 별도 경로를 유지한다.
    // 물리 bbox는 양수 길이다. 저장 current size의 음수 축을 행렬에서 함께
    // 정규화해야 프레임의 방향을 그림 자체의 반전으로 두 번 해석하지 않는다.
    let frame_x = if (shape.current_width as i32) < 0 {
        -1.0
    } else {
        1.0
    };
    let frame_y = if (shape.current_height as i32) < 0 {
        -1.0
    } else {
        1.0
    };
    let along_u = (u[0] * cos + u[1] * sin) * frame_x;
    let across_u = -u[0] * sin + u[1] * cos;
    let along_v = (-v[0] * sin + v[1] * cos) * frame_y;
    let across_v = v[0] * cos + v[1] * sin;
    if ![along_u, across_u, along_v, across_v]
        .iter()
        .all(|x| x.is_finite())
        || along_u == 0.0
        || along_v == 0.0
        || across_u.abs() > along_u.abs() * 1e-6
        || across_v.abs() > along_v.abs() * 1e-6
    {
        return transform;
    }
    transform.horz_flip = along_u < 0.0;
    transform.vert_flip = along_v < 0.0;
    transform
}

/// 저장·편집 경로가 기록한 회전 전 크기와 회전 후 외접 크기의 비율.
/// 묶음의 독립 행렬이나 다른 크기 계약에는 이 비율을 추정하지 않는다.
pub(crate) fn picture_pre_rotation_ratio(picture: &Picture) -> Option<[f64; 2]> {
    let shape = &picture.shape_attr;
    let angle = shape.rotation_angle.rem_euclid(360);
    let (w, h) = (
        (shape.current_width as i32).unsigned_abs() as f64,
        (shape.current_height as i32).unsigned_abs() as f64,
    );
    let (fw, fh) = (picture.common.width as f64, picture.common.height as f64);
    if angle == 0 || w <= 0.0 || h <= 0.0 || fw <= 0.0 || fh <= 0.0 {
        return None;
    }
    let (sin, cos) = (angle as f64).to_radians().sin_cos();
    let expected_w = w * cos.abs() + h * sin.abs();
    let expected_h = w * sin.abs() + h * cos.abs();
    // 저장 크기는 정수 HWPUNIT이다. 저장 시 반올림 한 단위까지의 차이만 허용한다.
    if (expected_w - fw).abs() > 1.0 || (expected_h - fh).abs() > 1.0 {
        return None;
    }
    Some([w / fw, h / fh])
}

pub(crate) fn pre_rotation_bbox(
    frame: &BoundingBox,
    transform: ShapeTransform,
    ratio: Option<[f64; 2]>,
) -> BoundingBox {
    let Some([rx, ry]) = ratio else {
        // 이전 생산자와 묶음 개체의 기존 계약은 유지한다.
        return transform.effective_image_bbox(frame);
    };
    let width = frame.width * rx;
    let height = frame.height * ry;
    BoundingBox::new(
        frame.x + (frame.width - width) / 2.0,
        frame.y + (frame.height - height) / 2.0,
        width,
        height,
    )
}

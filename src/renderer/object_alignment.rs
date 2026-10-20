//! 부동 개체의 기준 영역과 실제 가로 원점을 연결한다.

use crate::model::shape::HorzAlign;

/// 개체가 기준 영역보다 넓어도 중점·우단 정렬과 원본 오프셋을 보존한다.
/// 너비 차이는 여유 공간의 부호 있는 값이며, 개체 폭을 줄이는 예산이 아니다.
pub(crate) fn aligned_horizontal_origin(
    reference_x: f64,
    reference_width: f64,
    object_width: f64,
    alignment: HorzAlign,
    offset: f64,
) -> f64 {
    match alignment {
        HorzAlign::Left | HorzAlign::Inside => reference_x + offset,
        HorzAlign::Center => reference_x + (reference_width - object_width) / 2.0 + offset,
        HorzAlign::Right | HorzAlign::Outside => {
            reference_x + reference_width - object_width - offset
        }
    }
}

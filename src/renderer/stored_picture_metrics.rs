//! 저장 그림 프레임의 물리 크기. 선언 비율과 HWPUNIT을 구분한다.
//!
//! 상대 크기의 선언 값은 기준 영역에 대한 비율이며, 저장 줄과 그림의
//! 현재 크기는 HWPUNIT이다. 저장 흐름을 소비할 때 두 단위를 비교하거나
//! 상대 비율을 길이로 변환하지 않는다.

use crate::model::image::Picture;
use crate::model::shape::SizeCriterion;

pub(crate) fn stored_picture_size_hu(picture: &Picture) -> (i32, i32) {
    let dimension = |declared: u32, current: u32, criterion: SizeCriterion| match criterion {
        SizeCriterion::Absolute => declared as i32,
        _ => current as i32,
    };
    (
        dimension(
            picture.common.width,
            picture.shape_attr.current_width,
            picture.common.width_criterion,
        ),
        dimension(
            picture.common.height,
            picture.shape_attr.current_height,
            picture.common.height_criterion,
        ),
    )
}

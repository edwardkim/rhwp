//! 글자처럼 그림이 줄 구성에 제공하는 HWPUNIT 상자 크기.
//!
//! 용지·쪽 상대 비율로 확대된 인쇄 잉크와 줄 상자의 기준선 정렬 크기를 구분한다.
//! 저장 줄과 재조판이 소비하는 동일한 크기를 실제 그림 배치에도 전달한다.

use crate::model::image::Picture;

pub(crate) fn inline_picture_size_hu(picture: &Picture) -> (i32, i32) {
    crate::renderer::stored_picture_metrics::stored_picture_size_hu(picture)
}

/// 글줄이 예약하는 그림 상자는 잉크와 바깥 여백을 함께 포함한다.
/// 독립 한컴 재저장본도 위·아래 여백을 포함한 높이로 LineSeg를 발행한다.
/// 실제 잉크 배치는 기존 원점에서 같은 여백 안쪽을 소비한다.
pub(crate) fn inline_picture_occupied_size_hu(picture: &Picture) -> (i32, i32) {
    let (width, height) = inline_picture_size_hu(picture);
    let margin = &picture.common.margin;
    (
        width
            .saturating_add(i32::from(margin.left))
            .saturating_add(i32::from(margin.right)),
        height
            .saturating_add(i32::from(margin.top))
            .saturating_add(i32::from(margin.bottom)),
    )
}

/// HWP 5.0 표 44의 세로 정렬점. 줄 구성과 실제 그림 원점이 같은 값을 사용한다.
pub(crate) fn inline_picture_ascent_ratio(vertical_alignment: u8) -> f64 {
    match vertical_alignment {
        1 => 0.0,
        2 => 0.5,
        3 => 1.0,
        _ => 0.85,
    }
}

//! 글자처럼 그림이 줄 구성에 제공하는 HWPUNIT 상자 크기.
//!
//! 용지·쪽 상대 비율로 확대된 인쇄 잉크와 줄 상자의 기준선 정렬 크기를 구분한다.
//! 저장 줄과 재조판이 소비하는 동일한 크기를 실제 그림 배치에도 전달한다.

use crate::model::image::Picture;

pub(crate) fn inline_picture_size_hu(picture: &Picture) -> (i32, i32) {
    crate::renderer::stored_picture_metrics::stored_picture_size_hu(picture)
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

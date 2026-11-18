//! 저장 줄에 포함된 개체 높이와 쪽 원점을 재구성 사다리에 연결한다.

use crate::model::control::Control;
use crate::model::paragraph::{LineSeg, Paragraph};

/// 저장 줄이 없는 텍스트 문단을 건너뛰어 앞 저장 프레임의 점유 끝을 잇는다.
/// 개체나 편집 줄처럼 전진량을 설명할 수 없는 항목을 만나면 추측하지 않는다.
pub(super) fn preceding_saved_span(
    spans: &[Option<(i32, i32)>],
    reflowed_text_advances: &[Option<i32>],
    index: usize,
) -> Option<(i32, i32)> {
    let mut intervening = 0_i32;
    for previous in (0..index).rev() {
        if let Some((first, end)) = spans[previous] {
            return Some((first, end.saturating_add(intervening)));
        }
        intervening = intervening.saturating_add(reflowed_text_advances[previous]?);
    }
    None
}

/// 한 개의 개체가 유효한 저장 줄 전체를 점유한 경우다.
/// 캡션·편집된 표·수식처럼 점유를 같은 상자로 확인할 수 없으면 제외한다.
pub(super) fn owns_saved_object_line(para: &Paragraph) -> bool {
    crate::renderer::stored_frame_boundary::owns_saved_object_line(para)
}

/// 가시 본문과 작은 TAC 도형이 유효한 저장 글줄의 기준선 상자 안에 함께 들어간다.
pub(super) fn inline_graphics_fit_saved_text_line(para: &Paragraph, alignment: u8) -> bool {
    crate::renderer::stored_frame_boundary::inline_graphics_fit_saved_text_line(para, alignment)
}

/// 이전 저장 줄 상자가 본문을 채우고 다음 저장 원점이 위로 돌아간 경계다.
/// 큰 개체는 마지막 줄의 시작이 낮아도 그 높이가 쪽 끝까지 점유할 수 있다.
pub(super) fn saved_positive_frame_reset(
    previous: Option<(i32, i32)>,
    first: &LineSeg,
    body_height: i32,
) -> bool {
    crate::renderer::stored_frame_boundary::saved_positive_frame_reset(previous, first, body_height)
}

//! 원문 scalar 범위와 인라인 개체를 포함한 일반 글자의 논리 주소를 구분한다.

use crate::model::paragraph::Paragraph;

/// 같은 원문 위치의 인라인 개체 뒤에 표시되는 일반 글자의 시작 주소다.
/// 제어 순서 중간의 삽입 캐럿이나 자동 번호 자리표의 주소로 사용하지 않는다.
pub(crate) fn text_logical_starts(para: &Paragraph) -> Vec<usize> {
    let text_len = para.text.chars().count();
    let mut slots = vec![0usize; text_len + 1];
    for (control, position) in para.controls.iter().zip(para.control_text_positions()) {
        if control.is_logical_inline() {
            slots[position.min(text_len)] += 1;
        }
    }
    let mut preceding = 0;
    for (source, slots) in slots.iter_mut().enumerate() {
        preceding += *slots;
        *slots = source + preceding;
    }
    slots
}

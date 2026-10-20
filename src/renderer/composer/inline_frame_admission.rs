//! 저장 줄이 없는 인라인 호스트를 측정·배치의 같은 프레임에 연결한다.

use super::{line_breaking, Control, Paragraph};

pub(super) fn supports_uncached_inline_host(para: &Paragraph) -> bool {
    if !crate::renderer::para_has_no_stored_line_segs(para) {
        return false;
    }
    let mut has_inline_object = false;
    for control in &para.controls {
        match control {
            Control::Equation(equation) if equation.common.treat_as_char => {
                has_inline_object = true;
            }
            Control::Picture(picture) if picture.common.treat_as_char => {
                has_inline_object = true;
            }
            marker if line_breaking::control_is_width_neutral_marker(marker) => {}
            // 표·부동 개체·중첩 흐름은 각각의 기존 소유권 계약을 따른다.
            _ => return false,
        }
    }
    has_inline_object
}

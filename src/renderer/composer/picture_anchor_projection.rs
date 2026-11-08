//! 그림 앵커 조회는 원본 번호와 문자 축을 보존하는 전폭 채움이다.

use crate::model::{control::Control, paragraph::Paragraph};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum FramePurpose {
    Body,
    PictureAnchor,
}

/// 선두 Endnote의 표시 prefix만 별도 점유로 전달한다.
/// 중간 번호나 다른 nested flow의 소유를 이 조회로 가져오지 않는다.
pub(super) fn admits(paragraph: &Paragraph) -> bool {
    if paragraph.text.is_empty() {
        return false;
    }
    let positions = paragraph.control_text_positions();
    let mut floating_pictures = 0;
    for (index, control) in paragraph.controls.iter().enumerate() {
        match control {
            Control::Picture(picture) if !picture.common.treat_as_char => {
                floating_pictures += 1;
            }
            Control::Picture(picture) if picture.common.treat_as_char => {}
            Control::Equation(equation) if equation.common.treat_as_char => {}
            Control::Endnote(_) if positions.get(index) == Some(&0) => {}
            other if super::line_breaking::control_is_width_neutral_marker(other) => {}
            _ => return false,
        }
    }
    floating_pictures == 1
}

//! 재조판한 TAC 개체 줄 뒤의 보통 앵커 줄 범위를 한 번 정한다.

use crate::model::{control::Control, paragraph::Paragraph, shape::TextWrap};

/// 저장 쪽 경계를 재해석하지 않고 현재 구성한 줄 소유만 사용한다.
pub(crate) fn composed_object_tail(para: &Paragraph) -> Option<std::ops::Range<usize>> {
    if !crate::renderer::para_has_composed_object_rows(para) {
        return None;
    }
    let mut last_owned = None;
    for (index, control) in para.controls.iter().enumerate() {
        let common = match control {
            Control::Table(table) if table.common.treat_as_char => {
                let row = crate::renderer::layout::control_line_seg_index(para, index)?;
                last_owned = Some(last_owned.map_or(row, |last: usize| last.max(row)));
                continue;
            }
            Control::Table(table) => &table.common,
            Control::Shape(shape) => shape.common(),
            Control::Picture(picture) => &picture.common,
            marker if super::control_is_width_neutral_marker(marker) => continue,
            _ => return None,
        };
        // 흐름을 소비하는 다른 개체의 소유 줄은 여기서 다시 전진시키지 않는다.
        if common.treat_as_char
            || !matches!(
                common.text_wrap,
                TextWrap::BehindText | TextWrap::InFrontOfText
            )
        {
            return None;
        }
    }
    let start = last_owned?.checked_add(1)?;
    (start < para.line_segs.len()).then_some(start..para.line_segs.len())
}

//! 저장 쪽 경계에서 이미 감춘 두 빈 줄과 그 뒤 실제 줄 상자의 구분.

use crate::model::paragraph::{ColumnBreakType, LineSeg, Paragraph};

fn stored_empty_line(paragraph: &Paragraph) -> Option<&LineSeg> {
    if !paragraph.controls.is_empty()
        || !paragraph
            .text
            .chars()
            .all(|c| c.is_control() || c.is_whitespace())
        || paragraph.column_type != ColumnBreakType::None
        || paragraph.stored_text_partition_is_dirty()
        || paragraph.cell_format_vpos_dirty
    {
        return None;
    }
    let [line] = paragraph.line_segs.as_slice() else {
        return None;
    };
    (line.line_height > 0
        && line.line_spacing >= 0
        && line.segment_width > 0
        && line.is_first_segment()
        && line.is_last_segment()
        && line.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0)
        .then_some(line)
}

/// 숨긴 줄은 앞 쪽의 같은 끝 원점에 남고, 첫 실제 줄은 새 쪽의 0에서 재개한다.
/// 호출자는 비편집 저장 조판인지 확인한다. 원본 좌표를 새 줄의 px 위치로 고정하지 않는다.
pub(crate) fn has_two_suppressed_before(
    paragraphs: &[Paragraph],
    paragraph_index: usize,
    body_height_hu: i32,
) -> bool {
    if paragraph_index < 2 || body_height_hu <= 0 {
        return false;
    }
    let Some(current) = paragraphs.get(paragraph_index).and_then(stored_empty_line) else {
        return false;
    };
    let (Some(first), Some(second)) = (
        paragraphs
            .get(paragraph_index - 2)
            .and_then(stored_empty_line),
        paragraphs
            .get(paragraph_index - 1)
            .and_then(stored_empty_line),
    ) else {
        return false;
    };
    current.vertical_pos == 0
        && first.vertical_pos > 0
        && first.vertical_pos == second.vertical_pos
        && [first, second].iter().all(|line| {
            line.column_start == current.column_start
                && line.segment_width == current.segment_width
                && line
                    .vertical_pos
                    .checked_add(line.line_height)
                    .is_some_and(|end| end > body_height_hu)
        })
}

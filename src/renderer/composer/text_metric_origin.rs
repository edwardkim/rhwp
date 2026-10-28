//! 실제 줄 구성의 출처를 glyph 측정과 최종 run에 함께 전달한다.

use crate::model::paragraph::{LineSeg, Paragraph};

/// 저장 줄을 단순히 읽은 것과 현재 프레임이 새로 발행한 줄을 구분한다.
/// 공백 규칙의 값·글꼴 이름·문서 식별자로 출처를 추측하지 않는다.
pub(super) fn regenerated_row(paragraph: &Paragraph, line_index: usize) -> bool {
    let Some(line) = paragraph.line_segs.get(line_index) else {
        return true;
    };
    paragraph.stored_text_partition_is_dirty()
        || paragraph.layout_object_projection
        || line.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
        || paragraph
            .layout_space_metrics
            .get(line_index)
            .is_some_and(|(start, _)| *start == line.text_start)
}

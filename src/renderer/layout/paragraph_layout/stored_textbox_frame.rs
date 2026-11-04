//! 글상자에서 그대로 수용한 저장 줄은 원본 가로 프레임으로 정렬한다.

use crate::model::paragraph::{LineSeg, Paragraph};
use crate::renderer::composer::ComposedParagraph;

pub(super) fn owns_frame(
    paragraph: Option<&Paragraph>,
    composed: &ComposedParagraph,
    line_index: usize,
) -> bool {
    let Some(paragraph) = paragraph else {
        return false;
    };
    if paragraph.stored_text_partition_is_dirty()
        || paragraph.layout_object_projection
        || paragraph.line_segs.len() != composed.lines.len()
    {
        return false;
    }
    let Some(stored) = paragraph.line_segs.get(line_index) else {
        return false;
    };
    let Some(line) = composed.lines.get(line_index) else {
        return false;
    };
    // 재조판된 폭·시작을 과거 저장 프레임으로 덮지 않는다.
    stored.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0
        && stored.segment_width > 0
        && line.column_start == stored.column_start
        && line.segment_width == stored.segment_width
}

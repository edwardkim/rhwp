//! 저장 전폭 줄과 개체 옆 줄이 공유하는 단 기준 가로 프레임.

use crate::model::paragraph::{LineSeg, Paragraph};
use crate::renderer::layout_frame::ParagraphBox;

/// 원본 줄의 양쪽 끝이 문단 상자와 같으면 그림 배제가 아니라 문단 여백이다.
/// 왼쪽 cs 하나의 일치로 판단하지 않으며, 편집·합성 줄의 기하는 인증하지 않는다.
pub(super) fn is_full_paragraph_row(
    para: &Paragraph,
    line_index: usize,
    column_start: i32,
    segment_width: i32,
    paragraph_box: &ParagraphBox,
) -> bool {
    if !paragraph_box.origin_is_derivable()
        || para.stored_text_partition_is_dirty()
        || para.cell_format_vpos_dirty
    {
        return false;
    }
    let horizontal = paragraph_box.effective();
    para.line_segs.get(line_index).is_some_and(|row| {
        row.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0
            && row.column_start == column_start
            && row.segment_width == segment_width
            && segment_width > 0
            && column_start == horizontal.start
            && column_start.checked_add(segment_width) == Some(horizontal.end)
    })
}

/// 개체 옆 좁은 줄과 대비되는 원본 전폭 줄의 시작·끝이다.
pub(super) fn full_width_line_box(para: &Paragraph) -> Option<(i32, i32)> {
    let original: Vec<_> = para
        .line_segs
        .iter()
        .filter(|seg| seg.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0 && seg.segment_width > 0)
        .filter(|seg| {
            para.line_segs
                .iter()
                .filter(|other| other.vertical_pos == seg.vertical_pos)
                .count()
                == 1
        })
        .collect();
    let widest = original.iter().map(|seg| seg.segment_width).max()?;
    if original.iter().all(|seg| seg.segment_width == widest) {
        return None;
    }
    let full = original
        .iter()
        .filter(|seg| seg.segment_width == widest)
        .min_by_key(|seg| seg.column_start)?;
    Some((full.column_start, full.column_start + full.segment_width))
}

/// 같은 원본의 전폭 줄이 현재 문단 상자를 재현하면 각 저장 줄은 그 단의 좌표다.
/// cs와 여백의 우연한 일치나 넓이가 일정한 저장 줄만으로 인증하지 않는다.
pub(super) fn owns_column_row(
    para: &Paragraph,
    column_start: i32,
    segment_width: i32,
    paragraph_box: &ParagraphBox,
) -> bool {
    if para.stored_text_partition_is_dirty() || para.cell_format_vpos_dirty {
        return false;
    }
    let horizontal = paragraph_box.effective();
    full_width_line_box(para) == Some((horizontal.start, horizontal.end))
        && segment_width > 0
        && column_start >= horizontal.start
        && column_start
            .checked_add(segment_width)
            .is_some_and(|end| end <= horizontal.end)
}

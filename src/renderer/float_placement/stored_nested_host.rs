//! 저장 호스트 줄에서 인증한 중첩 표의 배제 프레임을 측정·배치에 함께 공급한다.

use super::ParagraphFloatPlacement;
use crate::model::{control::Control, paragraph::Paragraph, table::Table};

/// 문단 안의 한 표가 저장 글줄 뒤 또는 양수 오프셋 아래에 놓이는 원본 계약이다.
/// 본체 높이는 실제 행 측정 결과이며 바깥여백은 프레임이 한 번만 더한다.
/// 편집/재조판에 따른 저장 프레임의 무효화는 호출자가 확인한다.
pub(crate) fn stored_nested_host_frame(
    paragraph: &Paragraph,
    table: &Table,
    control: usize,
    origin: f64,
    body_height: f64,
    dpi: f64,
) -> Option<ParagraphFloatPlacement> {
    if paragraph.cell_format_vpos_dirty
        || paragraph
            .controls
            .iter()
            .filter(|control| matches!(control, Control::Table(_)))
            .count()
            != 1
    {
        return None;
    }
    ParagraphFloatPlacement::from_stored_host(paragraph, table, control, origin, body_height, dpi)
        .or_else(|| {
            ParagraphFloatPlacement::from_stored_head_host(
                paragraph,
                table,
                control,
                origin,
                body_height,
                dpi,
            )
        })
        .or_else(|| {
            ParagraphFloatPlacement::from_empty_stored_offset_host(
                paragraph,
                table,
                control,
                origin,
                body_height,
                dpi,
            )
        })
}

/// 배제 표 뒤의 빈 글줄도 칸의 물리 공간을 점유한다. 유효한 저장 사다리의
/// 마지막 줄 상자와 표의 바깥 상자를 함께 닫되, 마지막 줄 뒤 간격은 더하지 않는다.
pub(crate) fn stored_nested_host_content_bottom(
    paragraphs: &[Paragraph],
    host_index: usize,
    frame: ParagraphFloatPlacement,
    dpi: f64,
) -> f64 {
    let origin = paragraphs
        .get(host_index)
        .and_then(|p| p.line_segs.first())
        .map_or(0.0, |line| super::hwpunit_to_px(line.vertical_pos, dpi));
    let object_bottom = origin + frame.occupied_bottom;
    let mut previous = None;
    let mut end = 0_i64;
    for paragraph in paragraphs {
        if paragraph.cell_format_vpos_dirty
            || paragraph.stored_text_partition_is_dirty()
            || paragraph.line_segs.is_empty()
        {
            return object_bottom;
        }
        for line in &paragraph.line_segs {
            if line.vertical_pos < 0
                || line.line_height <= 0
                || line.tag & crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
                || previous.is_some_and(|top| line.vertical_pos < top)
                || (previous.is_some()
                    && (line.is_first_line_of_page() || line.is_first_line_of_column()))
            {
                return object_bottom;
            }
            previous = Some(line.vertical_pos);
            end = end.max(i64::from(line.vertical_pos) + i64::from(line.line_height));
        }
    }
    i32::try_from(end).ok().map_or(object_bottom, |end| {
        object_bottom.max(super::hwpunit_to_px(end, dpi))
    })
}

//! 저장된 글 호스트·표 끝·후속 줄이 같은 원점을 소유하는 관계.

use crate::model::{
    control::Control,
    paragraph::{ColumnBreakType, LineSeg, Paragraph},
    shape::VertRelTo,
};
use crate::renderer::pagination::PageItem;

/// 표 뒤에 남은 호스트 글줄을 보존하면서 후속 앞 간격의 중복 소비만 판정한다.
/// 저장 표의 전체 바깥 끝, 다음 저장 줄, 실제 흐름 커서가 같은 경우에만 적용한다.
pub(crate) fn shared_spacing(
    items: &[PageItem],
    paragraphs: &[Paragraph],
    para_index: usize,
    spacing_before: f64,
    stored_layout: bool,
    current_y: f64,
    dpi: f64,
) -> f64 {
    if !stored_layout
        || !spacing_before.is_finite()
        || spacing_before <= 0.0
        || !current_y.is_finite()
        || !dpi.is_finite()
        || dpi <= 0.0
    {
        return 0.0;
    }
    let [.., PageItem::Table {
        para_index: owner,
        control_index,
    }, PageItem::PartialParagraph {
        para_index: text_owner,
        start_line: 0,
        end_line,
    }] = items
    else {
        return 0.0;
    };
    if owner != text_owner || owner.checked_add(1) != Some(para_index) {
        return 0.0;
    }
    let (Some(host), Some(next)) = (paragraphs.get(*owner), paragraphs.get(para_index)) else {
        return 0.0;
    };
    let [line] = host.line_segs.as_slice() else {
        return 0.0;
    };
    let Some(next_line) = next.line_segs.first() else {
        return 0.0;
    };
    let [Control::Table(table)] = host.controls.as_slice() else {
        return 0.0;
    };
    if *control_index != 0
        || *end_line != host.line_segs.len()
        || host.text.trim().is_empty()
        || !next.controls.is_empty()
        || host.column_type != ColumnBreakType::None
        || next.column_type != ColumnBreakType::None
        || host.stored_text_partition_is_dirty()
        || next.stored_text_partition_is_dirty()
        || host.cell_format_vpos_dirty
        || next.cell_format_vpos_dirty
        || line.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
        || next_line.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
        || line.line_height <= 0
        || next_line.line_height <= 0
        || table.common.treat_as_char
        || !table.common.flow_with_text
        || !super::is_para_topbottom_float(&table.common)
        || table.common.vert_rel_to != VertRelTo::Para
        || table.caption.is_some()
        || table.common.height == 0
    {
        return 0.0;
    }
    let offset = super::signed_hwpunit(table.common.vertical_offset);
    if offset <= 0 {
        return 0.0;
    }
    let bottom = i64::from(line.vertical_pos)
        + i64::from(offset)
        + i64::from(table.outer_margin_top)
        + i64::from(table.common.height)
        + i64::from(table.outer_margin_bottom);
    if bottom != i64::from(next_line.vertical_pos)
        || (current_y * 7200.0 / dpi - bottom as f64).abs() > 1.0
    {
        return 0.0;
    }
    spacing_before
}

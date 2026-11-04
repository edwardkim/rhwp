//! 실제 저장 행 점유의 비끝 경계가 개체 선언과 일치하는 조각 프레임.

use crate::model::{control::Control, paragraph::Paragraph, table::Table};

use super::{
    control_leaves_table_anchor_empty, is_para_topbottom_float, TablePageBreak, VertAlign,
};

pub(super) fn owns_fragment_outer_frame(para: &Paragraph, table: &Table) -> bool {
    if table.page_break != TablePageBreak::RowBreak
        || !is_para_topbottom_float(&table.common)
        || table.common.vert_align != VertAlign::Top
        || table.caption.is_some()
        || !para.text.is_empty()
        || table.row_count < 2
    {
        return false;
    }
    let [host] = para.line_segs.as_slice() else {
        return false;
    };
    if host.tag & crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
        || host.segment_width != 0
        || host.vertical_pos < 0
    {
        return false;
    }
    let mut controls = para
        .controls
        .iter()
        .filter(|control| !control_leaves_table_anchor_empty(control));
    if !matches!(controls.next(), Some(Control::Table(_))) || controls.next().is_some() {
        return false;
    }
    let mut rows = vec![None; usize::from(table.row_count)];
    for cell in &table.cells {
        if cell.row_span != 1 || cell.height >= 0x8000_0000 {
            return false;
        }
        let [paragraph] = cell.paragraphs.as_slice() else {
            return false;
        };
        let [line] = paragraph.line_segs.as_slice() else {
            return false;
        };
        if !paragraph.controls.is_empty()
            || line.tag & crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
            || line.vertical_pos < 0
            || line.line_height <= 0
        {
            return false;
        }
        let padding = cell.effective_padding(&table.padding);
        let content = i64::from(line.vertical_pos)
            + i64::from(line.line_height)
            + i64::from(padding.top)
            + i64::from(padding.bottom);
        let Some(row) = rows.get_mut(usize::from(cell.row)) else {
            return false;
        };
        *row = Some(
            row.unwrap_or(0_i64)
                .max(content.max(i64::from(cell.height))),
        );
    }
    let Some(rows) = rows.into_iter().collect::<Option<Vec<_>>>() else {
        return false;
    };
    let mut prefix = 0_i64;
    rows[..rows.len() - 1]
        .iter()
        .enumerate()
        .any(|(index, height)| {
            prefix += height;
            prefix + i64::from(table.cell_spacing) * index as i64 == i64::from(table.common.height)
        })
}

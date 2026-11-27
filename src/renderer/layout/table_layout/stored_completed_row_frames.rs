//! 내용이 첫 쪽에서 끝나도 원본의 일반 행이 다음 쪽에 소유하는 물리 상자.

use super::{LayoutEngine, ResolvedStyleSet};
use crate::model::{
    control::Control,
    paragraph::{ColumnBreakType, LineSeg, Paragraph},
    shape::VertAlign,
    table::{Table, TablePageBreak},
};
use crate::renderer::{
    composer::control_is_width_neutral_marker,
    float_placement::{
        is_para_topbottom_float, rowbreak_fragment_bottom_inset_hu, StoredRowFrames,
    },
    hwpunit_to_px, px_to_hwpunit,
};

impl LayoutEngine {
    /// 첫 조각의 끝은 원본 본문 하단과 일치하고, 다음 글줄은 잔여 행
    /// 밴드 바로 뒤에 있거나 별도의 명시적 쪽 나누기에서 시작한다.
    pub(crate) fn stored_completed_row_frames(
        &self,
        host: &Paragraph,
        successor: &Paragraph,
        table: &Table,
        styles: &ResolvedStyleSet,
        body_height: f64,
    ) -> Option<StoredRowFrames> {
        let [anchor] = host.line_segs.as_slice() else {
            return None;
        };
        let next = successor.line_segs.first()?;
        if !self.profile.get().hwp5_stored_pagination_layout()
            || self.profile.get().session_edited()
            || host.stored_text_partition_is_dirty()
            || host.cell_format_vpos_dirty
            || successor.stored_text_partition_is_dirty()
            || successor.cell_format_vpos_dirty
            || !host.text.trim().is_empty()
            || !matches!(host.controls.as_slice(), [Control::Table(_)])
            || anchor.vertical_pos <= 0
            || anchor.segment_width != 0
            || next.vertical_pos < 0
            || next.vertical_pos >= anchor.vertical_pos
            || [anchor, next].iter().any(|line| {
                line.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0 || line.line_height <= 0
            })
            || table.row_count != 1
            || table.cells.is_empty()
            || table.common.treat_as_char
            || !is_para_topbottom_float(&table.common)
            || !table.common.flow_with_text
            || table.common.allow_overlap
            || table.common.vert_align != VertAlign::Top
            || table.common.vertical_offset != 0
            || table.layout_page_break() != TablePageBreak::RowBreak
            || table.caption.is_some()
            || table.cell_spacing != 0
            || table.outer_margin_top < 0
            || table.outer_margin_bottom < 0
            || self
                .render_normalization
                .borrow()
                .table_text_reflowed(table)
        {
            return None;
        }
        let opening_hu = i32::try_from(table.common.height).ok()?;
        let top = i64::from(anchor.vertical_pos) + i64::from(table.outer_margin_top);
        let bottom = top + i64::from(opening_hu) + i64::from(table.outer_margin_bottom);
        let body = i64::from(px_to_hwpunit(body_height, self.dpi));
        if opening_hu <= 0
            || (body - bottom - i64::from(rowbreak_fragment_bottom_inset_hu(table))).abs() > 2
        {
            return None;
        }

        let row_height = table.cells.iter().map(|cell| cell.height).max()?;
        let remaining_hu = i32::try_from(row_height.checked_sub(table.common.height)?).ok()?;
        let closing_end = i64::from(remaining_hu)
            + i64::from(table.outer_margin_top)
            + i64::from(table.outer_margin_bottom);
        let successor_breaks = successor.column_type == ColumnBreakType::Page
            || styles
                .para_styles
                .get(successor.para_shape_id as usize)
                .is_some_and(|style| style.page_break_before);
        if remaining_hu == 0
            || closing_end >= body
            || if successor_breaks {
                next.vertical_pos != 0
            } else {
                i64::from(next.vertical_pos) != closing_end
            }
        {
            return None;
        }

        let mut opening_cut = Vec::new();
        for index in self.indexed_row_cut_cell_order(table, 0) {
            let cell = &table.cells[index];
            if cell.row != 0
                || cell.row_span != 1
                || cell.text_direction != 0
                || cell.dirty_flag
                || cell.height < table.common.height
            {
                return None;
            }
            let padding = cell.effective_padding(&table.padding);
            let mut last_line_end = 0;
            for paragraph in &cell.paragraphs {
                if paragraph.stored_text_partition_is_dirty()
                    || paragraph.cell_format_vpos_dirty
                    || paragraph.line_segs.is_empty()
                    || !paragraph
                        .controls
                        .iter()
                        .all(control_is_width_neutral_marker)
                {
                    return None;
                }
                for line in &paragraph.line_segs {
                    let start = i64::from(line.vertical_pos);
                    let end = start + i64::from(line.line_height);
                    if line.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
                        || line.line_height <= 0
                        || start < last_line_end
                        || end + i64::from(padding.top) + i64::from(padding.bottom)
                            > i64::from(opening_hu)
                    {
                        return None;
                    }
                    last_line_end = end;
                }
            }
            let units = self.cell_units(cell, table, styles);
            if units.is_empty()
                || units
                    .iter()
                    .any(|unit| unit.reflowed_line_slot || unit.page_frame_reset_before)
            {
                return None;
            }
            opening_cut.push(units.len());
        }
        if opening_cut.len() != table.cells.len() {
            return None;
        }
        Some(StoredRowFrames {
            opening_anchor_hu: anchor.vertical_pos,
            opening_table_top_hu: i32::try_from(top).ok()?,
            opening_cut,
            opening_height: hwpunit_to_px(opening_hu, self.dpi),
            closing_height: hwpunit_to_px(remaining_hu, self.dpi),
            closing_is_blank: true,
        })
    }
}

//! 같은 원본 쪽의 일부 줄을 앞 실제 쪽에서 소비한 셀 컷의 저장 좌표.

use super::{LayoutEngine, ResolvedStyleSet};
use crate::model::{
    paragraph::{ColumnBreakType, LineSeg},
    table::{Cell, Table},
};

impl LayoutEngine {
    /// 실제 컷 유닛이 원본 문단의 쪽 reset 뒤 중간 줄을 가리키는지 인증한다.
    /// 재조판한 문자 분할이나 중첩 개체의 유닛을 저장 줄 번호로 추정하지 않는다.
    pub(crate) fn consumed_stored_cell_frame_prefix(
        &self,
        cell: &Cell,
        table: &Table,
        styles: &ResolvedStyleSet,
        start_unit: usize,
    ) -> Option<i32> {
        let units = self.cell_units(cell, table, styles);
        let unit = units.get(start_unit)?;
        let paragraph = cell.paragraphs.get(unit.para_idx)?;
        if paragraph.stored_text_partition_is_dirty()
            || paragraph.cell_format_vpos_dirty
            || paragraph.column_type != ColumnBreakType::None
            || !paragraph.controls.is_empty()
            || paragraph.source_line_seg_vertical_pos.is_some()
            || unit.vis_end != unit.vis_start.checked_add(1)?
        {
            return None;
        }
        let lines = &paragraph.line_segs;
        let selected = lines.get(unit.vis_start)?;
        if selected.vertical_pos <= 0 {
            return None;
        }
        let first = (1..unit.vis_start)
            .rev()
            .find(|&index| lines[index].vertical_pos == 0 && lines[index - 1].vertical_pos > 0)?;
        for pair in lines[first..=unit.vis_start].windows(2) {
            if pair.iter().any(|line| {
                line.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
                    || line.line_height <= 0
                    || line.line_spacing < 0
            }) || pair[0]
                .vertical_pos
                .checked_add(pair[0].line_height)?
                .checked_add(pair[0].line_spacing)?
                != pair[1].vertical_pos
            {
                return None;
            }
        }
        Some(selected.vertical_pos)
    }
}

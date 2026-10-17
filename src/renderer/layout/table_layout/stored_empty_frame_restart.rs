//! 선언 최소 셀 높이와 별개로 원본 빈 줄이 여는 쪽 프레임을 확인한다.

use super::{CellUnit, LayoutEngine, ResolvedStyleSet};
use crate::model::{
    paragraph::LineSeg,
    table::{Cell, Table, TablePageBreak},
};
use crate::renderer::hwpunit_to_px;

fn authentic(line: &LineSeg) -> bool {
    line.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0
        && line.vertical_pos >= 0
        && line.line_height > 0
        && line.line_spacing >= 0
}

impl LayoutEngine {
    /// 첫 제목과 다음 원본 인라인 상자가 서로 다른 쪽 원점0을 소유한다.
    /// 뒤 상자의 저장 높이와 다음 줄 전진이 정확할 때만 제목 조각의
    /// 최소 물리 공간을 보존한다. 문단마다 임의로0인 로컬 좌표는 제외한다.
    pub(crate) fn stored_heading_before_inline_frame_cut(
        &self,
        table: &Table,
        row: usize,
        start: &[usize],
        end: &[usize],
        styles: &ResolvedStyleSet,
    ) -> bool {
        use crate::model::control::Control;
        let profile = self.profile.get();
        if !profile.hwp5_stored_pagination_layout()
            || profile.session_edited()
            || self
                .render_normalization
                .borrow()
                .table_text_reflowed(table)
            || !start.is_empty()
            || end != [1]
            || table.common.treat_as_char
            || table.page_break != TablePageBreak::RowBreak
            || table.row_count <= 1
        {
            return false;
        }
        let owners = Self::row_cut_cell_order(table, row);
        let [index] = owners.as_slice() else {
            return false;
        };
        let cell = &table.cells[*index];
        if usize::from(cell.row) != row
            || cell.row_span != 1
            || cell.col_span != table.col_count
            || cell.dirty_flag
            || cell.paragraphs.len() < 3
        {
            return false;
        }
        let (heading, host, successor) = (
            &cell.paragraphs[0],
            &cell.paragraphs[1],
            &cell.paragraphs[2],
        );
        if [heading, host, successor].iter().any(|paragraph| {
            paragraph.stored_text_partition_is_dirty() || paragraph.cell_format_vpos_dirty
        }) || heading.text.trim().is_empty()
            || !heading.controls.is_empty()
            || !host.text.trim().is_empty()
            || !successor.controls.is_empty()
        {
            return false;
        }
        let ([heading_line], [host_line], Some(after)) = (
            heading.line_segs.as_slice(),
            host.line_segs.as_slice(),
            successor.line_segs.first(),
        ) else {
            return false;
        };
        let [Control::Table(child)] = host.controls.as_slice() else {
            return false;
        };
        if ![heading_line, host_line, after]
            .iter()
            .all(|line| authentic(line))
            || heading_line.vertical_pos != 0
            || host_line.vertical_pos != 0
            || !child.common.treat_as_char
            || child.caption.is_some()
            || i64::from(host_line.line_height)
                != i64::from(child.common.height)
                    + i64::from(child.outer_margin_top)
                    + i64::from(child.outer_margin_bottom)
            || i64::from(after.vertical_pos)
                != i64::from(host_line.line_height) + i64::from(host_line.line_spacing)
        {
            return false;
        }
        let units = self.cell_units(cell, table, styles);
        units
            .first()
            .is_some_and(|unit| unit.para_idx == 0 && unit.vis_start == 0 && unit.vis_end == 1)
            && units
                .get(1)
                .is_some_and(|unit| unit.para_idx == 1 && unit.page_frame_reset_before)
    }

    /// 다음 저장 줄이 빈 재시작 줄의 전체 전진량을 소비한다.
    /// 같은 셀의 원본 되감김 유닛만 허용하며 로컬 빈 장식 줄에는 적용하지 않는다.
    pub(super) fn stored_empty_restart_owns_advance(cell: &Cell, unit: &CellUnit) -> bool {
        if cell.dirty_flag
            || !unit.empty_spacer
            || !unit.page_frame_reset_before
            || !unit.stored_frame_break_before
            || unit.vis_start != 0
            || unit.vis_end != 1
        {
            return false;
        }
        let Some(paragraph) = cell.paragraphs.get(unit.para_idx) else {
            return false;
        };
        let Some(next) = cell.paragraphs.get(unit.para_idx + 1) else {
            return false;
        };
        if [paragraph, next].iter().any(|paragraph| {
            paragraph.stored_text_partition_is_dirty()
                || paragraph.cell_format_vpos_dirty
                || !paragraph.controls.is_empty()
        }) {
            return false;
        }
        let [line] = paragraph.line_segs.as_slice() else {
            return false;
        };
        next.line_segs.first().is_some_and(|following| {
            authentic(line)
                && authentic(following)
                && line.vertical_pos == 0
                && i64::from(following.vertical_pos)
                    == i64::from(line.line_height) + i64::from(line.line_spacing)
        })
    }

    /// 원본 프레임의 유닛 합이 저장된 마지막 전진 끝을 정확히 닫는지 확인한다.
    /// 셀 선언값은 최소 상자이지 여러 쪽에 걸친 내용의 물리 높이 상한이 아니다.
    pub(super) fn stored_empty_restart_closes_source_flow(
        &self,
        table: &Table,
        cell: &Cell,
        units: &[CellUnit],
        end: usize,
        styles: &ResolvedStyleSet,
    ) -> bool {
        let profile = self.profile.get();
        if !profile.hwp5_stored_pagination_layout()
            || profile.session_edited()
            || self
                .render_normalization
                .borrow()
                .table_text_reflowed(table)
            || table.common.treat_as_char
            || table.page_break != TablePageBreak::RowBreak
            || cell.row_span != 1
            || cell.col != 0
            || cell.col_span != table.col_count
            || cell.paragraphs.iter().any(|paragraph| {
                paragraph.stored_text_partition_is_dirty()
                    || paragraph.cell_format_vpos_dirty
                    || paragraph.line_segs.iter().any(|line| !authentic(line))
            })
        {
            return false;
        }
        let Some(next) = units.get(end) else {
            return false;
        };
        let Some(previous) = end.checked_sub(1).and_then(|index| units.get(index)) else {
            return false;
        };
        if !Self::stored_empty_restart_owns_advance(cell, next)
            || previous.para_idx + 1 != next.para_idx
            || previous.vis_start + 1 != previous.vis_end
        {
            return false;
        }
        let paragraph = &cell.paragraphs[previous.para_idx];
        let Some(line) = paragraph.line_segs.get(previous.vis_start) else {
            return false;
        };
        if !paragraph.controls.is_empty()
            || previous.vis_end != paragraph.line_segs.len()
            || line.vertical_pos <= 0
        {
            return false;
        }
        let start = units[..end]
            .iter()
            .rposition(|unit| unit.page_frame_reset_before)
            .unwrap_or(0);
        let first = &units[start];
        if !cell.paragraphs[first.para_idx]
            .line_segs
            .get(first.vis_start)
            .is_some_and(|line| line.vertical_pos == 0)
        {
            return false;
        }
        let source_end = i64::from(line.vertical_pos)
            + i64::from(line.line_height)
            + i64::from(line.line_spacing);
        let Ok(source_end) = i32::try_from(source_end) else {
            return false;
        };
        let after = styles
            .para_styles
            .get(paragraph.para_shape_id as usize)
            .map_or(0.0, |style| style.spacing_after);
        let occupied = units[start..end]
            .iter()
            .map(|unit| unit.height)
            .sum::<f64>();
        (occupied - hwpunit_to_px(source_end, self.dpi) - after).abs() <= hwpunit_to_px(2, self.dpi)
    }
}

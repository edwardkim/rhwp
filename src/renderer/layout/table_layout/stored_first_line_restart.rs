//! 원본 첫 줄의 공통 쪽 재시작과 물리 종료 간격.

use super::{CellUnit, LayoutEngine, ResolvedStyleSet, TablePageBreak};
use crate::model::shape::TextWrap;
use crate::renderer::hwpunit_to_px;

impl LayoutEngine {
    /// 첫 줄0 다음에도0으로 재시작하는 저장 프레임의 끝 간격이다.
    /// 이미 선택된 컷의 source 첫 줄과 같은 행의 다른 셀 사다리를 확인한다.
    /// 모든 줄이0인 입력이나 control 문단의 로컬 재시작을 프레임으로 승격하지 않는다.
    pub(super) fn native_first_line_restart_cut_closes_frame(
        &self,
        table: &crate::model::table::Table,
        cell: &crate::model::table::Cell,
        units: &[CellUnit],
        start_cut: usize,
        end_cut: usize,
    ) -> bool {
        let profile = self.profile.get();
        let stored_span = cell.row_span > 1 && profile.hwpx_stored_layout();
        if !(profile.hwp5_stored_pagination_layout() || stored_span)
            || profile.session_edited()
            || self
                .render_normalization
                .borrow()
                .table_text_reflowed(table)
            || table.common.treat_as_char
            || !matches!(table.page_break, TablePageBreak::RowBreak)
            || !matches!(table.common.text_wrap, TextWrap::TopAndBottom)
            || start_cut != 0
            || end_cut != 1
            || end_cut >= units.len()
        {
            return false;
        }
        let closing = &units[0];
        if closing.para_idx != 0 || closing.vis_start != 0 || closing.vis_end > 1 {
            return false;
        }
        fn source_restart(cell: &crate::model::table::Cell) -> bool {
            let mut lines = cell
                .paragraphs
                .iter()
                .flat_map(|paragraph| paragraph.line_segs.iter().map(move |seg| (paragraph, seg)));
            let (Some((first_para, first)), Some((next_para, next)), Some((after_para, after))) =
                (lines.next(), lines.next(), lines.next())
            else {
                return false;
            };
            [first_para, next_para, after_para]
                .iter()
                .all(|para| para.controls.is_empty())
                && [first, next, after].iter().all(|seg| {
                    seg.tag & crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0
                })
                && first.vertical_pos == 0
                && next.vertical_pos == 0
                && first.line_height > 0
                && next.line_height > 0
                && i64::from(after.vertical_pos)
                    == i64::from(next.line_height) + i64::from(next.line_spacing.max(0))
        }
        if !source_restart(cell) {
            return false;
        }
        let native_shared_row = profile.hwp5_stored_pagination_layout();
        let shared_restart = if cell.row_span > 1 && !native_shared_row {
            // HWPX 병합 셀은 같은 물리 span의 원본 소유자 둘로 경계를 확인한다.
            // 먼저 끝난 일반 셀은 그 뒤 span의 재시작을 입증하지 않는다.
            let genuine_source = |owner: &crate::model::table::Cell| {
                !owner.dirty_flag
                    && owner.paragraphs.iter().all(|para| {
                        !para.stored_text_partition_is_dirty()
                            && !para.cell_format_vpos_dirty
                            && para.controls.is_empty()
                    })
                    && source_restart(owner)
            };
            genuine_source(cell)
                && table.cells.iter().any(|other| {
                    other.row == cell.row
                        && other.row_span == cell.row_span
                        && other.col != cell.col
                        && genuine_source(other)
                })
        } else {
            let mut siblings = table
                .cells
                .iter()
                .filter(|other| other.row == cell.row && other.col != cell.col)
                .peekable();
            // Native 저장본의 쪽 경계는 같은 시작 행의 셀들이 공유한다.
            // 병합 셀도 일반 형제와 첫 줄 뒤에서 함께 재시작하면 동일한 컷이다.
            // rowspan 크기가 같아야 한다는 가정은 실제로 공유하는 경계를 놓친다.
            // 끝난 셀이나 원점0만 반복하는 셀은 source_restart가 거부한다.
            let genuine_source = |owner: &crate::model::table::Cell| {
                !owner.dirty_flag
                    && owner.paragraphs.iter().all(|para| {
                        !para.stored_text_partition_is_dirty() && !para.cell_format_vpos_dirty
                    })
                    && source_restart(owner)
            };
            genuine_source(cell) && siblings.peek().is_some() && siblings.all(genuine_source)
        };
        if !shared_restart {
            return false;
        }
        // 끝 컷은 첫 source 줄 뒤에 있어야 한다. 재합성으로 유닛 소유가 바뀌면 제외한다.
        let next = &units[end_cut];
        if !(next.para_idx == 0 && next.vis_start == 1 || next.para_idx == 1 && next.vis_start == 0)
        {
            return false;
        }
        true
    }

    pub(super) fn native_first_line_restart_cut_trailing_trim(
        &self,
        table: &crate::model::table::Table,
        cell: &crate::model::table::Cell,
        units: &[CellUnit],
        start_cut: usize,
        end_cut: usize,
        styles: &ResolvedStyleSet,
    ) -> f64 {
        if !self.native_first_line_restart_cut_closes_frame(table, cell, units, start_cut, end_cut)
        {
            return 0.0;
        }
        let closing = &units[0];
        let next = &units[end_cut];
        let para = &cell.paragraphs[0];
        let line_spacing = hwpunit_to_px(para.line_segs[0].line_spacing.max(0), self.dpi);
        let after = if next.para_idx == closing.para_idx {
            0.0
        } else {
            styles
                .para_styles
                .get(para.para_shape_id as usize)
                .map(|style| style.spacing_after)
                .unwrap_or(0.0)
                .max(0.0)
        };
        (line_spacing + after).min(closing.height.max(0.0))
    }
}

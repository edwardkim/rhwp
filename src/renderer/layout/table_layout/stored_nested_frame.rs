//! 저장된 중첩 프레임의 열린 컷은 부모 셀의 종료 안 여백을 소유하지 않는다.

use crate::model::table::{Cell, Table, TablePageBreak};

use super::{CellUnit, LayoutEngine, ResolvedStyleSet};

impl LayoutEngine {
    /// 투명 자식의 이어받은 박스에서 원본 다음 쪽으로 넘어가는 줄간격.
    /// 컷은 이미 정해졌으며 쪽 수용에 쓰는 논리 전진을 바꾸지 않는다.
    pub(super) fn stored_transparent_child_closing_advance(
        &self,
        table: &Table,
        cell: &Cell,
        units: &[CellUnit],
        start: usize,
        end: usize,
        styles: &ResolvedStyleSet,
    ) -> f64 {
        let Some((previous, next)) = end
            .checked_sub(1)
            .and_then(|last| units.get(last).zip(units.get(end)))
        else {
            return 0.0;
        };
        let profile = self.profile.get();
        if start == 0
            || start >= end
            || table.row_count != 1
            || table.col_count != 1
            || table.caption.is_some()
            || profile.session_edited()
            || !(profile.hwpx_stored_layout() || profile.hwp5_stored_pagination_layout())
            || previous.para_idx != next.para_idx
            || previous.vis_end != next.vis_start
            || !self.stored_in_paragraph_unit_is_original_restart(table, cell, next)
            || !self
                .nested_table_ptrs
                .borrow()
                .contains(&(std::ptr::from_ref(table) as usize))
            || Self::transparent_single_cell_wrapper(table, styles).is_none()
        {
            return 0.0;
        }
        self.stored_span_line_tail_spacing(previous)
    }

    pub(super) fn nested_source_frame_bottom_padding(
        &self,
        table: &Table,
        cell: &Cell,
        units: &[CellUnit],
        start: usize,
        end: usize,
        styles: &ResolvedStyleSet,
    ) -> f64 {
        let profile = self.profile.get();
        if !(profile.hwp5_stored_pagination_layout() || profile.hwpx_stored_layout())
            || profile.session_edited()
            || table.layout_page_break() != TablePageBreak::RowBreak
            || cell.row_span != 1
            || cell.dirty_flag
            || self
                .render_normalization
                .borrow()
                .table_text_reflowed(table)
            || end <= start
            || end >= units.len()
        {
            return 0.0;
        }
        let closing = &units[end - 1];
        let next = &units[end];
        // 재귀 셀 유닛은 자식의 실제 저장 reset을 부모 컷에 전달한다.
        // 임의 용량 컷·텍스트 줄·전체 셀 종료에는 이 여백을 회수하지 않는다.
        if !closing.mixed_nested_recursive
            || !next.mixed_nested_recursive
            || !next.stored_frame_break_before
            || closing.para_idx != next.para_idx
            || closing.mixed_nested_source_para_idx != next.mixed_nested_source_para_idx
        {
            return 0.0;
        }
        // 부모 선언이 앞 행 + 자식 첫 상자 + 부모 상하 안 여백으로 닫히면,
        // 이 조각의 아래 안 여백도 원본 물리 상자의 일부다. 열린 프레임이라는
        // 이유로 회수하면 실제 테두리가 그만큼 짧아진다. 별도 종료 증명이 없는
        // 기존 중첩 reset은 아래의 열린 컷 계약을 유지한다.
        if cell.paragraphs[closing.para_idx]
            .controls
            .iter()
            .find_map(|control| match control {
                crate::model::control::Control::Table(child) => self
                    .render_normalization
                    .borrow()
                    .stored_nested_text_frames(child),
                _ => None,
            })
            .is_some_and(|frames| frames.parent_opening_owns_bottom_padding)
        {
            return 0.0;
        }
        let (_, _, _, bottom) = self.resolve_stored_nested_cell_padding(cell, table, styles);
        bottom.min(closing.height)
    }
}

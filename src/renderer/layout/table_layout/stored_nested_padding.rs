//! 원본 중첩 행의 확정된 물리 높이로 컷 예약과 배치의 안 여백을 함께 판단한다.

use crate::model::control::Control;
use crate::model::paragraph::LineSeg;
use crate::model::table::{Cell, Table, TablePageBreak};
use crate::renderer::hwpunit_to_px;

use super::{LayoutEngine, ResolvedStyleSet};

impl LayoutEngine {
    /// 작은 셀 선언 대신 확장된 원본 행을 사용한다. 조각 높이를 기준으로 삼으면
    /// 같은 행의 앞뒤 컷이 서로 다른 안 여백을 쓰므로 원래 행 원장을 소비한다.
    pub(crate) fn resolve_stored_nested_cell_padding(
        &self,
        cell: &Cell,
        table: &Table,
        styles: &ResolvedStyleSet,
    ) -> (f64, f64, f64, f64) {
        let fallback = self.resolve_cell_padding(cell, table);
        let raw = cell.effective_padding(&table.padding);
        let raw_vertical = hwpunit_to_px(i32::from(raw.top) + i32::from(raw.bottom), self.dpi);
        let profile = self.profile.get();
        if raw_vertical <= fallback.2 + fallback.3
            || cell.row_span != 1
            || cell.dirty_flag
            || profile.session_edited()
            || !(profile.hwpx_stored_layout() || profile.hwp5_stored_pagination_layout())
            || !matches!(table.page_break, TablePageBreak::RowBreak)
            || self
                .render_normalization
                .borrow()
                .table_text_reflowed(table)
            || cell.paragraphs.iter().any(|paragraph| {
                paragraph.stored_text_partition_is_dirty() || paragraph.cell_format_vpos_dirty
            })
            || !cell.paragraphs.iter().any(|paragraph| {
                !paragraph.line_segs.is_empty()
                    && paragraph
                        .line_segs
                        .iter()
                        .all(|line| line.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0)
                    && paragraph.controls.iter().any(|control| {
                        matches!(control, Control::Table(child)
                            if !self.render_normalization.borrow().table_text_reflowed(child))
                    })
            })
        {
            return fallback;
        }
        let key = table as *const Table as usize;
        let row = usize::from(cell.row);
        let cached_height = self
            .stored_frame_row_plans
            .borrow()
            .get(&key)
            .and_then(|rows| rows.get(row))
            .copied();
        let height = cached_height.or_else(|| {
            // 컷 측정이 전체 배치보다 먼저 실행되어도 같은 행 생산자를 사용한다.
            // 중첩 호출이 호출자의 선언 신뢰 정책을 바꾸지 않도록 복구한다.
            let trust = self.declared_trust_allowed.get();
            let rows = self.resolve_row_heights_trusting_declared(
                table,
                usize::from(table.col_count),
                usize::from(table.row_count),
                None,
                styles,
                false,
                trust,
            );
            self.declared_trust_allowed.set(trust);
            rows.get(row).copied()
        });
        match height {
            Some(height) if height.is_finite() && height > raw_vertical => {
                self.resolve_cell_padding_for_physical_height(cell, table, height)
            }
            // 원래 물리 행에도 들어가지 않는 여백은 기존 비정상 여백 방어를 유지한다.
            _ => fallback,
        }
    }
}

//! 원본 plain 셀의 정렬 여유 공간을 소유하는 첫 내용 조각.

use super::{CellUnit, LayoutEngine, ResolvedStyleSet};
use crate::model::table::{Cell, Table, TablePageBreak, VerticalAlign};
use crate::renderer::hwpunit_to_px;

#[derive(Clone, Copy)]
pub(crate) struct StoredAlignedCutFrame {
    pub(crate) content_height: f64,
    pub(crate) physical_height: f64,
    padding: f64,
}

impl StoredAlignedCutFrame {
    /// RowCut의 예산은 안여백을 이미 제외했다. 같은 물리 요구량에서
    /// 안여백만 빼고 원본 정렬 여유 공간은 유지한다.
    pub(crate) fn content_requirement(self) -> f64 {
        self.physical_height - self.padding
    }
}

impl LayoutEngine {
    /// 선언 최소 높이가 전체 원본 글줄보다 클 때 남는 공간은 내용 유닛이
    /// 아니다. 첫 조각에서 한 번 소유하며, continuation에 다시 더하지 않는다.
    /// 내용이 선언 높이를 넘는 자동 확장 셀은 이 프레임을 갖지 않는다.
    pub(crate) fn stored_aligned_opening_cell_frame(
        &self,
        table: &Table,
        cell: &Cell,
        units: &[CellUnit],
        start: usize,
        end: usize,
        styles: &ResolvedStyleSet,
    ) -> Option<StoredAlignedCutFrame> {
        if start != 0
            || end == 0
            || end >= units.len()
            || cell.row_span != 1
            || cell.text_direction != 0
            || cell.vertical_align == VerticalAlign::Top
            || cell.height > i32::MAX as u32
            || table.common.treat_as_char
            || table.page_break != TablePageBreak::RowBreak
            || self
                .render_normalization
                .borrow()
                .table_text_reflowed(table)
        {
            return None;
        }
        let (_, _, top, bottom) = self.resolve_stored_nested_cell_padding(cell, table, styles);
        let padding = top + bottom;
        let full_content = units.iter().map(|unit| unit.height).sum::<f64>();
        // 같은 저장 줄의 출처 인증을 공유한다. 합성·편집 줄과 개체 셀의
        // 선언 높이를 일반 텍스트의 정렬 여유 공간으로 바꾸지 않는다.
        let minimum = self.stored_plain_cell_content_minimum(cell, full_content, padding)?;
        let slack = hwpunit_to_px(cell.height as i32, self.dpi) - minimum;
        if slack <= 0.0 {
            return None;
        }
        let trim =
            self.native_saved_reset_cut_trailing_trim(table, cell, units, start, end, styles);
        let content_height =
            (units[..end].iter().map(|unit| unit.height).sum::<f64>() - trim).max(0.0);
        Some(StoredAlignedCutFrame {
            content_height,
            physical_height: content_height + padding + slack,
            padding,
        })
    }
}

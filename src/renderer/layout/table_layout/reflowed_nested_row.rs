//! 중첩 RowBreak 행의 일반 글줄 컷을 상위 조각에도 보존한다.

use super::{CellUnit, LayoutEngine, NestedTableCut, NestedTableUnitCut, ResolvedStyleSet};
use crate::model::table::{Cell, Table};

/// 행 유닛 생산자가 실제로 계상한 호스트 앞·뒤 공간.
/// 자식 컷의 셀 내용과 분리해 같은 값을 예약과 viewport에서 소비한다.
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct NestedRowFlowEdges {
    pub before: f64,
    pub after: f64,
}

impl LayoutEngine {
    /// 독립 저장 줄로 닫힌 전체 표의 마지막 빈 행 공간을 같은 계산으로 회수한다.
    /// 부분 내용 컷은 선언된 행의 미소비 물리 띠를 소유하므로 이 전체 상자 규칙과 다르다.
    pub(super) fn fit_complete_stored_row_frame(
        &self,
        table: &Table,
        rows: &mut [f64],
    ) -> Option<f64> {
        use crate::model::shape::{TextWrap, VertRelTo};
        use crate::model::table::TablePageBreak;
        let profile = self.profile.get();
        if !(profile.hwp5_stored_pagination_layout() || profile.hwpx_stored_layout())
            || table.common.treat_as_char
            || table.common.text_wrap != TextWrap::TopAndBottom
            || table.common.vert_rel_to != VertRelTo::Para
            || table.page_break != TablePageBreak::RowBreak
            || self
                .render_normalization
                .borrow()
                .nested_table_has_reflowed_host(table)
        {
            return None;
        }
        let target = crate::renderer::table_row_frame::declared_tail_row_height(
            table,
            rows,
            crate::renderer::hwpunit_to_px(i32::from(table.cell_spacing), self.dpi),
            self.dpi,
        )?;
        let last = rows.last_mut()?;
        let delta = target - *last;
        *last = target;
        Some(delta)
    }

    /// 예약과 배치가 같은 행 유닛에서 같은 자식 커서를 만든다.
    pub(super) fn nested_row_projection_cut(units: &[CellUnit]) -> Option<NestedTableCut> {
        if units.iter().any(|unit| unit.nested_row.is_none()) {
            return None;
        }
        let first = units.first()?;
        let last = units.last()?;
        Some(NestedTableCut {
            start_row: first.nested_row?,
            end_row: last.nested_row? + 1,
            start_cut: first
                .nested_table_fragment
                .as_ref()
                .filter(|fragment| fragment.start_cut.iter().any(|&cut| cut > 0))
                .map(|fragment| fragment.start_cut.clone())
                .unwrap_or_default(),
            end_cut: last
                .nested_table_fragment
                .as_ref()
                .filter(|fragment| !fragment.terminal)
                .map(|fragment| fragment.end_cut.clone())
                .unwrap_or_default(),
            is_block_split: false,
            start_cut_is_block: false,
        })
    }

    pub(super) fn nested_plain_text_row_has_line_cuts(
        &self,
        row_cells: &[&Cell],
        driver_units: &[CellUnit],
    ) -> bool {
        // 현재 재조판한 글줄의 컷을 부모에 전달한다. 원본 저장 행을 단지 여러 줄이라는
        // 이유로 다시 나누지 않는다. 원본의 명시적 재시작은 별도 source 경로가 소유한다.
        driver_units.len() > 1
            && driver_units.iter().any(|unit| unit.reflowed_line_slot)
            && driver_units
                .iter()
                .all(|unit| unit.nested_row.is_none() && unit.vis_start < unit.vis_end)
            && !row_cells.is_empty()
            && row_cells.iter().all(|cell| {
                !cell.paragraphs.is_empty()
                    && cell
                        .paragraphs
                        .iter()
                        .all(|paragraph| paragraph.controls.is_empty())
            })
    }

    /// 자식 배치에 전달하는 것과 같은 시작·종료 컷의 물리 상자를 예약한다.
    /// 마지막 유닛을 소비하면 배치도 빈 end_cut으로 행 전체를 닫는다. 이때
    /// 마지막 줄까지의 내용 높이만 쓰면 선언 상자의 남은 띠가 원장에서 사라진다.
    pub(super) fn nested_plain_row_unit_height(
        &self,
        table: &Table,
        row_frame: (usize, f64),
        frame_start: &[usize],
        fragment: &NestedTableUnitCut,
        styles: &ResolvedStyleSet,
    ) -> f64 {
        let (row, whole_height) = row_frame;
        let origin = if frame_start.iter().all(|&cut| cut == 0) {
            &[][..]
        } else {
            frame_start
        };
        let end = if fragment.terminal {
            &[][..]
        } else {
            fragment.end_cut.as_slice()
        };
        let previous = if fragment.start_cut == frame_start {
            0.0
        } else {
            self.row_cut_content_height(table, row, origin, &fragment.start_cut, styles)
        };
        let required = self.row_cut_content_height(table, row, origin, end, styles);
        let required = if origin.is_empty() && fragment.terminal {
            required.max(whole_height)
        } else {
            required
        };
        (required - previous).max(0.0)
    }
}

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

/// A row can give up only the part of its last line advance shared by all
/// child cells at this cut. A cell already closed before this unit contributes
/// no new extent. Keep the full band if any still-open cell has no new cut.
pub(super) fn shared_closing_line_advance(
    children: &[std::sync::Arc<Vec<CellUnit>>],
    cut: &NestedTableUnitCut,
) -> f64 {
    children
        .iter()
        .zip(&cut.start_cut)
        .zip(&cut.end_cut)
        .filter(|((units, start), _)| **start < units.len())
        .map(|((units, start), end)| {
            if end > start && *end < units.len() {
                units[*end - 1].following_line_advance
            } else {
                0.0
            }
        })
        .reduce(f64::min)
        .unwrap_or(0.0)
}

/// The parent page has reopened this row inside its child text. Its last
/// visible line stops at its own box, before the advance to the next page.
pub(super) fn continuation_closing_advance(
    units: &[CellUnit],
    start: usize,
    end: usize,
) -> Option<f64> {
    if start == 0 || start >= end || end >= units.len() {
        return None;
    }
    let last = &units[end - 1];
    let same_row = |unit: &CellUnit| {
        last.nested_row.is_some()
            && unit.nested_row == last.nested_row
            && unit.para_idx == last.para_idx
    };
    let next = &units[end];
    (last.following_line_advance > 0.0
        && same_row(&units[start - 1])
        && same_row(&units[start])
        && same_row(next)
        && ((!next.hard_break_before && !next.stored_frame_break_before)
            || (next.page_frame_reset_before && next.stored_frame_break_before)))
        .then_some(last.following_line_advance.min(last.height))
}

impl LayoutEngine {
    /// A projected paragraph-before slot belongs to the next visible child line.
    /// Keep its source index in the cut so the reopened child retains its origin.
    pub(super) fn mixed_child_next_paragraph_spacing(
        &self,
        table: &Table,
        cell: &Cell,
        units: &[CellUnit],
        start: usize,
        end: usize,
        styles: &ResolvedStyleSet,
    ) -> Option<f64> {
        let [prior, spacing, next] = units.get(end.checked_sub(2)?..=end)? else {
            return None;
        };
        if end - 2 < start
            || [prior, spacing, next].iter().any(|unit| {
                unit.para_idx != spacing.para_idx
                    || !unit.mixed_nested_fragment
                    || unit.mixed_nested_recursive
                    || unit.mixed_nested_trailing
            })
            || spacing.mixed_nested_source_para_idx.is_some()
            || spacing.mixed_nested_content_height != 0.0
            || prior.mixed_nested_content_height <= 0.0
            || next.mixed_nested_content_height <= 0.0
        {
            return None;
        }
        let prior_para = prior.mixed_nested_source_para_idx?;
        let next_para = next.mixed_nested_source_para_idx?;
        if next_para != prior_para + 1 {
            return None;
        }
        let child = cell
            .paragraphs
            .get(spacing.para_idx)?
            .controls
            .iter()
            .find_map(|control| match control {
                crate::model::control::Control::Table(child) => Some(child.as_ref()),
                _ => None,
            })?;
        if !self.native_terminal_rowbreak_child_source_cursor_eligible(table, cell, child) {
            return None;
        }
        let source = child.cells.first()?.paragraphs.get(next_para)?;
        let before = styles
            .para_styles
            .get(source.para_shape_id as usize)?
            .spacing_before;
        (before > 0.0 && spacing.height == before).then_some(before)
    }

    /// 같은 자식 글줄 행의 중간에서 열고 닫는 블록 조각. 자식 RowCut이 이미
    /// 마지막 줄의 전진을 제외했으므로 부모는 그 상자와 실제 바깥 여백만 소비한다.
    pub(super) fn continued_plain_child_frame_height(
        &self,
        cell: &Cell,
        units: &[CellUnit],
        start: usize,
        end: usize,
        styles: &ResolvedStyleSet,
        purpose: super::row_physical_band::CutHeight,
    ) -> Option<f64> {
        use crate::model::control::Control;
        use crate::model::shape::TextWrap;
        use crate::renderer::hwpunit_to_px;

        continuation_closing_advance(units, start, end)?;
        let paragraph = cell.paragraphs.get(units[start].para_idx)?;
        let mut children = paragraph
            .controls
            .iter()
            .filter_map(|control| match control {
                Control::Table(child) => Some(child.as_ref()),
                _ => None,
            });
        let child = children.next()?;
        if children.next().is_some()
            || child.common.treat_as_char
            || !child.common.flow_with_text
            || child.common.text_wrap != TextWrap::TopAndBottom
        {
            return None;
        }
        let cut = Self::nested_row_projection_cut(&units[start..end])?;
        let margins = i32::from(child.outer_margin_top) + i32::from(child.outer_margin_bottom);
        Some(
            self.nested_cut_height(child, &cut, styles, purpose) + hwpunit_to_px(margins, self.dpi),
        )
    }

    /// 실제 재귀 글줄 컷 또는 저장 글줄이 없는 호스트의 재조판 행만 재귀 프레임을 연다.
    /// 저장 호스트의 원자 행은 기존 행 조각 경로가 첫 물리 프레임과 문단 범위를 소유한다.
    pub(super) fn nested_row_run_uses_recursive_frame(
        host: &crate::model::paragraph::Paragraph,
        units: &[CellUnit],
    ) -> bool {
        units
            .iter()
            .any(|unit| unit.nested_table_fragment.is_some())
            || (crate::renderer::para_has_no_stored_line_segs(host)
                && !units.is_empty()
                && units.iter().all(|unit| unit.nested_row.is_some()))
    }

    /// 원자 행 사이의 이어받기도 새 자식 표의 바깥 위 프레임을 연다.
    /// 부모 예약과 실제 자식 원점에서 같은 원본 여백을 한 번 소비한다.
    pub(in crate::renderer::layout) fn nested_row_continuation_outer_top(
        &self,
        child: &Table,
        cut: &NestedTableCut,
    ) -> Option<f64> {
        (!child.common.treat_as_char && cut.start_row > 0)
            .then(|| crate::renderer::hwpunit_to_px(i32::from(child.outer_margin_top), self.dpi))
    }

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
            || table.layout_page_break() != TablePageBreak::RowBreak
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
        outgrows_declared_frame: bool,
    ) -> bool {
        // 현재 재조판한 글줄의 컷을 부모에 전달한다. 원본 저장 행을 단지 여러 줄이라는
        // 이유로 다시 나누지 않는다. 원본의 명시적 재시작은 별도 source 경로가 소유한다.
        //
        // 저장된 재시작 경계가 없는 행이 선언 높이보다 자라면 실제 줄 경계를
        // 부모에도 전달한다. 남은 쪽에 수용할 컷은 부모의 실제 예산으로 정한다.
        // 칸의 단일 단 정의 등 자체 배치 상자가 없는 표식도 같은 글줄을 사용한다.
        driver_units.len() > 1
            && (outgrows_declared_frame
                || driver_units.iter().any(|unit| unit.reflowed_line_slot)
                || row_cells
                    .iter()
                    .any(|cell| cell.paragraphs.iter().any(|para| para.cell_lines_edited)))
            && driver_units
                .iter()
                .all(|unit| unit.nested_row.is_none() && unit.vis_start < unit.vis_end)
            && !row_cells.is_empty()
            && row_cells.iter().all(|cell| {
                !cell.paragraphs.is_empty()
                    && cell
                        .paragraphs
                        .iter()
                        .all(crate::renderer::cell_paragraph_has_only_text_metadata)
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

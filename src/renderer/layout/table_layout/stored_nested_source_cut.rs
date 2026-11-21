//! 상위 중첩 투영에서도 실제 자식 컷이 소유한 원본 재시작을 보존한다.

use crate::model::control::Control;
use crate::model::paragraph::LineSeg;
use crate::model::table::{Cell, Table};

use super::{CellUnit, LayoutEngine, NestedTableCut, ResolvedStyleSet};

impl LayoutEngine {
    /// 저장 글줄 뒤의 단일 자식 표가 아직 현재 컷에 들어오지 않은 호스트 문단.
    pub(in crate::renderer::layout) fn stored_carried_child_host(
        &self,
        table: &Table,
        cell: &Cell,
        start: usize,
        end: usize,
        styles: &ResolvedStyleSet,
    ) -> Option<usize> {
        let profile = self.profile.get();
        if start >= end
            || cell.dirty_flag
            || profile.session_edited()
            || !(profile.hwpx_stored_layout() || profile.hwp5_stored_pagination_layout())
        {
            return None;
        }
        let units = self.cell_units(cell, table, styles);
        let previous = units.get(end - 1)?;
        let next = units.get(end)?;
        if previous.para_idx != next.para_idx
            || previous.empty_spacer
            || previous.reflowed_line_slot
            || previous.vis_start >= previous.vis_end
            || next.vis_start != previous.vis_end
            || next.vis_start != next.vis_end
            || (next.nested_row.is_none() && !next.mixed_nested_fragment)
        {
            return None;
        }
        let paragraph = cell.paragraphs.get(previous.para_idx)?;
        let line = paragraph.line_segs.get(previous.vis_end - 1)?;
        (!paragraph.stored_text_partition_is_dirty()
            && !paragraph.cell_format_vpos_dirty
            && line.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0
            && paragraph
                .controls
                .iter()
                .filter(|control| matches!(control, Control::Table(_)))
                .count()
                == 1
            && !units[start..end].iter().any(|unit| {
                unit.para_idx == previous.para_idx
                    && (unit.nested_row.is_some() || unit.mixed_nested_fragment)
            }))
        .then_some(previous.para_idx)
    }

    /// 현재 조각의 첫 자식이 닫히는 좌표는 그 직후 저장 줄로 확인한다.
    /// 독립 줄 원점과 자식 컷이 같은 끝을 소유하면 그리기와 부모 행이 같은 물리 띠를 쓴다.
    pub(super) fn stored_nested_continuation_before_line(
        &self,
        parent: &Table,
        cell: &Cell,
        child: &Table,
        cut: &NestedTableCut,
        selected: &[CellUnit],
        styles: &ResolvedStyleSet,
    ) -> Option<f64> {
        let profile = self.profile.get();
        if profile.session_edited()
            || !(profile.hwpx_stored_layout() || profile.hwp5_stored_pagination_layout())
            || cell.dirty_flag
            || self
                .render_normalization
                .borrow()
                .table_text_reflowed(parent)
            || self
                .render_normalization
                .borrow()
                .table_text_reflowed(child)
            || cut.start_row != 0
            || cut.end_row != usize::from(child.row_count)
            || cut.start_cut.is_empty()
            || !cut.end_cut.is_empty()
            || cut.is_block_split
            || cut.start_cut_is_block
            || child.outer_margin_top != 0
            || child.outer_margin_bottom != 0
            || child.caption.is_some()
        {
            return None;
        }
        let owner = selected.first()?;
        let next = selected
            .iter()
            .find(|unit| unit.para_idx > owner.para_idx)?;
        if !owner.mixed_nested_recursive
            || next.para_idx != owner.para_idx + 1
            || next.vis_start != 0
            || next.mixed_nested_fragment
        {
            return None;
        }
        let host = cell.paragraphs.get(owner.para_idx)?;
        if !host.controls.iter().any(|control| {
            matches!(control, Control::Table(table) if std::ptr::eq(table.as_ref(), child))
        }) {
            return None;
        }
        let successor = cell.paragraphs.get(next.para_idx)?;
        let line = successor.line_segs.first()?;
        if !successor.controls.is_empty()
            || successor.stored_text_partition_is_dirty()
            || successor.cell_format_vpos_dirty
            || successor
                .source_line_seg_vertical_pos
                .as_ref()
                .is_some_and(|source| source.first() != Some(&line.vertical_pos))
            || line.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
            || line.vertical_pos <= 0
            || line.line_height <= 0
        {
            return None;
        }
        let height = self.nested_cut_physical_height(child, cut, styles);
        (crate::renderer::px_to_hwpunit(height, self.dpi) == line.vertical_pos).then_some(height)
    }

    /// 단일 자식 유닛의 시작 컷을 원본 글줄까지 추적한다.
    /// 블록 컷의 인덱스를 셀 유닛으로 해석하거나 로컬 문단 원점을 쪽 경계로 올리지 않는다.
    pub(super) fn stored_nested_source_unit_frame_break(
        &self,
        cell: &Cell,
        table: &Table,
        index: usize,
        styles: &ResolvedStyleSet,
    ) -> bool {
        self.stored_nested_unit_frame_break(cell, table, index, styles, false)
    }

    /// 완결 행으로 닫힌 개체 프레임만 상위 쪽 경계로 전달한다.
    /// 자식 문단의 저장 재시작은 그 자식의 내용 컷이며, 부모의 현재 쪽을
    /// 강제로 닫는 증거와 구분한다.
    pub(super) fn stored_nested_object_unit_frame_break(
        &self,
        cell: &Cell,
        table: &Table,
        index: usize,
        styles: &ResolvedStyleSet,
    ) -> bool {
        self.stored_nested_unit_frame_break(cell, table, index, styles, true)
    }

    fn stored_nested_unit_frame_break(
        &self,
        cell: &Cell,
        table: &Table,
        index: usize,
        styles: &ResolvedStyleSet,
        object_frame_only: bool,
    ) -> bool {
        let profile = self.profile.get();
        if profile.session_edited()
            || !(profile.hwpx_stored_layout() || profile.hwp5_stored_pagination_layout())
            || self
                .render_normalization
                .borrow()
                .table_text_reflowed(table)
        {
            return false;
        }
        let units = self.cell_units(cell, table, styles);
        let Some(unit) = units.get(index) else {
            return false;
        };
        if !object_frame_only
            && self.stored_in_paragraph_unit_is_original_restart(table, cell, unit)
        {
            return true;
        }
        let Some(child) = cell.paragraphs.get(unit.para_idx).and_then(|paragraph| {
            if paragraph.stored_text_partition_is_dirty() || paragraph.cell_format_vpos_dirty {
                return None;
            }
            paragraph.controls.iter().find_map(|control| match control {
                Control::Table(child) => Some(child.as_ref()),
                _ => None,
            })
        }) else {
            return false;
        };
        // 원본의 첫 개체 상자가 완결 행에서 닫힌 경우는 문자 줄 reset이 아니다.
        // 상위 컷에서도 같은 자식 행 계획을 인증하고, 그 행의 첫 유닛만 보존한다.
        if unit.stored_frame_break_before
            && unit
                .nested_row
                .is_some_and(|row| self.stored_measured_object_frame_row_count(child) == Some(row))
            && unit
                .nested_table_fragment
                .as_ref()
                .is_none_or(|fragment| fragment.start_cut.iter().all(|&start| start == 0))
        {
            return true;
        }
        if let Some((row, fragment)) = unit.nested_row.zip(unit.nested_table_fragment.as_ref()) {
            return self.stored_nested_source_cut_frame_break(
                child,
                row,
                &fragment.start_cut,
                styles,
                object_frame_only,
            );
        }
        if !unit.mixed_nested_recursive {
            return false;
        }
        let Some(cut) = self
            .mixed_nested_split_from_cut(cell, table, styles, index, index + 1, unit.para_idx)
            .and_then(|split| split.recursive_cut)
            .filter(|cut| !cut.start_cut_is_block && !cut.is_block_split)
        else {
            return false;
        };
        self.stored_nested_source_cut_frame_break(
            child,
            cut.start_row,
            &cut.start_cut,
            styles,
            object_frame_only,
        )
    }

    fn stored_nested_source_cut_frame_break(
        &self,
        table: &Table,
        row: usize,
        cut: &[usize],
        styles: &ResolvedStyleSet,
        object_frame_only: bool,
    ) -> bool {
        Self::row_cut_cell_order(table, row)
            .into_iter()
            .enumerate()
            .any(|(order, cell_index)| {
                let Some(&start) = cut.get(order) else {
                    return false;
                };
                let Some(cell) = table.cells.get(cell_index) else {
                    return false;
                };
                self.stored_nested_unit_frame_break(cell, table, start, styles, object_frame_only)
            })
    }
}

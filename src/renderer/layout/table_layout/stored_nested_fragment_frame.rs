//! 실제 자식 컷이 소유한 첫 물리 상자와 마지막 부모 호스트 줄을 공유한다.

use super::{
    row_physical_band::CutHeight, CellUnit, LayoutEngine, NestedTableCut, ResolvedStyleSet,
};
use crate::model::{control::Control, paragraph::LineSeg, paragraph::Paragraph, table::Table};
use crate::renderer::hwpunit_to_px;

pub(crate) struct StoredNestedOpeningFrame {
    pub(crate) height: f64,
    pub(crate) last_row_height: f64,
}

impl LayoutEngine {
    /// 부모 첫 조각의 별도 물리 상자다. 선택 유닛의 끝을 인증한 뒤 이미
    /// 수용한 완전 행을 빼며, 이 높이를 전체 논리 행의 소비량으로 쓰지 않는다.
    pub(crate) fn stored_nested_parent_opening_frame(
        &self,
        table: &Table,
        row: usize,
        cut: &[usize],
        measured_rows: &[f64],
        styles: &ResolvedStyleSet,
    ) -> Option<StoredNestedOpeningFrame> {
        if !self.row_cut_has_original_nested_frame_owners(table, row, cut, styles)
            || !self.stored_row_boxes_exceed_first_frame(table)
            || !(0..row).all(|owner| self.whole_fragment_row_uses_measured_height(table, owner))
        {
            return None;
        }
        let height = hwpunit_to_px(i32::try_from(table.common.height).ok()?, self.dpi);
        let prefix = measured_rows.get(..row)?.iter().sum::<f64>()
            + hwpunit_to_px(i32::from(table.cell_spacing), self.dpi) * row as f64;
        let last_row_height = height - prefix;
        let selected = self.row_cut_height(table, row, &[], cut, styles, CutHeight::Occupied);
        (prefix >= 0.0 && last_row_height >= selected).then_some(StoredNestedOpeningFrame {
            height,
            last_row_height,
        })
    }

    /// 모든 미완료 칸이 실제 원본 쪽 재시작을 공유하는 첫 자식 컷이다.
    /// 선언 상자의 남은 띠는 마지막 선택 행이 소유하며 줄을 추가 소비하지 않는다.
    pub(in crate::renderer::layout) fn stored_nested_opening_cut_frame(
        &self,
        table: &Table,
        cut: &NestedTableCut,
        styles: &ResolvedStyleSet,
    ) -> Option<StoredNestedOpeningFrame> {
        let row = cut.end_row.checked_sub(1)?;
        if cut.start_row != 0
            || !cut.start_cut.is_empty()
            || cut.is_block_split
            || cut.start_cut_is_block
            || cut.end_cut.is_empty()
            || cut.end_row > usize::from(table.row_count)
            || !self.stored_row_boxes_exceed_first_frame(table)
            || !self.row_cut_has_original_plain_text_owners(table, row, &cut.end_cut, styles)
            || !self.row_cut_ends_at_native_saved_page_frame(table, row, &[], &cut.end_cut, styles)
            || !(0..row).all(|row| self.whole_fragment_row_uses_measured_height(table, row))
        {
            return None;
        }
        let rows = self.resolve_row_heights(
            table,
            usize::from(table.col_count),
            usize::from(table.row_count),
            None,
            styles,
            true,
        );
        let height = hwpunit_to_px(i32::try_from(table.common.height).ok()?, self.dpi);
        let prefix = rows.get(..row)?.iter().sum::<f64>()
            + hwpunit_to_px(i32::from(table.cell_spacing), self.dpi) * row as f64;
        let last_row_height = height - prefix;
        let selected =
            self.row_cut_height(table, row, &[], &cut.end_cut, styles, CutHeight::Occupied);
        (last_row_height >= selected && last_row_height <= *rows.get(row)?).then_some(
            StoredNestedOpeningFrame {
                height,
                last_row_height,
            },
        )
    }

    /// 앞 글줄과 자식의 첫 컷을 함께 소유한 부모 행이다. 자식의 실제 저장
    /// 경계와 첫 상자를 인증하므로 개체 문단의 로컬0만으로 띠를 늘리지 않는다.
    pub(crate) fn row_cut_has_original_nested_frame_owners(
        &self,
        table: &Table,
        row: usize,
        cut: &[usize],
        styles: &ResolvedStyleSet,
    ) -> bool {
        let profile = self.profile.get();
        let owners = self.indexed_row_cut_cell_order(table, row);
        if !(profile.hwp5_stored_pagination_layout() || profile.hwpx_stored_layout())
            || profile.session_edited()
            || self
                .render_normalization
                .borrow()
                .table_text_reflowed(table)
            || table.common.treat_as_char
            || owners.is_empty()
            || owners.len() != cut.len()
        {
            return false;
        }
        let mut nested_boundary = false;
        for (&index, &end) in owners.iter().zip(cut) {
            let cell = &table.cells[index];
            let units = self.cell_units(cell, table, styles);
            if usize::from(cell.row) != row
                || cell.row_span != 1
                || cell.dirty_flag
                || end == 0
                || end > units.len()
                || units
                    .get(end)
                    .is_some_and(|unit| !unit.page_frame_reset_before)
            {
                return false;
            }
            for unit in &units[..end] {
                let para = &cell.paragraphs[unit.para_idx];
                if unit.reflowed_line_slot
                    || unit.mixed_nested_fragment
                    || para.stored_text_partition_is_dirty()
                    || para.cell_format_vpos_dirty
                    || para.line_segs.is_empty()
                    || para.line_segs.iter().any(|line| {
                        line.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
                            || line.vertical_pos < 0
                            || line.line_height <= 0
                            || line.line_spacing < 0
                    })
                {
                    return false;
                }
                if unit.nested_row.is_some() {
                    let [Control::Table(child)] = para.controls.as_slice() else {
                        return false;
                    };
                    let Some(split) = self.nested_table_split_from_cut_units(
                        cell,
                        table,
                        styles,
                        0,
                        end,
                        unit.para_idx,
                    ) else {
                        return false;
                    };
                    let Some(child_cut) = split.recursive_cut.as_ref() else {
                        return false;
                    };
                    if self
                        .stored_nested_opening_cut_frame(child, child_cut, styles)
                        .is_none()
                    {
                        return false;
                    }
                    nested_boundary |= end < units.len() && units[end].para_idx == unit.para_idx;
                } else if para
                    .controls
                    .iter()
                    .any(|control| !matches!(control, Control::Field(_)))
                {
                    return false;
                }
            }
        }
        nested_boundary
    }

    /// 자식 전체의 최소 행 합 대신 마지막 실제 저장 컷으로 호스트 종료식을
    /// 확인한다. 부모 유닛을 생산하는 중이므로 그 부모 캐시는 다시 요청하지 않는다.
    pub(super) fn projected_nested_terminal_host_line(
        &self,
        para: &Paragraph,
        child: &Table,
        para_index: usize,
        units: &[CellUnit],
        styles: &ResolvedStyleSet,
    ) -> Option<i32> {
        let owned: Vec<_> = units
            .iter()
            .filter(|unit| unit.para_idx == para_index)
            .collect();
        let resets: Vec<_> = owned
            .iter()
            .enumerate()
            .filter(|(index, unit)| *index > 0 && unit.page_frame_reset_before)
            .collect();
        // 첫 선언과 마지막 조각이 직접 닫히는 두 프레임 계약이다. 중간 조각의
        // 물리 높이를 잃은 다중 프레임을 전체 최소 높이로 추정하지 않는다.
        let [(start, first)] = resets.as_slice() else {
            return None;
        };
        let first_fragment = first.nested_table_fragment.as_ref()?;
        let last = *owned.last()?;
        let last_fragment = last.nested_table_fragment.as_ref()?;
        if !first.hard_break_before
            || !last_fragment.terminal
            || first_fragment.start_cut.is_empty()
            || owned
                .iter()
                .any(|unit| unit.reflowed_line_slot || unit.mixed_nested_fragment)
            || owned[..*start].iter().any(|unit| unit.nested_row.is_none())
        {
            return None;
        }
        let row = first.nested_row?;
        let opening = NestedTableCut {
            start_row: 0,
            end_row: row + 1,
            start_cut: vec![],
            end_cut: first_fragment.start_cut.clone(),
            is_block_split: false,
            start_cut_is_block: false,
        };
        let opening = self.stored_nested_opening_cut_frame(child, &opening, styles)?;
        let tail = NestedTableCut {
            start_row: row,
            end_row: last.nested_row? + 1,
            start_cut: first_fragment.start_cut.clone(),
            end_cut: vec![],
            is_block_split: false,
            start_cut_is_block: false,
        };
        if tail.end_row != usize::from(child.row_count) {
            return None;
        }
        let body = opening.height + self.nested_cut_physical_height(child, &tail, styles);
        crate::renderer::float_placement::stored_nested_terminal_host_line_height_hu(
            para, child, body, self.dpi,
        )
    }
}

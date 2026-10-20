//! 저장 TAC 줄이 소유한 중첩 표를 행으로 펼쳐도 원래 쪽 시작을 보존한다.

use crate::model::control::Control;
use crate::model::paragraph::LineSeg;
use crate::model::table::{Cell, Table, TablePageBreak};

use super::{CellUnit, LayoutEngine, ResolvedStyleSet};

struct StoredInlineOwnerFrame {
    line_top_hu: i32,
    line_height_hu: i32,
    child_row_count: usize,
}

impl LayoutEngine {
    /// 저장 객체 높이와 바깥여백으로 닫힌 원본 호스트 줄만 공유한다.
    fn stored_inline_owner_frame(
        &self,
        table: &Table,
        cell: &Cell,
        para_idx: usize,
    ) -> Option<StoredInlineOwnerFrame> {
        if !self.profile.get().hwpx_stored_layout()
            || self.profile.get().session_edited()
            || self
                .render_normalization
                .borrow()
                .table_text_reflowed(table)
            || table.common.treat_as_char
            || !matches!(table.page_break, TablePageBreak::RowBreak)
            || cell.row_span != 1
            || cell.dirty_flag
            || Self::row_has_prior_rowspan_cover(table, usize::from(cell.row))
        {
            return None;
        }
        let paragraph = cell.paragraphs.get(para_idx)?;
        if paragraph.stored_text_partition_is_dirty()
            || paragraph.cell_format_vpos_dirty
            || !paragraph.text.trim().is_empty()
            || paragraph.line_segs.len() != 1
            || paragraph.controls.len() != 1
        {
            return None;
        }
        let Control::Table(child) = &paragraph.controls[0] else {
            return None;
        };
        let owner = super::super::control_line_seg_index(paragraph, 0)?;
        let line = paragraph.line_segs.get(owner)?;
        if !child.common.treat_as_char
            || child.common.height == 0
            || line.line_height <= 0
            || line.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
        {
            return None;
        }
        // 한컴이 저장한 객체 높이와 바깥여백이 줄 상자를 닫아야 한다.
        // 합성 줄이나 내용 때문에 더 커진 표를 한 쪽짜리로 취급하지 않는다.
        let object_height = i64::from(child.common.height)
            + i64::from(child.outer_margin_top)
            + i64::from(child.outer_margin_bottom);
        if (i64::from(line.line_height) - object_height).abs() > 2 {
            return None;
        }
        Some(StoredInlineOwnerFrame {
            line_top_hu: line.vertical_pos,
            line_height_hu: line.line_height,
            child_row_count: usize::from(child.row_count),
        })
    }

    /// 각 셀의 원본 한 줄이 소유한 자식 표의 실제 점유 높이를 돌려준다.
    /// 열 전체를 병합하지 않은 셀도 독립된 호스트 줄을 소유한다.
    /// 같은 행 유닛의 전진량과 명시적으로 계상한 후행 간격을 구분하며 컷은 다시 만들지 않는다.
    pub(super) fn stored_inline_row_frame_content_height(
        &self,
        table: &Table,
        cell: &Cell,
        units: &[CellUnit],
        index: usize,
        styles: &ResolvedStyleSet,
    ) -> Option<f64> {
        let first = units.get(index)?;
        if !first.page_frame_reset_before || first.nested_row.is_some_and(|row| row != 0) {
            return None;
        }
        let frame = self.stored_inline_owner_frame(table, cell, first.para_idx)?;
        let paragraph = &cell.paragraphs[first.para_idx];
        if first.nested_row.is_none() {
            // 분할하지 않는 TAC 표는 같은 원본 한 줄을 단일 유닛으로
            // 소유한다. 자식 행으로 펼쳤는지와 저장 쪽 재시작은 별개다.
            // 실제 전진이 호스트 줄 높이+줄간격으로 닫힐 때만 인정한다.
            if first.vis_start != 0
                || first.vis_end != 1
                || first.empty_spacer
                || first.mixed_nested_fragment
                || first.mixed_nested_trailing
                || first.nested_table_fragment.is_some()
                || units
                    .get(index + 1)
                    .is_some_and(|next| next.para_idx == first.para_idx)
            {
                return None;
            }
            let height = crate::renderer::hwpunit_to_px(frame.line_height_hu, self.dpi);
            let advance = height
                + crate::renderer::hwpunit_to_px(paragraph.line_segs[0].line_spacing, self.dpi);
            return ((first.height - advance).abs() <= crate::renderer::hwpunit_to_px(2, self.dpi))
                .then_some(height);
        }
        let group: Vec<_> = units[index..]
            .iter()
            .take_while(|unit| unit.para_idx == first.para_idx && unit.nested_row.is_some())
            .collect();
        if group.len() != frame.child_row_count
            || !group
                .iter()
                .enumerate()
                .all(|(row, unit)| unit.nested_row == Some(row))
        {
            return None;
        }
        let height: f64 = group.iter().map(|unit| unit.height).sum();
        // 이 플래그는 다음 저장 줄이 호스트 높이+줄간격에서 시작한다는
        // 원본 등식으로 계상한 간격이다. 다음 흐름의 전진 공간을 자식 표의
        // 잉크 높이로 비교하면 온전한 한 줄도 저장 상자를 넘었다고 오판한다.
        let trailing = group
            .last()
            .filter(|unit| unit.host_trailing_spacing_charged)
            .map_or(0.0, |_| {
                crate::renderer::hwpunit_to_px(paragraph.line_segs[0].line_spacing.max(0), self.dpi)
            });
        let content_height = height - trailing;
        let spacing = styles
            .para_styles
            .get(paragraph.para_shape_id as usize)
            .map_or(0.0, |style| style.spacing_before + style.spacing_after);
        let stored_height = crate::renderer::hwpunit_to_px(frame.line_height_hu, self.dpi);
        (content_height.is_finite()
            && content_height > 0.0
            && content_height <= stored_height + spacing + super::ROW_CUT_CAPACITY_FP_EPSILON_PX)
            .then_some(content_height)
    }

    /// 조각 첫 유닛이 소유하며 측정에 포함된 원본 표 호스트의 앞 간격이다.
    pub(in crate::renderer::layout) fn stored_inline_owner_lead_for_cut(
        &self,
        table: &Table,
        cell: &Cell,
        styles: &ResolvedStyleSet,
        start_unit: usize,
    ) -> Option<(usize, f64)> {
        let units = self.cell_units(cell, table, styles);
        let first = units.get(start_unit)?;
        if (start_unit > 0 && !first.page_frame_reset_before)
            || first.vis_start != 0
            || first.nested_row.is_some_and(|row| row != 0)
            || first.mixed_nested_fragment
            || first.mixed_nested_trailing
            || start_unit
                .checked_sub(1)
                .and_then(|index| units.get(index))
                .is_some_and(|previous| previous.para_idx == first.para_idx)
        {
            return None;
        }
        let frame = self.stored_inline_owner_frame(table, cell, first.para_idx)?;
        let paragraph = &cell.paragraphs[first.para_idx];
        let before = styles
            .para_styles
            .get(paragraph.para_shape_id as usize)
            .map_or(0.0, |style| style.spacing_before);
        let before = if first.para_idx == 0 {
            crate::renderer::cell_first_para_stored_lead(paragraph, before, self.dpi)
        } else {
            before
        };
        // 줄 상단이 문단 앞 간격으로 설명될 때만 원점0의 첫 슬롯으로 판정한다.
        // 그 밖의 저장 위치를 이 간격이나 셀 원점으로 대체하지 않는다.
        if !before.is_finite()
            || before <= 0.0
            || !first.height.is_finite()
            || frame.line_top_hu != crate::renderer::px_to_hwpunit(before, self.dpi)
            || (first.nested_row.is_none()
                && first.height + super::ROW_CUT_CAPACITY_FP_EPSILON_PX
                    < crate::renderer::hwpunit_to_px(frame.line_height_hu, self.dpi) + before)
        {
            return None;
        }
        Some((first.para_idx, before))
    }

    /// 다음 저장 쪽의 TAC를 소비하지 않아도 현재 쪽의 행 상자는 본문 끝을 소유한다.
    /// 내용 컷과 빈 물리 띠를 분리하며, 원점0 재시작·닫힌 원본 호스트를 함께 확인한다.
    pub(crate) fn stored_inline_restart_row_end_frame(
        &self,
        table: &Table,
        row: usize,
        start: &[usize],
        end: &[usize],
        available: f64,
        styles: &ResolvedStyleSet,
    ) -> Option<f64> {
        if !start.is_empty()
            || !available.is_finite()
            || !self.stored_row_boxes_exceed_first_frame(table)
        {
            return None;
        }
        let cells = Self::row_cut_cell_order(table, row);
        if cells.is_empty() || cells.len() != end.len() {
            return None;
        }
        let mut owns_restart = false;
        let mut declared = 0.0_f64;
        for (&index, &cut) in cells.iter().zip(end) {
            let cell = table.cells.get(index)?;
            if cell.row_span != 1 || cell.height > i32::MAX as u32 || cell.dirty_flag {
                return None;
            }
            declared = declared.max(crate::renderer::hwpunit_to_px(cell.height as i32, self.dpi));
            let units = self.cell_units(cell, table, styles);
            if cut > units.len() {
                return None;
            }
            if cut == units.len() {
                continue;
            }
            let next = units.get(cut)?;
            let before = units.get(cut.checked_sub(1)?)?;
            if !next.hard_break_before
                || !next.page_frame_reset_before
                || next.nested_row.is_some_and(|row| row != 0)
                || next.vis_start != 0
                || before.para_idx == next.para_idx
            {
                return None;
            }
            let frame = self.stored_inline_owner_frame(table, cell, next.para_idx)?;
            if frame.line_top_hu != 0 {
                return None;
            }
            // 같은 유닛의 실제 자식 행·후행 간격을 소유 호스트 상자와 대조한다.
            self.stored_inline_row_frame_content_height(table, cell, &units, cut, styles)?;
            owns_restart = true;
        }
        let content = self.row_cut_content_height(table, row, start, end, styles);
        (owns_restart
            && available > content + crate::renderer::hwpunit_to_px(2, self.dpi)
            && available < declared)
            .then_some(available)
    }
}

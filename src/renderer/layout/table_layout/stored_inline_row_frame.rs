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
        styles: &ResolvedStyleSet,
    ) -> Option<StoredInlineOwnerFrame> {
        if !(self.profile.get().hwpx_stored_layout()
            || self.profile.get().hwp5_stored_pagination_layout())
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
        let caption_height = match &child.caption {
            None => 0,
            Some(caption) => {
                use crate::model::shape::CaptionDirection;
                if !matches!(
                    caption.direction,
                    CaptionDirection::Top | CaptionDirection::Bottom
                ) || caption.paragraphs.is_empty()
                    || caption.paragraphs.iter().any(|paragraph| {
                        paragraph.stored_text_partition_is_dirty()
                            || paragraph.cell_format_vpos_dirty
                            || paragraph.line_segs.is_empty()
                            || paragraph.line_segs.iter().any(|line| {
                                line.line_height <= 0
                                    || line.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
                            })
                    })
                {
                    return None;
                }
                i64::from(crate::renderer::px_to_hwpunit(
                    crate::renderer::composer::caption_height_px_in_context(
                        &child.caption,
                        Some(styles),
                        self.dpi,
                    ),
                    self.dpi,
                )) + i64::from(caption.spacing)
            }
        };
        let object_height = i64::from(child.common.height)
            + i64::from(child.outer_margin_top)
            + i64::from(child.outer_margin_bottom)
            + caption_height;
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
        let frame = self.stored_inline_owner_frame(table, cell, first.para_idx, styles)?;
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

    /// 닫힌 TAC에서 시작해 다음 닫힌 TAC의 원점0까지 이어지는 실제 저장 프레임이다.
    /// 첫 자식 atom만 남은 띠에 들어가도 같은 프레임의 뒤 본문을 앞 쪽으로 옮기지 않는다.
    pub(super) fn stored_native_inline_following_frame_height(
        &self,
        table: &Table,
        cell: &Cell,
        units: &[CellUnit],
        index: usize,
        styles: &ResolvedStyleSet,
    ) -> Option<f64> {
        let first = units.get(index)?;
        if !self.profile.get().hwp5_stored_pagination_layout()
            || !self.stored_row_boxes_exceed_first_frame(table)
            || index == 0
            || !first.page_frame_reset_before
            || first.vis_start != 0
            || first.nested_row.is_some_and(|row| row != 0)
        {
            return None;
        }
        let opening = self.stored_inline_owner_frame(table, cell, first.para_idx, styles)?;
        if opening.line_top_hu != 0 {
            return None;
        }
        let end = index
            + 1
            + units[index + 1..]
                .iter()
                .position(|unit| unit.page_frame_reset_before)?;
        let next = &units[end];
        if next.vis_start != 0
            || next.nested_row.is_some_and(|row| row != 0)
            || next.para_idx <= first.para_idx
            || self
                .stored_inline_owner_frame(table, cell, next.para_idx, styles)?
                .line_top_hu
                != 0
        {
            return None;
        }
        let paragraphs = cell.paragraphs.get(first.para_idx..next.para_idx)?;
        if paragraphs.iter().any(|paragraph| {
            paragraph.stored_text_partition_is_dirty()
                || paragraph.cell_format_vpos_dirty
                || paragraph.line_segs.is_empty()
                || paragraph.line_segs.iter().any(|line| {
                    line.line_height <= 0 || line.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
                })
        }) {
            return None;
        }
        let lines: Vec<_> = paragraphs.iter().flat_map(|para| &para.line_segs).collect();
        if lines
            .windows(2)
            .any(|pair| pair[1].vertical_pos < pair[0].vertical_pos)
        {
            return None;
        }
        let closing = lines.last()?;
        let source_advance = i64::from(closing.vertical_pos)
            + i64::from(closing.line_height)
            + i64::from(closing.line_spacing);
        let height: f64 = units[index..end].iter().map(|unit| unit.height).sum();
        let expected =
            crate::renderer::hwpunit_to_px(i32::try_from(source_advance).ok()?, self.dpi);
        // 컷 선택과 같은 실제 유닛이 저장 끝에 닫혀야 한다. 리셋 표지만으로
        // 로컬 좌표를 쪽 경계로 승격하거나 과거 선언 높이를 새 내용에 강요하지 않는다.
        (height.is_finite()
            && height > 0.0
            && height <= self.current_body_area.get().3
            && (height - expected).abs() <= crate::renderer::hwpunit_to_px(2, self.dpi))
        .then_some(height)
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
        let frame = self.stored_inline_owner_frame(table, cell, first.para_idx, styles)?;
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

    /// 닫힌 TAC 다음의 원본 재시작은 첫 Common 프레임을 끝낸다.
    /// 선택 내용과 물리 빈 띠를 구분하며, 앞 행이 자랐으면 원본 좌표를 재사용하지 않는다.
    pub(crate) fn stored_inline_opening_row_frame(
        &self,
        table: &Table,
        row: usize,
        end: &[usize],
        measured_rows: &[f64],
        styles: &ResolvedStyleSet,
    ) -> Option<f64> {
        if !self.stored_row_boxes_exceed_first_frame(table) || row >= measured_rows.len() {
            return None;
        }
        let source_rows = table.get_raw_row_heights();
        let spacing = crate::renderer::hwpunit_to_px(table.cell_spacing as i32, self.dpi);
        let prefix = measured_rows[..row].iter().sum::<f64>() + spacing * row as f64;
        let source_prefix = source_rows
            .get(..row)?
            .iter()
            .map(|&height| crate::renderer::hwpunit_to_px(height as i32, self.dpi))
            .sum::<f64>()
            + spacing * row as f64;
        let epsilon = crate::renderer::hwpunit_to_px(2, self.dpi);
        if (prefix - source_prefix).abs() > epsilon {
            return None;
        }
        let height = crate::renderer::hwpunit_to_px(table.common.height as i32, self.dpi) - prefix;
        let cells = Self::row_cut_cell_order(table, row);
        if cells.is_empty() || cells.len() != end.len() || height <= 0.0 {
            return None;
        }
        let mut owns_reset = false;
        for (&index, &cut) in cells.iter().zip(end) {
            let cell = table.cells.get(index)?;
            if cell.dirty_flag
                || cell.row_span != 1
                || cell.row as usize != row
                || height >= crate::renderer::hwpunit_to_px(cell.height as i32, self.dpi)
            {
                return None;
            }
            let units = self.cell_units(cell, table, styles);
            if cut == units.len() {
                continue;
            }
            let next = units.get(cut)?;
            let previous = units.get(cut.checked_sub(1)?)?;
            if previous.para_idx >= next.para_idx || next.vis_start != 0 {
                return None;
            }
            let before = self.stored_inline_owner_frame(table, cell, previous.para_idx, styles)?;
            let after = self.stored_inline_owner_frame(table, cell, next.para_idx, styles)?;
            let paragraph = &cell.paragraphs[next.para_idx];
            let lead = styles
                .para_styles
                .get(paragraph.para_shape_id as usize)?
                .spacing_before;
            if after.line_top_hu != crate::renderer::px_to_hwpunit(lead, self.dpi)
                || before.line_top_hu <= after.line_top_hu
            {
                return None;
            }
            let paragraphs = cell.paragraphs.get(..next.para_idx)?;
            let mut source_end = 0;
            for paragraph in paragraphs {
                if paragraph.stored_text_partition_is_dirty()
                    || paragraph.cell_format_vpos_dirty
                    || paragraph.line_segs.is_empty()
                {
                    return None;
                }
                for line in &paragraph.line_segs {
                    if line.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
                        || line.vertical_pos < source_end
                        || line.line_height <= 0
                        || line.line_spacing < 0
                    {
                        return None;
                    }
                    source_end = line.vertical_pos;
                }
            }
            let closing = cell.paragraphs[previous.para_idx].line_segs.last()?;
            let expected = crate::renderer::hwpunit_to_px(
                closing
                    .vertical_pos
                    .checked_add(closing.line_height)?
                    .checked_add(closing.line_spacing)?,
                self.dpi,
            );
            let advance: f64 = units[..cut].iter().map(|unit| unit.height).sum();
            if (advance - expected).abs() > epsilon {
                return None;
            }
            owns_reset = true;
        }
        let content = self.row_cut_content_height(table, row, &[], end, styles);
        (owns_reset && height >= content).then_some(height)
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
        if (!start.is_empty() && !self.profile.get().hwp5_stored_pagination_layout())
            || !available.is_finite()
            || !self.stored_row_boxes_exceed_first_frame(table)
        {
            return None;
        }
        let cells = Self::row_cut_cell_order(table, row);
        if cells.is_empty()
            || cells.len() != end.len()
            || (!start.is_empty() && cells.len() != start.len())
        {
            return None;
        }
        let mut owns_restart = false;
        let mut declared = 0.0_f64;
        for (slot, (&index, &cut)) in cells.iter().zip(end).enumerate() {
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
            let frame = self.stored_inline_owner_frame(table, cell, next.para_idx, styles)?;
            if frame.line_top_hu != 0 {
                return None;
            }
            // 같은 유닛의 실제 자식 행·후행 간격을 소유 호스트 상자와 대조한다.
            self.stored_inline_row_frame_content_height(table, cell, &units, cut, styles)?;
            if let Some(&opening) = start.get(slot) {
                // Native의 이어받은 조각도 같은 닫힌 원본 프레임에서 시작했으면
                // 뒤 TAC를 넘기는 물리 띠를 소유한다. 임의 중간 컷에는 적용하지 않는다.
                self.stored_native_inline_following_frame_height(
                    table, cell, &units, opening, styles,
                )?;
            }
            owns_restart = true;
        }
        let content = self.row_cut_content_height(table, row, start, end, styles);
        (owns_restart
            && available > content + crate::renderer::hwpunit_to_px(2, self.dpi)
            && available < declared)
            .then_some(available)
    }
}

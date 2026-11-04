//! 첫 분할 조각이 소유한 선언 행 상자와 그 상자 안의 정렬 계약.

use super::{LayoutEngine, ResolvedStyleSet};
use crate::model::{paragraph::LineSeg, table::Table, table::TablePageBreak};
use crate::renderer::hwpunit_to_px;

pub(crate) struct DeclaredRowFirstFrame {
    pub height: f64,
    cell_alignment_heights: Vec<(usize, f64)>,
}

impl DeclaredRowFirstFrame {
    pub fn cell_alignment_height(&self, cell_index: usize) -> Option<f64> {
        self.cell_alignment_heights
            .iter()
            .find_map(|&(index, height)| (index == cell_index).then_some(height))
    }
}

impl LayoutEngine {
    /// 마지막 그림 앵커 문단 전체를 이어받은 조각은 같은 유닛의 점유로 정렬한다.
    /// 줄과 그림을 겹쳐 계상한 원장 위에 앵커 줄 높이를 다시 얹지 않는다.
    pub(in crate::renderer::layout) fn stored_atomic_picture_suffix_content_height(
        &self,
        cell: &crate::model::table::Cell,
        table: &Table,
        styles: &ResolvedStyleSet,
        start: usize,
        end: usize,
    ) -> Option<f64> {
        let profile = self.profile.get();
        if !(profile.hwp5_stored_pagination_layout() || profile.hwpx_stored_layout())
            || profile.session_edited()
            || table.common.treat_as_char
            || table.layout_page_break() != TablePageBreak::RowBreak
            || cell.dirty_flag
            || self
                .render_normalization
                .borrow()
                .cell_frame_reflowed(table, cell)
        {
            return None;
        }
        let units = self.cell_units(cell, table, styles);
        let end = end.min(units.len());
        let first = units.get(start)?;
        if start == 0 || end != units.len() || start >= end || first.vis_start != 0 {
            return None;
        }
        let paragraph = &cell.paragraphs[first.para_idx];
        let selected = &units[start..end];
        if !paragraph.text.trim().is_empty()
            || !selected.iter().all(|unit| {
                unit.para_idx == first.para_idx
                    && !unit.reflowed_line_slot
                    && unit.nested_row.is_none()
                    && unit.nested_table_fragment.is_none()
                    && !unit.mixed_nested_fragment
            })
        {
            return None;
        }
        let mut picture_owner = false;
        // 앵커 글줄에도 flow 분류는 있지만 실제 개체 소유 범위는 그림 띠만 갖는다.
        // 소유자가 아닌 글줄의 None으로 인증을 중단하지 않는다.
        for unit in selected
            .iter()
            .filter(|unit| unit.top_and_bottom_flow && unit.non_inline_control_range.is_some())
        {
            let (first_control, last_control) = unit.non_inline_control_range?;
            for index in first_control..=last_control {
                let crate::model::control::Control::Picture(picture) =
                    paragraph.controls.get(index)?
                else {
                    return None;
                };
                crate::renderer::float_placement::stored_cell_flow_picture_frame(
                    &cell.paragraphs,
                    first.para_idx,
                    picture,
                    true,
                )?;
                picture_owner = true;
            }
        }
        if !picture_owner {
            return None;
        }
        let advance = selected.iter().map(|unit| unit.height).sum();
        Some(
            super::row_block_cut::CellCutFrame::new(self, cell, table, styles, &units, start)
                .content_height(end, advance),
        )
    }

    /// 첫 조각은 선언 높이를 남은 본문 안에서 소유하고, 이어받는 조각은 남은
    /// 내용만 소유한다. 내용보다 큰 독립 최소는 기존 물리 잔여 원장이 맡는다.
    /// 저장 그림 앵커를 통째로 이월한 컷은 현재 선택한 앞 내용만 정렬한다.
    pub(crate) fn row_declared_first_frame(
        &self,
        table: &Table,
        row: usize,
        row_frame_height: f64,
        end_cut: &[usize],
        styles: &ResolvedStyleSet,
    ) -> Option<DeclaredRowFirstFrame> {
        let reflowed = self.row_uses_reflow_physical_frame(table, row);
        if reflowed {
            if self
                .reflow_row_physical_minimum(table, row, row_frame_height, None, styles)
                .is_some()
            {
                return None;
            }
        }
        let cell_alignment_heights = if reflowed {
            Vec::new()
        } else {
            self.stored_row_atomic_picture_alignment(table, row, end_cut, styles)?
        };
        let height = table
            .cells
            .iter()
            .filter(|cell| usize::from(cell.row) == row && cell.row_span == 1)
            .filter(|cell| cell.height > 0 && cell.height <= i32::MAX as u32)
            .map(|cell| hwpunit_to_px(cell.height as i32, self.dpi))
            .reduce(f64::max)?;
        Some(DeclaredRowFirstFrame {
            height,
            cell_alignment_heights,
        })
    }

    /// 실제 컷이 저장된 그림 앵커 문단의 첫 유닛 앞에 있는지 확인한다.
    /// 단순히 뒤에 그림이 있다는 사실이나 선언 높이만으로는 인증하지 않는다.
    fn stored_row_atomic_picture_alignment(
        &self,
        table: &Table,
        row: usize,
        cut: &[usize],
        styles: &ResolvedStyleSet,
    ) -> Option<Vec<(usize, f64)>> {
        let profile = self.profile.get();
        if !(profile.hwp5_stored_pagination_layout() || profile.hwpx_stored_layout())
            || profile.session_edited()
            || table.common.treat_as_char
            || table.layout_page_break() != TablePageBreak::RowBreak
            || self
                .render_normalization
                .borrow()
                .table_text_reflowed(table)
        {
            return None;
        }
        let owners = self.indexed_row_cut_cell_order(table, row);
        if owners.is_empty() || owners.len() != cut.len() {
            return None;
        }
        let mut alignment = Vec::new();
        for (&index, &end) in owners.iter().zip(cut) {
            let cell = &table.cells[index];
            let units = self.cell_units(cell, table, styles);
            if usize::from(cell.row) != row || cell.row_span != 1 || cell.dirty_flag {
                return None;
            }
            let Some(next) = units.get(end) else {
                continue;
            };
            if end == 0 || units[end - 1].para_idx >= next.para_idx || next.vis_start != 0 {
                return None;
            }
            let anchor = &cell.paragraphs[next.para_idx];
            if !anchor.text.trim().is_empty() {
                return None;
            }
            let owner_end = end
                + units[end..]
                    .iter()
                    .take_while(|unit| unit.para_idx == next.para_idx)
                    .count();
            let owner_units = &units[end..owner_end];
            // 그림 띠를 소유하지 않는 유닛은 이 정렬 계약의 대상이 아니다.
            // 미발견은 호출자의 일반 배치로 돌려보내는 기존 None 반환을 유지한다.
            let band = owner_units.iter().position(|unit| {
                unit.top_and_bottom_flow && unit.non_inline_control_range.is_some()
            })?;
            if band == 0
                || !owner_units[..band].iter().all(|unit| {
                    !unit.reflowed_line_slot
                        && unit.non_inline_control_range.is_none()
                        && unit.nested_row.is_none()
                        && unit.nested_table_fragment.is_none()
                        && unit.vis_start < unit.vis_end
                })
            {
                return None;
            }
            // 원본의 단조 저장 프레임만 이 계약에 들어온다. 쪽 재시작과 편집 후
            // 재조판은 각각의 프레임 생산자가 높이와 정렬을 함께 소유한다.
            let mut previous = None;
            for paragraph in cell.paragraphs.iter().take(next.para_idx + 1) {
                if paragraph.stored_text_partition_is_dirty()
                    || paragraph.cell_format_vpos_dirty
                    || paragraph.line_segs.is_empty()
                {
                    return None;
                }
                for line in &paragraph.line_segs {
                    if line.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
                        || line.line_height <= 0
                        || line.line_spacing < 0
                        || line.vertical_pos < 0
                        || previous.is_some_and(|origin| line.vertical_pos < origin)
                    {
                        return None;
                    }
                    previous = Some(line.vertical_pos);
                }
            }
            // 컷은 앵커 줄과 그림을 다음 조각에 함께 넘긴다. 첫 물리 상자의
            // 가운데 정렬에는 그림 앞의 저장 빈 줄 상자가 남는다. 그 줄의
            // 잉크 점유 끝까지만 읽고 그림 띠는 포함하거나 다시 소비하지 않는다.
            let alignment_end = end + band;
            let advance = units[..alignment_end].iter().map(|unit| unit.height).sum();
            let content =
                super::row_block_cut::CellCutFrame::new(self, cell, table, styles, &units, 0)
                    .content_height(alignment_end, advance);
            alignment.push((index, content));
        }
        (!alignment.is_empty()).then_some(alignment)
    }
}

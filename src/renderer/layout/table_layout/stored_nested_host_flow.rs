//! 저장 종료 호스트를 실제 수용 유닛의 물리 흐름으로 배치한다.

use super::{LayoutEngine, ResolvedStyleSet};
use crate::model::{
    control::Control,
    table::{Cell, Table},
};
use crate::renderer::float_placement::{
    stored_nested_host_closes_after_previous_line, stored_nested_terminal_host_line_height_hu,
};

#[derive(Clone, Copy)]
pub(in crate::renderer::layout) enum StoredNestedHostFlow {
    /// 표 본체 뒤의 호스트 줄까지 온전한 원자 유닛 구간을 소유한다.
    BeforeLine { height: f64 },
    /// 마지막 중첩 표의 뒤 줄이다. 부분 컷에서도 줄을 표 시작점으로 쓰지 않는다.
    TerminalLine,
}

impl LayoutEngine {
    /// 자식 표가 새 쪽에서 시작하더라도 그 host의 저장 줄은 앞쪽 프레임을 닫는다.
    /// 부모의 수용된 행 상자 안에서 앞쪽 선택 글줄의 정렬 높이를 반환한다.
    pub(in crate::renderer::layout) fn stored_frame_before_nested_block_content_height(
        &self,
        cell: &Cell,
        table: &Table,
        styles: &ResolvedStyleSet,
        start: usize,
        end: usize,
    ) -> Option<f64> {
        use crate::model::{paragraph::LineSeg, table::VerticalAlign};
        if start == 0
            || start >= end
            || cell.dirty_flag
            || cell.vertical_align != VerticalAlign::Center
            || !self.profile.get().hwp5_stored_pagination_layout()
            || self.profile.get().session_edited()
            || self
                .render_normalization
                .borrow()
                .cell_frame_reflowed(table, cell)
        {
            return None;
        }
        let units = self.cell_units(cell, table, styles);
        let first = units.get(start)?;
        let last = units.get(end - 1)?;
        let next = units.get(end)?;
        let child_cut = next.nested_table_fragment.as_ref()?;
        if !first.page_frame_reset_before
            || !last.empty_spacer
            || next.nested_row != Some(0)
            || child_cut.start_cut.iter().any(|&cut| cut != 0)
            || units[start + 1..end].iter().any(|unit| {
                unit.reflowed_line_slot
                    || unit.page_frame_reset_before
                    || unit.nested_row.is_some()
                    || unit.mixed_nested_fragment
            })
        {
            return None;
        }
        let first_para = cell.paragraphs.get(first.para_idx)?;
        let first_line = first_para.line_segs.get(first.vis_start)?;
        let last_para = cell.paragraphs.get(last.para_idx)?;
        let previous = last_para.line_segs.get(last.vis_start)?;
        let host = cell.paragraphs.get(next.para_idx)?;
        let [Control::Table(child)] = host.controls.as_slice() else {
            return None;
        };
        let [closing] = host.line_segs.as_slice() else {
            return None;
        };
        if [first_para, last_para, host]
            .into_iter()
            .any(|para| para.stored_text_partition_is_dirty() || para.cell_format_vpos_dirty)
            || !host.text.trim().is_empty()
            || !crate::renderer::float_placement::is_para_topbottom_float(&child.common)
            || child.common.treat_as_char
            || [first_line, previous, closing].into_iter().any(|line| {
                line.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
                    || line.vertical_pos < 0
                    || line.line_height <= 0
            })
        {
            return None;
        }
        let previous_end = previous
            .vertical_pos
            .checked_add(previous.line_height)?
            .checked_add(previous.line_spacing)?;
        if previous_end != closing.vertical_pos {
            return None;
        }
        let extent = closing
            .vertical_pos
            .checked_add(closing.line_height)?
            .checked_sub(first_line.vertical_pos)?;
        (extent > 0).then(|| crate::renderer::hwpunit_to_px(extent, self.dpi))
    }

    pub(in crate::renderer::layout) fn stored_nested_host_physical_flow(
        &self,
        cell: &Cell,
        table: &Table,
        styles: &ResolvedStyleSet,
        para_index: usize,
        cut: Option<(usize, usize)>,
    ) -> Option<StoredNestedHostFlow> {
        if let Some(height) =
            self.block_following_host_flow_height(cell, table, styles, para_index, cut)
        {
            return Some(StoredNestedHostFlow::BeforeLine { height });
        }
        if cell.dirty_flag
            || self.profile.get().session_edited()
            || !(self.profile.get().hwp5_stored_pagination_layout()
                || self.profile.get().hwpx_stored_layout())
            || self
                .render_normalization
                .borrow()
                .table_text_reflowed(table)
        {
            return None;
        }
        let para = cell.paragraphs.get(para_index)?;
        let [Control::Table(child)] = para.controls.as_slice() else {
            return None;
        };
        if self
            .render_normalization
            .borrow()
            .table_text_reflowed(child)
        {
            return None;
        }
        let body = self.nested_table_body_height(child, styles);
        if para_index + 1 == cell.paragraphs.len()
            && stored_nested_terminal_host_line_height_hu(para, child, body, self.dpi).is_some()
        {
            return Some(StoredNestedHostFlow::TerminalLine);
        }
        if !stored_nested_host_closes_after_previous_line(
            &cell.paragraphs,
            para_index,
            child,
            body,
            self.dpi,
        ) {
            return None;
        }
        let units = self.cell_units(cell, table, styles);
        let lo = units.iter().position(|u| u.para_idx == para_index)?;
        let hi = units.iter().rposition(|u| u.para_idx == para_index)? + 1;
        let (start, end) = cut.unwrap_or((0, units.len()));
        // 같은 조각이 선행 줄과 완결 호스트를 모두 소유해야 한다. 앞 문단이
        // 이전 조각에 있거나 자식 내용 컷을 품는 유닛이면 기존 이어받기 계약을 쓴다.
        if start >= lo
            || end < hi
            || units.get(lo.checked_sub(1)?)?.para_idx + 1 != para_index
            || units[lo..hi].iter().any(|u| {
                u.nested_row.is_some()
                    || u.mixed_nested_fragment
                    || u.non_inline_control_range.is_some()
                    || u.empty_spacer
            })
        {
            return None;
        }
        let height = units[lo..hi].iter().map(|u| u.height).sum::<f64>();
        (height.is_finite() && height > 0.0).then_some(StoredNestedHostFlow::BeforeLine { height })
    }
}

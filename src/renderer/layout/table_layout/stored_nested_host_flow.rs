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
    pub(in crate::renderer::layout) fn stored_nested_host_physical_flow(
        &self,
        cell: &Cell,
        table: &Table,
        styles: &ResolvedStyleSet,
        para_index: usize,
        cut: Option<(usize, usize)>,
    ) -> Option<StoredNestedHostFlow> {
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

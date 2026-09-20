//! 조판 상태의 조회 입력과 확정 결과 반영 경계.
//! 현재 inline 흐름만 분리했으며 다른 상태 변경의 소유권은 아직 상위 구현에 있다.

use super::inline_flow::plan::InlineFlowInput;
use super::TypesetState;
use crate::renderer::inline_flow::InlineFlowPlan;
use crate::renderer::page_layout::LayoutRect;
use crate::renderer::pagination::PageItem;

impl TypesetState {
    pub(super) fn inline_flow_column(&self) -> LayoutRect {
        let layout = self.current_zone_layout.as_ref().unwrap_or(&self.layout);
        let mut column = layout
            .column_areas
            .get(self.current_column as usize)
            .copied()
            .unwrap_or(layout.body_area);
        column.y += self.current_zone_y_offset;
        column.height = (column.height - self.current_zone_y_offset).max(0.0);
        column
    }

    pub(super) fn inline_flow_input(&self, start: f64, preceding: bool) -> InlineFlowInput<'_> {
        InlineFlowInput {
            column: self.inline_flow_column(),
            body: &self.layout.body_area,
            page_width: self.layout.page_width,
            page_height: self.layout.page_height,
            start,
            exclusions: preceding.then_some(&self.side_wrap_exclusions),
        }
    }

    /// fit을 통과한 동일 plan을 배치 metadata와 높이에 함께 반영한다.
    pub(super) fn commit_inline_flow(&mut self, para_index: usize, plan: InlineFlowPlan) {
        self.current_items
            .push(PageItem::FullParagraph { para_index });
        self.current_height = plan.end;
        self.inline_box_flow_bottom = self.inline_box_flow_bottom.max(plan.end);
        self.inline_flow_plans.insert(para_index, plan);
        self.vpos_ladder_dirty = true;
    }
}

//! 본문 inline의 측정·fit·배치 결과 소유자.

use super::*;
use crate::renderer::{float_placement::ObjectPlacementFrame, inline_flow};

impl TypesetEngine {
    pub(super) fn typeset_inline_flow(
        &self,
        st: &mut TypesetState,
        para_index: usize,
        para: &Paragraph,
        styles: &ResolvedStyleSet,
        tables: &[MeasuredTable],
    ) -> bool {
        let column = st.inline_flow_column();
        if !inline_flow::supports(para, super::super::px_to_hwpunit(column.width, self.dpi)) {
            return false;
        }
        let build = |st: &TypesetState, start: f64, preceding: bool| {
            let column = st.inline_flow_column();
            let style = styles.para_styles.get(para.para_shape_id as usize)?;
            let container = super::super::page_layout::LayoutRect {
                x: column.x + style.margin_left,
                width: (column.width - style.margin_left - style.margin_right).max(0.0),
                ..column
            };
            let paper = super::super::page_layout::LayoutRect {
                x: 0.0,
                y: 0.0,
                width: st.layout.page_width,
                height: st.layout.page_height,
            };
            let exclusions: Vec<_> = if preceding {
                st.side_wrap_exclusions.values().cloned().collect()
            } else {
                Vec::new()
            };
            let mut plan = inline_flow::plan(
                para,
                para_index,
                styles,
                tables,
                &ObjectPlacementFrame {
                    container: &container,
                    column: &column,
                    body: &st.layout.body_area,
                    paper: &paper,
                    paragraph_y: column.y + start,
                    alignment: style.alignment,
                    dpi: self.dpi,
                },
                &exclusions,
            )?;
            plan.relative_to(column.x, column.y);
            Some(plan)
        };
        let Some(mut plan) = build(st, st.current_height, true) else {
            return false;
        };
        if !plan.carved {
            return false;
        }
        if plan.end > st.available_height() + 0.01 {
            // 새 단 후보가 실제로 수용 가능할 때만 상태를 전진한다.
            // 한 단보다 큰 혼합 문단의 fragment owner는 아직 기존 경로에 있다.
            let Some(candidate) = build(st, 0.0, false) else {
                return false;
            };
            if candidate.end > st.base_available_height() + 0.01 || st.current_items.is_empty() {
                return false;
            }
            st.advance_column_or_new_page();
            let Some(candidate) = build(st, st.current_height, false) else {
                return false;
            };
            plan = candidate;
        }
        st.current_items
            .push(PageItem::FullParagraph { para_index });
        st.current_height = plan.end;
        st.inline_box_flow_bottom = st.inline_box_flow_bottom.max(plan.end);
        st.inline_flow_plans.insert(para_index, plan);
        st.vpos_ladder_dirty = true;
        true
    }
}

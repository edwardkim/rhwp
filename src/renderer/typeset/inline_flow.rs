//! 본문 inline의 측정·fit·배치 결과 소유자.

pub(super) mod plan;

use super::{TypesetEngine, TypesetState};
use crate::model::control::Control;
use crate::model::paragraph::Paragraph;
use crate::renderer::height_measurer::MeasuredTable;
use crate::renderer::inline_flow;
use crate::renderer::style_resolver::ResolvedStyleSet;

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
        if !inline_flow::supports(para, super::super::px_to_hwpunit(column.width, self.dpi))
            && !inline_flow::supports_plain_text(para)
        {
            return false;
        }
        if st.side_wrap_exclusions.is_empty()
            && !para.controls.iter().any(|c| {
                matches!(c, Control::Picture(p) if !p.common.treat_as_char
                && p.common.text_wrap == crate::model::shape::TextWrap::Square)
            })
        {
            // 어울림 입력이 없는 기존 inline 문단에서는 토큰/plan도 만들지 않는다.
            return false;
        }
        let build = |st: &TypesetState, start: f64, preceding: bool| {
            plan::build_plan(
                st.inline_flow_input(start, preceding),
                para,
                para_index,
                styles,
                tables,
                self.dpi,
            )
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
            let Some(candidate) = build(st, st.current_height, true) else {
                return false;
            };
            if candidate.end > st.available_height() + 0.01 {
                // 실제 다음 단의 폭/각주 예산이 예비 후보와 다르면 기존 분할기로 넘긴다.
                // 수용 불가능한 plan을 확정 metadata로 게시하지 않는다.
                return false;
            }
            plan = candidate;
        }
        st.commit_inline_flow(para_index, plan);
        true
    }
}

//! 본문 inline의 측정·fit·배치 결과 소유자.

#[path = "inline_flow/plan.rs"]
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
        match self.typeset_inline_flow_with_storage(
            st,
            para_index,
            para,
            styles,
            tables,
            &crate::storage::Unbounded,
        ) {
            Ok(value) => value,
            Err(never) => match never {},
        }
    }

    pub(super) fn typeset_inline_flow_with_storage<S: crate::storage::Storage>(
        &self,
        st: &mut TypesetState,
        para_index: usize,
        para: &Paragraph,
        styles: &ResolvedStyleSet,
        tables: &[MeasuredTable],
        storage: &S,
    ) -> Result<bool, S::Error> {
        let column = st.inline_flow_column();
        let table_text_rows = inline_flow::supports_table_text_rows_with_storage(para, storage)?;
        let picture_rows = inline_flow::supports_picture_rows(para);
        // [#8282] 저장 줄이 있는 본문은 증언 없는 어울림 개체가 있을 때만 후보다.
        let reflow_stored_text =
            !st.unwitnessed_side_wraps.is_empty() && inline_flow::supports_stored_plain_text(para);
        if !inline_flow::supports(para, super::super::px_to_hwpunit(column.width, self.dpi))
            && !inline_flow::supports_plain_text(para)
            && !inline_flow::supports_square_float_host(para)
            && !reflow_stored_text
        {
            return Ok(false);
        }
        if !table_text_rows
            && !picture_rows
            && st.side_wrap_exclusions.is_empty()
            && !para
                .controls
                .iter()
                .any(crate::renderer::composer::control_is_square_float_object)
        {
            // 어울림 입력이 없는 기존 inline 문단에서는 토큰/plan도 만들지 않는다.
            return Ok(false);
        }
        let build = |st: &TypesetState, start: f64, preceding: bool| {
            let mut input = st.inline_flow_input(start, preceding);
            input.reflow_stored_text = reflow_stored_text;
            plan::build_plan_with_storage(
                input, para, para_index, styles, tables, self.dpi, storage,
            )
        };
        let Some(mut plan) = build(st, st.current_height, true)? else {
            return Ok(false);
        };
        // An unchanged plain row can continue a shared cursor, but an old
        // exclusion alone does not transfer ownership from a legacy paragraph.
        // In that case its paint cursor may differ from current_height, so
        // publishing an unrelated absolute plan would rewind the next row.
        let follows_shared_rows = st
            .current_items
            .last()
            .and_then(|item| st.inline_flow_plans.get(&item.para_index()))
            .is_some_and(|previous| (previous.end - st.current_height).abs() < 0.01);
        if !table_text_rows
            && !picture_rows
            && !plan.carved
            && (plan.text_rows.is_none() || !follows_shared_rows)
        {
            return Ok(false);
        }
        if reflow_stored_text
            && !plan.carved_by_any(
                st.unwitnessed_side_wraps
                    .iter()
                    .filter_map(|key| st.side_wrap_exclusions.get(key)),
                column.y,
                self.dpi,
            )
        {
            // 저장 줄은 증언된 개체에 대해서는 정본이다. 증언 없는 개체와 겹칠 때만 대체한다.
            return Ok(false);
        }
        // 마지막 줄 뒤 간격은 다음 흐름의 전진량이다. 표 경로와 같은
        // 실제 행·개체 하단으로 수용하고, 확정 뒤에는 end 전체를 전진한다.
        if plan.occupied_end() > st.available_height() + 0.01 {
            // 새 단 후보가 실제로 수용 가능할 때만 상태를 전진한다.
            // 한 단보다 큰 혼합 문단의 fragment owner는 아직 기존 경로에 있다.
            let Some(candidate) = build(st, 0.0, false)? else {
                return Ok(false);
            };
            if candidate.occupied_end() > st.base_available_height() + 0.01
                || st.current_items.is_empty()
            {
                return Ok(false);
            }
            st.advance_column_or_new_page_with_storage(storage)?;
            let Some(candidate) = build(st, st.current_height, true)? else {
                return Ok(false);
            };
            if candidate.occupied_end() > st.available_height() + 0.01 {
                // 실제 다음 단의 폭/각주 예산이 예비 후보와 다르면 기존 분할기로 넘긴다.
                // 수용 불가능한 plan을 확정 metadata로 게시하지 않는다.
                return Ok(false);
            }
            plan = candidate;
        }
        st.commit_inline_flow_with_storage(para_index, plan, storage)?;
        Ok(true)
    }
}

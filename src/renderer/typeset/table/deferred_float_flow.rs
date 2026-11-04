//! 이월한 부동 표의 호스트 글줄과 먼저 채운 본문을 같은 포맷 결과로 배치한다.

use crate::model::paragraph::ColumnBreakType;
use crate::renderer::typeset::{
    next_boundary_reverts_spacing_trim, para_has_visible_text, para_near_rowbreak_table,
    spacing_trim_restorable, ComposedParagraph, PageItem, Paragraph, ResolvedStyleSet,
    TypesetEngine, TypesetState,
};

pub(in crate::renderer::typeset) const MAX_PREFILL: usize = 8;

impl TypesetEngine {
    pub(in crate::renderer::typeset) fn pre_emit_deferred_host_lines(
        &self,
        st: &mut TypesetState,
        para_idx: usize,
        para: &Paragraph,
        composed_all: &[ComposedParagraph],
        styles: &ResolvedStyleSet,
    ) -> bool {
        if st.col_count != 1 || st.current_items.is_empty() {
            return false;
        }
        if st.pre_emitted_host_paras.contains(&para_idx) {
            return true;
        }
        let already_emitted = st.current_items.iter().any(|item| {
            matches!(item, PageItem::FullParagraph { para_index }
                | PageItem::PartialParagraph { para_index, start_line: 0, .. }
                if *para_index == para_idx)
        });
        if already_emitted {
            st.mark_pre_emitted_host(para_idx);
            return true;
        }
        let col_w = st
            .layout
            .column_areas
            .get(st.current_column as usize)
            .map(|a| a.width)
            .unwrap_or(st.layout.body_area.width);
        let host_fmt = self.format_paragraph(para, composed_all.get(para_idx), styles, Some(col_w));
        let host_lines = host_fmt.line_heights.len();
        let host_h = host_fmt.line_advances_sum(0..host_lines);
        // [Task #1763] 저장 flow 로 같은 쪽 후보임이 확인된 경우와 동일하게, host
        // 줄 자체는 추가 안전마진 없이 본문 가용 높이로 판정한다.
        //
        // [#6184] 단 **fit 판정에는 말미 줄간격을 빼고** 잰다 — 쪽 마지막 줄 뒤에는
        // 다음 줄이 없으므로 그 간격은 쪽을 채우지 않는다(#359 의 `height_for_fit`
        // 규약이고, 바로 아래 후속 문단 루프도 이미 그 값을 쓴다). 포함해서 재면
        // 156489124 pi=324 가 24.0px 로 잡혀 잔여 16.6px 를 넘겨 탈락하고, 줄이
        // 표와 함께 다음 쪽으로 밀려 그 쪽 흐름표 위에 겹쳐 그려진다. 한글은 같은
        // 줄을 1031.2..1047.2 로 본문 하단(1046.9)을 0.3px 넘겨 놓는다 — 말미
        // 간격을 요구하지 않는다는 뜻이다. 누적(advance)은 종전대로 전량.
        let host_fit = host_fmt.height_for_fit.min(host_h);
        if host_lines == 0 || st.current_height + host_fit > st.available_height() {
            return false;
        }
        st.append_item(PageItem::PartialParagraph {
            para_index: para_idx,
            start_line: 0,
            end_line: host_lines,
        });
        st.advance_flow_by(host_h);
        st.mark_pre_emitted_host(para_idx);
        // [#2015] vert_offset 이중계상 보정용 host 높이 기록.
        let host_trailing_spacing = host_fmt
            .line_spacings
            .last()
            .copied()
            .unwrap_or(0.0)
            .max(0.0);
        st.record_pre_emitted_host_height(para_idx, host_h, host_h - host_trailing_spacing);
        true
    }

    /// 저장 사다리가 없는 빈 호스트도 자신의 글줄을 현재 쪽에 남긴다.
    /// 부모 셀의 원자 밴드로 실제 이월이 확정된 호출 경로에서만 사용한다.
    #[allow(clippy::too_many_arguments)]
    pub(in crate::renderer::typeset) fn prefill_computed_deferred_float(
        &self,
        st: &mut TypesetState,
        para_idx: usize,
        para: &Paragraph,
        paragraphs_all: &[Paragraph],
        composed_all: &[ComposedParagraph],
        styles: &ResolvedStyleSet,
    ) {
        if para_has_visible_text(para)
            || !crate::renderer::para_has_no_stored_line_segs(para)
            || !self.pre_emit_deferred_host_lines(st, para_idx, para, composed_all, styles)
        {
            return;
        }
        self.prefill_computed_float_successors(
            st,
            para_idx,
            para,
            paragraphs_all,
            composed_all,
            styles,
        );
    }

    /// 호스트 프레임은 남고 독립 자식만 전량 이월될 때도 같은 후속 흐름을 채운다.
    #[allow(clippy::too_many_arguments)]
    pub(in crate::renderer::typeset) fn prefill_computed_float_successors(
        &self,
        st: &mut TypesetState,
        para_idx: usize,
        para: &Paragraph,
        paragraphs_all: &[Paragraph],
        composed_all: &[ComposedParagraph],
        styles: &ResolvedStyleSet,
    ) -> bool {
        if st.col_count != 1
            || st.current_items.is_empty()
            || para_has_visible_text(para)
            || !crate::renderer::para_has_no_stored_line_segs(para)
        {
            return false;
        }
        for next_idx in (para_idx + 1)..paragraphs_all.len().min(para_idx + 1 + MAX_PREFILL) {
            let next = &paragraphs_all[next_idx];
            if !next.controls.is_empty()
                || next.column_type != ColumnBreakType::None
                || !crate::renderer::para_has_no_stored_line_segs(next)
                || !self.try_append_prefilled_paragraph(
                    st,
                    next_idx,
                    paragraphs_all,
                    composed_all,
                    styles,
                )
            {
                break;
            }
        }
        true
    }

    /// 저장/재조판 경로가 같은 포맷·말미 간격·실제 본문 전진을 소비한다.
    pub(in crate::renderer::typeset) fn try_append_prefilled_paragraph(
        &self,
        st: &mut TypesetState,
        next_idx: usize,
        paragraphs_all: &[Paragraph],
        composed_all: &[ComposedParagraph],
        styles: &ResolvedStyleSet,
    ) -> bool {
        let next = &paragraphs_all[next_idx];
        let col_w = st
            .layout
            .column_areas
            .get(st.current_column as usize)
            .map(|a| a.width)
            .unwrap_or(st.layout.body_area.width);
        let fmt_n = self.format_paragraph(next, composed_all.get(next_idx), styles, Some(col_w));
        if st.current_height + fmt_n.height_for_fit > st.available_height() {
            return false;
        }
        let trim_sb =
            !st.profile.hwp3_layout() && !para_near_rowbreak_table(paragraphs_all, next_idx);
        st.append_item(PageItem::FullParagraph {
            para_index: next_idx,
        });
        st.advance_flow_by(fmt_n.flow_advance_height(
            next,
            st.col_count,
            trim_sb,
            st.vpos_ladder_dirty
                || st.vpos_page_base.or(st.vpos_lazy_base).is_none()
                || !spacing_trim_restorable(paragraphs_all, next_idx)
                || next_boundary_reverts_spacing_trim(
                    st.profile.hwpx_stored_layout() && !st.profile.hwp3_layout(),
                    paragraphs_all,
                    styles,
                    next_idx,
                    self.dpi,
                ),
            false,
        ));
        st.mark_prefilled_paragraph(next_idx);
        true
    }
}

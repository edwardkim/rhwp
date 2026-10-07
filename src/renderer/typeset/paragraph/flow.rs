//! 일반 문단의 fit/배치 담당자를 기존 순서대로 연결하는 조정 경계.
//! 판별 알고리즘은 각 Query에, 페이지 상태 변경은 기존 Command에 남긴다.
//! 엔진 profile은 예산 준비 시, state profile은 강제 경계 준비 후에 각각 읽는다.

use crate::model::paragraph::Paragraph;
use crate::renderer::pagination::PageItem;
use crate::renderer::style_resolver::ResolvedStyleSet;
use crate::renderer::typeset::paragraph::{self, metrics::FormattedParagraph};
use crate::renderer::typeset::TypesetState;

pub(in crate::renderer::typeset) struct ParagraphFlowInput<'a> {
    pub para_idx: usize,
    pub para: &'a Paragraph,
    pub fmt: &'a FormattedParagraph,
    pub paragraphs: &'a [Paragraph],
    pub styles: &'a ResolvedStyleSet,
    pub is_last_in_section: bool,
}

pub(in crate::renderer::typeset) fn place(
    st: &mut TypesetState,
    input: ParagraphFlowInput<'_>,
    dpi: f64,
    session_edited: impl FnOnce() -> bool,
) {
    let ParagraphFlowInput {
        para_idx,
        para,
        fmt,
        paragraphs,
        styles,
        is_last_in_section,
    } = input;
    let stored_frame_shared_spacing =
        crate::renderer::float_placement::stored_frame_successor_shared_spacing_px(
            &st.paragraph_float_placements,
            para_idx,
            fmt.spacing_before,
            st.current_height,
        );
    st.reclaim_flow_by(stored_frame_shared_spacing.min(st.current_height));
    let previous_is_partial_table = st.current_items.last().is_some_and(
        |item| matches!(item, PageItem::PartialTable { para_index, .. } if *para_index < para_idx),
    );
    let shared_spacing =
        crate::renderer::float_placement::stored_after_partial_table_shared_spacing_px(
            (st.profile.hwpx_stored_layout() || st.profile.hwp5_stored_pagination_layout())
                && !st.profile.session_edited(),
            previous_is_partial_table,
            para,
            fmt.spacing_before,
            st.current_height,
            dpi,
        );
    st.reclaim_flow_by(shared_spacing.min(st.current_height));
    let paragraph::FitBudget {
        strict_after_empty_host_float,
        layout_drift_safety_px,
        prev_is_partial_table,
        available,
    } = paragraph::prepare_fit_budget(st, para_idx, para, fmt, paragraphs, session_edited(), dpi);

    if paragraph::try_absorb_rowbreak_guide(st, prev_is_partial_table, para, paragraphs, para_idx) {
        return;
    }

    if paragraph::try_place_multicolumn_paragraph(st, para_idx, para, fmt, dpi) {
        return;
    }

    if paragraph::try_absorb_empty_paragraph(
        st,
        para_idx,
        para,
        fmt,
        is_last_in_section,
        available,
        layout_drift_safety_px,
    ) {
        return;
    }

    let paragraph::ForcedPageBoundary {
        native_hwp5_existing_footnote_reset_line,
        current_page_vpos_base,
        forced_page_break_line,
    } = paragraph::prepare_forced_page_boundary(
        st, para_idx, para, fmt, paragraphs, available, dpi,
    );
    if st.prefilled_line_prefixes.contains_key(&para_idx) {
        // 앞 쪽의 선행 조각이 소유한 줄을 whole-fit으로 다시 배치하지 않는다.
        let base_available = (st.base_available_height() - layout_drift_safety_px).max(0.0);
        paragraph::place_split_paragraph(
            st,
            para_idx,
            para,
            fmt,
            paragraphs,
            fmt.line_count(),
            base_available,
            layout_drift_safety_px,
            forced_page_break_line,
            native_hwp5_existing_footnote_reset_line,
            current_page_vpos_base,
            false,
            dpi,
        );
        return;
    }
    let paragraph::metrics::ParagraphFlowHints {
        body_bottom_vpos,
        trim_spacing_before_for_flow,
        trimmed_sb_gate,
    } = paragraph::metrics::flow_hints(
        para,
        fmt,
        paragraphs,
        para_idx,
        st.paragraph_flow_profile(),
        dpi,
    );

    let paragraph::WholeFitDecision {
        fits,
        stored_vpos_rewind_break,
        stored_vpos_rewind_overflow_break,
    } = paragraph::decide_whole_fit(
        st,
        para_idx,
        para,
        fmt,
        paragraphs,
        strict_after_empty_host_float,
        forced_page_break_line,
        current_page_vpos_base,
        available,
        dpi,
    );
    if fits {
        paragraph::place_fitted_paragraph(
            st,
            para_idx,
            para,
            fmt,
            paragraphs,
            styles,
            trim_spacing_before_for_flow,
            trimmed_sb_gate,
            body_bottom_vpos,
            dpi,
        );
        return;
    }

    if paragraph::try_place_overflow_paragraph(
        st,
        para_idx,
        para,
        fmt,
        paragraphs,
        styles,
        trim_spacing_before_for_flow,
        body_bottom_vpos,
        available,
        forced_page_break_line,
        dpi,
    ) {
        return;
    }

    paragraph::place_after_failed_fit(
        st,
        para_idx,
        para,
        fmt,
        paragraphs,
        styles,
        trim_spacing_before_for_flow,
        trimmed_sb_gate,
        body_bottom_vpos,
        available,
        layout_drift_safety_px,
        stored_vpos_rewind_break,
        stored_vpos_rewind_overflow_break,
        forced_page_break_line,
        native_hwp5_existing_footnote_reset_line,
        current_page_vpos_base,
        dpi,
    );
}

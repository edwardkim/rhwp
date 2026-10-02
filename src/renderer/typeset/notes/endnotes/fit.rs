//! 미주 수용·이월 후보 조회. 페이지 상태는 읽기 전용이다.

use crate::renderer::typeset::notes::endnotes::profile::{
    endnote_between_notes_margin, endnote_has_compact_separator_below,
    endnote_separator_below_margin, ENDNOTE_BETWEEN_NOTES_BASE_FLOW_HU,
};
use crate::renderer::typeset::{
    hwpunit_to_px, line_has_visible_text, line_has_visible_text_or_tac_equation,
    line_is_equation_tac_text_run_only, page_item_para_index, para_has_non_tac_picture_or_shape,
    para_has_treat_as_char_picture_or_shape, para_has_visible_text,
    para_has_visible_text_and_treat_as_char_equation, para_has_visible_text_or_equation,
    para_is_short_auto_endnote_marker, para_is_treat_as_char_picture_only,
    paragraph_by_global_index, ComposedParagraph, Control, EndnoteRef, FootnoteShape,
    FormattedParagraph, Paragraph, ResolvedStyleSet, TypesetEngine, TypesetState,
    ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX,
};

impl TypesetEngine {
    /// [Task #2106] P6/P7 판정: compact_endnote_own_vpos_span_fits (원본 무변경 이동, R9 패턴).
    #[allow(clippy::too_many_arguments)]
    pub(in crate::renderer::typeset) fn judge_compact_endnote_own_vpos_span_fits(
        &self,
        dpi: f64,
        this_content_bottom_offset: Option<i32>,
        remaining_height: f64,
        st: &TypesetState,
        available: f64,
        compact_between_notes_gap: bool,
        compact_endnote_separator_profile: bool,
        endnote_has_visible_payload: bool,
        internal_vpos_rewind: bool,
        local_vpos_rewind: bool,
        non_tac_object_height: Option<f64>,
        this_first_offset: Option<i32>,
    ) -> bool {
        compact_endnote_separator_profile
            && st.col_count > 1
            && st.current_height < available
            && compact_between_notes_gap
            && !local_vpos_rewind
            && (!internal_vpos_rewind || (st.current_items.is_empty() && st.current_height <= 1.0))
            && endnote_has_visible_payload
            && non_tac_object_height
                .map(|height| {
                    height <= remaining_height + ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX + 1.0
                })
                .unwrap_or(true)
            && matches!(
                (this_first_offset, this_content_bottom_offset),
                (Some(first), Some(bottom))
                    if hwpunit_to_px((bottom - first).max(0), dpi)
                        <= remaining_height
                            + ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX
                            + 1.0
            )
    }

    /// [Task #2106] P6/P7 판정: internal_rewind_full_advance_needed (원본 무변경 이동, R9 패턴).
    #[allow(clippy::too_many_arguments)]
    pub(in crate::renderer::typeset) fn judge_internal_rewind_full_advance_needed(
        &self,
        internal_rewind_target_is_reset: bool,
        internal_rewind_split: Option<usize>,
        st: &TypesetState,
        en_para: &Paragraph,
        available: f64,
        compact_endnote_separator_profile: bool,
        default_between_notes_gap: bool,
        en_fit: f64,
        endnote_has_visible_payload: bool,
        has_visible_endnote_separator: bool,
        internal_vpos_rewind: bool,
        split_endnote_to_fit: Option<usize>,
        total_advance_fit: f64,
    ) -> bool {
        internal_rewind_split
            .filter(|split| *split > 1)
            .filter(|split| split_endnote_to_fit.is_some_and(|fit_split| fit_split > *split))
            .filter(|_| {
                default_between_notes_gap
                    && compact_endnote_separator_profile
                    && has_visible_endnote_separator
                    && internal_vpos_rewind
                    && internal_rewind_target_is_reset
                    && st.col_count > 1
                    && st.current_column + 1 < st.col_count
                    && st.current_height > available * 0.90
                    && !st.current_items.is_empty()
                    && st.current_height + en_fit
                        <= available + ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX
                    && st.current_height + total_advance_fit
                        > available + ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX
                    && endnote_has_visible_payload
            })
            .and_then(|split| {
                let first = en_para.line_segs.first()?;
                let target = en_para.line_segs.get(split)?;
                (target.vertical_pos < first.vertical_pos).then_some(true)
            })
            .unwrap_or(false)
    }

    /// [Task #2106] P6/P7 판정: no_separator_tail_table_starts_next_column (원본 무변경 이동, R9 패턴).
    #[allow(clippy::too_many_arguments)]
    pub(in crate::renderer::typeset) fn judge_no_separator_tail_table_starts_next_column(
        &self,
        table_only_endnote_para_before_rewind: bool,
        st: &TypesetState,
        en_ctrl: &crate::model::footnote::Endnote,
        available: f64,
        endnote_start: i32,
        ep_idx: usize,
        has_visible_endnote_separator: bool,
        internal_vpos_rewind: bool,
        large_separator_block: bool,
        local_vpos_rewind: bool,
        this_first_offset: Option<i32>,
    ) -> bool {
        large_separator_block
            && !has_visible_endnote_separator
            && ep_idx > 0
            && st.col_count > 1
            && st.current_column + 1 < st.col_count
            && st.current_height > available * 0.95
            && !st.current_items.is_empty()
            && !local_vpos_rewind
            && !internal_vpos_rewind
            && table_only_endnote_para_before_rewind
            && en_ctrl.paragraphs.get(ep_idx + 1).is_some_and(|next_para| {
                matches!(
                    (
                        this_first_offset,
                        next_para
                            .line_segs
                            .first()
                            .map(|s| s.vertical_pos + endnote_start),
                    ),
                    (Some(cur), Some(next)) if next < cur
                )
            })
    }

    /// [Task #2106] P6/P7 판정: large_between_equation_tail_starts_next_column (원본 무변경 이동, R9 패턴).
    #[allow(clippy::too_many_arguments)]
    pub(in crate::renderer::typeset) fn judge_large_between_equation_tail_starts_next_column(
        &self,
        st: &TypesetState,
        fmt: &FormattedParagraph,
        en_para: &Paragraph,
        en_ctrl: &crate::model::footnote::Endnote,
        styles: &ResolvedStyleSet,
        composed: &ComposedParagraph,
        available: f64,
        compact_endnote_separator_profile: bool,
        default_between_notes_gap: bool,
        en_col_w: f64,
        ep_idx: usize,
        has_visible_endnote_separator: bool,
        internal_vpos_rewind: bool,
        large_between_small_equation_tail_bleeds_previous_column: bool,
        local_vpos_rewind: bool,
    ) -> bool {
        !default_between_notes_gap
            && compact_endnote_separator_profile
            && has_visible_endnote_separator
            && ep_idx > 0
            && st.col_count > 1
            && st.current_column + 1 < st.col_count
            && st.current_height > available * 0.90
            && st.current_height + fmt.line_advance(0) > available - 50.0
            && !st.current_items.is_empty()
            && !local_vpos_rewind
            && !internal_vpos_rewind
            && fmt.line_heights.len() == 1
            && !large_between_small_equation_tail_bleeds_previous_column
            && line_is_equation_tac_text_run_only(en_para, &composed, 0)
            && en_ctrl.paragraphs.get(ep_idx + 1).is_some_and(|next_para| {
                let next_comp =
                    crate::renderer::composer::compose_paragraph_in_context(next_para, styles);
                let next_fmt =
                    self.format_paragraph(next_para, Some(&next_comp), &styles, Some(en_col_w));
                next_fmt.line_heights.len() == 1
                    && next_fmt.line_advance(0) <= 24.0
                    && line_has_visible_text(&next_comp, 0)
                    && !para_has_treat_as_char_picture_or_shape(next_para)
            })
    }

    /// [Task #2106] P6/P7 판정: large_between_last_column_flow_tail_split (원본 무변경 이동, R9 패턴).
    #[allow(clippy::too_many_arguments)]
    pub(in crate::renderer::typeset) fn judge_large_between_last_column_flow_tail_split(
        &self,
        st: &TypesetState,
        fmt: &FormattedParagraph,
        en_para: &Paragraph,
        available: f64,
        compact_between_notes_gap: bool,
        compact_endnote_separator_profile: bool,
        default_between_notes_gap: bool,
        en_fit: f64,
        ep_idx: usize,
        has_visible_endnote_separator: bool,
        internal_vpos_rewind: bool,
        local_vpos_rewind: bool,
        visible_large_between_notes_gap: bool,
        zero_endnote_spacing_profile: bool,
    ) -> Option<usize> {
        if !default_between_notes_gap
            && compact_endnote_separator_profile
            && has_visible_endnote_separator
            && visible_large_between_notes_gap
            && !compact_between_notes_gap
            && !zero_endnote_spacing_profile
            && ep_idx > 0
            && st.col_count > 1
            && st.current_column + 1 >= st.col_count
            && st.current_height < available
            && st.current_height + en_fit > available - 60.0
            && st.current_height + en_fit
                <= available + ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX + 2.0
            && !st.current_items.is_empty()
            && !local_vpos_rewind
            && !internal_vpos_rewind
            && fmt.line_heights.len() >= 5
            && para_has_visible_text_or_equation(en_para)
            && !para_has_treat_as_char_picture_or_shape(en_para)
            && !para_has_non_tac_picture_or_shape(en_para)
        {
            Some(fmt.line_heights.len() - 1)
        } else {
            None
        }
    }

    /// [Task #2106] P6/P7 판정: allow_compact_question_title_tail (원본 무변경 이동, R9 패턴).
    #[allow(clippy::too_many_arguments)]
    pub(in crate::renderer::typeset) fn judge_allow_compact_question_title_tail(
        &self,
        new_endnote_between_notes_px: Option<f64>,
        new_endnote_advance_threshold: f64,
        no_separator_zero_between_notes: bool,
        st: &TypesetState,
        fmt: &FormattedParagraph,
        available: f64,
        compact_endnote_separator_profile: bool,
        default_between_notes_gap: bool,
        en_fit: f64,
        endnote_has_visible_payload: bool,
        ep_idx: usize,
        has_visible_endnote_separator: bool,
        large_separator_block: bool,
    ) -> bool {
        compact_endnote_separator_profile
            && !default_between_notes_gap
            && (has_visible_endnote_separator
                || !large_separator_block
                || no_separator_zero_between_notes)
            && ep_idx == 0
            && st.current_column + 1 < st.col_count
            && fmt.line_heights.len() == 1
            && endnote_has_visible_payload
            && st.current_height > available * new_endnote_advance_threshold
            && new_endnote_between_notes_px
                .map(|gap| {
                    st.current_height + fmt.line_advance(0) + gap
                        <= available + ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX
                })
                .unwrap_or(true)
            && st.current_height + en_fit <= available + ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX
    }

    /// [Task #2106] P6/P7 판정: large_between_last_column_render_title_tail_fits (원본 무변경 이동, R9 패턴).
    #[allow(clippy::too_many_arguments)]
    pub(in crate::renderer::typeset) fn judge_large_between_last_column_render_title_tail_fits(
        &self,
        large_between_question_title_render_y: Option<f64>,
        large_between_question_title_head_inside_frame: bool,
        st: &TypesetState,
        fmt: &FormattedParagraph,
        en_ref: &EndnoteRef,
        available: f64,
        compact_between_notes_gap: bool,
        compact_endnote_separator_profile: bool,
        default_between_notes_gap: bool,
        emitted_endnote_count: usize,
        endnote_has_visible_payload: bool,
        ep_idx: usize,
        has_visible_endnote_separator: bool,
        visible_large_between_notes_gap: bool,
    ) -> bool {
        !default_between_notes_gap
            && compact_endnote_separator_profile
            && has_visible_endnote_separator
            && visible_large_between_notes_gap
            && !compact_between_notes_gap
            && ep_idx == 0
            && emitted_endnote_count > 0
            && en_ref.number > 0
            && st.current_column + 1 >= st.col_count
            && fmt.line_heights.len() == 1
            && st.current_height > available * 0.80
            && st.current_height < available * 0.85
            && !st.current_items.is_empty()
            && large_between_question_title_head_inside_frame
            && large_between_question_title_render_y
                .map(|predicted_y| {
                    // 마지막 단의 20mm급 `미주 사이`는 제목 앞 렌더 gap을
                    // 만든 뒤 제목 한 줄만 쪽 하단에 남길 수 있다. 본문
                    // head group까지 같은 쪽에 들어가야 한다고 보면 문항
                    // 시작이 한컴보다 한 쪽 늦어진다.
                    predicted_y + fmt.line_advance(0)
                        <= available + ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX + 2.0
                })
                .unwrap_or(false)
            && endnote_has_visible_payload
    }

    /// [Task #2106] P6/P7 판정: large_between_last_column_rewind_title_tail_fits (원본 무변경 이동, R9 패턴).
    #[allow(clippy::too_many_arguments)]
    pub(in crate::renderer::typeset) fn judge_large_between_last_column_rewind_title_tail_fits(
        &self,
        large_between_question_title_head_inside_frame: bool,
        st: &TypesetState,
        fmt: &FormattedParagraph,
        en_ref: &EndnoteRef,
        available: f64,
        compact_between_notes_gap: bool,
        compact_endnote_separator_profile: bool,
        default_between_notes_gap: bool,
        emitted_endnote_count: usize,
        en_fit: f64,
        endnote_has_visible_payload: bool,
        endnote_has_vpos_rewind: bool,
        ep_idx: usize,
        has_visible_endnote_separator: bool,
        visible_large_between_notes_gap: bool,
    ) -> bool {
        !default_between_notes_gap
            && compact_endnote_separator_profile
            && has_visible_endnote_separator
            && visible_large_between_notes_gap
            && !compact_between_notes_gap
            && ep_idx == 0
            && emitted_endnote_count > 0
            && en_ref.number > 0
            && st.current_column + 1 >= st.col_count
            && fmt.line_heights.len() == 1
            && endnote_has_vpos_rewind
            && st.current_height > available * 0.90
            && st.current_height < available
            && !st.current_items.is_empty()
            && st.current_height + en_fit
                <= available + ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX + 2.0
            && large_between_question_title_head_inside_frame
            && endnote_has_visible_payload
    }

    /// [Task #2106] P6/P7 판정: zero_question_title_tail_fits_by_line_height (원본 무변경 이동, R9 패턴).
    #[allow(clippy::too_many_arguments)]
    pub(in crate::renderer::typeset) fn judge_zero_question_title_tail_fits_by_line_height(
        &self,
        st: &TypesetState,
        fmt: &FormattedParagraph,
        available: f64,
        compact_endnote_separator_profile: bool,
        endnote_has_visible_payload: bool,
        ep_idx: usize,
        h4f: f64,
        has_visible_endnote_separator: bool,
        zero_endnote_spacing_profile: bool,
    ) -> bool {
        compact_endnote_separator_profile
                && zero_endnote_spacing_profile
                && has_visible_endnote_separator
                && ep_idx == 0
                && fmt.line_heights.len() == 1
                // 0/0/0 미주는 한컴이 새 문항 제목 한 줄을
                // 왼쪽 단 하단에 남기고 큰 그림 풀이만 다음 단으로
                // 넘기는 경우가 있다. 기본 미주의 0.95 임계값을
                // 그대로 쓰면 제목까지 다음 단 상단으로 밀린다.
                && st.current_height > available * 0.88
                && st.current_height + h4f
                    <= available + ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX
                && (st.current_column + 1 >= st.col_count
                    || st.current_height + fmt.line_advance(0)
                        <= available + 1.0
                    || st.current_height + h4f
                        <= available
                            + ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX)
                && endnote_has_visible_payload
    }

    /// [Task #2106] P6/P7 판정: zero_between_large_separator_tail_group_fits (원본 무변경 이동, R9 패턴).
    #[allow(clippy::too_many_arguments)]
    pub(in crate::renderer::typeset) fn judge_zero_between_large_separator_tail_group_fits(
        &self,
        zero_between_large_separator_margin: bool,
        st: &TypesetState,
        fmt: &FormattedParagraph,
        en_para: &Paragraph,
        en_ref: &EndnoteRef,
        endnote_shape: Option<&FootnoteShape>,
        available: f64,
        compact_endnote_separator_profile: bool,
        ep_idx: usize,
        has_visible_endnote_separator: bool,
        internal_vpos_rewind: bool,
        large_separator_block: bool,
        later_endnote_vpos_rewinds_after_current: bool,
        local_vpos_rewind: bool,
    ) -> bool {
        compact_endnote_separator_profile
            && has_visible_endnote_separator
            && large_separator_block
            && endnote_shape
                .map(|shape| endnote_between_notes_margin(shape) == 0)
                .unwrap_or(false)
            && st.col_count > 1
            && st.current_column + 1 < st.col_count
            && ep_idx > 0
            && en_ref.number > 0
            && fmt.line_heights.len() == 1
            && !internal_vpos_rewind
            && !para_is_treat_as_char_picture_only(en_para)
            && para_has_visible_text_or_equation(en_para)
            && st.current_height > available * 0.95
            && st.current_height + fmt.line_advance(0)
                <= available + ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX + 70.0
            && !local_vpos_rewind
            && later_endnote_vpos_rewinds_after_current
    }

    /// [Task #2106] P6/P7 판정: zero_visible_text_tail_before_rewind_fits (원본 무변경 이동, R9 패턴).
    #[allow(clippy::too_many_arguments)]
    pub(in crate::renderer::typeset) fn judge_zero_visible_text_tail_before_rewind_fits(
        &self,
        st: &TypesetState,
        fmt: &FormattedParagraph,
        en_para: &Paragraph,
        en_ref: &EndnoteRef,
        composed: &ComposedParagraph,
        available: f64,
        compact_endnote_separator_profile: bool,
        en_fit: f64,
        ep_idx: usize,
        has_visible_endnote_separator: bool,
        internal_vpos_rewind: bool,
        later_endnote_vpos_rewinds_after_current: bool,
        local_vpos_rewind: bool,
        zero_endnote_spacing_profile: bool,
    ) -> bool {
        compact_endnote_separator_profile
            && zero_endnote_spacing_profile
            && has_visible_endnote_separator
            && st.col_count > 1
            && st.current_column + 1 < st.col_count
            && ep_idx > 0
            && en_ref.number > 0
            && fmt.line_heights.len() <= 2
            && !local_vpos_rewind
            && !internal_vpos_rewind
            && later_endnote_vpos_rewinds_after_current
            && !para_is_treat_as_char_picture_only(en_para)
            && !para_has_non_tac_picture_or_shape(en_para)
            && para_has_visible_text_or_equation(en_para)
            && line_has_visible_text(&composed, 0)
            && st.current_height > available * 0.96
            && st.current_height + en_fit
                <= available + ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX + 28.0
    }

    /// [Task #2106] P6/P7 판정: pre_emit_tail_before_non_tac_object_advance (원본 무변경 이동, R9 패턴).
    #[allow(clippy::too_many_arguments)]
    pub(in crate::renderer::typeset) fn judge_pre_emit_tail_before_non_tac_object_advance(
        &self,
        zero_between_large_separator_margin: bool,
        advance_for_fit: bool,
        st: &TypesetState,
        en_ctrl: &crate::model::footnote::Endnote,
        styles: &ResolvedStyleSet,
        endnote_shape: Option<&FootnoteShape>,
        available: f64,
        compact_endnote_separator_profile: bool,
        en_col_w: f64,
        endnote_has_text_or_equation: bool,
        ep_idx: usize,
        has_visible_endnote_separator: bool,
        large_separator_block: bool,
        non_tac_object_height: Option<f64>,
    ) -> bool {
        advance_for_fit
            && compact_endnote_separator_profile
            && has_visible_endnote_separator
            && (large_separator_block || zero_between_large_separator_margin)
            && endnote_shape
                .map(|shape| endnote_between_notes_margin(shape) == 0)
                .unwrap_or(false)
            && st.col_count > 1
            && st.current_column + 1 < st.col_count
            && non_tac_object_height.is_some()
            && !endnote_has_text_or_equation
            && ep_idx + 1 < en_ctrl.paragraphs.len()
            && st.current_height > available * 0.90
            && en_ctrl.paragraphs.get(ep_idx + 1).is_some_and(|next_para| {
                let next_comp =
                    crate::renderer::composer::compose_paragraph_in_context(next_para, styles);
                let next_fmt =
                    self.format_paragraph(next_para, Some(&next_comp), &styles, Some(en_col_w));
                para_has_visible_text_or_equation(next_para)
                    && !para_has_non_tac_picture_or_shape(next_para)
                    && !para_has_treat_as_char_picture_or_shape(next_para)
                    && next_fmt.line_heights.len() == 1
                    && st.current_height + next_fmt.total_height
                        <= available + ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX + 2.0
            })
    }

    /// [Task #2079] P6 판정: 저장 vpos 기준 tail 이 frame 밖인지 (보이는 구분자 프로파일).
    #[allow(clippy::too_many_arguments)]
    pub(in crate::renderer::typeset) fn judge_visible_separator_saved_vpos_tail_outside(
        &self,
        paragraphs: &[Paragraph],
        fmt: &FormattedParagraph,
        st: &TypesetState,
        en_para: &Paragraph,
        this_first_offset: Option<i32>,
        total_advance_fit: f64,
        available: f64,
        ep_idx: usize,
        compact_endnote_separator_profile: bool,
        has_visible_endnote_separator: bool,
        endnote_has_visible_payload: bool,
        zero_endnote_spacing_profile: bool,
        local_vpos_rewind: bool,
        internal_vpos_rewind: bool,
    ) -> bool {
        compact_endnote_separator_profile
            && zero_endnote_spacing_profile
            && has_visible_endnote_separator
            && ep_idx > 0
            && st.current_column + 1 >= st.col_count
            && st.current_height > available * 0.90
            && !local_vpos_rewind
            && !internal_vpos_rewind
            && endnote_has_visible_payload
            && !(fmt.line_heights.len() == 1
                && !para_is_treat_as_char_picture_only(en_para)
                && st.current_height + total_advance_fit
                    <= available + ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX)
            && st
                .current_items
                .iter()
                .filter_map(page_item_para_index)
                .find_map(|pi| {
                    paragraph_by_global_index(paragraphs, &st.endnote_paragraphs, pi)
                        .and_then(|p| p.line_segs.first())
                        .map(|s| s.vertical_pos)
                })
                .and_then(|base_vpos| {
                    this_first_offset.map(|first_vpos| {
                        let predicted_y = hwpunit_to_px((first_vpos - base_vpos).max(0), self.dpi);
                        predicted_y + total_advance_fit
                            > available - ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX
                    })
                })
                .unwrap_or(false)
    }

    /// [Task #2079] P6 판정: rewind 그림 직전의 큰 미주 사이 tail.
    #[allow(clippy::too_many_arguments)]
    pub(in crate::renderer::typeset) fn judge_large_between_tail_before_rewind_picture(
        &self,
        fmt: &FormattedParagraph,
        st: &TypesetState,
        en_para: &Paragraph,
        en_ctrl: &crate::model::footnote::Endnote,
        this_first_offset: Option<i32>,
        endnote_start: i32,
        available: f64,
        ep_idx: usize,
        default_between_notes_gap: bool,
        compact_endnote_separator_profile: bool,
        has_visible_endnote_separator: bool,
        local_vpos_rewind: bool,
        internal_vpos_rewind: bool,
        large_separator_block: bool,
    ) -> bool {
        !default_between_notes_gap
            && compact_endnote_separator_profile
            && (has_visible_endnote_separator || !large_separator_block)
            && ep_idx > 0
            && st.col_count > 1
            && st.current_column + 1 < st.col_count
            && st.current_height > available * 0.88
            && !st.current_items.is_empty()
            && !local_vpos_rewind
            && !internal_vpos_rewind
            && fmt.line_heights.len() == 1
            && para_has_visible_text_or_equation(en_para)
            && !para_is_treat_as_char_picture_only(en_para)
            && st.current_height + fmt.line_advance(0) > available - 50.0
            && en_ctrl.paragraphs.get(ep_idx + 1).is_some_and(|next_para| {
                para_is_treat_as_char_picture_only(next_para)
                    && matches!(
                            (
                                this_first_offset,
                                next_para.line_segs.first().map(|s| {
                                    s.vertical_pos + endnote_start
                                }),
                            ),
                            (Some(cur), Some(next)) if next < cur
                    )
            })
    }

    /// [Task #2079] P6 판정: rewind 직전 마지막 단 tail 이 다음 쪽에서 시작해야 하는지.
    #[allow(clippy::too_many_arguments)]
    pub(in crate::renderer::typeset) fn judge_no_separator_last_column_tail_before_rewind_starts_next_page(
        &self,
        fmt: &FormattedParagraph,
        st: &TypesetState,
        en_para: &Paragraph,
        en_ctrl: &crate::model::footnote::Endnote,
        this_first_offset: Option<i32>,
        endnote_start: i32,
        available: f64,
        ep_idx: usize,
        has_visible_endnote_separator: bool,
        local_vpos_rewind: bool,
        internal_vpos_rewind: bool,
        large_separator_block: bool,
    ) -> bool {
        large_separator_block
            && !has_visible_endnote_separator
            && ep_idx > 0
            && st.col_count > 1
            && st.current_column + 1 >= st.col_count
            && st.current_height > available * 0.90
            && !st.current_items.is_empty()
            && !local_vpos_rewind
            && !internal_vpos_rewind
            && fmt.line_heights.len() == 1
            && para_has_visible_text_or_equation(en_para)
            && !para_has_non_tac_picture_or_shape(en_para)
            && st.current_height + fmt.line_advance(0)
                <= available + ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX + 8.0
            && en_ctrl.paragraphs.get(ep_idx + 1).is_some_and(|next_para| {
                matches!(
                    (
                        this_first_offset,
                        next_para
                            .line_segs
                            .first()
                            .map(|s| s.vertical_pos + endnote_start),
                    ),
                    (Some(cur), Some(next)) if next < cur
                )
            })
    }

    /// [Task #2079] P6 판정: 그림 뒤 텍스트 tail 이 다음 쪽에서 시작해야 하는지 (구분자 없음 프로파일).
    #[allow(clippy::too_many_arguments)]
    pub(in crate::renderer::typeset) fn judge_no_separator_tail_after_picture_starts_next_page(
        &self,
        st: &TypesetState,
        fmt: &FormattedParagraph,
        en_para: &Paragraph,
        paragraphs: &[Paragraph],
        available: f64,
        ep_idx: usize,
        has_visible_endnote_separator: bool,
        local_vpos_rewind: bool,
        internal_vpos_rewind: bool,
        large_separator_block: bool,
    ) -> bool {
        large_separator_block
            && !has_visible_endnote_separator
            && ep_idx > 0
            && st.col_count > 1
            && st.current_column + 1 >= st.col_count
            && st.current_height > available * 0.93
            && !st.current_items.is_empty()
            && !local_vpos_rewind
            && !internal_vpos_rewind
            && fmt.line_heights.len() <= 2
            && para_has_visible_text_or_equation(en_para)
            && !para_has_treat_as_char_picture_or_shape(en_para)
            && !para_has_non_tac_picture_or_shape(en_para)
            && {
                let mut recent_pi: Vec<usize> = Vec::new();
                for pi in st
                    .current_items
                    .iter()
                    .rev()
                    .filter_map(page_item_para_index)
                {
                    if recent_pi.last().copied() == Some(pi) {
                        continue;
                    }
                    recent_pi.push(pi);
                    if recent_pi.len() >= 2 {
                        break;
                    }
                }
                match (recent_pi.first(), recent_pi.get(1)) {
                    (Some(last_pi), Some(prev_pi)) => {
                        let last_is_text_tail =
                            paragraph_by_global_index(paragraphs, &st.endnote_paragraphs, *last_pi)
                                .is_some_and(|prev_para| {
                                    para_has_visible_text_or_equation(prev_para)
                                        && !para_has_treat_as_char_picture_or_shape(prev_para)
                                        && !para_has_non_tac_picture_or_shape(prev_para)
                                });
                        let previous_is_tac_picture =
                            paragraph_by_global_index(paragraphs, &st.endnote_paragraphs, *prev_pi)
                                .is_some_and(para_is_treat_as_char_picture_only);
                        last_is_text_tail && previous_is_tac_picture
                    }
                    _ => false,
                }
            }
    }

    /// [Task #2079] P6 판정: 큰 below 마진 마지막 단의 제목 orphan.
    #[allow(clippy::too_many_arguments)]
    pub(in crate::renderer::typeset) fn judge_default_large_below_last_column_title_orphan(
        &self,
        fmt: &FormattedParagraph,
        st: &TypesetState,
        en_ctrl: &crate::model::footnote::Endnote,
        en_ref: &EndnoteRef,
        endnote_shape: Option<&FootnoteShape>,
        available: f64,
        ep_idx: usize,
        next_endnote_first_line_advance: Option<f64>,
        next_endnote_head_pair_advance: Option<f64>,
        default_between_notes_gap: bool,
        compact_endnote_separator_profile: bool,
        has_visible_endnote_separator: bool,
        endnote_has_visible_payload: bool,
    ) -> bool {
        compact_endnote_separator_profile
            && default_between_notes_gap
            && has_visible_endnote_separator
            && endnote_shape
                .map(|shape| {
                    endnote_separator_below_margin(shape) as i32
                        > ENDNOTE_BETWEEN_NOTES_BASE_FLOW_HU
                })
                .unwrap_or(false)
            && ep_idx == 0
            && en_ref.number > 0
            && st.current_column + 1 >= st.col_count
            && fmt.line_heights.len() == 1
            && st.current_height > available * 0.95
            && st.current_height + fmt.line_advance(0)
                <= available + ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX
            && en_ctrl.paragraphs.get(1).is_some_and(|next_para| {
                !para_has_visible_text(next_para) && para_has_visible_text_or_equation(next_para)
            })
            && (next_endnote_first_line_advance
                .map(|next_h| {
                    st.current_height + fmt.line_advance(0) + next_h
                        > available + ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX + 2.0
                })
                .unwrap_or(false)
                || next_endnote_head_pair_advance
                    .map(|next_h| {
                        st.current_height + fmt.line_advance(0) + next_h
                            > available + ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX + 2.0
                    })
                    .unwrap_or(false))
            && endnote_has_visible_payload
    }

    /// [Task #2079] P6 판정: compact below 마진 마지막 단의 제목 orphan.
    #[allow(clippy::too_many_arguments)]
    pub(in crate::renderer::typeset) fn judge_default_compact_below_last_column_title_orphan(
        &self,
        fmt: &FormattedParagraph,
        st: &TypesetState,
        en_ctrl: &crate::model::footnote::Endnote,
        en_ref: &EndnoteRef,
        endnote_shape: Option<&FootnoteShape>,
        available: f64,
        ep_idx: usize,
        next_endnote_first_line_advance: Option<f64>,
        next_endnote_head_pair_advance: Option<f64>,
        default_between_notes_gap: bool,
        compact_endnote_separator_profile: bool,
        has_visible_endnote_separator: bool,
        endnote_has_visible_payload: bool,
    ) -> bool {
        compact_endnote_separator_profile
            && default_between_notes_gap
            && has_visible_endnote_separator
            && endnote_shape
                .map(|shape| {
                    endnote_separator_below_margin(shape) as i32
                        <= ENDNOTE_BETWEEN_NOTES_BASE_FLOW_HU
                })
                .unwrap_or(false)
            && ep_idx == 0
            && en_ref.number > 0
            && st.current_column + 1 >= st.col_count
            && fmt.line_heights.len() == 1
            && st.current_height > available * 0.95
            && st.current_height + fmt.line_advance(0)
                <= available + ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX
            && ((en_ctrl.paragraphs.get(1).is_some_and(|next_para| {
                !para_has_visible_text(next_para) && para_has_visible_text_or_equation(next_para)
            }) && next_endnote_head_pair_advance
                .map(|next_h| {
                    st.current_height + fmt.line_advance(0) + next_h
                        > available + ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX + 2.0
                })
                .unwrap_or(false))
                || (st.profile.hwpx_stored_layout()
                    && en_ctrl
                        .paragraphs
                        .first()
                        .is_some_and(para_is_short_auto_endnote_marker)
                    && en_ctrl
                        .paragraphs
                        .get(1)
                        .is_some_and(para_has_visible_text_and_treat_as_char_equation)
                    && next_endnote_first_line_advance
                        .map(|next_h| {
                            st.current_height + fmt.line_advance(0) + next_h > available + 2.0
                        })
                        .unwrap_or(false)))
            && endnote_has_visible_payload
    }

    /// [Task #2079] P6 판정: 기본 프로파일 문항 제목 tail 이 줄높이 기준으로 들어가는지.
    #[allow(clippy::too_many_arguments)]
    pub(in crate::renderer::typeset) fn judge_default_question_title_tail_fits_by_line_height(
        &self,
        fmt: &FormattedParagraph,
        en_ctrl: &crate::model::footnote::Endnote,
        endnote_shape: Option<&FootnoteShape>,
        styles: &ResolvedStyleSet,
        available: f64,
        en_col_w: f64,
        h4f: f64,
        ep_idx: usize,
        st: &TypesetState,
        default_between_notes_gap: bool,
        compact_endnote_separator_profile: bool,
        has_visible_endnote_separator: bool,
        endnote_has_visible_payload: bool,
        zero_endnote_spacing_profile: bool,
    ) -> bool {
        compact_endnote_separator_profile
            && default_between_notes_gap
            && endnote_shape
                .map(endnote_has_compact_separator_below)
                .unwrap_or(false)
            && !zero_endnote_spacing_profile
            && ep_idx == 0
            && st.current_column + 1 < st.col_count
            && fmt.line_heights.len() == 1
            && st.current_height > available * 0.92
            && st.current_height + h4f <= available + ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX
            && en_ctrl
                .paragraphs
                .get(1)
                .map(|next_para| {
                    let next_comp =
                        crate::renderer::composer::compose_paragraph_in_context(next_para, styles);
                    let next_fmt = self.format_endnote_paragraph(
                        next_para,
                        Some(&next_comp),
                        &styles,
                        Some(en_col_w),
                    );
                    let next_h = next_fmt.height_for_fit;
                    let title_body_limit =
                        if has_visible_endnote_separator && st.current_height > available * 0.95 {
                            available + 2.0
                        } else {
                            available + ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX + 2.0
                        };
                    st.current_height + fmt.line_advance(0) + next_h <= title_body_limit
                })
                .unwrap_or(true)
            && endnote_has_visible_payload
    }

    /// [Task #2079] P6 판정: 늦은 compact 텍스트 tail 의 overflow 위험.
    #[allow(clippy::too_many_arguments)]
    pub(in crate::renderer::typeset) fn judge_late_compact_text_tail_overflow_risk(
        &self,
        fmt: &FormattedParagraph,
        st: &TypesetState,
        available: f64,
        en_fit: f64,
        total_advance_fit: f64,
        ep_idx: usize,
        has_treat_as_char_picture_shape: bool,
        default_nonzero_between_note_tail_candidate: bool,
        default_between_notes_gap: bool,
        compact_endnote_separator_profile: bool,
        has_visible_endnote_separator: bool,
        local_vpos_rewind: bool,
        internal_vpos_rewind: bool,
        endnote_has_visible_payload: bool,
        large_separator_block: bool,
        compact_between_notes_gap: bool,
    ) -> bool {
        compact_endnote_separator_profile
            && compact_between_notes_gap
            && !local_vpos_rewind
            && !internal_vpos_rewind
            && !has_treat_as_char_picture_shape
            && (fmt.line_heights.len() <= 2
                || (default_between_notes_gap
                    && ep_idx > 0
                    && en_fit > 60.0
                    && fmt.line_heights.len() <= 3))
            && endnote_has_visible_payload
            && (((large_separator_block
                || !default_between_notes_gap
                || (default_between_notes_gap
                    && default_nonzero_between_note_tail_candidate
                    && ep_idx > 0
                    && st.current_height > available * 0.90)
                || (default_between_notes_gap && ep_idx > 0 && en_fit > 24.0))
                && st.current_column + 1 < st.col_count
                && st.current_height > available * 0.96
                && st.current_height + total_advance_fit > available - 20.0)
                || (!default_between_notes_gap
                    && has_visible_endnote_separator
                    && st.current_column + 1 >= st.col_count
                    && ep_idx > 0
                    && st.current_height > available * 0.92
                    && st.current_height + total_advance_fit > available - 40.0))
    }

    /// [Task #2079] P6 판정: 큰 미주 사이 tail 이 저장 vpos render 기준 frame 을 넘는지.
    #[allow(clippy::too_many_arguments)]
    pub(in crate::renderer::typeset) fn judge_large_between_tail_render_overflows(
        &self,
        st: &TypesetState,
        paragraphs: &[Paragraph],
        fmt: &FormattedParagraph,
        en_para: &Paragraph,
        this_first_offset: Option<i32>,
        split_endnote_to_fit: Option<usize>,
        available: f64,
        ep_idx: usize,
        default_between_notes_gap: bool,
        compact_endnote_separator_profile: bool,
        has_visible_endnote_separator: bool,
        local_vpos_rewind: bool,
        internal_vpos_rewind: bool,
        large_separator_block: bool,
        visible_compact_sequential_tail_fits_current_column: bool,
    ) -> bool {
        if !default_between_notes_gap
            && compact_endnote_separator_profile
            && (has_visible_endnote_separator || !large_separator_block)
            && ep_idx > 0
            && st.col_count > 1
            && st.current_column + 1 < st.col_count
            && st.current_height > available * 0.85
            && !st.current_items.is_empty()
            && !local_vpos_rewind
            && !internal_vpos_rewind
            && split_endnote_to_fit.is_none()
            && !visible_compact_sequential_tail_fits_current_column
            && para_has_visible_text(en_para)
        {
            let prev_equation_only_tail = st
                .current_items
                .iter()
                .rev()
                .filter_map(page_item_para_index)
                .find_map(|pi| paragraph_by_global_index(paragraphs, &st.endnote_paragraphs, pi))
                .map(|prev_para| {
                    !para_has_visible_text(prev_para)
                        && prev_para.controls.iter().any(
                            |ctrl| matches!(ctrl, Control::Equation(eq) if eq.common.treat_as_char),
                        )
                })
                .unwrap_or(false);
            st.current_items
                .iter()
                .filter_map(page_item_para_index)
                .find_map(|pi| {
                    paragraph_by_global_index(paragraphs, &st.endnote_paragraphs, pi)
                        .and_then(|p| p.line_segs.first())
                        .map(|s| s.vertical_pos)
                })
                .and_then(|base_vpos| {
                    this_first_offset.map(|first_vpos| {
                        let predicted_y = hwpunit_to_px((first_vpos - base_vpos).max(0), self.dpi)
                            + st.current_start_height;
                        let rendered_h = fmt.line_advances_sum(0..fmt.line_heights.len());
                        // TAC 그림/수식으로 lazy base가 깊게 보정된 단에서는
                        // 저장 vpos 직접 예측이 실제 렌더 y보다 낮게 나올 수 있다.
                        // 직전 수식-only 문단 뒤의 한 줄짜리 풀이 tail은 남은
                        // 공간이 50px 이하이면 한컴처럼 다음 단에서 이어간다.
                        let near_bottom_tail = prev_equation_only_tail
                            && fmt.line_heights.len() == 1
                            && para_has_visible_text(en_para)
                            && !para_is_treat_as_char_picture_only(en_para)
                            && !para_has_treat_as_char_picture_or_shape(en_para)
                            && st.current_height > available * 0.90
                            && st.current_height + rendered_h
                                > available + ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX;
                        predicted_y + rendered_h
                            > available + ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX + 1.0
                            || near_bottom_tail
                    })
                })
                .unwrap_or(false)
        } else {
            false
        }
    }

    /// [Task #2079] P6 판정: 큰 미주 사이 마지막 단에서 문항 제목 tail 이 들어가는지.
    #[allow(clippy::too_many_arguments)]
    pub(in crate::renderer::typeset) fn judge_large_between_last_column_question_title_tail_fits(
        &self,
        st: &TypesetState,
        fmt: &FormattedParagraph,
        en_ctrl: &crate::model::footnote::Endnote,
        en_ref: &EndnoteRef,
        styles: &ResolvedStyleSet,
        available: f64,
        en_col_w: f64,
        ep_idx: usize,
        emitted_endnote_count: usize,
        new_endnote_between_notes_px: Option<f64>,
        large_between_question_title_head_inside_frame: bool,
        endnote_has_visible_payload: bool,
        default_between_notes_gap: bool,
        compact_endnote_separator_profile: bool,
        has_visible_endnote_separator: bool,
        visible_large_between_notes_gap: bool,
        compact_between_notes_gap: bool,
    ) -> bool {
        !default_between_notes_gap
            && compact_endnote_separator_profile
            && has_visible_endnote_separator
            && visible_large_between_notes_gap
            && !compact_between_notes_gap
            && ep_idx == 0
            && emitted_endnote_count > 0
            && en_ref.number > 0
            && st.current_column + 1 >= st.col_count
            && fmt.line_heights.len() == 1
            && st.current_height > available * 0.90
            && st.current_height < available
            && large_between_question_title_head_inside_frame
            && st.current_height + fmt.line_advance(0)
                <= available + ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX + 2.0
            && new_endnote_between_notes_px
                .map(|gap| {
                    // 마지막 단에서 새 미주 제목만 남길 때도
                    // `미주 사이`는 제목 앞에 소비된다. gap 없이
                    // 제목 한 줄만 fit으로 보면 20mm 문서에서 다음
                    // 쪽으로 가야 할 제목이 현재 쪽 하단에 고아로 남는다.
                    st.current_height + gap + fmt.line_advance(0)
                        <= available + ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX + 2.0
                })
                .unwrap_or(true)
            && (compact_between_notes_gap
                || new_endnote_between_notes_px
                    .map(|gap| {
                        let head_group_h: f64 = en_ctrl
                            .paragraphs
                            .iter()
                            .take(3)
                            .map(|head_para| {
                                let head_comp =
                                    crate::renderer::composer::compose_paragraph_in_context(
                                        head_para, styles,
                                    );
                                self.format_endnote_paragraph(
                                    head_para,
                                    Some(&head_comp),
                                    &styles,
                                    Some(en_col_w),
                                )
                                .total_height
                            })
                            .sum();
                        // 제목과 첫 풀이 일부만 단 하단에 고아로 남기지
                        // 않도록, 20mm급 large gap에서는 제목+본문 head
                        // group이 함께 들어갈 때만 마지막 단 tail을 허용한다.
                        st.current_height + gap + head_group_h
                            <= available + ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX + 2.0
                    })
                    .unwrap_or(true))
            && endnote_has_visible_payload
    }

    /// [Task #2079] P6 판정: 0/0/0 프로파일 문항 intro tail 이 rewind 직전 현재 단에 들어가는지.
    #[allow(clippy::too_many_arguments)]
    pub(in crate::renderer::typeset) fn judge_zero_question_intro_tail_before_rewind_fits(
        &self,
        st: &TypesetState,
        paragraphs: &[Paragraph],
        fmt: &FormattedParagraph,
        en_para: &Paragraph,
        en_ctrl: &crate::model::footnote::Endnote,
        en_ref: &EndnoteRef,
        composed: &ComposedParagraph,
        styles: &ResolvedStyleSet,
        available: f64,
        en_col_w: f64,
        ep_idx: usize,
        later_endnote_vpos_rewinds_after_current: bool,
        zero_endnote_spacing_profile: bool,
        compact_endnote_separator_profile: bool,
        has_visible_endnote_separator: bool,
        local_vpos_rewind: bool,
        internal_vpos_rewind: bool,
    ) -> bool {
        compact_endnote_separator_profile
            && zero_endnote_spacing_profile
            && has_visible_endnote_separator
            && (st.current_column + 1 < st.col_count || ep_idx == 1)
            && matches!(ep_idx, 1 | 2)
            && en_ref.number > 0
            && fmt.line_heights.len() == 1
            && !local_vpos_rewind
            && !internal_vpos_rewind
            && !para_is_treat_as_char_picture_only(en_para)
            && para_has_visible_text_or_equation(en_para)
            && en_ctrl
                .paragraphs
                .first()
                .is_some_and(|title_para| title_para.line_segs.len() == 1)
            && later_endnote_vpos_rewinds_after_current
            && st.current_height > available * 0.95
            && st.current_height + fmt.line_advance(0)
                <= available + ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX + 28.0
            && if st.current_column + 1 >= st.col_count {
                ep_idx == 1
                    && fmt.line_advance(0) <= 24.0
                    && line_has_visible_text_or_tac_equation(en_para, &composed, 0)
            } else if ep_idx == 1 {
                en_ctrl.paragraphs.get(ep_idx + 1).is_some_and(|next_para| {
                    let next_comp =
                        crate::renderer::composer::compose_paragraph_in_context(next_para, styles);
                    let next_fmt = self.format_endnote_paragraph(
                        next_para,
                        Some(&next_comp),
                        &styles,
                        Some(en_col_w),
                    );
                    next_fmt.line_heights.len() == 1
                        && next_fmt.line_advance(0) <= 24.0
                        && line_has_visible_text_or_tac_equation(next_para, &next_comp, 0)
                })
            } else {
                fmt.line_advance(0) <= 24.0
                    && line_has_visible_text_or_tac_equation(en_para, &composed, 0)
            }
    }

    /// [Task #2079] P6 판정: 보이는 구분자 프로파일에서 저장 vpos head 그룹이 frame 밖인지.
    #[allow(clippy::too_many_arguments)]
    pub(in crate::renderer::typeset) fn judge_visible_separator_vpos_head_group_outside(
        &self,
        st: &TypesetState,
        paragraphs: &[Paragraph],
        en_ctrl: &crate::model::footnote::Endnote,
        this_first_offset: Option<i32>,
        endnote_start: i32,
        available: f64,
        ep_idx: usize,
        emitted_endnote_count: usize,
        default_between_notes_gap: bool,
        compact_endnote_separator_profile: bool,
        has_visible_endnote_separator: bool,
        compact_between_notes_gap: bool,
    ) -> bool {
        compact_endnote_separator_profile
            && compact_between_notes_gap
            && default_between_notes_gap
            && has_visible_endnote_separator
            && ep_idx == 0
            && emitted_endnote_count > 0
            && st.current_column + 1 < st.col_count
            && st.current_height > available * 0.75
            && !st.current_items.is_empty()
            && st
                .current_items
                .iter()
                .filter_map(page_item_para_index)
                .find_map(|pi| {
                    paragraph_by_global_index(paragraphs, &st.endnote_paragraphs, pi)
                        .and_then(|p| p.line_segs.first())
                        .map(|s| s.vertical_pos)
                })
                .and_then(|base_vpos| {
                    let first_vpos = this_first_offset?;
                    let first_para_vpos = en_ctrl.paragraphs.first()?.line_segs.first()?;
                    let group_bottom = en_ctrl
                        .paragraphs
                        .iter()
                        .take(3)
                        .flat_map(|p| p.line_segs.iter())
                        .map(|s| {
                            s.vertical_pos
                                .saturating_add(s.line_height)
                                .saturating_add(s.line_spacing)
                                + endnote_start
                        })
                        .max()?;
                    let group_first = first_para_vpos.vertical_pos + endnote_start;
                    let group_h = hwpunit_to_px((group_bottom - group_first).max(0), self.dpi);
                    let predicted_y = hwpunit_to_px((first_vpos - base_vpos).max(0), self.dpi);
                    Some(
                        predicted_y > available * 0.85
                            && predicted_y + group_h
                                > available - ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX,
                    )
                })
                .unwrap_or(false)
    }

    /// [Task #2079] P6 판정: 기본 간격 + 큰 below 마진 프로파일에서 head 그룹이 frame 밖인지.
    #[allow(clippy::too_many_arguments)]
    pub(in crate::renderer::typeset) fn judge_default_between_large_below_head_group_outside(
        &self,
        st: &TypesetState,
        en_ctrl: &crate::model::footnote::Endnote,
        endnote_shape: Option<&FootnoteShape>,
        styles: &ResolvedStyleSet,
        available: f64,
        en_col_w: f64,
        ep_idx: usize,
        emitted_endnote_count: usize,
        default_between_notes_gap: bool,
        compact_endnote_separator_profile: bool,
        has_visible_endnote_separator: bool,
    ) -> bool {
        compact_endnote_separator_profile
            && default_between_notes_gap
            && has_visible_endnote_separator
            && ep_idx == 0
            && emitted_endnote_count > 0
            && st.current_column + 1 < st.col_count
            && st.current_height > available * 0.90
            && endnote_shape
                .map(|shape| {
                    endnote_separator_below_margin(shape) as i32
                        > ENDNOTE_BETWEEN_NOTES_BASE_FLOW_HU
                })
                .unwrap_or(false)
            && en_ctrl
                .paragraphs
                .first()
                .is_some_and(|title_para| title_para.line_segs.len() == 1)
            && en_ctrl.paragraphs.get(1).is_some_and(para_has_visible_text)
            && en_ctrl.paragraphs.get(2).is_some_and(|tail_para| {
                !para_has_visible_text(tail_para) && para_has_visible_text_or_equation(tail_para)
            })
            && {
                let head_group_h: f64 = en_ctrl
                    .paragraphs
                    .iter()
                    .take(3)
                    .map(|head_para| {
                        let head_comp = crate::renderer::composer::compose_paragraph_in_context(
                            head_para, styles,
                        );
                        self.format_endnote_paragraph(
                            head_para,
                            Some(&head_comp),
                            &styles,
                            Some(en_col_w),
                        )
                        .total_height
                    })
                    .sum();
                st.current_height + head_group_h
                    > available - ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX
            }
    }

    /// [Task #2079] P6 판정: 큰 미주 사이 마지막 단에서 저장 vpos head 그룹이 frame 밖인지.
    #[allow(clippy::too_many_arguments)]
    pub(in crate::renderer::typeset) fn judge_large_between_last_column_vpos_head_group_outside(
        &self,
        st: &TypesetState,
        paragraphs: &[Paragraph],
        en_ctrl: &crate::model::footnote::Endnote,
        large_between_question_title_render_y: Option<f64>,
        endnote_start: i32,
        available: f64,
        ep_idx: usize,
        emitted_endnote_count: usize,
        default_between_notes_gap: bool,
        compact_endnote_separator_profile: bool,
        has_visible_endnote_separator: bool,
        visible_large_between_notes_gap: bool,
        compact_between_notes_gap: bool,
    ) -> bool {
        !default_between_notes_gap
            && compact_endnote_separator_profile
            && has_visible_endnote_separator
            && visible_large_between_notes_gap
            && !compact_between_notes_gap
            && ep_idx == 0
            && emitted_endnote_count > 0
            && st.current_column + 1 >= st.col_count
            && st.current_height > available * 0.75
            && st.current_height < available * 0.85
            && !st.current_items.is_empty()
            && large_between_question_title_render_y
                .map(|predicted_y| {
                    let group_first = en_ctrl
                        .paragraphs
                        .first()
                        .and_then(|p| p.line_segs.first())
                        .map(|seg| seg.vertical_pos + endnote_start);
                    let group_bottom = en_ctrl
                        .paragraphs
                        .iter()
                        .take(3)
                        .flat_map(|p| p.line_segs.iter())
                        .map(|seg| {
                            seg.vertical_pos
                                .saturating_add(seg.line_height)
                                .saturating_add(seg.line_spacing)
                                + endnote_start
                        })
                        .max();
                    group_first
                        .zip(group_bottom)
                        .map(|(first, bottom)| {
                            let group_h = hwpunit_to_px((bottom - first).max(0), self.dpi);
                            predicted_y > available * 0.85
                                && predicted_y + group_h
                                    > available - ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX
                        })
                        .unwrap_or(false)
                })
                .unwrap_or(false)
    }

    /// [Task #2079] P6 판정: split head 가 저장 vpos 기준 render 로 frame 을 넘는지 (넘으면 split 취소).
    #[allow(clippy::too_many_arguments)]
    pub(in crate::renderer::typeset) fn judge_large_between_split_head_render_overflows(
        &self,
        st: &TypesetState,
        fmt: &FormattedParagraph,
        en_para: &Paragraph,
        paragraphs: &[Paragraph],
        styles: &ResolvedStyleSet,
        split_endnote_to_fit: Option<usize>,
        available: f64,
        en_col_w: f64,
        en_para_idx: usize,
        ep_idx: usize,
        default_between_notes_gap: bool,
        compact_endnote_separator_profile: bool,
        has_visible_endnote_separator: bool,
        local_vpos_rewind: bool,
        internal_vpos_rewind: bool,
        endnote_has_visible_payload: bool,
    ) -> bool {
        if !default_between_notes_gap
            && compact_endnote_separator_profile
            && has_visible_endnote_separator
            && ep_idx > 0
            && st.col_count > 1
            && st.current_height > available * 0.90
            && !st.current_items.is_empty()
            && !local_vpos_rewind
            && !internal_vpos_rewind
            && fmt.line_heights.len() > 1
            && para_has_visible_text_or_equation(en_para)
            && endnote_has_visible_payload
        {
            split_endnote_to_fit
                .and_then(|split_line| {
                    let predicted_y = self.predict_endnote_render_y(
                        st,
                        paragraphs,
                        styles,
                        available,
                        en_col_w,
                        en_para_idx,
                    )?;
                    let split_head_h = fmt.line_advances_sum(0..split_line);
                    Some(
                        predicted_y + split_head_h
                            > available + ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX + 1.0,
                    )
                })
                .unwrap_or(false)
        } else {
            false
        }
    }

    /// [Task #2079] P6 판정: 큰 미주 사이 마지막 단의 visual tail split 줄 수 (render vpos 시뮬 기반).
    #[allow(clippy::too_many_arguments)]
    pub(in crate::renderer::typeset) fn judge_large_between_last_column_visual_split(
        &self,
        st: &TypesetState,
        fmt: &FormattedParagraph,
        en_para: &Paragraph,
        paragraphs: &[Paragraph],
        styles: &ResolvedStyleSet,
        available: f64,
        en_col_w: f64,
        en_fit: f64,
        en_para_idx: usize,
        ep_idx: usize,
        default_between_notes_gap: bool,
        compact_endnote_separator_profile: bool,
        has_visible_endnote_separator: bool,
        visible_large_between_notes_gap: bool,
        compact_between_notes_gap: bool,
        zero_endnote_spacing_profile: bool,
        local_vpos_rewind: bool,
        internal_vpos_rewind: bool,
    ) -> Option<usize> {
        if !default_between_notes_gap
            && compact_endnote_separator_profile
            && has_visible_endnote_separator
            && visible_large_between_notes_gap
            && !compact_between_notes_gap
            && !zero_endnote_spacing_profile
            && ep_idx > 0
            && st.col_count > 1
            && st.current_column + 1 >= st.col_count
            && (st.current_height > available * 0.85
                || st.current_height + en_fit > available - 60.0)
            && st.current_height < available
            && (st.current_height + en_fit > available || fmt.line_heights.len() >= 3)
            && !st.current_items.is_empty()
            && !local_vpos_rewind
            && !internal_vpos_rewind
            && fmt.line_heights.len() > 1
            && para_has_visible_text_or_equation(en_para)
        {
            let predicted_y = self.predict_endnote_render_y(
                st,
                paragraphs,
                styles,
                available,
                en_col_w,
                en_para_idx,
            );

            predicted_y.and_then(|y| {
                if y >= available {
                    return None;
                }
                // 첫 줄 자체가 frame 안쪽에 들어오지 못하면 visual split으로
                // 단 하단에 남기지 않는다. 큰 미주 사이 문서에서는 이 줄들을
                // 남기면 다음 쪽의 문항 시작점이 연쇄적으로 위로 당겨진다.
                if y + fmt.line_advance(0)
                    > available + ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX + 2.0
                {
                    return None;
                }
                // 큰 미주 사이 문서의 마지막 단은 렌더 vpos가 직전
                // 문단들을 위로 당긴 뒤 남는 visual tail 공간을 사용한다.
                // pagination 누적 높이만 보면 부족하지만, 한컴/PDF는 다음
                // 문단의 마지막 1줄만 이월시키는 패턴이 있어 이 경로에만
                // 단 하단 visual 한도를 넓힌다.
                let flow_overflows = st.current_height + en_fit > available;
                let visual_tail_limit = if flow_overflows {
                    available + ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX + 46.0
                } else {
                    // flow 누적 높이는 들어가지만 저장 vpos 기반 render
                    // 마지막 줄이 frame 하단에 걸리는 큰 미주 사이 tail은
                    // 한컴처럼 마지막 줄부터 다음 쪽으로 넘긴다.
                    available + 1.0
                };
                let mut consumed = 0.0;
                let mut split = 0usize;
                for line_idx in 0..fmt.line_heights.len() {
                    let next = consumed + fmt.line_advance(line_idx);
                    if y + next > visual_tail_limit {
                        break;
                    }
                    consumed = next;
                    split = line_idx + 1;
                }
                (split > 0 && split < fmt.line_heights.len()).then_some(split)
            })
        } else {
            None
        }
    }

    /// [Task #2079] P6 판정: 큰 미주 사이 마지막 단에서 제목 tail 뒤 본문이 render vpos 기준 쪽 전진하는지.
    #[allow(clippy::too_many_arguments)]
    pub(in crate::renderer::typeset) fn judge_large_between_title_tail_body_advances_page(
        &self,
        st: &TypesetState,
        fmt: &FormattedParagraph,
        en_para: &Paragraph,
        en_ctrl: &crate::model::footnote::Endnote,
        en_ref: &EndnoteRef,
        paragraphs: &[Paragraph],
        styles: &ResolvedStyleSet,
        available: f64,
        en_col_w: f64,
        en_para_idx: usize,
        ep_idx: usize,
        default_between_notes_gap: bool,
        compact_endnote_separator_profile: bool,
        has_visible_endnote_separator: bool,
        local_vpos_rewind: bool,
        internal_vpos_rewind: bool,
        endnote_has_visible_payload: bool,
    ) -> bool {
        if !default_between_notes_gap
            && compact_endnote_separator_profile
            && has_visible_endnote_separator
            && ep_idx == 1
            && en_ref.number > 0
            && st.current_column + 1 >= st.col_count
            && st.current_height > available * 0.90
            && !st.current_items.is_empty()
            && !local_vpos_rewind
            && !internal_vpos_rewind
            && en_ctrl
                .paragraphs
                .first()
                .is_some_and(|title_para| title_para.line_segs.len() == 1)
            && fmt.line_heights.len() <= 2
            && para_has_visible_text_or_equation(en_para)
            && endnote_has_visible_payload
        {
            let predicted_y = self.predict_endnote_render_y(
                st,
                paragraphs,
                styles,
                available,
                en_col_w,
                en_para_idx,
            );

            predicted_y
                .map(|y| {
                    // 큰 미주 사이 문서의 마지막 단에서는 새 문항 제목
                    // 한 줄만 frame 안쪽 tail로 남기고, 첫 풀이 수식/본문이
                    // render vpos 기준으로 frame을 넘으면 다음 쪽에서 시작한다.
                    y + fmt.line_advance(0)
                        > available + ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX + 1.0
                })
                .unwrap_or(false)
        } else {
            false
        }
    }
}

//! 미주 내부 문단 흐름 조정. 준비·조회·상태 반영의 기존 순서를 보존한다.

use crate::renderer::typeset::notes::endnotes::profile::{
    endnote_between_notes_margin, endnote_has_absorbed_between_notes_gap,
    endnote_separator_below_margin, ENDNOTE_BETWEEN_NOTES_BASE_FLOW_HU,
};
use crate::renderer::typeset::{
    endnote_last_column_tail_overflows_frame, hwpunit_to_px, is_treat_as_char_equation_control,
    line_has_tac_equation_control, line_has_visible_text, line_is_equation_tac_text_run_only,
    page_item_para_index, para_has_non_tac_picture_or_shape,
    para_has_treat_as_char_picture_or_shape, para_has_visible_text,
    para_has_visible_text_or_equation, para_is_treat_as_char_picture_only,
    paragraph_by_global_index, ComposedParagraph, EndnoteRef, FootnoteShape, FormattedParagraph,
    MeasuredTable, Paragraph, ResolvedStyleSet, TypesetEngine, TypesetState,
    ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX,
};

/// 호출 시점의 관측값. 페이지 상태를 변경할 수 없는 Query 입력이다.
pub(super) struct TailFitInput<'a> {
    pub(super) default_question_group_title_tail: bool,
    pub(super) st: &'a TypesetState,
    pub(super) paragraphs: &'a [Paragraph],
    pub(super) styles: &'a ResolvedStyleSet,
    pub(super) measured_tables: &'a [MeasuredTable],
    pub(super) endnote_shape: Option<&'a FootnoteShape>,
    pub(super) endnote_refs: &'a [EndnoteRef],
    pub(super) en_ref_idx: usize,
    pub(super) en_ref: &'a EndnoteRef,
    pub(super) en_ctrl: &'a crate::model::footnote::Endnote,
    pub(super) ep_idx: usize,
    pub(super) en_para: &'a Paragraph,
    pub(super) fmt: &'a FormattedParagraph,
    pub(super) composed: &'a ComposedParagraph,
    pub(super) emitted_endnote_count: usize,
    pub(super) en_para_idx: usize,
    pub(super) internal_rewind_split: Option<usize>,
    pub(super) new_endnote_between_notes_px: Option<f64>,
    pub(super) non_tac_object_height: Option<f64>,
    pub(super) col_count: u16,
    pub(super) split_endnote_to_fit: Option<usize>,
    pub(super) large_between_last_column_visual_split: Option<usize>,
    pub(super) large_between_last_column_flow_tail_split: Option<usize>,
    pub(super) available: f64,
    pub(super) en_col_w: f64,
    pub(super) dpi: f64,
    pub(super) h4f: f64,
    pub(super) line_advances_sum: f64,
    pub(super) total_advance_fit: f64,
    pub(super) en_fit: f64,
    pub(super) endnote_boundary_gap_extra_px: f64,
    pub(super) endnote_has_vpos_rewind: bool,
    pub(super) default_nonzero_between_note_tail_candidate: bool,
    pub(super) compact_endnote_separator_profile: bool,
    pub(super) prev_rendered_endnote_is_title: bool,
    pub(super) local_vpos_rewind: bool,
    pub(super) has_visible_endnote_separator: bool,
    pub(super) internal_vpos_rewind: bool,
    pub(super) large_separator_block: bool,
    pub(super) zero_between_large_separator_margin: bool,
    pub(super) both_large_separator_default_between: bool,
    pub(super) endnote_has_text_or_equation: bool,
    pub(super) endnote_has_visible_payload: bool,
    pub(super) default_between_notes_gap: bool,
    pub(super) zero_endnote_spacing_profile: bool,
    pub(super) compact_between_notes_gap: bool,
    pub(super) visible_large_between_notes_gap: bool,
    pub(super) visible_zero_between_large_separator_gap: bool,
    pub(super) visible_large_between_zero_above_compact_below: bool,
    pub(super) allow_default_late_question_tail: bool,
    pub(super) has_treat_as_char_picture_shape: bool,
    pub(super) tac_picture_tail_height: Option<f64>,
    pub(super) no_separator_visible_multiline_tail_fits_with_bleed: bool,
    pub(super) next_endnote_title_fit_height: Option<f64>,
    pub(super) no_separator_saved_vpos_tail_outside: bool,
    pub(super) visible_separator_saved_vpos_tail_outside: bool,
    pub(super) compact_endnote_own_vpos_span_fits_for_flow: bool,
    pub(super) large_between_split_head_render_overflows: bool,
    pub(super) internal_rewind_head_allows_current_column: bool,
    pub(super) internal_rewind_head_overflows_current_column: bool,
    pub(super) internal_reset_split_head_render_overflows: bool,
    pub(super) internal_rewind_full_advance_needed: bool,
    pub(super) no_separator_tail_after_picture_starts_next_page: bool,
    pub(super) later_endnote_vpos_rewinds_after_current: bool,
    pub(super) large_between_equation_tail_starts_next_column: bool,
    pub(super) large_between_question_title_render_y: Option<f64>,
    pub(super) large_between_question_title_head_inside_frame: bool,
    pub(super) large_between_question_title_head_fits_flow: bool,
    pub(super) large_between_question_lead_group_render_outside: bool,
    pub(super) zero_between_large_separator_last_column_title_orphan: bool,
}

/// 기존 순서로 계산한 후보. 적용은 문단 조정자가 담당한다.
pub(super) struct TailFitResult {
    pub(super) allow_large_between_question_title_tail: bool,
    pub(super) allow_default_column_bottom_question_title_tail: bool,
    pub(super) allow_default_first_column_large_below_title_tail: bool,
    pub(super) new_endnote_advance_threshold: f64,
    pub(super) allow_compact_question_title_tail: bool,
    pub(super) allow_large_separator_first_column_tail: bool,
    pub(super) large_between_last_column_question_title_tail_fits: bool,
    pub(super) large_between_last_column_render_title_tail_fits: bool,
    pub(super) large_between_last_column_rewind_title_tail_fits: bool,
    pub(super) default_question_title_tail_fits_by_line_height: bool,
    pub(super) zero_question_title_tail_fits_by_line_height: bool,
    pub(super) large_between_zero_above_whole_note_small_bleed_fits: bool,
    pub(super) advance_for_fit: bool,
    pub(super) pre_emit_tail_before_non_tac_object_advance: bool,
}

impl TypesetEngine {
    pub(super) fn query_tail_fit(&self, input: TailFitInput<'_>) -> TailFitResult {
        let TailFitInput {
            default_question_group_title_tail,
            st,
            paragraphs,
            styles,
            measured_tables,
            endnote_shape,
            endnote_refs,
            en_ref_idx,
            en_ref,
            en_ctrl,
            ep_idx,
            en_para,
            fmt,
            composed,
            emitted_endnote_count,
            en_para_idx,
            internal_rewind_split,
            new_endnote_between_notes_px,
            non_tac_object_height,
            col_count,
            split_endnote_to_fit,
            large_between_last_column_visual_split,
            large_between_last_column_flow_tail_split,
            available,
            en_col_w,
            dpi,
            h4f,
            line_advances_sum,
            total_advance_fit,
            en_fit,
            endnote_boundary_gap_extra_px,
            endnote_has_vpos_rewind,
            default_nonzero_between_note_tail_candidate,
            compact_endnote_separator_profile,
            prev_rendered_endnote_is_title,
            local_vpos_rewind,
            has_visible_endnote_separator,
            internal_vpos_rewind,
            large_separator_block,
            zero_between_large_separator_margin,
            both_large_separator_default_between,
            endnote_has_text_or_equation,
            endnote_has_visible_payload,
            default_between_notes_gap,
            zero_endnote_spacing_profile,
            compact_between_notes_gap,
            visible_large_between_notes_gap,
            visible_zero_between_large_separator_gap,
            visible_large_between_zero_above_compact_below,
            allow_default_late_question_tail,
            has_treat_as_char_picture_shape,
            tac_picture_tail_height,
            no_separator_visible_multiline_tail_fits_with_bleed,
            next_endnote_title_fit_height,
            no_separator_saved_vpos_tail_outside,
            visible_separator_saved_vpos_tail_outside,
            compact_endnote_own_vpos_span_fits_for_flow,
            large_between_split_head_render_overflows,
            internal_rewind_head_allows_current_column,
            internal_rewind_head_overflows_current_column,
            internal_reset_split_head_render_overflows,
            internal_rewind_full_advance_needed,
            no_separator_tail_after_picture_starts_next_page,
            later_endnote_vpos_rewinds_after_current,
            large_between_equation_tail_starts_next_column,
            large_between_question_title_render_y,
            large_between_question_title_head_inside_frame,
            large_between_question_title_head_fits_flow,
            large_between_question_lead_group_render_outside,
            zero_between_large_separator_last_column_title_orphan,
        } = input;
        let allow_large_between_question_title_tail = !default_between_notes_gap
            && ep_idx == 0
            && en_ref.number > 0
            && st.current_column + 1 < st.col_count
            && fmt.line_heights.len() == 1
            && st.current_height < available
            && (large_between_question_title_head_inside_frame
                || large_between_question_title_head_fits_flow)
            && !large_between_question_lead_group_render_outside
            && st.current_height + fmt.line_advance(0)
                <= available + ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX + 2.0;
        let allow_default_column_bottom_question_title_tail = default_between_notes_gap
            && compact_endnote_separator_profile
            && ep_idx == 0
            && en_ref.number > 0
            && fmt.line_heights.len() == 1
            && !local_vpos_rewind
            && !internal_vpos_rewind
            && !st.current_items.is_empty()
            && default_question_group_title_tail
            && st.current_height < available
            && st.current_height > available * 0.88
            && st.current_height + fmt.line_advance(0)
                <= available + ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX + 2.0
            && para_has_visible_text_or_equation(en_para);
        let allow_default_first_column_large_below_title_tail =
            allow_default_column_bottom_question_title_tail
                && endnote_has_vpos_rewind
                && st.current_column + 1 < st.col_count
                && endnote_shape
                    .map(|shape| {
                        endnote_separator_below_margin(shape) as i32
                            > ENDNOTE_BETWEEN_NOTES_BASE_FLOW_HU
                    })
                    .unwrap_or(false);
        let new_endnote_advance_threshold = if default_between_notes_gap {
            if st.current_column + 1 < st.col_count {
                0.89
            } else {
                0.95
            }
        } else if st.current_column + 1 < st.col_count {
            0.88
        } else {
            0.95
        };
        let no_separator_zero_between_notes = !has_visible_endnote_separator
            && endnote_shape
                .map(|shape| endnote_between_notes_margin(shape) == 0)
                .unwrap_or(false);
        let allow_compact_question_title_tail = self.judge_allow_compact_question_title_tail(
            new_endnote_between_notes_px,
            new_endnote_advance_threshold,
            no_separator_zero_between_notes,
            st,
            &fmt,
            available,
            compact_endnote_separator_profile,
            default_between_notes_gap,
            en_fit,
            endnote_has_visible_payload,
            ep_idx,
            has_visible_endnote_separator,
            large_separator_block,
        );
        let allow_large_separator_first_column_tail = visible_large_between_notes_gap
            && ep_idx == 0
            && st.current_column + 1 < st.col_count
            && !large_between_question_lead_group_render_outside
            && st.current_height + en_fit <= available + ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX
            && endnote_has_visible_payload;
        let large_between_last_column_question_title_tail_fits = self
            .judge_large_between_last_column_question_title_tail_fits(
                st,
                &fmt,
                en_ctrl,
                en_ref,
                styles,
                available,
                en_col_w,
                ep_idx,
                emitted_endnote_count,
                new_endnote_between_notes_px,
                large_between_question_title_head_inside_frame,
                endnote_has_visible_payload,
                default_between_notes_gap,
                compact_endnote_separator_profile,
                has_visible_endnote_separator,
                visible_large_between_notes_gap,
                compact_between_notes_gap,
            );
        let large_between_last_column_render_title_tail_fits = self
            .judge_large_between_last_column_render_title_tail_fits(
                large_between_question_title_render_y,
                large_between_question_title_head_inside_frame,
                st,
                &fmt,
                en_ref,
                available,
                compact_between_notes_gap,
                compact_endnote_separator_profile,
                default_between_notes_gap,
                emitted_endnote_count,
                endnote_has_visible_payload,
                ep_idx,
                has_visible_endnote_separator,
                visible_large_between_notes_gap,
            );
        let large_between_last_column_rewind_title_tail_fits = self
            .judge_large_between_last_column_rewind_title_tail_fits(
                large_between_question_title_head_inside_frame,
                st,
                &fmt,
                en_ref,
                available,
                compact_between_notes_gap,
                compact_endnote_separator_profile,
                default_between_notes_gap,
                emitted_endnote_count,
                en_fit,
                endnote_has_visible_payload,
                endnote_has_vpos_rewind,
                ep_idx,
                has_visible_endnote_separator,
                visible_large_between_notes_gap,
            );
        let large_between_last_column_title_body_tail_fits = !default_between_notes_gap
            && compact_endnote_separator_profile
            && has_visible_endnote_separator
            && ep_idx == 1
            && en_ref.number > 0
            && prev_rendered_endnote_is_title
            && st.current_column + 1 >= st.col_count
            && fmt.line_heights.len() > 1
            && !local_vpos_rewind
            && !internal_vpos_rewind
            && st.current_height > available * 0.90
            && st.current_height < available
            && st.current_height + fmt.line_advances_sum(0..fmt.line_heights.len())
                <= available + ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX + 2.0
            && endnote_has_visible_payload
            && !para_has_non_tac_picture_or_shape(en_para);
        let late_question_title_small_overflow = allow_default_late_question_tail
            && ep_idx == 0
            && st.current_column + 1 >= st.col_count
            && st.current_height < available
            && st.current_height + en_fit <= available + 40.0;
        let late_question_intro_tail = allow_default_late_question_tail
            && ep_idx == 1
            && st.current_column + 1 >= st.col_count
            && st.current_height < available + 40.0
            && st.current_height + en_fit <= available + 90.0;
        let late_question_continuation_tail = allow_default_late_question_tail
            && ep_idx > 1
            && st.current_column + 1 >= st.col_count
            && st.current_height < available + 40.0
            && st.current_height + en_fit <= available + 90.0
            && endnote_has_visible_payload;
        let default_question_title_tail_fits_by_line_height = self
            .judge_default_question_title_tail_fits_by_line_height(
                &fmt,
                en_ctrl,
                endnote_shape,
                styles,
                available,
                en_col_w,
                h4f,
                ep_idx,
                st,
                default_between_notes_gap,
                compact_endnote_separator_profile,
                has_visible_endnote_separator,
                endnote_has_visible_payload,
                zero_endnote_spacing_profile,
            );
        let zero_question_title_tail_fits_by_line_height = self
            .judge_zero_question_title_tail_fits_by_line_height(
                st,
                &fmt,
                available,
                compact_endnote_separator_profile,
                endnote_has_visible_payload,
                ep_idx,
                h4f,
                has_visible_endnote_separator,
                zero_endnote_spacing_profile,
            );
        let zero_question_intro_tail_before_rewind_fits = self
            .judge_zero_question_intro_tail_before_rewind_fits(
                st,
                paragraphs,
                &fmt,
                en_para,
                en_ctrl,
                en_ref,
                &composed,
                styles,
                available,
                en_col_w,
                ep_idx,
                later_endnote_vpos_rewinds_after_current,
                zero_endnote_spacing_profile,
                compact_endnote_separator_profile,
                has_visible_endnote_separator,
                local_vpos_rewind,
                internal_vpos_rewind,
            );
        let zero_between_large_separator_tail_group_fits = self
            .judge_zero_between_large_separator_tail_group_fits(
                zero_between_large_separator_margin,
                st,
                &fmt,
                en_para,
                en_ref,
                endnote_shape,
                available,
                compact_endnote_separator_profile,
                ep_idx,
                has_visible_endnote_separator,
                internal_vpos_rewind,
                large_separator_block,
                later_endnote_vpos_rewinds_after_current,
                local_vpos_rewind,
            );
        let late_compact_text_tail_overflow_risk = self
            .judge_late_compact_text_tail_overflow_risk(
                &fmt,
                st,
                available,
                en_fit,
                total_advance_fit,
                ep_idx,
                has_treat_as_char_picture_shape,
                default_nonzero_between_note_tail_candidate,
                default_between_notes_gap,
                compact_endnote_separator_profile,
                has_visible_endnote_separator,
                local_vpos_rewind,
                internal_vpos_rewind,
                endnote_has_visible_payload,
                large_separator_block,
                compact_between_notes_gap,
            )
            // [#6544] 저장 사다리가 "이 단에 들어간다"고 말하고 **자기 회계로도 들어가면**
            // 위험 휴리스틱을 적용하지 않는다.
            //
            // 이 술어는 저장 증거를 보지 않는 순수 띠다 — "단의 96% 를 넘었고 이 문단을
            // 넣으면 하단 20px 안으로 들어온다"면 넘긴다. 그런데 `advance_for_fit` 안에서
            // `compact_endnote_own_vpos_span_fits_for_flow`(= 문단의 저장 vpos 폭이 남은
            // 공간에 든다)를 **뚫는 예외**로 등재돼 있어, 파일이 같은 단에 두라고 적어 둔
            // 문단까지 넘긴다.
            //
            // 3-09월_교육_통합_2023 13쪽 왼쪽 단: 저장 사다리가 pi=657·658·659 를 Δ=1352
            // 로 연속 기록하고 되감김(=단 경계)은 pi=660 에서 낸다. pi=658 을 넘길 때
            // 누계는 974.2 로 가용 1001.6 안이고 진행량 18.0 을 더해도 992.2 라 실제로
            // 들어간다 — 위험 판정이 근거 없이 발동한 것이다.
            //
            // 예외를 통째로 빼면 #1274·#1284 sweep 핀과 off_canvas 래칫이 걸린다. 여기서는
            // **넣어도 가용 안에 남는** 경우로만 좁힌다.
            && !(compact_endnote_own_vpos_span_fits_for_flow
                && st.current_height + total_advance_fit <= available);
        let zero_tac_picture_tail_bleeds_frame = compact_endnote_separator_profile
            && zero_endnote_spacing_profile
            && has_visible_endnote_separator
            && st.current_column + 1 < st.col_count
            && st.current_height > available * 0.70
            && para_is_treat_as_char_picture_only(en_para)
            && !local_vpos_rewind
            && !internal_vpos_rewind
            && fmt.line_heights.len() == 1
            && st.current_height + total_advance_fit
                > available - ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX;
        let visible_separator_large_tac_tail_candidate = compact_endnote_separator_profile
            && !zero_endnote_spacing_profile
            && has_visible_endnote_separator
            && st.col_count > 1
            && st.current_height > available * 0.60
            && tac_picture_tail_height.is_some()
            && !local_vpos_rewind
            && !internal_vpos_rewind
            && fmt.line_heights.len() == 1
            && tac_picture_tail_height.unwrap_or(total_advance_fit)
                > ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX * 3.0;
        let visible_separator_large_tac_tail_render_y =
            if visible_separator_large_tac_tail_candidate {
                self.predict_current_column_para_y(
                    &st,
                    en_para_idx,
                    paragraphs,
                    &styles,
                    measured_tables,
                    Some(en_col_w),
                )
            } else {
                None
            };
        let visible_separator_large_tac_tail_bottom = visible_separator_large_tac_tail_render_y
            .map(|render_y| render_y + tac_picture_tail_height.unwrap_or(h4f));
        let visible_separator_large_tac_tail_allows_small_bleed =
            visible_separator_large_tac_tail_candidate
                && visible_large_between_notes_gap
                && st.current_column + 1 >= st.col_count
                && en_ctrl.paragraphs.get(ep_idx + 1).is_some_and(|next_para| {
                    let next_comp =
                        crate::renderer::composer::compose_paragraph_in_context(next_para, styles);
                    let next_fmt = self.format_endnote_paragraph(
                        next_para,
                        Some(&next_comp),
                        &styles,
                        Some(en_col_w),
                    );
                    next_fmt.line_heights.len() == 1
                        && para_has_visible_text(next_para)
                        && !para_has_treat_as_char_picture_or_shape(next_para)
                        && !para_has_non_tac_picture_or_shape(next_para)
                });
        let visible_separator_large_tac_tail_overflow_limit =
            if visible_separator_large_tac_tail_allows_small_bleed {
                available + ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX
            } else {
                available + 1.0
            };
        let visible_separator_large_tac_tail_overflows_frame =
            visible_separator_large_tac_tail_bottom
                .is_some_and(|bottom| bottom > visible_separator_large_tac_tail_overflow_limit);
        let visible_separator_text_after_large_tac_tail_starts_next_page =
            !default_between_notes_gap
                && compact_endnote_separator_profile
                && has_visible_endnote_separator
                && visible_large_between_notes_gap
                && st.col_count > 1
                && st.current_column + 1 >= st.col_count
                && ep_idx > 1
                && st.current_height > available * 0.96
                && fmt.line_heights.len() == 1
                && para_has_visible_text(en_para)
                && !para_has_treat_as_char_picture_or_shape(en_para)
                && !para_has_non_tac_picture_or_shape(en_para)
                && en_ctrl
                    .paragraphs
                    .iter()
                    .take(ep_idx)
                    .skip(1)
                    .any(para_has_visible_text)
                && st
                    .current_items
                    .iter()
                    .rev()
                    .filter_map(page_item_para_index)
                    .next()
                    .and_then(|pi| {
                        paragraph_by_global_index(paragraphs, &st.endnote_paragraphs, pi).and_then(
                            |prev_para| {
                                prev_para
                                    .controls
                                    .iter()
                                    .filter_map(|ctrl| {
                                        crate::renderer::tac_object_flow_height_px(ctrl, dpi)
                                    })
                                    .reduce(f64::max)
                            },
                        )
                    })
                    .is_some_and(|height| height >= 80.0);
        let visible_separator_text_after_equation_tail_overflows_frame = !default_between_notes_gap
            && compact_endnote_separator_profile
            && has_visible_endnote_separator
            && visible_large_between_notes_gap
            && st.col_count > 1
            && st.current_column + 1 < st.col_count
            && ep_idx > 1
            && st.current_height > available * 0.90
            && fmt.line_heights.len() == 1
            && para_has_visible_text(en_para)
            && !para_has_treat_as_char_picture_or_shape(en_para)
            && !para_has_non_tac_picture_or_shape(en_para)
            && !local_vpos_rewind
            && !internal_vpos_rewind
            && st
                .current_items
                .last()
                .and_then(page_item_para_index)
                .is_some_and(|prev_pi| prev_pi + 1 == en_para_idx)
            && st
                .current_items
                .last()
                .and_then(page_item_para_index)
                .and_then(|prev_pi| {
                    paragraph_by_global_index(paragraphs, &st.endnote_paragraphs, prev_pi)
                })
                .is_some_and(|prev_para| {
                    !para_has_visible_text(prev_para)
                        && prev_para
                            .controls
                            .iter()
                            .any(|ctrl| is_treat_as_char_equation_control(Some(ctrl)))
                })
            && self
                .predict_current_column_para_y(
                    &st,
                    en_para_idx,
                    paragraphs,
                    &styles,
                    measured_tables,
                    Some(en_col_w),
                )
                .is_some_and(|render_y| render_y + fmt.line_advance(0) > available + 1.0);
        let zero_equation_text_run_tail_before_next_title_fits = compact_endnote_separator_profile
            && zero_endnote_spacing_profile
            && has_visible_endnote_separator
            && st.current_column + 1 < st.col_count
            && ep_idx + 1 == en_ctrl.paragraphs.len()
            && fmt.line_heights.len() == 1
            && line_is_equation_tac_text_run_only(en_para, &composed, 0)
            && next_endnote_title_fit_height.is_some_and(|next_h| {
                st.current_height + en_fit + next_h
                    <= available + ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX
            });
        let boundary_gap_tail_and_next_title_fit_current_column = !default_between_notes_gap
            && compact_endnote_separator_profile
            && has_visible_endnote_separator
            && ep_idx + 1 == en_ctrl.paragraphs.len()
            && st.current_column + 1 < st.col_count
            && st.current_height + total_advance_fit
                <= available + ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX
            && next_endnote_title_fit_height.is_some_and(|next_h| {
                st.current_height + total_advance_fit + next_h
                    <= available + ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX + 2.0
            });
        let no_separator_boundary_tail_without_gap_fits = large_separator_block
            && !has_visible_endnote_separator
            && endnote_boundary_gap_extra_px > 0.0
            && ep_idx + 1 == en_ctrl.paragraphs.len()
            && (st.current_column + 1 >= st.col_count
                || (st.current_column + 1 < st.col_count && st.current_height > available * 0.90))
            && !local_vpos_rewind
            && !internal_vpos_rewind
            && st.current_height < available
            && st.current_height + total_advance_fit
                <= available + ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX
            && para_has_visible_text_or_equation(en_para)
            && !para_has_non_tac_picture_or_shape(en_para);
        let endnote_boundary_gap_final_equation_tail_fits = endnote_boundary_gap_extra_px > 0.0
            && !default_between_notes_gap
            && compact_endnote_separator_profile
            && has_visible_endnote_separator
            && ep_idx + 1 == en_ctrl.paragraphs.len()
            && st.current_column + 1 >= st.col_count
            && !local_vpos_rewind
            && !internal_vpos_rewind
            && !para_has_visible_text(en_para)
            && line_is_equation_tac_text_run_only(en_para, &composed, 0)
            && st.current_height + total_advance_fit
                <= available + ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX + 80.0;
        let endnote_boundary_gap_tail_overflows_frame = endnote_boundary_gap_extra_px > 0.0
            && st.col_count > 1
            && ep_idx > 0
            && !local_vpos_rewind
            && !internal_vpos_rewind
            && st.current_height > available * 0.90
            && st.current_height + total_advance_fit + endnote_boundary_gap_extra_px
                > available + ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX
            && !boundary_gap_tail_and_next_title_fit_current_column
            && !no_separator_boundary_tail_without_gap_fits
            && !endnote_boundary_gap_final_equation_tail_fits
            && (para_has_visible_text_or_equation(en_para)
                || para_has_treat_as_char_picture_or_shape(en_para)
                || para_has_non_tac_picture_or_shape(en_para));
        let no_separator_final_tail_fits_by_visible_height = large_separator_block
            && !has_visible_endnote_separator
            && endnote_boundary_gap_extra_px > 0.0
            && ep_idx + 1 == en_ctrl.paragraphs.len()
            && st.current_column + 1 >= st.col_count
            && !local_vpos_rewind
            && !internal_vpos_rewind
            && st.current_height < available
            && para_has_visible_text_or_equation(en_para)
            && !para_has_non_tac_picture_or_shape(en_para)
            && en_fit > total_advance_fit + endnote_boundary_gap_extra_px + 20.0
            && st.current_height + total_advance_fit + endnote_boundary_gap_extra_px
                <= available + ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX + 2.0;
        let default_title_tail_body_advances_column = compact_endnote_separator_profile
            && default_between_notes_gap
            && has_visible_endnote_separator
            && ep_idx == 1
            && en_ref.number > 0
            && st.current_column + 1 < st.col_count
            && st.current_height > available * 0.925
            && st.current_height + fmt.total_height > available + 1.0
            && !local_vpos_rewind
            && !internal_vpos_rewind
            && en_ctrl
                .paragraphs
                .first()
                .is_some_and(|title_para| title_para.line_segs.len() == 1)
            && fmt.line_heights.len() <= 2
            && para_has_visible_text_or_equation(en_para)
            && endnote_has_visible_payload;
        let large_between_title_tail_body_advances_page = self
            .judge_large_between_title_tail_body_advances_page(
                st,
                &fmt,
                en_para,
                en_ctrl,
                en_ref,
                paragraphs,
                styles,
                available,
                en_col_w,
                en_para_idx,
                ep_idx,
                default_between_notes_gap,
                compact_endnote_separator_profile,
                has_visible_endnote_separator,
                local_vpos_rewind,
                internal_vpos_rewind,
                endnote_has_visible_payload,
            );
        let large_between_last_column_new_note_tail = !default_between_notes_gap
            && compact_endnote_separator_profile
            && has_visible_endnote_separator
            && ep_idx == 0
            && emitted_endnote_count > 0
            && st.current_column + 1 >= st.col_count
            && st.current_height > available * 0.90
            && (!endnote_has_vpos_rewind || st.current_height + en_fit > available)
            && !st.current_items.is_empty()
            && !large_between_last_column_question_title_tail_fits
            && !large_between_last_column_render_title_tail_fits
            && !large_between_last_column_rewind_title_tail_fits
            && endnote_has_visible_payload;
        let large_between_short_text_before_equation_tail_bleeds_previous_column =
            !default_between_notes_gap
                && compact_endnote_separator_profile
                && has_visible_endnote_separator
                && ep_idx > 0
                && en_ctrl.paragraphs.len().saturating_sub(ep_idx) >= 6
                && st.col_count > 1
                && st.current_column + 1 < st.col_count
                && st.current_height > available * 0.90
                && !st.current_items.is_empty()
                && !local_vpos_rewind
                && !internal_vpos_rewind
                && later_endnote_vpos_rewinds_after_current
                && fmt.line_heights.len() == 1
                && fmt.line_advance(0) <= 24.0
                && st.current_height + fmt.line_advance(0) <= available + 1.0
                && line_has_visible_text(&composed, 0)
                && !para_has_treat_as_char_picture_or_shape(en_para)
                && !para_has_non_tac_picture_or_shape(en_para)
                && en_ctrl.paragraphs.get(ep_idx + 1).is_some_and(|next_para| {
                    let next_comp =
                        crate::renderer::composer::compose_paragraph_in_context(next_para, styles);
                    let next_fmt = self.format_endnote_paragraph(
                        next_para,
                        Some(&next_comp),
                        &styles,
                        Some(en_col_w),
                    );
                    next_fmt.line_heights.len() == 1
                        && next_fmt.line_advance(0) <= 36.0
                        && line_is_equation_tac_text_run_only(next_para, &next_comp, 0)
                        && st.current_height + fmt.line_advance(0) + next_fmt.line_advance(0)
                            <= available + ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX + 80.0
                });
        let large_between_final_visible_equation_rewind_tail_starts_next_column =
            !default_between_notes_gap
                && compact_endnote_separator_profile
                && has_visible_endnote_separator
                && ep_idx > 0
                && en_ctrl.paragraphs.len().saturating_sub(ep_idx) <= 4
                && st.col_count > 1
                && st.current_column + 1 < st.col_count
                && st.current_height > available * 0.93
                && !st.current_items.is_empty()
                && !local_vpos_rewind
                && !internal_vpos_rewind
                && later_endnote_vpos_rewinds_after_current
                && fmt.line_heights.len() == 1
                && line_has_visible_text(&composed, 0)
                && !para_has_treat_as_char_picture_or_shape(en_para)
                && !para_has_non_tac_picture_or_shape(en_para)
                && en_ctrl.paragraphs.get(ep_idx + 1).is_some_and(|next_para| {
                    let next_comp =
                        crate::renderer::composer::compose_paragraph_in_context(next_para, styles);
                    let next_fmt = self.format_endnote_paragraph(
                        next_para,
                        Some(&next_comp),
                        &styles,
                        Some(en_col_w),
                    );
                    next_fmt.line_heights.len() == 1
                        && line_is_equation_tac_text_run_only(next_para, &next_comp, 0)
                });
        let large_between_lead_in_before_final_tail_starts_next_column = !default_between_notes_gap
            && compact_endnote_separator_profile
            && has_visible_endnote_separator
            && ep_idx > 0
            && ep_idx + 2 == en_ctrl.paragraphs.len()
            && endnote_refs.get(en_ref_idx + 1).is_some()
            && st.col_count > 1
            && st.current_column + 1 < st.col_count
            && !st.current_items.is_empty()
            && !internal_vpos_rewind
            && fmt.line_heights.len() == 1
            && line_has_visible_text(&composed, 0)
            && !para_has_treat_as_char_picture_or_shape(en_para)
            && !para_has_non_tac_picture_or_shape(en_para)
            && st.current_height + fmt.line_advance(0)
                <= available + ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX + 2.0
            && en_ctrl.paragraphs.get(ep_idx + 1).is_some_and(|next_para| {
                let next_comp =
                    crate::renderer::composer::compose_paragraph_in_context(next_para, styles);
                let next_fmt = self.format_endnote_paragraph(
                    next_para,
                    Some(&next_comp),
                    &styles,
                    Some(en_col_w),
                );
                let next_tail_gap = endnote_shape
                    .filter(|shape| {
                        let between_notes = endnote_between_notes_margin(shape) as i32;
                        between_notes > ENDNOTE_BETWEEN_NOTES_BASE_FLOW_HU
                            && !endnote_has_absorbed_between_notes_gap(shape)
                    })
                    .map(|shape| {
                        let between_notes = endnote_between_notes_margin(shape) as i32;
                        let saved_spacing = next_para
                            .line_segs
                            .last()
                            .map(|seg| seg.line_spacing.max(0))
                            .unwrap_or(0);
                        hwpunit_to_px((between_notes - saved_spacing).max(0), self.dpi)
                    })
                    .unwrap_or(0.0);
                let following_title_reserved = endnote_shape
                    .map(endnote_between_notes_margin)
                    .map(|gap| hwpunit_to_px(gap as i32, self.dpi))
                    .unwrap_or(0.0)
                    + 12.0;
                let next_is_tall_tail =
                    next_fmt.height_for_fit > 80.0 || next_fmt.line_heights.len() > 1;

                next_is_tall_tail
                    && st.current_height
                        + fmt.line_advance(0)
                        + next_fmt.height_for_fit
                        + next_tail_gap
                        + following_title_reserved
                        > available + ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX
            });
        let large_between_last_column_final_lead_tac_tail_starts_next_page =
            !default_between_notes_gap
                && compact_endnote_separator_profile
                && has_visible_endnote_separator
                && ep_idx > 0
                && ep_idx + 2 == en_ctrl.paragraphs.len()
                && endnote_refs.get(en_ref_idx + 1).is_some()
                && st.col_count > 1
                && st.current_column + 1 >= st.col_count
                && st.current_height > available * 0.85
                && !st.current_items.is_empty()
                && !local_vpos_rewind
                && !internal_vpos_rewind
                && fmt.line_heights.len() >= 2
                && para_has_visible_text_or_equation(en_para)
                && !para_has_treat_as_char_picture_or_shape(en_para)
                && !para_has_non_tac_picture_or_shape(en_para)
                && st.current_height + en_fit
                    <= available + ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX + 2.0
                && {
                    let last_line = fmt.line_heights.len() - 1;
                    line_has_tac_equation_control(en_para, &composed, last_line)
                }
                && en_ctrl.paragraphs.get(ep_idx + 1).is_some_and(|next_para| {
                    let next_comp =
                        crate::renderer::composer::compose_paragraph_in_context(next_para, styles);
                    let next_fmt = self.format_endnote_paragraph(
                        next_para,
                        Some(&next_comp),
                        &styles,
                        Some(en_col_w),
                    );
                    next_fmt.line_heights.len() == 1
                        && line_is_equation_tac_text_run_only(next_para, &next_comp, 0)
                        && st.current_height + en_fit + next_fmt.height_for_fit
                            > available - ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX
                });
        let zero_visible_text_tail_before_rewind_fits = self
            .judge_zero_visible_text_tail_before_rewind_fits(
                st,
                &fmt,
                en_para,
                en_ref,
                &composed,
                available,
                compact_endnote_separator_profile,
                en_fit,
                ep_idx,
                has_visible_endnote_separator,
                internal_vpos_rewind,
                later_endnote_vpos_rewinds_after_current,
                local_vpos_rewind,
                zero_endnote_spacing_profile,
            );
        let non_visible_endnote_tail_bleeds_previous_column = compact_endnote_separator_profile
            && default_between_notes_gap
            && ep_idx > 0
            && st.current_column + 1 >= st.col_count
            && !para_has_visible_text_or_equation(en_para)
            && !para_has_non_tac_picture_or_shape(en_para)
            && !local_vpos_rewind
            && !internal_vpos_rewind
            && st.current_height < available + ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX + 8.0
            && st.current_height + en_fit
                <= available + ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX + 80.0;
        let large_between_non_visible_tail_bleeds_previous_column =
            compact_endnote_separator_profile
                && !default_between_notes_gap
                && has_visible_endnote_separator
                && ep_idx > 0
                && st.current_column + 1 < st.col_count
                && !para_has_visible_text_or_equation(en_para)
                && !para_has_non_tac_picture_or_shape(en_para)
                && !local_vpos_rewind
                && !internal_vpos_rewind
                && st.current_height < available + ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX + 8.0
                && st.current_height + en_fit
                    <= available + ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX + 80.0;
        let zero_visible_last_column_text_tail_starts_next_page = compact_endnote_separator_profile
            && zero_endnote_spacing_profile
            && has_visible_endnote_separator
            && ep_idx > 0
            && st.current_column + 1 >= st.col_count
            && !local_vpos_rewind
            && !internal_vpos_rewind
            && !para_is_treat_as_char_picture_only(en_para)
            && para_has_visible_text_or_equation(en_para)
            && st.current_height > available * 0.96
            && st.current_height + fmt.total_height > available + 1.0
            && (fmt.line_heights.len() > 1
                    // 0/0/0 미주는 마지막 단 바닥의 한 줄짜리 설명 뒤에
                    // 큰 TAC 그림이 바로 이어지는 경우, 설명 줄도 현재
                    // frame 아래로 잘리므로 한컴처럼 다음 쪽으로 넘긴다.
                    || (fmt.line_heights.len() == 1
                        && st.current_height > available * 0.99
                        && en_ctrl
                            .paragraphs
                            .get(ep_idx + 1)
                            .is_some_and(para_is_treat_as_char_picture_only)));
        let zero_between_visible_last_column_text_tail_starts_next_page =
            compact_endnote_separator_profile
                && visible_zero_between_large_separator_gap
                && ep_idx > 0
                && st.current_column + 1 >= st.col_count
                && !local_vpos_rewind
                && !internal_vpos_rewind
                && !para_is_treat_as_char_picture_only(en_para)
                && para_has_visible_text_or_equation(en_para)
                && st.current_height > available * 0.96
                && st.current_height + fmt.total_height > available + 1.0
                && (fmt.line_heights.len() > 1
                    || (fmt.line_heights.len() == 1
                        && st.current_height > available * 0.99
                        && en_ctrl
                            .paragraphs
                            .get(ep_idx + 1)
                            .is_some_and(para_is_treat_as_char_picture_only)));
        // [#4318] 구분선 위/아래 20mm + 기본 미주 사이 마지막 단 한 줄 꼬리.
        let last_column_visible_text_tail_starts_next_page = compact_endnote_separator_profile
            && both_large_separator_default_between
            && has_visible_endnote_separator
            && ep_idx > 0
            && st.current_column + 1 >= st.col_count
            && !local_vpos_rewind
            && !internal_vpos_rewind
            && !para_is_treat_as_char_picture_only(en_para)
            && para_has_visible_text_or_equation(en_para)
            && fmt.line_heights.len() == 1
            && endnote_last_column_tail_overflows_frame(
                st.current_height,
                fmt.total_height,
                available,
            );
        let large_between_zero_above_whole_note_small_bleed_fits = compact_endnote_separator_profile
            && visible_large_between_zero_above_compact_below
            && ep_idx == 0
            && en_ref.number > 0
            && st.current_column + 1 >= st.col_count
            && !st.current_items.is_empty()
            && !local_vpos_rewind
            && !internal_vpos_rewind
            && st.current_height < available * 0.35
            && st.current_height + en_fit > available
            && st.current_height + en_fit
                <= available + ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX + 6.0
            && endnote_has_visible_payload;
        let advance_for_fit = ((st.current_height + en_fit > available
            && !no_separator_final_tail_fits_by_visible_height
            && !no_separator_visible_multiline_tail_fits_with_bleed
            && !large_between_zero_above_whole_note_small_bleed_fits)
            || (late_compact_text_tail_overflow_risk
                && !zero_equation_text_run_tail_before_next_title_fits)
            || zero_tac_picture_tail_bleeds_frame
            || visible_separator_large_tac_tail_overflows_frame
            || visible_separator_text_after_large_tac_tail_starts_next_page
            || visible_separator_text_after_equation_tail_overflows_frame
            || zero_visible_last_column_text_tail_starts_next_page
            || zero_between_visible_last_column_text_tail_starts_next_page
            || last_column_visible_text_tail_starts_next_page
            || endnote_boundary_gap_tail_overflows_frame
            || default_title_tail_body_advances_column
            || large_between_title_tail_body_advances_page
            || large_between_split_head_render_overflows
            || large_between_last_column_new_note_tail
            || no_separator_tail_after_picture_starts_next_page
            || zero_between_large_separator_last_column_title_orphan
            || large_between_equation_tail_starts_next_column
            || large_between_final_visible_equation_rewind_tail_starts_next_column
            || large_between_lead_in_before_final_tail_starts_next_column
            || large_between_last_column_final_lead_tac_tail_starts_next_page
            || no_separator_saved_vpos_tail_outside
            || visible_separator_saved_vpos_tail_outside
            || internal_rewind_head_overflows_current_column
            || internal_reset_split_head_render_overflows
            || internal_rewind_full_advance_needed)
            && (split_endnote_to_fit.is_none()
                || (late_compact_text_tail_overflow_risk
                    && !zero_equation_text_run_tail_before_next_title_fits)
                || internal_rewind_full_advance_needed)
            && large_between_last_column_visual_split.is_none()
            && large_between_last_column_flow_tail_split.is_none()
            && (!internal_rewind_head_allows_current_column
                || internal_reset_split_head_render_overflows
                || internal_rewind_full_advance_needed)
            && (!compact_endnote_own_vpos_span_fits_for_flow
                || late_compact_text_tail_overflow_risk
                || internal_rewind_head_overflows_current_column
                || default_title_tail_body_advances_column
                || large_between_title_tail_body_advances_page
                || large_between_split_head_render_overflows
                || visible_separator_large_tac_tail_overflows_frame
                || visible_separator_text_after_large_tac_tail_starts_next_page
                || visible_separator_text_after_equation_tail_overflows_frame
                || zero_visible_last_column_text_tail_starts_next_page
                || zero_between_visible_last_column_text_tail_starts_next_page
                || last_column_visible_text_tail_starts_next_page
                || zero_between_large_separator_last_column_title_orphan
                || large_between_last_column_final_lead_tac_tail_starts_next_page
                || internal_reset_split_head_render_overflows
                || internal_rewind_full_advance_needed)
            && !allow_compact_question_title_tail
            && !default_question_title_tail_fits_by_line_height
            && !zero_question_title_tail_fits_by_line_height
            && !zero_question_intro_tail_before_rewind_fits
            && !zero_visible_text_tail_before_rewind_fits
            && !zero_between_large_separator_tail_group_fits
            && !large_between_last_column_question_title_tail_fits
            && !large_between_last_column_render_title_tail_fits
            && !large_between_last_column_rewind_title_tail_fits
            && !large_between_last_column_title_body_tail_fits
            && (!default_between_notes_gap
                || internal_rewind_split.is_none()
                || internal_rewind_head_overflows_current_column
                || internal_rewind_full_advance_needed)
            && !late_question_title_small_overflow
            && !allow_large_between_question_title_tail
            && !large_between_last_column_question_title_tail_fits
            && !allow_default_column_bottom_question_title_tail
            && !late_question_intro_tail
            && !late_question_continuation_tail
            && !large_between_short_text_before_equation_tail_bleeds_previous_column
            && (!non_visible_endnote_tail_bleeds_previous_column
                || visible_separator_large_tac_tail_overflows_frame)
            && !large_between_non_visible_tail_bleeds_previous_column
            && !st.current_items.is_empty();
        let pre_emit_tail_before_non_tac_object_advance = self
            .judge_pre_emit_tail_before_non_tac_object_advance(
                zero_between_large_separator_margin,
                advance_for_fit,
                st,
                en_ctrl,
                styles,
                endnote_shape,
                available,
                compact_endnote_separator_profile,
                en_col_w,
                endnote_has_text_or_equation,
                ep_idx,
                has_visible_endnote_separator,
                large_separator_block,
                non_tac_object_height,
            );
        TailFitResult {
            allow_large_between_question_title_tail,
            allow_default_column_bottom_question_title_tail,
            allow_default_first_column_large_below_title_tail,
            new_endnote_advance_threshold,
            allow_compact_question_title_tail,
            allow_large_separator_first_column_tail,
            large_between_last_column_question_title_tail_fits,
            large_between_last_column_render_title_tail_fits,
            large_between_last_column_rewind_title_tail_fits,
            default_question_title_tail_fits_by_line_height,
            zero_question_title_tail_fits_by_line_height,
            large_between_zero_above_whole_note_small_bleed_fits,
            advance_for_fit,
            pre_emit_tail_before_non_tac_object_advance,
        }
    }
}

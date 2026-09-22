//! 미주 내부 문단 흐름 조정. 준비·조회·상태 반영의 기존 순서를 보존한다.

use crate::renderer::typeset::notes::endnotes::profile::{
    endnote_between_notes_margin, endnote_separator_below_margin,
    ENDNOTE_BETWEEN_NOTES_BASE_FLOW_HU,
};
use crate::renderer::typeset::{
    hwpunit_to_px, page_item_para_index, para_has_visible_text_or_equation,
    paragraph_by_global_index, EndnoteRef, FootnoteShape, FormattedParagraph, Paragraph,
    ResolvedStyleSet, TypesetEngine, TypesetState, ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX,
};

/// 호출 시점의 관측값. 페이지 상태를 변경할 수 없는 Query 입력이다.
pub(super) struct NewNoteFitInput<'a> {
    pub(super) st: &'a TypesetState,
    pub(super) paragraphs: &'a [Paragraph],
    pub(super) styles: &'a ResolvedStyleSet,
    pub(super) endnote_shape: Option<&'a FootnoteShape>,
    pub(super) en_ref: &'a EndnoteRef,
    pub(super) en_ctrl: &'a crate::model::footnote::Endnote,
    pub(super) ep_idx: usize,
    pub(super) en_para: &'a Paragraph,
    pub(super) fmt: &'a FormattedParagraph,
    pub(super) endnote_start: i32,
    pub(super) emitted_endnote_count: usize,
    pub(super) this_first_offset: Option<i32>,
    pub(super) new_endnote_between_notes_px: Option<f64>,
    pub(super) col_count: u16,
    pub(super) split_endnote_to_fit: Option<usize>,
    pub(super) available: f64,
    pub(super) en_col_w: f64,
    pub(super) dpi: f64,
    pub(super) total_advance_fit: f64,
    pub(super) en_fit: f64,
    pub(super) new_endnote_advance_threshold: f64,
    pub(super) endnote_has_vpos_rewind: bool,
    pub(super) compact_endnote_separator_profile: bool,
    pub(super) prev_endnote_had_inline_object_vpos_overestimate: bool,
    pub(super) local_vpos_rewind: bool,
    pub(super) has_visible_endnote_separator: bool,
    pub(super) internal_vpos_rewind: bool,
    pub(super) endnote_has_visible_payload: bool,
    pub(super) move_internal_rewind_equation_to_next: bool,
    pub(super) default_between_notes_gap: bool,
    pub(super) zero_endnote_spacing_profile: bool,
    pub(super) compact_between_notes_gap: bool,
    pub(super) visible_large_between_notes_gap: bool,
    pub(super) allow_default_late_question_tail: bool,
    pub(super) no_separator_new_note_head_fits_current_column: bool,
    pub(super) no_separator_last_column_new_note_head_without_gap_fits: bool,
    pub(super) next_endnote_first_para_fit_height: Option<f64>,
    pub(super) no_separator_compact_final_note_first_line_tail_fits: bool,
    pub(super) new_endnote_stale_forward_vpos: bool,
    pub(super) large_between_question_title_render_y: Option<f64>,
    pub(super) large_between_question_title_render_head_outside: bool,
    pub(super) large_between_question_lead_group_render_outside: bool,
    pub(super) next_endnote_first_line_advance: Option<f64>,
    pub(super) allow_large_between_question_title_tail: bool,
    pub(super) allow_default_column_bottom_question_title_tail: bool,
    pub(super) allow_default_first_column_large_below_title_tail: bool,
    pub(super) allow_compact_question_title_tail: bool,
    pub(super) allow_large_separator_first_column_tail: bool,
    pub(super) large_between_last_column_question_title_tail_fits: bool,
    pub(super) large_between_last_column_render_title_tail_fits: bool,
    pub(super) large_between_last_column_rewind_title_tail_fits: bool,
    pub(super) default_question_title_tail_fits_by_line_height: bool,
    pub(super) zero_question_title_tail_fits_by_line_height: bool,
    pub(super) large_between_zero_above_whole_note_small_bleed_fits: bool,
}

/// 기존 순서로 계산한 후보. 적용은 문단 조정자가 담당한다.
pub(super) struct NewNoteFitResult {
    pub(super) advance_for_new_endnote: bool,
    pub(super) advance_for_internal_rewind: bool,
}

impl TypesetEngine {
    pub(super) fn query_new_note_fit(&self, input: NewNoteFitInput<'_>) -> NewNoteFitResult {
        let NewNoteFitInput {
            st,
            paragraphs,
            styles,
            endnote_shape,
            en_ref,
            en_ctrl,
            ep_idx,
            en_para,
            fmt,
            endnote_start,
            emitted_endnote_count,
            this_first_offset,
            new_endnote_between_notes_px,
            col_count,
            split_endnote_to_fit,
            available,
            en_col_w,
            dpi,
            total_advance_fit,
            en_fit,
            new_endnote_advance_threshold,
            endnote_has_vpos_rewind,
            compact_endnote_separator_profile,
            prev_endnote_had_inline_object_vpos_overestimate,
            local_vpos_rewind,
            has_visible_endnote_separator,
            internal_vpos_rewind,
            endnote_has_visible_payload,
            move_internal_rewind_equation_to_next,
            default_between_notes_gap,
            zero_endnote_spacing_profile,
            compact_between_notes_gap,
            visible_large_between_notes_gap,
            allow_default_late_question_tail,
            no_separator_new_note_head_fits_current_column,
            no_separator_last_column_new_note_head_without_gap_fits,
            next_endnote_first_para_fit_height,
            no_separator_compact_final_note_first_line_tail_fits,
            new_endnote_stale_forward_vpos,
            large_between_question_title_render_y,
            large_between_question_title_render_head_outside,
            large_between_question_lead_group_render_outside,
            next_endnote_first_line_advance,
            allow_large_between_question_title_tail,
            allow_default_column_bottom_question_title_tail,
            allow_default_first_column_large_below_title_tail,
            allow_compact_question_title_tail,
            allow_large_separator_first_column_tail,
            large_between_last_column_question_title_tail_fits,
            large_between_last_column_render_title_tail_fits,
            large_between_last_column_rewind_title_tail_fits,
            default_question_title_tail_fits_by_line_height,
            zero_question_title_tail_fits_by_line_height,
            large_between_zero_above_whole_note_small_bleed_fits,
        } = input;
        let allow_default_question_title_tail = default_between_notes_gap
            && prev_endnote_had_inline_object_vpos_overestimate
            && ep_idx == 0
            && en_fit <= 24.0
            && st.current_height + en_fit <= available - 40.0;
        let allow_default_question_title_tail = allow_default_question_title_tail
                // 보이는 구분선의 기본 미주 사이에서는 새 문항 제목 한 줄이
                // 단 하단에 몰려 있지 않으면 한컴처럼 현재 단에 남긴다.
                // 전체 tail을 기준으로 밀면 문항 본문이 다음 단으로 과하게 넘어간다.
                || (default_between_notes_gap
                    && has_visible_endnote_separator
                    && ep_idx == 0
                    && st.current_column + 1 < st.col_count
                    && en_fit <= 24.0
                    && st.current_height < available * 0.85
                    && st.current_height + en_fit <= available - 40.0);
        let allow_default_question_title_tail = allow_default_question_title_tail
                // 구분선 아래가 큰 기본 미주에서는 저장 vpos rewind 때문에
                // 제목+head 묶음 전체가 current_height 기준보다 커 보일 수 있다.
                // 제목 앞 공식 "미주 사이" gap과 제목 한 줄이 현재 단에
                // 들어가면 한컴처럼 제목/head를 단 하단에 남기고 뒤에서
                // 자연스럽게 split되도록 advance를 막는다.
                || (default_between_notes_gap
                    && compact_endnote_separator_profile
                    && has_visible_endnote_separator
                    && endnote_has_vpos_rewind
                    && ep_idx == 0
                    && en_ref.number > 0
                    && st.current_column + 1 < st.col_count
                    && !st.current_items.is_empty()
                    && fmt.line_heights.len() == 1
                    && st.current_height > available * 0.85
                    && st.current_height < available * 0.93
                    && endnote_shape
                        .map(|shape| {
                            endnote_separator_below_margin(shape) as i32
                                > ENDNOTE_BETWEEN_NOTES_BASE_FLOW_HU
                        })
                        .unwrap_or(false)
                    && endnote_shape
                        .map(|shape| endnote_between_notes_margin(shape) as i32)
                        .filter(|gap_hu| {
                            st.current_height
                                + hwpunit_to_px(*gap_hu, self.dpi)
                                + en_fit
                                <= available
                                    + ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX
                                    + 2.0
                        })
                        .is_some()
                    && para_has_visible_text_or_equation(en_para));
        let allow_default_question_title_tail = allow_default_question_title_tail
                // 구분선 아래가 기본값 근방이어도 저장 vpos rewind가 있는
                // 새 미주 제목은 제목 앞 공식 "미주 사이" gap까지 현재
                // 단에 들어가면 하단 tail로 남긴다. head group 전체를
                // 기준으로 밀면 한컴보다 다음 단으로 일찍 넘어간다.
                || (default_between_notes_gap
                    && compact_endnote_separator_profile
                    && has_visible_endnote_separator
                    && endnote_has_vpos_rewind
                    && ep_idx == 0
                    && en_ref.number > 0
                    && st.current_column + 1 < st.col_count
                    && !st.current_items.is_empty()
                    && fmt.line_heights.len() == 1
                    && st.current_height > available * 0.85
                    && st.current_height < available * 0.90
                    && endnote_shape
                        .map(|shape| {
                            endnote_separator_below_margin(shape) as i32
                                <= ENDNOTE_BETWEEN_NOTES_BASE_FLOW_HU
                                && st.current_height
                                    + hwpunit_to_px(
                                        endnote_between_notes_margin(shape)
                                            as i32,
                                        self.dpi,
                                    )
                                    + en_fit
                                    <= available
                                        + ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX
                                        + 2.0
                        })
                        .unwrap_or(false)
                    && para_has_visible_text_or_equation(en_para));
        let rewind_endnote_head_near_bottom = endnote_has_vpos_rewind
            && st.current_height + total_advance_fit
                > available - ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX;
        let rewind_endnote_head_would_split = endnote_has_vpos_rewind
            && next_endnote_first_line_advance
                .map(|next_h| {
                    st.current_height + total_advance_fit + next_h
                        > available - ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX
                })
                .unwrap_or(false);
        let large_between_notes_head_near_bottom = !default_between_notes_gap
            && !compact_between_notes_gap
            && ep_idx == 0
            && emitted_endnote_count > 0
            && !no_separator_new_note_head_fits_current_column
            && !large_between_zero_above_whole_note_small_bleed_fits
            && new_endnote_between_notes_px
                .map(|gap| {
                    // 미주 사이가 기본값보다 큰 문서는 새 번호 제목을
                    // 한 줄짜리 tail로만 보지 않고, 번호 경계 gap까지
                    // 함께 현재 단에 들어가는지 판단해야 한다.
                    let reserved_head = en_fit.max(fmt.line_advance(0) + gap);
                    st.current_height + reserved_head
                        > available - ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX
                })
                .unwrap_or(false);
        let visible_separator_vpos_head_group_outside = self
            .judge_visible_separator_vpos_head_group_outside(
                st,
                paragraphs,
                en_ctrl,
                this_first_offset,
                endnote_start,
                available,
                ep_idx,
                emitted_endnote_count,
                default_between_notes_gap,
                compact_endnote_separator_profile,
                has_visible_endnote_separator,
                compact_between_notes_gap,
            );
        let default_between_large_below_head_group_outside = self
            .judge_default_between_large_below_head_group_outside(
                st,
                en_ctrl,
                endnote_shape,
                styles,
                available,
                en_col_w,
                ep_idx,
                emitted_endnote_count,
                default_between_notes_gap,
                compact_endnote_separator_profile,
                has_visible_endnote_separator,
            );
        let large_between_last_column_vpos_head_group_outside = self
            .judge_large_between_last_column_vpos_head_group_outside(
                st,
                paragraphs,
                en_ctrl,
                large_between_question_title_render_y,
                endnote_start,
                available,
                ep_idx,
                emitted_endnote_count,
                default_between_notes_gap,
                compact_endnote_separator_profile,
                has_visible_endnote_separator,
                visible_large_between_notes_gap,
                compact_between_notes_gap,
            );
        let large_between_notes_vpos_head_outside = large_between_notes_head_near_bottom
            || large_between_question_title_render_head_outside
            || large_between_question_lead_group_render_outside
            || visible_separator_vpos_head_group_outside
            || default_between_large_below_head_group_outside
            || large_between_last_column_vpos_head_group_outside
            || (!default_between_notes_gap
                && !compact_between_notes_gap
                && ep_idx == 0
                && !no_separator_new_note_head_fits_current_column
                && st.current_column + 1 >= st.col_count
                && st.current_height > available * 0.75
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
                            let predicted_y =
                                hwpunit_to_px((first_vpos - base_vpos).max(0), self.dpi);
                            predicted_y + fmt.line_advance(0)
                                > available - 2.0 * ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX
                        })
                    })
                    .unwrap_or(false));
        let zero_new_endnote_full_tail_fits_current_column = zero_endnote_spacing_profile
            && has_visible_endnote_separator
            && ep_idx == 0
            && st.current_column + 1 < st.col_count
            && !st.current_items.is_empty()
            && endnote_has_visible_payload
            && en_ctrl
                .paragraphs
                .iter()
                .flat_map(|p| p.line_segs.iter())
                .fold(None::<(i32, i32)>, |acc, seg| {
                    let first = seg.vertical_pos + endnote_start;
                    let bottom = first + seg.line_height.saturating_add(seg.line_spacing);
                    Some(match acc {
                        Some((min_first, max_bottom)) => {
                            (min_first.min(first), max_bottom.max(bottom))
                        }
                        None => (first, bottom),
                    })
                })
                .map(|(first, bottom)| {
                    let saved_span = hwpunit_to_px((bottom - first).max(0), self.dpi);
                    let sequential_span: f64 = en_ctrl
                        .paragraphs
                        .iter()
                        .map(|p| {
                            let comp =
                                crate::renderer::composer::compose_paragraph_in_context(p, styles);
                            self.format_endnote_paragraph(p, Some(&comp), &styles, Some(en_col_w))
                                .total_height
                        })
                        .sum();
                    let note_span = if endnote_has_vpos_rewind {
                        saved_span
                    } else {
                        saved_span.max(sequential_span)
                    };
                    st.current_height + note_span
                        <= available + ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX + 1.0
                })
                .unwrap_or(false);
        let zero_between_question_title_tail_fits_current_column = endnote_shape
            .map(|shape| {
                compact_endnote_separator_profile
                    && has_visible_endnote_separator
                    && endnote_between_notes_margin(shape) == 0
            })
            .unwrap_or(false)
            && ep_idx == 0
            && en_ref.number > 0
            && st.current_column + 1 < st.col_count
            && !st.current_items.is_empty()
            && fmt.line_heights.len() == 1
            && st.current_height + fmt.line_advance(0)
                <= available + ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX + 2.0;
        let no_separator_compact_new_note_tail_fits_current_column =
            compact_endnote_separator_profile
                && default_between_notes_gap
                && compact_between_notes_gap
                && !has_visible_endnote_separator
                && ep_idx == 0
                && emitted_endnote_count > 0
                && en_ref.number > 0
                && !local_vpos_rewind
                && !internal_vpos_rewind
                && fmt.line_heights.len() > 1
                && endnote_has_visible_payload
                && if st.current_column + 1 < st.col_count {
                    // 비가시 구분선의 첫 단은 새 미주 전체가 충분한
                    // 여유를 두고 들어갈 때만 조기 단 넘김을 억제한다.
                    st.current_height + en_fit <= available - 64.0
                } else {
                    // 마지막 단에서는 한컴처럼 여러 줄 tail의 소폭
                    // bleed를 허용하되, 한 줄짜리 다음 번호까지 끌고
                    // 오지는 않는다.
                    en_fit <= 32.0
                        && st.current_height + en_fit
                            <= available + ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX
                };
        let no_separator_default_first_column_single_line_tail_fits =
            compact_endnote_separator_profile
                && default_between_notes_gap
                && compact_between_notes_gap
                && !has_visible_endnote_separator
                && ep_idx == 0
                && emitted_endnote_count > 0
                && en_ref.number > 0
                && st.current_column + 1 < st.col_count
                && fmt.line_heights.len() == 1
                && !local_vpos_rewind
                && !internal_vpos_rewind
                && para_has_visible_text_or_equation(en_para)
                && st.current_height + fmt.line_advance(0) <= available - 64.0;
        let no_separator_compact_new_note_overflows_current_column =
            compact_endnote_separator_profile
                && default_between_notes_gap
                && compact_between_notes_gap
                && !has_visible_endnote_separator
                && ep_idx == 0
                && emitted_endnote_count > 0
                && en_ref.number > 0
                && fmt.total_height > 32.0
                && st.current_height + en_fit > available;
        let no_separator_compact_last_column_title_before_tall_next =
            compact_endnote_separator_profile
                && default_between_notes_gap
                && compact_between_notes_gap
                && !has_visible_endnote_separator
                && ep_idx == 0
                && emitted_endnote_count > 0
                && en_ref.number > 0
                && st.current_column + 1 >= st.col_count
                && fmt.line_heights.len() == 1
                && para_has_visible_text_or_equation(en_para)
                && st.current_height > available * 0.90
                && st.current_height + fmt.line_advance(0)
                    <= available + ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX
                && next_endnote_first_para_fit_height.is_some_and(|next_h| {
                    next_h > 64.0
                        && st.current_height + fmt.line_advance(0) + next_h
                            > available + ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX
                });
        let no_separator_compact_last_column_single_line_tail_too_low =
            compact_endnote_separator_profile
                && default_between_notes_gap
                && compact_between_notes_gap
                && !has_visible_endnote_separator
                && ep_idx == 0
                && emitted_endnote_count > 0
                && en_ref.number > 0
                && st.current_column + 1 >= st.col_count
                && fmt.line_heights.len() == 1
                && para_has_visible_text_or_equation(en_para)
                && st.current_height + fmt.line_advance(0)
                    > available - ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX * 0.5
                && next_endnote_first_para_fit_height.is_some_and(|next_h| next_h <= 18.0);
        let no_separator_compact_last_column_note_before_tall_next =
            compact_endnote_separator_profile
                && default_between_notes_gap
                && compact_between_notes_gap
                && !has_visible_endnote_separator
                && ep_idx == 0
                && emitted_endnote_count > 0
                && en_ref.number > 0
                && st.current_column + 1 >= st.col_count
                && fmt.line_heights.len() > 1
                && para_has_visible_text_or_equation(en_para)
                && st.current_height > available * 0.90
                && st.current_height + en_fit <= available
                && next_endnote_first_para_fit_height.is_some_and(|next_h| {
                    st.current_height + en_fit + next_h
                        > available + ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX
                });
        let no_separator_compact_last_column_multi_line_tail_too_low =
            compact_endnote_separator_profile
                && default_between_notes_gap
                && compact_between_notes_gap
                && !has_visible_endnote_separator
                && ep_idx == 0
                && emitted_endnote_count > 0
                && en_ref.number > 0
                && st.current_column + 1 >= st.col_count
                && fmt.line_heights.len() > 1
                && para_has_visible_text_or_equation(en_para)
                && split_endnote_to_fit.is_none()
                && st.current_height > available * 0.90
                // 비가시 구분선 미주의 마지막 단에서는 계산상
                // 맞더라도 렌더 라인 높이가 커져 하단이 잘릴 수
                // 있으므로 다중 행 새 미주는 충분한 여백을 둔다.
                && st.current_height + en_fit > available - 32.0;
        let advance_for_new_endnote = st.col_count > 1
            && compact_endnote_separator_profile
            && ep_idx == 0
            && emitted_endnote_count > 0
            && !no_separator_new_note_head_fits_current_column
            && !no_separator_last_column_new_note_head_without_gap_fits
            && !allow_default_late_question_tail
            && (!allow_default_column_bottom_question_title_tail
                || no_separator_compact_last_column_single_line_tail_too_low
                || no_separator_compact_last_column_title_before_tall_next
                || no_separator_compact_last_column_note_before_tall_next
                || no_separator_compact_last_column_multi_line_tail_too_low
                || (large_between_notes_vpos_head_outside
                    && !allow_default_first_column_large_below_title_tail))
            && !allow_default_question_title_tail
            && !allow_large_between_question_title_tail
            && !large_between_last_column_question_title_tail_fits
            && !large_between_last_column_render_title_tail_fits
            && !large_between_last_column_rewind_title_tail_fits
            && !default_question_title_tail_fits_by_line_height
            && !zero_question_title_tail_fits_by_line_height
            && !allow_compact_question_title_tail
            && !allow_large_separator_first_column_tail
            && !zero_new_endnote_full_tail_fits_current_column
            && !zero_between_question_title_tail_fits_current_column
            && !no_separator_compact_new_note_tail_fits_current_column
            && !no_separator_compact_final_note_first_line_tail_fits
            && !no_separator_default_first_column_single_line_tail_fits
            && !large_between_zero_above_whole_note_small_bleed_fits
            && (!endnote_has_vpos_rewind
                || rewind_endnote_head_near_bottom
                || rewind_endnote_head_would_split
                || large_between_notes_vpos_head_outside)
            && (!new_endnote_stale_forward_vpos || large_between_notes_vpos_head_outside)
            && (st.current_height > available * new_endnote_advance_threshold
                || large_between_notes_vpos_head_outside
                || no_separator_compact_new_note_overflows_current_column
                || no_separator_compact_last_column_single_line_tail_too_low
                || no_separator_compact_last_column_title_before_tall_next
                || no_separator_compact_last_column_note_before_tall_next
                || no_separator_compact_last_column_multi_line_tail_too_low)
            && !st.current_items.is_empty();
        let advance_for_internal_rewind =
            move_internal_rewind_equation_to_next && !st.current_items.is_empty();
        NewNoteFitResult {
            advance_for_new_endnote,
            advance_for_internal_rewind,
        }
    }
}

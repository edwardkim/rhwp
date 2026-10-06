//! 미주 내부 문단 흐름 조정. 준비·조회·상태 반영의 기존 순서를 보존한다.

use crate::renderer::typeset::{
    hwpunit_to_px, line_has_visible_text, line_is_equation_tac_text_run_only,
    para_has_non_tac_picture_or_shape, para_has_treat_as_char_picture_or_shape, ComposedParagraph,
    Control, EndnoteRef, FormattedParagraph, MeasuredTable, Paragraph, ResolvedStyleSet,
    TypesetEngine, TypesetState, ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX,
};

/// 호출 시점의 관측값. 페이지 상태를 변경할 수 없는 Query 입력이다.
pub(super) struct RewindTailInput<'a> {
    pub(super) st: &'a TypesetState,
    pub(super) paragraphs: &'a [Paragraph],
    pub(super) styles: &'a ResolvedStyleSet,
    pub(super) measured_tables: &'a [MeasuredTable],
    pub(super) en_ref: &'a EndnoteRef,
    pub(super) en_ctrl: &'a crate::model::footnote::Endnote,
    pub(super) ep_idx: usize,
    pub(super) en_para: &'a Paragraph,
    pub(super) fmt: &'a FormattedParagraph,
    pub(super) composed: &'a ComposedParagraph,
    pub(super) endnote_start: i32,
    pub(super) prev_en_bottom_vpos: Option<i32>,
    pub(super) emitted_endnote_count: usize,
    pub(super) en_para_idx: usize,
    pub(super) this_first_offset: Option<i32>,
    pub(super) this_bottom_offset: Option<i32>,
    pub(super) col_count: u16,
    pub(super) split_endnote_to_fit: Option<usize>,
    pub(super) available: f64,
    pub(super) en_col_w: f64,
    pub(super) dpi: f64,
    pub(super) h4f: f64,
    pub(super) en_fit: f64,
    pub(super) endnote_has_vpos_rewind: bool,
    pub(super) compact_endnote_separator_profile: bool,
    pub(super) local_vpos_rewind: bool,
    pub(super) has_visible_endnote_separator: bool,
    pub(super) large_vpos_jump_at_column_top: bool,
    pub(super) internal_vpos_rewind: bool,
    pub(super) large_separator_block: bool,
    pub(super) default_between_notes_gap: bool,
    pub(super) zero_endnote_spacing_profile: bool,
    pub(super) compact_between_notes_gap: bool,
    pub(super) visible_large_between_notes_gap: bool,
    pub(super) visible_compact_sequential_tail_fits_current_column: bool,
}

/// 기존 순서로 계산한 후보. 적용은 문단 조정자가 담당한다.
pub(super) struct RewindTailResult {
    pub(super) new_endnote_stale_forward_vpos: bool,
    pub(super) large_between_tail_render_overflows: bool,
    pub(super) large_between_tail_before_rewind_picture: bool,
    pub(super) no_separator_tail_table_starts_next_column: bool,
    pub(super) no_separator_last_column_tail_before_rewind_starts_next_page: bool,
    pub(super) no_separator_tail_after_picture_starts_next_page: bool,
    pub(super) later_endnote_vpos_rewinds_after_current: bool,
    pub(super) large_between_equation_tail_starts_next_column: bool,
    pub(super) large_between_title_tail_render_overflows: bool,
    pub(super) large_between_question_title_render_y: Option<f64>,
    pub(super) large_between_question_title_head_inside_frame: bool,
    pub(super) large_between_question_title_head_fits_flow: bool,
    pub(super) large_between_question_title_render_head_outside: bool,
    pub(super) large_between_question_lead_group_render_outside: bool,
    pub(super) large_between_last_column_visual_split: Option<usize>,
    pub(super) large_between_last_column_flow_tail_split: Option<usize>,
}

impl TypesetEngine {
    pub(super) fn query_rewind_tail(&self, input: RewindTailInput<'_>) -> RewindTailResult {
        let RewindTailInput {
            st,
            paragraphs,
            styles,
            measured_tables,
            en_ref,
            en_ctrl,
            ep_idx,
            en_para,
            fmt,
            composed,
            endnote_start,
            prev_en_bottom_vpos,
            emitted_endnote_count,
            en_para_idx,
            this_first_offset,
            this_bottom_offset,
            col_count,
            split_endnote_to_fit,
            available,
            en_col_w,
            dpi,
            h4f,
            en_fit,
            endnote_has_vpos_rewind,
            compact_endnote_separator_profile,
            local_vpos_rewind,
            has_visible_endnote_separator,
            large_vpos_jump_at_column_top,
            internal_vpos_rewind,
            large_separator_block,
            default_between_notes_gap,
            zero_endnote_spacing_profile,
            compact_between_notes_gap,
            visible_large_between_notes_gap,
            visible_compact_sequential_tail_fits_current_column,
        } = input;
        let new_endnote_stale_forward_vpos = compact_endnote_separator_profile
            && ep_idx == 0
            && emitted_endnote_count > 0
            && !local_vpos_rewind
            && !large_vpos_jump_at_column_top
            && !large_separator_block
            && matches!(
                (prev_en_bottom_vpos, this_first_offset, this_bottom_offset),
                (Some(prev), Some(_), Some(bottom))
                    if hwpunit_to_px((bottom - prev).max(0), self.dpi) > h4f + 100.0
            );
        let large_between_tail_render_overflows = self.judge_large_between_tail_render_overflows(
            st,
            paragraphs,
            &fmt,
            en_para,
            this_first_offset,
            split_endnote_to_fit,
            available,
            ep_idx,
            default_between_notes_gap,
            compact_endnote_separator_profile,
            has_visible_endnote_separator,
            local_vpos_rewind,
            internal_vpos_rewind,
            large_separator_block,
            visible_compact_sequential_tail_fits_current_column,
        );
        let large_between_tail_before_rewind_picture = self
            .judge_large_between_tail_before_rewind_picture(
                &fmt,
                st,
                en_para,
                en_ctrl,
                this_first_offset,
                endnote_start,
                available,
                ep_idx,
                default_between_notes_gap,
                compact_endnote_separator_profile,
                has_visible_endnote_separator,
                local_vpos_rewind,
                internal_vpos_rewind,
                large_separator_block,
            );
        let table_only_endnote_para_before_rewind = en_para.text.is_empty()
            && en_para
                .controls
                .iter()
                .any(|ctrl| matches!(ctrl, Control::Table(_)))
            && !en_para
                .controls
                .iter()
                .any(|ctrl| matches!(ctrl, Control::Equation(_)));
        let no_separator_tail_table_starts_next_column = self
            .judge_no_separator_tail_table_starts_next_column(
                table_only_endnote_para_before_rewind,
                st,
                en_ctrl,
                available,
                endnote_start,
                ep_idx,
                has_visible_endnote_separator,
                internal_vpos_rewind,
                large_separator_block,
                local_vpos_rewind,
                this_first_offset,
            );
        let no_separator_last_column_tail_before_rewind_starts_next_page = self
            .judge_no_separator_last_column_tail_before_rewind_starts_next_page(
                &fmt,
                st,
                en_para,
                en_ctrl,
                this_first_offset,
                endnote_start,
                available,
                ep_idx,
                has_visible_endnote_separator,
                local_vpos_rewind,
                internal_vpos_rewind,
                large_separator_block,
            );
        let no_separator_tail_after_picture_starts_next_page = self
            .judge_no_separator_tail_after_picture_starts_next_page(
                st,
                &fmt,
                en_para,
                paragraphs,
                available,
                ep_idx,
                has_visible_endnote_separator,
                local_vpos_rewind,
                internal_vpos_rewind,
                large_separator_block,
            );
        let later_endnote_vpos_rewinds_after_current = this_first_offset.is_some_and(|cur| {
            en_ctrl.paragraphs.iter().skip(ep_idx + 1).any(|next_para| {
                next_para
                    .line_segs
                    .first()
                    .map(|seg| seg.vertical_pos + endnote_start < cur)
                    .unwrap_or(false)
            })
        });
        let large_between_small_equation_tail_bleeds_previous_column = !default_between_notes_gap
            && compact_endnote_separator_profile
            && has_visible_endnote_separator
            && ep_idx == 1
            && en_ctrl.paragraphs.len().saturating_sub(ep_idx) >= 5
            && st.col_count > 1
            && st.current_column + 1 < st.col_count
            && st.current_height > available * 0.90
            && !st.current_items.is_empty()
            && !local_vpos_rewind
            && !internal_vpos_rewind
            && later_endnote_vpos_rewinds_after_current
            && fmt.line_heights.len() == 1
            && fmt.line_advance(0) <= 36.0
            && st.current_height + fmt.line_advance(0)
                <= available + ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX + 80.0
            && line_is_equation_tac_text_run_only(en_para, &composed, 0)
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
                    && next_fmt.line_advance(0) <= 24.0
                    && line_has_visible_text(&next_comp, 0)
                    && !para_has_treat_as_char_picture_or_shape(next_para)
                    && !para_has_non_tac_picture_or_shape(next_para)
            });
        let large_between_equation_tail_starts_next_column = self
            .judge_large_between_equation_tail_starts_next_column(
                st,
                &fmt,
                en_para,
                en_ctrl,
                styles,
                &composed,
                available,
                compact_endnote_separator_profile,
                default_between_notes_gap,
                en_col_w,
                ep_idx,
                has_visible_endnote_separator,
                internal_vpos_rewind,
                large_between_small_equation_tail_bleeds_previous_column,
                local_vpos_rewind,
            );
        let large_between_title_tail_render_overflows = if !default_between_notes_gap
            && ep_idx == 0
            && st.current_column + 1 >= st.col_count
            && en_ref.number > 0
            && fmt.line_heights.len() == 1
            && !st.current_items.is_empty()
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
                    let title_h = fmt.line_advance(0);
                    // 한컴은 큰 미주 사이 문서에서도 문항 제목 한 줄만
                    // 단 하단에 남는 tail을 허용한다. 다음 본문 첫 줄까지
                    // 같은 단에 넣을 수 없다는 이유만으로 제목을 새 쪽으로
                    // 밀면 2024-09 미주사이20 p13 문18처럼 한컴보다 한 쪽
                    // 늦어진다. 제목 자체가 frame을 넘는 경우만 advance한다.
                    y + title_h > available + ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX
                })
                .unwrap_or(false)
        } else {
            false
        };
        let large_between_question_title_render_y = if !default_between_notes_gap
            && ep_idx == 0
            && en_ref.number > 0
            && fmt.line_heights.len() == 1
            && st.current_height < available
            && st.current_height > available * 0.80
            && !st.current_items.is_empty()
        {
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
        let large_between_question_title_head_inside_frame = large_between_question_title_render_y
            .map(|predicted_y| {
                predicted_y + fmt.line_advance(0)
                    <= available + ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX
            })
            .unwrap_or(false);
        let large_between_question_title_head_fits_flow = !default_between_notes_gap
            && compact_endnote_separator_profile
            && has_visible_endnote_separator
            && ep_idx == 0
            && en_ref.number > 0
            && st.current_column + 1 < st.col_count
            && fmt.line_heights.len() == 1
            && st.current_height < available
            && st.current_height + fmt.line_advance(0)
                <= available + ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX + 2.0;
        let large_between_question_title_render_head_outside =
            large_between_question_title_render_y
                .map(|predicted_y| {
                    predicted_y + fmt.line_advance(0)
                        > available + ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX
                })
                .unwrap_or(false);
        let large_between_question_lead_group_render_outside = !default_between_notes_gap
            && compact_endnote_separator_profile
            && has_visible_endnote_separator
            && ep_idx == 0
            && en_ref.number > 0
            && !endnote_has_vpos_rewind
            && st.current_column + 1 < st.col_count
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
                        .take(4)
                        .flat_map(|p| p.line_segs.iter())
                        .map(|seg| {
                            seg.vertical_pos + seg.line_height + seg.line_spacing + endnote_start
                        })
                        .max();
                    group_first
                        .zip(group_bottom)
                        .map(|(first, bottom)| {
                            let group_h = hwpunit_to_px((bottom - first).max(0), self.dpi);
                            predicted_y + group_h
                                > available + ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX
                        })
                        .unwrap_or(false)
                })
                .unwrap_or(false);
        let large_between_last_column_visual_split = self
            .judge_large_between_last_column_visual_split(
                st,
                &fmt,
                en_para,
                paragraphs,
                styles,
                available,
                en_col_w,
                en_fit,
                en_para_idx,
                ep_idx,
                default_between_notes_gap,
                compact_endnote_separator_profile,
                has_visible_endnote_separator,
                visible_large_between_notes_gap,
                compact_between_notes_gap,
                zero_endnote_spacing_profile,
                local_vpos_rewind,
                internal_vpos_rewind,
            );
        let large_between_last_column_flow_tail_split = self
            .judge_large_between_last_column_flow_tail_split(
                st,
                &fmt,
                en_para,
                available,
                compact_between_notes_gap,
                compact_endnote_separator_profile,
                default_between_notes_gap,
                en_fit,
                ep_idx,
                has_visible_endnote_separator,
                internal_vpos_rewind,
                local_vpos_rewind,
                visible_large_between_notes_gap,
                zero_endnote_spacing_profile,
            );
        RewindTailResult {
            new_endnote_stale_forward_vpos,
            large_between_tail_render_overflows,
            large_between_tail_before_rewind_picture,
            no_separator_tail_table_starts_next_column,
            no_separator_last_column_tail_before_rewind_starts_next_page,
            no_separator_tail_after_picture_starts_next_page,
            later_endnote_vpos_rewinds_after_current,
            large_between_equation_tail_starts_next_column,
            large_between_title_tail_render_overflows,
            large_between_question_title_render_y,
            large_between_question_title_head_inside_frame,
            large_between_question_title_head_fits_flow,
            large_between_question_title_render_head_outside,
            large_between_question_lead_group_render_outside,
            large_between_last_column_visual_split,
            large_between_last_column_flow_tail_split,
        }
    }
}

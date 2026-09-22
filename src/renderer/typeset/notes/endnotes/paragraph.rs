//! 미주 내부 문단 흐름 조정. 준비·조회·상태 반영의 기존 순서를 보존한다.

use crate::renderer::typeset::notes::endnotes::content::prepend_endnote_marker_text;
use crate::renderer::typeset::notes::endnotes::debug::debug_print_endnote_line_segments;
use crate::renderer::typeset::notes::endnotes::profile::{
    en_ssot_debug, en_ssot_level, endnote_between_notes_margin,
    endnote_has_absorbed_between_notes_gap, endnote_has_visible_separator,
    endnote_separator_below_margin, EnSsotLevel, EndnoteFlowProfile,
    ENDNOTE_BETWEEN_NOTES_BASE_FLOW_HU,
};
use crate::renderer::typeset::notes::endnotes::types::{
    EnMetricsVars, EndnoteEmitVars, EndnoteFlowState,
};
use crate::renderer::typeset::{
    activate_square_picture_wrap_for_para, hwpunit_to_px, line_has_tac_equation_control,
    line_has_text_span, line_has_visible_text, line_leading_tac_equation_count,
    line_tac_picture_or_shape_height, maybe_register_square_picture_wrap_anchor,
    non_tac_picture_or_shape_block_height_px, non_tac_picture_or_shape_content_height_px,
    page_item_para_index, para_has_non_tac_picture_or_shape,
    para_has_treat_as_char_picture_or_shape, para_has_visible_text,
    para_has_visible_text_or_equation, para_is_treat_as_char_picture_only,
    paragraph_by_global_index, ComposedParagraph, EndnoteParaSource, EndnoteRef, FootnoteShape,
    MeasuredTable, PageDef, PageItem, Paragraph, ResolvedStyleSet, TypesetEngine, TypesetState,
    ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX,
};

mod initial_fit;
mod new_note_fit;
mod rewind_tail;
mod tail_fit;

impl TypesetEngine {
    /// [#2026 추출] 한 미주(en_ctrl)의 문단들을 조판·방출하는 en_para 루프 본체 —
    /// #1904 라운드 1 이연분. 미주-간 흐름 캐리는 `EndnoteFlowState` 값 왕복
    /// (컬렉션 pre_emitted 는 함수 로컬로 흡수). 원본 무변경 이동.
    #[allow(clippy::too_many_arguments)]
    pub(in crate::renderer::typeset) fn typeset_endnote_paragraphs(
        &self,
        st: &mut TypesetState,
        paragraphs: &[Paragraph],
        composed: &[ComposedParagraph],
        styles: &ResolvedStyleSet,
        section_index: usize,
        page_def: &PageDef,
        measured_tables: &[MeasuredTable],
        endnote_shape: Option<&FootnoteShape>,
        endnote_flow_profile: Option<EndnoteFlowProfile>,
        endnote_refs: &[EndnoteRef],
        en_ref_idx: usize,
        en_ref: &EndnoteRef,
        en_ctrl: &crate::model::footnote::Endnote,
        vars: EndnoteEmitVars,
        flow: EndnoteFlowState,
    ) -> EndnoteFlowState {
        let EndnoteEmitVars {
            boundary_prev_endnote_had_vpos_rewind,
            endnote_has_vpos_rewind,
            continued_endnote_tail_before_new_note,
            default_nonzero_between_note_tail_candidate,
            default_question_group_title_tail,
            compact_endnote_separator_profile,
            prev_endnote_had_inline_object_vpos_overestimate,
            endnote_start,
        } = vars;
        let EndnoteFlowState {
            mut vpos_offset,
            mut prev_en_bottom_vpos,
            mut prev_en_content_bottom_vpos,
            mut emitted_endnote_count,
            mut last_render_endnote_para_local_idx,
            mut cleared_single_line_internal_rewind_split,
            mut current_endnote_had_inline_object_vpos_overestimate,
        } = flow;
        let mut pre_emitted_endnote_para_indices = std::collections::HashSet::new();
        for (ep_idx, en_para) in en_ctrl.paragraphs.iter().enumerate() {
            if pre_emitted_endnote_para_indices.remove(&ep_idx) {
                emitted_endnote_count += 1;
                continue;
            }
            let mut compact_no_separator_para;
            let render_en_para = en_para;
            let compact_no_separator_spacing = endnote_shape
                .map(|shape| {
                    !endnote_has_visible_separator(shape)
                        && endnote_between_notes_margin(shape) == 0
                })
                .unwrap_or(false);
            let en_para = if compact_no_separator_spacing {
                compact_no_separator_para = en_para.clone();
                let divisor = 10;
                for line_seg in &mut compact_no_separator_para.line_segs {
                    line_seg.line_spacing = -(line_seg.line_height / divisor);
                }
                &compact_no_separator_para
            } else {
                en_para
            };
            let en_para_idx = paragraphs.len() + st.endnote_paragraphs.len();
            let mut en_para_copy = render_en_para.clone();
            if compact_no_separator_spacing {
                let divisor = 10;
                for line_seg in &mut en_para_copy.line_segs {
                    line_seg.line_spacing = -(line_seg.line_height / divisor);
                }
            }
            // line_segs vpos를 endnote 시작점 기준으로 오프셋
            for ls in &mut en_para_copy.line_segs {
                ls.vertical_pos += endnote_start;
            }
            // 첫 paragraph에 미주 번호 prepend
            if ep_idx == 0 {
                prepend_endnote_marker_text(&mut en_para_copy, en_ctrl);
            }
            let prev_render_endnote_para_local_idx = last_render_endnote_para_local_idx;
            let prev_rendered_endnote_is_title = prev_render_endnote_para_local_idx
                .and_then(|idx| st.endnote_paragraphs.get(idx))
                .map(|p| p.text.trim_start().starts_with('문'))
                .unwrap_or(false);
            let en_para_local_idx = st.endnote_paragraphs.len();
            st.append_endnote_paragraph(en_para_copy);
            st.append_endnote_source(EndnoteParaSource {
                section_index: en_ref.section_index,
                para_index: en_ref.para_index,
                control_index: en_ref.control_index,
                note_para_index: ep_idx,
            });
            last_render_endnote_para_local_idx = Some(en_para_local_idx);

            let composed = crate::renderer::composer::compose_paragraph_in_context(en_para, styles);
            let en_col_w = st
                .layout
                .column_areas
                .get(st.current_column as usize)
                .map(|a| a.width)
                .unwrap_or(st.layout.body_area.width);
            let fmt =
                self.format_endnote_paragraph(en_para, Some(&composed), &styles, Some(en_col_w));
            if std::env::var("RHWP_ENDNOTE_LINE_DEBUG").is_ok() {
                debug_print_endnote_line_segments(
                    en_ref.number,
                    ep_idx,
                    en_para,
                    &composed,
                    &fmt,
                    self.dpi,
                    endnote_start,
                );
            }
            if compact_endnote_separator_profile
                && st.col_count > 1
                && st.current_items.is_empty()
                && st.current_height < -0.5
                && ep_idx == 0
                && !para_is_treat_as_char_picture_only(en_para)
            {
                st.align_flow_to(0.0);
                st.record_column_flow_origin(0.0);
                st.reset_vpos_cursor();
                prev_en_bottom_vpos = None;
                prev_en_content_bottom_vpos = None;
            }
            let available = st.available_height();
            // [Task #1082] 다단 미주 누적/판정을 렌더 vpos 정규화와 정합.
            // 렌더는 미주를 px(vpos − 단 첫아이템 vpos)에 배치하므로 단 used
            // = px(마지막 bottom_vpos − 첫 first_vpos). 종전(#1062)은 미주 para
            // 내부 span(자체 높이)만 더해 미주 간 vpos 간격(빈줄/문단간격)을
            // 누락 → 단 과충전 → 렌더 overflow(3-09/10/11월 교육·실전).
            // 본 정합: 직전 배치 아이템 bottom 기준 vpos delta(px)로 누적.
            // 시드 prev_en_bottom_vpos = body→endnote 전환 시 본문 last bottom
            // (위 prev_body_bottom_vpos), 단 advance 후엔 None(자체 높이).
            // #1062 안전 floor(fmt.height_for_fit) 유지 — vpos delta 가
            // formatter 추정보다 작은 케이스 회귀 차단. 단단은 종전.
            let this_first_offset = en_para
                .line_segs
                .first()
                .map(|s| s.vertical_pos + endnote_start);
            let endnote_bottom_with_spacing = en_para
                .line_segs
                .iter()
                .map(|s| {
                    (
                        s.vertical_pos
                            .saturating_add(s.line_height)
                            .saturating_add(s.line_spacing)
                            + endnote_start,
                        s.line_spacing,
                    )
                })
                .max_by_key(|(bottom, _)| *bottom);
            let this_bottom_offset = endnote_bottom_with_spacing.map(|(bottom, _)| bottom);
            let this_content_bottom_offset = en_para
                .line_segs
                .iter()
                .map(|s| s.vertical_pos.saturating_add(s.line_height) + endnote_start)
                .max();
            // 다음 미주 묶음의 시작점도 렌더상 가장 낮은 줄 기준으로 갱신한다.
            // 마지막 LINE_SEG가 위쪽으로 되감기는 문단에서는 last 기준이
            // 다음 미주를 현재 쪽에 과도하게 붙인다.
            if let Some(tb) = this_bottom_offset {
                if tb > vpos_offset {
                    vpos_offset = tb;
                }
            }
            let trailing_ls_px = endnote_bottom_with_spacing
                .map(|(_, spacing)| hwpunit_to_px(spacing.max(0), self.dpi))
                .unwrap_or(0.0);
            let default_between_notes_gap_before_rewind = endnote_flow_profile
                .map(EndnoteFlowProfile::default_between_notes)
                .unwrap_or(false);
            let absorbed_between_notes_gap_before_rewind = endnote_flow_profile
                .map(|profile| profile.absorbed_between_notes_gap)
                .unwrap_or(false);
            let large_between_notes_gap_before_rewind = endnote_flow_profile
                .map(EndnoteFlowProfile::large_between_notes)
                .unwrap_or(false);
            let zero_endnote_spacing_profile_before_rewind = endnote_flow_profile
                .map(EndnoteFlowProfile::zero_spacing)
                .unwrap_or(false);
            let current_default_late_question_title = default_between_notes_gap_before_rewind
                && default_nonzero_between_note_tail_candidate
                && ep_idx == 0
                && st.current_column + 1 >= st.col_count;
            let has_visible_endnote_separator_before_rewind = endnote_flow_profile
                .map(|profile| profile.visible_separator)
                .unwrap_or(false);
            let large_separator_block_before_rewind = endnote_flow_profile
                .map(EndnoteFlowProfile::large_between_notes)
                .unwrap_or(false);
            // 같은 미주 안에서도 LINE_SEG vpos 가 되감기며 다음 단 시작을
            // 표시하는 문서가 있다. 특히 3-09월_교육_통합_2022.hwp 9쪽의
            // 문5) 풀이처럼 단 하단에서 다음 paragraph first_vpos 가 직전
            // bottom 보다 작아지는 경우, 한컴은 같은 단에 겹쳐 쌓지 않고
            // 다음 단으로 넘긴다.
            let local_rewind_advance_threshold = if absorbed_between_notes_gap_before_rewind {
                0.65
            } else if large_between_notes_gap_before_rewind {
                0.80
            } else {
                0.85
            };
            let zero_visible_local_rewind_equation_line_tail_fits =
                compact_endnote_separator_profile
                    && zero_endnote_spacing_profile_before_rewind
                    && has_visible_endnote_separator_before_rewind
                    && st.col_count > 1
                    && ep_idx > 0
                    && fmt.line_heights.len() == 1
                    && !para_is_treat_as_char_picture_only(en_para)
                    && line_has_tac_equation_control(en_para, &composed, 0)
                    && st.current_height + fmt.line_advance(0)
                        <= available + ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX;
            let zero_visible_local_rewind_text_run_para_fits = compact_endnote_separator_profile
                && zero_endnote_spacing_profile_before_rewind
                && has_visible_endnote_separator_before_rewind
                && ep_idx > 0
                && !para_is_treat_as_char_picture_only(en_para)
                && this_first_offset.is_some_and(|first| first <= endnote_start)
                && line_has_text_span(&composed, 0)
                && line_leading_tac_equation_count(en_para, &composed, 0) >= 2
                && st.current_height + fmt.height_for_fit
                    <= available + ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX;
            let zero_visible_last_column_local_rewind_text_fits = compact_endnote_separator_profile
                && zero_endnote_spacing_profile_before_rewind
                && has_visible_endnote_separator_before_rewind
                && st.current_column + 1 >= st.col_count
                && ep_idx > 0
                && !para_is_treat_as_char_picture_only(en_para)
                && para_has_visible_text_or_equation(en_para)
                && st.current_height + fmt.total_height
                    <= available + ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX + 2.0
                && matches!(
                    (prev_en_bottom_vpos, this_first_offset),
                    (Some(prev), Some(first)) if first < prev
                );
            let zero_between_visible_local_rewind_para_fits_current_column =
                compact_endnote_separator_profile
                    && has_visible_endnote_separator_before_rewind
                    && endnote_flow_profile
                        .map(|profile| {
                            profile.between_notes_hu == 0 && profile.large_separator_margin()
                        })
                        .unwrap_or(false)
                    && ep_idx > 0
                    && !para_is_treat_as_char_picture_only(en_para)
                    && para_has_visible_text_or_equation(en_para)
                    && st.current_height > available * 0.80
                    && st.current_height + fmt.total_height
                        <= available + ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX + 4.0
                    && matches!(
                        (prev_en_bottom_vpos, this_first_offset),
                        (Some(prev), Some(first)) if first < prev
                    );
            let no_separator_zero_local_rewind_para_fits_current_column =
                compact_endnote_separator_profile
                    && !has_visible_endnote_separator_before_rewind
                    && endnote_shape
                        .map(|shape| endnote_between_notes_margin(shape) == 0)
                        .unwrap_or(false)
                    && st.current_column + 1 < st.col_count
                    && (ep_idx > 0 || emitted_endnote_count > 0)
                    && !para_is_treat_as_char_picture_only(en_para)
                    && para_has_visible_text_or_equation(en_para)
                    && st.current_height + fmt.total_height
                        <= available + ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX + 4.0
                    && matches!(
                        (prev_en_bottom_vpos, this_first_offset),
                        (Some(prev), Some(first)) if first < prev
                    );
            let no_separator_local_rewind_final_tail_fits_current_column =
                compact_endnote_separator_profile
                    && large_separator_block_before_rewind
                    && !has_visible_endnote_separator_before_rewind
                    && st.current_column + 1 < st.col_count
                    && ep_idx > 0
                    && ep_idx + 2 >= en_ctrl.paragraphs.len()
                    && st.current_height > available * 0.90
                    && matches!(
                        (prev_en_bottom_vpos, this_first_offset),
                        (Some(prev), Some(first)) if first < prev
                    )
                    && {
                        let remaining_tail: f64 = en_ctrl
                            .paragraphs
                            .iter()
                            .skip(ep_idx)
                            .map(|tail_para| {
                                let tail_comp =
                                    crate::renderer::composer::compose_paragraph_in_context(
                                        tail_para, styles,
                                    );
                                self.format_endnote_paragraph(
                                    tail_para,
                                    Some(&tail_comp),
                                    &styles,
                                    Some(en_col_w),
                                )
                                .total_height
                            })
                            .sum();
                        // 구분선 없는 미주 끝의 짧은 rewind tail은 같은 단
                        // 하단에 남고, 다음 미주 제목부터 새 단으로 넘어간다.
                        // 이 tail까지 밀면 한컴/PDF보다 오른쪽 단이 늦게 시작한다.
                        st.current_height + remaining_tail
                            <= available + ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX + 4.0
                    };
            let zero_between_visible_local_rewind_final_tail_fits_current_column =
                compact_endnote_separator_profile
                    && has_visible_endnote_separator_before_rewind
                    && endnote_flow_profile
                        .map(|profile| {
                            profile.between_notes_hu == 0 && profile.large_separator_margin()
                        })
                        .unwrap_or(false)
                    && st.current_column + 1 < st.col_count
                    && ep_idx > 0
                    && ep_idx + 1 >= en_ctrl.paragraphs.len()
                    && fmt.line_heights.len() == 1
                    && !para_is_treat_as_char_picture_only(en_para)
                    && para_has_visible_text_or_equation(en_para)
                    && st.current_height > available * 0.90
                    && matches!(
                        (prev_en_bottom_vpos, this_first_offset),
                        (Some(prev), Some(first)) if first < prev
                    )
                    && {
                        let remaining_tail: f64 = en_ctrl
                            .paragraphs
                            .iter()
                            .skip(ep_idx)
                            .map(|tail_para| {
                                let tail_comp =
                                    crate::renderer::composer::compose_paragraph_in_context(
                                        tail_para, styles,
                                    );
                                self.format_endnote_paragraph(
                                    tail_para,
                                    Some(&tail_comp),
                                    &styles,
                                    Some(en_col_w),
                                )
                                .total_height
                            })
                            .sum();
                        // 미주 사이 0에서는 마지막 rewind tail과 다음 번호 제목
                        // 사이에 추가 미주 gap을 만들지 않는다. tail 자체가
                        // frame 안에 들어가면 현재 단 하단에 남긴다.
                        st.current_height + remaining_tail
                            <= available + ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX + 4.0
                    };
            let visible_separator_title_body_rewind_starts_next_column =
                compact_endnote_separator_profile
                    && has_visible_endnote_separator_before_rewind
                    && default_between_notes_gap_before_rewind
                    && endnote_flow_profile
                        .map(EndnoteFlowProfile::large_separator_margin)
                        .unwrap_or(false)
                    && st.col_count > 1
                    && st.current_column + 1 < st.col_count
                    && ep_idx == 1
                    && en_ref.number > 0
                    && st.current_height > available * 0.80
                    && en_ctrl
                        .paragraphs
                        .first()
                        .is_some_and(|title_para| title_para.line_segs.len() == 1)
                    && para_has_visible_text_or_equation(en_para)
                    && matches!(
                        (prev_en_bottom_vpos, this_first_offset),
                        (Some(prev), Some(first)) if first < prev
                    );
            if st.col_count > 1
                && !st.current_items.is_empty()
                && (st.current_height > available * local_rewind_advance_threshold
                    || visible_separator_title_body_rewind_starts_next_column)
                && !current_default_late_question_title
                && !zero_visible_local_rewind_equation_line_tail_fits
                && !zero_visible_local_rewind_text_run_para_fits
                && !zero_visible_last_column_local_rewind_text_fits
                && !zero_between_visible_local_rewind_para_fits_current_column
                && !no_separator_zero_local_rewind_para_fits_current_column
                && !no_separator_local_rewind_final_tail_fits_current_column
                && !zero_between_visible_local_rewind_final_tail_fits_current_column
                && matches!(
                    (prev_en_bottom_vpos, this_first_offset),
                    (Some(prev), Some(first)) if first < prev
                )
            {
                st.advance_column_or_new_page();
                prev_en_bottom_vpos = None;
                prev_en_content_bottom_vpos = None;
            }
            let local_vpos_rewind = matches!(
                (prev_en_bottom_vpos, this_first_offset),
                (Some(prev), Some(first)) if first < prev
            );
            let has_visible_endnote_separator = has_visible_endnote_separator_before_rewind;
            // 보이는 구분선 + 큰 미주 사이에서는 renderer가 이전 content floor를
            // 넘는 되감김을 순차 y로 유지한다. pagination도 같은 조건에서
            // TAC 그림 되감김 축약을 피해야 단 하단 overflow가 줄어든다.
            let local_vpos_rewind_crosses_prev_content = large_between_notes_gap_before_rewind
                && has_visible_endnote_separator
                && st.current_height > available * 0.225
                && matches!(
                    (prev_en_content_bottom_vpos, this_first_offset),
                    (Some(prev_content), Some(first)) if first < prev_content
                );
            let large_vpos_jump_at_column_top = st.col_count > 1
                && st.current_height < available * 0.20
                && matches!(
                    (prev_en_bottom_vpos, this_first_offset),
                    (Some(prev), Some(first))
                        if first > prev
                            && hwpunit_to_px(first - prev, self.dpi)
                                > available * 0.75
                );
            let internal_rewind_position = en_para
                .line_segs
                .windows(2)
                .position(|w| w[1].vertical_pos < w[0].vertical_pos)
                .map(|idx| idx + 1)
                .filter(|split| {
                    *split > 0
                        && *split < en_para.line_segs.len()
                        && *split < fmt.line_heights.len()
                });
            let internal_vpos_rewind = internal_rewind_position.is_some();
            let saved_page_reset_rewind = internal_rewind_position
                .and_then(|split| en_para.line_segs.get(split).map(|seg| (split, seg)))
                .map(|(split, seg)| {
                    split >= 4 && seg.vertical_pos <= 0 && st.current_height > available * 0.65
                })
                .unwrap_or(false);
            let large_separator_block = endnote_flow_profile
                .map(EndnoteFlowProfile::large_between_notes)
                .unwrap_or(false);
            let zero_between_large_separator_margin = endnote_flow_profile
                .map(EndnoteFlowProfile::visible_zero_between_large_separator_margin)
                .unwrap_or(false);
            // [#4318] 구분선 위/아래 20mm + 기본 미주 사이(7mm). 다른 미주
            // 모양까지 4px bleed·꼬리 넘김을 쓰면 쪽수/off-canvas 가 흔들린다.
            let both_large_separator_default_between = endnote_flow_profile
                .map(EndnoteFlowProfile::visible_both_large_separator_default_between)
                .unwrap_or(false);
            let endnote_has_text_or_equation = para_has_visible_text_or_equation(en_para);
            let endnote_has_visible_payload =
                endnote_has_text_or_equation || para_has_non_tac_picture_or_shape(en_para);
            let mut internal_rewind_split = if compact_endnote_separator_profile
                && st.col_count > 1
                && (st.current_height > available * 0.75 || saved_page_reset_rewind)
                && endnote_has_visible_payload
            {
                internal_rewind_position
            } else {
                None
            };
            let move_internal_rewind_equation_to_next = compact_endnote_separator_profile
                && internal_vpos_rewind
                && internal_rewind_split.is_none()
                && st.col_count > 1
                && st.current_height > available * 0.75
                && endnote_has_visible_payload;

            let col_count = st.col_count;
            let dpi = self.dpi;
            let h4f = fmt.height_for_fit;
            let tot = fmt.total_height;
            let default_between_notes_gap = endnote_flow_profile
                .map(EndnoteFlowProfile::default_between_notes)
                .unwrap_or(false);
            let zero_endnote_spacing_profile = endnote_flow_profile
                .map(EndnoteFlowProfile::zero_spacing)
                .unwrap_or(false);
            let compact_between_notes_gap = endnote_flow_profile
                .map(EndnoteFlowProfile::default_or_compact_between_notes)
                .unwrap_or(false);
            let absorbed_between_notes_gap = endnote_flow_profile
                .map(|profile| profile.absorbed_between_notes_gap)
                .unwrap_or(false);
            let visible_non_default_compact_between_gap = endnote_flow_profile
                .map(EndnoteFlowProfile::visible_non_default_compact_between_notes)
                .unwrap_or(false);
            let visible_large_between_notes_gap = endnote_flow_profile
                .map(EndnoteFlowProfile::visible_large_between_notes)
                .unwrap_or(false);
            let no_separator_large_between_notes_gap = endnote_flow_profile
                .map(EndnoteFlowProfile::no_separator_large_between_notes)
                .unwrap_or(false);
            let visible_zero_between_large_separator_gap = endnote_flow_profile
                .map(EndnoteFlowProfile::visible_zero_between_large_separator_margin)
                .unwrap_or(false);
            let visible_large_between_zero_above_compact_below = endnote_flow_profile
                .map(EndnoteFlowProfile::visible_large_between_zero_above_compact_below)
                .unwrap_or(false);
            // 기본 미주 사이 7mm의 번호 미주 tail은 단 하단에서도 제목 뒤
            // 풀이 본문 일부가 같은 쪽에 이어지는 경우가 있다. 20mm처럼
            // 커진 "미주 사이"는 별도 큰 gap 정책을 타야 한다.
            let allow_default_late_question_tail = default_between_notes_gap
                && !zero_endnote_spacing_profile
                && default_nonzero_between_note_tail_candidate
                && st.current_column + 1 >= st.col_count
                && (has_visible_endnote_separator
                    || (fmt.line_heights.len() > 1 && fmt.total_height <= 32.0));
            let suppress_late_question_gap_for_fit = allow_default_late_question_tail
                && st.current_column + 1 >= st.col_count
                && st.current_height > available * 0.90;
            let large_rewind_equation_tail_new_note_gap_absorbed = ep_idx == 0
                && emitted_endnote_count > 0
                && endnote_flow_profile
                    .map(EndnoteFlowProfile::visible_large_between_notes)
                    .unwrap_or(false)
                && boundary_prev_endnote_had_vpos_rewind
                && continued_endnote_tail_before_new_note
                && st.current_height < available * 0.35
                && prev_render_endnote_para_local_idx
                    .and_then(|idx| st.endnote_paragraphs.get(idx))
                    .map(|prev_para| {
                        !para_has_visible_text(prev_para)
                            && para_has_visible_text_or_equation(prev_para)
                    })
                    .unwrap_or(false);
            let new_endnote_between_notes_px = if ep_idx == 0
                && emitted_endnote_count > 0
                && compact_endnote_separator_profile
                && !suppress_late_question_gap_for_fit
                && !large_rewind_equation_tail_new_note_gap_absorbed
            {
                endnote_shape.map(|shape| {
                    let gap = endnote_between_notes_margin(shape) as i32;
                    let default_visible_tail_absorbed_gap = default_between_notes_gap
                        && has_visible_endnote_separator
                        && boundary_prev_endnote_had_vpos_rewind
                        && st.current_column + 1 >= st.col_count
                        && st.current_height > available * 0.25
                        && st.current_height < available * 0.50;
                    let effective_gap = if default_visible_tail_absorbed_gap {
                        0
                    } else {
                        gap
                    };
                    hwpunit_to_px(effective_gap, dpi)
                })
            } else {
                None
            };
            let same_endnote_body_first_line_advance = if ep_idx == 0
                && no_separator_large_between_notes_gap
            {
                en_ctrl.paragraphs.get(1).map(|body_para| {
                    let body_comp =
                        crate::renderer::composer::compose_paragraph_in_context(body_para, styles);
                    let body_fmt = self.format_endnote_paragraph(
                        body_para,
                        Some(&body_comp),
                        &styles,
                        Some(en_col_w),
                    );
                    body_fmt.line_advance(0)
                })
            } else {
                None
            };
            let no_separator_new_note_head_fits_current_column =
                no_separator_large_between_notes_gap
                    && ep_idx == 0
                    && emitted_endnote_count > 0
                    && new_endnote_between_notes_px
                        .map(|gap| {
                            st.current_height + fmt.line_advance(0) + gap
                                <= available + ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX
                        })
                        .unwrap_or(false);
            let no_separator_last_column_new_note_head_without_gap_fits =
                no_separator_large_between_notes_gap
                    && ep_idx == 0
                    && emitted_endnote_count > 0
                    && st.current_column + 1 >= st.col_count
                    && st.current_height > available * 0.80
                    && fmt.line_heights.len() <= 2
                    && fmt.total_height <= 32.0
                    && same_endnote_body_first_line_advance
                        .map(|body_head| {
                            // 구분선이 없는 마지막 단에서는 직전 미주의
                            // 마지막 line spacing이 이미 다음 번호와의
                            // 시각 gap을 갖는 경우가 있다. 한컴은 이 gap을
                            // 새 번호 앞에 다시 예약하지 않고, 제목과 첫 본문
                            // 줄이 들어가면 현재 쪽 하단 tail로 남긴다.
                            st.current_height + fmt.line_advance(0) + body_head
                                <= available + ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX + 8.0
                        })
                        .unwrap_or(false);
            let visible_separator_new_note_title_tail_fits_for_a2 =
                compact_endnote_separator_profile
                    && visible_non_default_compact_between_gap
                    && ep_idx == 0
                    && emitted_endnote_count > 0
                    && en_ref.number > 0
                    && fmt.line_heights.len() == 1
                    && !local_vpos_rewind
                    && !internal_vpos_rewind
                    && st.current_column + 1 < st.col_count
                    && st.current_height > available * 0.88
                    && st.current_height < available
                    && new_endnote_between_notes_px
                        .map(|gap| {
                            st.current_height + fmt.line_advance(0) + gap
                                <= available + ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX + 2.0
                        })
                        .unwrap_or_else(|| {
                            st.current_height + fmt.line_advance(0)
                                <= available + ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX + 2.0
                        })
                    && endnote_has_visible_payload;
            let min_vpos_rewind_height = en_para
                .line_segs
                .first()
                .map(|s| hwpunit_to_px(s.line_height.max(1), dpi))
                .unwrap_or(h4f);
            let has_treat_as_char_picture_shape = para_has_treat_as_char_picture_or_shape(en_para);
            let tac_picture_only_height = if para_is_treat_as_char_picture_only(en_para) {
                en_para
                    .controls
                    .iter()
                    .filter_map(|ctrl| crate::renderer::tac_object_flow_height_px(ctrl, dpi))
                    .reduce(f64::max)
            } else {
                None
            };
            let tac_picture_tail_height = (0..fmt.line_heights.len())
                .filter(|line_idx| !line_has_visible_text(&composed, *line_idx))
                .filter_map(|line_idx| {
                    line_tac_picture_or_shape_height(en_para, &composed, line_idx, dpi)
                })
                .chain(tac_picture_only_height)
                .reduce(f64::max);
            let tac_picture_tail_group_height =
                if let (Some(first), Some(pic_h)) = (this_first_offset, tac_picture_tail_height) {
                    let tail_bottom = en_ctrl
                        .paragraphs
                        .iter()
                        .skip(ep_idx + 1)
                        .flat_map(|p| p.line_segs.iter())
                        .map(|s| {
                            s.vertical_pos
                                .saturating_add(s.line_height)
                                .saturating_add(s.line_spacing)
                                + endnote_start
                        })
                        .max();
                    Some(
                        tail_bottom
                            .map(|bottom| hwpunit_to_px((bottom - first).max(0), dpi))
                            .unwrap_or(0.0)
                            .max(pic_h),
                    )
                } else {
                    None
                };
            let cap_large_separator_stale_forward_vpos = large_separator_block
                && compact_between_notes_gap
                && st.current_height < available * 0.70;
            let current_height_for_metrics = st.current_height;
            let current_column_has_tac_picture_only = st
                .current_items
                .iter()
                .filter_map(page_item_para_index)
                .any(|pi| {
                    paragraph_by_global_index(paragraphs, &st.endnote_paragraphs, pi)
                        .map(para_is_treat_as_char_picture_only)
                        .unwrap_or(false)
                });
            // [Task #1363] SSOT: layout 이 순차 format 으로 렌더하는 점유 높이.
            // Divergence A(내부 vpos rewind) 이전의 ground truth.
            let line_advances_sum = fmt.line_advances_sum(0..fmt.line_heights.len());
            let ssot_level = en_ssot_level();
            let ssot_debug = en_ssot_debug();

            let non_tac_object_height = if endnote_has_text_or_equation {
                None
            } else {
                non_tac_picture_or_shape_block_height_px(en_para, dpi)
            };
            let endnote_boundary_gap_extra_px = endnote_shape
                .filter(|shape| {
                    let between_notes = endnote_between_notes_margin(shape) as i32;
                    compact_endnote_separator_profile
                        && ep_idx + 1 == en_ctrl.paragraphs.len()
                        && endnote_refs.get(en_ref_idx + 1).is_some()
                        && between_notes > ENDNOTE_BETWEEN_NOTES_BASE_FLOW_HU
                        && !endnote_has_absorbed_between_notes_gap(shape)
                })
                .map(|shape| {
                    let between_notes = endnote_between_notes_margin(shape) as i32;
                    let saved_spacing = en_para
                        .line_segs
                        .last()
                        .map(|seg| seg.line_spacing.max(0))
                        .unwrap_or(0);
                    hwpunit_to_px((between_notes - saved_spacing).max(0), self.dpi)
                })
                .unwrap_or(0.0);
            let (raw_en_fit, _) = self.compute_endnote_metrics(
                prev_en_bottom_vpos,
                false,
                &mut current_endnote_had_inline_object_vpos_overestimate,
                EnMetricsVars {
                    available,
                    cap_large_separator_stale_forward_vpos,
                    col_count,
                    compact_endnote_separator_profile,
                    current_column_has_tac_picture_only,
                    current_height_for_metrics,
                    dpi,
                    en_para_idx,
                    h4f,
                    has_treat_as_char_picture_shape,
                    has_visible_endnote_separator,
                    internal_vpos_rewind,
                    large_separator_block,
                    large_vpos_jump_at_column_top,
                    line_advances_sum,
                    local_vpos_rewind,
                    local_vpos_rewind_crosses_prev_content,
                    min_vpos_rewind_height,
                    new_endnote_between_notes_px,
                    no_separator_new_note_head_fits_current_column,
                    ssot_debug,
                    ssot_level,
                    this_bottom_offset,
                    this_first_offset,
                    tot,
                    trailing_ls_px,
                },
            );
            let en_fit = non_tac_object_height
                .map(|height| raw_en_fit.max(height))
                .unwrap_or(raw_en_fit);
            let total_advance_fit = line_advances_sum.max(non_tac_object_height.unwrap_or(0.0));
            let initial_fit::InitialFitResult {
                remaining_height,
                a2_overflow_with_para,
                page_offcanvas_with_para,
                no_separator_visible_multiline_tail_fits_with_bleed,
                next_endnote_title_fit_height,
                next_endnote_first_para_fit_height,
                no_separator_saved_vpos_tail_outside,
                visible_separator_saved_vpos_tail_outside,
                compact_endnote_own_vpos_span_fits_for_flow,
                no_separator_compact_final_note_first_line_tail_fits,
                mut split_endnote_to_fit,
                late_internal_rewind_fit_split,
                compact_non_default_empty_column_rewind_fits,
                visible_compact_sequential_tail_fits_current_column,
            } = self.query_initial_fit(initial_fit::InitialFitInput {
                st,
                paragraphs,
                styles,
                endnote_refs,
                en_ref_idx,
                en_ref,
                en_ctrl,
                ep_idx,
                en_para,
                fmt: &fmt,
                composed: &composed,
                emitted_endnote_count,
                en_para_idx,
                this_first_offset,
                this_content_bottom_offset,
                non_tac_object_height,
                ssot_level,
                col_count,
                available,
                en_col_w,
                dpi,
                line_advances_sum,
                total_advance_fit,
                en_fit,
                compact_endnote_separator_profile,
                large_between_notes_gap_before_rewind,
                local_vpos_rewind,
                has_visible_endnote_separator,
                internal_vpos_rewind,
                large_separator_block,
                zero_between_large_separator_margin,
                both_large_separator_default_between,
                endnote_has_visible_payload,
                default_between_notes_gap,
                zero_endnote_spacing_profile,
                compact_between_notes_gap,
                allow_default_late_question_tail,
                has_treat_as_char_picture_shape,
            });
            if ssot_level >= EnSsotLevel::A2
                && a2_overflow_with_para == Some(true)
                && split_endnote_to_fit.is_none()
                && !visible_compact_sequential_tail_fits_current_column
                && !visible_separator_new_note_title_tail_fits_for_a2
                && !st.current_items.is_empty()
                && st.current_height > available * 0.5
                && !local_vpos_rewind
                && !internal_vpos_rewind
            {
                st.advance_column_or_new_page();
                prev_en_bottom_vpos = None;
            }
            // [#5886] HWPX 문단-사이 되감김 이후 순차 적층이 용지 밖이면
            // 단/쪽을 넘긴다. 같은 문서 .hwp 는 이미 넘긴다. 분할 가능한
            // 다줄(1375)과 문항 경계(1139)는 HWP 경로라 그대로.
            if page_offcanvas_with_para
                && !st.current_items.is_empty()
                && split_endnote_to_fit.is_none()
                && !internal_vpos_rewind
            {
                st.advance_column_or_new_page();
                prev_en_bottom_vpos = None;
                prev_en_content_bottom_vpos = None;
            }
            let large_between_split_head_render_overflows = self
                .judge_large_between_split_head_render_overflows(
                    st,
                    &fmt,
                    en_para,
                    paragraphs,
                    styles,
                    split_endnote_to_fit,
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
            if large_between_split_head_render_overflows {
                // pagination 기준으로는 split head가 들어가도, 저장 vpos를 적용한
                // 실제 render 위치가 frame을 넘으면 한컴처럼 문단 전체를 다음 단에서
                // 시작시킨다.
                split_endnote_to_fit = None;
            }
            let internal_rewind_head_fits_current_column = internal_rewind_split
                .map(|split| {
                    let head_h = fmt.line_advances_sum(0..split);
                    head_h <= remaining_height + ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX + 1.0
                })
                .unwrap_or(false);
            let single_line_internal_rewind_head_overflows_frame = internal_rewind_split == Some(1)
                && !default_between_notes_gap
                && ep_idx > 0
                && fmt.line_heights.len() > 1
                && st.current_height + fmt.line_advances_sum(0..1) > available + 1.0
                && endnote_has_visible_payload;
            let internal_rewind_head_allows_current_column =
                internal_rewind_head_fits_current_column
                    && !single_line_internal_rewind_head_overflows_frame;
            let internal_rewind_target_is_reset = internal_rewind_split
                .and_then(|split| en_para.line_segs.get(split))
                .map(|seg| seg.vertical_pos == 0)
                .unwrap_or(false);
            let preserve_reset_internal_rewind_split = internal_rewind_split == Some(1)
                && !default_between_notes_gap
                && has_visible_endnote_separator
                && st.current_column + 1 < st.col_count
                && st.current_height > available * 0.75
                && internal_rewind_target_is_reset
                && internal_rewind_head_allows_current_column
                && endnote_has_visible_payload;
            let internal_rewind_head_overflows_current_column = zero_endnote_spacing_profile
                && internal_rewind_split.is_some()
                && !internal_rewind_head_allows_current_column
                && st.current_height >= available;
            let preserve_single_line_internal_rewind_split = internal_rewind_split == Some(1)
                && !default_between_notes_gap
                && st.current_column + 1 < st.col_count
                && fmt.line_heights.len() > 1
                && internal_rewind_head_allows_current_column
                && (st.current_height + total_advance_fit
                    > available + ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX
                    || preserve_reset_internal_rewind_split)
                && endnote_has_visible_payload;
            let preserve_no_separator_last_column_single_line_rewind = internal_rewind_split
                == Some(1)
                && large_separator_block
                && !has_visible_endnote_separator
                && !default_between_notes_gap
                && ep_idx == 1
                && st.current_column + 1 >= st.col_count
                && fmt.line_heights.len() > 1
                && internal_rewind_head_allows_current_column
                && st.current_height + fmt.line_advances_sum(0..1)
                    <= available + ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX + 8.0
                && st
                    .current_items
                    .last()
                    .and_then(page_item_para_index)
                    .is_some_and(|prev_pi| prev_pi + 1 == en_para_idx)
                && endnote_has_visible_payload;
            let large_between_single_line_internal_rewind = internal_rewind_split == Some(1)
                && !default_between_notes_gap
                && endnote_has_visible_payload;
            let advance_large_between_single_line_rewind = large_between_single_line_internal_rewind
                && !preserve_no_separator_last_column_single_line_rewind
                && st.current_column + 1 >= st.col_count
                && st.current_height > available * 0.80
                && !st.current_items.is_empty();
            if advance_large_between_single_line_rewind {
                // 큰 `미주 사이` 문서의 마지막 단 하단에서 첫 줄부터
                // vpos가 되감기는 문단은 한컴/PDF처럼 다음 쪽에서 통째로
                // 시작해야 한다. 현재 쪽에 FullParagraph로 남기면 첫 줄이
                // frame 밖에 그려지고, 다음 쪽 문항 흐름이 한 줄만큼 당겨진다.
                st.advance_column_or_new_page();
                prev_en_bottom_vpos = None;
                internal_rewind_split = None;
            } else if large_between_single_line_internal_rewind
                && !preserve_single_line_internal_rewind_split
                && !preserve_no_separator_last_column_single_line_rewind
            {
                internal_rewind_split = None;
                cleared_single_line_internal_rewind_split = true;
            }
            let internal_reset_split_head_render_overflows = internal_rewind_split
                .filter(|split| *split > 1)
                .filter(|_| {
                    !default_between_notes_gap
                        && compact_endnote_separator_profile
                        && has_visible_endnote_separator
                        && internal_rewind_target_is_reset
                        && st.col_count > 1
                        && st.current_column + 1 >= st.col_count
                        && !st.current_items.is_empty()
                        && endnote_has_visible_payload
                })
                .and_then(|split| {
                    self.predict_current_column_para_y(
                        &st,
                        en_para_idx,
                        paragraphs,
                        &styles,
                        measured_tables,
                        Some(en_col_w),
                    )
                    .map(|render_y| {
                        render_y + fmt.line_advances_sum(0..split)
                            > available + ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX + 1.0
                    })
                })
                .unwrap_or(false);
            let internal_rewind_full_advance_needed = self
                .judge_internal_rewind_full_advance_needed(
                    internal_rewind_target_is_reset,
                    internal_rewind_split,
                    st,
                    en_para,
                    available,
                    compact_endnote_separator_profile,
                    default_between_notes_gap,
                    en_fit,
                    endnote_has_visible_payload,
                    has_visible_endnote_separator,
                    internal_vpos_rewind,
                    split_endnote_to_fit,
                    total_advance_fit,
                );
            if default_between_notes_gap
                && compact_endnote_separator_profile
                && compact_between_notes_gap
                && !has_visible_endnote_separator
                && internal_rewind_split.is_some()
                && ep_idx == 0
                && st.current_column + 1 >= st.col_count
                && st.current_height + total_advance_fit
                    <= available + ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX
                && endnote_has_visible_payload
            {
                // 비가시 구분선의 기본 미주에서는 저장 vpos rewind가
                // 있더라도 현재 단에 전체 문단이 실제 흐름 높이로 들어가면
                // 한컴처럼 문단을 쪼개지 않는다. 여기서 split하면 뒤따르는
                // 미주들이 한 단 늦게 밀릴 수 있다.
                internal_rewind_split = None;
            }
            let rewind_tail::RewindTailResult {
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
            } = self.query_rewind_tail(rewind_tail::RewindTailInput {
                st,
                paragraphs,
                styles,
                measured_tables,
                en_ref,
                en_ctrl,
                ep_idx,
                en_para,
                fmt: &fmt,
                composed: &composed,
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
            });
            if large_between_title_tail_render_overflows
                && !no_separator_last_column_new_note_head_without_gap_fits
            {
                st.advance_column_or_new_page();
                prev_en_bottom_vpos = None;
            }
            if large_between_tail_render_overflows
                || large_between_tail_before_rewind_picture
                || large_between_equation_tail_starts_next_column
                || no_separator_tail_table_starts_next_column
                || no_separator_last_column_tail_before_rewind_starts_next_page
            {
                st.advance_column_or_new_page();
                prev_en_bottom_vpos = None;
            }
            let next_endnote_first_line_advance = if ep_idx == 0 {
                en_ctrl.paragraphs.get(1).map(|next_para| {
                    let next_comp =
                        crate::renderer::composer::compose_paragraph_in_context(next_para, styles);
                    self.format_endnote_paragraph(
                        next_para,
                        Some(&next_comp),
                        &styles,
                        Some(en_col_w),
                    )
                    .line_advance(0)
                })
            } else {
                None
            };
            let next_endnote_head_pair_advance = if ep_idx == 0 {
                let mut total = 0.0;
                let mut count = 0;
                for next_para in en_ctrl.paragraphs.iter().skip(1).take(2) {
                    let next_comp =
                        crate::renderer::composer::compose_paragraph_in_context(next_para, styles);
                    let next_fmt = self.format_endnote_paragraph(
                        next_para,
                        Some(&next_comp),
                        &styles,
                        Some(en_col_w),
                    );
                    total += next_fmt.line_advance(0);
                    count += 1;
                }
                (count == 2).then_some(total)
            } else {
                None
            };
            let zero_between_large_separator_last_column_title_orphan =
                compact_endnote_separator_profile
                    && has_visible_endnote_separator
                    && endnote_shape
                        .map(|shape| {
                            endnote_between_notes_margin(shape) == 0
                                && shape.separator_above_margin_hu() as i32
                                    > ENDNOTE_BETWEEN_NOTES_BASE_FLOW_HU
                                && endnote_separator_below_margin(shape) as i32
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
                    && next_endnote_first_line_advance
                        .map(|next_h| {
                            st.current_height + fmt.line_advance(0) + next_h
                                > available + ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX + 2.0
                        })
                        .unwrap_or(false)
                    && endnote_has_visible_payload;
            let default_large_below_last_column_title_orphan = self
                .judge_default_large_below_last_column_title_orphan(
                    &fmt,
                    st,
                    en_ctrl,
                    en_ref,
                    endnote_shape,
                    available,
                    ep_idx,
                    next_endnote_first_line_advance,
                    next_endnote_head_pair_advance,
                    default_between_notes_gap,
                    compact_endnote_separator_profile,
                    has_visible_endnote_separator,
                    endnote_has_visible_payload,
                );
            let default_compact_below_last_column_title_orphan = self
                .judge_default_compact_below_last_column_title_orphan(
                    &fmt,
                    st,
                    en_ctrl,
                    en_ref,
                    endnote_shape,
                    available,
                    ep_idx,
                    next_endnote_first_line_advance,
                    next_endnote_head_pair_advance,
                    default_between_notes_gap,
                    compact_endnote_separator_profile,
                    has_visible_endnote_separator,
                    endnote_has_visible_payload,
                );
            if zero_between_large_separator_last_column_title_orphan
                || default_large_below_last_column_title_orphan
                || default_compact_below_last_column_title_orphan
            {
                st.advance_column_or_new_page();
                prev_en_bottom_vpos = None;
                prev_en_content_bottom_vpos = None;
            }
            let tail_fit::TailFitResult {
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
            } = self.query_tail_fit(tail_fit::TailFitInput {
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
                fmt: &fmt,
                composed: &composed,
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
            });
            if pre_emit_tail_before_non_tac_object_advance {
                if let Some(next_para) = en_ctrl.paragraphs.get(ep_idx + 1) {
                    let next_para_idx = paragraphs.len() + st.endnote_paragraphs.len();
                    let mut next_para_copy = next_para.clone();
                    for ls in &mut next_para_copy.line_segs {
                        ls.vertical_pos += endnote_start;
                    }
                    st.append_endnote_paragraph(next_para_copy);
                    st.append_endnote_source(EndnoteParaSource {
                        section_index: en_ref.section_index,
                        para_index: en_ref.para_index,
                        control_index: en_ref.control_index,
                        note_para_index: ep_idx + 1,
                    });
                    last_render_endnote_para_local_idx = Some(st.endnote_paragraphs.len() - 1);

                    let next_comp =
                        crate::renderer::composer::compose_paragraph_in_context(next_para, styles);
                    let next_fmt = self.format_endnote_paragraph(
                        next_para,
                        Some(&next_comp),
                        &styles,
                        Some(en_col_w),
                    );
                    st.append_item(PageItem::FullParagraph {
                        para_index: next_para_idx,
                    });
                    st.advance_flow_by(next_fmt.total_height);
                    st.mark_endnote_flow();
                    pre_emitted_endnote_para_indices.insert(ep_idx + 1);
                }
            }
            if advance_for_fit {
                st.advance_column_or_new_page();
                prev_en_bottom_vpos = None;
                prev_en_content_bottom_vpos = None;
                if internal_rewind_split == Some(1) {
                    internal_rewind_split = None;
                    cleared_single_line_internal_rewind_split = true;
                } else if absorbed_between_notes_gap && internal_vpos_rewind {
                    // 이전 단 하단에서 계산한 내부 rewind split은
                    // 새 단으로 advance한 뒤에는 더 이상 유효하지 않다.
                    // 그대로 들고 가면 빈 단에서 문단을 다시 쪼개
                    // 한컴보다 미주 흐름이 한 쪽 늦어진다.
                    internal_rewind_split = None;
                } else if internal_rewind_head_overflows_current_column {
                    // 현재 단에 split 머리도 들어가지 않는 internal rewind는
                    // 새 단/쪽에서 다시 전체 높이로 배치한다.
                    internal_rewind_split = None;
                } else if internal_reset_split_head_render_overflows {
                    // 저장 lineSeg reset은 실제 column/page split 신호지만,
                    // 현재 단의 render-y 기준으로 reset 앞 head가 이미 frame을
                    // 넘으면 현재 단 tail로 남기지 않고 다음 단/쪽에서 다시 본다.
                    internal_rewind_split = None;
                } else if internal_rewind_full_advance_needed {
                    // saved-vpos 압축 높이만 현재 단에 들어가는 기본 미주 rewind는
                    // head tail로 쪼개지 않고 다음 단에서 전체 문단으로 시작한다.
                    internal_rewind_split = None;
                }
            }
            let new_note_fit::NewNoteFitResult {
                advance_for_new_endnote,
                advance_for_internal_rewind,
            } = self.query_new_note_fit(new_note_fit::NewNoteFitInput {
                st,
                paragraphs,
                styles,
                endnote_shape,
                en_ref,
                en_ctrl,
                ep_idx,
                en_para,
                fmt: &fmt,
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
            });
            if advance_for_new_endnote {
                st.advance_column_or_new_page();
                prev_en_bottom_vpos = None;
                prev_en_content_bottom_vpos = None;
            }
            if advance_for_internal_rewind {
                st.advance_column_or_new_page();
                prev_en_bottom_vpos = None;
                prev_en_content_bottom_vpos = None;
            }
            let no_separator_default_tail_render_pullup_hu = if !advance_for_fit
                && !advance_for_new_endnote
                && !advance_for_internal_rewind
                && allow_default_column_bottom_question_title_tail
                && !has_visible_endnote_separator
                && st.current_column + 1 >= st.col_count
                && st.current_height > available * 0.92
                && fmt.line_heights.len() == 1
                && para_has_visible_text_or_equation(en_para)
            {
                Some(900)
            } else {
                None
            };
            if let Some(pullup_hu) = no_separator_default_tail_render_pullup_hu {
                // 비가시 구분선 기본 미주의 마지막 단 한 줄 tail 묶음은
                // 저장 vpos가 실제 frame 하단보다 한 줄가량 아래를 가리킬
                // 수 있다. Pagination은 tail을 현재 단에 남기되, 렌더 vpos만
                // 위로 당겨 127~129 같은 연속 번호가 frame 안에 보이게 한다.
                st.shift_endnote_render_lines(en_para_local_idx, -pullup_hu);
            }
            // 구분선 아래가 큰 기본 미주에서 제목 tail만 현재 단 하단에
            // 남는 경우, 저장 vpos가 한 기본 미주 gap만큼 위로 당겨질 수
            // 있다. 렌더 좌표만 보정하고 pagination 흐름은 유지한다.
            let default_large_below_rewind_title_tail_gap_hu = if !advance_for_new_endnote
                && !advance_for_internal_rewind
                && compact_endnote_separator_profile
                && default_between_notes_gap
                && has_visible_endnote_separator
                && endnote_has_vpos_rewind
                && ep_idx == 0
                && emitted_endnote_count > 0
                && en_ref.number > 0
                && st.current_column + 1 < st.col_count
                && st.current_height > available * 0.85
                && fmt.line_heights.len() == 1
                && endnote_shape
                    .map(|shape| {
                        endnote_separator_below_margin(shape) as i32
                            > ENDNOTE_BETWEEN_NOTES_BASE_FLOW_HU
                    })
                    .unwrap_or(false)
            {
                endnote_shape
                    .map(|shape| endnote_between_notes_margin(shape) as i32)
                    .filter(|gap_hu| {
                        st.current_height + hwpunit_to_px(*gap_hu, self.dpi) + en_fit
                            <= available + ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX + 2.0
                    })
            } else {
                None
            };
            if let Some(gap_hu) = default_large_below_rewind_title_tail_gap_hu {
                st.shift_endnote_render_lines(en_para_local_idx, gap_hu);
            }
            let tac_picture_rewinds_before_column_base = st.col_count > 1
                && compact_between_notes_gap
                && local_vpos_rewind
                && para_is_treat_as_char_picture_only(en_para)
                && st.current_column + 1 >= st.col_count
                && st.current_height
                    + tac_picture_tail_group_height
                        .or(tac_picture_only_height)
                        .unwrap_or(en_fit)
                    > available - ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX
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
                        this_first_offset.map(|first_vpos| first_vpos < base_vpos)
                    })
                    .unwrap_or(false);
            if tac_picture_rewinds_before_column_base {
                // 저장 vpos가 현재 단 시작보다 앞선 TAC 그림은 한컴에서
                // 하단 겹침으로 남기지 않고 다음 단/쪽에서 자체 높이를 소비한다.
                st.advance_column_or_new_page();
                prev_en_bottom_vpos = None;
                prev_en_content_bottom_vpos = None;
            }
            let tac_picture_rewind_height = if st.col_count > 1
                && local_vpos_rewind
                && !local_vpos_rewind_crosses_prev_content
                && para_is_treat_as_char_picture_only(en_para)
            {
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
                            hwpunit_to_px((first_vpos - base_vpos).max(0), self.dpi)
                        })
                    })
            } else {
                None
            };
            maybe_register_square_picture_wrap_anchor(
                &mut *st,
                paragraphs,
                en_para,
                en_para_idx,
                page_def,
            );
            // advance 후 재평가 — 새 단 첫 미주는 prev=None → 자체 높이.
            let (_, mut en_advance) = self.compute_endnote_metrics(
                prev_en_bottom_vpos,
                true,
                &mut current_endnote_had_inline_object_vpos_overestimate,
                EnMetricsVars {
                    available,
                    cap_large_separator_stale_forward_vpos,
                    col_count,
                    compact_endnote_separator_profile,
                    current_column_has_tac_picture_only,
                    current_height_for_metrics,
                    dpi,
                    en_para_idx,
                    h4f,
                    has_treat_as_char_picture_shape,
                    has_visible_endnote_separator,
                    internal_vpos_rewind,
                    large_separator_block,
                    large_vpos_jump_at_column_top,
                    line_advances_sum,
                    local_vpos_rewind,
                    local_vpos_rewind_crosses_prev_content,
                    min_vpos_rewind_height,
                    new_endnote_between_notes_px,
                    no_separator_new_note_head_fits_current_column,
                    ssot_debug,
                    ssot_level,
                    this_bottom_offset,
                    this_first_offset,
                    tot,
                    trailing_ls_px,
                },
            );
            if large_between_last_column_question_title_tail_fits
                || large_between_last_column_render_title_tail_fits
                || large_between_last_column_rewind_title_tail_fits
            {
                // 큰 미주 사이가 있는 마지막 단에서 새 미주 제목만
                // frame 안쪽 tail로 남길 때는 제목-본문 vpos 간격을
                // 현재 단 높이로 소비하지 않는다. 그 간격까지 소비하면
                // 같은 미주의 첫 본문 줄 split 기회를 잃고 다음 쪽으로
                // 통째로 넘어가 한컴보다 한 쪽 늦어진다.
                en_advance = en_advance.min(fmt.total_height);
            }
            if no_separator_last_column_new_note_head_without_gap_fits {
                // 구분선 없는 마지막 단에서는 저장 vpos에 남은 큰 미주 사이가
                // 직전 미주의 하단 여백으로 이미 보인다. 제목 advance까지 그
                // gap을 다시 소비하면 같은 미주의 첫 본문 줄이 한컴보다 다음
                // 쪽으로 밀리므로, 제목 자체 높이만 pagination에 반영한다.
                en_advance = en_advance.min(fmt.total_height);
            }
            if large_between_zero_above_whole_note_small_bleed_fits {
                // 구분선 위 0 + 큰 미주 사이에서는 새 문항 전체 vpos span이
                // 단 하단을 소폭 넘더라도 한컴은 제목을 현재 단에 남긴 뒤
                // 같은 미주의 본문을 순차적으로 이어 배치한다. 제목 emit에서
                // 전체 span을 한 번에 소비하면 본문이 다음 쪽으로 밀린다.
                en_advance = en_advance.min(fmt.total_height);
            }
            if zero_endnote_spacing_profile {
                if let Some(object_height) = non_tac_object_height {
                    // 0/0/0 미주에서는 구분선 주변 여백이 전혀 없어 비TAC
                    // 그림/도형 문단의 실제 객체 높이를 다음 미주 시작 위치에
                    // 반영해야 renderer와 pagination의 하단 기준이 맞는다.
                    en_advance = en_advance.max(object_height);
                }
            }
            if pre_emit_tail_before_non_tac_object_advance
                && non_tac_object_height.is_some()
                && !endnote_has_text_or_equation
            {
                if let Some(object_content_height) =
                    non_tac_picture_or_shape_content_height_px(en_para, dpi)
                {
                    // 미주 사이 0의 단 하단에서 뒤 텍스트 tail을 앞 단에
                    // 선배치한 경우, 한컴은 다음 단의 비TAC 그림 뒤 margin을
                    // 별도 빈 줄처럼 소비하지 않는다.
                    en_advance = object_content_height;
                }
            }
            if (advance_for_fit || advance_for_internal_rewind)
                && !default_between_notes_gap
                && compact_between_notes_gap
                && has_visible_endnote_separator
                && internal_vpos_rewind
                && !local_vpos_rewind
                && st.current_items.is_empty()
            {
                // 단 하단에서 다음 단/쪽으로 이동된 내부 rewind 미주는
                // 이동 전 하단 cur 기준의 축약 높이를 재사용하면 다음 미주가
                // renderer보다 위에서 시작해 하단 overflow가 난다. 새 단에서는
                // 문단 전체 line advance와 저장된 미주 사이 gap을 소비한다.
                let boundary_gap = endnote_shape
                    .map(endnote_between_notes_margin)
                    .map(|gap| hwpunit_to_px(gap as i32, dpi))
                    .unwrap_or(0.0);
                en_advance = en_advance.max(total_advance_fit + boundary_gap);
            }
            let compact_visible_last_column_non_reset_rewind_tail =
                compact_endnote_separator_profile
                    && compact_between_notes_gap
                    && has_visible_endnote_separator
                    && st.current_column + 1 >= st.col_count
                    && !internal_rewind_target_is_reset
                    && !late_internal_rewind_fit_split
                    && internal_rewind_split.is_some_and(|split| split > 1)
                    && split_endnote_to_fit.is_none();
            let mut split_endnote_emitted = false;
            let tall_line_internal_rewind_split = internal_rewind_split.filter(|split| {
                !compact_visible_last_column_non_reset_rewind_tail
                    && !late_internal_rewind_fit_split
                    && split
                        .checked_sub(1)
                        .and_then(|idx| en_para.line_segs.get(idx))
                        .map(|seg| seg.line_height >= 2000)
                        .unwrap_or(false)
            });
            let prioritized_internal_rewind_split = internal_rewind_split.filter(|split| {
                // 첫 줄 직후 되감기는 한컴 저장본에서 같은 단 fit 분할과 함께
                // 나타나는 경우가 있어, 기존 fit 후보가 있으면 그 분배를 유지한다.
                !compact_visible_last_column_non_reset_rewind_tail
                    && (!late_internal_rewind_fit_split
                        // late fit 후보가 단일 tail 제거 규칙으로 사라져도,
                        // lineSeg가 실제 0으로 reset되는 내부 분할은 HWP의
                        // column/page split 신호이므로 보존한다.
                        || (internal_rewind_target_is_reset
                            && *split > 1
                            && split_endnote_to_fit.is_none()))
                    && (*split > 1 || split_endnote_to_fit.is_none())
            });
            let suppress_empty_column_rewind_split = internal_rewind_position.is_some()
                && st.current_height < 5.0
                && st.current_height + total_advance_fit
                    <= available + ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX;
            let suppress_empty_column_fit_split = split_endnote_to_fit.is_some()
                && compact_endnote_separator_profile
                && !has_visible_endnote_separator
                && st.current_items.is_empty()
                && st.current_height < 5.0
                && !local_vpos_rewind
                && !internal_vpos_rewind
                && st.current_height + total_advance_fit
                    <= available + ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX;
            let split_candidate = if compact_non_default_empty_column_rewind_fits
                || suppress_empty_column_rewind_split
                || suppress_empty_column_fit_split
            {
                None
            } else {
                tall_line_internal_rewind_split
                    .or(prioritized_internal_rewind_split)
                    .or(large_between_last_column_visual_split)
                    .or(large_between_last_column_flow_tail_split)
                    .or(split_endnote_to_fit)
            };
            // [#4318] 구분선 20/20+기본 미주사이 마지막 단: LINE_SEG 마지막
            // 줄 vpos=0 reset 은 그 줄만 넘긴다. reset 앞 head 가 저장 vpos
            // 기준으로 단 하단을 넘기면 들어가는 줄까지 줄여 넘긴다.
            let split_candidate = if both_large_separator_default_between
                && compact_endnote_separator_profile
                && has_visible_endnote_separator
                && st.current_column + 1 >= st.col_count
                && saved_page_reset_rewind
            {
                match (
                    split_candidate,
                    self.predict_current_column_para_y(
                        &st,
                        en_para_idx,
                        paragraphs,
                        &styles,
                        measured_tables,
                        Some(en_col_w),
                    ),
                ) {
                    (Some(mut split), Some(render_y)) if split >= 2 => {
                        while split >= 2 && render_y + fmt.line_advances_sum(0..split) > available {
                            split -= 1;
                        }
                        Some(split)
                    }
                    (other, _) => other,
                }
            } else {
                split_candidate
            };
            if self.emit_endnote_split(
                st,
                &fmt,
                en_para,
                paragraphs,
                styles,
                split_candidate,
                en_para_idx,
                en_advance,
                en_col_w,
                available,
                col_count,
                compact_between_notes_gap,
                compact_endnote_separator_profile,
                endnote_has_text_or_equation,
                has_visible_endnote_separator,
                large_separator_block,
                line_advances_sum,
                non_tac_object_height,
                pre_emit_tail_before_non_tac_object_advance,
                ssot_debug,
                ssot_level,
                tac_picture_rewind_height,
            ) {
                split_endnote_emitted = true;
            }
            activate_square_picture_wrap_for_para(&mut *st, en_para_idx, en_para);
            // 다음 미주의 base 가 될 본 미주 bottom 기록.
            if split_endnote_emitted {
                prev_en_bottom_vpos = None;
                prev_en_content_bottom_vpos = None;
            } else if let Some(tb) = this_bottom_offset {
                prev_en_bottom_vpos = Some(tb);
                prev_en_content_bottom_vpos = this_content_bottom_offset.or(this_bottom_offset);
            }
            if local_vpos_rewind {
                st.mark_compact_endnote_rewind();
            }
        }
        EndnoteFlowState {
            vpos_offset,
            prev_en_bottom_vpos,
            prev_en_content_bottom_vpos,
            emitted_endnote_count,
            last_render_endnote_para_local_idx,
            cleared_single_line_internal_rewind_split,
            current_endnote_had_inline_object_vpos_overestimate,
        }
    }
}

//! 미주 내부 문단 흐름 조정. 준비·조회·상태 반영의 기존 순서를 보존한다.

use crate::renderer::typeset::notes::endnotes::content::prepend_endnote_marker_text;
use crate::renderer::typeset::notes::endnotes::profile::EnSsotLevel;
use crate::renderer::typeset::{
    endnote_last_column_tail_overflows_frame, hwpunit_to_px, line_has_visible_text_or_tac_equation,
    page_item_para_index, para_has_non_tac_picture_or_shape, para_has_visible_text_or_equation,
    para_is_treat_as_char_picture_only, paragraph_by_global_index, ComposedParagraph, Control,
    EndnoteRef, FormattedParagraph, Paragraph, ResolvedStyleSet, TypesetEngine, TypesetState,
    ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX, ENDNOTE_LAST_COLUMN_SPLIT_BLEED_PX,
    ENDNOTE_PAGE_OFFCANVAS_GUARD_PX,
};

/// 호출 시점의 관측값. 페이지 상태를 변경할 수 없는 Query 입력이다.
pub(super) struct InitialFitInput<'a> {
    pub(super) st: &'a TypesetState,
    pub(super) paragraphs: &'a [Paragraph],
    pub(super) styles: &'a ResolvedStyleSet,
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
    pub(super) this_first_offset: Option<i32>,
    pub(super) this_content_bottom_offset: Option<i32>,
    pub(super) non_tac_object_height: Option<f64>,
    pub(super) ssot_level: EnSsotLevel,
    pub(super) col_count: u16,
    pub(super) available: f64,
    pub(super) en_col_w: f64,
    pub(super) dpi: f64,
    pub(super) line_advances_sum: f64,
    pub(super) total_advance_fit: f64,
    pub(super) en_fit: f64,
    pub(super) compact_endnote_separator_profile: bool,
    pub(super) large_between_notes_gap_before_rewind: bool,
    pub(super) local_vpos_rewind: bool,
    pub(super) has_visible_endnote_separator: bool,
    pub(super) internal_vpos_rewind: bool,
    pub(super) large_separator_block: bool,
    pub(super) zero_between_large_separator_margin: bool,
    pub(super) both_large_separator_default_between: bool,
    pub(super) endnote_has_visible_payload: bool,
    pub(super) default_between_notes_gap: bool,
    pub(super) zero_endnote_spacing_profile: bool,
    pub(super) compact_between_notes_gap: bool,
    pub(super) allow_default_late_question_tail: bool,
    pub(super) has_treat_as_char_picture_shape: bool,
}

/// 기존 순서로 계산한 후보. 적용은 문단 조정자가 담당한다.
pub(super) struct InitialFitResult {
    pub(super) remaining_height: f64,
    pub(super) a2_overflow_with_para: Option<bool>,
    pub(super) page_offcanvas_with_para: bool,
    pub(super) no_separator_visible_multiline_tail_fits_with_bleed: bool,
    pub(super) next_endnote_title_fit_height: Option<f64>,
    pub(super) next_endnote_first_para_fit_height: Option<f64>,
    pub(super) no_separator_saved_vpos_tail_outside: bool,
    pub(super) visible_separator_saved_vpos_tail_outside: bool,
    pub(super) compact_endnote_own_vpos_span_fits_for_flow: bool,
    pub(super) no_separator_compact_final_note_first_line_tail_fits: bool,
    pub(super) split_endnote_to_fit: Option<usize>,
    pub(super) late_internal_rewind_fit_split: bool,
    pub(super) compact_non_default_empty_column_rewind_fits: bool,
    pub(super) visible_compact_sequential_tail_fits_current_column: bool,
}

impl TypesetEngine {
    pub(super) fn query_initial_fit(&self, input: InitialFitInput<'_>) -> InitialFitResult {
        let InitialFitInput {
            st,
            paragraphs,
            styles,
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
        } = input;
        let remaining_height = (available - st.current_height).max(0.0);
        // [#3707 진단] 미주 단 전환 판정의 입력. 동작 불변.
        // 높이 누적(EN_ACC)은 두 문서가 소수점까지 같은데 단 전환만 갈리므로,
        // 남는 후보는 여기 `available`/`en_col_w`/`current_height` 다.
        if std::env::var("RHWP_DIAG_ENCOL").is_ok() {
            eprintln!(
                "DIAG_ENCOL pi={} avail={:.1} cur_h={:.1} remain={:.1} col_w={:.1} fit={:.1} adv={:.1}",
                en_para_idx,
                available,
                st.current_height,
                remaining_height,
                en_col_w,
                en_fit,
                total_advance_fit,
            );
        }
        // [Task #1363 v2 Stage 3] A2: 새 para 를 이어붙인 렌더-정합 시뮬
        // bottom 으로 fit 판정 (saved line_segs 기반 → 렌더와 일치).
        // [#5886] 기본 B 에서는 문단-사이 되감김이 용지 밖(+56px)으로
        // 나갈 때만 시뮬을 켠다. 24px A2 overflow 를 B 에 흘리면
        // split_endnote_to_fit 가 1375/1139 질문 흐름을 가른다.
        // B 시뮬은 scratch LayoutEngine(렌더와 동일 경로)으로 하단을 읽는다.
        // acc 는 올리지 않는다.
        let page_offcanvas_sim = compact_endnote_separator_profile
            && st.profile.hwpx_stored_layout()
            && st.col_count > 1
            && !st.current_items.is_empty()
            && st.current_height > available * 0.5
            && (local_vpos_rewind
                || st.column_had_compact_endnote_rewind
                // [#6495] 되감김 신호가 **없는** 단도 넘친다. 3-09월_교육_통합_2022
                // 9쪽 오른쪽 단은 되감김이 한 번도 없는데 타이프셋 누계가
                // 1048.9 로 가용 1001.6 을 47px 넘고, 그린 줄이 용지 끝
                // 841.17pt(용지 841.9)에 닿는다. 누계가 이미 가용을 넘었으면
                // 신호와 무관하게 시뮬로 확인한다.
                || st.current_height > available);
        let simulated_endnote_bottom = if ssot_level >= EnSsotLevel::A2 || page_offcanvas_sim {
            self.simulate_endnote_column_bottom_y(
                &st,
                paragraphs,
                styles,
                available,
                en_col_w,
                Some(en_para_idx),
                ssot_level < EnSsotLevel::A2,
            )
        } else {
            None
        };
        let a2_overflow_with_para = if ssot_level >= EnSsotLevel::A2 {
            simulated_endnote_bottom
                .map(|bottom| bottom > available + ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX)
        } else {
            None
        };
        // [#5886] 허용 bleed 는 **용지 안에 남는 만큼**을 넘을 수 없다.
        //
        // 관문 이름 그대로 이것은 "용지 밖(off-canvas)"을 막는 장치인데, 허용치를
        // 단 하단 기준 고정 56px 로 두면 쪽 아래 여백이 그보다 좁은 문서에서
        // **여백과 56px 사이 구간이 통째로 사각지대**가 된다. 3-09월_교육_통합_2022
        // 12쪽이 그 예로, 단 하단 1092.3 에서 용지 1122.5 까지 30.2px 뿐인데
        // 관문은 +56px 을 넘을 때까지 침묵해 세 문단(시뮬 하단 1016.6·1034.6·1052.7)
        // 이 놓인 뒤에야 발동했다. 뒤 두 문단은 용지 밖이라 다시 그려지지도 않는다.
        //
        // 여백이 56px 이상인 문서는 종전과 같다 — 좁은 쪽에서만 조인다.
        let page_bottom_room =
            (st.layout.page_height - (st.layout.body_area.y + st.layout.body_area.height)).max(0.0);
        let page_offcanvas_guard_px = ENDNOTE_PAGE_OFFCANVAS_GUARD_PX.min(page_bottom_room);
        let page_offcanvas_with_para = page_offcanvas_sim
            && simulated_endnote_bottom
                .is_some_and(|bottom| bottom > available + page_offcanvas_guard_px);
        // 구분선 없는 큰 미주 block에서는 다줄 수식 문단의 advance가
        // frame을 약간 넘더라도 실제 보이는 줄은 하단 frame 안에 남는다.
        // 이 tail을 통째로 유지해야 다음 단의 새 문항 시작점이 한컴과 맞는다.
        let no_separator_tail_extra_bleed = if st.current_column + 1 < st.col_count {
            24.0
        } else {
            0.0
        };
        let no_separator_tail_min_height_ratio = if st.current_column + 1 < st.col_count {
            0.90
        } else {
            0.84
        };
        let no_separator_visible_multiline_tail_fits_with_bleed = large_separator_block
            && !has_visible_endnote_separator
            && ep_idx > 0
            && !local_vpos_rewind
            && !internal_vpos_rewind
            && fmt.line_heights.len() > 1
            && st.current_height > available * no_separator_tail_min_height_ratio
            && para_has_visible_text_or_equation(en_para)
            && !para_has_non_tac_picture_or_shape(en_para)
            && st.current_height + total_advance_fit
                <= available
                    + ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX
                    + no_separator_tail_extra_bleed;
        let next_endnote_title_fit_height = if ep_idx + 1 == en_ctrl.paragraphs.len() {
            endnote_refs.get(en_ref_idx + 1).and_then(|next_ref| {
                let next_host = paragraphs.get(next_ref.para_index)?;
                let Control::Endnote(next_ctrl) = next_host.controls.get(next_ref.control_index)?
                else {
                    return None;
                };
                let mut next_para = next_ctrl.paragraphs.first()?.clone();
                prepend_endnote_marker_text(&mut next_para, next_ctrl);

                let next_comp =
                    crate::renderer::composer::compose_paragraph_in_context(&next_para, styles);
                let next_fmt = self.format_endnote_paragraph(
                    &next_para,
                    Some(&next_comp),
                    &styles,
                    Some(en_col_w),
                );
                (next_fmt.line_heights.len() == 1
                    && line_has_visible_text_or_tac_equation(&next_para, &next_comp, 0))
                .then_some(next_fmt.height_for_fit)
            })
        } else {
            None
        };
        let next_endnote_first_para_fit_height = if ep_idx + 1 == en_ctrl.paragraphs.len() {
            endnote_refs.get(en_ref_idx + 1).and_then(|next_ref| {
                let next_host = paragraphs.get(next_ref.para_index)?;
                let Control::Endnote(next_ctrl) = next_host.controls.get(next_ref.control_index)?
                else {
                    return None;
                };
                let mut next_para = next_ctrl.paragraphs.first()?.clone();
                prepend_endnote_marker_text(&mut next_para, next_ctrl);

                let next_comp =
                    crate::renderer::composer::compose_paragraph_in_context(&next_para, styles);
                let next_fmt = self.format_endnote_paragraph(
                    &next_para,
                    Some(&next_comp),
                    &styles,
                    Some(en_col_w),
                );
                Some(next_fmt.height_for_fit)
            })
        } else {
            None
        };
        let next_next_endnote_first_para_fit_height = if ep_idx + 1 == en_ctrl.paragraphs.len() {
            endnote_refs.get(en_ref_idx + 2).and_then(|next_ref| {
                let next_host = paragraphs.get(next_ref.para_index)?;
                let Control::Endnote(next_ctrl) = next_host.controls.get(next_ref.control_index)?
                else {
                    return None;
                };
                let mut next_para = next_ctrl.paragraphs.first()?.clone();
                prepend_endnote_marker_text(&mut next_para, next_ctrl);

                let next_comp =
                    crate::renderer::composer::compose_paragraph_in_context(&next_para, styles);
                let next_fmt = self.format_endnote_paragraph(
                    &next_para,
                    Some(&next_comp),
                    &styles,
                    Some(en_col_w),
                );
                Some(next_fmt.height_for_fit)
            })
        } else {
            None
        };
        let next_endnote_head_has_large_tac_picture = if ep_idx + 1 == en_ctrl.paragraphs.len() {
            endnote_refs
                .get(en_ref_idx + 1)
                .and_then(|next_ref| {
                    let next_host = paragraphs.get(next_ref.para_index)?;
                    let Control::Endnote(next_ctrl) =
                        next_host.controls.get(next_ref.control_index)?
                    else {
                        return None;
                    };
                    Some(next_ctrl.paragraphs.iter().take(8).any(|next_para| {
                        if !para_is_treat_as_char_picture_only(next_para) {
                            return false;
                        }
                        let next_comp = crate::renderer::composer::compose_paragraph_in_context(
                            next_para, styles,
                        );
                        let next_fmt = self.format_endnote_paragraph(
                            next_para,
                            Some(&next_comp),
                            &styles,
                            Some(en_col_w),
                        );
                        next_fmt.height_for_fit > 80.0
                    }))
                })
                .unwrap_or(false)
        } else {
            false
        };
        let compact_endnote_own_vpos_span_fits = self.judge_compact_endnote_own_vpos_span_fits(
            dpi,
            this_content_bottom_offset,
            remaining_height,
            st,
            available,
            compact_between_notes_gap,
            compact_endnote_separator_profile,
            endnote_has_visible_payload,
            internal_vpos_rewind,
            local_vpos_rewind,
            non_tac_object_height,
            this_first_offset,
        );
        let compact_endnote_body_tail_overflows_frame = compact_endnote_own_vpos_span_fits
            && ep_idx > 0
            && !local_vpos_rewind
            && !internal_vpos_rewind
            && fmt.line_heights.len() > 1
            && st.current_height + total_advance_fit > available + 1.0
            && endnote_has_visible_payload
            && ((!default_between_notes_gap && st.current_height > available * 0.95)
                || (zero_between_large_separator_margin
                    && st.current_column + 1 >= st.col_count
                    && st.current_height > available * 0.80));
        let no_separator_saved_vpos_tail_outside = large_separator_block
            && !has_visible_endnote_separator
            && ep_idx > 0
            && st.current_column + 1 >= st.col_count
            && st.current_height > available * 0.90
            && st.current_height + en_fit > available - ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX
            && !local_vpos_rewind
            && !internal_vpos_rewind
            && endnote_has_visible_payload
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
                        predicted_y + fmt.line_advance(0)
                            > available - ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX
                    })
                })
                .unwrap_or(false);
        let visible_separator_saved_vpos_tail_outside = self
            .judge_visible_separator_saved_vpos_tail_outside(
                paragraphs,
                &fmt,
                st,
                en_para,
                this_first_offset,
                total_advance_fit,
                available,
                ep_idx,
                compact_endnote_separator_profile,
                has_visible_endnote_separator,
                endnote_has_visible_payload,
                zero_endnote_spacing_profile,
                local_vpos_rewind,
                internal_vpos_rewind,
            );
        let compact_endnote_own_vpos_span_fits_for_flow = compact_endnote_own_vpos_span_fits
            && !compact_endnote_body_tail_overflows_frame
            && !visible_separator_saved_vpos_tail_outside
            && !(large_separator_block
                && ep_idx == 0
                && st.current_column + 1 >= st.col_count
                && st.current_height + en_fit > available);
        let no_separator_compact_final_note_first_line_tail_fits = compact_endnote_separator_profile
            && default_between_notes_gap
            && compact_between_notes_gap
            && !has_visible_endnote_separator
            && ep_idx == 0
            && emitted_endnote_count > 0
            && en_ref.number > 0
            && st.current_column + 1 < st.col_count
            && fmt.line_heights.len() == 2
            && !local_vpos_rewind
            && !internal_vpos_rewind
            && para_has_visible_text_or_equation(en_para)
            && next_endnote_first_para_fit_height.is_none()
            && st.current_height > available * 0.90
            && st.current_height + fmt.line_advance(0)
                <= available + ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX
            && st.current_height + en_fit > available - 50.0;
        let split_endnote_to_fit = if compact_endnote_separator_profile
            && st.col_count > 1
            && !local_vpos_rewind
            && st.current_height < available
            && !compact_endnote_own_vpos_span_fits_for_flow
            && a2_overflow_with_para.unwrap_or(
                st.current_height + en_fit > available
                    || st.current_height + total_advance_fit > available,
            )
            && fmt.line_heights.len() > 1
            && endnote_has_visible_payload
        {
            let split_remaining_height = if has_visible_endnote_separator
                && st.current_column + 1 >= st.col_count
                && (!default_between_notes_gap || zero_between_large_separator_margin)
            {
                // 보이는 구분선의 마지막 단에서는 renderer의 저장 vpos
                // 보정이 하단으로 약간 내려갈 수 있다. 미주 사이가
                // 0이어도 구분선 위/아래가 큰 프로필은 같은 방식으로
                // 마지막 visible tail 한 줄을 현재 단에 남긴다.
                remaining_height + ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX
            } else {
                remaining_height
            };
            let mut sum = 0.0;
            let mut split = 0usize;
            for line_idx in 0..fmt.line_heights.len() {
                let line_h = fmt.line_advance(line_idx);
                if sum + line_h > split_remaining_height {
                    break;
                }
                sum += line_h;
                split = line_idx + 1;
            }
            (split > 0 && split < fmt.line_heights.len()).then_some(split)
        } else {
            None
        };
        let split_endnote_to_fit = if split_endnote_to_fit.is_none()
            && !default_between_notes_gap
            && compact_endnote_separator_profile
            && has_visible_endnote_separator
            && ep_idx > 0
            && st.current_column + 1 >= st.col_count
            && fmt.line_heights.len() >= 5
            && st.current_height > available * 0.84
            && st.current_height + en_fit <= available + ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX
            && en_ctrl.paragraphs.get(ep_idx + 1).is_some_and(|next_para| {
                let next_comp =
                    crate::renderer::composer::compose_paragraph_in_context(next_para, styles);
                let next_fmt = self.format_endnote_paragraph(
                    next_para,
                    Some(&next_comp),
                    &styles,
                    Some(en_col_w),
                );
                let next_first = next_fmt.line_advance(0);
                st.current_height + en_fit + next_first
                    > available - ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX
            }) {
            // 큰 미주 사이의 마지막 단 하단에서는 저장 vpos가 현재
            // 문단 마지막 줄을 다음 단/쪽의 첫 줄로 넘기는 경우가 있다.
            // 다음 문단 첫 줄이 들어가지 않는 상황이면 현재 다줄 문단을
            // 마지막 줄 직전에 쪼개 한컴의 tail 흐름을 따른다.
            Some(fmt.line_heights.len() - 1)
        } else {
            split_endnote_to_fit
        };
        let split_endnote_to_fit = if split_endnote_to_fit.is_none()
            && compact_endnote_separator_profile
            && zero_endnote_spacing_profile
            && has_visible_endnote_separator
            && ep_idx > 0
            && st.current_column + 1 >= st.col_count
            && fmt.line_heights.len() >= 3
            && st.current_height > available * 0.90
            && !local_vpos_rewind
            && !internal_vpos_rewind
            && endnote_has_visible_payload
        {
            let tail_split = fmt.line_heights.len() - 1;
            let head_h = fmt.line_advances_sum(0..tail_split);
            let tail_h = fmt.line_advance(tail_split);
            let head_fits =
                st.current_height + head_h <= available + ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX;
            let tail_overflows = st.current_height + head_h + tail_h
                > available - ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX;
            let last_line_visible =
                line_has_visible_text_or_tac_equation(en_para, &composed, tail_split);

            // 0/0/0 미주의 마지막 단에서는 저장 vpos 보정 때문에
            // 문단 전체 sequential 높이는 들어가도 마지막 줄만 frame
            // 아래로 내려갈 수 있다. 한컴은 이 tail 한 줄을 다음 쪽
            // 첫 줄로 넘기므로 마지막 줄 직전에 분할한다.
            (head_fits && tail_overflows && last_line_visible).then_some(tail_split)
        } else {
            split_endnote_to_fit
        };
        // [#4318] 구분선 위/아래 20mm + 기본 미주 사이 마지막 단: 0/0/0
        // 가드가 없어도 마지막 줄이 본문 하단을 넘기면 그 줄만 넘긴다.
        let split_endnote_to_fit = if split_endnote_to_fit.is_none()
            && compact_endnote_separator_profile
            && both_large_separator_default_between
            && has_visible_endnote_separator
            && ep_idx > 0
            && st.current_column + 1 >= st.col_count
            && fmt.line_heights.len() >= 2
            && !local_vpos_rewind
            && !internal_vpos_rewind
            && endnote_has_visible_payload
        {
            let tail_split = fmt.line_heights.len() - 1;
            let head_h = fmt.line_advances_sum(0..tail_split);
            let tail_h = fmt.line_advance(tail_split);
            let last_line_visible =
                line_has_visible_text_or_tac_equation(en_para, &composed, tail_split);
            (st.current_height + head_h <= available + ENDNOTE_LAST_COLUMN_SPLIT_BLEED_PX
                && endnote_last_column_tail_overflows_frame(
                    st.current_height + head_h,
                    tail_h,
                    available,
                )
                && last_line_visible)
                .then_some(tail_split)
        } else {
            split_endnote_to_fit
        };
        let split_endnote_to_fit = if split_endnote_to_fit.is_none()
            && compact_endnote_separator_profile
            && default_between_notes_gap
            && compact_between_notes_gap
            && !has_visible_endnote_separator
            && ep_idx == 0
            && emitted_endnote_count > 0
            && en_ref.number > 0
            && st.current_column + 1 >= st.col_count
            && fmt.line_heights.len() == 2
            && !local_vpos_rewind
            && !internal_vpos_rewind
            && para_has_visible_text_or_equation(en_para)
            && st.current_height > available - 70.0
            && next_endnote_first_para_fit_height.is_some_and(|next_h| next_h <= 18.0)
            && next_next_endnote_first_para_fit_height.is_some_and(|next_h| next_h <= 18.0)
        {
            // 마지막 단 하단에서 2줄 풀이 뒤에 짧은 한 줄 풀이들이
            // 이어지는 비가시 구분선 미주는, 한컴처럼 2줄 풀이의 첫 줄만
            // 현재 쪽에 남기고 tail 한 줄을 다음 쪽 첫 줄로 넘긴다.
            // 다음다음 풀이가 긴 경계는 제외한다.
            Some(1)
        } else {
            split_endnote_to_fit
        };
        let split_endnote_to_fit = if split_endnote_to_fit.is_none()
            && no_separator_compact_final_note_first_line_tail_fits
        {
            // 마지막 미주의 2줄 문단은 한컴처럼 첫 줄만 첫 단
            // 하단에 남기고 나머지를 다음 단으로 이어 붙인다.
            Some(1)
        } else {
            split_endnote_to_fit
        };
        let late_internal_rewind_fit_split = compact_endnote_separator_profile
            && internal_vpos_rewind
            && !default_between_notes_gap
            && !local_vpos_rewind
            && !has_treat_as_char_picture_shape
            && st.current_height > available * 0.90
            && split_endnote_to_fit.is_some_and(|split| {
                split >= 4 || (split == 1 && st.current_height > available * 0.97)
            });
        let split_endnote_to_fit = if late_internal_rewind_fit_split {
            Some(1)
        } else {
            split_endnote_to_fit
        };
        let split_endnote_to_fit = if !default_between_notes_gap
            && (compact_between_notes_gap || large_between_notes_gap_before_rewind)
            && has_visible_endnote_separator
            && internal_vpos_rewind
            && st.current_column + 1 < st.col_count
            && st.current_height > available * 0.90
        {
            split_endnote_to_fit.map(|split| {
                // 보이는 구분선 + 비기본/대형 "미주 사이" 샘플의 하단
                // internal-rewind 문단은 renderer가 저장 vpos/gap을
                // 적용해 마지막 포함 줄을 pagination보다 낮게 그린다.
                // split 후보의 마지막 줄을 다음 단으로 보내 overflow를
                // 사전에 차단한다.
                if split > 1 && split < fmt.line_heights.len() {
                    split - 1
                } else {
                    split
                }
            })
        } else {
            split_endnote_to_fit
        };
        let mut split_endnote_to_fit = split_endnote_to_fit.filter(|split| {
            let single_line_tail_split_at_bottom = *split == 1
                && !default_between_notes_gap
                && !allow_default_late_question_tail
                && !(late_internal_rewind_fit_split
                    && has_visible_endnote_separator
                    && compact_between_notes_gap)
                && endnote_has_visible_payload;
            let large_separator_title_tail_split = *split == 1
                && large_separator_block
                && ep_idx == 0
                && st.current_column + 1 >= st.col_count
                && st.current_height + en_fit > available
                && endnote_has_visible_payload;
            !single_line_tail_split_at_bottom && !large_separator_title_tail_split
        });
        if no_separator_visible_multiline_tail_fits_with_bleed {
            // 구분선이 없는 큰 미주 block에서 이미 보이는 다줄 tail이
            // 허용 bleed 안에 들어간다고 판정했다면, fit용 분할 후보도
            // 함께 제거해야 한다. 그렇지 않으면 문단을 4/2줄처럼
            // 쪼개 다음 쪽 문항 전체가 한컴보다 내려간다.
            split_endnote_to_fit = None;
        }
        let compact_non_default_empty_column_rewind_fits = compact_between_notes_gap
            && !default_between_notes_gap
            && internal_vpos_rewind
            && st.current_height <= 2.0
            && st.current_height + total_advance_fit
                <= available + ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX;
        if compact_non_default_empty_column_rewind_fits {
            // 이전 단 하단에서 다음 단/쪽으로 넘어온 내부 rewind 문단이
            // 새 단 맨 위에서 통째로 들어가면 다시 줄 단위로 쪼개지 않는다.
            // 여기서 분할하면 왼쪽 단에 수식 두 줄만 남고 다음 문항이
            // 오른쪽 단으로 밀려 한컴/PDF보다 한 쪽 많아진다.
            split_endnote_to_fit = None;
        }
        let visible_compact_sequential_tail_fits_current_column = compact_between_notes_gap
            && !default_between_notes_gap
            && has_visible_endnote_separator
            && ep_idx > 0
            && st.current_column + 1 < st.col_count
            && !local_vpos_rewind
            && !internal_vpos_rewind
            && endnote_has_visible_payload
            && st.current_height + total_advance_fit
                <= available + ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX;
        // [Task #1363 v2 Stage 3] A2: split 불가(단일줄 등) para 가 단을
        // 넘으면 먼저 단 advance (fit-or-advance). sim 이 렌더-정합이므로
        // overflow 판정이 신뢰 가능.
        InitialFitResult {
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
            split_endnote_to_fit,
            late_internal_rewind_fit_split,
            compact_non_default_empty_column_rewind_fits,
            visible_compact_sequential_tail_fits_current_column,
        }
    }
}

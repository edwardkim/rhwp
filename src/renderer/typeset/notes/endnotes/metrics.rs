//! 미주 fit/advance 계산. 전달된 미주별 accumulator만 갱신한다.

use crate::renderer::typeset::notes::endnotes::profile::EnSsotLevel;
use crate::renderer::typeset::notes::endnotes::types::EnMetricsVars;
use crate::renderer::typeset::{hwpunit_to_px, TypesetEngine};

impl TypesetEngine {
    /// [#2064 추출] 미주 fit/advance 메트릭 계산 (#1363 SSOT) — 라운드 5 산물의
    /// `compute_en_metrics` 클로저를 fn 으로 승격. 본문 무변경, 캡처는 `EnMetricsVars`.
    /// 반환: (en_fit, advance).
    #[allow(clippy::too_many_arguments)]
    pub(in crate::renderer::typeset) fn compute_endnote_metrics(
        &self,
        prev: Option<i32>,
        emit: bool,
        current_endnote_had_inline_object_vpos_overestimate: &mut bool,
        v: EnMetricsVars,
    ) -> (f64, f64) {
        let EnMetricsVars {
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
        } = v;
        if col_count > 1 {
            if let (Some(tf), Some(tb)) = (this_first_offset, this_bottom_offset) {
                let base = if local_vpos_rewind || large_vpos_jump_at_column_top {
                    tf
                } else {
                    prev.unwrap_or(tf)
                };
                let advance_px = hwpunit_to_px((tb - base).max(0), dpi);
                let compact_local_rewind = compact_endnote_separator_profile
                    && local_vpos_rewind
                    && !local_vpos_rewind_crosses_prev_content;
                // 한컴 저장본의 미주 LineSeg는 TAC 도형을 포함한 문단의
                // 다음 줄/문단 시작 vpos까지 이미 반영한다. formatter가
                // inline object 높이를 다시 큰 floor로 잡으면 2023 12쪽처럼
                // 다음 문제 시작이 한 단 늦게 밀린다.
                let inline_object_formatter_overestimate = compact_endnote_separator_profile
                    && has_treat_as_char_picture_shape
                    && !internal_vpos_rewind
                    && !compact_local_rewind
                    && !large_vpos_jump_at_column_top
                    && h4f > advance_px + 80.0
                    && advance_px > min_vpos_rewind_height + 40.0;
                if inline_object_formatter_overestimate {
                    *current_endnote_had_inline_object_vpos_overestimate = true;
                }
                let min_h = if inline_object_formatter_overestimate {
                    (advance_px - trailing_ls_px).max(min_vpos_rewind_height)
                } else if internal_vpos_rewind || compact_local_rewind {
                    min_vpos_rewind_height
                } else {
                    h4f
                };
                let stale_forward_vpos = compact_endnote_separator_profile
                    && !local_vpos_rewind
                    && !large_vpos_jump_at_column_top
                    && (!large_separator_block
                        || has_visible_endnote_separator
                        || cap_large_separator_stale_forward_vpos)
                    && advance_px > h4f + 100.0;
                let compact_internal_rewind_full_advance = compact_endnote_separator_profile
                    && internal_vpos_rewind
                    && !local_vpos_rewind
                    && !large_vpos_jump_at_column_top
                    && !has_treat_as_char_picture_shape
                    && current_height_for_metrics < available * 0.45
                    && tot > advance_px + 40.0;
                let cap_no_separator_stale_new_note = large_separator_block
                    && !has_visible_endnote_separator
                    && (current_height_for_metrics < available * 0.50
                        || (current_column_has_tac_picture_only
                            && current_height_for_metrics < available * 0.65)
                        || no_separator_new_note_head_fits_current_column);
                let capped_new_endnote_advance = if large_separator_block
                    && !has_visible_endnote_separator
                    && !cap_no_separator_stale_new_note
                {
                    None
                } else {
                    new_endnote_between_notes_px
                        .map(|gap| h4f + gap)
                        .filter(|cap| advance_px > *cap + 12.0)
                };
                let metric_advance_px = if compact_internal_rewind_full_advance {
                    tot
                } else if compact_local_rewind {
                    min_vpos_rewind_height
                } else if let Some(cap) = capped_new_endnote_advance {
                    cap
                } else if stale_forward_vpos {
                    h4f
                } else {
                    advance_px
                };
                let fit = (metric_advance_px - trailing_ls_px).max(min_h);
                let acc_legacy = metric_advance_px.max(min_h);
                // [Task #1363] Divergence A: 내부 vpos rewind para 는
                // layout 이 첫 줄만 vpos 로 배치한 뒤 나머지 줄을 순차
                // format 으로 렌더하므로 실제 점유 높이 = 전체
                // line_advances_sum. saved-vpos delta(metric_advance_px)
                // 는 rewind 로 과소 추정(pi=894 −61.2)되어 단 하단
                // 본문 초과를 유발 → SSOT 로 대체.
                // [Task #1363 Stage 5] 잔여 Divergence B(trailing-ls)·
                // 전면 SSOT 는 acc=line_advances_sum 로 닫을 수 없음(실증):
                //  · 전면: capped/stale/overlap para 를 렌더가 line_adv_sum
                //    보다 작게 겹쳐 그려 2022 overflow +166px 회귀.
                //  · uncapped sequential 한정: trailing-ls 가산이 미주
                //    질문 흐름(단 배치)을 흔들어 issue_1139/1261/1284 10건
                //    회귀. → 잔여 divergence 는 overflow 무영향이고 안전
                //    정합 불가하므로 보류. acc 는 A(rewind)/C(TAC)만 SSOT.
                // [#5886] 문단-사이 compact 되감김 과소 계상은 acc 를
                // line_advances_sum 으로 올리지 않는다. 그 경로는 2023/2024
                // 질문 흐름(1139·1375)과 overflow_cell 을 흔든다. 용지 밖
                // 잔여는 `page_offcanvas_with_para` 단 전환으로만 막는다.
                let acc = if ssot_level >= EnSsotLevel::A && internal_vpos_rewind {
                    line_advances_sum.max(min_vpos_rewind_height)
                } else {
                    acc_legacy
                };
                if emit && ssot_debug {
                    eprintln!(
                            "EN_SSOT pi={} rewind={} acc_legacy={:.1} acc={:.1} line_adv_sum={:.1} fit={:.1} h4f={:.1}",
                            en_para_idx,
                            internal_vpos_rewind,
                            acc_legacy,
                            acc,
                            line_advances_sum,
                            fit,
                            h4f,
                        );
                }
                (fit, acc)
            } else {
                if emit && ssot_debug {
                    eprintln!(
                            "EN_SSOT pi={} rewind={} acc_legacy={:.1} acc={:.1} line_adv_sum={:.1} fit={:.1} h4f={:.1}",
                            en_para_idx,
                            internal_vpos_rewind,
                            tot,
                            tot,
                            line_advances_sum,
                            h4f,
                            h4f,
                        );
                }
                (h4f, tot)
            }
        } else {
            (h4f, tot)
        }
    }
}

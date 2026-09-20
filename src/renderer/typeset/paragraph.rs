//! 문단 조판의 책임 경계.
//!
//! 구성된 줄 조회, 문단 구성 결과의 높이 조회, 저장 줄 간격 판정을 소유한다.
//! 문단 구성은 필요한 관측값을 읽고 결과만 반환한다.
//! fit 예산의 읽기 전용 계산은 fit에, 1회성 보정 소비는 state에 있다.
//! 줄 후보 계산은 scan, 그 뒤의 경계 보정은 split에 있다.
//! 진입 예산과 전체/넘침/빈 구성 결과 배치, 분할 진입·페이지 전환을 조정한다.
//! 저장 강제 경계와 최종 fit 선택, 표 문단은 상위에 남아 있다.
//! 하위 Query는 원본 IR이나 페이지 상태를 변경하지 않으며, 확정 조각 적용은 state가 맡는다.

pub(super) mod context;
mod entry;
pub(super) mod fit;
pub(super) mod format;
pub(super) mod line_queries;
pub(super) mod metrics;
pub(super) mod overflow;
pub(super) mod placement;
pub(super) mod scan;
pub(super) mod split;
pub(super) mod split_entry;
pub(super) mod stored_lines;

use super::{para_has_visible_text, para_is_treat_as_char_picture_only, TypesetState};
use crate::model::paragraph::Paragraph;
use crate::renderer::pagination::PageItem;
use crate::renderer::style_resolver::ResolvedStyleSet;
use metrics::FormattedParagraph;
use stored_lines::{next_boundary_reverts_spacing_trim, spacing_trim_restorable};

/// 진입 fit 판단 뒤의 줄 분할을 조정한다. 쪽 전환 후 후보를 다시 계산하며,
/// 전체 문단 재시도와 조각 배치 후 이월을 구분한다. 진입 시 고정한 기준 예산은 유지한다.
#[allow(clippy::too_many_arguments)]
pub(super) fn place_split_paragraph(
    st: &mut TypesetState,
    para_idx: usize,
    para: &Paragraph,
    fmt: &FormattedParagraph,
    paragraphs: &[Paragraph],
    line_count: usize,
    base_available: f64,
    layout_drift_safety_px: f64,
    forced_page_break_line: Option<usize>,
    native_hwp5_existing_footnote_reset_line: Option<usize>,
    current_page_vpos_base: Option<i32>,
    is_tac_picture_stack: bool,
    dpi: f64,
) {
    // 줄 단위 분할 루프
    let mut cursor_line: usize = 0;
    while cursor_line < line_count {
        let fn_margin = if st.current_footnote_height > 0.0 {
            st.footnote_safety_margin
        } else {
            0.0
        };
        let page_avail = if cursor_line == 0 {
            (base_available
                - st.current_footnote_height
                - fn_margin
                - st.current_height
                - st.current_zone_y_offset)
                .max(0.0)
        } else {
            base_available
        };

        let sp_b = if cursor_line == 0 {
            fmt.spacing_before
        } else {
            0.0
        };
        // Task #332 Stage 4b: partial split 의 줄 단위 fit 검사에도 layout drift 마진 적용
        let avail_for_lines = (page_avail - sp_b - layout_drift_safety_px).max(0.0);

        let scan::LineScanResult {
            end_line,
            cumulative,
            used_saved_tail_vpos_fit,
        } = scan::scan_lines(
            para,
            fmt,
            paragraphs,
            para_idx,
            cursor_line,
            line_count,
            avail_for_lines,
            forced_page_break_line,
            native_hwp5_existing_footnote_reset_line,
            current_page_vpos_base,
            is_tac_picture_stack,
            &st.paragraph_line_scan_page(),
            dpi,
        );

        let split::SplitBoundary {
            end_line,
            cumulative,
        } = split::refine_split_boundary(
            para,
            fmt,
            paragraphs.get(para_idx + 1),
            cursor_line,
            line_count,
            avail_for_lines,
            st.base_available_height(),
            st.profile.hwp5_stored_pagination_layout(),
            dpi,
            split::SplitBoundary {
                end_line,
                cumulative,
            },
        );

        let Some(fragment) = placement::plan_fragment(
            fmt,
            para_idx,
            cursor_line,
            line_count,
            sp_b,
            avail_for_lines,
            split::SplitBoundary {
                end_line,
                cumulative,
            },
            used_saved_tail_vpos_fit,
            &st.current_items,
        ) else {
            st.advance_column_or_new_page();
            continue;
        };
        st.commit_split_paragraph_fragment(fragment);

        if end_line >= line_count {
            break;
        }

        // move: 나머지 줄 → 다음 단/페이지
        st.advance_column_or_new_page();
        cursor_line = end_line;
    }
}

/// 진입 fit을 통과한 전체 문단을 배치한다. 항목 순서 확정 뒤 흐름 메트릭을 계산한다.
#[allow(clippy::too_many_arguments)]
pub(super) fn place_fitted_paragraph(
    st: &mut TypesetState,
    para_idx: usize,
    para: &Paragraph,
    fmt: &FormattedParagraph,
    paragraphs: &[Paragraph],
    styles: &ResolvedStyleSet,
    trim_spacing_before_for_flow: bool,
    trimmed_sb_gate: f64,
    body_bottom_vpos: Option<i32>,
    dpi: f64,
) {
    let defer_preceding_float =
        placement::defer_preceding_float(&st.current_items, paragraphs, para_idx, para);
    st.insert_fitted_paragraph(para_idx, defer_preceding_float);
    // [Task #391] 다단/단단 분기:
    //   - 단단 (col_count == 1): total_height (k-water-rfp p3 311px drift 차단, #359)
    //   - 다단 (col_count > 1): height_for_fit (exam_eng 8p 정상 단 채움 복원)
    // 다단에서는 layout 이 vpos 기반으로 항목을 단별로 stacking 하므로
    // typeset 누적 시 trailing_ls 인플레이션이 단을 조기 종료시킴.
    let advance = fmt.flow_advance_height(
        para,
        st.col_count,
        trim_spacing_before_for_flow,
        st.vpos_ladder_dirty
            || !spacing_trim_restorable(paragraphs, para_idx)
            || next_boundary_reverts_spacing_trim(
                st.profile.hwpx_stored_layout() && !st.profile.hwp3_layout(),
                paragraphs,
                styles,
                para_idx,
                dpi,
            ),
        st.vpos_page_base.is_none() && st.vpos_lazy_base.is_some(),
    );
    if std::env::var("RHWP_DIAG_ADV").is_ok() {
        eprintln!(
            "DIAG_ADV pi={} adv={:.1} total={:.1} h4f={:.1} sb={:.1} sa={:.1} cur={:.1}",
            para_idx,
            advance,
            fmt.total_height,
            fmt.height_for_fit,
            fmt.spacing_before,
            fmt.spacing_after,
            st.current_height,
        );
    }
    let trimmed_spacing_before = trimmed_sb_gate
        * fmt.flow_trimmed_spacing_before(
            para,
            st.col_count,
            trim_spacing_before_for_flow,
            st.vpos_ladder_dirty
                || !spacing_trim_restorable(paragraphs, para_idx)
                || next_boundary_reverts_spacing_trim(
                    st.profile.hwpx_stored_layout() && !st.profile.hwp3_layout(),
                    paragraphs,
                    styles,
                    para_idx,
                    dpi,
                ),
            st.vpos_page_base.is_none() && st.vpos_lazy_base.is_some(),
        );
    st.apply_full_paragraph_flow(
        advance,
        fmt.total_height,
        trimmed_spacing_before,
        body_bottom_vpos,
    );
}

/// 일반 fit 실패 뒤 atomic → tail 순서로 시도한다. 성공 시 호출자는 즉시 반환한다.
#[allow(clippy::too_many_arguments)]
pub(super) fn try_place_overflow_paragraph(
    st: &mut TypesetState,
    para_idx: usize,
    para: &Paragraph,
    fmt: &FormattedParagraph,
    paragraphs: &[Paragraph],
    styles: &ResolvedStyleSet,
    trim_spacing_before_for_flow: bool,
    body_bottom_vpos: Option<i32>,
    available: f64,
    forced_page_break_line: Option<usize>,
    dpi: f64,
) -> bool {
    let page = st.paragraph_overflow_page();
    if overflow::atomic_overflow_fits(para, fmt, paragraphs, para_idx, &page, available, dpi) {
        st.begin_atomic_overflow_paragraph(para_idx);
        let advance = fmt.flow_advance_height(
            para,
            st.col_count,
            trim_spacing_before_for_flow,
            st.vpos_ladder_dirty
                || !spacing_trim_restorable(paragraphs, para_idx)
                || next_boundary_reverts_spacing_trim(
                    st.profile.hwpx_stored_layout() && !st.profile.hwp3_layout(),
                    paragraphs,
                    styles,
                    para_idx,
                    dpi,
                ),
            false,
        );
        st.advance_atomic_overflow_paragraph(advance, fmt.total_height, body_bottom_vpos);
        return true;
    }
    if overflow::tail_overflow_candidate(
        para,
        fmt,
        paragraphs,
        para_idx,
        &page,
        forced_page_break_line,
    ) {
        let first_line_advance = fmt.line_advance(0);
        // 다음 문단이 어차피 쪽나누기로 페이지를 끝내므로, 다음 페이지 layout clamp 를
        // 막으려던 LAYOUT_DRIFT_SAFETY_PX(현재 페이지 한정) 여유는 이 경우 의미가 없다.
        // 따라서 safety 를 뺀 `available` 이 아니라 진짜 본문 하단(각주/존 차감 포함)인
        // available_height() 를 기준으로 초과량을 잰다.
        let true_available = st.available_height();
        // 초과량이 한 줄 미만(폰트 drift)일 때만 통째 배치.
        // (full-place 체크를 이미 통과 못 했으므로 overflow > -safety. 진짜 본문 하단
        //  기준으로 한 줄 미만 초과면 마지막 줄 spill 대신 통째 배치.)
        let overflow = st.current_height + fmt.height_for_fit - true_available;
        if overflow < first_line_advance {
            st.commit_tail_overflow_paragraph(para_idx, fmt.total_height, body_bottom_vpos);
            return true;
        }
    }
    false
}

/// 일반 fit/넘침 허용 실패 뒤 빈 구성 결과 또는 분할 진입을 조정한다.
#[allow(clippy::too_many_arguments)]
pub(super) fn place_after_failed_fit(
    st: &mut TypesetState,
    para_idx: usize,
    para: &Paragraph,
    fmt: &FormattedParagraph,
    paragraphs: &[Paragraph],
    styles: &ResolvedStyleSet,
    trim_spacing_before_for_flow: bool,
    trimmed_sb_gate: f64,
    body_bottom_vpos: Option<i32>,
    available: f64,
    layout_drift_safety_px: f64,
    stored_vpos_rewind_overflow_break: bool,
    forced_page_break_line: Option<usize>,
    native_hwp5_existing_footnote_reset_line: Option<usize>,
    current_page_vpos_base: Option<i32>,
    dpi: f64,
) {
    // split: 줄 단위 분할
    let line_count = fmt.line_heights.len();
    // [#2004] tac(글자처럼) 전면 그림이 줄마다 하나씩 쌓인 "이미지 스택" 문단은 저장
    // LINE_SEG 가 각 줄 vpos=0(각자 쪽 상단)으로 인코딩되어, 아래 hwp_authoritative
    // (다음 줄 vpos==0 이고 현재 줄 bottom 이 본문 안이면 현재 쪽 유지) 가 모든 줄을 한
    // 쪽에 쌓아 버린다. 이 문단만 hwp_authoritative 를 끄고 줄별 fit 분할(쪽당 1장)로
    // 되돌린다. 게이트는 formatter 의 stacked_tac_picture_heights 와 동일 의미.
    let tac_picture_only_para = para_is_treat_as_char_picture_only(para);
    let is_tac_picture_stack = tac_picture_only_para
        && line_count >= 2
        && fmt
            .line_heights
            .iter()
            .all(|h| *h > st.base_available_height() * 0.5);
    if line_count == 0 {
        st.begin_empty_line_paragraph(para_idx);
        // [Task #391] 다단/단단 분기:
        //   - 단단 (col_count == 1): total_height (k-water-rfp p3 311px drift 차단, #359)
        //   - 다단 (col_count > 1): height_for_fit (exam_eng 8p 정상 단 채움 복원)
        // 다단에서는 layout 이 vpos 기반으로 항목을 단별로 stacking 하므로
        // typeset 누적 시 trailing_ls 인플레이션이 단을 조기 종료시킴.
        let advance = fmt.flow_advance_height(
            para,
            st.col_count,
            trim_spacing_before_for_flow,
            st.vpos_ladder_dirty
                || !spacing_trim_restorable(paragraphs, para_idx)
                || next_boundary_reverts_spacing_trim(
                    st.profile.hwpx_stored_layout() && !st.profile.hwp3_layout(),
                    paragraphs,
                    styles,
                    para_idx,
                    dpi,
                ),
            false,
        );
        let trimmed_spacing_before = trimmed_sb_gate
            * fmt.flow_trimmed_spacing_before(
                para,
                st.col_count,
                trim_spacing_before_for_flow,
                st.vpos_ladder_dirty
                    || !spacing_trim_restorable(paragraphs, para_idx)
                    || next_boundary_reverts_spacing_trim(
                        st.profile.hwpx_stored_layout() && !st.profile.hwp3_layout(),
                        paragraphs,
                        styles,
                        para_idx,
                        dpi,
                    ),
                false,
            );
        st.apply_full_paragraph_flow(
            advance,
            fmt.total_height,
            trimmed_spacing_before,
            body_bottom_vpos,
        );
        return;
    }

    // Task #332 Stage 4a: partial split 시에도 동일 마진 적용
    let base_available = (st.base_available_height() - layout_drift_safety_px).max(0.0);

    let entry_fit = split_entry::inspect_entry(
        para,
        fmt,
        paragraphs,
        styles,
        para_idx,
        line_count,
        available,
        &st.paragraph_split_entry_page(),
        dpi,
    );
    if entry_fit.hangul2024_split_refit && !para_has_visible_text(para) && para.controls.is_empty()
    {
        st.mark_blank_paragraph_spill(para_idx);
    }
    if split_entry::should_advance(
        para,
        &entry_fit,
        available,
        stored_vpos_rewind_overflow_break,
        &st.paragraph_split_entry_page(),
        dpi,
    ) {
        st.advance_column_or_new_page();
    }

    place_split_paragraph(
        st,
        para_idx,
        para,
        fmt,
        paragraphs,
        line_count,
        base_available,
        layout_drift_safety_px,
        forced_page_break_line,
        native_hwp5_existing_footnote_reset_line,
        current_page_vpos_base,
        is_tac_picture_stack,
        dpi,
    );
}

/// 문단 진입 조정과 1회성 fit 예산 소비 결과. 실제 분할 경계는 이후에 계산한다.
pub(super) struct FitBudget {
    pub strict_after_empty_host_float: bool,
    pub layout_drift_safety_px: f64,
    pub prev_is_partial_table: bool,
    pub available: f64,
}

/// 저장 꼬리 → 진단 → 편집 그림 이월 → 보정 소비 → float 배제 → 예산 순서를 보존한다.
pub(super) fn prepare_fit_budget(
    st: &mut TypesetState,
    para_idx: usize,
    para: &Paragraph,
    fmt: &FormattedParagraph,
    paragraphs: &[Paragraph],
    session_edited: bool,
    dpi: f64,
) -> FitBudget {
    if entry::stored_tail_fills_page(
        para_idx,
        para,
        paragraphs,
        st.profile.hwp5_stored_pagination_layout(),
        st.current_height,
        dpi,
        || st.available_height(),
    ) {
        st.fill_paragraph_entry_page_tail();
    }

    // [#2243 진단] 문단 진입 시 누적 높이 — 항목별 실소비 델타 추적용. 동작 불변.
    if std::env::var("RHWP_DIAG_FLOW").is_ok() {
        eprintln!(
            "DIAG_FLOW pi={} cur_h={:.1} page={} items={} ct={:?}",
            para_idx,
            st.current_height,
            st.pages.len(),
            st.current_items.len(),
            para.column_type,
        );
    }

    if entry::edited_picture_requires_transition(
        para,
        fmt,
        session_edited,
        !st.current_items.is_empty(),
        st.current_height,
        dpi,
        || st.available_height(),
    ) {
        st.advance_column_or_new_page();
    }

    let strict_after_empty_host_float = st.take_strict_paragraph_fit(para);
    let layout_drift_safety_px = fit::layout_drift_safety_px(paragraphs);
    let prev_is_partial_table =
        matches!(st.current_items.last(), Some(PageItem::PartialTable { .. }));
    let safety = st.take_paragraph_safety_margin(
        strict_after_empty_host_float,
        prev_is_partial_table,
        layout_drift_safety_px,
    );
    let exclusion_probe_height = fit::exclusion_probe_height(fmt, st.profile.hwpx_stored_layout());
    st.apply_visible_float_exclusions(exclusion_probe_height);
    let footnote_margin_addback =
        st.take_paragraph_footnote_margin_addback(strict_after_empty_host_float);
    let tail_overflow =
        st.take_paragraph_tail_overflow(strict_after_empty_host_float, fmt.height_for_fit);
    let available =
        (st.available_height() - safety + footnote_margin_addback + tail_overflow).max(0.0);

    FitBudget {
        strict_after_empty_host_float,
        layout_drift_safety_px,
        prev_is_partial_table,
        available,
    }
}

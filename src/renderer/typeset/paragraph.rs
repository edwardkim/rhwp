//! 문단 조판의 책임 경계.
//!
//! 구성된 줄 조회, 문단 구성 결과의 높이 조회, 저장 줄 간격 판정을 소유한다.
//! 문단 구성은 필요한 관측값을 읽고 결과만 반환한다.
//! fit 예산의 읽기 전용 계산은 fit에, 1회성 보정 소비는 state에 있다.
//! 줄 후보 계산은 scan, 그 뒤의 경계 보정은 split에 있다.
//! 진입 예산과 전체/넘침/빈 구성 결과 배치, 분할 진입·페이지 전환을 조정한다.
//! flow는 일반 문단의 최상위 순서를, controls::paragraph_flow는 표 소유 문단을 조정한다.
//! 강제 경계 후보의 우선순위와 전체 fit 선택·호환성 spill을 조정한다.
//! 하위 Query는 원본 IR이나 페이지 상태를 변경하지 않으며, 확정 조각 적용은 state가 맡는다.

pub(super) mod boundary;
mod columns;
pub(super) mod context;
pub(super) mod empty;
mod entry;
pub(super) mod fit;
pub(super) mod flow;
pub(super) mod format;
pub(super) mod line_queries;
pub(super) mod metrics;
pub(super) mod overflow;
pub(super) mod placement;
pub(super) mod scan;
pub(super) mod split;
pub(super) mod split_entry;
pub(super) mod stored_lines;
pub(super) mod whole_fit;

use super::{
    hwpx_saved_reset_fragment_matches_current_flow, missing_lineseg_trailing_line_break,
    native_hwp5_existing_footnote_reset_overlap_break_line,
    native_hwp5_first_footnote_overlap_break_line,
    native_hwp5_text_reset_before_large_tac_topbottom_picture_break_line, page_item_vpos_base,
    para_has_visible_text, para_is_treat_as_char_picture_only, preceding_stored_vpos,
    stored_vpos_rewinds, TypesetState,
};
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
        dpi,
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
    stored_vpos_rewind_break: bool,
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
        stored_vpos_rewind_break,
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

/// 다단 분기 전에만 검사한다. 흡수 시 항목이나 숨김 횟수를 추가하지 않는다.
pub(super) fn try_absorb_rowbreak_guide(
    st: &mut TypesetState,
    prev_is_partial_table: bool,
    para: &Paragraph,
    paragraphs: &[Paragraph],
    para_idx: usize,
) -> bool {
    if empty::hide_rowbreak_guide(prev_is_partial_table, para, paragraphs, para_idx) {
        st.hide_empty_paragraph(para_idx);
        return true;
    }
    false
}

/// 다단 조판의 조기 반환 뒤에서만 실행한다. 숨김 옵션의 페이지 수명과 구역 끝 처리를 구분한다.
#[allow(clippy::too_many_arguments)]
pub(super) fn try_absorb_empty_paragraph(
    st: &mut TypesetState,
    para_idx: usize,
    para: &Paragraph,
    fmt: &FormattedParagraph,
    paragraphs: &[Paragraph],
    is_last_in_section: bool,
    available: f64,
    layout_drift_safety_px: f64,
) -> bool {
    // [Task #362] 한컴 빈 줄 감추기 (SectionDef bit 19, hide_empty_line):
    // 빈 paragraph 가 현재 공간을 overflow 시키면 height=0 으로 처리 (페이지 당 최대 2개).
    // Paginator (engine.rs:85-106) 와 동일 시멘틱.
    // (kps-ai p67~70 case: PartialTable 후속 빈 paragraphs 가 다수 발생, 한컴은 표시 안 함.)
    if st.hide_empty_line {
        st.begin_empty_paragraph_page();
        if empty::hide_overflowing_empty(
            para,
            fmt,
            !st.current_items.is_empty(),
            st.current_height,
            available,
            st.hidden_empty_lines,
        ) {
            st.commit_counted_hidden_paragraph(para_idx);
            return true;
        }
    }
    match empty::trailing_disposition(
        para,
        fmt,
        paragraphs,
        is_last_in_section,
        available,
        layout_drift_safety_px,
        &st.paragraph_empty_tail_page(),
    ) {
        empty::TailDisposition::Continue => false,
        empty::TailDisposition::Hidden => {
            st.hide_empty_paragraph(para_idx);
            true
        }
        empty::TailDisposition::Unadvanced => {
            st.place_unadvanced_empty_paragraph(para_idx);
            true
        }
    }
}

pub(super) struct WholeFitDecision {
    pub fits: bool,
    pub stored_vpos_rewind_break: bool,
    pub stored_vpos_rewind_overflow_break: bool,
}

/// 저장 근거 조회 → 진단 → 빈 문단 spill 기록 → 전체 fit 선택 순서를 보존한다.
#[allow(clippy::too_many_arguments)]
pub(super) fn decide_whole_fit(
    st: &mut TypesetState,
    para_idx: usize,
    para: &Paragraph,
    fmt: &FormattedParagraph,
    paragraphs: &[Paragraph],
    strict_after_empty_host_float: bool,
    forced_page_break_line: Option<usize>,
    current_page_vpos_base: Option<i32>,
    available: f64,
    dpi: f64,
) -> WholeFitDecision {
    let whole_fit::WholeFitEvidence {
        saved_single_line_bottom_fits,
        saved_list_tail_body_vpos_fits,
        page_end_fit_height,
        stored_vpos_rewind_break,
        stored_vpos_rewind_overflow_break,
        hangul2024_rewind_override,
    } = whole_fit::inspect(
        para_idx,
        para,
        fmt,
        paragraphs,
        strict_after_empty_host_float,
        forced_page_break_line,
        current_page_vpos_base,
        available,
        &st.paragraph_whole_fit_page(),
        dpi,
        || st.available_height(),
    );
    if std::env::var("RHWP_DIAG_COMPAT24").is_ok()
        && stored_vpos_rewinds(preceding_stored_vpos(paragraphs, para_idx), para)
    {
        eprintln!(
            "DIAG_COMPAT24 rewind-site pi={para_idx} break={stored_vpos_rewind_break} \
             cur={:.1} fit_h={page_end_fit_height:.1} avail={available:.1} \
             reclaimed={:.1} items={} forced={:?}",
            st.current_height,
            st.hangul2024_reclaimed,
            st.current_items.len(),
            forced_page_break_line,
        );
    }
    // [compat 2024] 저장 신호를 덮은 그 빈 문단만 한글 2024 처럼 쪽 하단
    // 여백으로 흘린다(place 적합 우회). 이웃 빈 문단까지 흘리면 2024 보다
    // 한 문단 과적재된다(idx22 실측). 되감김 덮음도 같은 자격을 준다.
    if hangul2024_rewind_override && !para_has_visible_text(para) && para.controls.is_empty() {
        st.mark_blank_paragraph_spill(para_idx);
    }
    let hangul2024_blank_spill = st.profile.hangul2024_layout()
        && st.hangul2024_spill_para == Some(para_idx)
        && !st.current_items.is_empty();
    if std::env::var("RHWP_DIAG_6031").is_ok()
        && st.current_height + page_end_fit_height > available
    {
        eprintln!(
            "DIAG_6031 pi={para_idx} cur={:.1} fit_h={page_end_fit_height:.1} avail={available:.1} single={saved_single_line_bottom_fits} list_tail={saved_list_tail_body_vpos_fits} base={:?}",
            st.current_height, current_page_vpos_base,
        );
    }
    WholeFitDecision {
        fits: forced_page_break_line.is_none()
            && !stored_vpos_rewind_break
            && (hangul2024_blank_spill
                || st.current_height + page_end_fit_height <= available
                || saved_single_line_bottom_fits
                || saved_list_tail_body_vpos_fits),
        stored_vpos_rewind_break,
        stored_vpos_rewind_overflow_break,
    }
}

/// 기존 각주 경계는 선택된 강제 경계와 별도로 후속 줄 스캔에서도 소비한다.
pub(super) struct ForcedPageBoundary {
    pub native_hwp5_existing_footnote_reset_line: Option<usize>,
    pub current_page_vpos_base: Option<i32>,
    pub forced_page_break_line: Option<usize>,
}

/// 기존 각주 경계를 먼저 조회하고, 저장/각주/그림 후보를 원래 단락 순서로 선택한다.
/// 이 조정자는 읽기 전용이다. 각주 측정 helper의 좁은 관측 경계 분리는 R4에 남긴다.
pub(super) fn prepare_forced_page_boundary(
    st: &TypesetState,
    para_idx: usize,
    para: &Paragraph,
    fmt: &FormattedParagraph,
    paragraphs: &[Paragraph],
    available: f64,
    dpi: f64,
) -> ForcedPageBoundary {
    // native HWP5 본문은 기존 각주가 있는 page tail에서도 `vpos=0` reset으로
    // 다음 physical page를 기록할 수 있다. 일반 reset은 과분할 위험이 있으므로,
    // 실제 FootnoteArea 경계와 source/flow가 함께 맞을 때만 강제 경계로 쓴다.
    let native_hwp5_existing_footnote_reset_line =
        native_hwp5_existing_footnote_reset_overlap_break_line(st, para, fmt, paragraphs, dpi);
    let current_page_vpos_base = st.vpos_page_base.or_else(|| {
        st.current_items
            .first()
            .and_then(|item| page_item_vpos_base(item, paragraphs))
    });
    let hwp3_converted_hwp5 = st.profile.hwp3_layout()
        && !st.profile.hwp3_native_layout()
        && !st.profile.hwpx_container();
    let internal_forced_page_break_line = boundary::internal_vpos_page_break_line(
        para,
        fmt.line_heights.len(),
        st.layout.body_area.height,
        dpi,
        st.profile.hwpx_stored_layout() || st.profile.hwp3_native_layout() || hwp3_converted_hwp5,
        st.profile.hwp5_stored_pagination_layout(),
        hwp3_converted_hwp5,
    )
    .filter(|break_line| {
        // HWPX의 reset은 local writer cursor도 재사용한다. 현재 flow와
        // anchor가 맞지 않는 reset은 physical page 경계로 승격하지 않는다.
        !st.profile.hwpx_stored_layout()
            || st.current_items.is_empty()
            || hwpx_saved_reset_fragment_matches_current_flow(
                st,
                para,
                0,
                *break_line,
                current_page_vpos_base.unwrap_or(0),
                dpi,
            )
    });
    let forced_page_break_line = internal_forced_page_break_line
        .or_else(|| {
            st.profile.hwpx_stored_layout().then(|| {
                boundary::hwpx_explicit_page_break_tail_line(
                    para,
                    paragraphs.get(para_idx + 1),
                    fmt.line_heights.len(),
                    st.layout.body_area.height,
                    dpi,
                )
            })?
        })
        .or_else(|| {
            native_hwp5_first_footnote_overlap_break_line(st, para, fmt, dpi)
                .map(|footnote_break| footnote_break.body_break_line)
        })
        .or_else(|| {
            missing_lineseg_trailing_line_break(
                para,
                fmt.line_heights.len(),
                st.current_height,
                available,
                fmt.line_spacings.last().copied().unwrap_or(0.0),
                st.profile.hwpx_stored_layout() || hwp3_converted_hwp5,
                hwp3_converted_hwp5,
            )
        })
        .or_else(|| {
            native_hwp5_text_reset_before_large_tac_topbottom_picture_break_line(
                st, para, fmt, paragraphs, para_idx, dpi,
            )
        })
        // full-fit early return보다 앞의 같은 chain에 넣어야 reset tail을 통째로
        // 배치해 separator와 겹치는 우회가 없다.
        .or(native_hwp5_existing_footnote_reset_line);
    ForcedPageBoundary {
        native_hwp5_existing_footnote_reset_line,
        current_page_vpos_base,
        forced_page_break_line,
    }
}

/// 일반 줄 분할과 구분되는 저장 다단 경로. 선택된 경계가 있으면 기존처럼 이 경로가
/// 문단을 소비한다. 유효 조각이 없어 루프를 종료해도 일반 fit 경로로 재진입하지 않는다.
pub(super) fn try_place_multicolumn_paragraph(
    st: &mut TypesetState,
    para_idx: usize,
    para: &Paragraph,
    fmt: &FormattedParagraph,
    dpi: f64,
) -> bool {
    let col_breaks = columns::detect_breaks(
        para,
        st.col_count,
        st.current_column,
        st.current_endnote_flow,
        st.layout.available_body_height(),
        dpi,
    );
    if col_breaks.len() <= 1 {
        return false;
    }
    let line_count = fmt.line_heights.len();
    for (bi, &break_start) in col_breaks.iter().enumerate() {
        let break_end = if bi + 1 < col_breaks.len() {
            col_breaks[bi + 1]
        } else {
            line_count
        };
        let Some(fragment) =
            columns::plan_fragment(para_idx, fmt, break_start, break_end, line_count)
        else {
            break;
        };
        st.commit_multicolumn_paragraph_fragment(fragment);
        // 마지막 조각이 아니면 다음 단으로 진행.
        if bi + 1 < col_breaks.len() {
            st.advance_after_multicolumn_fragment();
        }
    }
    true
}

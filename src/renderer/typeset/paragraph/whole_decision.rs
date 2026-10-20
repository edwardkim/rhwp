//! 전체 문단 수용과 저장 경계 근거의 조정. 상태 변경은 원래 순서를 유지한다.
use super::*;

/// 저장 근거 조회 → 진단 → 빈 문단 spill 기록 → 전체 fit 선택 순서를 보존한다.
#[allow(clippy::too_many_arguments)]
pub(in crate::renderer::typeset) fn decide_whole_fit(
    st: &mut TypesetState,
    para_idx: usize,
    para: &Paragraph,
    fmt: &FormattedParagraph,
    paragraphs: &[Paragraph],
    styles: &ResolvedStyleSet,
    strict_after_empty_host_float: bool,
    forced_page_break_line: Option<usize>,
    current_page_vpos_base: Option<i32>,
    available: f64,
    dpi: f64,
) -> WholeFitDecision {
    // 같은 확정 placement를 paint도 사용한다. 옛 host 좌표의 경계를
    // 이미 새 쪽에서 소비했다면 뒤 줄에 두 번째 경계를 강제하지 않는다.
    let accepted_opening_owner = (st.current_start_height == 0.0
        && !st.current_endnote_flow
        && !st.profile.session_edited()
        && (st.profile.hwp5_stored_pagination_layout() || st.profile.hwpx_stored_layout()))
    .then(|| {
        crate::renderer::float_placement::restarted_tac::accepted_column_origin(
            &st.current_items,
            &st.inline_placements,
            paragraphs,
            styles,
            dpi,
        )
    })
    .flatten()
    .filter(|(_, origin)| current_page_vpos_base == Some(*origin))
    .map(|(owner, _)| owner);
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
        accepted_opening_owner,
        stored_lines::stored_frame_prefix_consumed_before_page(st, paragraphs, para_idx, styles),
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

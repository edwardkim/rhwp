//! 기존 강제 쪽 경계 판단의 호출 순서와 반환 계약을 보존한다.
use super::*;

pub(in crate::renderer::typeset) fn prepare_forced_page_boundary(
    st: &TypesetState,
    para_idx: usize,
    para: &Paragraph,
    fmt: &FormattedParagraph,
    paragraphs: &[Paragraph],
    available: f64,
    note_measure: &super::super::notes::footnotes::measure::BodyFootnoteMeasure<'_>,
) -> ForcedPageBoundary {
    let dpi = note_measure.engine.dpi;
    let stored_frame_valid = !super::stored_lines::stored_frame_prefix_consumed_before_page(
        st,
        paragraphs,
        para_idx,
        note_measure.styles,
    );
    // native HWP5 본문은 기존 각주가 있는 page tail에서도 `vpos=0` reset으로
    // 다음 physical page를 기록할 수 있다. 일반 reset은 과분할 위험이 있으므로,
    // 실제 FootnoteArea 경계와 source/flow가 함께 맞을 때만 강제 경계로 쓴다.
    let native_hwp5_existing_footnote_boundary =
        native_hwp5_existing_footnote_reset_boundary(st, para, fmt, paragraphs, note_measure);
    // 흐름 스냅이 이미 확정한 지연 기준은 마지막 줄·내부 쪽 경계도 함께 소비한다.
    // 첫 빈 개체 호스트의 저장 위치는 표 밴드 뒤의 줄일 수 있어 쪽 원점으로 다시 쓰지 않는다.
    let current_page_vpos_base = st.vpos_page_base.or(st.vpos_lazy_base).or_else(|| {
        st.current_items
            .first()
            .and_then(|item| page_item_vpos_base(item, paragraphs))
    });
    // 쪽 소유가 저장 앵커와 현재 흐름으로 입증된 일반 본문은 시작 높이의
    // 비율로 다시 거절하지 않는다. 기존 세션 편집 플래그와 구성 줄 수,
    // 유효 저장 앵커를 확인하고 실제 재조판으로 사라진 reset은 재사용하지 않는다.
    // 같은 원본 줄 사다리와 실제 흐름은 컨테이너 형식과 무관하게 같은 쪽을 소유한다.
    let anchored_stored_body_reset_line = (!st.profile.session_edited()
        && stored_frame_valid
        && para_has_visible_text(para)
        && fmt.line_heights.len() == para.line_segs.len())
    .then(|| {
        (1..para.line_segs.len()).find(|&break_line| {
            para.line_segs[break_line].vertical_pos < para.line_segs[break_line - 1].vertical_pos
                && stored_body_reset_fragment_matches_current_flow(
                    st,
                    para,
                    0,
                    break_line,
                    current_page_vpos_base.unwrap_or(0),
                    fmt.spacing_before,
                    dpi,
                )
        })
    })
    .flatten();
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
        stored_frame_valid
            && (!st.profile.hwpx_stored_layout()
                || st.current_items.is_empty()
                || stored_body_reset_fragment_matches_current_flow(
                    st,
                    para,
                    0,
                    *break_line,
                    current_page_vpos_base.unwrap_or(0),
                    fmt.spacing_before,
                    dpi,
                ))
    });
    let stored_page_start_line = (!st.profile.session_edited())
        .then(|| boundary::stored_page_start_line(para, fmt.line_count(), 0))
        .flatten();
    let native_hwp3_stored_break_line = (st.profile.hwp3_native_layout()
        && !st.profile.session_edited())
    .then(|| boundary::native_hwp3_stored_page_break_line(para, fmt.line_heights.len()))
    .flatten();
    let forced_page_break_line = stored_page_start_line
        .or(native_hwp3_stored_break_line)
        .or(anchored_stored_body_reset_line)
        .or(internal_forced_page_break_line)
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
        .or(native_hwp5_existing_footnote_boundary.map(|boundary| boundary.break_line));
    if std::env::var("RHWP_DIAG_SPLIT_BUDGET").is_ok() && forced_page_break_line.is_some() {
        // 물리 예산 부족과 저장 경계 강제를 구분한다. 내용·동작은 바꾸지 않는다.
        eprintln!(
            "DIAG_FORCED_BOUNDARY section={} pi={para_idx} current={:.3} fit={:.3} total={:.3} available={available:.3} lines={} base={current_page_vpos_base:?} dirty={} stored_page={stored_page_start_line:?} anchored={anchored_stored_body_reset_line:?} internal={internal_forced_page_break_line:?} selected={forced_page_break_line:?}",
            st.section_index, st.current_height, fmt.height_for_fit, fmt.total_height,
            fmt.line_count(), st.vpos_ladder_dirty,
        );
    }
    ForcedPageBoundary {
        native_hwp5_existing_footnote_boundary,
        current_page_vpos_base,
        forced_page_break_line,
    }
}

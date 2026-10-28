//! 일반 문단의 fit/배치 담당자를 기존 순서대로 연결하는 조정 경계.
//! 판별 알고리즘은 각 Query에, 페이지 상태 변경은 기존 Command에 남긴다.
//! 엔진 profile은 예산 준비 시, state profile은 강제 경계 준비 후에 각각 읽는다.

use crate::model::paragraph::Paragraph;
use crate::renderer::pagination::PageItem;
use crate::renderer::style_resolver::ResolvedStyleSet;
use crate::renderer::typeset::paragraph::{self, metrics::FormattedParagraph};
use crate::renderer::typeset::{TypesetEngine, TypesetState};

pub(in crate::renderer::typeset) struct ParagraphFlowInput<'a> {
    pub para_idx: usize,
    pub para: &'a Paragraph,
    pub fmt: &'a FormattedParagraph,
    pub paragraphs: &'a [Paragraph],
    pub styles: &'a ResolvedStyleSet,
    pub is_last_in_section: bool,
}

pub(in crate::renderer::typeset) fn place(
    st: &mut TypesetState,
    input: ParagraphFlowInput<'_>,
    engine: &TypesetEngine,
) {
    let dpi = engine.dpi;
    let ParagraphFlowInput {
        para_idx,
        para,
        fmt,
        paragraphs,
        styles,
        is_last_in_section,
    } = input;
    let stored_frame_shared_spacing =
        crate::renderer::float_placement::stored_frame_successor_shared_spacing_px(
            &st.paragraph_float_placements,
            para_idx,
            fmt.spacing_before,
            st.current_height,
        );
    st.reclaim_flow_by(stored_frame_shared_spacing.min(st.current_height));
    let previous_is_partial_table = st.current_items.last().is_some_and(
        |item| matches!(item, PageItem::PartialTable { para_index, .. } if *para_index < para_idx),
    );
    let shared_spacing =
        crate::renderer::float_placement::stored_after_partial_table_shared_spacing_px(
            (st.profile.hwpx_stored_layout() || st.profile.hwp5_stored_pagination_layout())
                && !st.profile.session_edited(),
            previous_is_partial_table,
            para,
            fmt.spacing_before,
            st.current_height,
            dpi,
        );
    st.reclaim_flow_by(shared_spacing.min(st.current_height));
    // 표/저장 프레임의 후속 원점과 공유 간격을 확정한 뒤 물리 빈 꼬리를 잰다.
    // 구역 진입의 옛 커서를 재사용하면 표 뒤 빈 줄의 실제 위치를 놓친다.
    st.prepare_saved_empty_run(para_idx, paragraphs, dpi, |st| {
        crate::renderer::typeset::notes::footnotes::measure::BodyFootnoteMeasure { engine, styles }
            .stored_current_area_height(st, paragraphs)
    });
    paragraph::spacing::confirm_split_successor(st, para_idx, para, fmt, paragraphs);
    let entry_format = paragraph::spacing::confirmed_format(st, para_idx, fmt, paragraphs);
    let entry_format = entry_format.as_ref().unwrap_or(fmt);
    let paragraph::FitBudget {
        strict_after_empty_host_float,
        layout_drift_safety_px,
        split_safety,
        prev_is_partial_table,
        mut available,
        footnote_margin_addback,
    } = paragraph::prepare_fit_budget(
        st,
        para_idx,
        para,
        entry_format,
        paragraphs,
        engine.profile.get().session_edited(),
        dpi,
    );

    if paragraph::try_absorb_rowbreak_guide(
        st,
        prev_is_partial_table,
        para,
        paragraphs,
        para_idx,
        dpi,
    ) {
        return;
    }

    let early_format = paragraph::spacing::confirmed_format(st, para_idx, fmt, paragraphs);
    let early_format = early_format.as_ref().unwrap_or(fmt);
    if paragraph::try_place_multicolumn_paragraph(st, para_idx, para, early_format, dpi) {
        return;
    }

    if paragraph::try_absorb_empty_paragraph(
        st,
        para_idx,
        para,
        early_format,
        is_last_in_section,
        available,
        layout_drift_safety_px,
    ) {
        return;
    }

    // 뒤 호스트의 절대 프레임이 저장 본문 재시작을 닫으면 분할 전에 소유를 예약한다.
    // 현재 글줄 앵커까지 맞아야 로컬 되감김을 새 쪽으로 승격하지 않는다.
    if st.col_count == 1
        && (st.profile.hwp5_stored_pagination_layout() || st.profile.hwpx_stored_layout())
        && !st.profile.session_edited()
        && fmt.line_heights.len() == para.line_segs.len()
    {
        for break_line in 1..para.line_segs.len() {
            let line = &para.line_segs[break_line];
            if line.vertical_pos >= para.line_segs[break_line - 1].vertical_pos {
                continue;
            }
            let Some((owner, ci, placement)) =
                crate::renderer::float_placement::stored_future_paper_shape_frame(
                    &paragraphs[para_idx + 1..],
                    line,
                    &st.layout.body_area,
                    st.layout.page_width,
                    dpi,
                )
            else {
                continue;
            };
            let owner = para_idx + 1 + owner;
            if st.has_spilled_page_tail_float(owner, ci) {
                continue;
            }
            let mut page = st.paragraph_line_scan_page();
            page.next_page_stored_body_origin = Some(placement.occupied_bottom);
            let base = st.vpos_page_base.or(st.vpos_lazy_base).unwrap_or(0);
            if paragraph::scan::stored_body_reset_fragment_matches_current_flow(
                &page,
                para,
                0,
                break_line,
                base,
                fmt.spacing_before,
                dpi,
            ) {
                st.defer_stored_frame(crate::renderer::typeset::DeferredStoredFrameControl {
                    following_body_para_index: None,
                    kind: crate::renderer::typeset::DeferredStoredFrameKind::AbsoluteShape,
                    para_index: owner,
                    control_index: ci,
                    placement,
                });
                break;
            }
        }
    }
    let paragraph::ForcedPageBoundary {
        native_hwp5_existing_footnote_boundary,
        current_page_vpos_base,
        forced_page_break_line,
        anchored_stored_body_reset_line,
    } = paragraph::prepare_forced_page_boundary(
        st,
        para_idx,
        para,
        fmt,
        paragraphs,
        available,
        &crate::renderer::typeset::notes::footnotes::measure::BodyFootnoteMeasure {
            engine,
            styles,
        },
    );
    // 저장 원점과 현재 흐름이 함께 인증한 본문 prefix는 같은 각주 paint 상자를 쓴다.
    // 이 prefix에 새 각주 참조가 없으면 기존 선택 조각만 예약하면 된다. 이미 확정된
    // reset 앞 줄의 수용과 실제 배치가 같은 물리 경계를 소비하도록 한다.
    if st.col_count == 1
        && !st.profile.session_edited()
        && (st.profile.hwp5_stored_pagination_layout() || st.profile.hwpx_stored_layout())
    {
        if let Some(end) =
            anchored_stored_body_reset_line.filter(|&end| end > 0 && end < fmt.line_count())
        {
            let next_text_start = para.utf16_pos_to_char_idx(para.line_segs[end].text_start);
            let positions = para.control_text_positions();
            let prefix_has_no_new_note =
                para.controls.iter().enumerate().all(|(index, control)| {
                    matches!(control, crate::model::control::Control::Footnote(_))
                        && positions
                            .get(index)
                            .is_some_and(|&position| position >= next_text_start)
                });
            if prefix_has_no_new_note {
                let measure =
                    crate::renderer::typeset::notes::footnotes::measure::BodyFootnoteMeasure {
                        engine,
                        styles,
                    };
                if let Some(height) = measure.stored_current_area_height(st, paragraphs) {
                    let prefix_bottom = st.current_height
                        + fmt.spacing_before
                        + fmt.line_advances_sum(0..end - 1)
                        + fmt.line_heights[end - 1];
                    let physical_bottom = st.base_available_height()
                        - height
                        - st.current_zone_y_offset
                        - st.current_bottom_fixed_exclusion;
                    if prefix_bottom <= physical_bottom {
                        let previous = st.available_height();
                        st.reserve_painted_footnote_area(height);
                        // 진입에서 소비한 1회 각주 여백 환급도 함께 정산한다.
                        // float·본문 여유 등 다른 예약은 기존 예산에 그대로 남는다.
                        available = (available - footnote_margin_addback + st.available_height()
                            - previous)
                            .max(0.0);
                    }
                }
            }
        }
    }
    if st.prefilled_line_prefixes.contains_key(&para_idx) {
        // 앞 쪽의 선행 조각이 소유한 줄을 whole-fit으로 다시 배치하지 않는다.
        let base_available = st.base_available_height();
        paragraph::place_split_paragraph(
            st,
            para_idx,
            para,
            fmt,
            paragraphs,
            styles,
            fmt.line_count(),
            base_available,
            split_safety,
            forced_page_break_line,
            native_hwp5_existing_footnote_boundary,
            current_page_vpos_base,
            false,
            strict_after_empty_host_float,
            dpi,
        );
        return;
    }
    let current_format = paragraph::spacing::confirmed_format(st, para_idx, fmt, paragraphs);
    let current_format = current_format.as_ref().unwrap_or(fmt);
    let paragraph::metrics::ParagraphFlowHints {
        body_bottom_vpos,
        trim_spacing_before_for_flow,
        trimmed_sb_gate,
    } = paragraph::metrics::flow_hints(
        para,
        current_format,
        paragraphs,
        para_idx,
        st.paragraph_flow_profile(),
        dpi,
    );

    let paragraph::WholeFitDecision {
        fits,
        stored_vpos_rewind_break,
        stored_vpos_rewind_overflow_break,
    } = paragraph::decide_whole_fit(
        st,
        para_idx,
        para,
        fmt,
        paragraphs,
        styles,
        strict_after_empty_host_float,
        forced_page_break_line,
        current_page_vpos_base,
        available,
        dpi,
    );
    if fits {
        paragraph::place_fitted_paragraph(
            st,
            para_idx,
            para,
            fmt,
            paragraphs,
            styles,
            trim_spacing_before_for_flow,
            trimmed_sb_gate,
            body_bottom_vpos,
            dpi,
        );
        return;
    }

    if paragraph::try_place_overflow_paragraph(
        st,
        para_idx,
        para,
        fmt,
        paragraphs,
        styles,
        trim_spacing_before_for_flow,
        body_bottom_vpos,
        available,
        forced_page_break_line,
        // 저장 쪽 경계로 전체 fit을 거부했다면 atomic spill이 그 경계를 덮지 않는다.
        stored_vpos_rewind_break || stored_vpos_rewind_overflow_break,
        current_page_vpos_base,
        dpi,
    ) {
        return;
    }

    paragraph::place_after_failed_fit(
        st,
        para_idx,
        para,
        fmt,
        paragraphs,
        styles,
        trim_spacing_before_for_flow,
        trimmed_sb_gate,
        body_bottom_vpos,
        available,
        strict_after_empty_host_float,
        split_safety,
        stored_vpos_rewind_break,
        stored_vpos_rewind_overflow_break,
        forced_page_break_line,
        native_hwp5_existing_footnote_boundary,
        current_page_vpos_base,
        dpi,
    );
}

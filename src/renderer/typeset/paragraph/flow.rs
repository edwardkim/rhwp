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
    match place_with_storage(st, input, engine, &crate::storage::Unbounded) {
        Ok(value) => value,
        Err(never) => match never {},
    }
}

pub(in crate::renderer::typeset) fn place_with_storage<S: crate::storage::Storage>(
    st: &mut TypesetState,
    input: ParagraphFlowInput<'_>,
    engine: &TypesetEngine,
    storage: &S,
) -> Result<(), S::Error> {
    let dpi = engine.dpi;
    let ParagraphFlowInput {
        para_idx,
        para,
        fmt,
        paragraphs,
        styles,
        is_last_in_section,
    } = input;
    collect_own_line_notes(st, engine, para_idx, para, fmt, styles);
    let stored_frame_shared_spacing =
        crate::renderer::float_placement::stored_frame_successor_shared_spacing_px(
            &st.paragraph_float_placements,
            para_idx,
            fmt.spacing_before,
            st.current_height,
        );
    let stored_frame_shared_spacing = if stored_frame_shared_spacing > 0.0 {
        stored_frame_shared_spacing
    } else {
        crate::renderer::float_placement::stored_visible_host_successor_spacing(
            &st.current_items,
            paragraphs,
            para_idx,
            fmt.spacing_before,
            (st.profile.hwpx_stored_layout() || st.profile.hwp5_stored_pagination_layout())
                && !st.profile.session_edited(),
            st.current_height,
            dpi,
        )
    };
    // [#8149] 인증 배치가 없는 빈 host 자리차지 표 상자 뒤도 같은 앞 간격 소비를 따른다.
    let stored_frame_shared_spacing = if stored_frame_shared_spacing > 0.0 {
        stored_frame_shared_spacing
    } else {
        crate::renderer::float_placement::empty_host_float_successor_absorbed_spacing_px(
            st.current_items.last(),
            paragraphs,
            para_idx,
            fmt.spacing_before,
            (st.profile.hwpx_stored_layout() || st.profile.hwp5_stored_pagination_layout())
                && !st.profile.session_edited(),
            st.profile.hwpx_container(),
            st.profile.hwp5_origin_hwpx(),
            st.current_height,
            dpi,
        )
    };
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
    } = paragraph::prepare_fit_budget_with_storage(
        st,
        para_idx,
        para,
        entry_format,
        paragraphs,
        engine.profile.get().session_edited(),
        dpi,
        storage,
    )?;

    if paragraph::try_absorb_rowbreak_guide_with_storage(
        st,
        prev_is_partial_table,
        para,
        paragraphs,
        para_idx,
        dpi,
        storage,
    )? {
        return Ok(());
    }

    let early_format = paragraph::spacing::confirmed_format(st, para_idx, fmt, paragraphs);
    let early_format = early_format.as_ref().unwrap_or(fmt);
    if paragraph::try_place_multicolumn_paragraph_with_storage(
        st, para_idx, para, fmt, dpi, storage,
    )? {
        return Ok(());
    }

    if paragraph::try_absorb_empty_paragraph_with_storage(
        st,
        para_idx,
        para,
        early_format,
        is_last_in_section,
        available,
        layout_drift_safety_px,
        storage,
    )? {
        return Ok(());
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
                        // float·본문 여유 등 다른 예약은 기존 예산에 그대로 남는다.
                        available = (available + st.available_height() - previous).max(0.0);
                    }
                }
            }
        }
    }
    if st.prefilled_line_prefixes.contains_key(&para_idx) {
        // 앞 쪽의 선행 조각이 소유한 줄을 whole-fit으로 다시 배치하지 않는다.
        let base_available = st.base_available_height();
        paragraph::place_split_paragraph_with_storage(
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
            storage,
        )?;
        return Ok(());
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

    // 마지막 줄 앞의 줄이 참조한 각주는 그 뒤 본문 줄보다 먼저 쪽 아래를 차지한다.
    // 마지막 줄의 각주는 들어가지 않으면 다음 쪽으로 이어지므로 통째 fit 에서 빼지 않는다.
    let last_line = fmt.line_count().saturating_sub(1);
    let mut earlier_notes: Vec<_> = st
        .own_line_notes
        .iter()
        .filter(|note| note.para_index == para_idx && note.line < last_line)
        .cloned()
        .collect();
    earlier_notes.sort_by_key(|note| (note.line, note.number));
    let note_levels = st.own_line_note_levels(&earlier_notes, |line| {
        available
            - st.current_height
            - fmt.spacing_before
            - fmt.line_advances_sum(0..line)
            - fmt.line_heights.get(line).copied().unwrap_or(0.0)
    });
    let own_note_cost = note_levels.last().map_or(0.0, |level| {
        if note_levels.iter().any(|level| level.blocks_later_lines) {
            // 앞부분만 들어가는 각주 뒤 줄은 이 쪽에 들어가지 않는다 — 분할로 넘긴다.
            f64::INFINITY
        } else {
            level.cost
        }
    });
    let whole_available = (available - own_note_cost).max(0.0);
    if std::env::var_os("RHWP_DIAG_OWN_NOTES").is_some() && own_note_cost > 0.0 {
        eprintln!(
            "DIAG_OWN_WHOLE pi={para_idx} cost={own_note_cost:.1} available={whole_available:.1} current={:.1} fit_h={:.1} total={:.1}",
            st.current_height, fmt.height_for_fit, fmt.total_height
        );
    }
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
        whole_available,
        dpi,
    );
    if fits {
        paragraph::place_fitted_paragraph_with_storage(
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
            storage,
        )?;
        st.register_own_line_notes_with_storage(
            para_idx,
            0..fmt.line_count(),
            (fmt.total_height - fmt.height_for_fit).max(0.0),
            storage,
        )?;
        return Ok(());
    }

    if paragraph::try_place_overflow_paragraph_with_storage(
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
        storage,
    )? {
        return Ok(());
    }

    paragraph::place_after_failed_fit_with_storage(
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
        storage,
    )?;

    Ok(())
}

/// 문단의 본문 각주와 참조 줄을 모은다. 줄 대응은 저장 줄 상자가 측정 줄과
/// 일대일일 때만 쓴다(재조판 줄은 저장 text_start 와 다른 컷을 가진다).
fn collect_own_line_notes(
    st: &mut TypesetState,
    engine: &TypesetEngine,
    para_idx: usize,
    para: &Paragraph,
    fmt: &FormattedParagraph,
    styles: &ResolvedStyleSet,
) {
    let mut notes = Vec::new();
    if fmt.line_count() == 0 || fmt.line_count() != para.line_segs.len() {
        st.replace_own_line_notes(notes);
        return;
    }
    let stored_profile =
        st.profile.hwpx_stored_layout() || st.profile.hwp5_stored_pagination_layout();
    let measure =
        crate::renderer::typeset::notes::footnotes::measure::BodyFootnoteMeasure { engine, styles };
    for (control_index, control) in para.controls.iter().enumerate() {
        let crate::model::control::Control::Footnote(footnote) = control else {
            continue;
        };
        if stored_profile
            && !st.profile.session_edited()
            && crate::renderer::stored_footnote_is_bodyless(footnote)
        {
            continue;
        }
        let Some(line) = crate::renderer::layout::control_line_seg_index(para, control_index)
        else {
            continue;
        };
        let content_height = measure
            .block_content_height(st, footnote, None)
            .unwrap_or_else(|| measure.content_height(st, footnote));
        let mut lines = Vec::new();
        for note_para in &footnote.paragraphs {
            let composed = crate::renderer::composer::compose_paragraph(note_para);
            if composed.lines.is_empty() || composed.lines.len() != note_para.line_segs.len() {
                lines.clear();
                break;
            }
            lines.extend(composed.lines.iter().map(|line| {
                (
                    crate::renderer::hwpunit_to_px(line.line_height, engine.dpi),
                    crate::renderer::hwpunit_to_px(line.line_spacing, engine.dpi),
                )
            }));
        }
        let stored_split =
            crate::renderer::typeset::notes::footnotes::boundary::native_hwp5_footnote_reset_fragments(
                footnote, engine.dpi,
            )
            .filter(|split| !split.force_next_page);
        notes.push(crate::renderer::typeset::OwnLineNote {
            para_index: para_idx,
            line,
            control_index,
            number: footnote.number,
            content_height,
            lines,
            stored_split,
        });
    }
    st.replace_own_line_notes(notes);
}

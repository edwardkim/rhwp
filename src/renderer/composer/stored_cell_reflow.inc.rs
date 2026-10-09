/// Resolve stored rows when the caller has already proven that the paragraph
/// belongs to a non-TAC Square Picture/Shape wrap band.
///
/// A uniformly narrow stored ladder is not generally float evidence: ordinary
/// paragraph indentation has the same local shape. Only the pagination and
/// render paths that carry a real Square-wrap anchor may preserve that ladder
/// as externally owned geometry.
#[allow(clippy::too_many_arguments)]
pub(crate) fn recompose_stored_lines_in_frame_with_known_square_band(
    composed: &ComposedParagraph,
    para: &Paragraph,
    paragraph_box: ParagraphBox,
    inner_width_px: f64,
    styles: &ResolvedStyleSet,
    dpi: f64,
    stored_geometry_policy: impl Into<crate::model::provenance::StoredGeometryPolicy>,
    miss_policy: line_breaking::StoredRowMissPolicy,
    // [#6175] 용지/쪽 기준 float 증거는 uniform ladder의 결손 폭과 세로 band를
    // 함께 설명할 때만 저장 기하를 유지한다.
    float_carve_evidence: &[crate::renderer::float_placement::FloatCarveEvidence],
    known_square_band: bool,
    hwpx_stored_layout: bool,
    // 구역 전체가 합성 저장 사다리이면 행 경계도 현재 프레임에서 다시 만든다.
    stored_rows_untrusted: bool,
    // 현대 NO_LS 셀의 측정 소유자가 전달한 실측 표 높이다.
    // 이 증거가 없는 경로의 표 배치 소유권은 바꾸지 않는다.
    measured_inline_table_heights: &[(usize, u32)],
) -> Option<ComposedParagraph> {
    let stored_geometry_policy = stored_geometry_policy.into();
    // loader가 만든 합성 줄은 원본 저장 캐시가 아니다. 독립 개체가 없는
    // 본문은 현재 프레임에서 다시 구성하며 원본 IR은 보존한다.
    let fresh_body = (!para.line_segs.is_empty()
        && para.controls.is_empty()
        && crate::renderer::para_has_no_stored_line_segs(para))
    .then(|| {
        let mut body = para.clone();
        body.replace_line_segs(Vec::new());
        body
    });
    let para = fresh_body.as_ref().unwrap_or(para);
    // A degenerate box, or controls with their own layout owner, means there is
    // no frame to build. The composition stands as it is — there is no second
    // owner to hand it to.
    //
    // NO_LS 문단은 예외적으로 picture-band 게이트(비-TAC 그림 1개)도 허용한다 —
    // 저장 행이 없어 fill 이 유일한 소유자인데, Square 그림 host 라는 이유로
    // 여기서 사양하면 45자 합성 줄바꿈이 그대로 남아 감폭된 상자 폭을 넘는다
    // (아이콘 옆 설명 한 줄이 열 밖까지 이어지는 형상). 저장 행이 있는 문단의
    // 소유권 계약은 종전대로 본문 게이트만 통과한다.
    let has_kopub_face = para.char_shapes.iter().any(|reference| {
        styles
            .char_styles
            .get(reference.char_shape_id as usize)
            .is_some_and(|style| {
                style.font_families.iter().any(|face| {
                    face.contains("KoPub돋움체")
                        || face.contains("KoPub바탕체")
                        || face.to_lowercase().contains("kopub dotum")
                        || face.to_lowercase().contains("kopub batang")
                })
            })
    });
    let justified_kopub = hwpx_stored_layout
        && has_kopub_face
        && styles
            .para_styles
            .get(para.para_shape_id as usize)
            .is_some_and(|style| style.alignment == crate::model::style::Alignment::Justify);
    let justified_kopub_picture = justified_kopub
        && para.controls.iter().any(|control| {
            matches!(control, crate::model::control::Control::Picture(picture)
                if picture.common.treat_as_char)
        });
    let single_tac_table_with_text = para.text.chars().any(|c| !c.is_whitespace())
        && para
            .controls
            .iter()
            .filter(
                |control| matches!(control, Control::Table(table) if table.common.treat_as_char),
            )
            .count()
            == 1;
    let frame_admits_controls = line_breaking::supports_cached_body_frame_controls(para)
        || inline_frame_admission::supports_uncached_inline_host(para)
        || (justified_kopub_picture
            && para.controls.iter().all(|control| {
                matches!(control, crate::model::control::Control::Picture(picture)
                    if picture.common.treat_as_char)
            }))
        || (crate::renderer::para_has_no_stored_line_segs(para)
            && (known_square_band
                || para.controls.iter().any(|control| {
                    matches!(control, crate::model::control::Control::Picture(picture)
                    if !picture.common.treat_as_char
                    && picture.common.text_wrap == crate::model::shape::TextWrap::Square)
                }))
            && line_breaking::supports_picture_band_frame_controls(para))
        // 실측 인라인 표의 현재 프레임은 글자 없는 host와 여러 표도 구성한다.
        // 나머지 경로는 기존의 보이는 글자+단일 TAC 표 수용 계약을 유지한다.
        || ((!measured_inline_table_heights.is_empty() || single_tac_table_with_text)
            && line_breaking::supports_tac_table_band_frame_controls(para));
    if !paragraph_box.is_usable() || !frame_admits_controls {
        return None;
    }

    // 저장 원점 복원은 측정·배치와 읽기 전용 증거 프로브가 같은 상자를 사용한다.
    let paragraph_box = if stored_geometry_policy.requires_realign {
        paragraph_box
    } else {
        paragraph_box.for_stored_body_rows(para)
    };
    let mut frame = paragraph_box.frame(
        para.line_segs
            .first()
            .map(|segment| segment.vertical_pos)
            .unwrap_or(0),
    );
    frame.kopub_justified_space = justified_kopub_picture;
    // 일반 본문은 render-normalization이 새 줄과 후속 원점을 함께 발행한다.
    // 셀 및 별도 그림 공백 메트릭의 기존 계약만 이 지역 프레임에서 해소한다.
    let regenerated_spaces_change_metrics = frame.kopub_justified_space
        && stored_word_tail::regenerated_spaces_change_metrics(
            composed,
            styles,
            SpaceMetric::KoPubJustified,
        );
    // 캐시 수용 실패 정책이나 줄 수 감소만으로 원본을 무효화하지 않는다.
    // 본문과 같은 마커 기하 갱신과 실제 공백 메트릭 변경만 공통 프레임에 요청한다.
    let refreshes_marker_metrics =
        stored_word_tail::projects_current_marker_geometry(para, &paragraph_box, styles);
    let rejoinable_word_tail = !stored_geometry_policy.preserve_hwp3_origins
        && (regenerated_spaces_change_metrics || refreshes_marker_metrics)
        && stored_word_tail::can_rejoin_stored_word_tail(
            para,
            &frame,
            styles,
            dpi,
            justified_kopub_picture,
        );
    // 원본의 재정렬 요청은 일반 저장 줄의 압축 허용치와 별개로 수용을 거부한다.
    let stale = stored_geometry_policy.requires_realign
        || (!para.layout_field_values.is_empty()
            && (para.stored_text_partition_dirty
                || crate::renderer::para_has_no_stored_line_segs(para)))
        || rejoinable_word_tail
        || (stored_rows_untrusted && !para.line_segs.is_empty())
        || stored_rows_are_stale(composed, para, inner_width_px, styles)
        || line_breaking::remeasured_equation_start_row(para).is_some()
        || (stored_geometry_policy.stored_rows_before_hancom_2020
            && stored_last_row_is_rejoinable_word_tail(composed, para, styles)
            && fresh_rows_absorb_stored_tail(
                para,
                &frame,
                styles,
                dpi,
                stored_geometry_policy.preserve_hwp3_origins,
                miss_policy,
                float_carve_evidence,
                known_square_band,
            ));
    match line_breaking::resolve_stored_line_segs_in_frame(
        para,
        &mut frame,
        styles,
        dpi,
        stored_geometry_policy.preserve_hwp3_origins,
        miss_policy,
        stale,
        float_carve_evidence,
        known_square_band,
    ) {
        // Stale — the row cannot hold its own text — so the rebuilt row is the
        // frame's. Its fill tokenizes through `para.char_shapes`
        // (`line_breaking.rs`), which is the char-shape re-splitting #2632
        // needed, so there is nothing the retired #2525 owner did that the
        // frame does not.
        Some(line_breaking::StoredRowResolution::Reflowed) => {
            // The frame holds the rows; `project_line_segs` is the one place
            // they become `LineSeg` again.
            let mut reflowed_para = para.clone();
            reflowed_para.replace_line_segs_with_space_metrics(
                frame.project_line_segs(),
                frame.project_space_metrics(),
            );
            // [#6102] 프레임이 새로 새긴 행 경계는 이미 HWP5 문단 축이다 —
            // 원본의 [#5961] 보정폭을 물려받으면 fresh 경계가 이중 보정되어
            // 줄이 보정폭만큼 늦게 끊긴다(36360328: fill 이 char 51(=raw 83)에
            // 끊었는데 +8 재보정으로 59가 되어 첫 줄이 우단 밖 +75px).
            reflowed_para.hwpx_axis_shift = 0;
            let mut reflowed = compose_paragraph_in_context(&reflowed_para, styles);
            preserve_context_resolved_runs(composed, &mut reflowed);
            // 부분 재조판 앞행은 원래 저장 축과 이미 해석한 줄 소유를 보존한다.
            // HWPX의 저장 축 보정을 새 HWP5 축의 뒤쪽에 다시 적용하지 않는다.
            if let Some(prefix_len) = line_breaking::remeasured_equation_start_row(para) {
                if prefix_len <= composed.lines.len() && prefix_len <= reflowed.lines.len() {
                    reflowed.lines[..prefix_len].clone_from_slice(&composed.lines[..prefix_len]);
                }
            }
            let mut reconciled = composed.clone();
            reconciled.lines = reflowed.lines;
            if rejoinable_word_tail || !measured_inline_table_heights.is_empty() {
                reconciled.frame_geometry =
                    Some(std::sync::Arc::new(frame_geometry::FrameGeometry {
                        line_segs: reflowed_para.line_segs,
                        space_metrics: reflowed_para.layout_space_metrics,
                        inline_table_heights: measured_inline_table_heights.to_vec(),
                    }));
            }
            // Q2-D5-N1: source NO_LS has no usable width during the initial
            // composition handoff. The physical frame is the first owner that
            // knows the final intervals, so prepare Q2-C again only after that
            // partition is complete. Stored-row D4 outcomes stay untouched.
            if crate::renderer::para_has_no_stored_line_segs(para) {
                reconciled.horizontal_shaping =
                    line_breaking::compose_horizontal_shaping_handoff(para, &reconciled, styles);
            }
            Some(reconciled)
        }
        // `Stored`: returning the composition unchanged is the answer, not a
        // gap. The frame has already reproduced the stored physical-row key
        // exactly. The metrics lane remains deliberately unpublished
        // (§1.4.1's accept-arm write-back — see
        // `StoredRowResolution::Stored` and `LayoutFrame::try_admit_stored_rows`
        // for the measurement). `None`: the paragraph's controls have their own
        // layout owner and no frame was ever built for it.
        other => None,
    }
}

/// [#7702] 한/글 2020 이전 제품이 저장한 줄이 condense 낱말 흡수 없이 끊겼는가.
///
/// 같은 프레임에서 저장 줄을 버리고 다시 끊어 본다. 새 줄 나눔이 저장의 **마지막 줄만 없애고**
/// 나머지 줄의 시작은 그대로 두면 마지막 낱말 꼬리를 앞 줄이 흡수한 것이다 — 한/글 2020
/// 인쇄가 그렇게 끊는다(`1342000`, 한/글 2014 저장: 151쪽 `…순서에 맞게 시연하|기`,
/// 154쪽 `…찬반 토|론`). 판정과 실제 줄은 같은 프레임 결과다.
///
/// 앞 줄 경계까지 옮기는 새 줄 나눔은 받지 않는다. 같은 문서 114쪽 `[12융과04-02]` 칸은
/// 새로 끊으면 `…정보|가` 까지 앞 줄로 당기지만 한/글 2020 은 저장 줄 넷을 모두 따른다 —
/// 꽉 찬 줄 경계에서 우리 condense 판정이 한/글보다 한 글자 너그럽다. 근거가 있는 것은
/// 마지막 꼬리뿐이다.
///
/// 한/글 2020 으로 저장한 문서는 같은 모양이어도 묻지 않는다 — 한/글이 저장 줄을 따른다
/// (`21_언어_기출_편집가능본.hwp` 11.0.0.2129 1쪽 `…확인하|십시오.`,
/// `36388711…청년오피스.hwpx` 11.0.0.8227 1쪽 `…과학|기술(150)`).
#[allow(clippy::too_many_arguments)]
fn fresh_rows_absorb_stored_tail(
    para: &Paragraph,
    frame: &crate::renderer::layout_frame::LayoutFrame,
    styles: &ResolvedStyleSet,
    dpi: f64,
    legacy_hwp3_stored_geometry: bool,
    miss_policy: line_breaking::StoredRowMissPolicy,
    float_carve_evidence: &[crate::renderer::float_placement::FloatCarveEvidence],
    known_square_band: bool,
) -> bool {
    let mut trial = frame.clone();
    matches!(
        line_breaking::resolve_stored_line_segs_in_frame(
            para,
            &mut trial,
            styles,
            dpi,
            legacy_hwp3_stored_geometry,
            miss_policy,
            true,
            float_carve_evidence,
            known_square_band,
        ),
        Some(line_breaking::StoredRowResolution::Reflowed)
    ) && {
        let fresh = trial.project_line_segs();
        fresh.len() + 1 == para.line_segs.len()
            && fresh
                .iter()
                .zip(&para.line_segs)
                .all(|(fresh, stored)| fresh.text_start == stored.text_start)
    }
}

/// Read-only projection of the production stored-row cache decision.
///
/// Evidence queries use this entry instead of inferring validity from the mere
/// presence of `LineSeg` records. The probe owns an isolated composition and
/// frame, so it cannot publish rows or mutate the document. `Unmodelled` is a
/// first-class answer: legacy origins, externally-owned wrap geometry and
/// unsupported controls must not be mislabeled as cache rejection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StoredRowProbeDisposition {
    Admitted,
    Rejected,
    Unmodelled,
    NotApplicable,
}

impl StoredRowProbeDisposition {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Admitted => "admitted",
            Self::Rejected => "rejected",
            Self::Unmodelled => "unmodelled",
            Self::NotApplicable => "notApplicable",
        }
    }
}

pub(crate) fn probe_stored_row_disposition(
    para: &Paragraph,
    paragraph_box: ParagraphBox,
    styles: &ResolvedStyleSet,
    dpi: f64,
    stored_geometry_policy: impl Into<crate::model::provenance::StoredGeometryPolicy>,
    miss_policy: line_breaking::StoredRowMissPolicy,
) -> StoredRowProbeDisposition {
    let stored_geometry_policy = stored_geometry_policy.into();
    let has_authoritative_rows = !para.line_segs.is_empty()
        && !para
            .line_segs
            .iter()
            .all(|segment| segment.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0);
    if !has_authoritative_rows {
        return StoredRowProbeDisposition::NotApplicable;
    }
    if !paragraph_box.is_usable() {
        return StoredRowProbeDisposition::Unmodelled;
    }

    let composed = compose_paragraph_in_context(para, styles);
    let inner_width_px = paragraph_box.width_px(dpi);
    // 저장 원점 복원은 측정·배치와 읽기 전용 증거 프로브가 같은 상자를 사용한다.
    let paragraph_box = if stored_geometry_policy.requires_realign {
        paragraph_box
    } else {
        paragraph_box.for_stored_body_rows(para)
    };
    let mut frame = paragraph_box.frame(
        para.line_segs
            .first()
            .map(|segment| segment.vertical_pos)
            .unwrap_or(0),
    );
    let stale = stored_geometry_policy.requires_realign
        || stored_rows_are_stale(&composed, para, inner_width_px, styles)
        || (stored_geometry_policy.stored_rows_before_hancom_2020
            && stored_last_row_is_rejoinable_word_tail(&composed, para, styles)
            && fresh_rows_absorb_stored_tail(
                para,
                &frame,
                styles,
                dpi,
                stored_geometry_policy.preserve_hwp3_origins,
                miss_policy,
                &[],
                false,
            ));
    match line_breaking::resolve_stored_line_segs_in_frame(
        para,
        &mut frame,
        styles,
        dpi,
        stored_geometry_policy.preserve_hwp3_origins,
        miss_policy,
        stale,
        // 증거 프로브는 단 문맥을 갖지 않는다 — 개체 증거 없이 판정한다.
        &[],
        false,
    ) {
        Some(line_breaking::StoredRowResolution::Stored) => StoredRowProbeDisposition::Admitted,
        Some(line_breaking::StoredRowResolution::Reflowed) => StoredRowProbeDisposition::Rejected,
        None => StoredRowProbeDisposition::Unmodelled,
    }
}

/// Project context-resolved source runs onto Frame-computed line boundaries.
///
/// Header/footer fields and other model-one/display-many runs keep the marker in
/// `text` and the rendered value in `display_text`. Re-composing from `Paragraph`
/// alone cannot recreate document context such as the current filename or page
/// number, so a strict Frame reflow owns only the partition and row metrics; it
/// must not erase the already-resolved run payload.
fn preserve_context_resolved_runs(source: &ComposedParagraph, reflowed: &mut ComposedParagraph) {
    struct RunSpan<'a> {
        start: usize,
        end: usize,
        run: &'a ComposedTextRun,
    }

    // Only these payloads are owned by the already context-resolved source.
    // Style and language partitions remain the Frame composition's property.
    let mut context_spans = Vec::new();
    for line in &source.lines {
        let mut start = line.char_start;
        for run in &line.runs {
            let end = start.saturating_add(run.text.chars().count());
            if run.display_text.is_some()
                || run.footnote_marker.is_some()
                || run.char_overlap.is_some()
            {
                context_spans.push(RunSpan { start, end, run });
            }
            start = end;
        }
    }
    if context_spans.is_empty() {
        return;
    }

    for line in &mut reflowed.lines {
        let mut run_start = line.char_start;
        let mut overlaid = Vec::new();
        for run in std::mem::take(&mut line.runs) {
            let run_len = run.text.chars().count();
            let run_end = run_start.saturating_add(run_len);
            let overlapping: Vec<_> = context_spans
                .iter()
                .filter(|span| span.start < run_end && run_start < span.end)
                .collect();
            if overlapping.is_empty() {
                overlaid.push(run);
                run_start = run_end;
                continue;
            }

            let mut cuts = vec![run_start, run_end];
            for span in &overlapping {
                cuts.push(run_start.max(span.start));
                cuts.push(run_end.min(span.end));
            }
            cuts.sort_unstable();
            cuts.dedup();

            for bounds in cuts.windows(2) {
                let piece_start = bounds[0];
                let piece_end = bounds[1];
                if piece_start == piece_end {
                    continue;
                }

                let take_start = piece_start - run_start;
                let take_end = piece_end - run_start;
                let mut piece = run.clone();
                piece.text = run
                    .text
                    .chars()
                    .skip(take_start)
                    .take(take_end - take_start)
                    .collect();
                piece.display_text = run.display_for_model_range(take_start, take_end);
                if take_start != 0 || take_end != run_len {
                    piece.footnote_marker = None;
                    piece.char_overlap = None;
                }

                if let Some(span) = overlapping
                    .iter()
                    .find(|span| span.start <= piece_start && piece_end <= span.end)
                {
                    let source_start = piece_start - span.start;
                    let source_end = piece_end - span.start;
                    piece.display_text = span.run.display_for_model_range(source_start, source_end);
                    let owns_whole_payload = piece_start == span.start && piece_end == span.end;
                    piece.footnote_marker = owns_whole_payload
                        .then_some(span.run.footnote_marker)
                        .flatten();
                    piece.char_overlap = owns_whole_payload
                        .then(|| span.run.char_overlap.clone())
                        .flatten();
                }
                overlaid.push(piece);
            }
            run_start = run_end;
        }
        line.runs = overlaid;
    }
}

/// The cell's rebuild, resolved through the physical frame.
///
/// This is not the cell twin of the retired body owner. Every authentic
/// partition enters the shared Frame resolver: an exact cell-box match stays
/// stored, while changed geometry or mutation-dirty text falls through to
/// reflow. NO_LS takes the same owner directly. Both body and cell state a
/// [`ParagraphBox`] instead of passing a bare width.
///
/// Cells are frames. §2.10 item 3 proves it exhaustively: enumerating all 2,057
/// RTTI object locators, only three `CHwpList` subclasses return non-zero from
/// `vtable+8`, and `CHwpCell` is not one of them — so **every** nested flow
/// (cell, note, text box, caption) runs the same generic engine on the stack,
/// re-entrantly. §2.13 fixes what that frame looks like: a cell's rect is
/// cell-local, `left = 0`, `width = engine+0x40 = entry+0x10 − (marginLeft +
/// marginRight)`. That is exactly [`ParagraphBox::content`] over
/// `0..cell_inner_width`, which every call site now builds with
/// [`ParagraphBox::content_width_px`], and it is why the content arm takes no
/// geometry pitch (§2.10 item 5: for a nested list `ConfigureFrameBounds`
/// returns straight after setting the rect — the column-continuation state is
/// body-only).
///
/// The retired width-based rebuild is gone rather than kept beside this. It was
/// load-bearing for 15 behaviours and the frame reproduces **12** of them.
/// Three oracles were disturbed, not one, and they were not disturbed the same
/// way — an earlier revision of this note said "14 of 15" and described all
/// three as re-pinned, which the tests contradict:
///
/// - `issue_2308_saved_nested_width_keeps_fragment_geometry` — re-pinned to
///   this path's values, PDF row counts in its comment. The authority PDF
///   adjudicates **against the retired path**.
/// - `issue_2308_short_rowbreak_child_wraps_where_the_authority_pdf_wraps` —
///   `#[ignore]`d. The PDF adjudicates **against this path**: the frame carries
///   `를` onto p81 with 210 HWPUNIT (0.55% of the box) of headroom, and the
///   retired path landed on the PDF's break only because `char_shapes[0]`
///   blindness inflated the row by 585 HWPUNIT. The residual is recorded with
///   its operands rather than re-pinned. Its sibling
///   `issue_2308_short_rowbreak_child_uses_owner_content_box_only` holds the
///   part this path does satisfy and **runs**.
/// - `issue_2279_nested_cell_units_split_r27_not_r26` — `#[ignore]`d, and
///   **not** re-pinned. The adjudication is split here: the PDF prints that
///   cell as 4 lines, so it backs *this* path's row height and the retired
///   path's 5-line measure was wrong — but the page-cut expectation the pin
///   asserts was standing on that wrong measure, and is itself PDF-correct.
///   Removing the compensating error exposes an upstream difference that is
///   #2279/#2308's to answer, so the pin stays as written and waits.
///
/// Measured with the suite at 5986/0 when the routing landed.
///
/// **What this changes, and the one thing it costs.** `compose_lines`' NO_LS
/// fallback emits one run at `char_shapes[0]`, so the retired rebuild measured
/// and painted every character of a paragraph with the *first* char shape. The
/// frame's fill tokenizes through `para.char_shapes` instead. On
/// `76076_regulatory_analysis.hwp` p81 (`구내운반차` short RowBreak child, box
/// 38245 HWPUNIT) the shapes are all 13.0pt but differ in **letter spacing**, so
/// the body glyphs advance 1261 where `char_shapes[0]` advances 1300. Measured
/// at the decisive tokens:
///
/// ```text
/// 를   pen 36735  glyph 1261 (fit 1300)  sum 38035  over  −210  → accepted
/// 예   pen 38673  glyph 1208 (fit 1299)  sum 39972  over +1727  → refused
/// ```
///
/// The HWP 2024 PDF breaks before `를`; this path carries it onto p81 with 210
/// HWPUNIT — 2.8px, **0.55% of the box** — of headroom. The retired path landed
/// on the PDF's break only by accident: measuring the row's 15 맑은 고딕 glyphs
/// at 1300 instead of 1261 adds 585 HWPUNIT, which is what pushed `를` over. A
/// 0.55% width-estimation residual is not something the retired path knew; that
/// is why the oracle is marked with its operands rather than re-pinned.
pub fn recompose_cell_lines_in_frame(
    composed: &mut ComposedParagraph,
    para: &Paragraph,
    cell_box: ParagraphBox,
    styles: &ResolvedStyleSet,
    dpi: f64,
    stored_geometry_policy: impl Into<crate::model::provenance::StoredGeometryPolicy>,
) {
    let stored_geometry_policy = stored_geometry_policy.into();
    // 개체 흐름을 분리한 정규화가 이미 같은 줄의 높이·간격을 발행했다.
    // 일반 합성 단일 줄처럼 캐시를 지우고 채우면 그림 뒤에 글자 줄간격을
    // 다시 더해 측정·조각 상자가 부푼다. 원본 저장 줄로 승격하지 않고 그대로 소비한다.
    if para.layout_object_projection && !para.stored_text_partition_is_dirty() {
        return;
    }
    let has_synthetic_line_segs = !para.line_segs.is_empty()
        && para
            .line_segs
            .iter()
            .all(|seg| seg.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0);
    let has_authoritative_line_segs = !para.line_segs.is_empty() && !has_synthetic_line_segs;
    if para.line_segs.len() >= 2
        && has_synthetic_line_segs
        && !para.stored_text_partition_is_dirty()
    {
        // HWPX 로드 단계에서 셀 폭/높이/anchor 속성으로 합성한 lineSeg 경계는
        // 이미 문서 속성 기반 보정 결과다. 여기서 다시 폭 기준으로 합치고
        // 재분할하면 RowBreak 표의 쪽 나눔 기준 줄 수가 원본 세로 정보와 어긋난다.
        return;
    }
    if composed.lines.is_empty() || !cell_box.is_usable() {
        return;
    }

    let inner_width_px = cell_box.width_px(dpi);
    let rebuilt = if has_authoritative_line_segs {
        // OWPML 셀은 현재 속성의 내용 프레임으로 저장 키의 불일치를 해소한다.
        // 원 HWP5의 clean miss는 아직 이 프레임이 소유하지 않는 저장 기하다.
        // 편집 무효화·재정렬 요청은 출처와 별개로 공통 stale 판정을 따른다.
        // 외부 어울림·다른 물리 쪽 등 계산 불가능한 경로는 resolver가 거절한다.
        // 같은 셀의 외부 어울림이 정한 원점을 현재 문단 상자로 재현할 수는 없다.
        // 폭의 소유권만으로 저장 원점까지 덮어쓰지 않는다(#5818 로고 옆 글줄).
        let cell_box = cell_box.for_stored_body_rows(para);
        let frame_origin = cell_box.effective().start;
        let owns_stored_origins = para
            .line_segs
            .iter()
            .all(|line| line.column_start == frame_origin);
        let miss_policy = if stored_geometry_policy.hwpx_cell_geometry && owns_stored_origins {
            line_breaking::StoredRowMissPolicy::Reflow
        } else {
            line_breaking::StoredRowMissPolicy::UnmodelledUnlessStale
        };
        recompose_stored_lines_in_frame(
            composed,
            para,
            cell_box,
            inner_width_px,
            styles,
            dpi,
            stored_geometry_policy,
            miss_policy,
            // 셀 줄의 어울림 배제는 #5818 셀 계약이 따로 소유한다.
            &[],
        )
    } else {
        // NO_LS is the arm this owner normally serves. A synthetic single row
        // is not a source record to reproduce, so clear it on an isolated copy.
        let mut rebuild_input = para.clone();
        rebuild_input.line_segs.clear();
        recompose_stored_lines_in_frame(
            composed,
            &rebuild_input,
            cell_box,
            inner_width_px,
            styles,
            dpi,
            stored_geometry_policy,
            line_breaking::StoredRowMissPolicy::Reflow,
            &[],
        )
    };
    if let Some(rebuilt) = rebuilt {
        *composed = rebuilt;
    }
    // [#6802] 저장 한 줄을 지킨 줄의 넘치는 채움(리더)은 여기서 끊는다 — 측정(높이)과
    // 배치(페인트)가 같은 이 함수를 타므로 두 경로가 같은 줄을 본다.
    if has_authoritative_line_segs && !para.stored_text_partition_is_dirty() {
        trim_leader_fill_overflow(composed, inner_width_px, styles);
    }
}

/// 저장 `LINE_SEG`가 없는 들여쓴 셀 문단의 구간별 자간을 안전하게
/// 복원할 수 있는지 판정한다.
///
/// 셀 fallback은 역사적으로 첫 글자모양 하나로 재조판한다. 서로 다른 글꼴 크기까지
/// 일괄 복원하면 검증된 legacy pagination이 바뀌므로, 폭을 결정하는 글꼴·크기·장평은
/// 같고 자간만 달라지는 문단에 한해 실제 `CharShapeRef` 경계를 사용한다.
///
/// 추가로 문단 머리가 둘 이상의 literal ASCII 공백인 경우로 한정한다.
/// 이 형식은 한컴이 반각 들여쓰기 칸과 구간별 음수 자간을 줄 채움에
/// 함께 반영하는 native HWP5 장문 셀이다(#3128). 반면 글머리 문자로
/// 시작하는 짧은 child는 기존 owner-content-box 폭 계약을 계속 사용한다
/// (76076 p81→p82 `사고` / `를 예방…`).
pub(crate) fn missing_lineseg_indented_cell_has_uniform_metrics_with_tracking(
    para: &Paragraph,
    styles: &ResolvedStyleSet,
) -> bool {
    if !para.line_segs.is_empty()
        || para.char_shapes.len() < 2
        || para.text.chars().take_while(|value| *value == ' ').count() < 2
    {
        return false;
    }

    let mut resolved = para
        .char_shapes
        .iter()
        .filter_map(|char_shape| styles.char_styles.get(char_shape.char_shape_id as usize));
    let Some(first) = resolved.next() else {
        return false;
    };
    let first_tracking = &first.letter_spacings;
    let mut tracking_differs = false;

    for style in resolved {
        if style.font_family != first.font_family
            || style.font_families != first.font_families
            || (style.font_size - first.font_size).abs() > f64::EPSILON
            || style.bold != first.bold
            || style.italic != first.italic
            || (style.ratio - first.ratio).abs() > f64::EPSILON
            || style.ratios != first.ratios
            || style.kerning != first.kerning
        {
            return false;
        }
        tracking_differs |= (style.letter_spacing - first.letter_spacing).abs() > f64::EPSILON
            || style.letter_spacings.as_slice() != first_tracking.as_slice();
    }

    tracking_differs
}

/// [#2279 axis B] 셀 텍스트 오버플로 시 좌우 패딩 축소 — 렌더/측정 공용 코어.
///
/// 측정과 배치는 동일한 최소 셀 너비 규칙을 사용한다.
/// 과거 #2237/#2279의 86712 5줄/4줄 진단은 저장 LineSeg가 누락된 잘못된
/// 파생 입력을 근거로 삼았다. 정상 한컴 저장본은 측정·재조판 모두 4줄이다.
/// 그 진단 수치를 현재 정상 원본의 조판 계약으로 사용하지 않는다.
/// 한/글이 셀 안 줄에 보장하는 최소 너비 (HWPUNIT). 1440 = 정확히 0.2 인치.
///
/// 안 여백 합이 셀 폭에 육박하면 남는 폭이 0 에 가까워진다. 한/글은 그 폭으로 줄을 만들지
/// 않고 이 하한선까지 넓힌다. `samples/` HWP5 전수(셀-줄 91,866개)에서 저장된
/// `LineSeg.segment_width` 를 남는 폭과 맞대면 경계가 뚜렷하다.
///
/// | 남는 폭 | 줄 수 | `sw == 1440` |
/// | --- | --- | --- |
/// | 0 이상 1440 미만 | 7,862 | **98.7%** |
/// | 1440 초과 | 83,127 | 0.8% |
///
/// 근거·재현: `mydocs/report/2026-08-28-cell-min-line-width.md`
const HANGUL_MIN_CELL_LINE_WIDTH_HWPUNIT: f64 = 1440.0;

/// 남는 폭이 하한선보다 좁으면 **오른쪽** 안 여백을 깎아 하한선을 확보한다.
///
/// 왼쪽은 건드리지 않는다 — 한/글은 글자 시작 위치를 유지한다(셀보호2 실측: 첫 글자가
/// 셀 좌단 + 2834 HWPUNIT 그대로, 미리보기 이미지와 2px 이내 일치).
///
/// 셀이 하한선보다 좁으면 오른쪽 여백이 음수가 된다. 이는 글자가 셀 밖으로 나간다는 뜻이고
/// 한/글도 그렇게 한다(셀 폭 564 에 sw=1440). 호출부는 `pad_right` 를 `cell_w - pl - pr`
/// 뺄셈에만 쓰므로 음수가 그대로 폭으로 환원된다.
pub(crate) fn floored_cell_line_width_padding(
    pad_left: f64,
    pad_right: f64,
    cell_w: f64,
    dpi: f64,
) -> (f64, f64) {
    let min_width = HANGUL_MIN_CELL_LINE_WIDTH_HWPUNIT / 7200.0 * dpi;
    let available = cell_w - pad_left - pad_right;

    // 폭이 없는 셀(선언 없음 → 0)에는 적용하지 않는다. 하한선은 실제 셀에 대한 규칙이고,
    // 폭을 모르는 자리에 19.2px 를 만들어 주는 것은 측정이 아니라 추측이다.
    //
    // 남는 폭이 **음수**인 구간도 다루지 않는다. `aim=true` 로는 표본에 0건이고,
    // `aim=false` 는 86.9% 로 갈린다(한 파일의 100줄이 여백을 무시하고 셀 폭을 다 쓴다).
    // 근거가 있는 구간은 `0 <= 남는 폭 < 1440` 뿐이고, 거기서만 적용한다.
    if cell_w <= 0.0 || available < 0.0 || available >= min_width {
        return (pad_left, pad_right);
    }
    (pad_left, cell_w - pad_left - min_width)
}

/// 셀 안 글자 상자의 너비. 최소 줄 너비 하한선을 포함한다.
///
/// 같은 식 `(cell_w - pad_left - pad_right).max(0)` 이 렌더 4곳·측정 3곳·편집 2곳에
/// 복사돼 있었다. 하한선을 그중 일부에만 넣으면 측정 줄 수와 렌더 줄 수가 갈려 행 높이가
/// 어긋난다(#2279 가 겪은 것이 정확히 그 발산이다). 계산을 한 자리로 모아 호출부가
/// 규칙을 고를 수 없게 한다.
pub(crate) fn cell_inner_text_width(cell_w: f64, pad_left: f64, pad_right: f64, dpi: f64) -> f64 {
    let (pad_left, pad_right) = floored_cell_line_width_padding(pad_left, pad_right, cell_w, dpi);
    // 원본 정수 폭과 안 여백은 정수 축에서 먼저 빼고 한 번만 픽셀로 바꾼다.
    // 각각 정확히 왕복하는 피연산자의 부동소수 뺄셈도 9980HU를 9979HU로
    // 떨어뜨릴 수 있다. 연속 좌표에는 이 출처를 추정하거나 허용치를 더하지 않는다.
    let source_units = [cell_w, pad_left, pad_right].map(|value| {
        let unit = crate::renderer::px_to_hwpunit(value, dpi);
        (crate::renderer::hwpunit_to_px(unit, dpi).to_bits() == value.to_bits())
            .then_some(i64::from(unit))
    });
    if let [Some(width), Some(left), Some(right)] = source_units {
        if let Ok(inner) = i32::try_from((width - left - right).max(0)) {
            return crate::renderer::hwpunit_to_px(inner, dpi);
        }
    }
    (cell_w - pad_left - pad_right).max(0.0)
}

pub(crate) fn shrunk_cell_horizontal_padding(
    pad_left: f64,
    pad_right: f64,
    cell_w: f64,
    // 칸의 글자 상자 높이(px). 줄바꿈 결과가 여기에 들어가면 여백을 깎지 않는다.
    inner_height_px: f64,
    composed_paras: &[ComposedParagraph],
    paragraphs: &[Paragraph],
    styles: &ResolvedStyleSet,
    preserve_cell_padding: bool,
    line_wrap_squeeze: bool,
    stored_geometry_policy: crate::model::provenance::StoredGeometryPolicy,
    dpi: f64,
) -> (f64, f64) {
    // 하한선을 먼저 적용한다. `preserve_cell_padding`(aim=true) 은 "저장된 안 여백을
    // 임의로 깎지 않는다"는 뜻이지 "줄을 0 폭으로 만든다"는 뜻이 아니다 — 한/글은 aim 과
    // 무관하게 하한선을 지킨다(셀보호2 는 aim=true 이고 남는 폭 283 인데 sw=1440).
    let (pad_left, pad_right) = floored_cell_line_width_padding(pad_left, pad_right, cell_w, dpi);

    if preserve_cell_padding {
        return (pad_left, pad_right);
    }

    // [#6145] **"한 줄로 입력"(`lineWrap=SQUEEZE`) 칸은 안 여백을 깎지 않는다.**
    //
    // 이 규칙은 "글자가 넘치면 여백을 1px 까지 내주어 자리를 만든다"는 뜻인데, SQUEEZE
    // 칸에서 한/글이 하는 일은 정반대다 — 여백은 그대로 두고 **자간을 줄여** 글자를
    // 안쪽 폭에 밀어 넣는다. 156607916 6쪽 마지막 열이 그 예로, 저장 lineseg 가
    // `horzsize=9340`(=93.40pt)를 못박아 두었는데도 여백을 깎아 97.56pt 를 내주는 바람에
    // 자간이 덜 줄고 글자가 우측 괘선 밖 +0.69pt 로 나갔다(한/글은 −3.02pt 안쪽).
    if line_wrap_squeeze {
        return (pad_left, pad_right);
    }

    // [Task #617] 다중 줄(2 줄 이상) 단락이 line_segs 로 분배 완료된 경우,
    // HWP 가 가용 폭에 맞춰 자간을 분배하고 줄바꿈을 확정한 상태이므로
    // 자연 폭 추정으로 다시 깎으면 오버 페인팅. 단일 줄 셀(좁은 수치 셀
    // 등에서 오버플로우 가능성 있음) 은 종전 휴리스틱으로 보호한다.
    let any_multiline_distributed = paragraphs.iter().any(|p| p.line_segs.len() >= 2);
    if !stored_geometry_policy.requires_realign && any_multiline_distributed {
        return (pad_left, pad_right);
    }

    let mut max_line_w = 0.0f64;
    for comp in composed_paras {
        for line in &comp.lines {
            let mut w = 0.0;
            for run in &line.runs {
                let mut ts = run.text_style(styles);
                if run.char_overlap.is_some() {
                    let fs = if ts.font_size > 0.0 {
                        ts.font_size
                    } else {
                        12.0
                    };
                    let chars: Vec<char> = run.text.chars().collect();
                    w += fs * char_overlap_advance_units(&chars) as f64;
                    continue;
                }
                // 자연 폭 측정: 음수 자간을 제거하여 글리프가 서로 겹치지 않는 최소 폭을 얻음
                if ts.letter_spacing < 0.0 {
                    ts.letter_spacing = 0.0;
                }
                // [Task #555] PUA 옛한글 변환 후 자모 시퀀스 폭 사용.
                // (estimate_text_width 는 ts.ratio 를 자체 반영함.)
                w += estimate_text_width(effective_text_for_metrics(run), &ts);
            }
            if w > max_line_w {
                max_line_w = w;
            }
        }
    }
    let available = (cell_w - pad_left - pad_right).max(0.0);
    // Task #347: estimate_text_width는 영어 본문(Times New Roman 등) 자연 폭을
    // 5~15%까지 과대 추정할 수 있어, HWP가 이미 줄바꿈한 본문에서도
    // padding 축소가 잘못 트리거됨. 15% 이내 초과는 정상으로 보고 미축소.
    //
    // [#7413] 이 사전 필터는 **그대로 둔다.** 아래 높이 판정이 "깎지 않아도 되는 칸"을
    // 걸러내는 일을 하고, 이 필터를 함께 없애면 종전에 깎지 않던 칸까지 새로 깎게 되어
    // 측정 범위 밖의 동작 확장이 된다(samples 전수 A/B: 변화 174 → 234).
    let overflow_threshold = available * 1.15;
    if max_line_w <= overflow_threshold || cell_w <= 2.0 {
        return (pad_left, pad_right);
    }

    // [#7413] **줄바꿈으로 해결되는 문단은 깎지 않는다.**
    //
    // 종전 조건은 "자연 폭 > 가용 너비 × 1.15" 였다. 그건 "이 문단은 줄바꿈이 필요하다"는
    // 뜻일 뿐이라, 줄바꿈해도 칸 높이에 들어가는 문단까지 전부 걸렸다. 1.15 는 Task #347 이
    // 영문 자연 폭 과대 추정을 흡수하려고 둔 값이고 한/글 출력에서 나온 값이 아니다.
    //
    // 축소가 실제로 지탱하는 계약은 "칸 안 글자가 칸 밖으로 새지 않는다" 하나다(실측:
    // 축소를 끄면 samples 전수에서 겹침이 4건 늘고, 그 넷이 전부 줄바꿈 결과가 행 높이를
    // 넘는 자리다 — 20099369_yeongwol_forms 3쪽 칸 w=41.9/h=43.6 가 2줄 45px 를 요구,
    // issue5169 12쪽 Cell5 h=59.2 가 4줄 80.3px, Cell51 h=29.4 가 2줄 46.7px).
    //
    // 그러므로 묻는 것을 바꾼다 — "줄바꿈한 결과가 칸 높이를 넘는가". 넘지 않으면 줄바꿈이
    // 답이고 여백은 그대로 둔다. 정본 대조 가능한 10쪽(80168 3쪽·2022 국립국어원 3쪽·
    // 156160455 1쪽)에서 종전 축소는 전부 한/글에서 멀어지게 하고 있었다.
    if inner_height_px > 0.0 && available > 0.0 {
        // 줄 수를 **추정하지 않고 실제로 센다.** `ceil(자연 폭 / 가용 폭)` 은 어절
        // 줄바꿈이 줄 끝에 남기는 낭비를 반영하지 못해 낙관적이다(issue5169 `Cell5`:
        // 추정 52.3px vs 실제 4줄 83.2px). 폭 판정을 통과한 칸에서만 도는 경로라
        // 조합 비용은 그 칸들로 한정된다.
        // 줄 메트릭의 원본 단위에서 합산한다. 매 줄을 실수로 환산해 더하면
        // 정확히 들어가는 여러 문단도 누적 오차 때문에 넘침으로 바뀔 수 있다.
        let mut wrapped_height_hu = 0i64;
        let mut last_line_spacing_hu = 0i64;
        let mut stored_fits_fewer_lines = false;
        for (idx, para) in paragraphs.iter().enumerate() {
            let mut comp = match composed_paras.get(idx) {
                Some(c) => c.clone(),
                None => compose_paragraph_in_context(para, styles),
            };
            recompose_cell_lines_in_frame(
                &mut comp,
                para,
                ParagraphBox::content_for_style(
                    available,
                    styles.para_styles.get(para.para_shape_id as usize),
                    dpi,
                ),
                styles,
                dpi,
                stored_geometry_policy,
            );
            // [#7418] 유효한 저장 줄(비합성 LINE_SEG)이 재구성보다 적으면 저장 줄 수를 따른다.
            // 재정렬 요청은 저장 줄 구성이 무효하다는 뜻이므로 이 증거를 수용하지 않는다.
            // 이때 여백 유지 여부는 같은 정책으로 재구성한 실제 줄 높이에서 판단한다.
            let stored_lines = para
                .line_segs
                .iter()
                .filter(|seg| {
                    seg.tag & crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0
                })
                .count();
            if !stored_geometry_policy.requires_realign
                && stored_lines > 0
                && comp.lines.len() > stored_lines
            {
                stored_fits_fewer_lines = true;
            }
            for line in &comp.lines {
                // 실제 줄 피치는 `line_height + line_spacing` 이다. `line_height` 만
                // 쓰면 줄간격이 빠져 높이를 과소평가한다(20099369 3쪽 칸: 1300 + 390
                // = 1690 HWPUNIT = 22.53px, 렌더 실측 22.5px).
                wrapped_height_hu += i64::from(line.line_height) + i64::from(line.line_spacing);
                last_line_spacing_hu = i64::from(line.line_spacing);
            }
        }
        // [#7418] 칸의 **마지막** 줄 뒤에는 이을 줄이 없어 그 줄간격은 칸을 채우지 않는다 —
        // 행 높이 측정(`height_measurer`, 칸 마지막 줄의 말미 줄간격 제외)과 같은 규칙.
        // 70833 pi=83 4행: 3줄 56.0(말미 5.33 포함) > 안높이 55.4 로 여백을 깎아 한/글보다
        // 5.7px 왼쪽에서 시작하고 줄 끝이 달라졌다. 말미를 빼면 50.7 ≤ 55.4 다.
        wrapped_height_hu -= last_line_spacing_hu.max(0);
        let wrapped_height = wrapped_height_hu as f64 * dpi / crate::renderer::HWPUNIT_PER_INCH;
        if std::env::var_os("RHWP_DIAG_SHRINK").is_some() {
            println!(
                "D_SHRINK cell_w={cell_w:.1} avail={available:.1} inner_h={inner_height_px:.15} wrapped_h={wrapped_height:.15} max_line_w={max_line_w:.1}"
            );
        }
        // 줄 피치를 더하고 셀 여백을 빼면 같은 물리 높이가 인접한 f64 값으로 반올림될 수 있다.
        // 표현 가능한 값 한 단계의 차이는 여백을 축소할 오버플로우가 아니라 연산 오차다.
        if wrapped_height <= inner_height_px.next_up() && !stored_fits_fewer_lines {
            return (pad_left, pad_right);
        }
    }
    let min_pad = 1.0;
    let total_pad = pad_left + pad_right;
    let max_reducible = (total_pad - 2.0 * min_pad).max(0.0);
    if max_reducible <= 0.0 {
        return (pad_left, pad_right);
    }
    let deficit = max_line_w - available;
    let reduction = deficit.min(max_reducible);
    let new_total = total_pad - reduction;
    let new_left = if total_pad > 0.0 {
        pad_left * new_total / total_pad
    } else {
        new_total / 2.0
    };
    let new_right = new_total - new_left;
    (new_left, new_right)
}

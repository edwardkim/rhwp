//! Ordinary row scan step. Queries keep the page state read-only; this step owns scan-result updates.

use crate::renderer::typeset::{
    controls, is_reparsed_single_column_cell_split_row, paragraph,
    row_has_stored_cross_paragraph_zero_reset, row_has_stored_same_vpos_split_signal,
    row_split_meets_min_top_keep, rowbreak_row_has_internal_saved_vpos_reset,
    rowbreak_table_has_internal_saved_vpos_reset, table, BlockRowScanVars, BlockTableRowScan,
    Control, TypesetEngine, MIN_TOP_KEEP_PX, TERMINAL_ROW_BOTTOM_SQUEEZE_MAX_REST_PX,
    TERMINAL_ROW_BOTTOM_SQUEEZE_MIN_HEADROOM_PX, TERMINAL_ROW_BOTTOM_SQUEEZE_TOLERANCE_PX,
};

use super::{ScanInput, ScanProgress, ScanStep};

impl TypesetEngine {
    pub(super) fn scan_ordinary_row_step(
        &self,
        input: ScanInput<'_>,
        progress: ScanProgress,
        cs_before: f64,
    ) -> ScanStep {
        let ScanInput {
            st,
            layout_engine,
            mt,
            table,
            styles,
            cut_row_h,
            whole_row_fit_h,
            rowspan_touched,
            start_cut,
            v,
            table_storage_declares_splits,
        } = input;
        let BlockRowScanVars {
            cursor_row,
            row_count,
            cs,
            can_intra_split,
            is_continuation,
            avail_for_rows,
            header_overhead,
            landscape_rowbreak_bleed,
            landscape_whole_row_tolerance,
            landscape_short_row_tolerance,
            landscape_short_row_max_height,
            strict_painted_bottom_fit,
            source_first_fragment_overflow_allowance,
            source_first_fragment_row_end,
            start_row_height_override,
        } = v;
        let ScanProgress {
            mut r,
            scan,
            mut bleed_absorbed_row_height,
        } = progress;
        let BlockTableRowScan {
            mut consumed,
            mut end_row,
            mut split_block_start,
            mut split_end_cut,
            mut split_end_limit,
            mut end_row_height_override,
        } = scan;
        let block_query = table::scan::RowBlockQuery {
            layout_engine,
            mt,
            table,
            styles,
            cut_row_h,
            rowspan_touched,
            cs,
        };
        let keep_scanning = (|| {
            // rowspan 셀이 걸친 행 — 기본은 MeasuredTable 높이로 통째 배치한다.
            //
            // 다만 RowBreak 표의 큰 rowspan 블록 안에 있는 일반 내용 행은 한컴처럼
            // 해당 행의 row_span==1 셀을 기준으로 내부 분할을 허용한다. 작은 보호
            // 블록은 위의 block path 에서 이미 처리되며, 여기서는 block path 대상이
            // 아닌 큰 블록의 과도한 이월만 줄인다.
            let row_query = table::scan::row::RowScanQuery {
                rows: &block_query,
                r,
                cursor_row,
                start_cut,
                start_row_height_override,
            };
            let rowbreak_rowspan_row_splittable =
                mt.allows_row_break_split() && can_intra_split && mt.is_row_splittable(r);
            if rowspan_touched[r] && !rowbreak_rowspan_row_splittable {
                // 실제로 수용한 앞 행들의 높이(consumed)를 사용한다. 이전 행의
                // 증분까지 포함하며, 늘린 높이가 안 맞으면 원래 높이로 되돌리지 않는다.
                let h = row_query.required_height(cut_row_h[r], consumed, cs_before);
                if r == cursor_row || consumed + cs_before + h <= avail_for_rows {
                    consumed += cs_before + h;
                    r += 1;
                    end_row = r;
                    return true;
                }
                // [#3820 Stage 76] 이전 행에서 시작한 rowspan이 닿는 짧은
                // RowBreak 행은 실제 텍스트 한 줄이 현재 쪽의 잔여에 이미 모두
                // 들어가도, 선언 높이만 커서 통째로 다음 쪽으로 밀릴 수 있다.
                // 이 경우 한컴은 마지막 행 밴드를 남은 물리 높이로 끝내고 다음
                // 행부터 재개한다(76076 p35→p36 `주요내용`). 일반 rowspan 행을
                // 전역으로 분할하지 않고, prior-span + 비중첩 + 내용 완전 소비
                // 조건에서만 렌더 높이 상한을 carry 한다.
                let table::scan::row::RowBandShape {
                    has_prior_rowspan_cover,
                    row_has_nested,
                } = row_query.band_shape();
                let rest = (avail_for_rows - consumed - cs_before).max(0.0);
                let row_start_cut: &[usize] = if r == cursor_row { start_cut } else { &[] };
                if mt.allows_row_break_split()
                    && can_intra_split
                    && r > cursor_row
                    && has_prior_rowspan_cover
                    && !row_has_nested
                    && rest > 0.0
                {
                    let table::scan::row::RowBandProbe {
                        probe,
                        visible_height,
                    } = row_query.probe_band(row_start_cut, rest);
                    if std::env::var("RHWP_DIAG_SCAN").is_ok() {
                        eprintln!(
                        "DIAG_SCAN RSPAN_BAND? r={} h={:.1} rest={:.1} visible={:.1} fully={} nested={}",
                        r, h, rest, visible_height, probe.fully_consumed, row_has_nested
                    );
                    }
                    // Stage 76의 긴 declared-row tail은 내용 뒤에 충분한 물리 blank
                    // band가 남을 때만 현재 fragment에 보존한다. content가 남은
                    // 공간을 거의 전부 쓰는 경우까지 이 경로를 열면 76076 p18의
                    // `해당 없음`처럼 한 행의 텍스트만 먼저 잘려 p19의 source owner가
                    // 앞당겨진다. 한컴은 그 근소한 pseudo-tail은 보존하지 않고 행 전체를
                    // 다음 쪽으로 넘긴다.
                    if table::scan::row::retains_blank_tail(&probe, visible_height, rest) {
                        consumed += cs_before + rest;
                        r += 1;
                        end_row = r;
                        end_row_height_override = Some(rest);
                        // 다음 조각은 같은 행의 full cut에서 재개해, 남은 빈
                        // 밴드만 그린 뒤 다음 물리 행으로 넘어간다.
                        split_end_cut = probe.end_cut;
                        split_end_limit = rest;
                        if std::env::var("RHWP_DIAG_SCAN").is_ok() {
                            eprintln!(
                            "DIAG_SCAN RSPAN_BAND r={} limit={:.1} visible={:.1} declared={:.1}",
                            r - 1, rest, visible_height, h
                        );
                        }
                        return false;
                    }
                }
                // [#2236 진단] rowspan 행 경계 정지 — 동작 불변.
                if std::env::var("RHWP_DIAG_SCAN").is_ok() {
                    eprintln!(
                        "DIAG_SCAN RSPAN_STOP r={} consumed={:.1} h={:.1} avail={:.1} rest={:.1}",
                        r,
                        consumed,
                        h,
                        avail_for_rows,
                        avail_for_rows - consumed
                    );
                }
                end_row = r;
                return false;
            }

            // [Task #1022] 일반 행 r — 부분 행은 row_cut_content_height
            // (`cut_row_h`)로 자르되, #3820의 native HWP5 rewind 형상에서 온전한
            // 행을 남길지 판단할 때는 renderer가 paint할 footprint를 사용한다.
            let row_start_cut: &[usize] = if r == cursor_row { &start_cut } else { &[] };
            let row_total = row_query.whole_row_height(row_start_cut, whole_row_fit_h);
            // 온전한 행 후보에는 rowspan 잔여 내용도 예약한다. 아래에서 실제
            // end_cut을 선택하면 row_cut_content_height로 분할 높이를 다시 측정하고,
            // 렌더러도 같은 end_cut을 받아 잔여 전체 높이 보정을 생략한다.
            let row_total = row_query.required_height(row_total, consumed, cs_before);
            let table::scan::source_frame::SourceFrameSelection {
                terminal_response_before_empty_spacer,
                two_line_terminal_response_source_frame,
                stored_source_frame,
                terminal_source_frame,
                continued_source_frame,
                opening_source_frame,
                mid_source_frame,
                whole_row_fits,
            } = table::scan::source_frame::SourceFrameQuery {
                row: &row_query,
                row_start_cut,
                row_count,
                is_continuation,
                profile: &st.profile,
            }
            .resolve(
                table::scan::source_frame::WholeRowBudget {
                    consumed,
                    cs_before,
                    row_total,
                    avail_for_rows,
                    strict_painted_bottom_fit,
                    source_first_fragment_overflow_allowance,
                    source_first_fragment_row_end,
                },
                || self.render_normalization.table_text_reflowed(table),
                |row| Self::row_has_no_text_or_controls(table, row),
            );
            if whole_row_fits {
                // 행 전체가 예산 안에 들어감.
                bleed_absorbed_row_height = None;
                consumed += cs_before + row_total;
                r += 1;
                end_row = r;
                return true;
            }
            if r > cursor_row
                && terminal_response_before_empty_spacer
                && mt.row_heights.get(r).is_some_and(|stored_height| {
                    consumed + cs_before + *stored_height <= avail_for_rows + 0.5
                })
            {
                consumed += cs_before + row_total;
                r += 1;
                end_row = r;
                return true;
            }
            // Landscape RowBreak continuations use the stored physical row frame.
            // Keep the baseline whole-row allowance separate from the larger
            // short-row allowance, and never apply the latter to rowspan or a
            // row containing an internal saved page boundary.
            // 흡수 형상(연속 조각 경계)이되 행내 분할 가능해 흡수 대신 분할로
            // 돌린 행 — 아래 고아 가드가 이 행의 정상 컷(첫 줄 유지)을 content
            // 높이 미달로 기각해 행 통째 이월로 되돌리지 않도록 표시한다.
            let mut landscape_boundary_splittable = false;
            let landscape_query = table::scan::landscape::LandscapeRowQuery {
                row: &row_query,
                row_start_cut,
                profile: &st.profile,
                landscape_rowbreak_bleed,
                is_continuation,
                header_overhead,
                bleed_absorbed_row_height,
                can_intra_split,
                table_storage_declares_splits,
                budget: table::scan::landscape::LandscapeRowBudget {
                    consumed,
                    cs_before,
                    row_total,
                    avail_for_rows,
                },
            };
            let landscape_whole_row_shape =
                landscape_query.whole_row_shape(landscape_whole_row_tolerance);
            if landscape_whole_row_shape
            // [#6307] 행내 분할 가능한 다줄 행은 얹지 않는다 — 한컴 2022 는 이런 행을
            // 본문 하한에서 가른다 (hwpctl_ParameterSetID p11 실측: 2줄 행
            // 통짜 흡수 시 +25.7px 로 바탕쪽 로고 밴드까지 침범). 흡수는
            // 가를 수 없는 행(단일 줄·이미지 셀)의 경계 구제만 맡고,
            // 가를 수 있는 행은 아래 인트라-분할이 한컴처럼 첫 줄(들)만
            // 남긴다 (landscape_boundary_band_keep).
            && !landscape_query.boundary_splittable()
            {
                bleed_absorbed_row_height = Some(row_total);
                consumed += cs_before + row_total;
                r += 1;
                end_row = r;
                return true;
            }
            if landscape_query.short_row_shape(
                landscape_short_row_max_height,
                landscape_short_row_tolerance,
                || rowbreak_row_has_internal_saved_vpos_reset(table, r),
            ) {
                // [#6307] 행내 분할 가능한 다줄 행은 whole-row 분기와 같은 이유로 얹지
                // 않는다 — 한컴은 본문 하한에서 가른다 (hwpctl_ParameterSetID p11).
                if !landscape_query.boundary_splittable() {
                    bleed_absorbed_row_height = Some(row_total);
                    consumed += cs_before + row_total;
                    r += 1;
                    end_row = r;
                    return true;
                }
                landscape_boundary_splittable = true;
            }
            if landscape_whole_row_shape && landscape_query.boundary_splittable() {
                landscape_boundary_splittable = true;
            }
            if landscape_boundary_splittable && std::env::var("RHWP_DIAG_6307").is_ok() {
                eprintln!(
                "DIAG6307 r={} row_total={:.1} band={:.1} avail={:.1} hdr={:.1} reset={} tbl_reset={} rows={}",
                r,
                row_total,
                avail_for_rows - consumed - cs_before,
                avail_for_rows,
                header_overhead,
                rowbreak_row_has_internal_saved_vpos_reset(table, r),
                rowbreak_table_has_internal_saved_vpos_reset(table),
                row_count,
            );
            }
            // 행 r 이 예산 초과 — 인트라-분할 시도.
            // [Task #77] 분할 불가 행(이미지 셀 등)은 통째 배치 / 다음 페이지.
            // `MeasuredTable`은 2행 이상 중첩 표만 `nested_split_row_count`로
            // 기록한다. 그러나 native HWP5 short parent의 마지막 1×1 child는
            // `cell_units`가 fragment를 만들더라도 그 값이 1이라 atomic으로 남는다.
            // 동일 storage/physical-height gate와 실제 multi-unit 확인을 통해서만
            // 해당 행을 `advance_row_cut`에 전달한다 (76076 p81→82).
            let row_entry = table::scan::row_entry::RowEntryQuery {
                row: &row_query,
                row_start_cut,
            };
            let terminal_single_source_note_row =
                row_entry.terminal_note_shape(strict_painted_bottom_fit, row_count);
            if terminal_single_source_note_row {
                let table::scan::row_entry::TerminalNoteProbe {
                    remaining_band,
                    source_cut,
                } = row_entry.terminal_note_probe(avail_for_rows, consumed, cs_before);
                if remaining_band > 0.0
                    && source_cut.fully_consumed
                    && source_cut.consumed_height > 0.0
                {
                    // 마지막 주석의 실제 저장 line이 남은 band 안에 모두 있으므로,
                    // 선언 row 높이의 빈 아래 영역은 별도 physical page를 소유하지 않는다.
                    consumed += cs_before + remaining_band;
                    r += 1;
                    end_row = r;
                    end_row_height_override = Some(remaining_band);
                    return true;
                }
            }
            let table::scan::row_entry::RowSplitGate {
                native_short_parent_child_splittable,
                splittable,
            } = row_entry.split_gate(can_intra_split, {
                // [#7288] «쪽 경계에서» 가 행 내부 컷을 허용하는지 묻는다. 값 2 «나눔» 만
                // 무조건 자르고, 값 0 «나누지 않음»·값 1 «셀 단위로 나눔» 은 이 조각의
                // **온전한 밴드**에도 행이 안 들어갈 때 — 곧 어느 쪽에도 못 넣을 때 —
                // 만 불가피하게 자른다. 그 밖에는 `splittable=false` 로 떨어져 행 경계에서
                // 조각을 끝내고(`end_row = r`) 다음 쪽에서 행을 통째로 재개한다. 한/글
                // 정본: 편람 PDF 158→159쪽이 큰 행을 통째로 넘기며 앞쪽 바닥을 비운다.
                let row_needs_whole_band =
                    r == cursor_row && cut_row_h.get(r).copied().unwrap_or(0.0) > avail_for_rows;
                !crate::renderer::typeset::none_table_is_atomic_here(table)
                    || table_storage_declares_splits
                    || row_needs_whole_band
            });
            if !splittable {
                // [#2236 진단] 분할 불가 정지 — 동작 불변.
                if std::env::var("RHWP_DIAG_SCAN").is_ok() {
                    eprintln!(
                        "DIAG_SCAN UNSPLITTABLE r={} consumed={:.1} row_total={:.1} rest={:.1}",
                        r,
                        consumed,
                        row_total,
                        avail_for_rows - consumed
                    );
                }
                if r == cursor_row {
                    // 페이지 시작 행 — 강제 통째 배치(오버플로 감수).
                    consumed += cs_before + row_total;
                    end_row = r + 1;
                } else {
                    end_row = r;
                }
                return false;
            }
            let padding = row_entry.padding();
            let content_budget = (avail_for_rows - consumed - cs_before - padding).max(0.0);
            let native_hwp5_internal_reset_row_tail = row_entry.native_reset_tail(&st.profile);
            // A visible terminal response followed by a no-text/no-control row is
            // a two-part physical row: the spacer owns no ink, while the
            // response carries the stored page frame. A direct HWPX opening
            // frame with one visible source owner has the same exact boundary.
            // This is structural source evidence and deliberately does not
            // depend on a document shape, stored table size, or line count.
            // Stored vpos-frame resets are source-owned physical fragment boundaries.
            // First take the ordinary budget cut, then extend only to the end of
            // the recorded source frame when that exact CellUnit boundary is known.
            let (mut res, mut budget) = layout_engine.advance_row_cut_with_mixed_nested_reserve(
                table,
                r,
                row_start_cut,
                content_budget,
                styles,
            );
            let source_tail_query = table::scan::source_tail::SourceTailQuery {
                row: &row_query,
                row_start_cut,
                profile: &st.profile,
                terminal_response_before_empty_spacer,
                terminal_source_frame,
                continued_source_frame,
                opening_source_frame,
                mid_source_frame,
            };
            let table::scan::source_tail::SourceTailGate {
                enabled: source_tail_enabled,
                mid_frame_only,
            } = source_tail_query.gate(&res);
            let mut uses_source_frame_tail = false;
            if source_tail_enabled {
                let source_tail_cut = source_tail_query.candidate(&res, stored_source_frame);
                if let Some(mut source_tail_cut) = source_tail_cut {
                    if let Some(correction) =
                        source_tail_query.mirrored_correction(&res, &source_tail_cut, padding)
                    {
                        source_tail_cut.end_cut = correction.end_cut;
                        source_tail_cut.consumed_height = correction.consumed_height;
                        source_tail_cut.fully_consumed = false;
                    }
                    let table::scan::source_tail::extension::SourceTailFit {
                        extension,
                        mid_extension_ok,
                        source_tail_owns_this_page,
                    } = source_tail_query.extension_fit(
                        &res,
                        &source_tail_cut,
                        mid_frame_only,
                        avail_for_rows,
                    );
                    if extension > 0.5 && mid_extension_ok && source_tail_owns_this_page {
                        // Downstream fit/retry decisions must reason in the
                        // same frame-sized budget as the cut.  The precise
                        // physical overfill is measured from the painted
                        // candidate below.
                        budget = source_tail_cut.consumed_height;
                        res = source_tail_cut;
                        uses_source_frame_tail = true;
                    }
                }
            }
            // [#2236 진단] 인트라 컷 시도 결과 — 동작 불변.
            if std::env::var("RHWP_DIAG_SCAN").is_ok() {
                eprintln!(
                "DIAG_SCAN CUT_TRY r={} budget={:.1} padding={:.1} consumed_h={:.1} fully={} end_cut={:?}",
                r, budget, padding, res.consumed_height, res.fully_consumed, res.end_cut
            );
            }
            // Native HWP5의 empty-host → 1×1 child → 내부 표 형상은 child 본문의
            // 앞 몇 줄만 쪽 끝에 두면 실제 ink가 다음 internal table보다 한 쪽 먼저
            // 누출한다. 선행 묶음 전체가 새 본문에는 들어가는 경우에만, 현재 row를
            // 소비하지 않고 새 page에서 다시 시작한다 (86712 r27).
            if r > cursor_row
                && layout_engine.should_defer_fresh_rowbreak_wrapper_prefix(
                    table,
                    r,
                    row_start_cut,
                    &res.end_cut,
                    st.layout.body_area.height,
                    styles,
                )
            {
                if std::env::var("RHWP_DIAG_SCAN").is_ok() {
                    eprintln!(
                        "DIAG_SCAN DEFER_WRAPPER_PREFIX r={} end_cut={:?}",
                        r, res.end_cut
                    );
                }
                end_row = r;
                return false;
            }
            if r == cursor_row && row_start_cut.is_empty() {
                if let Some(safe_end_cut) = layout_engine
                    .fresh_rowbreak_wrapper_safe_prefix_end_cut(
                        table,
                        r,
                        row_start_cut,
                        &res.end_cut,
                        styles,
                    )
                {
                    let safe_total = layout_engine.row_cut_content_height(
                        table,
                        r,
                        row_start_cut,
                        &safe_end_cut,
                        styles,
                        false,
                    );
                    res.end_cut = safe_end_cut;
                    res.consumed_height = (safe_total - padding).max(0.0);
                    if std::env::var("RHWP_DIAG_SCAN").is_ok() {
                        eprintln!(
                            "DIAG_SCAN SAFE_WRAPPER_PREFIX r={} consumed_h={:.1} end_cut={:?}",
                            r, res.consumed_height, res.end_cut
                        );
                    }
                }
            }
            if res.fully_consumed {
                // [#2097→#5714] 표를 **완결하는 마지막 행**이 콘텐츠는 잔여에 다
                // 들어가는데 선언 높이만 소폭 넘을 때, 한글은 행 밴드를 잔여로
                // 압축해 쪽을 완결한다(1741000 r14: 선언 80.3 → 밴드 69.7, 한글
                // 2024 PDF 실측 — p2 상단 새 행은 전체 높이, 말미 행만 압축).
                // f8c784235 가 삭제한 BOTTOM_SQUEEZE 계약의 말미-행 한정 복원:
                // 종전에는 앞 조각의 유령 tail 밴드가 다음 조각 첫 행을 눌러 이
                // 핀을 우연히 대신했는데, 그 밴드 이월을 #5714 가 막으면서 실제
                // 계약이 필요해졌다. 허용치·잔여 상한·콘텐츠 여유 하한은 삭제 전
                // 상수 그대로(1741000 실측 기반), 중간 블록의 압축/이월 판별
                // 불가(kps-ai 반증)는 말미-행 한정으로 배제한다.
                let squeeze_rest = (avail_for_rows - consumed - cs_before).max(0.0);
                let terminal_row_bottom_squeeze = r + 1 == row_count
                    && r > cursor_row
                    && mt.allows_row_break_split()
                    && !rowspan_touched[r]
                    && row_start_cut.is_empty()
                    && row_total > squeeze_rest + 0.5
                    && row_total <= squeeze_rest + TERMINAL_ROW_BOTTOM_SQUEEZE_TOLERANCE_PX
                    && squeeze_rest <= TERMINAL_ROW_BOTTOM_SQUEEZE_MAX_REST_PX
                    && squeeze_rest - (res.consumed_height + padding)
                        >= TERMINAL_ROW_BOTTOM_SQUEEZE_MIN_HEADROOM_PX
                    && !table.cells.iter().any(|cell| {
                        cell.row as usize == r
                            && cell.paragraphs.iter().any(|paragraph| {
                                paragraph
                                    .controls
                                    .iter()
                                    .any(|control| matches!(control, Control::Table(_)))
                            })
                    });
                if terminal_row_bottom_squeeze {
                    if std::env::var("RHWP_DIAG_SCAN").is_ok() {
                        eprintln!(
                        "DIAG_SCAN TERMINAL_SQUEEZE r={} rest={:.1} row_total={:.1} content={:.1}",
                        r, squeeze_rest, row_total, res.consumed_height
                    );
                    }
                    consumed += cs_before + squeeze_rest;
                    r += 1;
                    end_row = r;
                    end_row_height_override = Some(squeeze_rest);
                    return true;
                }
                // [#2236] rowspan 블록 중간 행 밴드 컷: 행 자체 콘텐츠는 예산 안에
                // 전부 들어가지만(fully_consumed) 행 높이가 rowspan 이웃/선언으로
                // 늘어나 행 전체는 예산 초과인 경우, 한글은 쪽 경계에서 행 밴드를
                // 컷해 페이지를 본문 높이 끝까지 채운다 (21761835 p1/p3/p5 경계
                // 낭비 157/37/39px, 한글 PDF는 매 경계 만충). 콘텐츠-소진 컷을
                // 밴드 컷으로 수용 — RowBreak + rowspan 걸침 행 한정.
                let band_cut_ok = rowspan_touched[r]
                    && mt.allows_row_break_split()
                    && r > cursor_row
                    && !res.end_cut.is_empty()
                    && res.consumed_height >= MIN_TOP_KEEP_PX
                    && budget >= MIN_TOP_KEEP_PX
                    && row_total > budget + 0.5;
                if band_cut_ok {
                    end_row = r + 1;
                    split_end_cut = res.end_cut.clone();
                    split_end_limit = budget.max(res.consumed_height);
                    consumed += cs_before + split_end_limit;
                    if std::env::var("RHWP_DIAG_SCAN").is_ok() {
                        eprintln!(
                            "DIAG_SCAN BAND_CUT r={} limit={:.1} content={:.1} row_total={:.1}",
                            r, split_end_limit, res.consumed_height, row_total
                        );
                    }
                    return false;
                }
                // A terminal response immediately followed by a no-text/no-control row can
                // exceed the composed row metric only by the measured-versus-stored
                // row drift.  Use that exact drift rather than a template allowance.
                let stored_terminal_response_tail_fits = r > cursor_row
                    && terminal_response_before_empty_spacer
                    && (uses_source_frame_tail
                        || mt.row_heights.get(r).is_some_and(|stored_height| {
                            row_total <= budget + (row_total - *stored_height).max(0.0) + 0.5
                        }));
                let two_line_terminal_response_source_frame_fits =
                    two_line_terminal_response_source_frame.is_some_and(|source_frame_height| {
                        row_total <= budget + source_frame_height + 0.5
                    });
                // 단일 유닛 행 — 분할 불가, 페이지 시작이면 강제, 아니면 다음으로.
                if r == cursor_row {
                    consumed += cs_before + row_total;
                    end_row = r + 1;
                } else if stored_terminal_response_tail_fits
                    || two_line_terminal_response_source_frame_fits
                {
                    consumed += cs_before + row_total;
                    end_row = row_count;
                } else {
                    end_row = r;
                }
                return false;
            }
            // 분할 행의 표시 높이(per-cell content+visible pad). advance_row_cut 의
            // consumed_height 는 패딩을 제외하므로, 좁은 #2439 strict 경로의 orphan
            // 판정은 렌더러가 실제로 그리는 이 높이를 사용한다(content 24px + pad 3.8px).
            let split_total = layout_engine.row_cut_content_height(
                table,
                r,
                row_start_cut,
                &res.end_cut,
                styles,
                false,
            );
            // [#3738 Stage 15] native HWP5의 RowBreak 표에 저장된 셀 내부 reset은
            // 같은 row의 앞부분을 현재 쪽 끝에 두고 tail을 다음 쪽에서 재개하라는
            // 물리 경계다. 이때 content-only 첫 cut은 25px orphan 경계에 몇 px
            // 못 미칠 수 있지만, 실제로 보이는 셀 조각은 top/bottom padding까지
            // 포함해 경계를 충족한다. content-only guard로 통째 이월하면 표 24의
            // row 4가 p77에서 재배치되어 그림 51까지 다음 쪽으로 밀린다. native
            // HWP5·비-TAC·RowBreak·같은 row의 stored reset·앞선 행이 이미 있는
            // 경우, 그리고 HWPX Q5의 saved-frame response tail에 한정해 #2439와
            // 같은 painted-height 판정을 사용한다.
            let row_split_min_keep_uses_painted_height = strict_painted_bottom_fit
            || native_hwp5_internal_reset_row_tail
            || uses_source_frame_tail
            || native_short_parent_child_splittable
            // [#6860] 문단 경계의 저장 reset도 한컴이 첫 줄을 남긴 증거다.
            // 24px 내용 + 3.8px 패딩은 25px 최소 표시 높이를 만족한다.
            // 일반 고아 줄 기준이나 아래의 실제 페이지 예산 검사는 완화하지 않는다.
            || (st.profile.hwpx_stored_layout()
                && mt.allows_row_break_split()
                && res.consumed_height > 0.5
                && row_has_stored_cross_paragraph_zero_reset(table, r));
            // [#6035] HWPX 저장 사다리가 이 행을 **쪽 경계에서 줄 단위로 나눈
            // 흔적**(셀 문단의 비전진 동일-vpos 연속 seg 쌍, 좌우분할 아님)을
            // 담고 있으면, 완결 유닛 ≥1 컷에 25px 고아 가드를 적용하지 않는다 —
            // 한글은 그 자리에서 한 줄만 남기는 분할을 실제로 수행했다(2804253
            // 5쪽: 잔여 41.3px 에 '다. 원자재…' 첫 줄 20.8px 유지 — 저장 ladder
            // 0/1560/1560, rhwp 는 행 통째 이월로 5쪽 하단 31pt 공백 + 총 12쪽
            // vs 한글 11쪽). 큰 글줄(10pt+)에서는 한 줄이 25px 미만이라 정상
            // 줄-단위 분할이 상시 기각되는 구조였다. 저장 흔적 없는 행과 예산
            // 초과 컷(아래 재시도/이월 판정)은 종전 그대로다.
            let cellbreak_complete_unit_keep = st.profile.hwpx_stored_layout()
                && mt.allows_row_break_split()
                && res.consumed_height > 0.5
                && res.end_cut.iter().any(|units| *units > 0)
                && row_has_stored_same_vpos_split_signal(table, r);
            // [Task #713] sliver(orphan) 회피 — 일반 표는 기존 content-only 기준을
            // 유지한다. 패딩 포함 painted 기준은 좁은 #2439 strict 표, saved internal
            // reset, 그리고 선언 높이보다 큰 1×1 child가 실제 multi-unit으로 검증된
            // native short parent에만 적용한다. 마지막 경우는 PDF가 border·label과
            // 함께 보이는 첫 child line을 현재 쪽 owner로 고정하지만 content-only
            // 높이가 25px에 근소하게 못 미치는 76076 p81→82 구조다.
            // [#6307 landscape 경계 분할] 흡수 형상에서 분할로 돌린 행의 컷은 한컴처럼
            // 본문 하한 밴드로 남는다 — 남는 밴드(avail-consumed ≥ 고아 기준)가
            // 실제 painted 높이이므로 content-only 기각을 적용하지 않는다.
            let landscape_boundary_band_keep = landscape_boundary_splittable
                && res.consumed_height > 0.5
                && res.end_cut.iter().any(|units| *units > 0)
                && (avail_for_rows - consumed - cs_before) >= MIN_TOP_KEEP_PX;
            // [#6761] 저장 사다리가 이 행을 **첫 줄 뒤에서** 나눈 자리(셀 첫 줄 `vpos=0` 다음
            // 줄도 0)와 이번 컷이 보이는 모든 셀에서 같은 unit 이면, 한컴이 그 자리에 한 줄만
            // 남긴 분할이다 — 25px 고아 기준은 그 한 줄(10pt 17.6px)을 늘 기각한다.
            // `1480000-201900042 <표 2-5>` r=3: 컷 [1,1] = 저장 되감김 [1,1], 기각하면 행 전체가
            // 다음 쪽으로 가 그 쪽 마지막 줄이 본문 바닥을 13px 넘는다.
            // 저장값이 모두 0 인 입력과 가르도록 한 셀 이상에서 되감긴 줄 다음 seg 의 전진을
            // 요구하고, 컷이 저장 경계와 하나라도 다르면 종전 기준을 그대로 쓴다.
            let stored_zero_origin_rewind_keep = (st.profile.hwp5_stored_pagination_layout()
                || st.profile.hwpx_stored_layout())
                && mt.allows_row_break_split()
                && !table.common.treat_as_char
                && row_start_cut.is_empty()
                && res.consumed_height > 0.5
                && {
                    let rewinds =
                        layout_engine.row_stored_zero_origin_rewind_unit_indices(table, r, styles);
                    let visible = layout_engine.row_visible_source_cell_flags(table, r, styles);
                    let visible_indices: Vec<usize> = visible
                        .iter()
                        .enumerate()
                        .filter(|(_, shown)| **shown)
                        .map(|(idx, _)| idx)
                        .collect();
                    !visible_indices.is_empty()
                        && visible_indices.iter().all(|idx| {
                            let cut = res.end_cut.get(*idx).copied().unwrap_or(0);
                            cut > 0
                                && rewinds
                                    .get(*idx)
                                    .is_some_and(|(units, _)| units.first() == Some(&cut))
                        })
                        && visible_indices.iter().any(|idx| {
                            let cut = res.end_cut.get(*idx).copied().unwrap_or(0);
                            rewinds
                                .get(*idx)
                                .is_some_and(|(_, confirmed)| confirmed.contains(&cut))
                        })
                };
            if r > cursor_row
                && !cellbreak_complete_unit_keep
                && !landscape_boundary_band_keep
                && !stored_zero_origin_rewind_keep
                && !row_split_meets_min_top_keep(
                    res.consumed_height,
                    split_total,
                    row_split_min_keep_uses_painted_height,
                )
            {
                end_row = r;
            } else {
                let split_candidate_rows_height = consumed + cs_before + split_total;
                // HWPX RowBreak 조각은 작은 측정 drift에는 종전 여유를 유지한다.
                // 다만 1×1 nested child가 든 행은 inner viewport의 물리 tail이
                // `advance_row_cut` 논리 높이보다 크게 그려질 수 있다. 이 tail을
                // 64px HWPX 일반 여유로 수용하면 다음 source unit이 현재 page clip
                // 뒤에 숨고 마지막 page가 사라진다 (#3637 HWP 2020 p26 → p27,
                // #2097 75544 p65 → p66). 새 continuation 시작 또는 새 표 안의
                // 후행 nested row라는 두 물리 경계에만 정확한 재-cut을 적용한다.
                const MIXED_NESTED_OWNER_DRIFT_MIN_PX: f64 = 16.0;
                // source owner가 drift하는 것은 현재 분할 row에 1×1 nested child가
                // 직접 있는 경우로 확인됐다. 1×1 child가 없는 giant cell(#1949)은
                // 같은 측정 차이를 보여도 이 보정 대상이 아니다.
                let row_has_single_cell_nested = table.cells.iter().any(|cell| {
                    cell.row as usize == r
                        && cell.paragraphs.iter().any(|paragraph| {
                            paragraph.controls.iter().any(|control| {
                                matches!(control, Control::Table(nested)
                                    if nested.row_count == 1 && nested.col_count == 1)
                            })
                        })
                });
                let continuation_nested_owner_boundary =
                    r == cursor_row && is_continuation && !row_start_cut.is_empty();
                // 일반 native continuation에 strict cut을 넓히면 원본 giant-cell이
                // 한컴 115쪽보다 1쪽 더 생긴다. 저장 뒤에도 남는 1열→2열 split
                // topology에만 실제 paint tail 재-cut을 허용한다 (#4138).
                let native_split_continuation_row_tail = st.profile.hwp5_stored_pagination_layout()
                    && mt.allows_row_break_split()
                    && r == cursor_row
                    && is_continuation
                    && !row_start_cut.is_empty()
                    && is_reparsed_single_column_cell_split_row(table, r);
                // 새 표의 앞선 행들이 현재 쪽에 먼저 놓인 뒤 마지막 1×1 child 행이
                // 시작될 때는 logical cut tail이 0px로 보고될 수 있다. 그러나 실제
                // child viewport·frame은 다음 쪽 source unit을 가리므로, 이 역시
                // 한 fragment owner로 취급해야 한다 (#2097 75544 p65 → p66).
                let fresh_late_nested_row =
                    r > cursor_row && !is_continuation && row_start_cut.is_empty();
                let nested_physical_tail = split_total > res.consumed_height + padding + 0.5;
                let mixed_nested_owner_guard = st.profile.hwpx_stored_layout()
                    && row_has_single_cell_nested
                    && (continuation_nested_owner_boundary
                        || (fresh_late_nested_row && !nested_physical_tail))
                    && split_candidate_rows_height - avail_for_rows
                        > MIXED_NESTED_OWNER_DRIFT_MIN_PX;
                // An ordinary stored HWPX RowBreak cut can differ from its
                // painted footprint only by the cell padding that the logical
                // CellUnit cut omits.  Preserve that measured difference; a
                // document-independent pixel tolerance would otherwise let
                // unrelated rows consume a physical page tail.
                let hwpx_stored_rowbreak_cut = st.profile.hwpx_stored_layout()
                    && !table.common.treat_as_char
                    && mt.allows_row_break_split()
                    && !mixed_nested_owner_guard;
                let measured_rowbreak_paint_tail =
                    (split_total - res.consumed_height - padding).max(0.0);
                let stored_frame_tail_overflow = if uses_source_frame_tail {
                    // `split_total` is the painted row footprint, whereas
                    // the source frame is selected in CellUnit content
                    // space. Admit exactly that selected frame's paint
                    // overfill, never an unrelated fixed allowance.
                    (split_candidate_rows_height - avail_for_rows).max(0.0)
                } else {
                    0.0
                };
                // Native HWP5 can record `common.height` through the leading
                // header row while the first physical fragment continues into
                // the next body row.  The stored frame's unused physical space
                // authorizes that next row only; admit the exact CellUnit
                // capacity overfill selected there, rather than turning the
                // entire frame slack into a general tolerance.
                let saved_first_fragment_next_row_cut = !is_continuation
                    && cursor_row == 0
                    && r > cursor_row
                    && row_start_cut.is_empty()
                    && source_first_fragment_overflow_allowance > 0.0
                    && source_first_fragment_row_end == Some(r);
                let saved_first_fragment_next_row_cut_overflow =
                    if saved_first_fragment_next_row_cut {
                        (res.consumed_height - budget).max(0.0)
                    } else {
                        0.0
                    };
                let split_row_overflow_tolerance = if uses_source_frame_tail {
                    stored_frame_tail_overflow
                } else if saved_first_fragment_next_row_cut {
                    saved_first_fragment_next_row_cut_overflow
                } else if source_first_fragment_overflow_allowance > 0.0
                    && source_first_fragment_row_end == Some(r + 1)
                {
                    source_first_fragment_overflow_allowance
                } else if hwpx_stored_rowbreak_cut {
                    measured_rowbreak_paint_tail
                } else if native_split_continuation_row_tail || mixed_nested_owner_guard {
                    0.1
                } else {
                    0.1
                };
                if (r > cursor_row
                    || mixed_nested_owner_guard
                    || native_split_continuation_row_tail)
                    && split_candidate_rows_height > avail_for_rows + split_row_overflow_tolerance
                {
                    // 보이는 조각은 orphan 기준을 통과해도 row-area 예산은 넘을 수 있다.
                    // [#2070] 종전에는 즉시 통이월했으나, advance_row_cut 이 예산을
                    // 수 px 초과하는 컷을 고른 경우(80168 pi=936: budget 903.1 에
                    // consumed 921.7, cand 957.9 > avail 941.3 → 행 통짜 이월로 3쪽)
                    // 한글은 같은 자리에서 조각 분할을 시작한다(PDF p108). 초과분만큼
                    // 예산을 줄여 한 번 재시도하고, 그래도 초과면 종전대로 이월한다.
                    let over = split_candidate_rows_height - avail_for_rows;
                    // Ordinary overfill requires only the measured excess.
                    // The guarded mixed-nested form has an additional physical
                    // tail that is absent from `advance_row_cut`'s logical
                    // height; reserve it too, so the next page begins at the
                    // first omitted source unit rather than one line late.
                    let painted_tail = (split_total - res.consumed_height - padding).max(0.0);
                    let retry_uses_painted_tail =
                        mixed_nested_owner_guard || native_split_continuation_row_tail;
                    let retry_budget = if retry_uses_painted_tail {
                        (budget - over - painted_tail - 0.5).max(0.0)
                    } else {
                        (budget - over - 0.5).max(0.0)
                    };
                    let (res2, retry_budget) = layout_engine
                        .advance_row_cut_with_mixed_nested_reserve(
                            table,
                            r,
                            row_start_cut,
                            retry_budget,
                            styles,
                        );
                    let mut retried = false;
                    if !res2.fully_consumed {
                        let split_total2 = layout_engine.row_cut_content_height(
                            table,
                            r,
                            row_start_cut,
                            &res2.end_cut,
                            styles,
                            false,
                        );
                        let cand2 = consumed + cs_before + split_total2;
                        let retry_split_row_overflow_tolerance = if uses_source_frame_tail {
                            stored_frame_tail_overflow
                        } else if saved_first_fragment_next_row_cut {
                            (res2.consumed_height - retry_budget).max(0.0)
                        } else if source_first_fragment_overflow_allowance > 0.0
                            && source_first_fragment_row_end == Some(r + 1)
                        {
                            source_first_fragment_overflow_allowance
                        } else if hwpx_stored_rowbreak_cut {
                            (split_total2 - res2.consumed_height - padding).max(0.0)
                        } else if native_split_continuation_row_tail || mixed_nested_owner_guard {
                            0.1
                        } else {
                            0.1
                        };
                        if row_split_meets_min_top_keep(
                            res2.consumed_height,
                            split_total2,
                            row_split_min_keep_uses_painted_height,
                        ) && cand2 <= avail_for_rows + retry_split_row_overflow_tolerance
                        {
                            end_row = r + 1;
                            split_end_cut = res2.end_cut.clone();
                            split_end_limit = res2.consumed_height;
                            consumed += cs_before + split_total2;
                            retried = true;
                        }
                    }
                    // `end_row = r` 는 "이 행을 통째로 다음 쪽으로 이월" 이라는 뜻
                    // 이므로, 행 앞에 이미 배치된 행이 있을 때만 성립한다. 그러나
                    // `r == cursor_row` 인 continuation 조각은 이미 이 행 중간
                    // (`row_start_cut`)에서 시작하므로 이월할 앞부분이 없다. 이때
                    // 재시도 실패로 `end_row = r` 로 되돌리면 호출부가
                    // `end_row >= row_count && split_end_limit == 0` 을 "나머지가 이
                    // 쪽에 다 들어감" 으로 읽어 남은 유닛 전부를 클립 없이 한 쪽에
                    // 쏟는다. mixed-nested 재시도 예산은 실측 초과분(`over`)에서
                    // painted tail 을 한 번 더 빼므로 이 tail 이 큰 거대 셀에서는
                    // 예산이 0 에 수렴해 이 0-전진 경로로 떨어진다
                    // (table_giant_cell_overfill: budget 1005.4 → retry 12.4,
                    // 남은 4,577px 이 39쪽 한 장에 겹쳐 렌더). 예산 컷 `res` 자체는
                    // orphan 기준을 통과한 유효한 전진이므로, 0-전진 대신 그 컷을
                    // 쓴다.
                    let continuation_row_must_advance = r == cursor_row
                        && is_continuation
                        && !row_start_cut.is_empty()
                        && !res.end_cut.is_empty()
                        && res.consumed_height > 0.0
                        && row_split_meets_min_top_keep(
                            res.consumed_height,
                            split_total,
                            row_split_min_keep_uses_painted_height,
                        );
                    if !retried && continuation_row_must_advance {
                        end_row = r + 1;
                        split_end_cut = res.end_cut.clone();
                        split_end_limit = res.consumed_height;
                        consumed += cs_before + split_total;
                        retried = true;
                    }
                    if !retried {
                        end_row = r;
                    }
                } else {
                    end_row = r + 1;
                    split_end_cut = res.end_cut.clone();
                    split_end_limit = res.consumed_height;
                    consumed += cs_before + split_total;
                }
            }
            false
        })();
        let scan = BlockTableRowScan {
            consumed,
            end_row,
            split_block_start,
            split_end_cut,
            split_end_limit,
            end_row_height_override,
        };
        ScanStep {
            progress: ScanProgress {
                r,
                scan,
                bleed_absorbed_row_height,
            },
            keep_scanning,
        }
    }
}

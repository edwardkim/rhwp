//! Read-only scan and footnote-aware refit. Returns cuts; does not emit page items.

use crate::renderer::typeset::{
    controls, row_geometry_table, table, table_declared_object_covers_cell_row_frames,
    BlockRowScanVars, BlockTableRowScan, Control, Footnote, TypesetEngine, TypesetState,
};

use super::super::TableContinuationCursor;
use super::{FragmentBudget, FragmentInput, FragmentProfile};

impl TypesetEngine {
    pub(super) fn scan_table_fragment(
        &self,
        st: &TypesetState,
        input: FragmentInput<'_>,
        budget: &FragmentBudget,
        profile: &mut FragmentProfile,
    ) -> BlockTableRowScan {
        let para_idx = input.source.para_index;
        let table = input.source.table;
        let row_geometry_table = input.source.row_geometry_table;
        let mt = input.source.measured_table;
        let styles = input.source.styles;
        let row_count = input.prepared.row_count;
        let cs = input.prepared.cell_spacing;
        let can_intra_split = input.prepared.can_intra_split;
        let layout_engine = &input.prepared.layout_engine;
        let rowspan_touched = &input.prepared.rowspan_touched;
        // [#7531] 쪽을 넘어온 걸침 칸의 끝 행은 앞 쪽 빈 띠를 덜어낸 높이로 잰다.
        let cut_row_h = input
            .start
            .relieved_cut_row_heights
            .as_deref()
            .or(input.fragment_row_heights)
            .unwrap_or(&input.prepared.cut_row_heights);
        let whole_row_fit_h = input
            .start
            .relieved_whole_row_fit_heights
            .as_deref()
            .or(input.fragment_row_heights)
            .unwrap_or(&input.prepared.whole_row_fit_heights);
        let total_footnote = input.prepared.total_footnote_height;
        let fn_margin = input.prepared.footnote_margin;
        let strict_following_plain_text_fit = input.prepared.strict_following_plain_text_fit;
        let source_next_positive_rewind = input.prepared.source_next_positive_rewind;
        let cursor_row = input.start.cursor_row;
        let is_continuation = input.start.is_continuation;
        let start_row_height_override = input.start.start_row_height_override;
        let start_cut = &input.start.start_cut;
        let FragmentBudget {
            scan_row_count,
            saved_first_fragment_source_frame,
            source_first_fragment_row_end,
            source_first_fragment_overflow_allowance,
            header_overhead,
            avail_for_rows,
            reflow_frame_avail_for_rows,
            ..
        } = *budget;
        let source_cut_heights = budget.source_complete_frame_last_row.map(|(row, height)| {
            let mut heights = cut_row_h.to_vec();
            heights[row] = height;
            heights
        });
        let source_fit_heights = budget.source_complete_frame_last_row.map(|(row, height)| {
            let mut heights = whole_row_fit_h.to_vec();
            heights[row] = height;
            heights
        });
        let cut_row_h = source_cut_heights.as_deref().unwrap_or(cut_row_h);
        let whole_row_fit_h = source_fit_heights.as_deref().unwrap_or(whole_row_fit_h);
        // The original content cut and the two boxes are one accepted result.
        // A blank band is reserved here, before emit and paint, and never
        // consumed as an extra content unit on the following page.
        if let Some(frames) = input.prepared.source_single_cell_row_frames.filter(|_| {
            input.prepared.table_footnotes.is_empty()
                && std::ptr::eq(table, row_geometry_table)
                && cursor_row == 0
        }) {
            let opening = !is_continuation && start_cut.is_empty();
            let closing = is_continuation && start_cut.as_slice() == [frames.opening_end];
            let height = if opening {
                frames.opening_height
            } else {
                frames.closing_height
            };
            if (opening || closing) && height <= avail_for_rows {
                return BlockTableRowScan {
                    source_frame_band: 0.0,
                    consumed: height,
                    accepted_end_row_box: None,
                    end_row: 1,
                    split_block_start: None,
                    split_end_cut: if opening {
                        vec![frames.opening_end]
                    } else {
                        Vec::new()
                    },
                    split_end_limit: if opening { height } else { 0.0 },
                    end_row_height_override: Some(height),
                };
            }
        }
        // 실제 기존 각주 경계 안에 들어오는 것으로 준비 단계에서 확인한 빈
        // 시작 조각은 유닛을 소비하지 않는다. 양수 물리 전진과 0 컷을 함께
        // 반환해 다음 조각이 같은 그림·캡션을 처음부터 소유하게 한다.
        if !is_continuation && cursor_row == 0 && start_cut.is_empty() {
            if let Some(frame) = input.prepared.empty_opening_row_frame {
                if frame.opening_height <= avail_for_rows + 0.5 {
                    return BlockTableRowScan {
                        source_frame_band: 0.0,
                        consumed: frame.opening_height,
                        accepted_end_row_box: None,
                        end_row: 1,
                        split_block_start: None,
                        split_end_cut: vec![0],
                        split_end_limit: frame.opening_height,
                        end_row_height_override: Some(frame.opening_height),
                    };
                }
            }
        }
        // A synchronized source reset closes the first physical row box.
        // Decide its demand before accepting content; the same source plan is
        // carried by emission and consumed by the partial-table painter.
        if !is_continuation && cursor_row == 0 && start_cut.is_empty() && header_overhead == 0.0 {
            if let Some((cut, opening, _)) = layout_engine.saved_first_row_opening_frame(
                table,
                styles,
                st.layout.body_area.height,
            ) {
                if std::env::var_os("RHWP_DIAG_SCAN").is_some() {
                    eprintln!("DIAG_SCAN SOURCE_FIRST_ROW_FRAME pi={para_idx} cut={cut:?} opening={opening:.3} available={avail_for_rows:.3}");
                }
                if opening <= avail_for_rows + 0.1 {
                    return BlockTableRowScan {
                        source_frame_band: 0.0,
                        consumed: opening,
                        accepted_end_row_box: None,
                        end_row: 1,
                        split_block_start: None,
                        split_end_cut: cut,
                        split_end_limit: opening,
                        end_row_height_override: Some(opening),
                    };
                }
                // No accepted content or physical space on this page. Keep
                // the exact source cut for a fresh page rather than forcing a
                // smaller box now and enlarging it during paint.
                if !st.current_items.is_empty()
                    || st.current_height > 0.0
                    || st.current_footnote_height > 0.0
                {
                    return BlockTableRowScan {
                        source_frame_band: 0.0,
                        consumed: 0.0,
                        accepted_end_row_box: None,
                        end_row: cursor_row,
                        split_block_start: None,
                        split_end_cut: Vec::new(),
                        split_end_limit: 0.0,
                        end_row_height_override: None,
                    };
                }
            }
        }
        // [Task #993] 컷 기반 행 경계 walk — cursor_row 부터 avail_for_rows
        // 안에 들어가는 행을 advance_row_cut(단일 권위 함수)으로 누적 배치한다.
        // 예산을 못 채우거나 vpos 리셋(hard break)을 만난 첫 행이 분할 행이
        // 된다. rowspan 보호 블록(#398/#474)은 블록 전체를 한 단위로 다룬다.
        // 측정 공간이 advance_row_cut/cell_units 로 단일화되어 렌더러와
        // 정의상 일치한다(px content_offset·MeasuredTable 누적 제거).
        const LANDSCAPE_ROWBREAK_WHOLE_ROW_TOLERANCE_PX: f64 = 36.0;
        const LANDSCAPE_ROWBREAK_SHORT_ROW_TOLERANCE_PX: f64 = 260.0;
        const LANDSCAPE_ROWBREAK_SHORT_ROW_MAX_HEIGHT_PX: f64 = 260.0;
        const HWPX_LANDSCAPE_ROWBREAK_WHOLE_ROW_TOLERANCE_PX: f64 = 48.0;
        const HWPX_LANDSCAPE_ROWBREAK_SHORT_ROW_TOLERANCE_PX: f64 = 320.0;
        const HWPX_LANDSCAPE_ROWBREAK_SHORT_ROW_MAX_HEIGHT_PX: f64 = 320.0;
        let landscape_rowbreak_bleed = st.layout.body_area.height < 700.0;
        let landscape_whole_row_tolerance = if st.profile.hwpx_stored_layout() {
            HWPX_LANDSCAPE_ROWBREAK_WHOLE_ROW_TOLERANCE_PX
        } else {
            LANDSCAPE_ROWBREAK_WHOLE_ROW_TOLERANCE_PX
        };
        let landscape_short_row_tolerance = if st.profile.hwpx_stored_layout() {
            HWPX_LANDSCAPE_ROWBREAK_SHORT_ROW_TOLERANCE_PX
        } else {
            LANDSCAPE_ROWBREAK_SHORT_ROW_TOLERANCE_PX
        };
        let landscape_short_row_max_height = if st.profile.hwpx_stored_layout() {
            HWPX_LANDSCAPE_ROWBREAK_SHORT_ROW_MAX_HEIGHT_PX
        } else {
            LANDSCAPE_ROWBREAK_SHORT_ROW_MAX_HEIGHT_PX
        };
        // [Task #1025] split_block_start: 블록 분할 시 연속분 커서 복귀 기록.
        let issue2424_scan_started = profile.enabled.then(std::time::Instant::now);
        let scan_vars = BlockRowScanVars {
            cursor_row,
            row_count: scan_row_count,
            cs,
            can_intra_split,
            is_continuation,
            avail_for_rows,
            reflow_frame_avail_for_rows,
            header_overhead,
            landscape_rowbreak_bleed,
            landscape_whole_row_tolerance,
            landscape_short_row_tolerance,
            landscape_short_row_max_height,
            strict_painted_bottom_fit: strict_following_plain_text_fit,
            source_first_fragment_overflow_allowance,
            source_first_fragment_row_end,
            start_cut_is_block: input.start.start_cut_is_block,
            start_row_height_override,
            source_terminal_suffix_start: input.prepared.source_terminal_suffix_start,
        };
        let run_scan = |vars: BlockRowScanVars| {
            let mut scan = self.scan_block_table_split_rows(
                st,
                layout_engine,
                mt,
                row_geometry_table,
                styles,
                cut_row_h,
                whole_row_fit_h,
                input.fragment_row_heights.unwrap_or(&mt.row_heights),
                rowspan_touched,
                &start_cut,
                &input.start.straddle_page_bands,
                vars,
                BlockTableRowScan {
                    source_frame_band: 0.0,
                    consumed: 0.0,
                    accepted_end_row_box: None,
                    end_row: cursor_row,
                    split_block_start: None,
                    split_end_cut: Vec::new(),
                    split_end_limit: 0.0,
                    end_row_height_override: None,
                },
            );
            // 원본 첫 상자가 본문 하단을 닫는 경우, 후속 쪽의 공통 저장 컷은
            // 잉크뿐 아니라 남은 빈 밴드도 소유한다. 각주 재수용도 이 closure를
            // 다시 호출하므로 현재 실제 예산 안에서만 물리 프레임을 수용한다.
            if is_continuation
                && scan.split_block_start.is_none()
                && scan.split_end_limit > 0.0
                && st.current_zone_y_offset == 0.0
                && layout_engine.original_body_closed_table_frame(
                    input.source.paragraph,
                    table,
                    st.layout.body_area.height,
                )
                && scan.end_row.checked_sub(1).is_some_and(|row| {
                    layout_engine.row_cut_closes_shared_original_frame(
                        table,
                        row,
                        &scan.split_end_cut,
                        styles,
                    ) || layout_engine.row_cut_closes_original_restart_frame(
                        table,
                        row,
                        &scan.split_end_cut,
                        input.fragment_row_heights.unwrap_or(&mt.row_heights),
                        styles,
                    )
                })
            {
                let top = budget.fragment_placement.map_or(
                    st.current_height + budget.host_before_overhead + budget.vert_offset_overhead,
                    |placement| placement.table_top,
                ) + budget.caption_extra;
                let frame_height =
                    crate::renderer::float_placement::single_cell_page_fragment_bottom(
                        table,
                        st.layout.body_area.height,
                        self.dpi,
                    ) - top
                        - vars.header_overhead;
                if frame_height >= scan.consumed && frame_height <= vars.avail_for_rows {
                    let last_height = scan.end_row_height_override.unwrap_or_else(|| {
                        let row = scan.end_row - 1;
                        let row_start = if row == cursor_row {
                            start_cut.as_slice()
                        } else {
                            &[]
                        };
                        // split_end_limit은 내용 유닛 전진량이다. 안여백 등을 포함한
                        // 실제 행 점유를 바탕으로 빈 밴드를 더해야 같은 상자를 예약한다.
                        layout_engine.row_cut_content_height(
                            row_geometry_table,
                            row,
                            row_start,
                            &scan.split_end_cut,
                            styles,
                        )
                    });
                    scan.end_row_height_override = Some(last_height + frame_height - scan.consumed);
                    scan.consumed = frame_height;
                    if std::env::var("RHWP_TABLE_DRIFT").is_ok() {
                        eprintln!("SOURCE_BODY_FRAME: pi={} end_row={} end_cut={:?} frame_h={:.3} last_h={:.3}",
                            para_idx, scan.end_row, scan.split_end_cut, scan.consumed,
                            scan.end_row_height_override.unwrap_or(0.0));
                    }
                }
            }
            scan
        };
        let BlockTableRowScan {
            mut source_frame_band,
            mut consumed,
            mut accepted_end_row_box,
            mut end_row,
            mut split_block_start,
            mut split_end_cut,
            mut split_end_limit,
            mut end_row_height_override,
        } = run_scan(scan_vars);
        if let Some(started) = issue2424_scan_started {
            profile.scan.0 += started.elapsed();
            profile.scan.1 += 1;
        }
        if end_row <= cursor_row {
            end_row = cursor_row + 1;
        }
        if split_end_cut.is_empty() {
            if let Some((row, height)) = budget.source_complete_frame_last_row {
                if end_row == row + 1 {
                    end_row_height_override = Some(height);
                }
            }
        }
        // 첫 source fragment가 선택한 마지막 행은 scanner에서는 measured
        // boundary까지 소비하지만, paint는 common object frame의 남은 물리 높이로
        // 끝나야 한다. 이 값은 source frame과 그 직전 행들의 합으로 계산한다.
        if source_next_positive_rewind
            && budget.source_complete_frame_last_row.is_none()
            && !table_declared_object_covers_cell_row_frames(table, self.dpi)
            && source_first_fragment_row_end == Some(end_row)
            && (split_end_limit <= 0.0
                || (split_block_start.is_none()
                    && end_row_height_override.is_some_and(|height| {
                        layout_engine.row_complete_cut_content_height(table, end_row - 1, styles)
                            <= height + 0.5
                    })
                    && table
                        .cells
                        .iter()
                        .filter(|cell| cell.row as usize == end_row - 1)
                        .all(|cell| {
                            cell.row_span == 1
                                && cell.vertical_align == crate::model::table::VerticalAlign::Top
                        })))
        {
            if let Some((frame_height, _)) = saved_first_fragment_source_frame {
                let before_last = cut_row_h
                    .iter()
                    .take(end_row.saturating_sub(1))
                    .sum::<f64>()
                    + cs * end_row.saturating_sub(2) as f64;
                end_row_height_override = Some((frame_height - before_last).max(0.0));
                // The accepted source frame replaces the physical height of
                // this opening fragment. Its content cut does not change,
                // but the reservation must include the same blank band that
                // paint receives through the last-row override. Otherwise
                // emit reserves that band a second time as unused page space.
                let source_frame_height = before_last + end_row_height_override.unwrap_or(0.0);
                source_frame_band = (source_frame_height - consumed).max(0.0);
                consumed = source_frame_height;
                // 내용이 끝난 일반 행도 같은 저장 물리 프레임을 사용한다.
                // 내용 예산으로 빈 밴드를 계산하면 다음 쪽이 그 차이만큼
                // 밀린다. 그리는 쪽 상한 절삭은 별도이며 선언 공간을 줄이지 않는다.
            }
        }
        // The accepted source frame owns physical space independently of its
        // complete content cut. A protected next block can stop the row walk
        // before the nearest source-frame row candidate. Keep that actual cut,
        // and account for the frame's blank tail in both reservation and paint.
        if !is_continuation
            && cursor_row == 0
            && start_cut.is_empty()
            && end_row > 0
            && end_row < row_count
            && split_block_start.is_none()
            && split_end_cut.is_empty()
            && split_end_limit <= 0.0
            // Minimum growth keeps the complete source row and its saved
            // blank tail. A compressed row has a different physical cut.
            && end_row_height_override
                .is_none_or(|height| height >= whole_row_fit_h[end_row - 1])
            && (st.profile.hwp5_stored_pagination_layout() || st.profile.hwpx_stored_layout())
            && !st.profile.session_edited()
            && !input.source.paragraph.stored_text_partition_is_dirty()
            && !self.render_normalization.table_text_reflowed(table)
        {
            if let Some((frame, _)) = saved_first_fragment_source_frame {
                let whole_rows = whole_row_fit_h[..end_row].iter().sum::<f64>()
                    + cs * end_row.saturating_sub(1) as f64;
                if frame > consumed
                    && frame <= avail_for_rows + 0.5
                    && (whole_rows - consumed).abs() <= 0.5
                {
                    let before_last = whole_rows - whole_row_fit_h[end_row - 1];
                    end_row_height_override = Some(frame - before_last);
                    consumed = frame;
                }
            }
        }
        // [#3674 진단] 표 행 분할 스캔 입력/결과 — 동작 불변.
        if std::env::var("RHWP_DIAG_SPLITSCAN").is_ok() {
            eprintln!(
                "DIAG_SPLITSCAN pi={} sec={} cursor={} end_row={} consumed={:.1} avail={:.1} hdr={:.1} rows={} intra={} cont={}",
                para_idx, st.section_index, cursor_row, end_row, consumed, avail_for_rows,
                header_overhead, row_count, can_intra_split, is_continuation,
            );
        }

        // [#2097] 첫 조각 각주 예약-컷 재정합 — 한글은 각주를 앵커 줄과 함께
        // 움직이므로(인서트-인지 컷), 표 전체 각주 선-예약(available 차감)은
        // 앵커가 첫 조각에 없을 때 컷 예산만 잠식한다 (2572521 p3: 예약
        // 111.1px == 컷 부족분, 한글 p3 각주 없음 + 잔여 94.4px = 앵커
        // 줄+각주 크기 실측 정합). 앵커 전부가 조각 밖이면 예약을 해제하되
        // 첫 앵커 직전을 예산 상한으로 재스캔한다 (앵커 줄은 자기 각주와
        // 함께 다음 조각으로 — 한글 규칙).
        let table_fn_reserved = (total_footnote - st.current_footnote_height).max(0.0)
            + if st.current_footnote_height <= 0.0 {
                fn_margin
            } else {
                0.0
            };
        let is_split_fragment = end_row < row_count || split_end_limit > 0.0;
        if table_fn_reserved > 0.5
            && !is_continuation
            && cursor_row == 0
            && start_cut.is_empty()
            && is_split_fragment
        {
            let mut anchor_offsets: Vec<f64> = Vec::new();
            let mut anchor_unresolved = false;
            for cell in &row_geometry_table.cells {
                for (cp_idx, cp) in cell.paragraphs.iter().enumerate() {
                    if !cp
                        .controls
                        .iter()
                        .any(|c| matches!(c, Control::Footnote(_)))
                    {
                        continue;
                    }
                    let row = (cell.row as usize).min(row_count.saturating_sub(1));
                    let row_prefix: f64 = (0..row).map(|x| cut_row_h[x] + cs).sum();
                    match layout_engine.cell_para_unit_offset(
                        cell,
                        row_geometry_table,
                        styles,
                        cp_idx,
                    ) {
                        Some(off) => anchor_offsets.push(row_prefix + off),
                        None => anchor_unresolved = true,
                    }
                }
            }
            let min_anchor = anchor_offsets.iter().copied().fold(f64::INFINITY, f64::min);
            let all_beyond =
                !anchor_unresolved && !anchor_offsets.is_empty() && min_anchor >= consumed - 0.5;
            if all_beyond {
                let pad = if mt.allows_row_break_split() {
                    layout_engine.row_remaining_visible_padding_height(
                        row_geometry_table,
                        cursor_row,
                        &[],
                        styles,
                    )
                } else {
                    mt.max_padding_for_row(cursor_row)
                };
                let avail_refit = (avail_for_rows + table_fn_reserved).min(min_anchor + pad + 0.1);
                if avail_refit > avail_for_rows + 0.5 {
                    let issue2424_refit_started = profile.enabled.then(std::time::Instant::now);
                    let refit = run_scan(BlockRowScanVars {
                        avail_for_rows: avail_refit,
                        ..scan_vars
                    });
                    if let Some(started) = issue2424_refit_started {
                        profile.refit.0 += started.elapsed();
                        profile.refit.1 += 1;
                    }
                    // 재스캔 조각도 앵커 미포함일 때만 채택 (상한이 보증하나
                    // squeeze 허용치로 소폭 넘을 수 있어 재확인).
                    if refit.consumed > consumed + 0.5 && refit.consumed <= min_anchor + pad + 0.5 {
                        if std::env::var("RHWP_DIAG_SCAN").is_ok() {
                            eprintln!(
                                "DIAG_SCAN FN_REFIT pi={} consumed {:.1} -> {:.1} reserved={:.1} min_anchor={:.1}",
                                para_idx,
                                consumed,
                                refit.consumed,
                                table_fn_reserved,
                                min_anchor
                            );
                        }
                        source_frame_band = refit.source_frame_band;
                        consumed = refit.consumed;
                        end_row = refit.end_row;
                        split_block_start = refit.split_block_start;
                        split_end_cut = refit.split_end_cut;
                        split_end_limit = refit.split_end_limit;
                        end_row_height_override = refit.end_row_height_override;
                        accepted_end_row_box = refit.accepted_end_row_box;
                        if end_row <= cursor_row {
                            end_row = cursor_row + 1;
                        }
                    }
                }
            }
        }

        // 아래 캡션과 종료 바깥 여백이 마지막 유닛을 닫는다.
        // 시작 캡션은 이미 첫 조각 예산에서 계상됐다.
        // 현재 쪽에서 마지막 유닛을 닫을 수 없으면 수용한 앞 조각을 다시 스캔한다.
        // 높이·컷을 확정한 뒤 end_row만 바꾸지 않는다.
        if end_row >= row_count
            && (split_end_limit == 0.0
                || input.closes_terminal_content_cut(
                    end_row,
                    split_end_limit,
                    &split_end_cut,
                    split_block_start,
                    end_row_height_override,
                ))
            && (input.prepared.caption_overhead > 0.0
                || budget.terminal_outer_bottom_overhead > budget.fragment_outer_bottom_overhead)
        {
            let closing_overhead = if input.prepared.caption_is_top {
                // 시작 캡션은 이미 page_avail에서 뺐다.
                0.0
            } else {
                input.prepared.caption_overhead
            } + (budget.terminal_outer_bottom_overhead
                - budget.fragment_outer_bottom_overhead)
                .max(0.0);
            let closing_height = consumed + header_overhead + closing_overhead;
            if closing_height > budget.page_avail {
                let rows = table::scan::RowBlockQuery {
                    layout_engine,
                    mt,
                    table: row_geometry_table,
                    styles,
                    cut_row_h,
                    rowspan_touched,
                    cs,
                    straddle_page_bands: &input.start.straddle_page_bands,
                };
                let last_range = rows.candidate(row_count - 1);
                let last = rows.candidate(last_range.b_start);
                let terminal_start = if last.protected || last.rowbreak_rowspan_block {
                    last.b_start
                } else {
                    row_count - 1
                };
                if terminal_start <= cursor_row
                    && st.current_height > 0.0
                    && !st.current_items.is_empty()
                    && closing_height + budget.caption_extra + budget.host_before_overhead
                        <= st.base_available_height()
                {
                    // 앞선 표 유닛을 하나도 확정할 수 없으므로 커서를 보존한 채
                    // 실제 앞 항목이 점유한 쪽을 비운다. 다른 항목이 없는 프레임의
                    // 양수 저장 앵커 때문에 이월을 무한 반복하지 않아야 한다.
                    return BlockTableRowScan {
                        source_frame_band: 0.0,
                        consumed: 0.0,
                        accepted_end_row_box: None,
                        end_row: cursor_row,
                        split_block_start: None,
                        split_end_cut: Vec::new(),
                        split_end_limit: 0.0,
                        end_row_height_override: None,
                    };
                }
                let refit = run_scan(if terminal_start > cursor_row {
                    BlockRowScanVars {
                        row_count: terminal_start,
                        ..scan_vars
                    }
                } else {
                    BlockRowScanVars {
                        avail_for_rows: (avail_for_rows - closing_overhead).max(0.0),
                        ..scan_vars
                    }
                });
                if refit.end_row > cursor_row && refit.consumed > 0.0 {
                    source_frame_band = refit.source_frame_band;
                    consumed = refit.consumed;
                    end_row = refit.end_row;
                    split_block_start = refit.split_block_start;
                    split_end_cut = refit.split_end_cut;
                    split_end_limit = refit.split_end_limit;
                    end_row_height_override = refit.end_row_height_override;
                    accepted_end_row_box = refit.accepted_end_row_box;
                }
            }
        }

        // Refit has finished: only now can a complete continuation reserve its
        // remaining physical band without changing content ownership.
        if is_continuation
            && start_cut.is_empty()
            && end_row > cursor_row
            && end_row < row_count
            && split_block_start.is_none()
            && split_end_cut.is_empty()
            && split_end_limit <= 0.0
            // A minimum extension still owns the complete row. Only a
            // compressed band prevents the saved frame from consuming its tail.
            && end_row_height_override
                .is_none_or(|height| height >= whole_row_fit_h[end_row - 1])
            && self.stored_header_physical_rows(st, input.source)
            && row_geometry_table.cells.iter().any(|cell| {
                let first = cell.row as usize;
                first < end_row && first + cell.row_span as usize > end_row
            })
        {
            let whole_rows = whole_row_fit_h[cursor_row..end_row].iter().sum::<f64>()
                + cs * (end_row - cursor_row).saturating_sub(1) as f64;
            // Logical line advances selected the content cut above. Only its
            // unused physical tail uses the same closing boundary as paint;
            // subtracting that band before scanning would move valid units.
            let physical_rows_available = self.stored_header_fragment_row_budget(
                st,
                input,
                budget.page_avail,
                budget.host_before_overhead,
            ) - header_overhead;
            if physical_rows_available > consumed && (whole_rows - consumed).abs() <= 0.5 {
                let before_last = whole_rows - whole_row_fit_h[end_row - 1];
                end_row_height_override = Some(physical_rows_available - before_last);
                consumed = physical_rows_available;
            }
        }

        if split_block_start.is_none()
            && !split_end_cut.is_empty()
            && split_end_limit > 0.0
            && end_row_height_override.is_none()
            && end_row > cursor_row
        {
            // split_end_limit counts units only. Padding already belongs to the
            // accepted last-row box and cannot also belong to the prefix.
            let row = end_row - 1;
            let row_start = if row == cursor_row {
                start_cut.as_slice()
            } else {
                &[]
            };
            let selected = layout_engine.row_cut_content_height(
                row_geometry_table,
                row,
                row_start,
                &split_end_cut,
                styles,
            );
            let before_last = consumed - selected;
            let source_frame_available =
                crate::renderer::float_placement::single_cell_page_fragment_bottom(
                    table,
                    st.available_height(),
                    self.dpi,
                ) - st.current_height
                    - budget.host_before_overhead
                    - budget.vert_offset_overhead
                    - budget.caption_extra
                    - header_overhead;
            if let Some(height) = layout_engine.stored_shared_row_end_frame(
                row_geometry_table,
                row,
                &split_end_cut,
                avail_for_rows.min(source_frame_available) - before_last,
                styles,
            ) {
                consumed += height - selected;
                end_row_height_override = Some(height);
            }
        }

        // 반복 제목행의 원본 쪽 프레임은 refit이 선택한 내용과 같은 물리 상자를 예약한다.
        // 내용을 다 소비한 컷에도 마지막 줄 간격만 남은 꼬리를 별도 행으로 세지 않는다.
        if split_block_start.is_none()
            && split_end_limit > 0.0
            && !split_end_cut.is_empty()
            && end_row > cursor_row
            && self.stored_header_physical_rows(st, input.source)
        {
            let row = end_row - 1;
            let row_start = if row == cursor_row {
                start_cut.as_slice()
            } else {
                &[]
            };
            let selected = end_row_height_override.unwrap_or_else(|| {
                layout_engine.row_cut_content_height(
                    row_geometry_table,
                    row,
                    row_start,
                    &split_end_cut,
                    styles,
                )
            });
            let before_last = consumed - selected;
            let physical = self.stored_header_fragment_row_budget(
                st,
                input,
                budget.page_avail,
                budget.host_before_overhead,
            ) - header_overhead
                - before_last;
            if let Some(height) = layout_engine.stored_ordinary_row_end_frame(
                row_geometry_table,
                row,
                row_start,
                &split_end_cut,
                physical.min(avail_for_rows - before_last),
                styles,
            ) {
                consumed += height - selected;
                end_row_height_override = Some(height);
            }
        }

        // 온전한 앞 제목만 소유한 원본 조각은 다음 인라인 상자를 가져오지
        // 않고 현재 셀의 최소 물리 밴드를 끝낸다. 같은 높이를 paint에 전달한다.
        if split_block_start.is_none()
            && end_row > cursor_row
            && layout_engine.stored_heading_before_inline_frame_cut(
                row_geometry_table,
                end_row - 1,
                if end_row - 1 == cursor_row {
                    start_cut
                } else {
                    &[]
                },
                &split_end_cut,
                styles,
            )
        {
            let row = end_row - 1;
            let selected = end_row_height_override.unwrap_or_else(|| {
                layout_engine.row_cut_content_height(
                    row_geometry_table,
                    row,
                    &[],
                    &split_end_cut,
                    styles,
                )
            });
            let top = budget.fragment_placement.map_or(
                st.current_height + budget.host_before_overhead + budget.vert_offset_overhead,
                |placement| placement.table_top,
            );
            let physical = st.layout.body_area.height
                - crate::renderer::typeset::hwpunit_to_px(
                    i32::from(table.outer_margin_bottom),
                    self.dpi,
                )
                - top
                - budget.caption_extra
                - header_overhead
                - (consumed - selected);
            let minimum = row_geometry_table
                .cells
                .iter()
                .filter(|cell| usize::from(cell.row) == row)
                .map(|cell| {
                    crate::renderer::typeset::hwpunit_to_px(
                        cell.height.min(i32::MAX as u32) as i32,
                        self.dpi,
                    )
                })
                .fold(0.0, f64::max);
            if physical >= selected
                && physical < minimum
                && physical <= avail_for_rows - (consumed - selected)
            {
                consumed += physical - selected;
                end_row_height_override = Some(physical);
            }
        }

        BlockTableRowScan {
            source_frame_band,
            consumed,
            accepted_end_row_box,
            end_row,
            split_block_start,
            split_end_cut,
            split_end_limit,
            end_row_height_override,
        }
    }
}

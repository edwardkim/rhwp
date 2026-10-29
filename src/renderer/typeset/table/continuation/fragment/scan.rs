//! Read-only scan and footnote-aware refit. Returns cuts; does not emit page items.

use crate::renderer::typeset::{
    controls, table, table_declared_object_covers_cell_row_frames, BlockRowScanVars,
    BlockTableRowScan, Control, Footnote, TypesetEngine, TypesetState,
};

use super::super::TableContinuationCursor;
use super::{FragmentBudget, FragmentInput, FragmentProfile};

// 스캔이 실제 수용한 행 상자에는 내용 컷 밖의 여백·빈 띠도 포함된다.
// 그 행을 내용 높이로 다시 빼면 그 차이가 앞 행 prefix에 중복 계상된다.
fn accepted_row_box(
    row: usize,
    height_override: Option<f64>,
    partial_row_box: Option<(usize, f64)>,
) -> Option<f64> {
    height_override.or_else(|| {
        partial_row_box
            .filter(|(owner, _)| *owner == row)
            .map(|(_, height)| height)
    })
}

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
            reflow_blank_avail_for_rows,
            reflow_blank_first_fragment_shortfall,
            reflow_frame_first_fragment_shortfall,
            first_fragment_shortfall,
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
                    plain_cut_owns_blank_band: false,
                    consumed: height,
                    accepted_end_row_box: None,
                    accepted_start_row_box: None,
                    accepted_partial_row_box: None,
                    end_row: 1,
                    split_block_start: None,
                    split_end_cut: if opening {
                        vec![frames.opening_end]
                    } else {
                        Vec::new()
                    },
                    split_end_limit: if opening { height } else { 0.0 },
                    split_end_reserved: 0.0,
                    split_block_physical_height: None,
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
                        plain_cut_owns_blank_band: false,
                        consumed: frame.opening_height,
                        accepted_end_row_box: None,
                        accepted_start_row_box: None,
                        accepted_partial_row_box: None,
                        end_row: 1,
                        split_block_start: None,
                        split_end_cut: vec![0],
                        split_end_limit: frame.opening_height,
                        split_end_reserved: 0.0,
                        split_block_physical_height: None,
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
                        plain_cut_owns_blank_band: false,
                        consumed: opening,
                        accepted_end_row_box: None,
                        accepted_start_row_box: None,
                        accepted_partial_row_box: None,
                        end_row: 1,
                        split_block_start: None,
                        split_end_cut: cut,
                        split_end_limit: opening,
                        split_end_reserved: 0.0,
                        split_block_physical_height: None,
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
                        plain_cut_owns_blank_band: false,
                        consumed: 0.0,
                        accepted_end_row_box: None,
                        accepted_start_row_box: None,
                        accepted_partial_row_box: None,
                        end_row: cursor_row,
                        split_block_start: None,
                        split_end_cut: Vec::new(),
                        split_end_limit: 0.0,
                        split_end_reserved: 0.0,
                        split_block_physical_height: None,
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
            reflow_blank_avail_for_rows,
            header_overhead,
            landscape_rowbreak_bleed,
            has_table_footnotes: !input.prepared.table_footnotes.is_empty(),
            landscape_whole_row_tolerance,
            landscape_short_row_tolerance,
            landscape_short_row_max_height,
            strict_painted_bottom_fit: strict_following_plain_text_fit,
            source_first_fragment_overflow_allowance,
            source_first_fragment_row_end,
            start_cut_is_block: input.start.start_cut_is_block,
            start_row_height_override,
            source_terminal_suffix_start: input.prepared.source_terminal_suffix_start,
            opening_block_cuts_after_first_row: input
                .prepared
                .opening_rowspan_block_cuts_after_first_row
                && !is_continuation
                && cursor_row == 0,
            source_carried_band_row: budget.source_carried_band_row,
            unreserved_outer_bottom: (crate::renderer::hwpunit_to_px(
                i32::from(table.outer_margin_bottom),
                self.dpi,
            ) - budget.fragment_outer_bottom_overhead)
                .max(0.0),
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
                    plain_cut_owns_blank_band: false,
                    consumed: 0.0,
                    accepted_end_row_box: None,
                    accepted_start_row_box: None,
                    accepted_partial_row_box: None,
                    end_row: cursor_row,
                    split_block_start: None,
                    split_end_cut: Vec::new(),
                    split_end_limit: 0.0,
                    split_end_reserved: 0.0,
                    split_block_physical_height: None,
                    end_row_height_override: None,
                },
            );
            // 원본 첫 상자가 본문 하단을 닫는 경우, 후속 쪽의 공통 저장 컷은
            // 잉크뿐 아니라 남은 빈 밴드도 소유한다. 각주 재수용도 이 closure를
            // 다시 호출하므로 현재 실제 예산 안에서만 물리 프레임을 수용한다.
            let original_body_frame_bottom = layout_engine.original_body_closed_table_bottom(
                input.source.paragraph,
                table,
                st.layout.body_area.height,
            );
            if let Some(frame_bottom) = original_body_frame_bottom.filter(|_| {
                is_continuation
                    && scan.split_block_start.is_none()
                    && scan.split_end_limit > 0.0
                    && st.current_zone_y_offset == 0.0
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
            }) {
                let top = budget.fragment_placement.map_or(
                    st.current_height + budget.host_before_overhead + budget.vert_offset_overhead,
                    |placement| placement.table_top,
                ) + budget.caption_extra;
                let frame_height = frame_bottom - top - vars.header_overhead;
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
            mut plain_cut_owns_blank_band,
            mut consumed,
            mut accepted_end_row_box,
            mut accepted_start_row_box,
            mut accepted_partial_row_box,
            mut end_row,
            mut split_block_start,
            mut split_end_cut,
            mut split_end_limit,
            mut split_end_reserved,
            mut split_block_physical_height,
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
                source_frame_band = (source_frame_band + source_frame_height - consumed).max(0.0);
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
        // [#7732] 표 각주를 이 쪽에서 빼면 남는 각주는 이미 정해진 쪽 각주뿐이라 경계가 확정이다.
        // 그 경계를 지키는 40px 여유도 함께 되돌린다(한/글은 각주 구분선 바로 위까지 행을 놓는다).
        let table_fn_reserved = (total_footnote - st.current_footnote_height).max(0.0) + fn_margin;
        let is_split_fragment = end_row < row_count || split_end_limit > 0.0;
        // [#7732] 앵커 줄이 자기 각주와 함께 다음 조각으로 간다는 #2097 가정은 한/글 출력과
        // 다르다. 한/글은 칸 각주 때문에 행을 줄이지 않고, 앵커가 이 조각에 있어도 각주는
        // 조각 뒤 남는 공간이나 표가 끝나는 쪽에 놓는다(합성 `gen_cell_notes_multipage.py`).
        // 그래서 표가 이 쪽에서 끝나지 않는 한 예약 전체를 되돌리고 앵커 위치로 상한을 두지 않는다.
        if table_fn_reserved > 0.5
            && !is_continuation
            && cursor_row == 0
            && start_cut.is_empty()
            && is_split_fragment
        {
            // [#8081] 0 으로 자른 첫 조각 예산에는 모자란 몫이 빠져 있다 — 되돌린 예약에서
            // 그 몫을 먼저 갚아야 실제 남은 공간이 된다.
            let avail_refit = avail_for_rows + table_fn_reserved - first_fragment_shortfall;
            if avail_refit > avail_for_rows + 0.5 {
                let issue2424_refit_started = profile.enabled.then(std::time::Instant::now);
                // 일반 행 컷이 따로 쓰는 물리 프레임 상한도 같은 몫을 되돌린다.
                let reflow_refit = reflow_frame_avail_for_rows.map(|height| {
                    height.max(height + table_fn_reserved - reflow_frame_first_fragment_shortfall)
                });
                let blank_refit = reflow_blank_avail_for_rows.map(|height| {
                    height.max(height + table_fn_reserved - reflow_blank_first_fragment_shortfall)
                });
                let refit = run_scan(BlockRowScanVars {
                    avail_for_rows: avail_refit,
                    reflow_frame_avail_for_rows: reflow_refit,
                    reflow_blank_avail_for_rows: blank_refit,
                    ..scan_vars
                });
                if let Some(started) = issue2424_refit_started {
                    profile.refit.0 += started.elapsed();
                    profile.refit.1 += 1;
                }
                // 되돌린 예산으로 표가 이 쪽에서 끝나면 각주도 이 쪽 몫이라 예약을 유지한다.
                let refit_continues = refit.end_row < row_count || refit.split_end_limit > 0.0;
                if refit.consumed > consumed + 0.5 && refit_continues {
                    if std::env::var("RHWP_DIAG_SCAN").is_ok() {
                        eprintln!(
                            "DIAG_SCAN FN_REFIT pi={} consumed {:.1} -> {:.1} reserved={:.1}",
                            para_idx, consumed, refit.consumed, table_fn_reserved,
                        );
                    }
                    source_frame_band = refit.source_frame_band;
                    plain_cut_owns_blank_band = refit.plain_cut_owns_blank_band;
                    consumed = refit.consumed;
                    end_row = refit.end_row;
                    split_block_start = refit.split_block_start;
                    split_end_cut = refit.split_end_cut;
                    split_end_limit = refit.split_end_limit;
                    split_end_reserved = refit.split_end_reserved;
                    split_block_physical_height = refit.split_block_physical_height;
                    end_row_height_override = refit.end_row_height_override;
                    accepted_end_row_box = refit.accepted_end_row_box;
                    accepted_start_row_box = refit.accepted_start_row_box;
                    accepted_partial_row_box = refit.accepted_partial_row_box;
                    if end_row <= cursor_row {
                        end_row = cursor_row + 1;
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
                    consumed,
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
                    carried_row_box: input
                        .start
                        .start_cut_is_block
                        .then_some(start_row_height_override)
                        .flatten()
                        .map(|height| (cursor_row, height)),
                };
                let last_range = rows.candidate(row_count - 1);
                let last = rows.candidate(last_range.b_start);
                // 아직 소비하지 않은 마지막 병합 블록과 종료 예산이 새 쪽에
                // 함께 들어가면 한 행만 이월해 소유를 갈라 놓지 않는다.
                // 새 쪽보다 큰 블록이나 이미 시작한 블록의 내부 컷은 유지한다.
                let (terminal_note_content, terminal_note_count) = input
                    .prepared
                    .table_footnotes
                    .iter()
                    .filter(|note| note.row >= last.b_start)
                    .fold((0.0, 0usize), |(height, count), note| {
                        (height + note.content_height, count + 1)
                    });
                let terminal_note_reservation = st
                    .projected_fresh_footnote_height(terminal_note_content, terminal_note_count)
                    + if terminal_note_count > 0 {
                        st.footnote_safety_margin
                    } else {
                        0.0
                    };
                let terminal_block_fits = last.block_size > 1
                    && last.b_start >= cursor_row
                    && rows.required_height(&last, &[])
                        + header_overhead
                        + closing_overhead
                        + budget.caption_extra
                        + budget.host_before_overhead
                        + terminal_note_reservation
                        <= st.base_available_height();
                let terminal_start =
                    if last.protected || last.rowbreak_rowspan_block || terminal_block_fits {
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
                        plain_cut_owns_blank_band: false,
                        consumed: 0.0,
                        accepted_end_row_box: None,
                        accepted_start_row_box: None,
                        accepted_partial_row_box: None,
                        end_row: cursor_row,
                        split_block_start: None,
                        split_end_cut: Vec::new(),
                        split_end_limit: 0.0,
                        split_end_reserved: 0.0,
                        split_block_physical_height: None,
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
                    plain_cut_owns_blank_band = refit.plain_cut_owns_blank_band;
                    consumed = refit.consumed;
                    end_row = refit.end_row;
                    split_block_start = refit.split_block_start;
                    split_end_cut = refit.split_end_cut;
                    split_end_limit = refit.split_end_limit;
                    split_end_reserved = refit.split_end_reserved;
                    split_block_physical_height = refit.split_block_physical_height;
                    end_row_height_override = refit.end_row_height_override;
                    accepted_end_row_box = refit.accepted_end_row_box;
                    accepted_start_row_box = refit.accepted_start_row_box;
                    accepted_partial_row_box = refit.accepted_partial_row_box;
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
            let selected = accepted_row_box(row, end_row_height_override, accepted_partial_row_box)
                .unwrap_or_else(|| {
                    layout_engine.row_cut_content_height(
                        row_geometry_table,
                        row,
                        row_start,
                        &split_end_cut,
                        styles,
                    )
                });
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
                split_end_reserved = height;
            }
        }
        // refit의 내용 컷과 물리 상자를 함께 예약하고 완료 컷의 작은 줄간격 꼬리는 새 행으로 만들지 않는다.
        if split_block_start.is_none()
            && split_end_limit > 0.0
            && !split_end_cut.is_empty()
            && end_row > cursor_row
        {
            let row = end_row - 1;
            let row_start = if row == cursor_row {
                start_cut.as_slice()
            } else {
                &[]
            };
            let selected = accepted_row_box(row, end_row_height_override, accepted_partial_row_box)
                .unwrap_or_else(|| {
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
            let available = physical.min(
                budget
                    .reflow_blank_avail_for_rows
                    .unwrap_or(avail_for_rows)
                    .min(avail_for_rows)
                    - before_last,
            );
            // 두 저장 프레임·후속 원점이 닫은 상자는 임시 예약 대신 실제 소유 컷의 내용 하한과 비교한다.
            let source_opening = (!is_continuation
                && cursor_row == 0
                && start_cut.is_empty()
                && end_row == 1
                && input.prepared.table_footnotes.is_empty()
                && std::ptr::eq(table, row_geometry_table))
            .then(|| {
                input
                    .source
                    .paragraphs_all
                    .get(para_idx + 1)
                    .and_then(|next| {
                        layout_engine.stored_single_cell_two_frame_opening(
                            input.source.paragraph,
                            next,
                            table,
                            &split_end_cut,
                            styles,
                        )
                    })
            })
            .flatten()
            .filter(|height| {
                *height
                    >= layout_engine.row_cut_content_height(
                        row_geometry_table,
                        row,
                        row_start,
                        &split_end_cut,
                        styles,
                    )
                    && *height <= available
            });
            // 인라인 반복 제목행은 유지하며 일반 칸의 원본 최소·저장 컷은 제목행 유무와 별도로 확인한다.
            let inline_frame = self
                .stored_header_physical_rows(st, input.source)
                .then(|| {
                    layout_engine
                        .stored_inline_restart_row_end_frame(
                            row_geometry_table,
                            row,
                            row_start,
                            &split_end_cut,
                            available,
                            styles,
                        )
                        .or_else(|| {
                            layout_engine.stored_inline_picture_preceding_row_frame(
                                row_geometry_table,
                                row,
                                row_start,
                                &split_end_cut,
                                available,
                                styles,
                            )
                        })
                })
                .flatten()
                .filter(|height| *height >= selected);
            if let Some(height) = source_opening.or(inline_frame).or_else(|| {
                let carried = (row == cursor_row && !input.start.start_cut_is_block)
                    .then_some(start_row_height_override)
                    .flatten();
                layout_engine.stored_ordinary_row_end_frame(
                    row_geometry_table,
                    row,
                    (row_start, &split_end_cut),
                    available,
                    carried,
                    styles,
                )
            }) {
                if inline_frame.is_some() {
                    source_frame_band += height - selected;
                }
                consumed += height - selected;
                end_row_height_override = Some(height);
                split_end_reserved = height;
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
            let selected = accepted_row_box(row, end_row_height_override, accepted_partial_row_box)
                .unwrap_or_else(|| {
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
                split_end_reserved = physical;
            }
        }

        // 내용을 완료한 행도 최종 수용한 원본 상자에 물리 잔여가 있을 수 있다.
        // 유닛과 수용 상자를 여기서 함께 확정해 emit이 빈 컷을 다음 행으로
        // 해석하면서 독립 최소를 버리지 않게 한다. 블록의 셀 순서 컷과
        // 기존 부분 내용 컷은 각 원장의 소유를 유지한다.
        if split_block_start.is_none()
            && split_end_cut.is_empty()
            && split_end_limit <= 0.0
            && end_row > cursor_row
            && end_row < row_count
        {
            if let Some(height) = end_row_height_override {
                if let Some(cut) = layout_engine.stored_complete_row_physical_tail_cut(
                    row_geometry_table,
                    end_row - 1,
                    height,
                    styles,
                ) {
                    split_end_cut = cut;
                    split_end_limit = height;
                    split_end_reserved = height;
                }
            }
        }

        // 재조판 호스트의 자리차지 조각은 수용한 행의 물리 상자를 쪽 끝까지
        // 소유한다. 내용 유닛 컷은 늘리지 않고, 같은 예산 안의 빈 띠만 예약한다.
        // 저장 프레임·걸침 블록이 이미 높이를 소유하면 그 결과를 유지한다.
        if input.prepared.computed_host_tail.is_some()
            && split_block_start.is_none()
            && end_row > cursor_row
            && split_end_limit > 0.0
            && !split_end_cut.is_empty()
            && end_row_height_override.is_none()
        {
            let row = end_row - 1;
            let row_start: &[usize] = if row == cursor_row { start_cut } else { &[] };
            let selected = layout_engine.row_cut_content_height(
                row_geometry_table,
                row,
                row_start,
                &split_end_cut,
                styles,
            );
            let full = layout_engine.row_cut_content_height(
                row_geometry_table,
                row,
                row_start,
                &[],
                styles,
            );
            let before = (consumed - selected).max(0.0);
            let capacity = (avail_for_rows - before).max(0.0);
            let plain_row = row_geometry_table
                .cells
                .iter()
                .all(|cell| usize::from(cell.row) != row || cell.row_span == 1);
            if plain_row && capacity > selected && capacity < full {
                end_row_height_override = Some(capacity);
                consumed = before + capacity;
            }
        }
        BlockTableRowScan {
            source_frame_band,
            plain_cut_owns_blank_band,
            consumed,
            accepted_end_row_box,
            accepted_start_row_box,
            accepted_partial_row_box,
            end_row,
            split_block_start,
            split_end_cut,
            split_end_limit,
            split_end_reserved,
            split_block_physical_height,
            end_row_height_override,
        }
    }
}

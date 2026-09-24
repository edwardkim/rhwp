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
        let cut_row_h = &input.prepared.cut_row_heights;
        let whole_row_fit_h = &input.prepared.whole_row_fit_heights;
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
            ..
        } = *budget;
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
        let BlockTableRowScan {
            mut consumed,
            mut end_row,
            mut split_block_start,
            mut split_end_cut,
            mut split_end_limit,
            mut end_row_height_override,
        } = self.scan_block_table_split_rows(
            st,
            layout_engine,
            mt,
            row_geometry_table,
            styles,
            cut_row_h,
            whole_row_fit_h,
            rowspan_touched,
            &start_cut,
            BlockRowScanVars {
                cursor_row,
                row_count: scan_row_count,
                cs,
                can_intra_split,
                is_continuation,
                avail_for_rows,
                header_overhead,
                landscape_rowbreak_bleed,
                landscape_whole_row_tolerance,
                landscape_short_row_tolerance,
                landscape_short_row_max_height,
                strict_painted_bottom_fit: strict_following_plain_text_fit,
                source_first_fragment_overflow_allowance,
                source_first_fragment_row_end,
                start_row_height_override,
            },
            BlockTableRowScan {
                consumed: 0.0,
                end_row: cursor_row,
                split_block_start: None,
                split_end_cut: Vec::new(),
                split_end_limit: 0.0,
                end_row_height_override: None,
            },
        );
        if let Some(started) = issue2424_scan_started {
            profile.scan.0 += started.elapsed();
            profile.scan.1 += 1;
        }
        if end_row <= cursor_row {
            end_row = cursor_row + 1;
        }
        // 첫 source fragment가 선택한 마지막 행은 scanner에서는 measured
        // boundary까지 소비하지만, paint는 common object frame의 남은 물리 높이로
        // 끝나야 한다. 이 값은 source frame과 그 직전 행들의 합으로 계산한다.
        if source_next_positive_rewind
            && !table_declared_object_covers_cell_row_frames(table, self.dpi)
            && split_end_limit <= 0.0
            && source_first_fragment_row_end == Some(end_row)
        {
            if let Some((frame_height, _)) = saved_first_fragment_source_frame {
                let before_last = cut_row_h
                    .iter()
                    .take(end_row.saturating_sub(1))
                    .sum::<f64>()
                    + cs * end_row.saturating_sub(2) as f64;
                end_row_height_override = Some((frame_height - before_last).max(0.0));
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
                    let refit = self.scan_block_table_split_rows(
                        st,
                        layout_engine,
                        mt,
                        row_geometry_table,
                        styles,
                        cut_row_h,
                        whole_row_fit_h,
                        rowspan_touched,
                        &start_cut,
                        BlockRowScanVars {
                            cursor_row,
                            row_count: scan_row_count,
                            cs,
                            can_intra_split,
                            is_continuation,
                            avail_for_rows: avail_refit,
                            header_overhead,
                            landscape_rowbreak_bleed,
                            landscape_whole_row_tolerance,
                            landscape_short_row_tolerance,
                            landscape_short_row_max_height,
                            strict_painted_bottom_fit: strict_following_plain_text_fit,
                            source_first_fragment_overflow_allowance,
                            source_first_fragment_row_end,
                            start_row_height_override,
                        },
                        BlockTableRowScan {
                            consumed: 0.0,
                            end_row: cursor_row,
                            split_block_start: None,
                            split_end_cut: Vec::new(),
                            split_end_limit: 0.0,
                            end_row_height_override: None,
                        },
                    );
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
                        consumed = refit.consumed;
                        end_row = refit.end_row;
                        split_block_start = refit.split_block_start;
                        split_end_cut = refit.split_end_cut;
                        split_end_limit = refit.split_end_limit;
                        end_row_height_override = refit.end_row_height_override;
                        if end_row <= cursor_row {
                            end_row = cursor_row + 1;
                        }
                    }
                }
            }
        }

        BlockTableRowScan {
            consumed,
            end_row,
            split_block_start,
            split_end_cut,
            split_end_limit,
            end_row_height_override,
        }
    }
}

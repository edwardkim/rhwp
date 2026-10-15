//! 표 도메인 조정. 기존 평가·상태 쓰기 순서를 보존한다.

use crate::renderer::typeset::{
    rowbreak_table_has_internal_saved_vpos_reset, table, BlockRowScanVars, BlockTableRowScan,
    MeasuredTable, ResolvedStyleSet, TypesetEngine, TypesetState, MIN_TOP_KEEP_PX,
};

#[derive(Clone, Copy)]
struct ScanInput<'a> {
    st: &'a TypesetState,
    layout_engine: &'a crate::renderer::layout::LayoutEngine,
    mt: &'a MeasuredTable,
    table: &'a crate::model::table::Table,
    styles: &'a ResolvedStyleSet,
    cut_row_h: &'a [f64],
    whole_row_fit_h: &'a [f64],
    opening_source_frames: &'a [f64],
    rowspan_touched: &'a [bool],
    start_cut: &'a [usize],
    straddle_page_bands: &'a [crate::renderer::pagination::StraddleCellCarry],
    v: BlockRowScanVars,
    table_storage_declares_splits: bool,
}

struct ScanProgress {
    r: usize,
    scan: BlockTableRowScan,
    bleed_absorbed_row_height: Option<f64>,
}

struct ScanStep {
    progress: ScanProgress,
    keep_scanning: bool,
}

mod block_step;
mod row_step;

impl TypesetEngine {
    /// [Task #2085] 표 분할점 행-스캔: cursor_row 부터 이번 조각(가용 높이
    /// avail_for_rows)에 들어가는 행/블록/셀-컷을 결정한다. 원본 무변경 통이동 —
    /// landscape/rowbreak 허용치의 소스분기(is_hwpx_source)는 caller 에 잔류.
    #[allow(clippy::too_many_arguments)]
    pub(in crate::renderer::typeset) fn scan_block_table_split_rows(
        &self,
        st: &TypesetState,
        layout_engine: &crate::renderer::layout::LayoutEngine,
        mt: &MeasuredTable,
        table: &crate::model::table::Table,
        styles: &ResolvedStyleSet,
        cut_row_h: &[f64],
        whole_row_fit_h: &[f64],
        source_row_heights: &[f64],
        rowspan_touched: &[bool],
        start_cut: &[usize],
        straddle_page_bands: &[crate::renderer::pagination::StraddleCellCarry],
        v: BlockRowScanVars,
        scan: BlockTableRowScan,
    ) -> BlockTableRowScan {
        let table_storage_declares_splits = rowbreak_table_has_internal_saved_vpos_reset(table);
        // Scan-local facts: compute each unchanged source frame once, rather
        // than recertifying every cell for each ordinary row.
        let mut opening_source_frames = vec![0.0_f64; v.row_count];
        for cell in table.cells.iter().filter(|cell| cell.row_span > 1) {
            if let (Some(slot), Some(height)) = (
                opening_source_frames.get_mut(usize::from(cell.row)),
                layout_engine.stored_initial_plain_text_frame_height(cell, table, styles),
            ) {
                *slot = slot.max(height);
            }
        }
        let input = ScanInput {
            st,
            layout_engine,
            mt,
            table,
            styles,
            cut_row_h,
            whole_row_fit_h,
            opening_source_frames: &opening_source_frames,
            rowspan_touched,
            start_cut,
            straddle_page_bands,
            v,
            table_storage_declares_splits,
        };
        let mut progress = ScanProgress {
            r: v.cursor_row,
            scan,
            bleed_absorbed_row_height: None,
        };
        let source_span_reservation = start_cut.is_empty() && v.start_row_height_override.is_none();
        let fresh_row_budget = if st.current_items.is_empty() && st.current_height == 0.0 {
            // A fresh column already includes its note and zone constraints.
            v.avail_for_rows
        } else {
            (st.base_available_height() - v.header_overhead).max(0.0)
        };
        while progress.r < v.row_count {
            let r = progress.r;
            if v.source_terminal_suffix_start
                .is_some_and(|start| v.cursor_row < start && r >= start)
            {
                break;
            }
            let cs_before = if r > v.cursor_row { v.cs } else { 0.0 };
            if source_span_reservation {
                if let Some(minimum) = layout_engine.stored_span_fragment_minimum(
                    table,
                    v.cursor_row,
                    r + 1,
                    source_row_heights,
                    fresh_row_budget,
                    None,
                    MIN_TOP_KEEP_PX,
                ) {
                    if minimum > v.avail_for_rows {
                        if std::env::var("RHWP_DIAG_SCAN").is_ok() {
                            eprintln!(
                                "DIAG_SCAN SOURCE_SPAN_DEFER r={r} minimum={minimum:.3} avail={:.3}",
                                v.avail_for_rows
                            );
                        }
                        break;
                    }
                }
            }
            // 넘겨받은 물리 잔여도 현재 예산에 들어갈 때만 통째로 수용한다.
            // 더 큰 잔여는 일반 컷 스캔에서 내용과 함께 여러 조각으로 나눈다.
            if r == v.cursor_row {
                if let Some(height) = v.start_row_height_override.filter(|height| {
                    !layout_engine.row_uses_reflow_physical_frame(table, r)
                        && layout_engine.row_cut_units_exhausted(table, r, start_cut, styles)
                        && progress.scan.consumed + height <= v.avail_for_rows
                }) {
                    progress.scan.consumed += height;
                    progress.r += 1;
                    progress.scan.end_row = progress.r;
                    continue;
                }
            }
            let block_query = table::scan::RowBlockQuery {
                layout_engine,
                mt,
                table,
                styles,
                cut_row_h,
                rowspan_touched,
                cs: v.cs,
                straddle_page_bands,
            };
            let block = block_query.candidate(r);
            let step = if (block.protected || block.rowbreak_rowspan_block) && block.b_start == r {
                self.scan_rowspan_block_step(input, progress, block, cs_before)
            } else {
                self.scan_ordinary_row_step(input, progress, cs_before)
            };
            progress = step.progress;
            if !step.keep_scanning {
                break;
            }
        }
        // Choose the physical fragment once, after its row/unit ownership is
        // known. Extra occupancy belongs to its last row, not every candidate.
        if source_span_reservation
            && progress.scan.end_row > v.cursor_row
            && progress.scan.split_end_cut.is_empty()
            && progress.scan.split_end_limit <= 0.0
            && progress.scan.split_block_start.is_none()
        {
            if let Some(minimum) = layout_engine.stored_span_fragment_minimum(
                table,
                v.cursor_row,
                progress.scan.end_row,
                source_row_heights,
                fresh_row_budget,
                Some(v.avail_for_rows),
                MIN_TOP_KEEP_PX,
            ) {
                let last = progress.scan.end_row - 1;
                let height = progress
                    .scan
                    .end_row_height_override
                    .unwrap_or(source_row_heights[last]);
                let physical = source_row_heights[v.cursor_row..progress.scan.end_row]
                    .iter()
                    .sum::<f64>()
                    + v.cs * (progress.scan.end_row - v.cursor_row).saturating_sub(1) as f64
                    + height
                    - source_row_heights[last];
                // An override also changes saved-frame selection. Its presence
                // must not be created by floating-point summation noise alone.
                let roundoff = f64::EPSILON
                    * (minimum.abs() + physical.abs()).max(1.0)
                    * (progress.scan.end_row - v.cursor_row + 2) as f64;
                if minimum - physical > roundoff && minimum <= v.avail_for_rows {
                    progress.scan.end_row_height_override = Some(height + minimum - physical);
                    progress.scan.consumed = progress.scan.consumed.max(minimum);
                    if std::env::var("RHWP_DIAG_SCAN").is_ok() {
                        eprintln!(
                            "DIAG_SCAN SOURCE_SPAN_FRAME rows={}..{} physical={physical:.3} minimum={minimum:.3} last={:.3}",
                            v.cursor_row,
                            progress.scan.end_row,
                            progress.scan.end_row_height_override.unwrap_or(height)
                        );
                    }
                }
            }
        }
        progress.scan
    }
}

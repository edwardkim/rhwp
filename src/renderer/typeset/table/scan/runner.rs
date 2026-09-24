//! 표 도메인 조정. 기존 평가·상태 쓰기 순서를 보존한다.

use crate::renderer::typeset::{
    rowbreak_table_has_internal_saved_vpos_reset, table, BlockRowScanVars, BlockTableRowScan,
    MeasuredTable, ResolvedStyleSet, TypesetEngine, TypesetState,
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
    rowspan_touched: &'a [bool],
    start_cut: &'a [usize],
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
        rowspan_touched: &[bool],
        start_cut: &[usize],
        v: BlockRowScanVars,
        scan: BlockTableRowScan,
    ) -> BlockTableRowScan {
        let table_storage_declares_splits = rowbreak_table_has_internal_saved_vpos_reset(table);
        let input = ScanInput {
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
        };
        let mut progress = ScanProgress {
            r: v.cursor_row,
            scan,
            bleed_absorbed_row_height: None,
        };
        while progress.r < v.row_count {
            let r = progress.r;
            let cs_before = if r > v.cursor_row { v.cs } else { 0.0 };
            // Consume a carried physical tail before scanning the next row.
            if r == v.cursor_row {
                if let Some(height) = v.start_row_height_override {
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
        progress.scan
    }
}

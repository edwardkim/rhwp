//! One continuation iteration: budget, read-only row scan/refit, then emission.
//! The start cut is snapshotted once before queued notes can advance the cursor.

use crate::renderer::typeset::{
    row_geometry_table, BlockTableContinuationSource, TableContinuationIteration, TypesetEngine,
    TypesetState,
};

use super::{BlockTableContinuationPreparedState, TableContinuationCursor};

mod budget;
mod emit;
mod scan;

pub(super) struct FragmentProfile {
    pub(super) enabled: bool,
    pub(super) iterations: u64,
    pub(super) scan: (std::time::Duration, u32),
    pub(super) refit: (std::time::Duration, u32),
}

struct FragmentStart {
    cursor_row: usize,
    is_continuation: bool,
    start_cut_is_block: bool,
    start_row_height_override: Option<f64>,
    start_cut: Vec<usize>,
    fragment_starts_intra_row: bool,
}

#[derive(Clone, Copy)]
struct FragmentInput<'a> {
    source: BlockTableContinuationSource<'a>,
    prepared: &'a BlockTableContinuationPreparedState,
    start: &'a FragmentStart,
    row_cursor_is_nested: bool,
}

#[derive(Clone, Copy)]
struct FragmentBudget {
    caption_extra: f64,
    host_before_overhead: f64,
    terminal_outer_bottom_overhead: f64,
    fragment_outer_bottom_overhead: f64,
    vert_offset_overhead: f64,
    page_avail: f64,
    fragment_placement: Option<crate::renderer::float_placement::ParagraphFloatPlacement>,
    scan_row_count: usize,
    saved_first_fragment_source_frame: Option<(f64, f64)>,
    source_first_fragment_row_end: Option<usize>,
    source_first_fragment_overflow_allowance: f64,
    header_overhead: f64,
    avail_for_rows: f64,
    /// [#7095] 본문을 통째로 담은 1×1 RowBreak 쪽 조각 형상인지.
    single_cell_fragment_shape: bool,
    /// [#7095] 저장 host 원점이 있는 조각이 칠하는 상자 높이(px). 원점이 없는 조각은
    /// 방출 시점의 흐름 좌표로 같은 상자를 다시 잰다.
    single_cell_box_height: Option<f64>,
}

impl TypesetEngine {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn step_block_table_fragment(
        &self,
        prepared: &BlockTableContinuationPreparedState,
        st: &mut TypesetState,
        continuation: &mut TableContinuationCursor,
        source: BlockTableContinuationSource<'_>,
        row_cursor_is_nested: bool,
        profile: &mut FragmentProfile,
    ) -> TableContinuationIteration {
        profile.iterations += 1;
        let start_cut = continuation.start_cut.clone();
        let start = FragmentStart {
            cursor_row: continuation.row,
            is_continuation: continuation.is_continuation,
            start_cut_is_block: continuation.start_cut_is_block,
            start_row_height_override: continuation.start_row_height_override,
            fragment_starts_intra_row: continuation.start_cut_is_block || !start_cut.is_empty(),
            start_cut,
        };
        let input = FragmentInput {
            source,
            prepared,
            start: &start,
            row_cursor_is_nested,
        };
        let row_geometry_table = input.source.row_geometry_table;
        let styles = input.source.styles;
        let can_intra_split = input.prepared.can_intra_split;
        let layout_engine = &input.prepared.layout_engine;
        let cursor_row = input.start.cursor_row;
        let start_cut_is_block = input.start.start_cut_is_block;
        let start_row_height_override = input.start.start_row_height_override;
        let start_cut = &input.start.start_cut;
        // 이전 분할에서 모든 콘텐츠가 소진된 행은 건너뜀.
        // [Task #1025] 블록 컷(start_cut_is_block)은 per-row(row_span==1) 컷이 아니라
        // 블록-셀 인덱스다. advance_row_cut(per-row)로 판정하면 블록 첫 행이 소진돼도
        // 거대 셀이 남은 경우를 "소진"으로 오판해 cursor 를 전진시키고 start_cut 을
        // 비워 블록 컷을 잃는다(연속분이 거대 셀을 처음부터 다시 렌더 → overflow).
        // 블록 컷이면 이 가드를 건너뛰어 컷을 보존한다.
        if !start_cut_is_block
            && !start_cut.is_empty()
            && start_row_height_override.is_none()
            && can_intra_split
            && layout_engine
                .advance_row_cut(row_geometry_table, cursor_row, &start_cut, f64::MAX, styles)
                .consumed_height
                <= 0.0
        {
            continuation.skip_consumed_row();
            return TableContinuationIteration::Skipped;
        }

        let budget = self.prepare_table_fragment_budget(st, input);
        let scan = self.scan_table_fragment(st, input, &budget, profile);
        self.emit_table_fragment(st, continuation, input, &budget, scan)
    }
}

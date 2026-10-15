//! One continuation iteration: budget, read-only row scan/refit, then emission.
//! The start cut is snapshotted once before queued notes can advance the cursor.

use crate::renderer::typeset::{
    row_geometry_table, BlockTableContinuationSource, TableContinuationIteration, TypesetEngine,
    TypesetState,
};

use super::{BlockTableContinuationPreparedState, TableContinuationCursor};

mod budget;
mod coanchored_origin;
mod emit;
mod physical_rows;
mod scan;
mod stored_frame_budget;

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
    stored_fragment_outer_frame: bool,
    stored_fragment_body_bottom_inset: Option<f64>,
    start_cut: Vec<usize>,
    /// [#7531] 이 조각에서 쪽을 넘어온 걸침 칸의 끝 행 높이에서 덜 양 `(행, px)`.
    straddle_row_relief: Vec<(usize, f64)>,
    straddle_page_bands: Vec<crate::renderer::pagination::StraddleCellCarry>,
    /// 덜기를 적용한 scan 용 행 높이. 덜기가 없으면 준비 상태의 값을 그대로 쓴다.
    relieved_cut_row_heights: Option<Vec<f64>>,
    relieved_whole_row_fit_heights: Option<Vec<f64>>,
}

#[derive(Clone, Copy)]
struct FragmentInput<'a> {
    source: BlockTableContinuationSource<'a>,
    prepared: &'a BlockTableContinuationPreparedState,
    start: &'a FragmentStart,
    row_cursor_is_nested: bool,
    fragment_row_heights: Option<&'a [f64]>,
}

impl FragmentInput<'_> {
    /// A content cut may finish every unit while retaining its paint cut.
    /// Scan must reserve the closing band before emit ends that continuation.
    fn closes_terminal_content_cut(
        &self,
        end_row: usize,
        end_limit: f64,
        end_cut: &[usize],
        block_start: Option<usize>,
        height_override: Option<f64>,
    ) -> bool {
        let rows = self.prepared.row_count;
        end_row >= rows
            && end_limit > 0.0
            && !end_cut.is_empty()
            && block_start.is_none()
            && !self.start.start_cut_is_block
            && !self.row_cursor_is_nested
            && height_override.is_none()
            && self.source.measured_table.allows_row_break_split()
            && self.prepared.caption_overhead <= 0.0
            && !self.prepared.queue_table_footnotes
            && self.prepared.can_intra_split
            && self.prepared.layout_engine.row_cut_units_exhausted(
                self.source.row_geometry_table,
                rows - 1,
                end_cut,
                self.source.styles,
            )
            && self
                .prepared
                .layout_engine
                .straddle_continuation_demand(
                    self.source.row_geometry_table,
                    rows - 1,
                    rows - 1,
                    end_cut,
                    false,
                    None,
                    &self.source.measured_table.row_heights,
                    &self.start.straddle_page_bands,
                    self.source.styles,
                    (rows, true),
                )
                .is_none_or(|remaining| remaining <= 0.0)
    }
}

#[derive(Clone, Copy)]
struct FragmentBudget {
    caption_extra: f64,
    host_before_overhead: f64,
    terminal_outer_bottom_overhead: f64,
    fragment_outer_bottom_overhead: f64,
    fragment_opens_outer_top: bool,
    physical_content_limit: f64,
    vert_offset_overhead: f64,
    page_avail: f64,
    /// 원본 물리 프레임이 반복해서 닫히는 하단. 종료 인증·예약·방출이 함께 쓴다.
    source_frame_bottom: Option<f64>,
    fragment_placement: Option<crate::renderer::float_placement::ParagraphFloatPlacement>,
    scan_row_count: usize,
    saved_first_fragment_source_frame: Option<(f64, f64)>,
    stored_partial_frame_height: Option<f64>,
    source_first_fragment_row_end: Option<usize>,
    /// Complete source frame: the last row's physical remainder, shared by scan and paint.
    source_complete_frame_last_row: Option<(usize, f64)>,
    source_first_fragment_overflow_allowance: f64,
    header_overhead: f64,
    avail_for_rows: f64,
    /// 일반 재조판 행의 선언 최소 높이가 소비할 paint 하단. 블록 컷은 별도 ledger를 쓴다.
    reflow_frame_avail_for_rows: Option<f64>,
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
        // 소스 컷은 앞 조각의 끝과 연속이어야 한다. 저장 프레임의 꼬리 spacer는
        // 그 연속 범위 안에서 소비하되, 다음 물리 프레임의 높이/원점에는 싣지 않는다.
        // 예약·선택은 paint와 같은 helper로 구한 물리 시작점을 소비한다.
        let start_cut = if continuation.start_cut_is_block {
            continuation.start_cut.clone()
        } else {
            prepared.layout_engine.skip_leading_free_spacers_in_row_cut(
                source.row_geometry_table,
                continuation.row,
                &continuation.start_cut,
                source.styles,
            )
        };
        // 완결한 독립 문단 뒤에서 시작해도 앞 조각의 쪽 끝 띠는 병합 셀 소유다.
        // 시작 행의 부분 상자는 컷/override가 맡고, 뒤의 온전한 끝 행만 띠를 덜 센다.
        let starts_after_complete_paragraph = !continuation.start_cut_is_block
            && !continuation.start_cut.is_empty()
            && prepared
                .layout_engine
                .row_cut_completes_unlinked_plain_paragraphs(
                    source.row_geometry_table,
                    continuation.row,
                    &[],
                    &continuation.start_cut,
                    source.styles,
                );
        // 실제 수용 상자 원장이 있는 일반 컷은 문단 내부 시작도 같은 상자를 소비한다.
        // 원장이 끊긴 블록 경로는 기존 완결 문단/온전한 시작의 판정을 유지한다.
        let has_actual_span_ledger = !continuation.straddle_page_bands.is_empty()
            && continuation
                .straddle_page_bands
                .iter()
                .all(|carry| carry.accepted_physical_height.is_some());
        // 이 계획은 앞 조각의 실제 상자와 빈 띠를 이미 차감한 공통 결과다.
        // 기존 별도 띠 차감을 다시 적용하면 예약과 paint의 행 높이가 달라진다.
        let physical_rows = self.stored_fragment_row_plan(st, prepared, continuation, source);
        let straddle_row_relief = if physical_rows.is_some()
            || continuation.start_cut_is_block
            || ((!continuation.start_cut.is_empty()
                || continuation.start_row_height_override.is_some())
                && !starts_after_complete_paragraph
                && !has_actual_span_ledger)
        {
            Vec::new()
        } else {
            let mut relief = prepared.layout_engine.straddle_page_band_relief(
                source.row_geometry_table,
                &source.measured_table.row_heights,
                continuation.row,
                &start_cut,
                continuation.start_row_height_override,
                &continuation.straddle_page_bands,
                source.styles,
            );
            if !continuation.start_cut.is_empty()
                || continuation.start_row_height_override.is_some()
            {
                relief.retain(|(row, _)| *row > continuation.row);
            }
            relief
        };
        let relieve = |heights: &Vec<f64>| {
            let mut heights = heights.clone();
            for &(row, amount) in &straddle_row_relief {
                if let Some(height) = heights.get_mut(row) {
                    *height = (*height - amount).max(0.0);
                }
            }
            heights
        };
        let (relieved_cut_row_heights, relieved_whole_row_fit_heights) =
            if straddle_row_relief.is_empty() {
                (None, None)
            } else {
                (
                    Some(relieve(&prepared.cut_row_heights)),
                    Some(relieve(&prepared.whole_row_fit_heights)),
                )
            };
        let start = FragmentStart {
            straddle_page_bands: continuation.straddle_page_bands.clone(),
            straddle_row_relief,
            relieved_cut_row_heights,
            relieved_whole_row_fit_heights,
            cursor_row: continuation.row,
            is_continuation: continuation.is_continuation,
            start_cut_is_block: continuation.start_cut_is_block,
            start_row_height_override: continuation.start_row_height_override,
            stored_fragment_outer_frame: continuation.stored_fragment_outer_frame,
            stored_fragment_body_bottom_inset: continuation.stored_fragment_body_bottom_inset,
            start_cut,
        };
        let fragment_measurement = physical_rows.as_ref().map(|rows| {
            let mut measured = source.measured_table.clone();
            measured.row_heights.clone_from(rows);
            measured.cumulative_heights = vec![0.0];
            for (row, &height) in rows.iter().enumerate() {
                let spacing = if row > 0 { measured.cell_spacing } else { 0.0 };
                let next =
                    measured.cumulative_heights.last().copied().unwrap_or(0.0) + height + spacing;
                measured.cumulative_heights.push(next);
            }
            measured.total_height = measured.cumulative_heights.last().copied().unwrap_or(0.0)
                + measured.caption_height;
            measured
        });
        let source = BlockTableContinuationSource {
            measured_table: fragment_measurement
                .as_ref()
                .unwrap_or(source.measured_table),
            ..source
        };
        let input = FragmentInput {
            source,
            prepared,
            start: &start,
            row_cursor_is_nested,
            fragment_row_heights: physical_rows.as_deref(),
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
            && layout_engine.row_cut_units_exhausted(
                row_geometry_table,
                cursor_row,
                &start_cut,
                styles,
            )
        {
            continuation.skip_consumed_row();
            return TableContinuationIteration::Skipped;
        }

        let mut budget = self.prepare_table_fragment_budget(st, input);
        let mut scan = self.scan_table_fragment(st, input, &budget, profile);
        let spanning_bottom =
            crate::renderer::float_placement::column_rowbreak_grid_bottom_margin_px(
                budget.fragment_opens_outer_top,
                source.table,
                start.is_continuation,
                scan.split_end_limit == 0.0
                    || scan.split_block_start.is_some()
                    || start.start_cut_is_block,
                scan.end_row < prepared.row_count || scan.split_end_limit > 0.0,
                self.dpi,
            )
            .max(budget.fragment_outer_bottom_overhead);
        if spanning_bottom > budget.fragment_outer_bottom_overhead {
            // Intersect the physical object bottom with the already-reserved
            // footnote/footer capacity; a narrower budget has this clearance.
            let extra =
                (budget.page_avail - budget.physical_content_limit + spanning_bottom).max(0.0);
            budget.fragment_outer_bottom_overhead = spanning_bottom;
            budget.page_avail -= extra;
            budget.avail_for_rows = (budget.avail_for_rows - extra).max(0.0);
            if extra > 0.0 {
                scan = self.scan_table_fragment(st, input, &budget, profile);
            }
        }
        if scan.end_row == cursor_row && scan.consumed == 0.0 {
            // 캡션 종료 예산 때문에 마지막 유닛 전체를 이월했다.
            // 현재 조각에서는 내용이나 물리 높이를 소비하지 않았다.
            st.advance_column_or_new_page();
            return TableContinuationIteration::Skipped;
        }
        self.emit_table_fragment(st, continuation, input, &budget, scan)
    }
}

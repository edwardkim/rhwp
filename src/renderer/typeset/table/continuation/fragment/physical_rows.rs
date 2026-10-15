//! Stored rowspan minimums consume physical boxes accepted by earlier fragments.

use crate::renderer::typeset::{
    hwpunit_to_px, BlockTableContinuationPreparedState, BlockTableContinuationSource,
    TypesetEngine, TypesetState,
};

use super::super::TableContinuationCursor;
use super::FragmentInput;

pub(super) struct StoredFragmentEnd {
    pub(super) row: usize,
    pub(super) complete_cut: bool,
    pub(super) block_start: Option<usize>,
    pub(super) last_height: Option<f64>,
    pub(super) restart_frame_closes_body: bool,
}

impl TypesetEngine {
    pub(super) fn stored_header_fragment_placement(
        &self,
        st: &TypesetState,
        input: FragmentInput<'_>,
        outer_top: f64,
    ) -> Option<crate::renderer::float_placement::ParagraphFloatPlacement> {
        if !input.start.is_continuation || !self.stored_header_physical_rows(st, input.source) {
            return None;
        }
        let top = st.current_height + outer_top;
        Some(crate::renderer::float_placement::ParagraphFloatPlacement {
            flow: crate::renderer::float_placement::ParagraphFloatFlow::NextLine,
            anchor_y: st.current_height,
            stored_host_origin: None,
            stored_successor_line_origin: None,
            table_left: None,
            table_top: top,
            occupied_bottom: top,
        })
    }

    pub(super) fn stored_header_fragment_row_budget(
        &self,
        st: &TypesetState,
        input: FragmentInput<'_>,
        available: f64,
        outer_top: f64,
    ) -> f64 {
        if !input.start.is_continuation || !self.stored_header_physical_rows(st, input.source) {
            return available;
        }
        // Paint closes a page-filling fragment at this same physical boundary.
        // Its accepted row boxes must exclude the unpainted bottom band before
        // a later closing span subtracts those boxes from its declared height.
        let bottom = crate::renderer::float_placement::single_cell_page_fragment_bottom(
            input.source.table,
            st.layout.body_area.height,
            self.dpi,
        );
        available.min(
            (bottom - st.current_zone_y_offset - st.layout.pagination_tolerance_px - outer_top)
                .max(0.0),
        )
    }

    pub(super) fn stored_header_physical_rows(
        &self,
        st: &TypesetState,
        source: BlockTableContinuationSource<'_>,
    ) -> bool {
        crate::renderer::float_placement::stored_repeated_header_rowbreak_outer_frame(
            st.profile,
            source.paragraph,
            source.table,
        ) && !self.render_normalization.table_text_reflowed(source.table)
    }

    pub(super) fn stored_fragment_row_plan(
        &self,
        st: &TypesetState,
        prepared: &BlockTableContinuationPreparedState,
        cursor: &TableContinuationCursor,
        source: BlockTableContinuationSource<'_>,
    ) -> Option<Vec<f64>> {
        let keeps_source_boxes = self.stored_header_physical_rows(st, source)
            || prepared
                .layout_engine
                .has_stored_continuous_spans(source.row_geometry_table);
        if !keeps_source_boxes
            || cursor.start_cut_is_block
            || !cursor.accepted_row_boxes.iter().any(Option::is_some)
        {
            return None;
        }
        let table = source.row_geometry_table;
        // 이 높이는 앞 emit이 생산해 시작 행 스캔과 paint에 그대로 넘긴
        // 물리 상자다. 앞 쪽 상자와 합쳐 이후 병합 종료 행의 요구를 잰다.
        let resumed_row_box = cursor.stored_restart_row_box.and_then(|(row, before)| {
            (row == cursor.row && !cursor.start_cut.is_empty())
                .then_some(cursor.start_row_height_override?)
                .map(|current| before + current)
        });
        // 일반 칸을 모두 소비했어도 걸침 칸의 남은 유닛 때문에 커서는 같은
        // 행에 머물 수 있다. 이때 앞 조각이 이미 수용한 행 상자를 병합 최소
        // 높이의 차감 원장에 포함하고, 미소비 일반 행의 추정값과 구분한다.
        let completed_start_row_box = (!cursor.start_cut.is_empty()
            && prepared.layout_engine.row_cut_consumes_all_ordinary_units(
                table,
                cursor.row,
                &cursor.start_cut,
                source.styles,
            ))
        .then(|| cursor.accepted_row_boxes.get(cursor.row).copied().flatten())
        .flatten();
        let start_row_box = resumed_row_box.or(completed_start_row_box);
        // A cut's start owner subtracts every preceding physical row of its
        // spanning cell. Do not mix accepted boxes with original estimates for
        // unknown partial rows: that can repeat already consumed source units.
        let prefix_known = |first: usize, end: usize| {
            (first..end).all(|row| {
                (row == cursor.row && start_row_box.is_some())
                    || cursor
                        .accepted_row_boxes
                        .get(row)
                        .is_some_and(Option::is_some)
            })
        };
        if table.cells.iter().any(|cell| {
            let first = usize::from(cell.row);
            first < cursor.row
                && cursor.row < first + usize::from(cell.row_span)
                && !prefix_known(first, cursor.row)
        }) {
            return None;
        }
        let original = &source.measured_table.row_heights;
        let mut heights = original.clone();
        for (row, accepted) in cursor
            .accepted_row_boxes
            .iter()
            .enumerate()
            .take(cursor.row)
        {
            if let (Some(height), Some(target)) = (accepted, heights.get_mut(row)) {
                *target = *height;
            }
        }
        if let Some(height) = start_row_box {
            heights[cursor.row] = height;
        }
        let accepted_prefix_end = cursor.row + usize::from(start_row_box.is_some());
        // A resumed first row owns a cut, not a complete source row. Keep its
        // existing cut/remaining-band contract while later whole rows consume
        // the accepted physical prefix from earlier fragments.
        let first_whole_row = cursor.row
            + usize::from(
                !cursor.start_cut.is_empty() || cursor.start_row_height_override.is_some(),
            );
        for row in first_whole_row..heights.len() {
            // Only a closing span whose preceding physical boxes changed can
            // invalidate the globally measured remainder. Other rows keep their
            // original measurement, including content-driven growth.
            let closing: Vec<_> = table
                .cells
                .iter()
                .filter(|cell| {
                    cell.row_span > 1
                        && cell.height < 0x8000_0000
                        && cell.row as usize + cell.row_span as usize == row + 1
                        && prefix_known(usize::from(cell.row), accepted_prefix_end.min(row))
                })
                .collect();
            if !closing.iter().any(|cell| {
                let first = cell.row as usize;
                (first..accepted_prefix_end.min(row))
                    .any(|prior| (heights[prior] - original[prior]).abs() > 0.5)
            }) {
                continue;
            }
            // Whole-row content and single-row declarations provide the floor.
            // Rows with no single-row owner retain the original unknown-row
            // distribution rather than inventing a new one here.
            if !table
                .cells
                .iter()
                .any(|cell| cell.row as usize == row && cell.row_span == 1)
            {
                continue;
            }
            let floor =
                prepared
                    .layout_engine
                    .row_cut_content_height(table, row, &[], &[], source.styles);
            let required = closing.iter().fold(floor, |height, cell| {
                let first = cell.row as usize;
                let used = heights[first..row].iter().sum::<f64>()
                    + prepared.cell_spacing * (row - first) as f64;
                height.max(hwpunit_to_px(cell.height as i32, self.dpi) - used)
            });
            heights[row] = required;
        }
        if std::env::var("RHWP_TABLE_DRIFT").is_ok() {
            eprintln!(
                "TABLE_FRAGMENT_ROWS: pi={} cursor={} original={:?} accepted={:?} resolved={:?}",
                source.para_index, cursor.row, original, cursor.accepted_row_boxes, heights
            );
        }
        Some(heights)
    }

    pub(super) fn record_stored_fragment_rows(
        &self,
        st: &TypesetState,
        cursor: &mut TableContinuationCursor,
        input: FragmentInput<'_>,
        end: StoredFragmentEnd,
    ) {
        let keeps_source_boxes = self.stored_header_physical_rows(st, input.source)
            || input
                .prepared
                .layout_engine
                .has_stored_continuous_spans(input.source.row_geometry_table);
        if !keeps_source_boxes || input.start.start_cut_is_block {
            return;
        }
        let heights = input
            .fragment_row_heights
            .unwrap_or(&input.source.measured_table.row_heights);
        let first = input.start.cursor_row
            + usize::from(
                !input.start.start_cut.is_empty()
                    || input.start.start_row_height_override.is_some(),
            );
        // A partial end block can contain several rows. None of those boxes
        // is a whole-row measurement. Without an explicit closing height the
        // final row can also fold during paint; its physical box is unknown.
        let whole_end = end.block_start.unwrap_or_else(|| {
            if end.complete_cut && end.last_height.is_some() {
                end.row
            } else {
                end.row.saturating_sub(1)
            }
        });
        cursor.accepted_row_boxes.resize(heights.len(), None);
        if let Some((row, before)) = cursor.stored_restart_row_box {
            if row == input.start.cursor_row && row + 1 < end.row {
                if let Some(current) = input.start.start_row_height_override {
                    cursor.accepted_row_boxes[row] = Some(before + current);
                    cursor.stored_restart_row_box = None;
                }
            }
        }
        // 아직 끝내지 않은 행의 컷은 완료 행으로 기록하지 않는다. 원본
        // 본문 프레임에서 실제 확정한 상자만 다음 같은 행에 전달한다.
        if end.restart_frame_closes_body && !end.complete_cut && end.block_start.is_none() {
            if let (Some(row), Some(height)) = (end.row.checked_sub(1), end.last_height) {
                let before = cursor
                    .stored_restart_row_box
                    .filter(|&(owner, _)| owner == row)
                    .map_or(0.0, |(_, height)| height);
                cursor.stored_restart_row_box = Some((row, before + height));
            }
        }
        if first >= whole_end {
            return;
        }
        if std::env::var("RHWP_TABLE_DRIFT").is_ok() {
            eprintln!(
                "TABLE_ACCEPTED_ROWS: pi={} rows={}..{} last={:?} measured={:?}",
                input.source.para_index, first, whole_end, end.last_height, heights
            );
        }
        for (row, &height) in heights.iter().enumerate().take(whole_end).skip(first) {
            cursor.accepted_row_boxes[row] = Some(if row + 1 == end.row {
                end.last_height.unwrap_or(height)
            } else {
                height
            });
        }
    }
}

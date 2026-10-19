//! 블록 내용 컷과 실제 수용한 원본 행 상자의 공통 전달.

use crate::renderer::typeset::{BlockTableRowScan, TypesetEngine};

use super::super::{StoredBlockPhysicalFrame, TableContinuationCursor};
use super::{FragmentBudget, FragmentInput};

pub(super) struct StoredBlockAcceptance {
    pub(super) frame: StoredBlockPhysicalFrame,
    pub(super) rows: Vec<f64>,
    pub(super) extra: f64,
}

impl TypesetEngine {
    pub(super) fn accept_stored_block_frame(
        &self,
        cursor: &TableContinuationCursor,
        input: FragmentInput<'_>,
        budget: &FragmentBudget,
        scan: &BlockTableRowScan,
    ) -> Option<StoredBlockAcceptance> {
        let first = scan.split_block_start?;
        let (_, end, _) = input.source.measured_table.row_block_for(first);
        let occupied = scan.split_block_physical_height?;
        if scan.end_row != end || occupied <= 0.0 || scan.end_row_height_override.is_some() {
            return None;
        }
        let layout = &input.prepared.layout_engine;
        let table = input.source.row_geometry_table;
        let styles = input.source.styles;
        let mut frame = if let Some(frame) = cursor
            .stored_block_frame
            .as_ref()
            .filter(|frame| frame.start == first && frame.end == end)
        {
            frame.clone()
        } else {
            StoredBlockPhysicalFrame {
                start: first,
                end,
                rows: layout.stored_block_source_rows(
                    table,
                    (first, end),
                    &scan.split_end_cut,
                    styles,
                )?,
                accepted: 0.0,
            }
        };
        let current = layout.stored_block_remaining_rows(
            table,
            (first, end),
            &frame.rows,
            frame.accepted,
            if input.start.cursor_row == first {
                &input.start.start_cut
            } else {
                &[]
            },
            styles,
        );
        let before = scan.consumed - occupied;
        let room = budget
            .reflow_blank_avail_for_rows
            .unwrap_or(budget.avail_for_rows)
            .min(budget.avail_for_rows)
            - before;
        if room < occupied {
            return None;
        }
        let future_content =
            layout.row_block_cut_remaining_height(table, first, end, &scan.split_end_cut, styles);
        let remaining = frame.rows.iter().sum::<f64>() - frame.accepted;
        let height = room
            .min((remaining - future_content).max(occupied))
            .max(occupied);
        let mut rows = input.source.measured_table.row_heights.clone();
        let mut rest = height;
        for (row, &source) in (first..end).zip(&current) {
            let accepted = rest.min(source).max(0.0);
            rows[row] = accepted;
            rest -= accepted;
        }
        // 내용이 원본 최소보다 커지면 마지막 행이 그 증가분을 소유한다.
        if rest > 0.0 {
            rows[end - 1] += rest;
        }
        frame.accepted += height;
        if std::env::var_os("RHWP_TABLE_DRIFT").is_some() {
            eprintln!("TABLE_ACCEPTED_BLOCK: pi={} rows={}..{} source={:?} physical={:.3} content={:.3} accepted={:.3} remaining={:.3}",
                input.source.para_index, first, end, frame.rows, height, occupied,
                frame.accepted, (remaining - height).max(0.0));
        }
        Some(StoredBlockAcceptance {
            frame,
            rows,
            extra: height - occupied,
        })
    }
}

//! Original line frames can restart at zero beyond the opening object box.

use crate::model::paragraph::LineSeg;
use crate::model::shape::TextWrap;
use crate::model::table::{Cell, Table, TablePageBreak};

use super::{CellUnit, LayoutEngine, ResolvedStyleSet};

/// 원본 셀 문단의 원점0 재시작을 저장 물리 경계로 인증한 근거.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SavedZeroOriginWitness {
    /// 행이 표의 선언 첫 개체 프레임(`common.height`) 경계를 가로지르거나, 같은 행의
    /// 다른 원본 사다리가 같은 경계를 확인한다 — 저장 쪽 나눔과 무관한 기하 근거.
    OpeningFrame,
    /// 첫 프레임 뒤 행의 재시작. 저장 당시의 쪽 나눔이 그 행 안에 있었다는 기록일 뿐이라,
    /// 지금의 흐름에서도 그 행이 쪽 끝에 걸릴 때만 물리 경계가 된다.
    LaterPage,
}

impl LayoutEngine {
    /// A completed sibling cannot certify another owner's unfinished cut.
    pub(crate) fn row_cut_ends_at_saved_zero_origin_frame(
        &self,
        table: &Table,
        row: usize,
        start_cut: &[usize],
        end_cut: &[usize],
        styles: &ResolvedStyleSet,
    ) -> bool {
        let mut cells: Vec<_> = table
            .cells
            .iter()
            .filter(|cell| usize::from(cell.row) == row && cell.row_span == 1)
            .collect();
        cells.sort_by_key(|cell| cell.col);
        let mut unfinished = false;
        for (index, cell) in cells.into_iter().enumerate() {
            let units = self.cell_units(cell, table, styles);
            let start = start_cut.get(index).copied().unwrap_or(0);
            let end = end_cut.get(index).copied().unwrap_or(start);
            if end >= units.len() {
                continue;
            }
            if end <= start
                || !self.cut_starts_saved_zero_origin_text_frame(table, cell, &units, end)
            {
                return false;
            }
            unfinished = true;
        }
        unfinished
    }

    pub(crate) fn row_cut_ends_at_shared_zero_origin_frame(
        &self,
        table: &Table,
        row: usize,
        end_cut: &[usize],
        styles: &ResolvedStyleSet,
    ) -> bool {
        let mut cells: Vec<_> = table
            .cells
            .iter()
            .filter(|cell| cell.row as usize == row && cell.row_span == 1)
            .collect();
        cells.sort_by_key(|cell| cell.col);
        if cells.len() != end_cut.len() {
            return false;
        }
        let mut boundary_owner = false;
        for (cell, &end) in cells.iter().zip(end_cut) {
            let units = self.cell_units(cell, table, styles);
            if end >= units.len() {
                // A completed sibling has no next-frame owner to restart.
                continue;
            }
            if end != 1
                || !self.cut_starts_saved_zero_origin_text_frame(table, cell, &units, end)
                || !self.row_shared_zero_origin_frame_peer(table, cell)
            {
                return false;
            }
            boundary_owner = true;
        }
        boundary_owner
    }

    pub(super) fn row_shared_zero_origin_frame_peer(&self, table: &Table, cell: &Cell) -> bool {
        table.cells.iter().any(|peer| {
            if peer.row != cell.row
                || peer.col == cell.col
                || peer.row_span <= 1
                || peer.dirty_flag
                || peer.paragraphs.iter().any(|para| {
                    para.stored_text_partition_is_dirty()
                        || para.cell_format_vpos_dirty
                        || !para.controls.is_empty()
                })
            {
                return false;
            }
            let mut lines = peer.paragraphs.iter().flat_map(|para| &para.line_segs);
            let (Some(first), Some(second), Some(after)) =
                (lines.next(), lines.next(), lines.next())
            else {
                return false;
            };
            [first, second, after].iter().all(|line| {
                line.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0
                    && line.line_height > 0
                    && line.line_spacing >= 0
            }) && first.vertical_pos == 0
                && second.vertical_pos == 0
                && i64::from(after.vertical_pos)
                    == i64::from(second.line_height) + i64::from(second.line_spacing)
        })
    }

    /// 행의 단일 행 칸에 원본 원점0 재시작이 있으면 그 인증 종류(첫 개체 프레임이 우선).
    pub(crate) fn row_saved_zero_origin_text_frame(
        &self,
        table: &Table,
        row: usize,
    ) -> Option<SavedZeroOriginWitness> {
        let mut found = None;
        for cell in table
            .cells
            .iter()
            .filter(|cell| cell.row as usize == row && cell.row_span == 1)
        {
            for para in &cell.paragraphs {
                for lines in para.line_segs.windows(2) {
                    match self.stored_zero_origin_text_frame_break_witness(
                        table, cell, &lines[0], &lines[1],
                    ) {
                        Some(SavedZeroOriginWitness::OpeningFrame) => {
                            return Some(SavedZeroOriginWitness::OpeningFrame)
                        }
                        Some(SavedZeroOriginWitness::LaterPage) => {
                            found = Some(SavedZeroOriginWitness::LaterPage)
                        }
                        None => {}
                    }
                }
            }
        }
        found
    }

    pub(super) fn stored_zero_origin_text_frame_break(
        &self,
        table: &Table,
        cell: &Cell,
        previous: &LineSeg,
        next: &LineSeg,
    ) -> bool {
        self.stored_zero_origin_text_frame_break_witness(table, cell, previous, next)
            .is_some()
    }

    /// 원점0 재시작이 무엇으로 인증됐는가 — [`SavedZeroOriginWitness`].
    pub(super) fn stored_zero_origin_text_frame_break_witness(
        &self,
        table: &Table,
        cell: &Cell,
        previous: &LineSeg,
        next: &LineSeg,
    ) -> Option<SavedZeroOriginWitness> {
        let profile = self.profile.get();
        if !(profile.hwp5_stored_pagination_layout() || profile.hwpx_stored_layout())
            || profile.session_edited()
            || self
                .render_normalization
                .borrow()
                .table_text_reflowed(table)
            || table.common.treat_as_char
            || table.common.text_wrap != TextWrap::TopAndBottom
            || table.page_break != TablePageBreak::RowBreak
            || cell.row_span != 1
            || cell.dirty_flag
            || table.common.height == 0
            || table.common.height > i32::MAX as u32
            || (previous.tag | next.tag) & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
            || previous.vertical_pos != 0
            || next.vertical_pos != 0
            || previous.line_height <= 0
            || next.line_height <= 0
            || previous.line_spacing < 0
            || previous.text_start >= next.text_start
            || previous.column_start != next.column_start
            || previous.segment_width <= 0
            || previous.segment_width != next.segment_width
        {
            return None;
        }
        // A paragraph change can reset a writer-local cursor. Require adjacent
        // source lines in this paragraph, then a physical frame or an independent
        // spanning owner whose next source ladder confirms the same row boundary.
        if !cell.paragraphs.iter().any(|para| {
            !para.stored_text_partition_is_dirty()
                && !para.cell_format_vpos_dirty
                && para.controls.is_empty()
                && !para.text.trim().is_empty()
                && para.line_segs.windows(2).enumerate().any(|(index, lines)| {
                    if !std::ptr::eq(&lines[0], previous) || !std::ptr::eq(&lines[1], next) {
                        return false;
                    }
                    // A terminal line needs no later position. When a
                    // following source line exists, its advance must agree
                    // with this restart; a later valid pair cannot certify
                    // the current cut or a manually broken source ladder.
                    let paragraph_index = cell
                        .paragraphs
                        .iter()
                        .position(|candidate| std::ptr::eq(candidate, para))
                        .expect("source paragraph belongs to cell");
                    let after = para.line_segs.get(index + 2).or_else(|| {
                        cell.paragraphs
                            .get(paragraph_index + 1)
                            .and_then(|following| following.line_segs.first())
                    });
                    after.is_none_or(|line| {
                        i64::from(line.vertical_pos)
                            == i64::from(next.line_height) + i64::from(next.line_spacing)
                    })
                })
        }) {
            return None;
        }
        if self.row_shared_zero_origin_frame_peer(table, cell) {
            return Some(SavedZeroOriginWitness::OpeningFrame);
        }
        // The original opening-frame witness owns positive trailing spacing.
        // A zero-spacing pair alone does not identify a physical page boundary.
        if previous.line_spacing == 0 {
            return None;
        }
        let rows = table.get_raw_row_heights();
        let row = usize::from(cell.row);
        let prefix = rows.get(..row)?;
        let &height = rows.get(row)?;
        if prefix.iter().any(|&height| height > i32::MAX as u32)
            || height == 0
            || height > i32::MAX as u32
        {
            return None;
        }
        let start = prefix.iter().map(|&height| i64::from(height)).sum::<i64>()
            + i64::from(table.cell_spacing) * row as i64;
        let frame = i64::from(table.common.height);
        // The first object box cannot bound every later page of a table.
        // An original in-paragraph return to zero in the same text column
        // still owns a line-frame boundary after that opening box. The raw
        // row boxes must independently prove that the object is a fragment;
        // paragraph-local origins and a complete object box do not suffice.
        // A merged-only row can have no single-row declaration. Zero then
        // contributes only a lower bound: it cannot authenticate an exact
        // opening cut, but cannot erase the known preceding physical space.
        if prefix.iter().all(|&height| height > 0)
            && start < frame
            && frame < start + i64::from(height)
        {
            Some(SavedZeroOriginWitness::OpeningFrame)
        } else if start >= frame && self.stored_row_boxes_exceed_first_frame(table) {
            Some(SavedZeroOriginWitness::LaterPage)
        } else {
            None
        }
    }

    pub(super) fn cut_starts_saved_zero_origin_text_frame(
        &self,
        table: &Table,
        cell: &Cell,
        units: &[CellUnit],
        next: usize,
    ) -> bool {
        let Some(previous) = next.checked_sub(1).and_then(|index| units.get(index)) else {
            return false;
        };
        let Some(following) = units.get(next) else {
            return false;
        };
        if previous.para_idx != following.para_idx || previous.vis_end != following.vis_start {
            return false;
        }
        cell.paragraphs.get(previous.para_idx).is_some_and(|para| {
            para.line_segs
                .get(previous.vis_end.saturating_sub(1))
                .zip(para.line_segs.get(following.vis_start))
                .is_some_and(|(before, after)| {
                    self.stored_zero_origin_text_frame_break(table, cell, before, after)
                })
        })
    }
}

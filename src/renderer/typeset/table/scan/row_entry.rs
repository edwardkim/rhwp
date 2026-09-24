//! 일반 행 분할 진입 Query. 주석 행 컷과 분할 가능성·패딩을 조회하며 수용/이월은 부모 소유다.

use super::row::RowScanQuery;
use super::RowBlockQuery;
use crate::model::provenance::LayoutCompatibilityProfile;
use crate::renderer::layout::table_layout::RowCutResult;
use crate::renderer::typeset::{is_synthetic_line_seg, para_has_visible_text};

pub(in crate::renderer::typeset) struct RowEntryQuery<'a> {
    pub(in crate::renderer::typeset) row: &'a RowScanQuery<'a>,
    pub(in crate::renderer::typeset) row_start_cut: &'a [usize],
}

pub(in crate::renderer::typeset) struct TerminalNoteProbe {
    pub(in crate::renderer::typeset) remaining_band: f64,
    pub(in crate::renderer::typeset) source_cut: RowCutResult,
}

pub(in crate::renderer::typeset) struct RowSplitGate {
    pub(in crate::renderer::typeset) native_short_parent_child_splittable: bool,
    pub(in crate::renderer::typeset) splittable: bool,
}

impl RowEntryQuery<'_> {
    pub(in crate::renderer::typeset) fn terminal_note_shape(
        &self,
        strict_painted_bottom_fit: bool,
        row_count: usize,
    ) -> bool {
        let Self { row, row_start_cut } = *self;
        let RowScanQuery {
            rows,
            r,
            cursor_row,
            ..
        } = *row;
        let RowBlockQuery {
            mt,
            table,
            rowspan_touched,
            ..
        } = *rows;
        !strict_painted_bottom_fit
            && mt.allows_row_break_split()
            && r > cursor_row
            && r + 1 == row_count
            && row_start_cut.is_empty()
            && !rowspan_touched[r]
            && {
                let mut cells = table
                    .cells
                    .iter()
                    .filter(|cell| cell.row as usize == r && cell.row_span == 1);
                cells.next().is_some_and(|cell| {
                    cells.next().is_none()
                        && cell.col_span as usize == table.col_count as usize
                        && cell.paragraphs.iter().all(|paragraph| {
                            paragraph.controls.is_empty()
                                && para_has_visible_text(paragraph)
                                && paragraph
                                    .line_segs
                                    .iter()
                                    .filter(|seg| !is_synthetic_line_seg(seg))
                                    .count()
                                    == 1
                        })
                })
            }
    }

    /// 형상 guard 안에서만 실행한다. 남은 예산이 0이어도 원본처럼 컷을 먼저 조회한다.
    pub(in crate::renderer::typeset) fn terminal_note_probe(
        &self,
        avail_for_rows: f64,
        consumed: f64,
        cs_before: f64,
    ) -> TerminalNoteProbe {
        let Self { row, row_start_cut } = *self;
        let RowScanQuery { rows, r, .. } = *row;
        let RowBlockQuery {
            layout_engine,
            table,
            styles,
            ..
        } = *rows;
        let remaining_band = (avail_for_rows - consumed - cs_before).max(0.0);
        let source_cut =
            layout_engine.advance_row_cut(table, r, row_start_cut, remaining_band, styles);

        TerminalNoteProbe {
            remaining_band,
            source_cut,
        }
    }

    /// native child 조회는 can_intra_split보다 먼저 실행하는 기존 순서를 보존한다.
    /// `intra_row_cut_here` 는 [#7288] «쪽 경계에서» 가 이 자리의 행 내부 컷을 허용하는지다.
    /// 값 0 «나누지 않음» 표의 원자 규칙은 호출처가 판정해 넘긴다.
    pub(in crate::renderer::typeset) fn split_gate(
        &self,
        can_intra_split: bool,
        intra_row_cut_here: bool,
    ) -> RowSplitGate {
        let RowScanQuery { rows, r, .. } = *self.row;
        let RowBlockQuery {
            layout_engine,
            mt,
            table,
            styles,
            ..
        } = *rows;
        let native_short_parent_child_splittable =
            layout_engine.native_short_parent_child_row_is_fragmentable(table, r, styles);
        let splittable = can_intra_split
            && intra_row_cut_here
            && (mt.is_row_splittable(r) || native_short_parent_child_splittable);

        RowSplitGate {
            native_short_parent_child_splittable,
            splittable,
        }
    }

    pub(in crate::renderer::typeset) fn padding(&self) -> f64 {
        let Self { row, row_start_cut } = *self;
        let RowScanQuery { rows, r, .. } = *row;
        let RowBlockQuery {
            layout_engine,
            mt,
            table,
            styles,
            ..
        } = *rows;
        if mt.allows_row_break_split() {
            layout_engine.row_remaining_visible_padding_height(table, r, row_start_cut, styles)
        } else {
            mt.max_padding_for_row(r)
        }
    }

    pub(in crate::renderer::typeset) fn native_reset_tail(
        &self,
        profile: &LayoutCompatibilityProfile,
    ) -> bool {
        let Self { row, row_start_cut } = *self;
        let RowScanQuery {
            rows,
            r,
            cursor_row,
            ..
        } = *row;
        let RowBlockQuery {
            layout_engine,
            mt,
            table,
            styles,
            ..
        } = *rows;
        profile.hwp5_stored_pagination_layout()
            && !table.common.treat_as_char
            && mt.allows_row_break_split()
            && r > cursor_row
            && row_start_cut.is_empty()
            && layout_engine.row_block_has_internal_hard_break(table, r, r + 1, styles)
    }
}

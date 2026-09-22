//! 블록 컷 결과의 수용 가능성 Query. 컷 실행·진단·누적 예약·커서 변경은 소유하지 않는다.
//! 기존 분기/허용치의 타당성을 새로 승인하지 않고 조회 경계만 분리한다.

use super::{RowBlockCandidate, RowBlockQuery};
use crate::renderer::layout::table_layout::RowCutResult;
use crate::renderer::typeset::{is_synthetic_line_seg, para_has_visible_text, MIN_TOP_KEEP_PX};

/// 같은 컷 후보의 불변 관측값. 전체 페이지 상태 대신 행 위치와 시작 컷만 받는다.
pub(in crate::renderer::typeset) struct BlockCutQuery<'a> {
    pub(in crate::renderer::typeset) rows: &'a RowBlockQuery<'a>,
    pub(in crate::renderer::typeset) block: &'a RowBlockCandidate,
    pub(in crate::renderer::typeset) r: usize,
    pub(in crate::renderer::typeset) cursor_row: usize,
    pub(in crate::renderer::typeset) blk_start_cut: &'a [usize],
    pub(in crate::renderer::typeset) res: &'a RowCutResult,
}

impl BlockCutQuery<'_> {
    /// 기존 저장 줄 완결 분기의 마지막 행 높이. guard 통과 때만 내용 높이를 측정한다.
    pub(in crate::renderer::typeset) fn source_complete_last_row_band(
        &self,
        budget: f64,
    ) -> Option<f64> {
        let Self {
            rows,
            block,
            r,
            cursor_row,
            blk_start_cut,
            res,
        } = *self;
        let RowBlockQuery {
            layout_engine,
            mt,
            table,
            styles,
            cut_row_h,
            cs,
            ..
        } = *rows;
        let RowBlockCandidate {
            b_start,
            b_end,
            block_size,
            rowbreak_use_row_offsets,
            ..
        } = *block;
        // RowBreak rowspan block의 선언 높이가 현재 body band를 넘더라도,
        // 실제 저장 line으로 만든 block content가 그 band 안에서 완결될 수
        // 있다. 이때 넘치는 부분은 cell의 의도된 내용이 아니라 선언된
        // 아래 blank 영역이다. 그 blank가 별도 physical page를 소유하면
        // 1741000처럼 짧은 tail page가 생긴다.
        //
        // source line이 없는 fresh reflow, nested/control block, cell 내부
        // hard break는 이 계약에 포함하지 않는다. 그런 형상은 선언 높이가
        // 실제 content frame을 대표하지 않을 수 있으므로 기존 split/이월
        // 경로가 계속 소유한다.
        // label cell 하나가 block 전체 행을 덮고, 각 행에는 그 label의
        // 오른쪽 폭 전체를 차지하는 response cell 하나만 있는 form 구조다.
        // 일반 평가 grid처럼 label 오른쪽에 여러 독립 열이 있으면 선언
        // blank도 각 열의 frame 일부이므로 이 경로로 압축하지 않는다.
        let block_is_label_response_form = table
            .cells
            .iter()
            .find(|cell| cell.row as usize == b_start && cell.row_span as usize == block_size)
            .is_some_and(|label| {
                (b_start..b_end).all(|row| {
                    let mut row_cells = table.cells.iter().filter(|cell| {
                        cell.row as usize == row && !(row == b_start && cell.col == label.col)
                    });
                    row_cells.next().is_some_and(|response| {
                        row_cells.next().is_none()
                            && response.row_span == 1
                            && response.col == label.col + label.col_span
                            && response.col_span + label.col_span == table.col_count
                    })
                })
            });
        let source_complete_rowspan_block = block_is_label_response_form
            && mt.allows_row_break_split()
            && r > cursor_row
            && blk_start_cut.is_empty()
            && !rowbreak_use_row_offsets
            && res.fully_consumed
            && !res.hit_hard_break
            && table
                .cells
                .iter()
                .filter(|cell| {
                    let cell_start = cell.row as usize;
                    let cell_end = cell_start + cell.row_span as usize;
                    cell_start < b_end && cell_end > b_start
                })
                .all(|cell| {
                    cell.paragraphs.iter().all(|paragraph| {
                        paragraph.controls.is_empty()
                            && (!para_has_visible_text(paragraph)
                                || paragraph
                                    .line_segs
                                    .iter()
                                    .any(|seg| !is_synthetic_line_seg(seg)))
                    })
                });

        if source_complete_rowspan_block {
            let remaining_band = budget;
            let source_content_height =
                layout_engine.row_block_content_height(table, b_start, b_end, &[], &[], styles);
            let before_last_row = (b_start..b_end.saturating_sub(1))
                .map(|row| cut_row_h[row])
                .sum::<f64>()
                + cs * b_end.saturating_sub(b_start + 1) as f64;
            let last_row_band = remaining_band - before_last_row;
            if source_content_height > 0.0
                && source_content_height <= remaining_band
                && last_row_band > 0.0
                && last_row_band <= cut_row_h[b_end - 1]
            {
                return Some(last_row_band);
            }
        }
        None
    }

    pub(in crate::renderer::typeset) fn allows_split(&self, genuinely_page_larger: bool) -> bool {
        let Self {
            block,
            r,
            cursor_row,
            res,
            ..
        } = *self;
        let rowbreak_rowspan_block = block.rowbreak_rowspan_block;
        if rowbreak_rowspan_block {
            r == cursor_row || (res.hit_hard_break && res.consumed_height >= MIN_TOP_KEEP_PX)
        } else {
            r == cursor_row || (genuinely_page_larger && res.consumed_height >= MIN_TOP_KEEP_PX)
        }
    }

    pub(in crate::renderer::typeset) fn retries_band(
        &self,
        allow_block_split: bool,
        can_intra_split: bool,
        block_h: f64,
        budget: f64,
    ) -> bool {
        let Self {
            rows,
            block,
            r,
            cursor_row,
            blk_start_cut,
            res,
        } = *self;
        let mt = rows.mt;
        let rowbreak_use_row_offsets = block.rowbreak_use_row_offsets;
        (res.fully_consumed || !allow_block_split)
            && mt.allows_row_break_split()
            && can_intra_split
            && !rowbreak_use_row_offsets
            && r > cursor_row
            && blk_start_cut.is_empty()
            && block_h > budget + 0.5
            && budget >= MIN_TOP_KEEP_PX
    }
}

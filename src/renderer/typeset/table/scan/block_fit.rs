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
        // [#7418] 종전에는 label 칸 하나 + 오른쪽 응답 칸 하나인 서식으로만 좁혔다("일반 격자는
        // 선언 blank 도 각 열의 frame 일부"라는 추론). 한/글 2020 정본은 그 서식만 압축하는 게
        // 아니다 — `22037757` 1쪽의 59×5 표 rowspan 묶음(행 11~12, 칸 선언 170.3px)은 선언으로
        // 본문을 5.8px 넘지만 내용이 들어가, 한/글이 1쪽에 싣고 행 12 를 본문 바닥(1010.8px)에서
        // 자른다(선언 94.6 → 88.4px). 다음 쪽에 이어지는 조각은 없다.
        //
        // 다만 **묶음 안에서 행으로 나뉘는 열이 하나일 때**로 좁힌다. 그 형상에서만 잘리는 것이
        // 한 열의 선언 빈 꼬리이고, 옛 label-응답 서식은 그 2열 특수형이다. 22037757 은 다섯 열
        // 중 넷이 묶음 전체를 span 하고 col 1 만 행으로 나뉜다(자르는 양 94.6 → 93.6 = 1.0px).
        //
        // ⚠ 열 여럿이 독립이면 선언 blank 는 각 열 frame 의 일부라 잘라선 안 된다. 실측:
        // `task2097/3248363_upmu_bunjang.hwpx` 는 네 열 중 셋이 행마다 독립인데(묶음 행 6~7)
        // 이 경로가 행 7 을 270.2 → **149.1px** 로 121px 잘라, 칸 글자가 용지 바닥 1122.5 아래
        // 1124.4·1147.1·1169.8px 에 그려졌다(off-canvas 1 · overflow_cell 3줄 신규).
        let block_subdivided_column_count = {
            let mut cols: Vec<u16> = table
                .cells
                .iter()
                .filter(|cell| {
                    let cell_start = cell.row as usize;
                    let cell_end = cell_start + cell.row_span as usize;
                    cell_start < b_end
                        && cell_end > b_start
                        && (cell.row_span as usize) < block_size
                })
                .map(|cell| cell.col)
                .collect();
            cols.sort_unstable();
            cols.dedup();
            cols.len()
        };
        let source_complete_rowspan_block = block_subdivided_column_count <= 1
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

//! 행 스캔의 rowspan 블록 분류·높이 Query.
//! 컷 후보를 소비하거나 페이지 예산/커서를 바꾸지 않으며 기존 계산 순서를 보존한다.

use crate::model::table::Table;
use crate::renderer::height_measurer::MeasuredTable;
use crate::renderer::layout::LayoutEngine;
use crate::renderer::style_resolver::ResolvedStyleSet;

pub(in crate::renderer::typeset) mod block_fit;
pub(in crate::renderer::typeset) mod block_fragment;
pub(in crate::renderer::typeset) mod landscape;
pub(in crate::renderer::typeset) mod row;
pub(in crate::renderer::typeset) mod row_entry;
pub(in crate::renderer::typeset) mod source_frame;

/// 원본 측정 행과 컷용 행 높이의 구분을 유지한 읽기 전용 스캔 입력.
pub(in crate::renderer::typeset) struct RowBlockQuery<'a> {
    pub(in crate::renderer::typeset) layout_engine: &'a LayoutEngine,
    pub(in crate::renderer::typeset) mt: &'a MeasuredTable,
    pub(in crate::renderer::typeset) table: &'a Table,
    pub(in crate::renderer::typeset) styles: &'a ResolvedStyleSet,
    pub(in crate::renderer::typeset) cut_row_h: &'a [f64],
    pub(in crate::renderer::typeset) rowspan_touched: &'a [bool],
    pub(in crate::renderer::typeset) cs: f64,
}

pub(in crate::renderer::typeset) struct RowBlockCandidate {
    pub(in crate::renderer::typeset) b_start: usize,
    pub(in crate::renderer::typeset) b_end: usize,
    pub(in crate::renderer::typeset) block_size: usize,
    pub(in crate::renderer::typeset) protected: bool,
    pub(in crate::renderer::typeset) rowbreak_rowspan_block: bool,
    pub(in crate::renderer::typeset) rowbreak_use_row_offsets: bool,
    rowbreak_block_content_exceeds_row_sum: bool,
}

impl RowBlockQuery<'_> {
    /// 분류는 매 행 원래 시점에 수행한다. 높이 계산은 블록 진입 뒤 별도로 질의한다.
    pub(in crate::renderer::typeset) fn candidate(&self, r: usize) -> RowBlockCandidate {
        let Self {
            layout_engine,
            mt,
            table,
            styles,
            cut_row_h,
            rowspan_touched,
            cs,
        } = *self;
        // rowspan 보호 블록 — 블록 전체를 분할 없이 한 단위로.
        let (b_start, b_end, _) = mt.row_block_for(r);
        let block_size = b_end.saturating_sub(b_start);
        let block_has_any_rowspan = block_size >= 2
            && (b_start..b_end).any(|x| rowspan_touched.get(x).copied().unwrap_or(false));
        let block_has_protectable_rowspan = block_has_any_rowspan
            && block_size <= crate::renderer::height_measurer::BLOCK_UNIT_MAX_ROWS;
        let rowbreak_hard_break_row =
            if mt.allows_row_break_split() && b_start == r && block_has_protectable_rowspan {
                layout_engine.row_block_first_internal_hard_break_row(table, b_start, b_end, styles)
            } else {
                None
            };
        let rowbreak_has_internal_hard_break = rowbreak_hard_break_row.is_some();
        let protected = block_has_protectable_rowspan
            && (!mt.allows_row_break_split() || !rowbreak_has_internal_hard_break);
        // [Task #1086] RowBreak 표는 행 경계 분할 정책이라 보호 블록
        // snap 은 피하지만, rowspan label 이 걸친 블록 안의 큰 row_span==1
        // 셀은 셀 내부 hard-break(vpos reset) 기준으로 쪼갤 수 있어야 한다.
        // 이때는 기존 블록 컷 경로를 재사용해 rowspan 셀과 일반 셀의 cut
        // 인덱스를 같은 정의로 렌더러까지 전달한다.
        let rowbreak_block_content_exceeds_row_sum =
            if mt.allows_row_break_split() && b_start == r && block_has_any_rowspan {
                let row_sum = (b_start..b_end).map(|x| cut_row_h[x]).sum::<f64>()
                    + cs * block_size.saturating_sub(1) as f64;
                let block_content =
                    layout_engine.row_block_content_height(table, b_start, b_end, &[], &[], styles)
                        + cs * block_size.saturating_sub(1) as f64;
                block_content > row_sum + 0.5
            } else {
                false
            };
        let rowbreak_rowspan_block = mt.allows_row_break_split()
            && b_start == r
            && block_has_any_rowspan
            && (rowbreak_has_internal_hard_break || rowbreak_block_content_exceeds_row_sum);
        // #1486: hard-break가 rowspan 블록 첫 행의 큰 셀 안에 있을 때만
        // 행 시작 y offset을 빼서 아래 행 셀을 다음 조각에 남긴다.
        // #1105처럼 hard-break가 뒤 행 셀 안에 있는 블록은 기존 블록 컷을
        // 유지해야 첫 조각의 `end_cut`이 한컴 기준과 맞는다.
        let rowbreak_use_row_offsets =
            rowbreak_rowspan_block && rowbreak_hard_break_row == Some(b_start);
        RowBlockCandidate {
            b_start,
            b_end,
            block_size,
            protected,
            rowbreak_rowspan_block,
            rowbreak_use_row_offsets,
            rowbreak_block_content_exceeds_row_sum,
        }
    }

    pub(in crate::renderer::typeset) fn row_offsets(&self, block: &RowBlockCandidate) -> Vec<f64> {
        let Self { cut_row_h, cs, .. } = *self;
        let RowBlockCandidate {
            b_start,
            b_end,
            block_size,
            rowbreak_use_row_offsets,
            ..
        } = *block;
        if rowbreak_use_row_offsets {
            let mut offsets = Vec::with_capacity(block_size);
            let mut top = 0.0;
            for br in b_start..b_end {
                offsets.push(top);
                top += cut_row_h[br] + if br + 1 < b_end { cs } else { 0.0 };
            }
            offsets
        } else {
            Vec::new()
        }
    }

    pub(in crate::renderer::typeset) fn fragment_height(
        &self,
        block: &RowBlockCandidate,
        row_end: usize,
        block_start_cut: &[usize],
        block_end_cut: &[usize],
    ) -> f64 {
        let Self {
            layout_engine,
            table,
            styles,
            cut_row_h,
            cs,
            ..
        } = *self;
        let RowBlockCandidate { b_start, b_end, .. } = *block;
        if block_start_cut.is_empty() && block_end_cut.is_empty() {
            return (b_start..row_end).map(|x| cut_row_h[x]).sum::<f64>()
                + cs * row_end.saturating_sub(b_start + 1) as f64;
        }

        let mut total = 0.0;
        let mut has_row = false;
        for br in b_start..row_end {
            let row_h = layout_engine.row_block_cut_row_content_height(
                table,
                b_start,
                b_end,
                br,
                block_start_cut,
                block_end_cut,
                styles,
            );
            if row_h > 0.0 {
                if has_row {
                    total += cs;
                }
                total += row_h;
                has_row = true;
            }
        }
        total
    }

    pub(in crate::renderer::typeset) fn required_height(
        &self,
        block: &RowBlockCandidate,
        blk_start_cut: &[usize],
    ) -> f64 {
        let Self {
            layout_engine,
            table,
            styles,
            cut_row_h,
            cs,
            ..
        } = *self;
        let RowBlockCandidate {
            b_start,
            b_end,
            block_size,
            rowbreak_use_row_offsets,
            rowbreak_block_content_exceeds_row_sum,
            ..
        } = *block;
        if blk_start_cut.is_empty() && !rowbreak_block_content_exceeds_row_sum {
            (b_start..b_end).map(|x| cut_row_h[x]).sum::<f64>()
                + cs * block_size.saturating_sub(1) as f64
        } else if rowbreak_use_row_offsets {
            // [#2287] 연속분(start_cut)의 per-row 합산은 row_span==1
            // 셀만 집계해, 걸친 rowspan 셀의 잔여 유닛이 0 으로
            // 평가된다 — 블록이 즉시 "fits" 로 종료되어 선언 잔여가
            // 통째로 증발(교육부 연결맵 47×9: 잔여 1904px → 표마다
            // 누적, 표 밀집 문서 -40~-64쪽 + 렌더 y=3026 오버플로).
            // spacer-트림 잔여(rowspan 셀 포함, 컷 워크 의미론 미러)로
            // 하한을 잡는다. per-row 합산이 유의한 높이를 주는 부분
            // 계상 사례(59043 병리 표: 음수 패딩 + 측정/렌더 발산,
            // #2237 계열)는 기존 동작 보존 — **완전 증발(=0)** 만 보정.
            // start_cut 없는 첫 조각(#1486)은 불변.
            let frag_h = self.fragment_height(block, b_end, blk_start_cut, &[]);
            if blk_start_cut.is_empty() || frag_h > 0.5 {
                frag_h
            } else {
                frag_h.max(layout_engine.row_block_cut_remaining_height(
                    table,
                    b_start,
                    b_end,
                    blk_start_cut,
                    styles,
                ))
            }
        } else {
            layout_engine.row_block_content_height(
                table,
                b_start,
                b_end,
                blk_start_cut,
                &[],
                styles,
            ) + cs * block_size.saturating_sub(1) as f64
        }
    }
}

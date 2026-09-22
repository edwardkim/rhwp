//! 채택한 블록 컷의 행 범위·점유 높이 Query. 끝 컷 복사와 누적 예약은 부모 스캔 소유다.

use super::{RowBlockCandidate, RowBlockQuery};
use crate::renderer::layout::table_layout::RowCutResult;

/// 기존 컷 또는 밴드 재시도 컷을 빌린 선택 결과. 컷 유닛을 복사하거나 상태를 바꾸지 않는다.
pub(in crate::renderer::typeset) struct SelectedBlockCut<'a> {
    pub(in crate::renderer::typeset) cut_res: &'a RowCutResult,
    cut_offsets: &'a [f64],
    use_offsets: bool,
}

impl<'a> SelectedBlockCut<'a> {
    pub(in crate::renderer::typeset) fn select(
        rowbreak_use_row_offsets: bool,
        res: &'a RowCutResult,
        block_row_offsets: &'a [f64],
        band_fill: &'a Option<(RowCutResult, Vec<f64>)>,
    ) -> Self {
        let (cut_res, cut_offsets) = if let Some((res2, offsets)) = band_fill {
            (res2, offsets.as_slice())
        } else {
            (res, block_row_offsets)
        };
        let use_offsets = rowbreak_use_row_offsets || band_fill.is_some();

        Self {
            cut_res,
            cut_offsets,
            use_offsets,
        }
    }

    pub(in crate::renderer::typeset) fn end_row(&self, block: &RowBlockCandidate) -> usize {
        let Self {
            cut_res,
            cut_offsets,
            use_offsets,
        } = *self;
        let RowBlockCandidate { b_start, b_end, .. } = *block;
        if use_offsets {
            let mut render_end = b_start + 1;
            for (idx, row_top) in cut_offsets.iter().enumerate() {
                if *row_top < cut_res.consumed_height - 0.1 {
                    render_end = b_start + idx + 1;
                }
            }
            render_end.min(b_end).max(b_start + 1)
        } else {
            b_end
        }
    }

    /// 부모가 끝 컷을 기록한 뒤 원래 시점에 호출한다. 측정을 select 시점으로 앞당기지 않는다.
    pub(in crate::renderer::typeset) fn occupied_height(
        &self,
        rows: &RowBlockQuery<'_>,
        block: &RowBlockCandidate,
        blk_start_cut: &[usize],
        end_row: usize,
    ) -> f64 {
        let Self {
            cut_res,
            use_offsets,
            ..
        } = *self;
        let RowBlockQuery {
            layout_engine,
            table,
            styles,
            ..
        } = *rows;
        let RowBlockCandidate { b_start, b_end, .. } = *block;
        if use_offsets {
            // [#2287] 오프셋(밴드) 컷의 페이지 소비 권위는 컷 워크의
            // consumed_height(예산 내 가시 밴드)다. per-row 합산
            // (block_fragment_height)은 rowspan 걸침 셀의 유닛을 행
            // 단위로 배분하지 못해 양방향으로 발산한다:
            // - 연속분(start_cut)에서 row_span==1 필터로 **0 평가**
            //   (완전 증발 — 표 밀집 -40~-64쪽의 결함 1), 또는
            // - 첫 조각에서 걸침 셀 유닛 전량이 계상되어 **블록 전체로
            //   과대** (교육부 47×9 r2..4: frag 2354.6 vs 컷 450.7 —
            //   p25 가 2396px 로 만재되어 p26 sliver + p30 tail
            //   overflow, PR #2290 P1 리뷰).
            // frag_total 은 렌더 조각 표시용 참고값으로만 두고, 소비는
            // 컷 워크와 발산할 때 consumed_height 로 정정한다.
            let frag_total = rows.fragment_height(block, end_row, blk_start_cut, &cut_res.end_cut);
            if (frag_total - cut_res.consumed_height).abs() <= 0.5 {
                frag_total
            } else {
                cut_res.consumed_height
            }
        } else {
            layout_engine.row_block_content_height(
                table,
                b_start,
                b_end,
                blk_start_cut,
                &cut_res.end_cut,
                styles,
            )
        }
    }
}

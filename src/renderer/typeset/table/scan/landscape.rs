//! 가로 용지 일반 행 수용 형상 Query. 기존 허용치를 보존하며 수용·이월 상태는 부모 소유다.

use super::row::RowScanQuery;
use super::RowBlockQuery;
use crate::model::provenance::LayoutCompatibilityProfile;

pub(in crate::renderer::typeset) struct LandscapeRowQuery<'a> {
    pub(in crate::renderer::typeset) row: &'a RowScanQuery<'a>,
    pub(in crate::renderer::typeset) row_start_cut: &'a [usize],
    pub(in crate::renderer::typeset) profile: &'a LayoutCompatibilityProfile,
    pub(in crate::renderer::typeset) landscape_rowbreak_bleed: bool,
    pub(in crate::renderer::typeset) is_continuation: bool,
    pub(in crate::renderer::typeset) header_overhead: f64,
    pub(in crate::renderer::typeset) bleed_absorbed_row_height: Option<f64>,
    pub(in crate::renderer::typeset) can_intra_split: bool,
    pub(in crate::renderer::typeset) table_storage_declares_splits: bool,
    pub(in crate::renderer::typeset) budget: LandscapeRowBudget,
}

/// 같은 스캔 반복의 불변 예산. 부모가 행을 수용하면 즉시 다음 반복으로 넘어간다.
pub(in crate::renderer::typeset) struct LandscapeRowBudget {
    pub(in crate::renderer::typeset) consumed: f64,
    pub(in crate::renderer::typeset) cs_before: f64,
    pub(in crate::renderer::typeset) row_total: f64,
    pub(in crate::renderer::typeset) avail_for_rows: f64,
}

impl LandscapeRowQuery<'_> {
    pub(in crate::renderer::typeset) fn whole_row_shape(
        &self,
        landscape_whole_row_tolerance: f64,
    ) -> bool {
        let Self {
            row,
            row_start_cut,
            landscape_rowbreak_bleed,
            is_continuation,
            header_overhead,
            bleed_absorbed_row_height,
            ..
        } = *self;
        let RowScanQuery {
            rows,
            r,
            cursor_row,
            ..
        } = *row;
        let mt = rows.mt;
        let LandscapeRowBudget {
            consumed,
            cs_before,
            row_total,
            avail_for_rows,
        } = self.budget;
        landscape_rowbreak_bleed
                && mt.allows_row_break_split()
                && is_continuation
                && header_overhead > 0.0
                && row_start_cut.is_empty()
                && r > cursor_row
                // [#5828] 같은 높이 행의 연속 흡수 금지는 아래 short-row 분기와
                // 공유한다 — 균일 pitch 기계 표가 두 분기를 번갈아 타며 행을
                // 계속 받는 것을 막는다.
                && !bleed_absorbed_row_height
                    .is_some_and(|prev| (prev - row_total).abs() < 0.5)
                && consumed + cs_before + row_total
                    <= avail_for_rows + landscape_whole_row_tolerance
    }

    /// 전체 행 수용 실패 뒤에만 호출한다. profile별 저장 reset 조회를 미리 실행하지 않는다.
    pub(in crate::renderer::typeset) fn short_row_shape(
        &self,
        landscape_short_row_max_height: f64,
        landscape_short_row_tolerance: f64,
        has_internal_saved_vpos_reset: impl Fn() -> bool,
    ) -> bool {
        let Self {
            row,
            row_start_cut,
            profile,
            landscape_rowbreak_bleed,
            is_continuation,
            header_overhead,
            bleed_absorbed_row_height,
            ..
        } = *self;
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
            rowspan_touched,
            ..
        } = *rows;
        let LandscapeRowBudget {
            consumed,
            cs_before,
            row_total,
            avail_for_rows,
        } = self.budget;
        landscape_rowbreak_bleed
                && mt.allows_row_break_split()
                && is_continuation
                && header_overhead > 0.0
                && row_start_cut.is_empty()
                && r > cursor_row
                && !rowspan_touched[r]
                // Direct HWPX의 local reset은 physical frame이 아니다. 선언 cell 안에
                // source frame이 완결될 때만 short-row bleed를 막아 source owner를
                // 보존한다.
                && !(((profile.hwp5_stored_pagination_layout() || profile.hwp5_origin_hwpx())
                    && has_internal_saved_vpos_reset())
                    || (profile.hwpx_stored_layout()
                        && !profile.hwp5_origin_hwpx()
                        && layout_engine.row_has_stored_vpos_frame_rewind(table, r)))
                && row_total <= landscape_short_row_max_height
                // [#5828] 이 흡수는 **경계에 걸친 행 하나**를 위한 것이다(#1672 의
                // 의도). 종전에는 tolerance 가 누적 consumed 에 계속 적용돼 경계를
                // 넘어선 뒤에도 행을 받았다 — 156505020 구역3(가용 604.8px)에서
                // 쪽마다 26행(843px, 용지 +121px)을 얹어 한글 43쪽 vs rhwp 33쪽.
                // 직전까지의 consumed 가 예산 안일 때만 이 행으로 경계를 한 번
                // 넘고, 그 다음 행부터는 이 분기가 닫힌다(한글 실측: 쪽당 17행).
                // [#5828] 이 흡수는 경계에 걸친 짧은 잔여 행을 위한 것이다
                // (#1672). 종전에는 tolerance 가 누적 consumed 에 계속 적용돼 균일
                // 32.4px 행 기계 표에서 8행을 연속 흡수했다 — 156505020 구역3:
                // 쪽마다 26행(843px, 용지 +121px), 한글 43쪽 vs rhwp 33쪽. 한글
                // 정합이 확인된 이종 행 문서(편람 383쪽 핀, #4763)의 연속 흡수는
                // 전부 서로 다른 높이(38.7~280.2px)이므로, **같은 높이 행의 연속
                // 흡수**만 막는다 — 균일 pitch 기계 표는 쪽당 1행에서 닫히고
                // 이종 행 수동 문서의 계약은 그대로 유지된다.
                && !bleed_absorbed_row_height
                    .is_some_and(|prev| (prev - row_total).abs() < 0.5)
                && consumed + cs_before + row_total
                    <= avail_for_rows + landscape_short_row_tolerance
    }

    /// 세 호출 지점의 선행 guard와 반복 조회를 보존한다. 형상과 합쳐 선행 계산하지 않는다.
    pub(in crate::renderer::typeset) fn boundary_splittable(&self) -> bool {
        let Self {
            row,
            can_intra_split,
            table_storage_declares_splits,
            ..
        } = *self;
        let mt = row.rows.mt;
        let r = row.r;
        can_intra_split && mt.is_row_splittable(r) && !table_storage_declares_splits
    }
}

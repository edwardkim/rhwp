//! 일반 행·rowspan 행의 요구 높이와 잔여 밴드 Query. 예산 수용과 상태 반영은 부모 소유다.

use super::RowBlockQuery;
use crate::model::control::Control;
use crate::renderer::layout::table_layout::RowCutResult;
use crate::renderer::typeset::MIN_TOP_KEEP_PX;

pub(in crate::renderer::typeset) struct RowScanQuery<'a> {
    pub(in crate::renderer::typeset) rows: &'a RowBlockQuery<'a>,
    pub(in crate::renderer::typeset) r: usize,
    pub(in crate::renderer::typeset) cursor_row: usize,
    pub(in crate::renderer::typeset) start_cut: &'a [usize],
    pub(in crate::renderer::typeset) start_row_height_override: Option<f64>,
}

pub(in crate::renderer::typeset) struct RowBandShape {
    pub(in crate::renderer::typeset) has_prior_rowspan_cover: bool,
    pub(in crate::renderer::typeset) row_has_nested: bool,
}

pub(in crate::renderer::typeset) struct RowBandProbe {
    pub(in crate::renderer::typeset) probe: RowCutResult,
    pub(in crate::renderer::typeset) visible_height: f64,
}

impl RowScanQuery<'_> {
    /// 앞 행이 실제 수용한 높이를 차감한다. 원본 측정 행과 컷용 행 높이를 합치지 않는다.
    pub(in crate::renderer::typeset) fn required_height(
        &self,
        height: f64,
        consumed: f64,
        cs_before: f64,
    ) -> f64 {
        let Self {
            rows,
            r,
            cursor_row,
            start_cut,
            start_row_height_override,
        } = *self;
        let RowBlockQuery {
            layout_engine,
            mt,
            table,
            styles,
            ..
        } = *rows;
        layout_engine
            .straddle_continuation_demand(
                table,
                r,
                cursor_row,
                start_cut,
                start_row_height_override,
                &mt.row_heights,
                styles,
                (r + 1, true),
            )
            .map_or(height, |need| height.max(need - consumed - cs_before))
    }

    pub(in crate::renderer::typeset) fn whole_row_height(
        &self,
        row_start_cut: &[usize],
        whole_row_fit_h: &[f64],
    ) -> f64 {
        let Self { rows, r, .. } = *self;
        let RowBlockQuery {
            layout_engine,
            table,
            styles,
            ..
        } = *rows;
        if row_start_cut.is_empty() {
            whole_row_fit_h[r]
        } else {
            // 연속분 cursor_row — 시작 컷 적용. row_cut_content_height 가
            // 셀별 (content+pad) 행 max 를 반환(분할 행이므로 cell.height
            // 강제 없음).
            layout_engine.row_cut_content_height(table, r, row_start_cut, &[], styles, false)
        }
    }

    pub(in crate::renderer::typeset) fn band_shape(&self) -> RowBandShape {
        let Self { rows, r, .. } = *self;
        let table = rows.table;
        let has_prior_rowspan_cover = table.cells.iter().any(|c| {
            c.row_span > 1 && (c.row as usize) < r && r < c.row as usize + c.row_span as usize
        });
        let row_has_nested = table.cells.iter().any(|c| {
            c.row as usize == r
                && c.paragraphs.iter().any(|p| {
                    p.controls
                        .iter()
                        .any(|ctrl| matches!(ctrl, Control::Table(_)))
                })
        });

        RowBandShape {
            has_prior_rowspan_cover,
            row_has_nested,
        }
    }

    /// 부모의 기존 guard 안에서만 실행한다. 컷 조회와 실제 표시 높이 조회 순서를 유지한다.
    pub(in crate::renderer::typeset) fn probe_band(
        &self,
        row_start_cut: &[usize],
        rest: f64,
    ) -> RowBandProbe {
        let Self { rows, r, .. } = *self;
        let RowBlockQuery {
            layout_engine,
            table,
            styles,
            ..
        } = *rows;
        let padding =
            layout_engine.row_remaining_visible_padding_height(table, r, row_start_cut, styles);
        let content_budget = (rest - padding).max(0.0);
        let probe = layout_engine.advance_row_cut(table, r, row_start_cut, content_budget, styles);
        let visible_height = layout_engine.row_cut_content_height(
            table,
            r,
            row_start_cut,
            &probe.end_cut,
            styles,
            false,
        );

        RowBandProbe {
            probe,
            visible_height,
        }
    }
}

pub(in crate::renderer::typeset) fn retains_blank_tail(
    probe: &RowCutResult,
    visible_height: f64,
    rest: f64,
) -> bool {
    let retained_blank_tail = (rest - visible_height).max(0.0);
    probe.fully_consumed && visible_height <= rest + 0.5 && retained_blank_tail >= MIN_TOP_KEEP_PX
}

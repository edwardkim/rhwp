//! 일반 행·rowspan 행의 요구 높이와 잔여 밴드 Query. 예산 수용과 상태 반영은 부모 소유다.

use super::RowBlockQuery;
use crate::model::control::Control;
use crate::renderer::layout::{table_layout::RowCutResult, LayoutEngine};
use crate::renderer::typeset::{MIN_TOP_KEEP_PX, TERMINAL_ROW_BOTTOM_SQUEEZE_TOLERANCE_PX};

pub(in crate::renderer::typeset) struct RowScanQuery<'a> {
    pub(in crate::renderer::typeset) rows: &'a RowBlockQuery<'a>,
    pub(in crate::renderer::typeset) r: usize,
    pub(in crate::renderer::typeset) cursor_row: usize,
    pub(in crate::renderer::typeset) start_cut: &'a [usize],
    pub(in crate::renderer::typeset) start_cut_is_block: bool,
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
    /// 유닛 소비 완료와 실제 칸 상자의 수용을 구분한다.
    /// 밴드 컷은 내용과 안 여백이 모두 들어갈 때만 남은 물리 공간을 소유한다.
    pub(in crate::renderer::typeset) fn complete_cut_band_height(
        &self,
        end_cut: &[usize],
        available: f64,
    ) -> Option<f64> {
        let start = if self.r == self.cursor_row {
            self.start_cut
        } else {
            &[]
        };
        let occupied = self.rows.layout_engine.row_cut_content_height(
            self.rows.table,
            self.r,
            start,
            end_cut,
            self.rows.styles,
        );
        (available.is_finite() && available > 0.0 && occupied <= available).then_some(available)
    }

    /// 문단 유지 계약을 통과한 컷이 실제로 소유할 상자를 수용 전에 확정한다.
    /// 내용 높이만으로 최소 조각을 검사한 뒤 paint에서 상자를 늘리지 않는다.
    /// 반환값은 누적 예약과 부분 행 상자에 함께 전달한다.
    pub(in crate::renderer::typeset) fn plain_cut_physical_height(
        &self,
        end_cut: &[usize],
        visible_height: f64,
        available_height: f64,
        owns_minimum: bool,
        fills_page_end: bool,
    ) -> Option<f64> {
        let table = self.rows.table;
        if fills_page_end
            && table.cells.iter().any(|cell| {
                cell.row_span > 1
                    && crate::renderer::layout::cell_crosses_fragment_end(cell, self.r + 1, end_cut)
            })
        {
            return Some(available_height.max(visible_height));
        }
        if !owns_minimum {
            return None;
        }
        let minimum = self.rows.layout_engine.reflow_row_physical_minimum(
            table,
            self.r,
            self.rows.mt.row_heights[self.r],
            Some(end_cut),
            self.rows.styles,
        )?;
        let remaining = if self.r == self.cursor_row {
            self.start_row_height_override.unwrap_or(minimum)
        } else {
            minimum
        };
        Some(if visible_height <= available_height {
            remaining.min(available_height).max(visible_height)
        } else {
            visible_height
        })
    }

    /// 통짜/분할 경로가 내용 완결 후 같은 실제 끝 띠를 소비한다.
    /// 다른 칸의 미래 공간과 이 행에서 닫히는 원본 병합 칸을 구분한다.
    pub(in crate::renderer::typeset) fn complete_source_band_can_close(
        &self,
        row_heights: &[f64],
        row_total: f64,
        physical: f64,
    ) -> bool {
        let Self {
            rows,
            r,
            cursor_row,
            ..
        } = *self;
        let continues = rows.table.cells.iter().any(|cell| {
            let first = cell.row as usize;
            cell.row_span > 1 && first <= r && first + cell.row_span as usize > r + 1
        });
        row_total <= physical + TERMINAL_ROW_BOTTOM_SQUEEZE_TOLERANCE_PX
            && (!continues
                || rows.layout_engine.stored_closing_span_content_fits(
                    rows.table,
                    r,
                    cursor_row,
                    row_heights,
                    physical,
                    rows.styles,
                ))
    }

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
            start_cut_is_block,
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
                start_cut_is_block,
                start_row_height_override,
                &mt.row_heights,
                rows.straddle_page_bands,
                styles,
                (r + 1, true),
            )
            .map_or(height, |need| {
                height.max(need.required() - consumed - cs_before)
            })
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
        if r == self.cursor_row {
            if let Some(height) = LayoutEngine::carried_span_row_physical_height(
                table,
                r,
                row_start_cut,
                self.start_cut_is_block,
                self.start_row_height_override,
            ) {
                return height;
            }
        }
        if row_start_cut.is_empty() {
            whole_row_fit_h[r]
        } else {
            // 연속분 cursor_row — 시작 컷 적용. row_cut_content_height 가
            // 셀별 (content+pad) 행 max 를 반환(분할 행이므로 cell.height
            // 강제 없음).
            let content =
                layout_engine.row_cut_content_height(table, r, row_start_cut, &[], styles);
            // The continuation cursor carries space already accepted by the
            // previous fragment, independently of its consumed content units.
            self.start_row_height_override
                .map_or(content, |physical| content.max(physical))
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
        let (probe, _) =
            layout_engine.advance_row_cut_for_physical_band(table, r, row_start_cut, rest, styles);
        let visible_height =
            layout_engine.row_cut_content_height(table, r, row_start_cut, &probe.end_cut, styles);

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

/// [#5585] 일반 행 밴드 컷(내용과 안 여백은 남은 쪽에 다 들어가지만 선언 행 높이가 넘치는
/// 행을 쪽 경계에서 자름)을 받을 수 있는 행의 형상. rowspan 이 걸친 행은 `#2236` 밴드 컷이
/// 따로 맡고, 쪽의 첫 행은 이미 강제 배치된다(호출부 조건).
///
/// 칸 내용이 글줄 유닛만으로 이뤄진 행만 자른다. 칸 안의 그림·도형·표·수식·각주 등
/// 개체는 내용 컷(`end_cut`)에 높이가 잡히지 않아, 글줄이 다 들어가도 개체가 선언 높이를
/// 채우고 있을 수 있다 — 한글도 그런 행은 통째로 넘긴다(1220000-202100003 23쪽 13×7 표
/// 행 3: 글앞으로 그림 세 장이 선언 247.6px 를 채우고 글줄은 29.3px, 한글 41쪽 유지).
///
/// 빈 밴드는 선언 행 높이가 내용보다 클 때만 생긴다 — 측정 행 높이가 칸의 선언 높이를
/// 넘으면 그 높이는 내용이 만든 것이고 내용 컷이 그 내용을 다 담지 못한 것이므로 기존
/// 행내 분할 경로에 맡긴다.
pub(in crate::renderer::typeset) fn ordinary_band_row_shape(
    table: &crate::model::table::Table,
    row: usize,
    row_total: f64,
    dpi: f64,
) -> bool {
    row_content_is_line_units_only(table, row)
        && row_total <= declared_row_height_px(table, row, dpi) + 0.5
}

/// 행의 선언 높이 — 이 행에서 시작해 이 행에서 끝나는 칸의 저장 높이 중 최댓값(px).
fn declared_row_height_px(table: &crate::model::table::Table, row: usize, dpi: f64) -> f64 {
    table
        .cells
        .iter()
        .filter(|cell| cell.row as usize == row && cell.row_span <= 1)
        .map(|cell| crate::renderer::hwpunit_to_px(cell.height as i32, dpi))
        .fold(0.0f64, f64::max)
}

/// 행의 칸들이 글줄 유닛 밖의 개체(표·도형·그림·수식·양식·각주/미주)를 품지 않는가.
fn row_content_is_line_units_only(table: &crate::model::table::Table, row: usize) -> bool {
    table
        .cells
        .iter()
        .filter(|cell| cell.row as usize == row)
        .flat_map(|cell| cell.paragraphs.iter())
        .flat_map(|paragraph| paragraph.controls.iter())
        .all(|control| {
            !matches!(
                control,
                Control::Table(_)
                    | Control::Shape(_)
                    | Control::Picture(_)
                    | Control::Equation(_)
                    | Control::Form(_)
                    | Control::Footnote(_)
                    | Control::Endnote(_)
                    | Control::Unknown(_)
            )
        })
}

/// 이 행이 병합 칸의 경계를 갖는다 — 병합 칸이 이 행에서 시작하거나 끝난다. 그런 칸은
/// 남은 내용을 다음 쪽에서 같은 행의 컷으로 이어야 하므로(한/글도 그 띠를 다음 쪽
/// 첫머리에 잇는다) 기존 밴드 규칙을 쓴다. 앞 행의 병합 칸 안쪽에만 있는 행이
/// «남은 높이로 끝내기» 규칙의 대상이다.
pub(in crate::renderer::typeset) fn row_bounds_a_rowspan(
    table: &crate::model::table::Table,
    r: usize,
) -> bool {
    table.cells.iter().any(|cell| {
        let first = cell.row as usize;
        let last = first + cell.row_span.max(1) as usize - 1;
        cell.row_span > 1 && (first == r || last == r)
    })
}

/// 내용 상자 높이가 남은 높이를 넘지 않는다. 픽셀 미만 허용치는 없다 — 한/글은
/// 0.04px 넘치는 행도 다음 쪽으로 넘긴다(`samples/table_scattered_header_rowbreak.hwp`
/// 2쪽 끝 행 61). 부동소수 오차만 받는다.
pub(in crate::renderer::typeset) fn row_box_fits(height: f64, rest: f64) -> bool {
    height <= rest + 1e-6
}

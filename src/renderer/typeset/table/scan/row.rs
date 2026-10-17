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

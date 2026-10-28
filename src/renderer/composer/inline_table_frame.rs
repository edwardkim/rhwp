//! 저장 줄이 없는 TAC 표의 흐름과 배치가 공유하는 실제 측정 결과.

use crate::model::table::Table;
use crate::model::{control::Control, paragraph::Paragraph};
use crate::renderer::height_measurer::{HeightMeasurer, MeasuredTable};
use crate::renderer::style_resolver::ResolvedStyleSet;

pub(crate) fn inline_table_needs_reflow_measurement(table: &Table) -> bool {
    table.common.treat_as_char
        && (super::table_cells_have_no_stored_lines(table)
            || inline_table_width_cache_is_stale(table))
}

pub(crate) fn inline_table_width_cache_is_stale(table: &Table) -> bool {
    table
        .closed_inline_grid_width_hu()
        .is_some_and(|width| width != table.common.width)
}

/// 재조판 셀의 내용·행 하한·캡션을 개체 선언 높이로 다시 덮어쓰지 않는다.
/// 저장 줄이 있는 표는 기존 저장 프레임 경로에서 측정 결과를 공급한다.
pub(crate) fn measured_reflow_inline_table(
    table: &Table,
    styles: &ResolvedStyleSet,
    dpi: f64,
) -> Option<MeasuredTable> {
    inline_table_needs_reflow_measurement(table).then(|| {
        HeightMeasurer::new(dpi)
            .with_session_edited(true)
            .measure_table(table, 0, 0, styles)
    })
}

/// 저장 줄 없는 본문도 셀과 같은 표 실측을 현재 줄의 소유 정보로 발행한다.
/// 캡션을 제외한 본체 높이는 앵커용이며, 줄의 전체 점유는 기존 조판기가 측정한다.
pub(crate) fn reflow_inline_table_heights(
    paragraph: &Paragraph,
    styles: &ResolvedStyleSet,
    dpi: f64,
) -> Vec<(usize, u32)> {
    if !crate::renderer::para_has_no_stored_line_segs(paragraph) {
        return Vec::new();
    }
    paragraph
        .controls
        .iter()
        .enumerate()
        .filter_map(|(index, control)| {
            let Control::Table(table) = control else {
                return None;
            };
            let measured = measured_reflow_inline_table(table, styles, dpi)?;
            let body = measured.total_height - measured.caption_height;
            let height = u32::try_from(crate::renderer::px_to_hwpunit(body, dpi)).ok()?;
            (height > 0).then_some((index, height))
        })
        .collect()
}

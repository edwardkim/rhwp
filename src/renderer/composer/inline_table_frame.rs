//! 저장 줄이 없는 TAC 표의 흐름과 배치가 공유하는 실제 측정 결과.

use crate::model::table::Table;
use crate::model::{control::Control, paragraph::Paragraph};
use crate::renderer::height_measurer::{HeightMeasurer, MeasuredTable};
use crate::renderer::style_resolver::ResolvedStyleSet;

pub(crate) fn inline_table_needs_reflow_measurement(table: &Table) -> bool {
    match inline_table_needs_reflow_measurement_with_storage(table, &crate::storage::Unbounded) {
        Ok(value) => value,
        Err(never) => match never {},
    }
}
pub(crate) fn inline_table_needs_reflow_measurement_with_storage<S: crate::storage::Storage>(
    table: &Table,
    storage: &S,
) -> Result<bool, S::Error> {
    Ok(table.common.treat_as_char
        && (super::table_cells_have_no_stored_lines(table)
            || inline_table_width_cache_is_stale_with_storage(table, storage)?))
}
pub(crate) fn inline_table_width_cache_is_stale(table: &Table) -> bool {
    match inline_table_width_cache_is_stale_with_storage(table, &crate::storage::Unbounded) {
        Ok(value) => value,
        Err(never) => match never {},
    }
}
pub(crate) fn inline_table_width_cache_is_stale_with_storage<S: crate::storage::Storage>(
    table: &Table,
    storage: &S,
) -> Result<bool, S::Error> {
    Ok(table
        .closed_inline_grid_width_hu_with_storage(storage)?
        .is_some_and(|width| width != table.common.width))
}

/// 재조판 셀의 내용·행 하한·캡션을 개체 선언 높이로 다시 덮어쓰지 않는다.
/// 저장 줄이 있는 표는 기존 저장 프레임 경로에서 측정 결과를 공급한다.
pub(crate) fn measured_reflow_inline_table(
    table: &Table,
    styles: &ResolvedStyleSet,
    dpi: f64,
) -> Option<MeasuredTable> {
    match measured_reflow_inline_table_with_storage(table, styles, dpi, &crate::storage::Unbounded)
    {
        Ok(value) => value,
        Err(never) => match never {},
    }
}
pub(crate) fn measured_reflow_inline_table_with_storage<S: crate::storage::Storage>(
    table: &Table,
    styles: &ResolvedStyleSet,
    dpi: f64,
    storage: &S,
) -> Result<Option<MeasuredTable>, S::Error> {
    if !inline_table_needs_reflow_measurement_with_storage(table, storage)? {
        return Ok(None);
    }
    HeightMeasurer::new(dpi)
        .with_session_edited(true)
        .measure_table_with_storage(table, 0, 0, styles, storage)
        .map(Some)
}

/// 글자(공백만이어도)가 있는 host 의 양수 세로 오프셋 TAC 표는 줄 상자 밖에 놓인다.
///
/// 조판(`place_table_with_text` 의 표 앞 글 경로, 같은 `!text.is_empty()` 와 부호 있는
/// 세로 오프셋 판정)은 이 host 의 줄을 모두 표 앞 글로 쌓은 뒤 표 높이를 따로 전진한다. 이 표를 프레임 줄의 점유로 발행하면 표 높이가
/// 줄 전진과 표 배치에 두 번 계상되어 표가 쪽 아래 경계 밖으로 밀린다(#8463).
fn table_placed_below_host_lines(paragraph: &Paragraph, table: &Table) -> bool {
    !paragraph.text.is_empty()
        && crate::renderer::float_placement::captioned_table_flow_vertical_offset_hu(table) > 0
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
            if table_placed_below_host_lines(paragraph, table) {
                return None;
            }
            let measured = measured_reflow_inline_table(table, styles, dpi)?;
            let body = measured.total_height - measured.caption_height;
            let height = u32::try_from(crate::renderer::px_to_hwpunit(body, dpi)).ok()?;
            (height > 0).then_some((index, height))
        })
        .collect()
}

pub(crate) fn reflow_inline_table_heights_with_storage<S: crate::storage::Storage>(
    paragraph: &Paragraph,
    styles: &ResolvedStyleSet,
    dpi: f64,
    storage: &S,
) -> Result<Vec<(usize, u32)>, S::Error> {
    let mut heights = storage.vec(0)?;
    if !crate::renderer::para_has_no_stored_line_segs(paragraph) {
        return Ok(heights);
    }
    for (index, control) in paragraph.controls.iter().enumerate() {
        let Control::Table(table) = control else {
            continue;
        };
        if table_placed_below_host_lines(paragraph, table)
            || !inline_table_needs_reflow_measurement_with_storage(table, storage)?
        {
            continue;
        }
        let measured = HeightMeasurer::new(dpi)
            .with_session_edited(true)
            .measure_table_with_storage(table, 0, 0, styles, storage)?;
        if let Ok(height) = u32::try_from(crate::renderer::px_to_hwpunit(
            measured.total_height - measured.caption_height,
            dpi,
        )) {
            if height > 0 {
                storage.push(&mut heights, (index, height))?;
            }
        }
    }
    Ok(heights)
}

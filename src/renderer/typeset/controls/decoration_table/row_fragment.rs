//! 장식 표도 행 내부의 내용 컷과 물리 높이를 같은 셀 유닛 결과로 전달한다.

use crate::model::table::{Table, TablePageBreak};
use crate::renderer::{
    layout::LayoutEngine,
    pagination::OverlayRowFragment,
    style_resolver::ResolvedStyleSet,
    typeset::{FormattedTable, TypesetEngine, TypesetState},
};

pub(super) fn query(
    engine: &TypesetEngine,
    state: &TypesetState,
    table: &Table,
    formatted: &FormattedTable,
    styles: &ResolvedStyleSet,
    row: usize,
    available: f64,
) -> Option<OverlayRowFragment> {
    if !matches!(table.page_break, TablePageBreak::RowBreak)
        || table.cells.iter().any(|cell| {
            usize::from(cell.row) <= row
                && row < usize::from(cell.row) + usize::from(cell.row_span)
                && cell.row_span > 1
        })
        || available <= 0.0
    {
        return None;
    }
    let layout = LayoutEngine::new(engine.dpi);
    engine.prime_section_cell_context(&layout);
    layout.set_layout_profile(state.profile);
    layout.prime_column_layout_env(&state.layout);
    layout.set_render_normalization_overlay(std::sync::Arc::clone(&engine.render_normalization));
    layout.remember_stored_frame_rows(table, &formatted.row_heights);
    let cut = layout.advance_row_cut(table, row, &[], available, styles);
    if cut.fully_consumed || cut.end_cut.iter().all(|end| *end == 0) {
        return None;
    }
    // 강제 진행으로 얻은 컷은 실제 패딩까지 예산 안에 들 때만 수용한다.
    let opening_height = layout.row_cut_content_height(table, row, &[], &cut.end_cut, styles);
    if opening_height <= 0.0 || opening_height > available {
        return None;
    }
    let remaining_height = layout.row_cut_content_height(table, row, &cut.end_cut, &[], styles);
    Some(OverlayRowFragment {
        end_cut: cut.end_cut,
        opening_height,
        remaining_height,
    })
}

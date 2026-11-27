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
    if !matches!(table.layout_page_break(), TablePageBreak::RowBreak)
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
    if let Some(opening_height) = layout.stored_overlay_opening_closing_row(
        table,
        row,
        &formatted.row_heights,
        available,
        styles,
    ) {
        return Some(OverlayRowFragment {
            fully_consumed: true,
            end_cut: Vec::new(),
            opening_height,
            remaining_height: 0.0,
            owns_stored_frame: true,
        });
    }
    let is_overlay = super::row_contains_cell_overlay(table, row);
    let overlay_frame =
        is_overlay.then(|| super::overlay_opening_frame(table, available, engine.dpi));
    let cut = layout.advance_row_cut(table, row, &[], overlay_frame.unwrap_or(available), styles);
    if (!is_overlay && cut.fully_consumed) || cut.end_cut.iter().all(|end| *end == 0) {
        return None;
    }
    let content_height = layout.row_cut_content_height(table, row, &[], &cut.end_cut, styles);
    let declared = crate::renderer::height_measurer::declared_row_height_px(table, row, engine.dpi);
    let overlay_used = overlay_frame.map(|frame| frame.min(content_height.max(declared)));
    let source_frame = layout
        .stored_inline_opening_row_frame(table, row, &cut.end_cut, &formatted.row_heights, styles)
        .filter(|height| *height <= available);
    let opening_height = source_frame.or(overlay_used).unwrap_or(content_height);
    if opening_height <= 0.0 || opening_height > available {
        return None;
    }
    let remaining_height = if cut.fully_consumed {
        0.0
    } else {
        layout.row_cut_content_height(table, row, &cut.end_cut, &[], styles)
    };
    Some(OverlayRowFragment {
        fully_consumed: cut.fully_consumed,
        end_cut: cut.end_cut,
        opening_height,
        remaining_height,
        owns_stored_frame: source_frame.is_some() || overlay_frame.is_some(),
    })
}

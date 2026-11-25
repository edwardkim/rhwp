//! 장식 표도 행 내부의 내용 컷과 물리 높이를 같은 셀 유닛 결과로 전달한다.

use crate::model::table::{Table, TablePageBreak};
use crate::renderer::{
    hwpunit_to_px,
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
    let mut cut = layout.advance_row_cut(table, row, &[], available, styles);
    // 자식 표를 나눈 부모 칸의 물리 높이는 쪽 예산에서 바깥 표 여백만 제외한다.
    // 자식 여백은 각 컷의 수용·세로 정렬에서 해당 자식의 끝에만 소비한다.
    let overlay_frame = layout
        .row_cut_splits_cell_overlay(table, row, &cut.end_cut, styles)
        .then(|| {
            crate::renderer::float_placement::rowbreak_fragment_bounds(
                table,
                available - hwpunit_to_px(i32::from(table.outer_margin_top), engine.dpi),
                engine.dpi,
                false,
                true,
            )
            .blank_bottom
        });
    if let Some(frame) = overlay_frame {
        cut = layout.advance_row_cut(table, row, &[], frame, styles);
    }
    if cut.fully_consumed || cut.end_cut.iter().all(|end| *end == 0) {
        return None;
    }
    let content_height = layout.row_cut_content_height(table, row, &[], &cut.end_cut, styles);
    let source_frame = layout
        .stored_inline_opening_row_frame(table, row, &cut.end_cut, &formatted.row_heights, styles)
        .filter(|height| *height <= available);
    let opening_height = source_frame.or(overlay_frame).unwrap_or(content_height);
    if opening_height <= 0.0 || opening_height > available {
        return None;
    }
    let remaining_height = layout.row_cut_content_height(table, row, &cut.end_cut, &[], styles);
    Some(OverlayRowFragment {
        end_cut: cut.end_cut,
        opening_height,
        remaining_height,
        owns_stored_frame: source_frame.is_some() || overlay_frame.is_some(),
    })
}

//! 장식 표도 행 내부의 내용 컷과 물리 높이를 같은 셀 유닛 결과로 전달한다.

use crate::model::control::Control;
use crate::model::table::{Table, TablePageBreak, VerticalAlign};
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
    let cut = layout.advance_row_cut(table, row, &[], available, styles);
    if cut.fully_consumed || cut.end_cut.iter().all(|end| *end == 0) {
        return None;
    }
    // 강제 진행으로 얻은 컷은 실제 패딩까지 예산 안에 들 때만 수용한다.
    let content_height = layout.row_cut_content_height(table, row, &[], &cut.end_cut, styles);
    let source_frame = layout
        .stored_inline_opening_row_frame(table, row, &cut.end_cut, &formatted.row_heights, styles)
        .filter(|height| *height <= available);
    // 본문보다 긴 표에서 칸 안 글 앞/뒤 중첩 표가 나뉘면, 한/글은 앞 칸 조각을 쪽 끝 가까이
    // 채우고 그 띠 안에서 칸 세로 정렬로 이번 조각의 중첩 표 상자를 놓는다(가운데 정렬이면
    // 위·아래 여백이 같다). 이 조각의 정렬 높이는 중첩 표의 바깥 아래 여백을 세지 않으므로,
    // 칸 조각을 쪽 끝(바깥 표 위·아래 여백 제외)에서 그 여백만큼 줄여 같은 배치를 만든다.
    // 저장 프레임이 있으면 그것을 쓴다.
    let overlay_cell_frame =
        if source_frame.is_none() && formatted.effective_height > state.base_available_height() {
            table
                .cells
                .iter()
                .filter(|cell| {
                    usize::from(cell.row) == row
                        && cell.vertical_align != VerticalAlign::Top
                        && layout.cell_overlay_table_reach(cell, table, styles) > 0.0
                })
                .flat_map(|cell| cell.paragraphs.iter().flat_map(|para| para.controls.iter()))
                .filter_map(|control| match control {
                    Control::Table(nested) if !nested.common.treat_as_char => {
                        Some(i32::from(nested.outer_margin_bottom))
                    }
                    _ => None,
                })
                .max()
                .map(|nested_margin_bottom| {
                    available
                        - hwpunit_to_px(
                            i32::from(table.outer_margin_top)
                                + i32::from(table.outer_margin_bottom)
                                + nested_margin_bottom,
                            engine.dpi,
                        )
                })
                .filter(|frame| *frame > content_height)
        } else {
            None
        };
    let opening_height = source_frame
        .or(overlay_cell_frame)
        .unwrap_or(content_height);
    if opening_height <= 0.0 || opening_height > available {
        return None;
    }
    let remaining_height = layout.row_cut_content_height(table, row, &cut.end_cut, &[], styles);
    Some(OverlayRowFragment {
        end_cut: cut.end_cut,
        opening_height,
        remaining_height,
        owns_stored_frame: source_frame.is_some() || overlay_cell_frame.is_some(),
    })
}

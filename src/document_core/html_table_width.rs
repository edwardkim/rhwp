//! HTML 표의 열은 자식 개체와 줄 나눔 토큰의 최소 점유 폭을 담는다.
//! 선호 폭이 작은 셀을 paint에서 숨기지 않고 import 단계의 공용 그리드를 넓힌다.

use crate::model::{
    control::Control,
    paragraph::Paragraph,
    table::{Cell, Table},
};
use crate::renderer::{
    composer::{self, BreakToken, ParagraphBox},
    hwpunit_to_px, px_to_hwpunit,
    style_resolver::ResolvedStyleSet,
};

fn paragraph_minimum_width(para: &Paragraph, styles: &ResolvedStyleSet, dpi: f64) -> u32 {
    let style = styles.para_styles.get(para.para_shape_id as usize);
    let chars: Vec<_> = para.text.chars().collect();
    // 재측정 규칙을 만들지 않고 현재 줄 나눔의 동일한 어절·글자 토큰을 소비한다.
    let text_width = composer::tokenize_paragraph(
        &chars,
        &para.char_offsets,
        &para.char_shapes,
        styles,
        style.map_or(0, |s| s.english_break_unit),
        style.map_or(0, |s| s.korean_break_unit),
    )
    .into_iter()
    .filter_map(|token| match token {
        BreakToken::Text { width, .. } => Some(width),
        _ => None,
    })
    .fold(0.0_f64, f64::max);
    let margins = style.map_or(0.0, |s| s.margin_left + s.margin_right + s.indent.max(0.0));
    let text_width = px_to_hwpunit((text_width + margins).max(0.0), dpi).max(0) as u32;
    // HTML parser는 중첩 표를 독립 문단으로 만든다. 글과 다른 줄의 개체 폭은
    // 글 폭에 합산하지 않으며 부모 셀이 그 바깥 여백까지 담아야 한다.
    let object_width = para
        .controls
        .iter()
        .filter_map(|control| match control {
            Control::Table(table) => Some(
                table
                    .get_column_widths()
                    .iter()
                    .copied()
                    .sum::<u32>()
                    .saturating_add(table.outer_margin_left.max(0) as u32)
                    .saturating_add(table.outer_margin_right.max(0) as u32),
            ),
            Control::Picture(picture) => Some(
                picture
                    .common
                    .width
                    .saturating_add(picture.common.margin.left.max(0) as u32)
                    .saturating_add(picture.common.margin.right.max(0) as u32),
            ),
            _ => None,
        })
        .max()
        .unwrap_or(0);
    text_width.max(object_width)
}

pub(super) fn reserve_content_widths(
    cells: &mut [Cell],
    columns: &mut [u32],
    table_padding: &crate::model::Padding,
    styles: &ResolvedStyleSet,
    dpi: f64,
) {
    // 자식 표는 재귀 import를 마친 상태다. 해당 폭을 아래에서 위로 부모 열에 예약한다.
    for cell in cells.iter() {
        let effective = cell.effective_padding(table_padding);
        let padding = effective.left.max(0) as u32 + effective.right.max(0) as u32;
        let required = cell
            .paragraphs
            .iter()
            .map(|para| paragraph_minimum_width(para, styles, dpi))
            .max()
            .unwrap_or(0)
            .saturating_add(padding);
        let start = usize::from(cell.col);
        let end = start + usize::from(cell.col_span.max(1));
        let Some(span) = columns.get_mut(start..end) else {
            continue;
        };
        let deficit = required.saturating_sub(span.iter().sum());
        let count = span.len() as u32;
        for (index, width) in span.iter_mut().enumerate() {
            *width =
                width.saturating_add(deficit / count + u32::from((index as u32) < deficit % count));
        }
    }
    for cell in cells {
        let start = usize::from(cell.col);
        let end = start + usize::from(cell.col_span.max(1));
        let Some(span) = columns.get(start..end) else {
            continue;
        };
        let width = span.iter().sum();
        cell.width = width;
        if let Some(raw) = cell.raw_list_extra.get_mut(..4) {
            raw.copy_from_slice(&width.to_le_bytes());
        }
        let effective = cell.effective_padding(table_padding);
        let inner = composer::cell_inner_text_width(
            hwpunit_to_px(width.min(i32::MAX as u32) as i32, dpi),
            hwpunit_to_px(i32::from(effective.left), dpi),
            hwpunit_to_px(i32::from(effective.right), dpi),
            dpi,
        );
        for para in &mut cell.paragraphs {
            composer::reflow_line_segs(
                para,
                ParagraphBox::content_width_px(inner, dpi),
                styles,
                dpi,
            );
        }
    }
}

/// Import 결과는 저장 LineSeg를 재사용하기 전에 현재 내용의 행 높이를 확정한다.
pub(super) fn reserve_content_heights(table: &mut Table, styles: &ResolvedStyleSet, dpi: f64) {
    let measured = crate::renderer::height_measurer::HeightMeasurer::new(dpi)
        .with_session_edited(true)
        .measure_table(table, 0, 0, styles);
    let heights: Vec<u32> = measured
        .row_heights
        .iter()
        .map(|height| px_to_hwpunit(*height, dpi).max(0) as u32)
        .collect();
    for cell in &mut table.cells {
        let start = usize::from(cell.row);
        let end = start + usize::from(cell.row_span.max(1));
        if let Some(rows) = heights.get(start..end) {
            cell.height = rows.iter().sum();
        }
    }
    table.sync_ctrl_height(heights.iter().sum());
}

//! 가시 NO_LS 호스트의 실제 줄 구성을 공통 부동 셀 흐름에 전달한다.

use crate::model::{paragraph::Paragraph, provenance::StoredGeometryPolicy, table::Table};
use crate::renderer::{
    composer::{self, ComposedParagraph, ParagraphBox},
    hwpunit_to_px,
    style_resolver::ResolvedStyleSet,
};

use super::{has_reflow_block_cell_host, ReflowBlockCellFlow};

/// 실제 셀 프레임의 폭을 알고 있는 측정·전체/부분 배치의 공통 진입점이다.
pub(crate) fn reflow_block_cell_flow_in_width(
    paragraphs: &[Paragraph],
    styles: &ResolvedStyleSet,
    dpi: f64,
    inner_width: f64,
    policy: StoredGeometryPolicy,
    table_body_height: impl FnMut(&Table) -> f64,
) -> Option<ReflowBlockCellFlow> {
    if !has_reflow_block_cell_host(paragraphs) || !inner_width.is_finite() || inner_width <= 0.0 {
        return None;
    }
    let composed: Vec<_> = paragraphs
        .iter()
        .map(|para| {
            let mut result = composer::compose_paragraph_in_context(para, styles);
            if crate::renderer::para_has_no_stored_line_segs(para) && !para.text.is_empty() {
                // 수용된 컨트롤은 비인라인 표뿐이다. 글줄 폭·명시적 개행은 기존
                // 셀 조판기에 맡기고 외부 표 본체를 글줄 높이에 더하지 않는다.
                composer::recompose_cell_lines_in_frame(
                    &mut result,
                    para,
                    ParagraphBox::content_for_style(
                        inner_width,
                        styles.para_styles.get(para.para_shape_id as usize),
                        dpi,
                    ),
                    styles,
                    dpi,
                    policy,
                );
            }
            result
        })
        .collect();
    reflow_block_cell_flow_with_composed(paragraphs, &composed, styles, dpi, table_body_height)
}

/// 이미 구성한 줄은 새로 너비·줄 수를 추측하지 않고 그대로 소비한다.
pub(crate) fn reflow_block_cell_flow_with_composed(
    paragraphs: &[Paragraph],
    composed: &[ComposedParagraph],
    styles: &ResolvedStyleSet,
    dpi: f64,
    table_body_height: impl FnMut(&Table) -> f64,
) -> Option<ReflowBlockCellFlow> {
    if paragraphs.len() != composed.len() || !has_reflow_block_cell_host(paragraphs) {
        return None;
    }
    let heights: Vec<_> = paragraphs
        .iter()
        .zip(composed)
        .enumerate()
        .map(|(pi, (para, comp))| {
            let frame = comp.paragraph_with_frame_geometry(para);
            let stored = !frame.line_segs.is_empty();
            let style = styles.para_styles.get(para.para_shape_id as usize);
            comp.lines
                .iter()
                .enumerate()
                .map(|(li, line)| {
                    let last = pi + 1 == paragraphs.len() && li + 1 == comp.lines.len();
                    let raw = hwpunit_to_px(line.line_height, dpi);
                    let font = line
                        .runs
                        .iter()
                        .filter_map(|run| styles.char_styles.get(run.char_style_id as usize))
                        .map(|style| style.font_size)
                        .fold(0.0_f64, f64::max);
                    let height = if stored {
                        raw
                    } else {
                        crate::renderer::corrected_line_height_for_variant_synthetic(
                            raw,
                            font,
                            style.map_or(crate::model::style::LineSpacingType::Percent, |s| {
                                s.line_spacing_type
                            }),
                            style.map_or(160.0, |s| s.line_spacing),
                            last,
                        )
                    };
                    height
                        + if last {
                            0.0
                        } else {
                            hwpunit_to_px(line.line_spacing, dpi)
                        }
                })
                .sum()
        })
        .collect();
    super::reflow_block_cell::reflow_block_cell_flow_with_heights(
        paragraphs,
        styles,
        dpi,
        Some(&heights),
        table_body_height,
    )
}

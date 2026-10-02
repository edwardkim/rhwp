//! 읽기 전용 단/배제 영역에서 inline 배치 후보를 계산한다. 페이지 상태는 변경하지 않는다.

use std::collections::BTreeMap;

use crate::model::paragraph::Paragraph;
use crate::renderer::float_placement::ObjectPlacementFrame;
use crate::renderer::height_measurer::MeasuredTable;
use crate::renderer::inline_flow::{self, InlineFlowPlan};
use crate::renderer::layout_frame::FrameExclusion;
use crate::renderer::page_layout::LayoutRect;
use crate::renderer::style_resolver::ResolvedStyleSet;

/// 참조만 보관하며 배제 영역 복사는 기존 스타일 조회 이후에 수행한다.
pub(in crate::renderer::typeset) struct InlineFlowInput<'a> {
    pub column: LayoutRect,
    pub body: &'a LayoutRect,
    pub page_width: f64,
    pub page_height: f64,
    pub start: f64,
    pub exclusions: Option<&'a BTreeMap<(usize, usize), FrameExclusion>>,
}

pub(super) fn build_plan(
    input: InlineFlowInput<'_>,
    para: &Paragraph,
    para_index: usize,
    styles: &ResolvedStyleSet,
    tables: &[MeasuredTable],
    dpi: f64,
) -> Option<InlineFlowPlan> {
    let column = input.column;
    let style = styles.para_styles.get(para.para_shape_id as usize)?;
    let container = LayoutRect {
        x: column.x + style.margin_left,
        width: (column.width - style.margin_left - style.margin_right).max(0.0),
        ..column
    };
    let paper = LayoutRect {
        x: 0.0,
        y: 0.0,
        width: input.page_width,
        height: input.page_height,
    };
    let exclusions: Vec<_> = if let Some(exclusions) = input.exclusions {
        exclusions.values().cloned().collect()
    } else {
        Vec::new()
    };
    let frame = ObjectPlacementFrame {
        container: &container,
        column: &column,
        body: input.body,
        paper: &paper,
        paragraph_y: column.y + input.start,
        alignment: style.alignment,
        dpi,
    };
    let mut plan = if inline_flow::supports_plain_text(para) {
        inline_flow::plan_plain_text(para, styles, &frame, &exclusions)?
    } else {
        inline_flow::plan(para, para_index, styles, tables, &frame, &exclusions)?
    };
    plan.relative_to(column.x, column.y);
    Some(plan)
}

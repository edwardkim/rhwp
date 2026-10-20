//! 떠 있는 수식의 선언 프레임과 표시 내용 상자를 분리한다.

use crate::model::{control::Equation, style::Alignment};
use crate::renderer::equation::{self, inline_box};
use crate::renderer::hwpunit_to_px;
use crate::renderer::page_layout::LayoutRect;
use crate::renderer::render_tree::{
    BoundingBox, EquationNode, PageLayoutContext, RenderNode, RenderNodeType,
};

use super::LayoutEngine;

impl LayoutEngine {
    /// 실제 개체의 기준 영역·정렬·부호 있는 오프셋을 공통 배치 계약으로 해소한다.
    /// 선언 프레임은 정렬에, 논리 내용 상자는 수식 표시와 기준선에 사용한다.
    /// 이미 줄에 등록된 인라인 수식의 소유는 호출자가 유지한다.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn layout_floating_equation(
        &self,
        tree: &mut PageLayoutContext,
        parent: &mut RenderNode,
        eq: &Equation,
        section_index: usize,
        para_index: usize,
        control_index: usize,
        col_area: &LayoutRect,
        body_area: &LayoutRect,
        paper_area: &LayoutRect,
        para_y: f64,
        alignment: Alignment,
    ) {
        let horizontal = inline_box::horizontal_metrics(eq);
        let eq_h = hwpunit_to_px(equation::flow_height_hwp(eq) as i32, self.dpi);
        let (frame_w, frame_h) = if eq.common.treat_as_char {
            // 별도 인라인 좌표가 없는 경로의 기존 문단 정렬을 유지한다.
            (
                hwpunit_to_px(horizontal.occupied_width_hwp(), self.dpi),
                eq_h,
            )
        } else {
            self.resolve_object_size(&eq.common, col_area, body_area, paper_area)
        };
        let (eq_x, eq_y) = self.compute_object_position(
            &eq.common, frame_w, frame_h, col_area, col_area, body_area, paper_area, para_y,
            alignment,
        );

        let ast = equation::parser::EqParser::for_document(equation::document_script::tokenize(
            &eq.script,
        ))
        .parse();
        let font_size_px = hwpunit_to_px(eq.font_size as i32, self.dpi);
        let layout_box = equation::layout_for_paint(eq, &ast, self.dpi);
        let color_str = equation::svg_render::eq_color_to_svg(eq.color);
        let svg_content =
            equation::svg_render::render_equation_svg(&layout_box, &color_str, font_size_px);
        parent.children.push(RenderNode::new(
            tree.next_id(),
            RenderNodeType::Equation(EquationNode {
                svg_content,
                layout_box,
                color_str,
                color: eq.color,
                script: eq.script.clone(),
                font_size: font_size_px,
                section_index: Some(section_index),
                para_index: Some(para_index),
                control_index: Some(control_index),
                cell_index: None,
                cell_para_index: None,
                note_ref: None,
            }),
            BoundingBox::new(
                horizontal.content_origin_x(eq_x, self.dpi),
                eq_y,
                hwpunit_to_px(horizontal.content_width_hwp, self.dpi),
                eq_h,
            ),
        ));
    }
}

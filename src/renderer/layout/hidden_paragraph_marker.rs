//! 조판에서 이미 흡수한 빈 문단의 편집용 문단부호와 커서 소속을 보존한다.

use super::paragraph_layout::paragraph_active_text_style;
use super::*;

impl LayoutEngine {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn layout_hidden_paragraph_marker(
        &self,
        tree: &mut PageLayoutContext,
        col_node: &mut RenderNode,
        para: &Paragraph,
        composed: &ComposedParagraph,
        styles: &ResolvedStyleSet,
        col_area: &LayoutRect,
        flow_y: f64,
        section_index: usize,
        para_index: usize,
    ) {
        // hidden_empty_paras의 소유 문단은 물리 줄을 예약하지 않는다. 일반 문단
        // 배치를 다시 호출하면 저장 vpos와 앞 간격을 재소비하고 조판과 달라진다.
        // 편집 표시의 크기는 원본 글자모양에서 얻고 흐름 원점은 그대로 소비한다.
        let (style, char_shape_id) = paragraph_active_text_style(styles, Some(para), 0);
        let height = style.font_size;
        let baseline = height * 0.85;
        let para_style = styles.para_styles.get(composed.para_style_id as usize);
        let left = para_style.map_or(0.0, |s| {
            crate::renderer::equation_tac_flow::paragraph_effective_margin_left(
                s.margin_left,
                s.indent,
                0,
            )
        });
        let right = para_style.map_or(0.0, |s| s.margin_right);
        let width = col_area.width - left - right;
        let x = col_area.x
            + left
            + match para_style.map(|s| s.alignment) {
                Some(Alignment::Center | Alignment::Distribute) => width / 2.0,
                Some(Alignment::Right) => width,
                _ => 0.0,
            };
        let mut line = RenderNode::new(
            tree.next_id(),
            RenderNodeType::TextLine(TextLineNode::with_para(
                height,
                baseline,
                section_index,
                para_index,
            )),
            BoundingBox::new(x, flow_y, 0.0, height),
        )
        .with_editor_only();
        line.children.push(RenderNode::new(
            tree.next_id(),
            RenderNodeType::TextRun(TextRunNode {
                // 공백도 원본 문자 오프셋을 보존하되 인쇄 흐름에는 참여하지 않는다.
                text: para.text.clone(),
                style,
                char_shape_id,
                para_shape_id: Some(composed.para_style_id),
                section_index: Some(section_index),
                para_index: Some(para_index),
                char_start: Some(0),
                cell_context: None,
                is_para_end: true,
                is_line_break_end: false,
                rotation: 0.0,
                is_vertical: false,
                char_overlap: None,
                border_fill_id: 0,
                baseline,
                field_marker: FieldMarkerType::None,
                layout_positions: None,
                display_text: None,
            }),
            BoundingBox::new(x, flow_y, 0.0, height),
        ));
        col_node.children.push(line);
    }
}

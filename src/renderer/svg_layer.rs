use crate::paint::{ClipKind, GroupKind, LayerNode, LayerNodeKind, PageLayerTree, PaintOp};

use super::layer_renderer::{LayerRenderResult, LayerRenderer};
use super::render_tree::{
    BoundingBox, GroupNode, PageRenderTree, RenderNode, RenderNodeType, TableCellNode,
};
use super::svg::SvgRenderer;

/// PageLayerTree를 SVG로 재생하는 transition renderer.
///
/// 1차 전환에서는 layer tree를 temporary render node tree로 다시 조립해
/// 기존 SVG leaf/backend 로직을 그대로 재사용한다.
pub struct SvgLayerRenderer {
    renderer: SvgRenderer,
    next_generated_id: u32,
}

impl SvgLayerRenderer {
    pub fn new() -> Self {
        Self {
            renderer: SvgRenderer::new(),
            next_generated_id: 1_000_000,
        }
    }

    pub fn output(&self) -> &str {
        self.renderer.output()
    }

    pub fn inner_mut(&mut self) -> &mut SvgRenderer {
        &mut self.renderer
    }

    fn build_render_tree(&mut self, tree: &PageLayerTree) -> PageRenderTree {
        let mut render_tree = PageRenderTree::new(0, tree.page_width, tree.page_height);
        render_tree.root.bbox = tree.root.bounds;
        render_tree.root.children = self.expand_children(&tree.root);
        render_tree
    }

    fn expand_children(&mut self, node: &LayerNode) -> Vec<RenderNode> {
        match &node.kind {
            LayerNodeKind::Group { children, .. } => children
                .iter()
                .flat_map(|child| self.expand_node(child))
                .collect(),
            LayerNodeKind::ClipRect { .. } | LayerNodeKind::Leaf { .. } => self.expand_node(node),
        }
    }

    fn expand_node(&mut self, node: &LayerNode) -> Vec<RenderNode> {
        match &node.kind {
            LayerNodeKind::Group {
                children,
                group_kind,
                ..
            } => {
                let mut render_node = RenderNode::new(
                    self.take_node_id(node.source_node_id),
                    self.group_kind_to_render_node_type(group_kind),
                    node.bounds,
                );
                render_node.children = children
                    .iter()
                    .flat_map(|child| self.expand_node(child))
                    .collect();
                vec![render_node]
            }
            LayerNodeKind::ClipRect {
                clip,
                child,
                clip_kind,
            } => {
                let node_type = match clip_kind {
                    ClipKind::Body => RenderNodeType::Body {
                        clip_rect: Some(*clip),
                    },
                    ClipKind::TableCell => match &child.kind {
                        LayerNodeKind::Group {
                            group_kind: GroupKind::TableCell(cell),
                            ..
                        } => {
                            let mut cell = cell.clone();
                            cell.clip = true;
                            RenderNodeType::TableCell(cell)
                        }
                        _ => RenderNodeType::TableCell(TableCellNode {
                            col: 0,
                            row: 0,
                            col_span: 1,
                            row_span: 1,
                            border_fill_id: 0,
                            text_direction: 0,
                            clip: true,
                            model_cell_index: None,
                        }),
                    },
                    ClipKind::Generic => RenderNodeType::Body {
                        clip_rect: Some(*clip),
                    },
                };

                let mut render_node = RenderNode::new(
                    self.take_node_id(node.source_node_id),
                    node_type,
                    node.bounds,
                );
                render_node.children = self.expand_children(child);
                vec![render_node]
            }
            LayerNodeKind::Leaf { ops } => ops
                .iter()
                .map(|op| self.paint_op_to_render_node(op, node.source_node_id))
                .collect(),
        }
    }

    fn paint_op_to_render_node(&mut self, op: &PaintOp, source_node_id: Option<u32>) -> RenderNode {
        match op {
            PaintOp::PageBackground { bbox, background } => RenderNode::new(
                self.take_node_id(source_node_id),
                RenderNodeType::PageBackground(background.clone()),
                *bbox,
            ),
            PaintOp::TextRun { bbox, run } => RenderNode::new(
                self.take_node_id(source_node_id),
                RenderNodeType::TextRun(run.clone()),
                *bbox,
            ),
            PaintOp::FootnoteMarker { bbox, marker } => RenderNode::new(
                self.take_node_id(source_node_id),
                RenderNodeType::FootnoteMarker(marker.clone()),
                *bbox,
            ),
            PaintOp::Line { bbox, line } => RenderNode::new(
                self.take_node_id(source_node_id),
                RenderNodeType::Line(line.clone()),
                *bbox,
            ),
            PaintOp::Rectangle { bbox, rect } => RenderNode::new(
                self.take_node_id(source_node_id),
                RenderNodeType::Rectangle(rect.clone()),
                *bbox,
            ),
            PaintOp::Ellipse { bbox, ellipse } => RenderNode::new(
                self.take_node_id(source_node_id),
                RenderNodeType::Ellipse(ellipse.clone()),
                *bbox,
            ),
            PaintOp::Path { bbox, path } => RenderNode::new(
                self.take_node_id(source_node_id),
                RenderNodeType::Path(path.clone()),
                *bbox,
            ),
            PaintOp::Image { bbox, image } => RenderNode::new(
                self.take_node_id(source_node_id),
                RenderNodeType::Image(image.clone()),
                *bbox,
            ),
            PaintOp::Equation { bbox, equation } => RenderNode::new(
                self.take_node_id(source_node_id),
                RenderNodeType::Equation(equation.clone()),
                *bbox,
            ),
            PaintOp::FormObject { bbox, form } => RenderNode::new(
                self.take_node_id(source_node_id),
                RenderNodeType::FormObject(form.clone()),
                *bbox,
            ),
            PaintOp::Placeholder { bbox, placeholder } => RenderNode::new(
                self.take_node_id(source_node_id),
                RenderNodeType::Placeholder(placeholder.clone()),
                *bbox,
            ),
            PaintOp::RawSvg { bbox, raw } => RenderNode::new(
                self.take_node_id(source_node_id),
                RenderNodeType::RawSvg(raw.clone()),
                *bbox,
            ),
        }
    }

    fn group_kind_to_render_node_type(&self, group_kind: &GroupKind) -> RenderNodeType {
        match group_kind {
            GroupKind::Generic => RenderNodeType::Group(GroupNode {
                section_index: None,
                para_index: None,
                control_index: None,
            }),
            GroupKind::MasterPage => RenderNodeType::MasterPage,
            GroupKind::Header => RenderNodeType::Header,
            GroupKind::Footer => RenderNodeType::Footer,
            GroupKind::Body => RenderNodeType::Body { clip_rect: None },
            GroupKind::Column(index) => RenderNodeType::Column(*index),
            GroupKind::FootnoteArea => RenderNodeType::FootnoteArea,
            GroupKind::TextLine(line) => RenderNodeType::TextLine(line.clone()),
            GroupKind::Table(table) => RenderNodeType::Table(table.clone()),
            GroupKind::TableCell(cell) => RenderNodeType::TableCell(cell.clone()),
            GroupKind::TextBox => RenderNodeType::TextBox,
            GroupKind::Group(group) => RenderNodeType::Group(group.clone()),
        }
    }

    fn take_node_id(&mut self, source_node_id: Option<u32>) -> u32 {
        if let Some(id) = source_node_id {
            return id;
        }
        let id = self.next_generated_id;
        self.next_generated_id += 1;
        id
    }
}

impl LayerRenderer for SvgLayerRenderer {
    fn render_page(&mut self, tree: &PageLayerTree) -> LayerRenderResult<()> {
        self.renderer.show_paragraph_marks = tree.output_options.show_paragraph_marks;
        self.renderer.show_control_codes = tree.output_options.show_control_codes;
        self.renderer.debug_overlay = tree.output_options.debug_overlay;
        let render_tree = self.build_render_tree(tree);
        self.renderer.render_tree(&render_tree);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::paint::{LayerBuilder, LayerOutputOptions, RenderProfile};
    use crate::renderer::render_tree::{
        PageNode, PlaceholderNode, RawSvgNode, RectangleNode, TextRunNode,
    };
    use crate::renderer::svg::SvgRenderer;
    use crate::renderer::{ShapeStyle, TextStyle};

    #[test]
    fn replays_basic_layer_tree_to_same_svg() {
        let mut render_tree = PageRenderTree::new(0, 400.0, 300.0);
        render_tree.root.node_type = RenderNodeType::Page(PageNode {
            page_index: 0,
            width: 400.0,
            height: 300.0,
            section_index: 0,
        });
        let mut line = RenderNode::new(
            10,
            RenderNodeType::TextLine(crate::renderer::render_tree::TextLineNode::new(20.0, 15.0)),
            BoundingBox::new(20.0, 20.0, 120.0, 20.0),
        );
        line.children.push(RenderNode::new(
            11,
            RenderNodeType::TextRun(TextRunNode {
                text: "레이어".to_string(),
                style: TextStyle {
                    font_family: "Noto Sans CJK KR".to_string(),
                    font_size: 14.0,
                    ..Default::default()
                },
                char_shape_id: None,
                para_shape_id: None,
                section_index: None,
                para_index: None,
                char_start: None,
                cell_context: None,
                is_para_end: false,
                is_line_break_end: false,
                rotation: 0.0,
                is_vertical: false,
                char_overlap: None,
                border_fill_id: 0,
                baseline: 15.0,
                field_marker: Default::default(),
            }),
            BoundingBox::new(20.0, 20.0, 60.0, 20.0),
        ));
        render_tree.root.children.push(line);
        render_tree.root.children.push(RenderNode::new(
            12,
            RenderNodeType::Rectangle(RectangleNode::new(
                0.0,
                ShapeStyle {
                    fill_color: Some(0x00F0F0F0),
                    stroke_color: Some(0x00000000),
                    stroke_width: 1.0,
                    ..Default::default()
                },
                None,
            )),
            BoundingBox::new(18.0, 18.0, 90.0, 28.0),
        ));

        let mut legacy = SvgRenderer::new();
        legacy.render_tree(&render_tree);

        let mut builder = LayerBuilder::new(RenderProfile::Screen);
        let layer_tree = builder.build(&render_tree);
        let mut layer = SvgLayerRenderer::new();
        layer.render_page(&layer_tree).unwrap();

        assert_eq!(layer.output(), legacy.output());
    }

    #[test]
    fn replays_raw_svg_and_placeholder_to_same_svg() {
        let mut render_tree = PageRenderTree::new(0, 160.0, 100.0);
        render_tree.root.node_type = RenderNodeType::Page(PageNode {
            page_index: 0,
            width: 160.0,
            height: 100.0,
            section_index: 0,
        });
        render_tree.root.children.push(RenderNode::new(
            21,
            RenderNodeType::RawSvg(RawSvgNode {
                svg: "<g><circle cx=\"20\" cy=\"20\" r=\"8\" fill=\"#ff0000\"/></g>\n".to_string(),
            }),
            BoundingBox::new(0.0, 0.0, 40.0, 40.0),
        ));
        render_tree.root.children.push(RenderNode::new(
            22,
            RenderNodeType::Placeholder(PlaceholderNode {
                fill_color: 0x00F8F8F8,
                stroke_color: 0x00000000,
                label: "OLE".to_string(),
            }),
            BoundingBox::new(50.0, 10.0, 80.0, 50.0),
        ));

        let mut legacy = SvgRenderer::new();
        legacy.render_tree(&render_tree);

        let mut builder = LayerBuilder::new(RenderProfile::Screen);
        let layer_tree = builder.build(&render_tree);
        let mut layer = SvgLayerRenderer::new();
        layer.render_page(&layer_tree).unwrap();

        assert_eq!(layer.output(), legacy.output());
    }

    #[test]
    fn render_page_consumes_layer_output_options() {
        let mut render_tree = PageRenderTree::new(0, 80.0, 60.0);
        render_tree.root.node_type = RenderNodeType::Page(PageNode {
            page_index: 0,
            width: 80.0,
            height: 60.0,
            section_index: 0,
        });
        render_tree.root.children.push(RenderNode::new(
            31,
            RenderNodeType::TextRun(TextRunNode {
                text: "a b".to_string(),
                style: TextStyle {
                    font_size: 14.0,
                    ..Default::default()
                },
                char_shape_id: None,
                para_shape_id: None,
                section_index: None,
                para_index: None,
                char_start: None,
                cell_context: None,
                is_para_end: true,
                is_line_break_end: false,
                rotation: 0.0,
                is_vertical: false,
                char_overlap: None,
                border_fill_id: 0,
                baseline: 16.0,
                field_marker: Default::default(),
            }),
            BoundingBox::new(10.0, 15.0, 40.0, 20.0),
        ));

        let mut builder =
            LayerBuilder::new(RenderProfile::Screen).with_output_options(LayerOutputOptions {
                show_paragraph_marks: true,
                show_control_codes: true,
                ..Default::default()
            });
        let layer_tree = builder.build(&render_tree);
        let mut layer = SvgLayerRenderer::new();
        layer.render_page(&layer_tree).unwrap();

        let output = layer.output();
        assert!(
            output.contains("\u{2228}"),
            "layer output options should enable visible space marks:\n{output}"
        );
        assert!(
            output.contains("\u{21b5}"),
            "layer output options should enable paragraph end marks:\n{output}"
        );
    }
}

use crate::paint::layer_tree::{
    CacheHint, ClipKind, GroupKind, LayerNode, LayerNodeKind, PageLayerTree,
};
use crate::paint::paint_op::PaintOp;
use crate::paint::profile::RenderProfile;
use crate::renderer::render_tree::{PageRenderTree, RenderNode, RenderNodeType};

/// semantic render tree를 visual layer tree로 내린다.
pub struct LayerBuilder {
    profile: RenderProfile,
}

impl LayerBuilder {
    pub fn new(profile: RenderProfile) -> Self {
        Self { profile }
    }

    pub fn build(&mut self, tree: &PageRenderTree) -> PageLayerTree {
        let (page_width, page_height) = match &tree.root.node_type {
            RenderNodeType::Page(page) => (page.width, page.height),
            _ => (tree.root.bbox.width, tree.root.bbox.height),
        };

        let root = LayerNode::group(
            tree.root.bbox,
            Some(tree.root.id),
            self.build_children(&tree.root),
            self.cache_hint_for(&tree.root.node_type),
            GroupKind::Generic,
        );

        PageLayerTree::with_profile(page_width, page_height, root, self.profile)
    }

    fn build_children(&mut self, node: &RenderNode) -> Vec<LayerNode> {
        node.children
            .iter()
            .filter_map(|child| self.build_node(child))
            .collect()
    }

    fn build_node(&mut self, node: &RenderNode) -> Option<LayerNode> {
        if !node.visible {
            return None;
        }

        match &node.node_type {
            RenderNodeType::PageBackground(background) => Some(LayerNode::leaf(
                node.bbox,
                Some(node.id),
                vec![PaintOp::PageBackground {
                    bbox: node.bbox,
                    background: background.clone(),
                }],
            )),
            RenderNodeType::TextRun(run) => Some(LayerNode::leaf(
                node.bbox,
                Some(node.id),
                vec![PaintOp::TextRun {
                    bbox: node.bbox,
                    run: run.clone(),
                }],
            )),
            RenderNodeType::FootnoteMarker(marker) => Some(LayerNode::leaf(
                node.bbox,
                Some(node.id),
                vec![PaintOp::FootnoteMarker {
                    bbox: node.bbox,
                    marker: marker.clone(),
                }],
            )),
            RenderNodeType::Line(line) => Some(LayerNode::leaf(
                node.bbox,
                Some(node.id),
                vec![PaintOp::Line {
                    bbox: node.bbox,
                    line: line.clone(),
                }],
            )),
            RenderNodeType::Rectangle(rect) => Some(LayerNode::leaf(
                node.bbox,
                Some(node.id),
                vec![PaintOp::Rectangle {
                    bbox: node.bbox,
                    rect: rect.clone(),
                }],
            )),
            RenderNodeType::Ellipse(ellipse) => Some(LayerNode::leaf(
                node.bbox,
                Some(node.id),
                vec![PaintOp::Ellipse {
                    bbox: node.bbox,
                    ellipse: ellipse.clone(),
                }],
            )),
            RenderNodeType::Path(path) => Some(LayerNode::leaf(
                node.bbox,
                Some(node.id),
                vec![PaintOp::Path {
                    bbox: node.bbox,
                    path: path.clone(),
                }],
            )),
            RenderNodeType::Image(image) => Some(LayerNode::leaf(
                node.bbox,
                Some(node.id),
                vec![PaintOp::Image {
                    bbox: node.bbox,
                    image: image.clone(),
                }],
            )),
            RenderNodeType::Equation(equation) => Some(LayerNode::leaf(
                node.bbox,
                Some(node.id),
                vec![PaintOp::Equation {
                    bbox: node.bbox,
                    equation: equation.clone(),
                }],
            )),
            RenderNodeType::FormObject(form) => Some(LayerNode::leaf(
                node.bbox,
                Some(node.id),
                vec![PaintOp::FormObject {
                    bbox: node.bbox,
                    form: form.clone(),
                }],
            )),
            RenderNodeType::Placeholder(placeholder) => Some(LayerNode::leaf(
                node.bbox,
                Some(node.id),
                vec![PaintOp::Placeholder {
                    bbox: node.bbox,
                    placeholder: placeholder.clone(),
                }],
            )),
            RenderNodeType::RawSvg(raw) => Some(LayerNode::leaf(
                node.bbox,
                Some(node.id),
                vec![PaintOp::RawSvg {
                    bbox: node.bbox,
                    raw: raw.clone(),
                }],
            )),
            RenderNodeType::Body {
                clip_rect: Some(clip),
            } => {
                let child = LayerNode::group(
                    node.bbox,
                    Some(node.id),
                    self.build_children(node),
                    self.cache_hint_for(&node.node_type),
                    GroupKind::Body,
                );
                Some(LayerNode::clip_rect(
                    node.bbox,
                    Some(node.id),
                    *clip,
                    child,
                    ClipKind::Body,
                ))
            }
            RenderNodeType::TableCell(cell) if cell.clip => {
                let child = LayerNode::group(
                    node.bbox,
                    Some(node.id),
                    self.build_children(node),
                    self.cache_hint_for(&node.node_type),
                    GroupKind::TableCell(cell.clone()),
                );
                Some(LayerNode::clip_rect(
                    node.bbox,
                    Some(node.id),
                    node.bbox,
                    child,
                    ClipKind::TableCell,
                ))
            }
            _ => Some(LayerNode::group(
                node.bbox,
                Some(node.id),
                self.build_children(node),
                self.cache_hint_for(&node.node_type),
                self.group_kind_for(&node.node_type),
            )),
        }
    }

    fn cache_hint_for(&self, node_type: &RenderNodeType) -> CacheHint {
        match node_type {
            RenderNodeType::Header | RenderNodeType::Footer | RenderNodeType::MasterPage => {
                CacheHint::StaticSubtree
            }
            RenderNodeType::PageBackground(_)
                if matches!(self.profile, RenderProfile::FastPreview) =>
            {
                CacheHint::PreferRaster
            }
            _ => CacheHint::None,
        }
    }

    fn group_kind_for(&self, node_type: &RenderNodeType) -> GroupKind {
        match node_type {
            RenderNodeType::MasterPage => GroupKind::MasterPage,
            RenderNodeType::Header => GroupKind::Header,
            RenderNodeType::Footer => GroupKind::Footer,
            RenderNodeType::Body { .. } => GroupKind::Body,
            RenderNodeType::Column(index) => GroupKind::Column(*index),
            RenderNodeType::FootnoteArea => GroupKind::FootnoteArea,
            RenderNodeType::TextLine(line) => GroupKind::TextLine(line.clone()),
            RenderNodeType::Table(table) => GroupKind::Table(table.clone()),
            RenderNodeType::TableCell(cell) => GroupKind::TableCell(cell.clone()),
            RenderNodeType::TextBox => GroupKind::TextBox,
            RenderNodeType::Group(group) => GroupKind::Group(group.clone()),
            _ => GroupKind::Generic,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::control::FormType;
    use crate::renderer::equation::layout::{LayoutBox, LayoutKind};
    use crate::renderer::render_tree::{
        BoundingBox, EllipseNode, EquationNode, FootnoteMarkerNode, FormObjectNode, ImageNode,
        LineNode, PageBackgroundNode, PageNode, PathNode, PlaceholderNode, RawSvgNode,
        RectangleNode, RenderNode, RenderNodeType, TableCellNode, TextRunNode,
    };
    use crate::renderer::{LineStyle, PathCommand, ShapeStyle, TextStyle};

    #[test]
    fn builds_body_clip_layer() {
        let mut tree = PageRenderTree::new(0, 800.0, 600.0);
        tree.root.node_type = RenderNodeType::Page(PageNode {
            page_index: 0,
            width: 800.0,
            height: 600.0,
            section_index: 0,
        });
        let body = RenderNode::new(
            1,
            RenderNodeType::Body {
                clip_rect: Some(BoundingBox::new(10.0, 20.0, 300.0, 400.0)),
            },
            BoundingBox::new(10.0, 20.0, 300.0, 400.0),
        );
        tree.root.children.push(body);

        let mut builder = LayerBuilder::new(RenderProfile::Screen);
        let layer_tree = builder.build(&tree);

        assert_eq!(layer_tree.page_width, 800.0);
        match &layer_tree.root.kind {
            LayerNodeKind::Group { children, .. } => {
                assert_eq!(children.len(), 1);
                match &children[0].kind {
                    LayerNodeKind::ClipRect {
                        clip, clip_kind, ..
                    } => {
                        assert_eq!(clip.x, 10.0);
                        assert_eq!(*clip_kind, ClipKind::Body);
                    }
                    other => panic!("expected clip rect, got {other:?}"),
                }
            }
            other => panic!("expected root group, got {other:?}"),
        }
    }

    #[test]
    fn preserves_leaf_payloads() {
        let mut tree = PageRenderTree::new(0, 800.0, 600.0);
        tree.root.children.push(RenderNode::new(
            1,
            RenderNodeType::PageBackground(PageBackgroundNode {
                background_color: Some(0x00FFFFFF),
                border_color: None,
                border_width: 0.0,
                gradient: None,
                image: None,
            }),
            BoundingBox::new(0.0, 0.0, 800.0, 600.0),
        ));
        tree.root.children.push(RenderNode::new(
            2,
            RenderNodeType::TableCell(TableCellNode {
                col: 0,
                row: 0,
                col_span: 1,
                row_span: 1,
                border_fill_id: 0,
                text_direction: 0,
                clip: true,
                model_cell_index: None,
            }),
            BoundingBox::new(100.0, 200.0, 150.0, 80.0),
        ));

        let mut builder = LayerBuilder::new(RenderProfile::Screen);
        let layer_tree = builder.build(&tree);

        match &layer_tree.root.kind {
            LayerNodeKind::Group { children, .. } => {
                assert_eq!(children.len(), 2);
                match &children[0].kind {
                    LayerNodeKind::Leaf { ops } => {
                        assert!(matches!(ops[0], PaintOp::PageBackground { .. }));
                    }
                    other => panic!("expected leaf, got {other:?}"),
                }
                match &children[1].kind {
                    LayerNodeKind::ClipRect { clip_kind, .. } => {
                        assert_eq!(*clip_kind, ClipKind::TableCell);
                    }
                    other => panic!("expected clip rect, got {other:?}"),
                }
            }
            other => panic!("expected root group, got {other:?}"),
        }
    }

    #[test]
    fn lowers_all_leaf_variants_to_explicit_paint_ops() {
        let cases: Vec<(RenderNodeType, fn(&PaintOp) -> bool, &'static str)> = vec![
            (
                RenderNodeType::PageBackground(PageBackgroundNode {
                    background_color: Some(0x00FFFFFF),
                    border_color: None,
                    border_width: 0.0,
                    gradient: None,
                    image: None,
                }),
                |op| matches!(op, PaintOp::PageBackground { .. }),
                "PageBackground",
            ),
            (
                RenderNodeType::TextRun(text_run("leaf")),
                |op| matches!(op, PaintOp::TextRun { .. }),
                "TextRun",
            ),
            (
                RenderNodeType::FootnoteMarker(FootnoteMarkerNode {
                    number: 1,
                    text: "1)".to_string(),
                    base_font_size: 12.0,
                    font_family: "serif".to_string(),
                    color: 0x00000000,
                    section_index: 0,
                    para_index: 0,
                    control_index: 0,
                }),
                |op| matches!(op, PaintOp::FootnoteMarker { .. }),
                "FootnoteMarker",
            ),
            (
                RenderNodeType::Line(LineNode::new(0.0, 0.0, 12.0, 0.0, LineStyle::default())),
                |op| matches!(op, PaintOp::Line { .. }),
                "Line",
            ),
            (
                RenderNodeType::Rectangle(RectangleNode::new(0.0, ShapeStyle::default(), None)),
                |op| matches!(op, PaintOp::Rectangle { .. }),
                "Rectangle",
            ),
            (
                RenderNodeType::Ellipse(EllipseNode::new(ShapeStyle::default(), None)),
                |op| matches!(op, PaintOp::Ellipse { .. }),
                "Ellipse",
            ),
            (
                RenderNodeType::Path(PathNode::new(
                    vec![PathCommand::MoveTo(0.0, 0.0), PathCommand::LineTo(8.0, 4.0)],
                    ShapeStyle::default(),
                    None,
                )),
                |op| matches!(op, PaintOp::Path { .. }),
                "Path",
            ),
            (
                RenderNodeType::Image(ImageNode::new(1, Some(vec![0x89, b'P', b'N', b'G']))),
                |op| matches!(op, PaintOp::Image { .. }),
                "Image",
            ),
            (
                RenderNodeType::Equation(equation_node()),
                |op| matches!(op, PaintOp::Equation { .. }),
                "Equation",
            ),
            (
                RenderNodeType::FormObject(form_object_node()),
                |op| matches!(op, PaintOp::FormObject { .. }),
                "FormObject",
            ),
            (
                RenderNodeType::Placeholder(PlaceholderNode {
                    fill_color: 0x00F0F0F0,
                    stroke_color: 0x00000000,
                    label: "OLE".to_string(),
                }),
                |op| matches!(op, PaintOp::Placeholder { .. }),
                "Placeholder",
            ),
            (
                RenderNodeType::RawSvg(RawSvgNode {
                    svg: "<g><path d=\"M0 0L1 1\"/></g>".to_string(),
                }),
                |op| matches!(op, PaintOp::RawSvg { .. }),
                "RawSvg",
            ),
        ];

        for (idx, (node_type, is_expected_op, label)) in cases.into_iter().enumerate() {
            let mut tree = PageRenderTree::new(0, 100.0, 100.0);
            tree.root.children.push(RenderNode::new(
                100 + idx as u32,
                node_type,
                BoundingBox::new(1.0, 2.0, 30.0, 20.0),
            ));

            let mut builder = LayerBuilder::new(RenderProfile::Screen);
            let layer_tree = builder.build(&tree);

            let LayerNodeKind::Group { children, .. } = &layer_tree.root.kind else {
                panic!("expected root group for {label}");
            };
            let LayerNodeKind::Leaf { ops } = &children[0].kind else {
                panic!("expected leaf for {label}, got {:?}", children[0].kind);
            };
            assert_eq!(ops.len(), 1, "{label} should lower to one paint op");
            assert!(is_expected_op(&ops[0]), "{label} lowered to {:?}", ops[0]);
        }
    }

    fn text_run(text: &str) -> TextRunNode {
        TextRunNode {
            text: text.to_string(),
            style: TextStyle::default(),
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
            baseline: 12.0,
            field_marker: Default::default(),
        }
    }

    fn equation_node() -> EquationNode {
        EquationNode {
            svg_content: "<text>x</text>".to_string(),
            layout_box: LayoutBox {
                x: 0.0,
                y: 0.0,
                width: 8.0,
                height: 12.0,
                baseline: 10.0,
                kind: LayoutKind::Text("x".to_string()),
            },
            color_str: "#000000".to_string(),
            color: 0x00000000,
            font_size: 12.0,
            section_index: None,
            para_index: None,
            control_index: None,
            cell_index: None,
            cell_para_index: None,
        }
    }

    fn form_object_node() -> FormObjectNode {
        FormObjectNode {
            form_type: FormType::PushButton,
            caption: "OK".to_string(),
            text: String::new(),
            fore_color: "#000000".to_string(),
            back_color: "#ffffff".to_string(),
            value: 0,
            enabled: true,
            section_index: 0,
            para_index: 0,
            control_index: 0,
            name: "button".to_string(),
            cell_location: None,
        }
    }
}

use crate::paint::layer_tree::{CacheHint, ClipKind, LayerNode, LayerNodeKind, PageLayerTree};
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
        );

        PageLayerTree::new(page_width, page_height, root)
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
            RenderNodeType::Body {
                clip_rect: Some(clip),
            } => {
                let child = LayerNode::group(
                    node.bbox,
                    Some(node.id),
                    self.build_children(node),
                    self.cache_hint_for(&node.node_type),
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::renderer::render_tree::{
        BoundingBox, PageBackgroundNode, PageNode, RenderNode, RenderNodeType, TableCellNode,
    };

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
}

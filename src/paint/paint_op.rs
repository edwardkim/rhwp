use crate::renderer::render_tree::{
    BoundingBox, EllipseNode, EquationNode, FootnoteMarkerNode, FormObjectNode, ImageNode,
    LineNode, PageBackgroundNode, PathNode, PlaceholderNode, RawSvgNode, RectangleNode,
    TextRunNode,
};

/// backend가 재생하는 leaf paint operation.
///
/// 1차 전환에서는 기존 leaf payload를 최대한 그대로 유지해
/// semantic container 해석과 leaf draw payload 분리부터 달성한다.
#[derive(Debug, Clone)]
pub enum PaintOp {
    PageBackground {
        bbox: BoundingBox,
        background: PageBackgroundNode,
    },
    TextRun {
        bbox: BoundingBox,
        run: TextRunNode,
    },
    FootnoteMarker {
        bbox: BoundingBox,
        marker: FootnoteMarkerNode,
    },
    Line {
        bbox: BoundingBox,
        line: LineNode,
    },
    Rectangle {
        bbox: BoundingBox,
        rect: RectangleNode,
    },
    Ellipse {
        bbox: BoundingBox,
        ellipse: EllipseNode,
    },
    Path {
        bbox: BoundingBox,
        path: PathNode,
    },
    Image {
        bbox: BoundingBox,
        image: ImageNode,
    },
    Equation {
        bbox: BoundingBox,
        equation: EquationNode,
    },
    FormObject {
        bbox: BoundingBox,
        form: FormObjectNode,
    },
    Placeholder {
        bbox: BoundingBox,
        placeholder: PlaceholderNode,
    },
    RawSvg {
        bbox: BoundingBox,
        raw: RawSvgNode,
    },
}

impl PaintOp {
    pub fn bounds(&self) -> BoundingBox {
        match self {
            PaintOp::PageBackground { bbox, .. }
            | PaintOp::TextRun { bbox, .. }
            | PaintOp::FootnoteMarker { bbox, .. }
            | PaintOp::Line { bbox, .. }
            | PaintOp::Rectangle { bbox, .. }
            | PaintOp::Ellipse { bbox, .. }
            | PaintOp::Path { bbox, .. }
            | PaintOp::Image { bbox, .. }
            | PaintOp::Equation { bbox, .. }
            | PaintOp::FormObject { bbox, .. }
            | PaintOp::Placeholder { bbox, .. }
            | PaintOp::RawSvg { bbox, .. } => *bbox,
        }
    }
}

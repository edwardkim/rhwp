use crate::renderer::render_tree::{
    BoundingBox, EllipseNode, EquationNode, FootnoteMarkerNode, FormObjectNode, ImageNode,
    LineNode, PageBackgroundNode, PathNode, PlaceholderNode, RawSvgNode, RectangleNode,
    TextRunNode,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextDecorationKind {
    Underline,
    Strikethrough,
    EmphasisDot,
}

impl TextDecorationKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Underline => "underline",
            Self::Strikethrough => "strikethrough",
            Self::EmphasisDot => "emphasisDot",
        }
    }
}

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
    /// HWP 글자겹침의 명시 visual op.
    ///
    /// 전환기에는 paired TextRun 안에도 legacy mirror payload를 남긴다.
    /// 새 backend는 이 op를 선택하고 TextRun mirror를 건너뛸 수 있다.
    CharOverlap {
        bbox: BoundingBox,
        run: TextRunNode,
    },
    /// 문단 끝/줄 바꿈/필드 마커처럼 source text와 visual projection이 다른 표식.
    TextControlMark {
        bbox: BoundingBox,
        run: TextRunNode,
    },
    /// 탭 리더 visual geometry.
    TabLeader {
        bbox: BoundingBox,
        run: TextRunNode,
    },
    /// 밑줄/취소선/강조점 visual geometry.
    TextDecoration {
        bbox: BoundingBox,
        run: TextRunNode,
        kind: TextDecorationKind,
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
            | PaintOp::CharOverlap { bbox, .. }
            | PaintOp::TextControlMark { bbox, .. }
            | PaintOp::TabLeader { bbox, .. }
            | PaintOp::TextDecoration { bbox, .. }
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

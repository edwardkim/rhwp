use crate::model::style::UnderlineType;
use crate::model::ColorRef;
use crate::paint::font::{GlyphRunReplayEligibility, ShapeKey, TextDirection, WritingMode};
use crate::paint::layer_tree::{TextSourceRange, TextSourceSpan};
use crate::renderer::render_tree::{
    BoundingBox, EllipseNode, EquationNode, FootnoteMarkerNode, FormObjectNode, ImageNode,
    LineNode, PageBackgroundNode, PathNode, PlaceholderNode, RawSvgNode, RectangleNode,
    TextRunNode,
};
use crate::renderer::TextStyle;

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
    GlyphRun {
        bbox: BoundingBox,
        run: Box<LayerGlyphRunPaint>,
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
            | PaintOp::GlyphRun { bbox, .. }
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

#[derive(Debug, Clone)]
pub struct LayerGlyphRunPaint {
    pub source: TextSourceSpan,
    pub variant: PaintVariantMeta,
    pub paint_style: PaintTextStyle,
    pub shape_key: ShapeKey,
    pub placement: TextRunPlacement,
    pub glyph_ids: Vec<u32>,
    pub positions: Vec<LayerPoint>,
    pub advances: Option<Vec<LayerVector>>,
    pub clusters: Vec<GlyphCluster>,
    pub direction: TextDirection,
    pub bidi_level: Option<u8>,
    pub writing_mode: WritingMode,
    pub orientation: GlyphRunOrientation,
    pub glyph_transforms: Option<Vec<GlyphTransform>>,
    pub diagnostics: GlyphRunDiagnostics,
}

/// Variant grouping metadata for TextRun/GlyphRun alternatives.
///
/// Consumers select one `variant_id` per `equivalence_group` and paint every
/// part belonging to that variant. A `TextRun` fallback remains required.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaintVariantMeta {
    pub equivalence_group: String,
    pub variant_id: String,
    pub variant_kind: TextVariantKind,
    pub part_index: u32,
    pub part_count: u32,
    pub is_default_fallback: bool,
    pub requires: Vec<String>,
    pub quality: Option<TextVariantQuality>,
    pub anchor_op_id: Option<String>,
    pub local_paint_order: Option<u32>,
}

impl PaintVariantMeta {
    pub fn text_run_default(equivalence_group: impl Into<String>) -> Self {
        Self {
            equivalence_group: equivalence_group.into(),
            variant_id: "textRun".to_string(),
            variant_kind: TextVariantKind::TextRun,
            part_index: 0,
            part_count: 1,
            is_default_fallback: true,
            requires: Vec::new(),
            quality: None,
            anchor_op_id: None,
            local_paint_order: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextVariantKind {
    TextRun,
    GlyphRun,
}

impl TextVariantKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::TextRun => "textRun",
            Self::GlyphRun => "glyphRun",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextVariantQuality {
    Exact,
    PositionAdjusted,
    Approximate,
    DiagnosticOnly,
    Omitted,
}

impl TextVariantQuality {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Exact => "exact",
            Self::PositionAdjusted => "positionAdjusted",
            Self::Approximate => "approximate",
            Self::DiagnosticOnly => "diagnosticOnly",
            Self::Omitted => "omitted",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LayerPoint {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LayerVector {
    pub dx: f64,
    pub dy: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LayerAffineTransform {
    pub a: f64,
    pub b: f64,
    pub c: f64,
    pub d: f64,
    pub e: f64,
    pub f: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TextRunPlacement {
    pub run_to_page: LayerAffineTransform,
    pub baseline_y: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GlyphRunOrientation {
    Horizontal,
    VerticalUpright,
    VerticalSideways,
    MixedPerGlyph,
}

impl GlyphRunOrientation {
    pub fn from_text_run(run: &TextRunNode) -> Self {
        if !run.is_vertical {
            Self::Horizontal
        } else if run.rotation.abs() > f64::EPSILON {
            Self::VerticalSideways
        } else {
            Self::VerticalUpright
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Horizontal => "horizontal",
            Self::VerticalUpright => "vertical-upright",
            Self::VerticalSideways => "vertical-sideways",
            Self::MixedPerGlyph => "mixedPerGlyph",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GlyphTransform {
    pub xx: f32,
    pub xy: f32,
    pub yx: f32,
    pub yy: f32,
    pub tx: f32,
    pub ty: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GlyphRange {
    pub start: u32,
    pub end: u32,
}

impl GlyphRange {
    pub fn new(start: u32, end: u32) -> Self {
        Self { start, end }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GlyphClusterFlag {
    Ligature,
    FallbackBoundary,
}

impl GlyphClusterFlag {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ligature => "ligature",
            Self::FallbackBoundary => "fallbackBoundary",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlyphCluster {
    pub source_range_utf8: TextSourceRange,
    pub source_range_utf16: Option<TextSourceRange>,
    pub text_range_utf8: Option<TextSourceRange>,
    pub glyph_range: GlyphRange,
    pub flags: Vec<GlyphClusterFlag>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlyphRunDiagnostics {
    pub quality: TextVariantQuality,
    pub replay_eligibility: GlyphRunReplayEligibility,
    pub strict_visual_eligible: bool,
    pub max_origin_delta_px: f64,
    pub max_advance_delta_px: f64,
    pub max_residual_after_adjustment_px: f64,
    pub cluster_mismatch_count: u32,
    pub missing_glyph_count: u32,
    pub used_fallback_font_count: u32,
    pub reason: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PaintTextStyle {
    pub font_family: String,
    pub font_size: f64,
    pub color: ColorRef,
    pub bold: bool,
    pub italic: bool,
    pub underline: UnderlineType,
    pub strikethrough: bool,
    pub ratio: f64,
    pub tab_leaders: Vec<crate::renderer::TabLeaderInfo>,
    pub outline_type: u8,
    pub shadow_type: u8,
    pub shadow_color: ColorRef,
    pub shadow_offset_x: f64,
    pub shadow_offset_y: f64,
    pub emboss: bool,
    pub engrave: bool,
    pub superscript: bool,
    pub subscript: bool,
    pub emphasis_dot: u8,
    pub underline_shape: u8,
    pub strike_shape: u8,
    pub underline_color: ColorRef,
    pub strike_color: ColorRef,
    pub shade_color: ColorRef,
}

impl From<&TextStyle> for PaintTextStyle {
    fn from(style: &TextStyle) -> Self {
        Self {
            font_family: style.font_family.clone(),
            font_size: style.font_size,
            color: style.color,
            bold: style.bold,
            italic: style.italic,
            underline: style.underline,
            strikethrough: style.strikethrough,
            ratio: style.ratio,
            tab_leaders: style.tab_leaders.clone(),
            outline_type: style.outline_type,
            shadow_type: style.shadow_type,
            shadow_color: style.shadow_color,
            shadow_offset_x: style.shadow_offset_x,
            shadow_offset_y: style.shadow_offset_y,
            emboss: style.emboss,
            engrave: style.engrave,
            superscript: style.superscript,
            subscript: style.subscript,
            emphasis_dot: style.emphasis_dot,
            underline_shape: style.underline_shape,
            strike_shape: style.strike_shape,
            underline_color: style.underline_color,
            strike_color: style.strike_color,
            shade_color: style.shade_color,
        }
    }
}

impl PaintTextStyle {
    /// Returns whether a backend may replay this text as a simple fill-only
    /// positioned glyph run without losing HWP text effects.
    pub fn is_fill_only_glyph_replay(&self) -> bool {
        let ratio = if self.ratio > 0.0 { self.ratio } else { 1.0 };
        (ratio - 1.0).abs() <= 0.001
            && self.tab_leaders.is_empty()
            && self.underline == UnderlineType::None
            && !self.strikethrough
            && self.outline_type == 0
            && self.shadow_type == 0
            && !self.emboss
            && !self.engrave
            && !self.superscript
            && !self.subscript
            && self.emphasis_dot == 0
            && (self.shade_color & 0x00FF_FFFF) == 0x00FF_FFFF
    }
}

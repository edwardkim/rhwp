use crate::model::style::UnderlineType;
use crate::model::ColorRef;
use crate::paint::font::{GlyphRunReplayEligibility, ShapeKey, TextDirection, WritingMode};
use crate::paint::layer_tree::{TextSourceRange, TextSourceSpan};
use crate::paint::resources::{ImageResourceId, ResourceArena, SvgResourceId};
use crate::renderer::render_tree::{
    BoundingBox, EllipseNode, EquationNode, FootnoteMarkerNode, FormObjectNode, ImageNode,
    LineNode, PageBackgroundNode, PathNode, PlaceholderNode, RawSvgNode, RectangleNode,
    TextRunNode,
};
use crate::renderer::{PathCommand, TextStyle};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolvedImageKind {
    FormatConverted,
    BakedWatermark,
}

#[derive(Debug, Clone)]
pub struct ResolvedImagePayload {
    pub data: Vec<u8>,
    pub mime: &'static str,
    pub kind: ResolvedImageKind,
    pub suppress_effects: bool,
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
    GlyphOutline {
        bbox: BoundingBox,
        outline: Box<LayerGlyphOutlinePaint>,
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
        resolved: Option<Box<ResolvedImagePayload>>,
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
            | PaintOp::GlyphOutline { bbox, .. }
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

/// Strict-visual text alternative that carries producer-resolved glyph paths.
///
/// A GlyphOutline is still a text variant, not a generic Path. Consumers must
/// select it through the same equivalence group as the TextRun fallback and
/// reject it when the backend cannot preserve the declared payload.
#[derive(Debug, Clone)]
pub struct LayerGlyphOutlinePaint {
    pub source: TextSourceSpan,
    pub variant: PaintVariantMeta,
    pub payload_kind: GlyphOutlinePayloadKind,
    pub color_layers: Option<ColorLayersPayload>,
    pub bitmap_glyph: Option<BitmapGlyphPayload>,
    pub svg_glyph: Option<SvgGlyphPayload>,
    pub paint_style: PaintTextStyle,
    pub placement: TextRunPlacement,
    pub paths: Vec<LayerGlyphOutlinePath>,
    pub stroke: Option<GlyphOutlineStrokeStyle>,
    pub diagnostics: GlyphRunDiagnostics,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GlyphOutlinePayloadKind {
    MonochromeFill,
    MonochromeFillStroke,
    ColorLayers,
    BitmapGlyph,
    SvgGlyph,
}

impl GlyphOutlinePayloadKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::MonochromeFill => "monochromeFill",
            Self::MonochromeFillStroke => "monochromeFillStroke",
            Self::ColorLayers => "colorLayers",
            Self::BitmapGlyph => "bitmapGlyph",
            Self::SvgGlyph => "svgGlyph",
        }
    }
}

impl LayerGlyphOutlinePaint {
    pub fn has_exclusive_payload_family(&self) -> bool {
        let has_stroke = self.stroke.is_some();
        let has_color_layers = self.color_layers.is_some();
        let has_bitmap_glyph = self.bitmap_glyph.is_some();
        let has_svg_glyph = self.svg_glyph.is_some();
        match self.payload_kind {
            GlyphOutlinePayloadKind::MonochromeFill => {
                !has_stroke && !has_color_layers && !has_bitmap_glyph && !has_svg_glyph
            }
            GlyphOutlinePayloadKind::MonochromeFillStroke => {
                has_stroke && !has_color_layers && !has_bitmap_glyph && !has_svg_glyph
            }
            GlyphOutlinePayloadKind::ColorLayers => {
                !has_stroke && has_color_layers && !has_bitmap_glyph && !has_svg_glyph
            }
            GlyphOutlinePayloadKind::BitmapGlyph => {
                !has_stroke && !has_color_layers && has_bitmap_glyph && !has_svg_glyph
            }
            GlyphOutlinePayloadKind::SvgGlyph => {
                !has_stroke && !has_color_layers && !has_bitmap_glyph && has_svg_glyph
            }
        }
    }

    /// Stable, export-local identity for payload resources that sit behind a
    /// GlyphOutline sidecar. The key starts with replay decision metadata and
    /// may append an interned resource digest when bytes are available, so
    /// color/bitmap/SVG payload families do not share a cache slot merely
    /// because a producer reused the same numeric resource id.
    pub fn payload_resource_key(&self) -> Option<String> {
        self.payload_resource_key_with_resources(None)
    }

    pub fn payload_resource_key_with_resources(
        &self,
        resources: Option<&ResourceArena>,
    ) -> Option<String> {
        if !self.has_payload_resource_key() {
            return None;
        }
        match self.payload_kind {
            GlyphOutlinePayloadKind::MonochromeFill
            | GlyphOutlinePayloadKind::MonochromeFillStroke => None,
            GlyphOutlinePayloadKind::ColorLayers => self
                .color_layers
                .as_ref()
                .map(color_layers_payload_resource_key),
            GlyphOutlinePayloadKind::BitmapGlyph => self
                .bitmap_glyph
                .as_ref()
                .map(|payload| bitmap_glyph_payload_resource_key(payload, resources)),
            GlyphOutlinePayloadKind::SvgGlyph => self
                .svg_glyph
                .as_ref()
                .map(|payload| svg_glyph_payload_resource_key(payload, resources)),
        }
    }

    pub fn has_payload_resource_key(&self) -> bool {
        match self.payload_kind {
            GlyphOutlinePayloadKind::MonochromeFill
            | GlyphOutlinePayloadKind::MonochromeFillStroke => false,
            GlyphOutlinePayloadKind::ColorLayers => {
                self.color_layers.as_ref().is_some_and(|payload| {
                    payload.has_colrv0_resolved_layer_contract()
                        || payload.has_colrv1_supported_graph_contract()
                })
            }
            GlyphOutlinePayloadKind::BitmapGlyph => self
                .bitmap_glyph
                .as_ref()
                .is_some_and(BitmapGlyphPayload::has_strict_visual_contract),
            GlyphOutlinePayloadKind::SvgGlyph => self
                .svg_glyph
                .as_ref()
                .is_some_and(SvgGlyphPayload::has_static_sanitized_contract),
        }
    }
}

fn color_layers_payload_resource_key(payload: &ColorLayersPayload) -> String {
    let mut key = format!(
        "glyphPayload:colorLayers:format:{}:source:{}:palette:{}:range:{}:glyphRange:{}",
        payload.color_format.as_str(),
        font_color_glyph_ref_key(payload.source_font_ref.as_ref()),
        palette_ref_key(payload.palette_ref.as_ref()),
        optional_text_range_key(payload.source_range_utf8),
        optional_glyph_range_key(payload.glyph_range),
    );
    if let Some(graph) = &payload.paint_graph {
        key.push_str(":graph:");
        key.push_str(&color_paint_graph_key(graph));
    } else if !payload.layers.is_empty() {
        key.push_str(":layers:");
        for (idx, layer) in payload.layers.iter().enumerate() {
            if idx > 0 {
                key.push('|');
            }
            key.push_str(&format!(
                "{}:{}:{}:{}:{}:{}",
                layer
                    .layer_index
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "-".to_string()),
                layer
                    .glyph_id
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "-".to_string()),
                optional_glyph_range_key(layer.glyph_range),
                optional_text_range_key(layer.source_range_utf8),
                font_color_glyph_ref_key(layer.source_font_ref.as_ref()),
                layer
                    .palette_index
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "-".to_string()),
            ));
        }
    }
    key
}

fn color_paint_graph_key(graph: &ColorPaintGraphPayload) -> String {
    let mut key = format!("root:{}:nodes:", graph.root_node_id);
    for (idx, node) in graph.nodes.iter().enumerate() {
        if idx > 0 {
            key.push('|');
        }
        key.push_str(&format!(
            "{}:{}:{}:{}",
            node.node_id,
            node.kind.as_str(),
            optional_glyph_range_key(node.glyph_range),
            font_color_glyph_ref_key(node.source_font_ref.as_ref()),
        ));
    }
    key
}

fn bitmap_glyph_payload_resource_key(
    payload: &BitmapGlyphPayload,
    resources: Option<&ResourceArena>,
) -> String {
    let mut key = format!(
        "glyphPayload:bitmapGlyph:imageRef:{}:range:{}:glyphRange:{}:placement:{}:alphaPremultiplied:{}:scaling:{}:filtering:{}:transform:{}",
        payload.image_ref.0,
        text_range_key(payload.source_range_utf8),
        glyph_range_key(payload.glyph_range),
        bbox_key(payload.placement),
        payload.alpha_premultiplied,
        payload.scaling_policy.as_str(),
        payload.filtering.as_str(),
        optional_affine_key(payload.transform_to_run),
    );
    if let Some(resource_key) =
        resources.and_then(|resources| resources.image_resource_key(payload.image_ref))
    {
        key.push_str(":resource:");
        key.push_str(resource_key);
    }
    key
}

fn svg_glyph_payload_resource_key(
    payload: &SvgGlyphPayload,
    resources: Option<&ResourceArena>,
) -> String {
    let mut key = format!(
        "glyphPayload:svgGlyph:svgRef:{}:range:{}:glyphRange:{}:viewBox:{}:intrinsicSize:{}:staticSanitized:{}:script:{}:animation:{}:external:{}:interactive:{}:transform:{}",
        payload.svg_ref.0,
        text_range_key(payload.source_range_utf8),
        glyph_range_key(payload.glyph_range),
        bbox_key(payload.view_box),
        payload
            .intrinsic_size
            .map(|size| format!("{:.6},{:.6}", size.dx, size.dy))
            .unwrap_or_else(|| "-".to_string()),
        payload.static_sanitized,
        payload.script_allowed,
        payload.animation_allowed,
        payload.external_resources_allowed,
        payload.interactivity_allowed,
        optional_affine_key(payload.transform_to_run),
    );
    if let Some(resource_key) =
        resources.and_then(|resources| resources.svg_resource_key(payload.svg_ref))
    {
        key.push_str(":resource:");
        key.push_str(resource_key);
    }
    key
}

fn font_color_glyph_ref_key(value: Option<&FontColorGlyphRef>) -> String {
    match value {
        Some(value) => format!(
            "face:{}:glyph:{}:palette:{}:format:{}",
            value.face_key.as_deref().unwrap_or("-"),
            value
                .glyph_id
                .map(|glyph_id| glyph_id.to_string())
                .unwrap_or_else(|| "-".to_string()),
            value
                .palette_index
                .map(|palette_index| palette_index.to_string())
                .unwrap_or_else(|| "-".to_string()),
            value
                .color_format
                .map(ColorGlyphFormat::as_str)
                .unwrap_or("-"),
        ),
        None => "-".to_string(),
    }
}

fn palette_ref_key(value: Option<&PaletteRef>) -> String {
    match value {
        Some(value) => format!(
            "id:{}:index:{}:digest:{}",
            value.id.as_deref().unwrap_or("-"),
            value
                .index
                .map(|index| index.to_string())
                .unwrap_or_else(|| "-".to_string()),
            value.cpal_digest.as_deref().unwrap_or("-"),
        ),
        None => "-".to_string(),
    }
}

fn optional_text_range_key(range: Option<TextSourceRange>) -> String {
    range.map(text_range_key).unwrap_or_else(|| "-".to_string())
}

fn text_range_key(range: TextSourceRange) -> String {
    format!("{}..{}", range.start, range.end)
}

fn optional_glyph_range_key(range: Option<GlyphRange>) -> String {
    range
        .map(glyph_range_key)
        .unwrap_or_else(|| "-".to_string())
}

fn glyph_range_key(range: GlyphRange) -> String {
    format!("{}..{}", range.start, range.end)
}

fn bbox_key(bbox: BoundingBox) -> String {
    format!(
        "{:.6},{:.6},{:.6},{:.6}",
        bbox.x, bbox.y, bbox.width, bbox.height
    )
}

fn optional_affine_key(transform: Option<LayerAffineTransform>) -> String {
    transform.map(affine_key).unwrap_or_else(|| "-".to_string())
}

fn affine_key(transform: LayerAffineTransform) -> String {
    format!(
        "{:.6},{:.6},{:.6},{:.6},{:.6},{:.6}",
        transform.a, transform.b, transform.c, transform.d, transform.e, transform.f
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorGlyphFormat {
    ColrV0,
    ColrV1,
    Other,
}

impl ColorGlyphFormat {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ColrV0 => "colrV0",
            Self::ColrV1 => "colrV1",
            Self::Other => "other",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FontColorGlyphRef {
    pub face_key: Option<String>,
    pub glyph_id: Option<u32>,
    pub palette_index: Option<u16>,
    pub color_format: Option<ColorGlyphFormat>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PaletteRef {
    pub id: Option<String>,
    pub index: Option<u16>,
    pub cpal_digest: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedColor {
    pub color_space: Option<String>,
    pub rgba: [f32; 4],
}

#[derive(Debug, Clone)]
pub struct ColorLayerNode {
    pub layer_index: Option<u32>,
    pub glyph_id: Option<u32>,
    pub glyph_range: Option<GlyphRange>,
    pub source_range_utf8: Option<TextSourceRange>,
    pub source_font_ref: Option<FontColorGlyphRef>,
    pub commands: Option<Vec<PathCommand>>,
    pub fill: Option<ResolvedColor>,
    pub fill_rule: Option<GlyphOutlineFillRule>,
    pub palette_index: Option<u16>,
    pub color: Option<ColorRef>,
    pub opacity: Option<f64>,
    pub transform_to_run: Option<LayerAffineTransform>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorPaintGraphNodeKind {
    SolidPath,
    LinearGradientPath,
    RadialGradientPath,
    SweepGradientPath,
    Transform,
    Composite,
}

impl ColorPaintGraphNodeKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::SolidPath => "solidPath",
            Self::LinearGradientPath => "linearGradientPath",
            Self::RadialGradientPath => "radialGradientPath",
            Self::SweepGradientPath => "sweepGradientPath",
            Self::Transform => "transform",
            Self::Composite => "composite",
        }
    }

    pub fn is_colrv1_stage1_supported(self) -> bool {
        matches!(
            self,
            Self::SolidPath
                | Self::LinearGradientPath
                | Self::RadialGradientPath
                | Self::SweepGradientPath
                | Self::Transform
        )
    }
}

#[derive(Debug, Clone)]
pub struct ColorPaintSolidPathNode {
    pub commands: Vec<PathCommand>,
    pub fill: ResolvedColor,
    pub fill_rule: GlyphOutlineFillRule,
    pub source_glyph_id: Option<u32>,
    pub palette_index: Option<u16>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ColorGradientStop {
    pub offset: f64,
    pub color: ResolvedColor,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ColorLinearGradient {
    pub x0: f64,
    pub y0: f64,
    pub x1: f64,
    pub y1: f64,
    pub stops: Vec<ColorGradientStop>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ColorRadialGradient {
    pub cx: f64,
    pub cy: f64,
    pub radius: f64,
    pub stops: Vec<ColorGradientStop>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ColorSweepGradient {
    pub cx: f64,
    pub cy: f64,
    pub start_angle_degrees: f64,
    pub end_angle_degrees: f64,
    pub stops: Vec<ColorGradientStop>,
}

#[derive(Debug, Clone)]
pub struct ColorPaintLinearGradientPathNode {
    pub commands: Vec<PathCommand>,
    pub gradient: ColorLinearGradient,
    pub fill_rule: GlyphOutlineFillRule,
    pub source_glyph_id: Option<u32>,
    pub palette_index: Option<u16>,
}

#[derive(Debug, Clone)]
pub struct ColorPaintRadialGradientPathNode {
    pub commands: Vec<PathCommand>,
    pub gradient: ColorRadialGradient,
    pub fill_rule: GlyphOutlineFillRule,
    pub source_glyph_id: Option<u32>,
    pub palette_index: Option<u16>,
}

#[derive(Debug, Clone)]
pub struct ColorPaintSweepGradientPathNode {
    pub commands: Vec<PathCommand>,
    pub gradient: ColorSweepGradient,
    pub fill_rule: GlyphOutlineFillRule,
    pub source_glyph_id: Option<u32>,
    pub palette_index: Option<u16>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ColorPaintTransformNode {
    pub child_node_id: u32,
    pub transform: LayerAffineTransform,
}

#[derive(Debug, Clone)]
pub struct ColorPaintGraphNode {
    pub node_id: u32,
    pub kind: ColorPaintGraphNodeKind,
    pub solid_path: Option<ColorPaintSolidPathNode>,
    pub linear_gradient_path: Option<ColorPaintLinearGradientPathNode>,
    pub radial_gradient_path: Option<ColorPaintRadialGradientPathNode>,
    pub sweep_gradient_path: Option<ColorPaintSweepGradientPathNode>,
    pub transform: Option<ColorPaintTransformNode>,
    pub source_range_utf8: Option<TextSourceRange>,
    pub glyph_range: Option<GlyphRange>,
    pub source_font_ref: Option<FontColorGlyphRef>,
}

#[derive(Debug, Clone)]
pub struct ColorPaintGraphPayload {
    pub root_node_id: u32,
    pub nodes: Vec<ColorPaintGraphNode>,
}

const MAX_COLRV1_GRAPH_NODES: usize = 64;
const MAX_COLRV1_GRAPH_DEPTH: usize = 64;

fn text_source_range_is_valid(range: TextSourceRange) -> bool {
    range.end >= range.start
}

fn glyph_range_is_valid(range: GlyphRange) -> bool {
    range.end >= range.start
}

fn path_commands_are_finite(commands: &[PathCommand]) -> bool {
    !commands.is_empty()
        && commands.iter().all(|command| match *command {
            PathCommand::MoveTo(x, y) | PathCommand::LineTo(x, y) => x.is_finite() && y.is_finite(),
            PathCommand::CurveTo(x1, y1, x2, y2, x, y) => {
                [x1, y1, x2, y2, x, y].into_iter().all(f64::is_finite)
            }
            PathCommand::ArcTo(rx, ry, rotation, _, _, x, y) => {
                [rx, ry, rotation, x, y].into_iter().all(f64::is_finite)
            }
            PathCommand::ClosePath => true,
        })
}

fn resolved_color_is_valid(color: &ResolvedColor) -> bool {
    color
        .color_space
        .as_ref()
        .map(|color_space| !color_space.is_empty())
        .unwrap_or(true)
        && color
            .rgba
            .iter()
            .all(|component| component.is_finite() && (0.0..=1.0).contains(component))
}

fn color_gradient_stops_are_valid(stops: &[ColorGradientStop]) -> bool {
    if stops.len() < 2 {
        return false;
    }
    let mut previous_offset = f64::NEG_INFINITY;
    stops.iter().all(|stop| {
        let valid = stop.offset.is_finite()
            && (0.0..=1.0).contains(&stop.offset)
            && stop.offset >= previous_offset
            && resolved_color_is_valid(&stop.color);
        previous_offset = stop.offset;
        valid
    })
}

fn color_sweep_is_supported_full_circle(start_angle_degrees: f64, end_angle_degrees: f64) -> bool {
    start_angle_degrees.is_finite()
        && end_angle_degrees.is_finite()
        && start_angle_degrees < end_angle_degrees
        && (end_angle_degrees - start_angle_degrees - 360.0).abs() <= 1e-9
}

fn graph_leaf_metadata_is_valid(node: &ColorPaintGraphNode) -> bool {
    node.source_range_utf8
        .is_some_and(text_source_range_is_valid)
        && node.glyph_range.is_some_and(GlyphRange::is_non_empty)
        && node.source_font_ref.is_some()
}

impl ColorPaintGraphPayload {
    /// Compatibility alias for the P19 first supported COLRv1 graph contract.
    /// New code should use `has_colrv1_supported_graph_contract`.
    #[deprecated(note = "use has_colrv1_supported_graph_contract")]
    pub fn has_colrv1_stage1_contract(&self) -> bool {
        self.has_colrv1_supported_graph_contract()
    }

    pub fn has_colrv1_supported_graph_contract(&self) -> bool {
        use std::collections::{HashMap, HashSet};

        if self.nodes.is_empty() || self.nodes.len() > MAX_COLRV1_GRAPH_NODES {
            return false;
        }

        let mut nodes_by_id = HashMap::with_capacity(self.nodes.len());
        for node in &self.nodes {
            if nodes_by_id.insert(node.node_id, node).is_some() {
                return false;
            }
        }

        let mut visited = HashSet::new();
        let mut node_id = self.root_node_id;
        let mut depth = 1usize;
        loop {
            if depth > MAX_COLRV1_GRAPH_DEPTH || !visited.insert(node_id) {
                return false;
            }
            let Some(node) = nodes_by_id.get(&node_id) else {
                return false;
            };
            match node.kind {
                ColorPaintGraphNodeKind::SolidPath => {
                    if visited.len() != self.nodes.len()
                        || node.transform.is_some()
                        || node.linear_gradient_path.is_some()
                        || node.radial_gradient_path.is_some()
                        || node.sweep_gradient_path.is_some()
                        || !graph_leaf_metadata_is_valid(node)
                    {
                        return false;
                    }
                    let Some(solid) = node.solid_path.as_ref() else {
                        return false;
                    };
                    return path_commands_are_finite(&solid.commands)
                        && resolved_color_is_valid(&solid.fill);
                }
                ColorPaintGraphNodeKind::LinearGradientPath => {
                    if visited.len() != self.nodes.len()
                        || node.solid_path.is_some()
                        || node.transform.is_some()
                        || node.radial_gradient_path.is_some()
                        || node.sweep_gradient_path.is_some()
                        || !graph_leaf_metadata_is_valid(node)
                    {
                        return false;
                    }
                    let Some(gradient_path) = node.linear_gradient_path.as_ref() else {
                        return false;
                    };
                    return path_commands_are_finite(&gradient_path.commands)
                        && gradient_path.gradient.x0.is_finite()
                        && gradient_path.gradient.y0.is_finite()
                        && gradient_path.gradient.x1.is_finite()
                        && gradient_path.gradient.y1.is_finite()
                        && color_gradient_stops_are_valid(&gradient_path.gradient.stops);
                }
                ColorPaintGraphNodeKind::RadialGradientPath => {
                    if visited.len() != self.nodes.len()
                        || node.solid_path.is_some()
                        || node.transform.is_some()
                        || node.linear_gradient_path.is_some()
                        || node.sweep_gradient_path.is_some()
                        || !graph_leaf_metadata_is_valid(node)
                    {
                        return false;
                    }
                    let Some(gradient_path) = node.radial_gradient_path.as_ref() else {
                        return false;
                    };
                    return path_commands_are_finite(&gradient_path.commands)
                        && gradient_path.gradient.cx.is_finite()
                        && gradient_path.gradient.cy.is_finite()
                        && gradient_path.gradient.radius.is_finite()
                        && gradient_path.gradient.radius > 0.0
                        && color_gradient_stops_are_valid(&gradient_path.gradient.stops);
                }
                ColorPaintGraphNodeKind::SweepGradientPath => {
                    if visited.len() != self.nodes.len()
                        || node.solid_path.is_some()
                        || node.transform.is_some()
                        || node.linear_gradient_path.is_some()
                        || node.radial_gradient_path.is_some()
                        || !graph_leaf_metadata_is_valid(node)
                    {
                        return false;
                    }
                    let Some(gradient_path) = node.sweep_gradient_path.as_ref() else {
                        return false;
                    };
                    return path_commands_are_finite(&gradient_path.commands)
                        && gradient_path.gradient.cx.is_finite()
                        && gradient_path.gradient.cy.is_finite()
                        && color_sweep_is_supported_full_circle(
                            gradient_path.gradient.start_angle_degrees,
                            gradient_path.gradient.end_angle_degrees,
                        )
                        && color_gradient_stops_are_valid(&gradient_path.gradient.stops);
                }
                ColorPaintGraphNodeKind::Transform => {
                    if node.solid_path.is_some()
                        || node.linear_gradient_path.is_some()
                        || node.radial_gradient_path.is_some()
                        || node.sweep_gradient_path.is_some()
                    {
                        return false;
                    }
                    if node
                        .source_range_utf8
                        .is_some_and(|range| !text_source_range_is_valid(range))
                        || node
                            .glyph_range
                            .is_some_and(|range| !glyph_range_is_valid(range))
                    {
                        return false;
                    }
                    let Some(transform) = node.transform.as_ref() else {
                        return false;
                    };
                    if !layer_affine_is_finite(transform.transform) {
                        return false;
                    }
                    node_id = transform.child_node_id;
                    depth += 1;
                }
                ColorPaintGraphNodeKind::Composite => return false,
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct ColorLayersPayload {
    pub color_format: ColorGlyphFormat,
    pub source_font_ref: Option<FontColorGlyphRef>,
    pub palette_ref: Option<PaletteRef>,
    pub layers: Vec<ColorLayerNode>,
    pub paint_graph: Option<ColorPaintGraphPayload>,
    pub source_range_utf8: Option<TextSourceRange>,
    pub glyph_range: Option<GlyphRange>,
}

impl ColorLayersPayload {
    pub fn has_colrv0_resolved_layer_contract(&self) -> bool {
        self.color_format == ColorGlyphFormat::ColrV0
            && self.paint_graph.is_none()
            && !self.layers.is_empty()
            && self.layers.iter().all(|layer| {
                layer
                    .commands
                    .as_ref()
                    .is_some_and(|commands| !commands.is_empty())
                    && layer.fill.is_some()
                    && layer.fill_rule.is_some()
                    && layer
                        .source_range_utf8
                        .is_some_and(text_source_range_is_non_empty)
                    && layer.glyph_range.is_some_and(GlyphRange::is_non_empty)
                    && layer
                        .transform_to_run
                        .map(layer_affine_is_finite)
                        .unwrap_or(true)
                    && layer
                        .opacity
                        .map(|opacity| opacity.is_finite())
                        .unwrap_or(true)
            })
    }

    /// Compatibility alias for the P19 first supported COLRv1 graph contract.
    /// New code should use `has_colrv1_supported_graph_contract`.
    #[deprecated(note = "use has_colrv1_supported_graph_contract")]
    pub fn has_colrv1_stage1_graph_contract(&self) -> bool {
        self.has_colrv1_supported_graph_contract()
    }

    pub fn has_colrv1_supported_graph_contract(&self) -> bool {
        self.color_format == ColorGlyphFormat::ColrV1
            && self.layers.is_empty()
            && self
                .source_range_utf8
                .is_some_and(text_source_range_is_valid)
            && self.glyph_range.is_some_and(GlyphRange::is_non_empty)
            && self.source_font_ref.is_some()
            && self
                .paint_graph
                .as_ref()
                .is_some_and(ColorPaintGraphPayload::has_colrv1_supported_graph_contract)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BitmapGlyphScalingPolicy {
    SourceExact,
    PixelAligned,
    BackendDefault,
}

impl BitmapGlyphScalingPolicy {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::SourceExact => "sourceExact",
            Self::PixelAligned => "pixelAligned",
            Self::BackendDefault => "backendDefault",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BitmapGlyphFiltering {
    Nearest,
    Linear,
}

impl BitmapGlyphFiltering {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Nearest => "nearest",
            Self::Linear => "linear",
        }
    }
}

#[derive(Debug, Clone)]
pub struct BitmapGlyphPayload {
    pub image_ref: ImageResourceId,
    pub source_range_utf8: TextSourceRange,
    pub glyph_range: GlyphRange,
    pub placement: BoundingBox,
    pub alpha_premultiplied: bool,
    pub scaling_policy: BitmapGlyphScalingPolicy,
    pub filtering: BitmapGlyphFiltering,
    pub transform_to_run: Option<LayerAffineTransform>,
}

impl BitmapGlyphPayload {
    pub fn has_strict_visual_contract(&self) -> bool {
        text_source_range_is_non_empty(self.source_range_utf8)
            && self.glyph_range.is_non_empty()
            && bbox_is_finite_positive(self.placement)
            && !matches!(
                self.scaling_policy,
                BitmapGlyphScalingPolicy::BackendDefault
            )
            && self
                .transform_to_run
                .map(layer_affine_is_finite)
                .unwrap_or(true)
    }
}

#[derive(Debug, Clone)]
pub struct SvgGlyphPayload {
    pub svg_ref: SvgResourceId,
    pub source_range_utf8: TextSourceRange,
    pub glyph_range: GlyphRange,
    pub view_box: BoundingBox,
    pub intrinsic_size: Option<LayerVector>,
    pub static_sanitized: bool,
    pub script_allowed: bool,
    pub animation_allowed: bool,
    pub external_resources_allowed: bool,
    pub interactivity_allowed: bool,
    pub transform_to_run: Option<LayerAffineTransform>,
}

impl SvgGlyphPayload {
    pub fn has_static_sanitized_contract(&self) -> bool {
        text_source_range_is_non_empty(self.source_range_utf8)
            && self.glyph_range.is_non_empty()
            && bbox_is_finite_positive(self.view_box)
            && self
                .intrinsic_size
                .map(|size| {
                    size.dx.is_finite() && size.dy.is_finite() && size.dx > 0.0 && size.dy > 0.0
                })
                .unwrap_or(true)
            && self.static_sanitized
            && !self.script_allowed
            && !self.animation_allowed
            && !self.external_resources_allowed
            && !self.interactivity_allowed
            && self
                .transform_to_run
                .map(layer_affine_is_finite)
                .unwrap_or(true)
    }
}

fn text_source_range_is_non_empty(range: TextSourceRange) -> bool {
    range.end > range.start
}

fn layer_affine_is_finite(transform: LayerAffineTransform) -> bool {
    transform.a.is_finite()
        && transform.b.is_finite()
        && transform.c.is_finite()
        && transform.d.is_finite()
        && transform.e.is_finite()
        && transform.f.is_finite()
}

fn bbox_is_finite_positive(bbox: BoundingBox) -> bool {
    bbox.x.is_finite()
        && bbox.y.is_finite()
        && bbox.width.is_finite()
        && bbox.height.is_finite()
        && bbox.width > 0.0
        && bbox.height > 0.0
}

#[derive(Debug, Clone)]
pub struct LayerGlyphOutlinePath {
    pub glyph_id: u32,
    pub source_range_utf8: TextSourceRange,
    pub glyph_range: GlyphRange,
    pub commands: Vec<PathCommand>,
    pub fill_rule: GlyphOutlineFillRule,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GlyphOutlineFillRule {
    NonZero,
    EvenOdd,
}

impl GlyphOutlineFillRule {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::NonZero => "nonzero",
            Self::EvenOdd => "evenodd",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlyphOutlineStrokeStyle {
    pub color: ColorRef,
    pub width: f64,
    pub join: GlyphOutlineStrokeJoin,
    pub cap: GlyphOutlineStrokeCap,
    pub miter_limit: f64,
    pub paint_order: GlyphOutlinePaintOrder,
}

impl GlyphOutlineStrokeStyle {
    pub fn is_strict_subset(&self) -> bool {
        self.width.is_finite()
            && self.width > 0.0
            && self.miter_limit.is_finite()
            && self.miter_limit >= 1.0
            && matches!(self.join, GlyphOutlineStrokeJoin::Miter)
            && matches!(self.cap, GlyphOutlineStrokeCap::Butt)
            && matches!(
                self.paint_order,
                GlyphOutlinePaintOrder::FillThenStroke | GlyphOutlinePaintOrder::StrokeThenFill
            )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GlyphOutlineStrokeJoin {
    Miter,
    Round,
    Bevel,
}

impl GlyphOutlineStrokeJoin {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Miter => "miter",
            Self::Round => "round",
            Self::Bevel => "bevel",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GlyphOutlineStrokeCap {
    Butt,
    Round,
    Square,
}

impl GlyphOutlineStrokeCap {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Butt => "butt",
            Self::Round => "round",
            Self::Square => "square",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GlyphOutlinePaintOrder {
    FillOnly,
    StrokeOnly,
    FillThenStroke,
    StrokeThenFill,
}

impl GlyphOutlinePaintOrder {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::FillOnly => "fillOnly",
            Self::StrokeOnly => "strokeOnly",
            Self::FillThenStroke => "fillThenStroke",
            Self::StrokeThenFill => "strokeThenFill",
        }
    }
}

/// Variant grouping metadata for TextRun/GlyphRun/GlyphOutline alternatives.
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
    GlyphOutline,
}

impl TextVariantKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::TextRun => "textRun",
            Self::GlyphRun => "glyphRun",
            Self::GlyphOutline => "glyphOutline",
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

    pub fn is_non_empty(self) -> bool {
        self.end > self.start
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

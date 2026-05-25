//! 시각 레이어 IR 모듈
//!
//! semantic render tree를 backend-friendly layer tree로 변환한다.

pub mod builder;
pub mod font;
mod json;
pub mod layer_tree;
pub mod paint_op;
pub mod profile;
pub mod resources;
pub mod schema;
pub mod text_shape;
pub mod text_v2;
pub mod text_variants;

pub use builder::LayerBuilder;
pub use font::{
    BinaryResourceKind, BinaryResourceRef, FontBlobKey, FontBlobResource, FontDigest,
    FontExternalRef, FontFaceKey, FontFaceResource, FontFallbackPolicyId, FontInstanceKey,
    FontPortability, FontPortabilityKind, FontResourceSource, FontResourceTable,
    GlyphRunReplayEligibility, LanguageTag, LocalizedName, OpenTypeFeatureSetting, ScriptTag,
    ShapeKey, ShapingEngineId, TextDirection, VariationAxisValue, WritingMode,
};
pub use layer_tree::{
    CacheHint, ClipKind, GroupKind, LayerNode, LayerNodeKind, LayerOutputOptions, PageLayerTree,
    TextSourceAnnotation, TextSourceEntry, TextSourceId, TextSourceRange, TextSourceSpan,
    TextSourceTable,
};
pub use paint_op::{
    BitmapGlyphFiltering, BitmapGlyphPayload, BitmapGlyphScalingPolicy, ColorGlyphFormat,
    ColorGradientStop, ColorLayerNode, ColorLayersPayload, ColorLinearGradient,
    ColorPaintGraphNode, ColorPaintGraphNodeKind, ColorPaintGraphPayload,
    ColorPaintLinearGradientPathNode, ColorPaintRadialGradientPathNode, ColorPaintSolidPathNode,
    ColorPaintSweepGradientPathNode, ColorPaintTransformNode, ColorRadialGradient,
    ColorSweepGradient, FontColorGlyphRef, GlyphCluster, GlyphClusterFlag, GlyphOutlineFillRule,
    GlyphOutlinePaintOrder, GlyphOutlinePayloadKind, GlyphOutlineStrokeCap, GlyphOutlineStrokeJoin,
    GlyphOutlineStrokeStyle, GlyphRange, GlyphRunDiagnostics, GlyphRunOrientation, GlyphTransform,
    LayerAffineTransform, LayerGlyphOutlinePaint, LayerGlyphOutlinePath, LayerGlyphRunPaint,
    LayerPoint, LayerVector, PaintOp, PaintTextStyle, PaintVariantMeta, PaletteRef, ResolvedColor,
    ResolvedImageKind, ResolvedImagePayload, SvgGlyphPayload, TextDecorationKind, TextRunPlacement,
    TextVariantKind, TextVariantQuality,
};
pub use profile::RenderProfile;
pub use resources::{
    font_blob_resource_key, image_resource_key, resource_digest_hex, svg_resource_key,
    FontBlobResourceId, ImageResourceId, ResourceArena, SvgResourceId, RESOURCE_KEY_ALGORITHM,
};
pub use schema::{
    LayerTreeSchema, LAYER_TREE_SCHEMA, PAGE_LAYER_TREE_COORDINATE_SYSTEM,
    PAGE_LAYER_TREE_RESOURCE_TABLE_MINOR_VERSION, PAGE_LAYER_TREE_RESOURCE_TABLE_VERSION,
    PAGE_LAYER_TREE_SCHEMA_MINOR_VERSION, PAGE_LAYER_TREE_SCHEMA_VERSION, PAGE_LAYER_TREE_UNIT,
};
pub use text_shape::{
    FontRequest, FontResolver, GlyphRunQuality, NoopFontResolver, ResolvedFontFace,
    ResolvedGlyphRun, TextShapeDiagnostic, TextShapeLowerer, TextShapeReport,
};
pub use text_v2::{
    TextV2CompatibilityProfile, TextV2Diagnostics, TextV2LineBreakRisk, TextV2LineBreakRiskLevel,
    TextV2SlotDiagnostic, TextV2ValidationIssue, TextV2ValidationSeverity, TextV2VariantDiagnostic,
};
pub use text_variants::{validate_text_variant_scope, TextVariantScopeError};

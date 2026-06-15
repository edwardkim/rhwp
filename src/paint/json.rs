use std::fmt::Write as _;

use base64::Engine;

use crate::document_core::helpers::{color_ref_to_css, json_escape as raw_json_escape};
use crate::model::control::FormType;
use crate::model::image::ImageEffect;
use crate::model::style::{ImageFillMode, UnderlineType};
use crate::paint::ResourceArena;
use crate::paint::{
    BitmapGlyphPayload, CacheHint, ClipKind, ColorLayerNode, ColorLayersPayload,
    ColorPaintGraphNode, ColorPaintGraphPayload, FontColorGlyphRef, FontResourceTable,
    GlyphCluster, GlyphOutlinePayloadKind, GlyphOutlineStrokeStyle, GlyphRunDiagnostics,
    GlyphTransform, GroupKind, LayerAffineTransform, LayerGlyphOutlinePath, LayerNode,
    LayerNodeKind, LayerPoint, LayerVector, PageLayerTree, PaintOp, PaintTextStyle,
    PaintVariantMeta, PaletteRef, RenderProfile, ResolvedColor, ResolvedImageKind, ShapeKey,
    SvgGlyphPayload, TextDecorationKind, TextSourceAnnotation, TextSourceEntry, TextSourceId,
    TextSourceRange, TextSourceSpan, TextSourceTable, TextV2Diagnostics, LAYER_TREE_SCHEMA,
};
use crate::renderer::composer::expand_pua_display_text;
use crate::renderer::layout::compute_char_positions;
use crate::renderer::render_tree::{
    BoundingBox, FieldMarkerType, RenderLayerInfo, ShapeTransform, TextRunNode,
};
use crate::renderer::{
    ArrowStyle, GradientFillInfo, LineRenderType, LineStyle, PathCommand, PatternFillInfo,
    ShadowStyle, ShapeStyle, StrokeDash, TabLeaderInfo, TextStyle,
};

const KNOWN_TEXT_FEATURES: &[&str] = &[
    "fontResources",
    "fontResources.blobFaceSplit",
    "text.variantGroups",
    "text.shapeDiagnostics",
    "text.v2.diagnostics",
    "text.v2.slotDiagnostics",
    "text.v2.validationIssues",
    "text.lineBreakRiskTelemetry",
    "text.fallbackFreeStrictProfile",
    "text.glyphRun",
    "text.outlineGlyph",
    "text.glyphOutline",
    "text.glyphOutline.strictSidecar",
    "text.glyphOutline.monochromeFill",
    "text.glyphOutline.monochromeFillStroke",
    "text.glyphOutline.colorLayers",
    "text.glyphOutline.colorLayers.colrV0",
    "text.glyphOutline.colorLayers.colrV1",
    "text.glyphOutline.bitmapGlyph",
    "text.glyphOutline.svgGlyph",
    "text.glyphOutline.svgGlyph.vectorResourceId",
    "text.glyphOutline.payloadResourceKey",
    "text.glyphOutline.payloadResourceDigestKey",
    "text.specialVisualOps",
    "text.charOverlapOp",
    "text.controlMarkOp",
    "text.tabLeaderOp",
    "text.decorationOp",
    "text.displayText",
    "text.vertical.mixedPerGlyph",
];

impl PageLayerTree {
    pub fn to_json(&self) -> String {
        let mut buf = String::with_capacity(32_768);
        buf.push('{');
        let _ = write!(
            buf,
            "\"schemaVersion\":{},\"schemaMinorVersion\":{},\"schema\":{{\"major\":{},\"minor\":{}}},\"resourceTableVersion\":{},\"resourceTableMinorVersion\":{},\"resourceTable\":{{\"major\":{},\"minor\":{}}},\"unit\":{},\"coordinateSystem\":{},\"profile\":{},\"buildOptions\":{{\"showTransparentBorders\":{},\"clipEnabled\":{}}},\"debugOptions\":{{\"debugOverlay\":{}}},\"outputOptions\":{{\"showParagraphMarks\":{},\"showControlCodes\":{},\"showTransparentBorders\":{},\"clipEnabled\":{},\"debugOverlay\":{}}},\"pageWidth\":{:.3},\"pageHeight\":{:.3},\"root\":",
            LAYER_TREE_SCHEMA.schema_version,
            LAYER_TREE_SCHEMA.schema_minor_version,
            LAYER_TREE_SCHEMA.schema_version,
            LAYER_TREE_SCHEMA.schema_minor_version,
            LAYER_TREE_SCHEMA.resource_table_version,
            LAYER_TREE_SCHEMA.resource_table_minor_version,
            LAYER_TREE_SCHEMA.resource_table_version,
            LAYER_TREE_SCHEMA.resource_table_minor_version,
            json_escape(LAYER_TREE_SCHEMA.unit),
            json_escape(LAYER_TREE_SCHEMA.coordinate_system),
            json_escape(render_profile_str(self.profile)),
            self.output_options.show_transparent_borders,
            self.output_options.clip_enabled,
            self.output_options.debug_overlay,
            self.output_options.show_paragraph_marks,
            self.output_options.show_control_codes,
            self.output_options.show_transparent_borders,
            self.output_options.clip_enabled,
            self.output_options.debug_overlay,
            self.page_width,
            self.page_height
        );
        let mut text_source_state = TextSourceExportState::default();
        self.root
            .write_json(&mut buf, &mut text_source_state, &self.resources);
        buf.push_str(",\"textSources\":");
        write_text_source_entries(&mut buf, &self.text_sources);
        buf.push_str(",\"fontResources\":");
        write_font_resources(&mut buf, self.resources.font_resources());
        write_text_export_metadata(&mut buf, &self.root, &self.resources);
        buf.push_str(",\"textV2\":");
        TextV2Diagnostics::from_layer_tree(self).write_json(&mut buf);
        buf.push('}');
        buf
    }
}

fn write_text_export_metadata(buf: &mut String, root: &LayerNode, resources: &ResourceArena) {
    let externalized_visuals = externalized_text_visuals(root);
    let text_variant_features = collect_text_variant_features(root, resources);
    let has_variant_groups = text_variant_features.has_variant_groups();
    let has_glyph_runs = text_variant_features.has_glyph_runs;
    let has_glyph_outlines = text_variant_features.has_glyph_outlines;
    let has_glyph_outline_color_layers = text_variant_features.has_glyph_outline_color_layers;
    let has_glyph_outline_bitmap = text_variant_features.has_glyph_outline_bitmap;
    let has_glyph_outline_svg = text_variant_features.has_glyph_outline_svg;
    let has_glyph_outline_payload_resource_keys =
        text_variant_features.has_glyph_outline_payload_resource_keys;
    let has_glyph_outline_payload_resource_digest_keys =
        text_variant_features.has_glyph_outline_payload_resource_digest_keys;
    let has_display_text = text_variant_features.has_display_text;
    buf.push_str(",\"usedFeatures\":[\"text.paintStyle\",\"text.sourceTable\",\"text.sourceSpan\",\"text.v2.placement\",\"text.v2.clusters\",\"text.v2.diagnostics\",\"text.projectionKind\",\"text.legacyVisuals\",\"layer.optionMetadata\"");
    if has_display_text {
        buf.push_str(",\"text.displayText\"");
    }
    if has_glyph_runs {
        buf.push_str(",\"fontResources\",\"text.glyphRun\"");
    }
    if has_glyph_outlines {
        buf.push_str(",\"text.glyphOutline\",\"text.glyphOutline.strictSidecar\"");
    }
    if has_glyph_outline_color_layers {
        buf.push_str(",\"text.glyphOutline.colorLayers\"");
    }
    if has_glyph_outline_bitmap {
        buf.push_str(",\"text.glyphOutline.bitmapGlyph\"");
    }
    if has_glyph_outline_svg {
        buf.push_str(",\"text.glyphOutline.svgGlyph\"");
        buf.push_str(",\"text.glyphOutline.svgGlyph.vectorResourceId\"");
    }
    if has_glyph_outline_payload_resource_keys {
        buf.push_str(",\"text.glyphOutline.payloadResourceKey\"");
    }
    if has_glyph_outline_payload_resource_digest_keys {
        buf.push_str(",\"text.glyphOutline.payloadResourceDigestKey\"");
    }
    if has_variant_groups {
        buf.push_str(",\"text.variantGroups\"");
    }
    if externalized_visuals.contains(&"charOverlap") {
        buf.push_str(",\"text.charOverlapOp\"");
    }
    if externalized_visuals.contains(&"controlMarks") {
        buf.push_str(",\"text.controlMarkOp\"");
    }
    if externalized_visuals.contains(&"tabLeaders") {
        buf.push_str(",\"text.tabLeaderOp\"");
    }
    if externalized_visuals.contains(&"decorations") {
        buf.push_str(",\"text.decorationOp\"");
    }
    let mut optional_features = Vec::new();
    if has_glyph_runs {
        optional_features.push("fontResources");
        optional_features.push("text.glyphRun");
    }
    if has_glyph_outlines {
        optional_features.push("text.glyphOutline");
        optional_features.push("text.glyphOutline.strictSidecar");
    }
    if has_glyph_outline_color_layers {
        optional_features.push("text.glyphOutline.colorLayers");
    }
    if has_glyph_outline_bitmap {
        optional_features.push("text.glyphOutline.bitmapGlyph");
    }
    if has_glyph_outline_svg {
        optional_features.push("text.glyphOutline.svgGlyph");
        optional_features.push("text.glyphOutline.svgGlyph.vectorResourceId");
    }
    if has_glyph_outline_payload_resource_keys {
        optional_features.push("text.glyphOutline.payloadResourceKey");
    }
    if has_glyph_outline_payload_resource_digest_keys {
        optional_features.push("text.glyphOutline.payloadResourceDigestKey");
    }
    buf.push_str("],\"optionalFeatures\":[");
    for (idx, feature) in optional_features.iter().enumerate() {
        if idx > 0 {
            buf.push(',');
        }
        buf.push_str(&json_escape(feature));
    }
    buf.push_str("],\"knownFeatures\":[");
    for (idx, feature) in KNOWN_TEXT_FEATURES.iter().enumerate() {
        if idx > 0 {
            buf.push(',');
        }
        buf.push_str(&json_escape(feature));
    }
    buf.push_str("],\"requiredFeatures\":[],\"text\":{\"defaultVariant\":\"textRun\",\"variants\":[\"textRun\"");
    if has_glyph_runs {
        buf.push_str(",\"glyphRun\"");
    }
    if has_glyph_outlines {
        buf.push_str(",\"glyphOutline\"");
    }
    buf.push_str("],\"variantSelection\":\"exclusiveVariantSet\",\"sourceTextPreserved\":true,\"clusterEncoding\":[\"utf8\",\"utf16\"],\"fallbackRequired\":true,\"placementAuthority\":\"compatibilityProjection\",\"externalizedVisuals\":[");
    for (idx, visual) in externalized_visuals.iter().enumerate() {
        if idx > 0 {
            buf.push(',');
        }
        buf.push_str(&json_escape(visual));
    }
    buf.push_str("]}");
}

#[derive(Debug, Clone, Copy, Default)]
struct TextVariantFeatureFlags {
    has_glyph_runs: bool,
    has_glyph_outlines: bool,
    has_glyph_outline_color_layers: bool,
    has_glyph_outline_bitmap: bool,
    has_glyph_outline_svg: bool,
    has_glyph_outline_payload_resource_keys: bool,
    has_glyph_outline_payload_resource_digest_keys: bool,
    has_display_text: bool,
}

impl TextVariantFeatureFlags {
    fn has_variant_groups(self) -> bool {
        self.has_glyph_runs || self.has_glyph_outlines
    }
}

fn collect_text_variant_features(
    root: &LayerNode,
    resources: &ResourceArena,
) -> TextVariantFeatureFlags {
    let mut features = TextVariantFeatureFlags::default();
    let mut stack = vec![root];
    while let Some(node) = stack.pop() {
        match &node.kind {
            LayerNodeKind::Group { children, .. } => {
                for child in children {
                    stack.push(child);
                }
            }
            LayerNodeKind::ClipRect { child, .. } => stack.push(child),
            LayerNodeKind::Leaf { ops } => {
                for op in ops {
                    match op {
                        PaintOp::TextRun { run, .. } => {
                            features.has_display_text |= display_text_for_text_run(run).is_some()
                        }
                        PaintOp::GlyphRun { .. } => features.has_glyph_runs = true,
                        PaintOp::GlyphOutline { outline, .. } => {
                            features.has_glyph_outlines = true;
                            features.has_glyph_outline_color_layers |= matches!(
                                outline.payload_kind,
                                GlyphOutlinePayloadKind::ColorLayers
                            );
                            features.has_glyph_outline_bitmap |= matches!(
                                outline.payload_kind,
                                GlyphOutlinePayloadKind::BitmapGlyph
                            );
                            features.has_glyph_outline_svg |=
                                matches!(outline.payload_kind, GlyphOutlinePayloadKind::SvgGlyph);
                            features.has_glyph_outline_payload_resource_keys |=
                                outline.has_payload_resource_key();
                            features.has_glyph_outline_payload_resource_digest_keys |=
                                has_payload_resource_digest_key(outline, resources);
                        }
                        _ => {}
                    }
                }
            }
        }
        if features.has_glyph_runs
            && features.has_glyph_outlines
            && features.has_glyph_outline_color_layers
            && features.has_glyph_outline_bitmap
            && features.has_glyph_outline_svg
            && features.has_glyph_outline_payload_resource_keys
            && features.has_glyph_outline_payload_resource_digest_keys
            && features.has_display_text
        {
            return features;
        }
    }
    features
}

fn has_payload_resource_digest_key(
    outline: &crate::paint::LayerGlyphOutlinePaint,
    resources: &ResourceArena,
) -> bool {
    if !outline.has_payload_resource_key() {
        return false;
    }
    match outline.payload_kind {
        GlyphOutlinePayloadKind::BitmapGlyph => outline
            .bitmap_glyph
            .as_ref()
            .is_some_and(|payload| resources.image_bytes(payload.image_ref).is_some()),
        GlyphOutlinePayloadKind::SvgGlyph => outline
            .svg_glyph
            .as_ref()
            .is_some_and(|payload| resources.svg_fragment(payload.svg_ref).is_some()),
        GlyphOutlinePayloadKind::ColorLayers
        | GlyphOutlinePayloadKind::MonochromeFill
        | GlyphOutlinePayloadKind::MonochromeFillStroke => false,
    }
}

fn externalized_text_visuals(root: &LayerNode) -> Vec<&'static str> {
    let mut has_char_overlap = false;
    let mut has_control_marks = false;
    let mut has_tab_leaders = false;
    let mut has_decorations = false;
    let mut stack = vec![root];
    while let Some(node) = stack.pop() {
        match &node.kind {
            LayerNodeKind::Group { children, .. } => {
                for child in children {
                    stack.push(child);
                }
            }
            LayerNodeKind::ClipRect { child, .. } => stack.push(child),
            LayerNodeKind::Leaf { ops } => {
                has_char_overlap |= ops
                    .iter()
                    .any(|op| matches!(op, PaintOp::CharOverlap { .. }));
                has_control_marks |= ops
                    .iter()
                    .any(|op| matches!(op, PaintOp::TextControlMark { .. }));
                has_tab_leaders |= ops.iter().any(|op| matches!(op, PaintOp::TabLeader { .. }));
                has_decorations |= ops
                    .iter()
                    .any(|op| matches!(op, PaintOp::TextDecoration { .. }));
            }
        }
    }
    let mut visuals = Vec::new();
    if has_char_overlap {
        visuals.push("charOverlap");
    }
    if has_control_marks {
        visuals.push("controlMarks");
    }
    if has_tab_leaders {
        visuals.push("tabLeaders");
    }
    if has_decorations {
        visuals.push("decorations");
    }
    visuals
}

impl LayerNode {
    fn write_json(
        &self,
        buf: &mut String,
        text_sources: &mut TextSourceExportState,
        resources: &ResourceArena,
    ) {
        buf.push('{');
        buf.push_str("\"bounds\":");
        write_bbox(buf, self.bounds);
        if let Some(source_node_id) = self.source_node_id {
            let _ = write!(buf, ",\"sourceNodeId\":{}", source_node_id);
        }
        if let Some(layer) = self.layer {
            buf.push_str(",\"layer\":");
            write_render_layer_info(buf, layer);
        }

        match &self.kind {
            LayerNodeKind::Group {
                children,
                cache_hint,
                group_kind,
            } => {
                buf.push_str(",\"kind\":\"group\",\"groupKind\":");
                write_group_kind(buf, group_kind);
                let _ = write!(
                    buf,
                    ",\"cacheHint\":{},\"children\":[",
                    json_escape(cache_hint_str(*cache_hint))
                );
                for (idx, child) in children.iter().enumerate() {
                    if idx > 0 {
                        buf.push(',');
                    }
                    child.write_json(buf, text_sources, resources);
                }
                buf.push(']');
            }
            LayerNodeKind::ClipRect {
                clip,
                child,
                clip_kind,
            } => {
                buf.push_str(",\"kind\":\"clipRect\",\"clip\":");
                write_bbox(buf, *clip);
                let _ = write!(
                    buf,
                    ",\"clipKind\":{}",
                    json_escape(clip_kind_str(*clip_kind))
                );
                buf.push_str(",\"child\":");
                child.write_json(buf, text_sources, resources);
            }
            LayerNodeKind::Leaf { ops } => {
                buf.push_str(",\"kind\":\"leaf\",\"ops\":[");
                let leaf_visuals = LeafTextVisualOps::from_ops(ops);
                for (idx, op) in ops.iter().enumerate() {
                    if idx > 0 {
                        buf.push(',');
                    }
                    op.write_json(buf, text_sources, &leaf_visuals, resources);
                }
                buf.push(']');
            }
        }
        buf.push('}');
    }
}

impl PaintOp {
    fn write_json(
        &self,
        buf: &mut String,
        text_sources: &mut TextSourceExportState,
        leaf_visuals: &LeafTextVisualOps,
        resources: &ResourceArena,
    ) {
        match self {
            PaintOp::PageBackground { bbox, background } => {
                buf.push('{');
                buf.push_str("\"type\":\"pageBackground\",\"bbox\":");
                write_bbox(buf, *bbox);
                if let Some(color) = background.background_color {
                    let _ = write!(
                        buf,
                        ",\"backgroundColor\":{}",
                        json_escape(&color_ref_to_css(color))
                    );
                }
                if let Some(color) = background.border_color {
                    let _ = write!(
                        buf,
                        ",\"borderColor\":{}",
                        json_escape(&color_ref_to_css(color))
                    );
                }
                let _ = write!(buf, ",\"borderWidth\":{:.3}", background.border_width);
                if let Some(gradient) = &background.gradient {
                    buf.push_str(",\"gradient\":");
                    write_gradient(buf, gradient);
                }
                if let Some(image) = &background.image {
                    let base64_data = base64::engine::general_purpose::STANDARD.encode(&image.data);
                    let _ = write!(
                        buf,
                        ",\"image\":{{\"fillMode\":{},\"base64\":{}}}",
                        json_escape(image_fill_mode_str(image.fill_mode)),
                        json_escape(&base64_data),
                    );
                }
                buf.push('}');
            }
            PaintOp::TextRun { bbox, run } => {
                buf.push('{');
                buf.push_str("\"type\":\"textRun\",\"bbox\":");
                write_bbox(buf, *bbox);
                let source = text_sources.next_text_run_span(run);
                let display_text = display_text_for_text_run(run);
                let _ = write!(
                    buf,
                    ",\"text\":{},\"baseline\":{:.3},\"rotation\":{:.3},\"isVertical\":{},\"orientation\":{},\"projectionKind\":{},\"clusterBasis\":\"legacyPosition\"",
                    json_escape(&run.text),
                    run.baseline,
                    run.rotation,
                    run.is_vertical,
                    json_escape(text_orientation_str(run)),
                    json_escape(text_projection_kind_str(run)),
                );
                if let Some(display_text) = &display_text {
                    let _ = write!(buf, ",\"displayText\":{}", json_escape(display_text));
                }
                buf.push_str(",\"placement\":");
                write_text_run_placement(buf, *bbox, run);
                buf.push_str(",\"clusters\":");
                write_text_clusters(buf, run);
                buf.push_str(",\"source\":");
                write_text_source_span(buf, &source);
                if let Some(equivalence_group) = leaf_visuals.variant_group_for_source(source.id) {
                    buf.push_str(",\"variant\":");
                    write_paint_variant_meta(
                        buf,
                        &PaintVariantMeta::text_run_default(equivalence_group),
                    );
                }
                buf.push_str(",\"style\":");
                write_text_style(buf, &run.style);
                buf.push_str(",\"paintStyle\":");
                write_text_style(buf, &run.style);
                write_text_legacy_visuals(buf, run, leaf_visuals);
                buf.push_str(",\"positions\":");
                write_text_positions(buf, run);
                if let Some(display_text) = &display_text {
                    buf.push_str(",\"displayPositions\":");
                    if display_text.is_empty() {
                        buf.push_str("[]");
                    } else {
                        write_text_positions_for_text(buf, display_text, &run.style);
                    }
                }
                if !run.style.tab_leaders.is_empty() {
                    buf.push_str(",\"tabLeaders\":");
                    write_tab_leaders(buf, &run.style.tab_leaders);
                }
                let _ = write!(
                    buf,
                    ",\"isParaEnd\":{},\"isLineBreakEnd\":{},\"fieldMarker\":",
                    run.is_para_end, run.is_line_break_end,
                );
                write_field_marker(buf, run.field_marker);
                buf.push_str(",\"charOverlap\":");
                write_char_overlap(buf, run.char_overlap.as_ref());
                buf.push('}');
            }
            PaintOp::GlyphRun { bbox, run } => {
                buf.push('{');
                buf.push_str("\"type\":\"glyphRun\",\"bbox\":");
                write_bbox(buf, *bbox);
                buf.push_str(",\"source\":");
                write_text_source_span(buf, &run.source);
                buf.push_str(",\"variant\":");
                write_paint_variant_meta(buf, &run.variant);
                buf.push_str(",\"paintStyle\":");
                write_paint_text_style(buf, &run.paint_style);
                buf.push_str(",\"shapeKey\":");
                write_shape_key(buf, &run.shape_key);
                buf.push_str(",\"placement\":");
                write_text_run_placement_value(buf, run.placement);
                buf.push_str(",\"glyphIds\":[");
                for (idx, glyph_id) in run.glyph_ids.iter().enumerate() {
                    if idx > 0 {
                        buf.push(',');
                    }
                    let _ = write!(buf, "{}", glyph_id);
                }
                buf.push_str("],\"positions\":");
                write_points(buf, &run.positions);
                if let Some(advances) = &run.advances {
                    buf.push_str(",\"advances\":");
                    write_vectors(buf, advances);
                }
                buf.push_str(",\"clusters\":");
                write_glyph_clusters(buf, &run.clusters);
                let _ = write!(
                    buf,
                    ",\"direction\":{},\"writingMode\":{},\"orientation\":{}",
                    json_escape(run.direction.as_str()),
                    json_escape(run.writing_mode.as_str()),
                    json_escape(run.orientation.as_str()),
                );
                if let Some(bidi_level) = run.bidi_level {
                    let _ = write!(buf, ",\"bidiLevel\":{}", bidi_level);
                }
                if let Some(transforms) = &run.glyph_transforms {
                    buf.push_str(",\"glyphTransforms\":");
                    write_glyph_transforms(buf, transforms);
                }
                buf.push_str(",\"diagnostics\":");
                write_glyph_run_diagnostics(buf, &run.diagnostics);
                buf.push('}');
            }
            PaintOp::GlyphOutline { bbox, outline } => {
                buf.push('{');
                buf.push_str("\"type\":\"glyphOutline\",\"bbox\":");
                write_bbox(buf, *bbox);
                buf.push_str(",\"source\":");
                write_text_source_span(buf, &outline.source);
                buf.push_str(",\"variant\":");
                write_paint_variant_meta(buf, &outline.variant);
                let _ = write!(
                    buf,
                    ",\"payloadKind\":{}",
                    json_escape(outline.payload_kind.as_str())
                );
                if let Some(payload_resource_key) =
                    outline.payload_resource_key_with_resources(Some(resources))
                {
                    let _ = write!(
                        buf,
                        ",\"payloadResourceKey\":{}",
                        json_escape(&payload_resource_key)
                    );
                }
                buf.push_str(",\"paintStyle\":");
                write_paint_text_style(buf, &outline.paint_style);
                buf.push_str(",\"placement\":");
                write_text_run_placement_value(buf, outline.placement);
                buf.push_str(",\"paths\":");
                write_glyph_outline_paths(buf, &outline.paths);
                if let Some(stroke) = &outline.stroke {
                    buf.push_str(",\"stroke\":");
                    write_glyph_outline_stroke(buf, stroke);
                }
                if let Some(color_layers) = &outline.color_layers {
                    buf.push_str(",\"colorLayers\":");
                    write_color_layers_payload(buf, color_layers);
                }
                if let Some(bitmap_glyph) = &outline.bitmap_glyph {
                    buf.push_str(",\"bitmapGlyph\":");
                    write_bitmap_glyph_payload(buf, bitmap_glyph);
                }
                if let Some(svg_glyph) = &outline.svg_glyph {
                    buf.push_str(",\"svgGlyph\":");
                    write_svg_glyph_payload(buf, svg_glyph);
                }
                buf.push_str(",\"diagnostics\":");
                write_glyph_run_diagnostics(buf, &outline.diagnostics);
                buf.push('}');
            }
            PaintOp::CharOverlap { bbox, run } => {
                buf.push('{');
                buf.push_str("\"type\":\"charOverlap\",\"bbox\":");
                write_bbox(buf, *bbox);
                if let Some(source) = text_sources.last_source.as_ref() {
                    buf.push_str(",\"source\":");
                    write_text_source_span(buf, source);
                }
                let _ = write!(
                    buf,
                    ",\"text\":{},\"baseline\":{:.3},\"rotation\":{:.3},\"isVertical\":{},\"orientation\":{}",
                    json_escape(&run.text),
                    run.baseline,
                    run.rotation,
                    run.is_vertical,
                    json_escape(text_orientation_str(run)),
                );
                buf.push_str(",\"style\":");
                write_text_style(buf, &run.style);
                buf.push_str(",\"paintStyle\":");
                write_text_style(buf, &run.style);
                buf.push_str(",\"positions\":");
                write_text_positions(buf, run);
                buf.push_str(",\"charOverlap\":");
                write_char_overlap(buf, run.char_overlap.as_ref());
                buf.push('}');
            }
            PaintOp::TextControlMark { bbox, run } => {
                buf.push('{');
                buf.push_str("\"type\":\"textControlMark\",\"bbox\":");
                write_bbox(buf, *bbox);
                if let Some(source) = text_sources.last_source.as_ref() {
                    buf.push_str(",\"source\":");
                    write_text_source_span(buf, source);
                }
                let _ = write!(
                    buf,
                    ",\"fieldMarker\":{},\"isParaEnd\":{},\"isLineBreakEnd\":{}",
                    json_escape(field_marker_str(run.field_marker)),
                    run.is_para_end,
                    run.is_line_break_end,
                );
                if let FieldMarkerType::ShapeMarker(index) = run.field_marker {
                    let _ = write!(buf, ",\"shapeMarkerIndex\":{}", index);
                }
                buf.push('}');
            }
            PaintOp::TabLeader { bbox, run } => {
                buf.push('{');
                buf.push_str("\"type\":\"tabLeader\",\"bbox\":");
                write_bbox(buf, *bbox);
                if let Some(source) = text_sources.last_source.as_ref() {
                    buf.push_str(",\"source\":");
                    write_text_source_span(buf, source);
                }
                buf.push_str(",\"leaders\":");
                write_tab_leaders(buf, &run.style.tab_leaders);
                let _ = write!(
                    buf,
                    ",\"color\":{},\"fontSize\":{:.3},\"baseline\":{:.3}}}",
                    json_escape(&color_ref_to_css(run.style.color)),
                    run.style.font_size,
                    run.baseline,
                );
            }
            PaintOp::TextDecoration { bbox, run, kind } => {
                buf.push('{');
                buf.push_str("\"type\":\"textDecoration\",\"bbox\":");
                write_bbox(buf, *bbox);
                if let Some(source) = text_sources.last_source.as_ref() {
                    buf.push_str(",\"source\":");
                    write_text_source_span(buf, source);
                }
                buf.push_str(",\"decoration\":");
                write_text_decoration(buf, *kind, run);
                buf.push('}');
            }
            PaintOp::FootnoteMarker { bbox, marker } => {
                buf.push('{');
                buf.push_str("\"type\":\"footnoteMarker\",\"bbox\":");
                write_bbox(buf, *bbox);
                let _ = write!(
                    buf,
                    ",\"text\":{},\"fontFamily\":{},\"fontSize\":{:.3},\"color\":{}",
                    json_escape(&marker.text),
                    json_escape(&marker.font_family),
                    (marker.base_font_size * 0.55).max(7.0),
                    json_escape(&color_ref_to_css(marker.color)),
                );
                buf.push('}');
            }
            PaintOp::Line { bbox, line } => {
                buf.push('{');
                buf.push_str("\"type\":\"line\",\"bbox\":");
                write_bbox(buf, *bbox);
                let _ = write!(
                    buf,
                    ",\"x1\":{:.3},\"y1\":{:.3},\"x2\":{:.3},\"y2\":{:.3},\"style\":",
                    line.x1, line.y1, line.x2, line.y2
                );
                write_line_style(buf, &line.style);
                buf.push_str(",\"transform\":");
                write_transform(buf, line.transform);
                buf.push('}');
            }
            PaintOp::Rectangle { bbox, rect } => {
                buf.push('{');
                buf.push_str("\"type\":\"rectangle\",\"bbox\":");
                write_bbox(buf, *bbox);
                let _ = write!(
                    buf,
                    ",\"cornerRadius\":{:.3},\"style\":",
                    rect.corner_radius
                );
                write_shape_style(buf, &rect.style);
                if let Some(gradient) = &rect.gradient {
                    buf.push_str(",\"gradient\":");
                    write_gradient(buf, gradient);
                }
                buf.push_str(",\"transform\":");
                write_transform(buf, rect.transform);
                buf.push('}');
            }
            PaintOp::Ellipse { bbox, ellipse } => {
                buf.push('{');
                buf.push_str("\"type\":\"ellipse\",\"bbox\":");
                write_bbox(buf, *bbox);
                buf.push_str(",\"style\":");
                write_shape_style(buf, &ellipse.style);
                if let Some(gradient) = &ellipse.gradient {
                    buf.push_str(",\"gradient\":");
                    write_gradient(buf, gradient);
                }
                buf.push_str(",\"transform\":");
                write_transform(buf, ellipse.transform);
                buf.push('}');
            }
            PaintOp::Path { bbox, path } => {
                buf.push('{');
                buf.push_str("\"type\":\"path\",\"bbox\":");
                write_bbox(buf, *bbox);
                buf.push_str(",\"commands\":");
                write_path_commands(buf, &path.commands);
                buf.push_str(",\"style\":");
                write_shape_style(buf, &path.style);
                if let Some(gradient) = &path.gradient {
                    buf.push_str(",\"gradient\":");
                    write_gradient(buf, gradient);
                }
                if let Some((x1, y1, x2, y2)) = path.connector_endpoints {
                    let _ = write!(
                        buf,
                        ",\"connectorEndpoints\":{{\"x1\":{:.3},\"y1\":{:.3},\"x2\":{:.3},\"y2\":{:.3}}}",
                        x1, y1, x2, y2
                    );
                }
                if let Some(line_style) = &path.line_style {
                    buf.push_str(",\"lineStyle\":");
                    write_line_style(buf, line_style);
                }
                buf.push_str(",\"transform\":");
                write_transform(buf, path.transform);
                buf.push('}');
            }
            PaintOp::Image {
                bbox,
                image,
                resolved,
            } => {
                buf.push('{');
                buf.push_str("\"type\":\"image\",\"bbox\":");
                write_bbox(buf, *bbox);
                if let Some(payload) = resolved.as_deref() {
                    let base64_data =
                        base64::engine::general_purpose::STANDARD.encode(&payload.data);
                    let _ = write!(
                        buf,
                        ",\"mime\":\"{}\",\"base64\":{}",
                        payload.mime,
                        json_escape(&base64_data)
                    );
                    if matches!(payload.kind, ResolvedImageKind::BakedWatermark) {
                        buf.push_str(",\"bakedWatermark\":true");
                    }
                } else if let Some(data) = &image.data {
                    // Task #516 Stage 5.2: overlay layer 의 <img> data URL 생성용 mime 노출.
                    // PCX 등 비표준은 PNG 변환 후 emit (CLI SVG 와 동일 정책 적용).
                    let mime = crate::renderer::svg::detect_image_mime_type(data);
                    let (final_mime, final_data): (&str, std::borrow::Cow<[u8]>) =
                        if mime == "image/x-pcx" {
                            match crate::renderer::svg::pcx_bytes_to_png_bytes(data) {
                                Some(png) => ("image/png", std::borrow::Cow::Owned(png)),
                                None => (mime, std::borrow::Cow::Borrowed(data.as_slice())),
                            }
                        } else if mime == "image/bmp" {
                            match crate::renderer::svg::bmp_bytes_to_png_bytes(data) {
                                Some(png) => ("image/png", std::borrow::Cow::Owned(png)),
                                None => (mime, std::borrow::Cow::Borrowed(data.as_slice())),
                            }
                        } else {
                            (mime, std::borrow::Cow::Borrowed(data.as_slice()))
                        };
                    let base64_data =
                        base64::engine::general_purpose::STANDARD.encode(&*final_data);
                    let _ = write!(
                        buf,
                        ",\"mime\":\"{}\",\"base64\":{}",
                        final_mime,
                        json_escape(&base64_data)
                    );
                }
                if let Some(fill_mode) = image.fill_mode {
                    let _ = write!(
                        buf,
                        ",\"fillMode\":{}",
                        json_escape(image_fill_mode_str(fill_mode))
                    );
                }
                if let Some((width, height)) = image.original_size {
                    let _ = write!(
                        buf,
                        ",\"originalSize\":{{\"width\":{:.3},\"height\":{:.3}}}",
                        width, height
                    );
                }
                if let Some((left, top, right, bottom)) = image.crop {
                    let _ = write!(
                        buf,
                        ",\"crop\":{{\"left\":{},\"top\":{},\"right\":{},\"bottom\":{}}}",
                        left, top, right, bottom
                    );
                }
                let _ = write!(
                    buf,
                    ",\"effect\":{},\"brightness\":{},\"contrast\":{}",
                    json_escape(image_effect_str(image.effect)),
                    image.brightness,
                    image.contrast
                );
                // 워터마크 메타정보 (Task #516, AI 활용)
                let attr = crate::model::image::ImageAttr {
                    brightness: image.brightness,
                    contrast: image.contrast,
                    effect: image.effect,
                    bin_data_id: image.bin_data_id,
                    external_path: None,
                };
                if let Some(preset) = attr.watermark_preset() {
                    let _ = write!(buf, ",\"watermark\":{{\"preset\":\"{}\"}}", preset);
                }
                // 텍스트 흐름 wrap 모드 (Task #516, 다층 레이어 분리용).
                // BehindText / InFrontOfText 인 경우 web 측이 별도 overlay layer 로 분리.
                if let Some(wrap) = image.text_wrap {
                    let _ = write!(buf, ",\"wrap\":{}", json_escape(text_wrap_str(wrap)));
                }
                buf.push_str(",\"transform\":");
                write_transform(buf, image.transform);
                buf.push('}');
            }
            PaintOp::Equation { bbox, equation } => {
                buf.push('{');
                buf.push_str("\"type\":\"equation\",\"bbox\":");
                write_bbox(buf, *bbox);
                let _ = write!(
                    buf,
                    ",\"svgContent\":{},\"color\":{},\"fontSize\":{:.3}",
                    json_escape(&equation.svg_content),
                    json_escape(&equation.color_str),
                    equation.font_size
                );
                buf.push('}');
            }
            PaintOp::FormObject { bbox, form } => {
                buf.push('{');
                buf.push_str("\"type\":\"formObject\",\"bbox\":");
                write_bbox(buf, *bbox);
                let _ = write!(
                    buf,
                    ",\"formType\":{},\"caption\":{},\"text\":{},\"foreColor\":{},\"backColor\":{},\"value\":{},\"enabled\":{}",
                    json_escape(form_type_str(form.form_type)),
                    json_escape(&form.caption),
                    json_escape(&form.text),
                    json_escape(&form.fore_color),
                    json_escape(&form.back_color),
                    form.value,
                    form.enabled,
                );
                buf.push('}');
            }
            PaintOp::Placeholder { bbox, placeholder } => {
                buf.push('{');
                buf.push_str("\"type\":\"placeholder\",\"bbox\":");
                write_bbox(buf, *bbox);
                let _ = write!(
                    buf,
                    ",\"fillColor\":{},\"strokeColor\":{},\"label\":{}",
                    json_escape(&color_ref_to_css(placeholder.fill_color)),
                    json_escape(&color_ref_to_css(placeholder.stroke_color)),
                    json_escape(&placeholder.label),
                );
                buf.push('}');
            }
            PaintOp::RawSvg { bbox, raw } => {
                buf.push('{');
                buf.push_str("\"type\":\"rawSvg\",\"bbox\":");
                write_bbox(buf, *bbox);
                let _ = write!(buf, ",\"svg\":{}", json_escape(&raw.svg));
                buf.push('}');
            }
        }
    }
}

fn write_bbox(buf: &mut String, bbox: BoundingBox) {
    let _ = write!(
        buf,
        "{{\"x\":{:.3},\"y\":{:.3},\"width\":{:.3},\"height\":{:.3}}}",
        bbox.x, bbox.y, bbox.width, bbox.height
    );
}

#[derive(Default)]
struct TextSourceExportState {
    next_id: u32,
    last_source: Option<TextSourceSpan>,
}

impl TextSourceExportState {
    fn next_text_run_span(&mut self, run: &TextRunNode) -> TextSourceSpan {
        let span = TextSourceSpan {
            id: TextSourceId(self.next_id),
            utf8_range: TextSourceRange::new(0, run.text.len() as u32),
            utf16_range: TextSourceRange::new(0, run.text.encode_utf16().count() as u32),
            stable_source_key: stable_text_source_key(run),
        };
        self.next_id = self.next_id.saturating_add(1);
        self.last_source = Some(span.clone());
        span
    }
}

#[derive(Debug, Clone, Default)]
struct LeafTextVisualOps {
    char_overlap: bool,
    control_marks: bool,
    tab_leaders: bool,
    decorations: bool,
    glyph_variant_groups: Vec<(u32, String)>,
}

impl LeafTextVisualOps {
    fn from_ops(ops: &[PaintOp]) -> Self {
        let mut visuals = Self::default();
        for op in ops {
            match op {
                PaintOp::CharOverlap { .. } => visuals.char_overlap = true,
                PaintOp::TextControlMark { .. } => visuals.control_marks = true,
                PaintOp::TabLeader { .. } => visuals.tab_leaders = true,
                PaintOp::TextDecoration { .. } => visuals.decorations = true,
                PaintOp::GlyphRun { run, .. } => visuals
                    .glyph_variant_groups
                    .push((run.source.id.0, run.variant.equivalence_group.clone())),
                PaintOp::GlyphOutline { outline, .. } => visuals.glyph_variant_groups.push((
                    outline.source.id.0,
                    outline.variant.equivalence_group.clone(),
                )),
                _ => {}
            }
        }
        visuals
    }

    fn variant_group_for_source(&self, source: TextSourceId) -> Option<String> {
        self.glyph_variant_groups
            .iter()
            .find_map(|(id, group)| (*id == source.0).then(|| group.clone()))
    }
}

fn stable_text_source_key(run: &TextRunNode) -> Option<String> {
    let section = run.section_index?;
    let para = run.para_index?;
    let char_start = run.char_start.unwrap_or(0);
    let mut key = format!("section:{section}/para:{para}/char:{char_start}");
    if let Some(cell) = &run.cell_context {
        let path = cell
            .path
            .iter()
            .map(|entry| {
                format!(
                    "{}:{}:{}:{}",
                    entry.control_index,
                    entry.cell_index,
                    entry.cell_para_index,
                    entry.text_direction
                )
            })
            .collect::<Vec<_>>()
            .join(".");
        key.push_str("/cell:");
        key.push_str(&cell.parent_para_index.to_string());
        key.push(':');
        key.push_str(&path);
    }
    Some(key)
}

fn write_text_source_entries(buf: &mut String, table: &TextSourceTable) {
    buf.push('[');
    for (idx, entry) in table.entries.iter().enumerate() {
        if idx > 0 {
            buf.push(',');
        }
        write_text_source_entry(buf, entry);
    }
    buf.push(']');
}

fn write_font_resources(buf: &mut String, table: &FontResourceTable) {
    buf.push_str("{\"blobs\":[");
    for (idx, blob) in table.blobs.iter().enumerate() {
        if idx > 0 {
            buf.push(',');
        }
        let _ = write!(
            buf,
            "{{\"id\":{},\"source\":{},\"portability\":{}",
            json_escape(&blob.id.0),
            json_escape(blob.source.as_str()),
            json_escape(blob.portability.kind().as_str()),
        );
        if let Some(digest) = &blob.digest {
            let _ = write!(
                buf,
                ",\"digest\":{{\"algorithm\":{},\"value\":{}}}",
                json_escape(&digest.algorithm),
                json_escape(&digest.value),
            );
        }
        if let Some(data_ref) = &blob.data_ref {
            let _ = write!(
                buf,
                ",\"dataRef\":{{\"kind\":{},\"id\":{}}}",
                json_escape(data_ref.kind.as_str()),
                json_escape(&data_ref.id),
            );
        }
        buf.push('}');
    }
    buf.push_str("],\"faces\":[");
    for (idx, face) in table.faces.iter().enumerate() {
        if idx > 0 {
            buf.push(',');
        }
        let _ = write!(
            buf,
            "{{\"id\":{},\"blobKey\":{},\"faceIndex\":{}",
            json_escape(&face.id.0),
            json_escape(&face.blob_key.0),
            face.face_index,
        );
        if let Some(postscript_name) = &face.postscript_name {
            let _ = write!(buf, ",\"postscriptName\":{}", json_escape(postscript_name));
        }
        buf.push_str(",\"familyNames\":");
        write_localized_names(buf, &face.family_names);
        buf.push_str(",\"styleNames\":");
        write_localized_names(buf, &face.style_names);
        if let Some(weight_class) = face.weight_class {
            let _ = write!(buf, ",\"weightClass\":{}", weight_class);
        }
        if let Some(width_class) = face.width_class {
            let _ = write!(buf, ",\"widthClass\":{}", width_class);
        }
        if let Some(italic) = face.italic {
            let _ = write!(buf, ",\"italic\":{}", italic);
        }
        buf.push('}');
    }
    buf.push_str("]}");
}

fn write_localized_names(buf: &mut String, names: &[crate::paint::LocalizedName]) {
    buf.push('[');
    for (idx, name) in names.iter().enumerate() {
        if idx > 0 {
            buf.push(',');
        }
        let _ = write!(buf, "{{\"value\":{}", json_escape(&name.value));
        if let Some(locale) = &name.locale {
            let _ = write!(buf, ",\"locale\":{}", json_escape(locale));
        }
        buf.push('}');
    }
    buf.push(']');
}

fn write_text_source_entry(buf: &mut String, entry: &TextSourceEntry) {
    let _ = write!(
        buf,
        "{{\"id\":{},\"text\":{},\"utf8Range\":",
        entry.id.0,
        json_escape(&entry.text),
    );
    write_text_source_range(buf, entry.utf8_range);
    buf.push_str(",\"utf16Range\":");
    write_text_source_range(buf, entry.utf16_range);
    if let Some(stable_source_key) = &entry.stable_source_key {
        let _ = write!(
            buf,
            ",\"stableSourceKey\":{}",
            json_escape(stable_source_key)
        );
    }
    buf.push_str(",\"annotations\":");
    write_text_source_annotations(buf, &entry.annotations);
    buf.push('}');
}

fn write_text_source_span(buf: &mut String, span: &TextSourceSpan) {
    let _ = write!(buf, "{{\"id\":{},\"utf8Range\":", span.id.0);
    write_text_source_range(buf, span.utf8_range);
    buf.push_str(",\"utf16Range\":");
    write_text_source_range(buf, span.utf16_range);
    if let Some(stable_source_key) = &span.stable_source_key {
        let _ = write!(
            buf,
            ",\"stableSourceKey\":{}",
            json_escape(stable_source_key)
        );
    }
    buf.push('}');
}

fn write_text_source_range(buf: &mut String, range: TextSourceRange) {
    let _ = write!(buf, "{{\"start\":{},\"end\":{}}}", range.start, range.end);
}

fn write_text_source_annotations(buf: &mut String, annotations: &[TextSourceAnnotation]) {
    buf.push('[');
    for (idx, annotation) in annotations.iter().enumerate() {
        if idx > 0 {
            buf.push(',');
        }
        match annotation {
            TextSourceAnnotation::FieldMarker {
                marker,
                range_utf8,
                range_utf16,
            } => {
                let _ = write!(
                    buf,
                    "{{\"kind\":\"fieldMarker\",\"marker\":{},\"rangeUtf8\":",
                    json_escape(field_marker_str(*marker))
                );
                write_text_source_range(buf, *range_utf8);
                buf.push_str(",\"rangeUtf16\":");
                write_text_source_range(buf, *range_utf16);
                if let FieldMarkerType::ShapeMarker(index) = marker {
                    let _ = write!(buf, ",\"shapeMarkerIndex\":{}", index);
                }
                buf.push('}');
            }
            TextSourceAnnotation::ParagraphEnd {
                offset_utf8,
                offset_utf16,
            } => {
                let _ = write!(
                    buf,
                    "{{\"kind\":\"paragraphEnd\",\"offsetUtf8\":{},\"offsetUtf16\":{}}}",
                    offset_utf8, offset_utf16
                );
            }
            TextSourceAnnotation::LineBreakEnd {
                offset_utf8,
                offset_utf16,
            } => {
                let _ = write!(
                    buf,
                    "{{\"kind\":\"lineBreakEnd\",\"offsetUtf8\":{},\"offsetUtf16\":{}}}",
                    offset_utf8, offset_utf16
                );
            }
        }
    }
    buf.push(']');
}

fn write_paint_variant_meta(buf: &mut String, variant: &PaintVariantMeta) {
    let _ = write!(
        buf,
        "{{\"equivalenceGroup\":{},\"variantId\":{},\"variantKind\":{},\"partIndex\":{},\"partCount\":{},\"isDefaultFallback\":{}",
        json_escape(&variant.equivalence_group),
        json_escape(&variant.variant_id),
        json_escape(variant.variant_kind.as_str()),
        variant.part_index,
        variant.part_count,
        variant.is_default_fallback,
    );
    if !variant.requires.is_empty() {
        buf.push_str(",\"requires\":[");
        for (idx, feature) in variant.requires.iter().enumerate() {
            if idx > 0 {
                buf.push(',');
            }
            let _ = write!(buf, "{}", json_escape(feature));
        }
        buf.push(']');
    }
    if let Some(quality) = variant.quality {
        let _ = write!(buf, ",\"quality\":{}", json_escape(quality.as_str()));
    }
    if let Some(anchor_op_id) = &variant.anchor_op_id {
        let _ = write!(buf, ",\"anchorOpId\":{}", json_escape(anchor_op_id));
    }
    if let Some(local_paint_order) = variant.local_paint_order {
        let _ = write!(buf, ",\"localPaintOrder\":{}", local_paint_order);
    }
    buf.push('}');
}

fn write_group_kind(buf: &mut String, group_kind: &GroupKind) {
    match group_kind {
        GroupKind::Generic => buf.push_str("{\"kind\":\"generic\"}"),
        GroupKind::MasterPage => buf.push_str("{\"kind\":\"masterPage\"}"),
        GroupKind::Header => buf.push_str("{\"kind\":\"header\"}"),
        GroupKind::Footer => buf.push_str("{\"kind\":\"footer\"}"),
        GroupKind::Body => buf.push_str("{\"kind\":\"body\"}"),
        GroupKind::Column(index) => {
            let _ = write!(buf, "{{\"kind\":\"column\",\"index\":{}}}", index);
        }
        GroupKind::FootnoteArea => buf.push_str("{\"kind\":\"footnoteArea\"}"),
        GroupKind::TextLine(line) => {
            let _ = write!(
                buf,
                "{{\"kind\":\"textLine\",\"lineHeight\":{:.3},\"baseline\":{:.3}}}",
                line.line_height, line.baseline
            );
        }
        GroupKind::Table(table) => {
            let _ = write!(
                buf,
                "{{\"kind\":\"table\",\"rowCount\":{},\"colCount\":{},\"borderFillId\":{}}}",
                table.row_count, table.col_count, table.border_fill_id
            );
        }
        GroupKind::TableCell(cell) => {
            let _ = write!(
                buf,
                "{{\"kind\":\"tableCell\",\"row\":{},\"col\":{},\"rowSpan\":{},\"colSpan\":{},\"borderFillId\":{},\"textDirection\":{},\"clip\":{}",
                cell.row,
                cell.col,
                cell.row_span,
                cell.col_span,
                cell.border_fill_id,
                cell.text_direction,
                cell.clip
            );
            if let Some(index) = cell.model_cell_index {
                let _ = write!(buf, ",\"modelCellIndex\":{}", index);
            }
            buf.push('}');
        }
        GroupKind::TextBox => buf.push_str("{\"kind\":\"textBox\"}"),
        GroupKind::Group(group) => {
            buf.push_str("{\"kind\":\"group\"");
            if let Some(section_index) = group.section_index {
                let _ = write!(buf, ",\"sectionIndex\":{}", section_index);
            }
            if let Some(para_index) = group.para_index {
                let _ = write!(buf, ",\"paraIndex\":{}", para_index);
            }
            if let Some(control_index) = group.control_index {
                let _ = write!(buf, ",\"controlIndex\":{}", control_index);
            }
            buf.push('}');
        }
    }
}

fn cache_hint_str(value: CacheHint) -> &'static str {
    match value {
        CacheHint::None => "none",
        CacheHint::StaticSubtree => "staticSubtree",
        CacheHint::PreferRaster => "preferRaster",
        CacheHint::PreferVectorRecording => "preferVectorRecording",
    }
}

fn clip_kind_str(value: ClipKind) -> &'static str {
    match value {
        ClipKind::Body => "body",
        ClipKind::TableCell => "tableCell",
        ClipKind::TextBox => "textBox",
        ClipKind::Generic => "generic",
    }
}

fn write_text_style(buf: &mut String, style: &TextStyle) {
    buf.push('{');
    let _ = write!(
        buf,
        "\"fontFamily\":{},\"fontSize\":{:.3},\"color\":{},\"bold\":{},\"italic\":{},\"ratio\":{:.6},\"underline\":{},\"underlineShape\":{},\"strikethrough\":{},\"strikeShape\":{},\"outlineType\":{},\"shadowType\":{},\"shadowColor\":{},\"shadowOffsetX\":{:.3},\"shadowOffsetY\":{:.3},\"emboss\":{},\"engrave\":{},\"superscript\":{},\"subscript\":{},\"underlineColor\":{},\"strikeColor\":{},\"shadeColor\":{},\"emphasisDot\":{}",
        json_escape(&style.font_family),
        style.font_size,
        json_escape(&color_ref_to_css(style.color)),
        style.bold,
        style.italic,
        style.ratio,
        json_escape(underline_type_str(style.underline)),
        style.underline_shape,
        style.strikethrough,
        style.strike_shape,
        style.outline_type,
        style.shadow_type,
        json_escape(&color_ref_to_css(style.shadow_color)),
        style.shadow_offset_x,
        style.shadow_offset_y,
        style.emboss,
        style.engrave,
        style.superscript,
        style.subscript,
        json_escape(&color_ref_to_css(style.underline_color)),
        json_escape(&color_ref_to_css(style.strike_color)),
        json_escape(&color_ref_to_css(style.shade_color)),
        style.emphasis_dot,
    );
    buf.push('}');
}

fn write_paint_text_style(buf: &mut String, style: &PaintTextStyle) {
    buf.push('{');
    let _ = write!(
        buf,
        "\"fontFamily\":{},\"fontSize\":{:.3},\"color\":{},\"bold\":{},\"italic\":{},\"ratio\":{:.6},\"underline\":{},\"underlineShape\":{},\"strikethrough\":{},\"strikeShape\":{},\"outlineType\":{},\"shadowType\":{},\"shadowColor\":{},\"shadowOffsetX\":{:.3},\"shadowOffsetY\":{:.3},\"emboss\":{},\"engrave\":{},\"superscript\":{},\"subscript\":{},\"underlineColor\":{},\"strikeColor\":{},\"shadeColor\":{},\"emphasisDot\":{}",
        json_escape(&style.font_family),
        style.font_size,
        json_escape(&color_ref_to_css(style.color)),
        style.bold,
        style.italic,
        style.ratio,
        json_escape(underline_type_str(style.underline)),
        style.underline_shape,
        style.strikethrough,
        style.strike_shape,
        style.outline_type,
        style.shadow_type,
        json_escape(&color_ref_to_css(style.shadow_color)),
        style.shadow_offset_x,
        style.shadow_offset_y,
        style.emboss,
        style.engrave,
        style.superscript,
        style.subscript,
        json_escape(&color_ref_to_css(style.underline_color)),
        json_escape(&color_ref_to_css(style.strike_color)),
        json_escape(&color_ref_to_css(style.shade_color)),
        style.emphasis_dot,
    );
    buf.push('}');
}

fn write_text_positions(buf: &mut String, run: &TextRunNode) {
    write_text_positions_for_text(buf, &run.text, &run.style);
}

fn write_text_positions_for_text(buf: &mut String, text: &str, style: &TextStyle) {
    let positions = compute_char_positions(text, style);
    buf.push('[');
    for (idx, position) in positions.iter().enumerate() {
        if idx > 0 {
            buf.push(',');
        }
        let _ = write!(buf, "{:.3}", position);
    }
    buf.push(']');
}

fn display_text_for_text_run(run: &TextRunNode) -> Option<String> {
    let display_text = expand_pua_display_text(&run.text);
    (display_text != run.text.as_str()).then_some(display_text)
}

fn write_tab_leaders(buf: &mut String, leaders: &[TabLeaderInfo]) {
    buf.push('[');
    for (idx, leader) in leaders.iter().enumerate() {
        if idx > 0 {
            buf.push(',');
        }
        let _ = write!(
            buf,
            "{{\"startX\":{:.3},\"endX\":{:.3},\"fillType\":{}}}",
            leader.start_x, leader.end_x, leader.fill_type
        );
    }
    buf.push(']');
}

fn write_field_marker(buf: &mut String, marker: FieldMarkerType) {
    match marker {
        FieldMarkerType::None => buf.push_str("{\"kind\":\"none\"}"),
        FieldMarkerType::FieldBegin => buf.push_str("{\"kind\":\"fieldBegin\"}"),
        FieldMarkerType::FieldEnd => buf.push_str("{\"kind\":\"fieldEnd\"}"),
        FieldMarkerType::FieldBeginEnd => buf.push_str("{\"kind\":\"fieldBeginEnd\"}"),
        FieldMarkerType::ShapeMarker(index) => {
            let _ = write!(
                buf,
                "{{\"kind\":\"shapeMarker\",\"controlIndex\":{}}}",
                index
            );
        }
    }
}

fn field_marker_str(value: FieldMarkerType) -> &'static str {
    match value {
        FieldMarkerType::None => "none",
        FieldMarkerType::FieldBegin => "fieldBegin",
        FieldMarkerType::FieldEnd => "fieldEnd",
        FieldMarkerType::FieldBeginEnd => "fieldBeginEnd",
        FieldMarkerType::ShapeMarker(_) => "shapeMarker",
    }
}

fn write_char_overlap(
    buf: &mut String,
    overlap: Option<&crate::renderer::composer::CharOverlapInfo>,
) {
    if let Some(overlap) = overlap {
        let _ = write!(
            buf,
            "{{\"borderType\":{},\"innerCharSize\":{}}}",
            overlap.border_type, overlap.inner_char_size
        );
    } else {
        buf.push_str("null");
    }
}

fn text_orientation_str(run: &TextRunNode) -> &'static str {
    if !run.is_vertical {
        "horizontal"
    } else if run.rotation.abs() > f64::EPSILON {
        "vertical-sideways"
    } else {
        "vertical-upright"
    }
}

fn text_projection_kind_str(run: &TextRunNode) -> &'static str {
    if run.char_overlap.is_some() {
        "syntheticVisual"
    } else if run.field_marker != FieldMarkerType::None {
        "fieldProjection"
    } else if run.text.is_empty() && (run.is_para_end || run.is_line_break_end) {
        "controlProjection"
    } else {
        "verbatim"
    }
}

fn write_text_legacy_visuals(
    buf: &mut String,
    run: &TextRunNode,
    leaf_visuals: &LeafTextVisualOps,
) {
    let has_decorations = run.style.underline != UnderlineType::None
        || run.style.strikethrough
        || run.style.emphasis_dot > 0;
    if run.char_overlap.is_none()
        && !leaf_visuals.control_marks
        && run.style.tab_leaders.is_empty()
        && !has_decorations
    {
        return;
    }

    buf.push_str(",\"legacyVisuals\":{");
    let mut wrote = false;
    if run.char_overlap.is_some() {
        let state = if leaf_visuals.char_overlap {
            "mirror"
        } else {
            "canonical"
        };
        let _ = write!(buf, "\"charOverlap\":{}", json_escape(state));
        wrote = true;
    }
    if leaf_visuals.control_marks {
        if wrote {
            buf.push(',');
        }
        buf.push_str("\"controlMarks\":\"mirror\"");
        wrote = true;
    }
    if !run.style.tab_leaders.is_empty() {
        if wrote {
            buf.push(',');
        }
        let state = if leaf_visuals.tab_leaders {
            "mirror"
        } else {
            "canonical"
        };
        let _ = write!(buf, "\"tabLeaders\":{}", json_escape(state));
        wrote = true;
    }
    if has_decorations {
        if wrote {
            buf.push(',');
        }
        let state = if leaf_visuals.decorations {
            "mirror"
        } else {
            "canonical"
        };
        let _ = write!(buf, "\"decorations\":{}", json_escape(state));
    }
    buf.push('}');
}

fn write_text_run_placement(buf: &mut String, bbox: BoundingBox, run: &TextRunNode) {
    let radians = run.rotation.to_radians();
    let (sin, cos) = radians.sin_cos();
    let local_origin_x = -bbox.width / 2.0;
    let local_origin_y = -bbox.height / 2.0 + run.baseline;
    let center_x = bbox.x + bbox.width / 2.0;
    let center_y = bbox.y + bbox.height / 2.0;
    let _ = write!(
        buf,
        "{{\"runToPage\":{{\"a\":{:.6},\"b\":{:.6},\"c\":{:.6},\"d\":{:.6},\"e\":{:.6},\"f\":{:.6}}},\"baselineY\":0.000000}}",
        cos,
        sin,
        -sin,
        cos,
        center_x + cos * local_origin_x - sin * local_origin_y,
        center_y + sin * local_origin_x + cos * local_origin_y,
    );
}

fn write_text_run_placement_value(buf: &mut String, placement: crate::paint::TextRunPlacement) {
    buf.push_str("{\"runToPage\":");
    write_affine_transform(buf, placement.run_to_page);
    let _ = write!(buf, ",\"baselineY\":{:.6}}}", placement.baseline_y);
}

fn write_affine_transform(buf: &mut String, transform: LayerAffineTransform) {
    let _ = write!(
        buf,
        "{{\"a\":{:.6},\"b\":{:.6},\"c\":{:.6},\"d\":{:.6},\"e\":{:.6},\"f\":{:.6}}}",
        transform.a, transform.b, transform.c, transform.d, transform.e, transform.f,
    );
}

fn write_text_clusters(buf: &mut String, run: &TextRunNode) {
    let positions = compute_char_positions(&run.text, &run.style);
    let mut utf16_start = 0_u32;
    let chars = run
        .text
        .char_indices()
        .map(|(offset, ch)| (offset as u32, ch))
        .collect::<Vec<_>>();

    buf.push('[');
    for (idx, (utf8_start, ch)) in chars.iter().enumerate() {
        if idx > 0 {
            buf.push(',');
        }
        let utf8_end = chars
            .get(idx + 1)
            .map_or(run.text.len() as u32, |(next, _)| *next);
        let utf16_end = utf16_start + ch.len_utf16() as u32;
        let origin_x = positions.get(idx).copied().unwrap_or_default();
        let projection = text_projection_kind_str(run);
        buf.push_str("{\"sourceRangeUtf8\":");
        write_text_source_range(buf, TextSourceRange::new(*utf8_start, utf8_end));
        buf.push_str(",\"textRangeUtf8\":");
        write_text_source_range(buf, TextSourceRange::new(*utf8_start, utf8_end));
        buf.push_str(",\"textRangeUtf16\":");
        write_text_source_range(buf, TextSourceRange::new(utf16_start, utf16_end));
        let _ = write!(
            buf,
            ",\"projection\":{},\"origin\":{{\"x\":{:.6},\"y\":0.000000}}",
            json_escape(projection),
            origin_x
        );
        if let Some(next_x) = positions.get(idx + 1) {
            let _ = write!(
                buf,
                ",\"advance\":{{\"dx\":{:.6},\"dy\":0.000000}}",
                next_x - origin_x
            );
        }
        if run.char_overlap.is_some() {
            buf.push_str(",\"flags\":[\"specialVisual\",\"notShapingCandidate\"]");
        }
        buf.push('}');
        utf16_start = utf16_end;
    }
    buf.push(']');
}

fn write_shape_key(buf: &mut String, shape_key: &ShapeKey) {
    buf.push_str("{\"fontInstance\":{");
    let instance = &shape_key.font_instance;
    let _ = write!(
        buf,
        "\"faceKey\":{},\"sizePx\":{:.6},\"syntheticBold\":{},\"syntheticItalic\":{}",
        json_escape(&instance.face_key.0),
        instance.size_px,
        instance.synthetic_bold,
        instance.synthetic_italic,
    );
    buf.push_str(",\"variations\":[");
    for (idx, axis) in instance.variations.iter().enumerate() {
        if idx > 0 {
            buf.push(',');
        }
        let _ = write!(
            buf,
            "{{\"tag\":{},\"value\":{:.6}}}",
            json_escape(&axis.tag),
            axis.value
        );
    }
    buf.push_str("]}");
    let _ = write!(
        buf,
        ",\"direction\":{},\"writingMode\":{},\"shapingEngine\":{},\"fallbackPolicy\":{}",
        json_escape(shape_key.direction.as_str()),
        json_escape(shape_key.writing_mode.as_str()),
        json_escape(&shape_key.shaping_engine.0),
        json_escape(&shape_key.fallback_policy.0),
    );
    if let Some(script) = &shape_key.script {
        let _ = write!(buf, ",\"script\":{}", json_escape(&script.0));
    }
    if let Some(language) = &shape_key.language {
        let _ = write!(buf, ",\"language\":{}", json_escape(&language.0));
    }
    buf.push_str(",\"features\":[");
    for (idx, feature) in shape_key.features.iter().enumerate() {
        if idx > 0 {
            buf.push(',');
        }
        let _ = write!(
            buf,
            "{{\"tag\":{},\"enabled\":{}",
            json_escape(&feature.tag),
            feature.enabled
        );
        if let Some(value) = feature.value {
            let _ = write!(buf, ",\"value\":{}", value);
        }
        buf.push('}');
    }
    buf.push_str("]}");
}

fn write_points(buf: &mut String, points: &[LayerPoint]) {
    buf.push('[');
    for (idx, point) in points.iter().enumerate() {
        if idx > 0 {
            buf.push(',');
        }
        let _ = write!(buf, "{{\"x\":{:.6},\"y\":{:.6}}}", point.x, point.y);
    }
    buf.push(']');
}

fn write_vectors(buf: &mut String, vectors: &[LayerVector]) {
    buf.push('[');
    for (idx, vector) in vectors.iter().enumerate() {
        if idx > 0 {
            buf.push(',');
        }
        let _ = write!(buf, "{{\"dx\":{:.6},\"dy\":{:.6}}}", vector.dx, vector.dy);
    }
    buf.push(']');
}

fn write_glyph_clusters(buf: &mut String, clusters: &[GlyphCluster]) {
    buf.push('[');
    for (idx, cluster) in clusters.iter().enumerate() {
        if idx > 0 {
            buf.push(',');
        }
        buf.push('{');
        buf.push_str("\"sourceRangeUtf8\":");
        write_text_source_range(buf, cluster.source_range_utf8);
        if let Some(range) = cluster.source_range_utf16 {
            buf.push_str(",\"sourceRangeUtf16\":");
            write_text_source_range(buf, range);
        }
        if let Some(range) = cluster.text_range_utf8 {
            buf.push_str(",\"textRangeUtf8\":");
            write_text_source_range(buf, range);
        }
        let _ = write!(
            buf,
            ",\"glyphRange\":{{\"start\":{},\"end\":{}}}",
            cluster.glyph_range.start, cluster.glyph_range.end
        );
        if !cluster.flags.is_empty() {
            buf.push_str(",\"flags\":[");
            for (flag_idx, flag) in cluster.flags.iter().enumerate() {
                if flag_idx > 0 {
                    buf.push(',');
                }
                let _ = write!(buf, "{}", json_escape(flag.as_str()));
            }
            buf.push(']');
        }
        buf.push('}');
    }
    buf.push(']');
}

fn write_glyph_outline_paths(buf: &mut String, paths: &[LayerGlyphOutlinePath]) {
    buf.push('[');
    for (idx, path) in paths.iter().enumerate() {
        if idx > 0 {
            buf.push(',');
        }
        let _ = write!(buf, "{{\"glyphId\":{},\"sourceRangeUtf8\":", path.glyph_id);
        write_text_source_range(buf, path.source_range_utf8);
        let _ = write!(
            buf,
            ",\"glyphRange\":{{\"start\":{},\"end\":{}}},\"fillRule\":{}",
            path.glyph_range.start,
            path.glyph_range.end,
            json_escape(path.fill_rule.as_str())
        );
        buf.push_str(",\"commands\":");
        write_path_commands(buf, &path.commands);
        buf.push('}');
    }
    buf.push(']');
}

fn write_glyph_outline_stroke(buf: &mut String, stroke: &GlyphOutlineStrokeStyle) {
    let _ = write!(
        buf,
        "{{\"color\":{},\"width\":{:.6},\"join\":{},\"cap\":{},\"miterLimit\":{:.6},\"paintOrder\":{},\"strictSubset\":{}}}",
        json_escape(&color_ref_to_css(stroke.color)),
        stroke.width,
        json_escape(stroke.join.as_str()),
        json_escape(stroke.cap.as_str()),
        stroke.miter_limit,
        json_escape(stroke.paint_order.as_str()),
        stroke.is_strict_subset()
    );
}

fn write_color_layers_payload(buf: &mut String, payload: &ColorLayersPayload) {
    let _ = write!(
        buf,
        "{{\"colorFormat\":{}",
        json_escape(payload.color_format.as_str())
    );
    if let Some(source_font_ref) = &payload.source_font_ref {
        buf.push_str(",\"sourceFontRef\":");
        write_font_color_glyph_ref(buf, source_font_ref);
    }
    if let Some(palette_ref) = &payload.palette_ref {
        buf.push_str(",\"paletteRef\":");
        write_palette_ref(buf, palette_ref);
    }
    buf.push_str(",\"layers\":");
    write_color_layer_nodes(buf, &payload.layers);
    if let Some(graph) = &payload.paint_graph {
        buf.push_str(",\"paintGraph\":");
        write_color_paint_graph(buf, graph);
    }
    if let Some(range) = payload.source_range_utf8 {
        buf.push_str(",\"sourceRangeUtf8\":");
        write_text_source_range(buf, range);
    }
    if let Some(range) = payload.glyph_range {
        let _ = write!(
            buf,
            ",\"glyphRange\":{{\"start\":{},\"end\":{}}}",
            range.start, range.end
        );
    }
    let _ = write!(
        buf,
        ",\"colrv0ResolvedLayerContract\":{},\"colrv1Stage1GraphContract\":{},\"colrv1SupportedGraphContract\":{}",
        payload.has_colrv0_resolved_layer_contract(),
        payload.has_colrv1_supported_graph_contract(),
        payload.has_colrv1_supported_graph_contract()
    );
    buf.push('}');
}

fn write_color_layer_nodes(buf: &mut String, layers: &[ColorLayerNode]) {
    buf.push('[');
    for (idx, layer) in layers.iter().enumerate() {
        if idx > 0 {
            buf.push(',');
        }
        buf.push('{');
        if let Some(layer_index) = layer.layer_index {
            let _ = write!(buf, "\"layerIndex\":{}", layer_index);
        } else {
            buf.push_str("\"layerIndex\":null");
        }
        if let Some(glyph_id) = layer.glyph_id {
            let _ = write!(buf, ",\"glyphId\":{}", glyph_id);
        }
        if let Some(range) = layer.glyph_range {
            let _ = write!(
                buf,
                ",\"glyphRange\":{{\"start\":{},\"end\":{}}}",
                range.start, range.end
            );
        }
        if let Some(range) = layer.source_range_utf8 {
            buf.push_str(",\"sourceRangeUtf8\":");
            write_text_source_range(buf, range);
        }
        if let Some(source_font_ref) = &layer.source_font_ref {
            buf.push_str(",\"sourceFontRef\":");
            write_font_color_glyph_ref(buf, source_font_ref);
        }
        if let Some(commands) = &layer.commands {
            buf.push_str(",\"commands\":");
            write_path_commands(buf, commands);
        }
        if let Some(fill) = &layer.fill {
            buf.push_str(",\"fill\":");
            write_resolved_color(buf, fill);
        }
        if let Some(fill_rule) = layer.fill_rule {
            let _ = write!(buf, ",\"fillRule\":{}", json_escape(fill_rule.as_str()));
        }
        if let Some(palette_index) = layer.palette_index {
            let _ = write!(buf, ",\"paletteIndex\":{}", palette_index);
        }
        if let Some(color) = layer.color {
            let _ = write!(buf, ",\"color\":{}", json_escape(&color_ref_to_css(color)));
        }
        if let Some(opacity) = layer.opacity {
            let _ = write!(buf, ",\"opacity\":{:.6}", opacity);
        }
        if let Some(transform) = layer.transform_to_run {
            buf.push_str(",\"transformToRun\":");
            write_affine_transform(buf, transform);
        }
        buf.push('}');
    }
    buf.push(']');
}

fn write_color_paint_graph(buf: &mut String, graph: &ColorPaintGraphPayload) {
    let _ = write!(buf, "{{\"rootNodeId\":{},\"nodes\":[", graph.root_node_id);
    for (idx, node) in graph.nodes.iter().enumerate() {
        if idx > 0 {
            buf.push(',');
        }
        write_color_paint_graph_node(buf, node);
    }
    buf.push_str("]}");
}

fn write_color_paint_graph_node(buf: &mut String, node: &ColorPaintGraphNode) {
    let _ = write!(
        buf,
        "{{\"nodeId\":{},\"kind\":{}",
        node.node_id,
        json_escape(node.kind.as_str())
    );
    if let Some(solid) = &node.solid_path {
        buf.push_str(",\"solidPath\":");
        write_color_paint_solid_path_node(buf, solid);
    }
    if let Some(gradient_path) = &node.linear_gradient_path {
        buf.push_str(",\"linearGradientPath\":");
        write_color_paint_linear_gradient_path_node(buf, gradient_path);
    }
    if let Some(gradient_path) = &node.radial_gradient_path {
        buf.push_str(",\"radialGradientPath\":");
        write_color_paint_radial_gradient_path_node(buf, gradient_path);
    }
    if let Some(gradient_path) = &node.sweep_gradient_path {
        buf.push_str(",\"sweepGradientPath\":");
        write_color_paint_sweep_gradient_path_node(buf, gradient_path);
    }
    if let Some(transform) = &node.transform {
        buf.push_str(",\"transform\":");
        write_color_paint_transform_node(buf, transform);
    }
    if let Some(range) = node.source_range_utf8 {
        buf.push_str(",\"sourceRangeUtf8\":");
        write_text_source_range(buf, range);
    }
    if let Some(range) = node.glyph_range {
        let _ = write!(
            buf,
            ",\"glyphRange\":{{\"start\":{},\"end\":{}}}",
            range.start, range.end
        );
    }
    if let Some(source_font_ref) = &node.source_font_ref {
        buf.push_str(",\"sourceFontRef\":");
        write_font_color_glyph_ref(buf, source_font_ref);
    }
    buf.push('}');
}

fn write_color_paint_solid_path_node(
    buf: &mut String,
    solid: &crate::paint::ColorPaintSolidPathNode,
) {
    buf.push_str("{\"commands\":");
    write_path_commands(buf, &solid.commands);
    buf.push_str(",\"fill\":");
    write_resolved_color(buf, &solid.fill);
    let _ = write!(
        buf,
        ",\"fillRule\":{}",
        json_escape(solid.fill_rule.as_str())
    );
    if let Some(source_glyph_id) = solid.source_glyph_id {
        let _ = write!(buf, ",\"sourceGlyphId\":{}", source_glyph_id);
    }
    if let Some(palette_index) = solid.palette_index {
        let _ = write!(buf, ",\"paletteIndex\":{}", palette_index);
    }
    buf.push('}');
}

fn write_color_gradient_stops(buf: &mut String, stops: &[crate::paint::ColorGradientStop]) {
    buf.push('[');
    for (idx, stop) in stops.iter().enumerate() {
        if idx > 0 {
            buf.push(',');
        }
        let _ = write!(buf, "{{\"offset\":{:.6}", stop.offset);
        buf.push_str(",\"color\":");
        write_resolved_color(buf, &stop.color);
        buf.push('}');
    }
    buf.push(']');
}

fn write_color_paint_linear_gradient_path_node(
    buf: &mut String,
    gradient_path: &crate::paint::ColorPaintLinearGradientPathNode,
) {
    buf.push_str("{\"commands\":");
    write_path_commands(buf, &gradient_path.commands);
    let _ = write!(
        buf,
        ",\"gradient\":{{\"x0\":{:.6},\"y0\":{:.6},\"x1\":{:.6},\"y1\":{:.6},\"stops\":",
        gradient_path.gradient.x0,
        gradient_path.gradient.y0,
        gradient_path.gradient.x1,
        gradient_path.gradient.y1
    );
    write_color_gradient_stops(buf, &gradient_path.gradient.stops);
    let _ = write!(
        buf,
        "}},\"fillRule\":{}",
        json_escape(gradient_path.fill_rule.as_str())
    );
    if let Some(source_glyph_id) = gradient_path.source_glyph_id {
        let _ = write!(buf, ",\"sourceGlyphId\":{}", source_glyph_id);
    }
    if let Some(palette_index) = gradient_path.palette_index {
        let _ = write!(buf, ",\"paletteIndex\":{}", palette_index);
    }
    buf.push('}');
}

fn write_color_paint_radial_gradient_path_node(
    buf: &mut String,
    gradient_path: &crate::paint::ColorPaintRadialGradientPathNode,
) {
    buf.push_str("{\"commands\":");
    write_path_commands(buf, &gradient_path.commands);
    let _ = write!(
        buf,
        ",\"gradient\":{{\"cx\":{:.6},\"cy\":{:.6},\"radius\":{:.6},\"stops\":",
        gradient_path.gradient.cx, gradient_path.gradient.cy, gradient_path.gradient.radius
    );
    write_color_gradient_stops(buf, &gradient_path.gradient.stops);
    let _ = write!(
        buf,
        "}},\"fillRule\":{}",
        json_escape(gradient_path.fill_rule.as_str())
    );
    if let Some(source_glyph_id) = gradient_path.source_glyph_id {
        let _ = write!(buf, ",\"sourceGlyphId\":{}", source_glyph_id);
    }
    if let Some(palette_index) = gradient_path.palette_index {
        let _ = write!(buf, ",\"paletteIndex\":{}", palette_index);
    }
    buf.push('}');
}

fn write_color_paint_sweep_gradient_path_node(
    buf: &mut String,
    gradient_path: &crate::paint::ColorPaintSweepGradientPathNode,
) {
    buf.push_str("{\"commands\":");
    write_path_commands(buf, &gradient_path.commands);
    let _ = write!(
        buf,
        ",\"gradient\":{{\"cx\":{:.6},\"cy\":{:.6},\"startAngleDegrees\":{:.6},\"endAngleDegrees\":{:.6},\"stops\":",
        gradient_path.gradient.cx,
        gradient_path.gradient.cy,
        gradient_path.gradient.start_angle_degrees,
        gradient_path.gradient.end_angle_degrees
    );
    write_color_gradient_stops(buf, &gradient_path.gradient.stops);
    let _ = write!(
        buf,
        "}},\"fillRule\":{}",
        json_escape(gradient_path.fill_rule.as_str())
    );
    if let Some(source_glyph_id) = gradient_path.source_glyph_id {
        let _ = write!(buf, ",\"sourceGlyphId\":{}", source_glyph_id);
    }
    if let Some(palette_index) = gradient_path.palette_index {
        let _ = write!(buf, ",\"paletteIndex\":{}", palette_index);
    }
    buf.push('}');
}

fn write_color_paint_transform_node(
    buf: &mut String,
    transform: &crate::paint::ColorPaintTransformNode,
) {
    let _ = write!(buf, "{{\"childNodeId\":{}", transform.child_node_id);
    buf.push_str(",\"transform\":");
    write_affine_transform(buf, transform.transform);
    buf.push('}');
}

fn write_bitmap_glyph_payload(buf: &mut String, payload: &BitmapGlyphPayload) {
    let _ = write!(
        buf,
        "{{\"imageRef\":{},\"sourceRangeUtf8\":",
        payload.image_ref.0
    );
    write_text_source_range(buf, payload.source_range_utf8);
    let _ = write!(
        buf,
        ",\"glyphRange\":{{\"start\":{},\"end\":{}}},\"placement\":",
        payload.glyph_range.start, payload.glyph_range.end
    );
    write_bbox(buf, payload.placement);
    let _ = write!(
        buf,
        ",\"alphaPremultiplied\":{},\"scalingPolicy\":{},\"filtering\":{},\"strictVisualContract\":{}",
        payload.alpha_premultiplied,
        json_escape(payload.scaling_policy.as_str()),
        json_escape(payload.filtering.as_str()),
        payload.has_strict_visual_contract()
    );
    if let Some(transform) = payload.transform_to_run {
        buf.push_str(",\"transformToRun\":");
        write_affine_transform(buf, transform);
    }
    buf.push('}');
}

fn write_svg_glyph_payload(buf: &mut String, payload: &SvgGlyphPayload) {
    let _ = write!(
        buf,
        "{{\"svgRef\":{},\"vectorResourceId\":{},\"sourceRangeUtf8\":",
        payload.svg_ref.0, payload.svg_ref.0
    );
    write_text_source_range(buf, payload.source_range_utf8);
    let _ = write!(
        buf,
        ",\"glyphRange\":{{\"start\":{},\"end\":{}}},\"viewBox\":",
        payload.glyph_range.start, payload.glyph_range.end
    );
    write_bbox(buf, payload.view_box);
    if let Some(size) = payload.intrinsic_size {
        let _ = write!(
            buf,
            ",\"intrinsicSize\":{{\"width\":{:.6},\"height\":{:.6}}}",
            size.dx, size.dy
        );
    }
    let _ = write!(
        buf,
        ",\"staticSanitized\":{},\"scriptAllowed\":{},\"animationAllowed\":{},\"externalResourcesAllowed\":{},\"interactivityAllowed\":{},\"staticSanitizedContract\":{}",
        payload.static_sanitized,
        payload.script_allowed,
        payload.animation_allowed,
        payload.external_resources_allowed,
        payload.interactivity_allowed,
        payload.has_static_sanitized_contract()
    );
    if let Some(transform) = payload.transform_to_run {
        buf.push_str(",\"transformToRun\":");
        write_affine_transform(buf, transform);
    }
    buf.push('}');
}

fn write_font_color_glyph_ref(buf: &mut String, value: &FontColorGlyphRef) {
    buf.push('{');
    let mut wrote = false;
    if let Some(face_key) = &value.face_key {
        let _ = write!(buf, "\"faceKey\":{}", json_escape(face_key));
        wrote = true;
    }
    if let Some(glyph_id) = value.glyph_id {
        if wrote {
            buf.push(',');
        }
        let _ = write!(buf, "\"glyphId\":{}", glyph_id);
        wrote = true;
    }
    if let Some(palette_index) = value.palette_index {
        if wrote {
            buf.push(',');
        }
        let _ = write!(buf, "\"paletteIndex\":{}", palette_index);
        wrote = true;
    }
    if let Some(color_format) = value.color_format {
        if wrote {
            buf.push(',');
        }
        let _ = write!(
            buf,
            "\"colorFormat\":{}",
            json_escape(color_format.as_str())
        );
    }
    buf.push('}');
}

fn write_palette_ref(buf: &mut String, value: &PaletteRef) {
    buf.push('{');
    let mut wrote = false;
    if let Some(id) = &value.id {
        let _ = write!(buf, "\"id\":{}", json_escape(id));
        wrote = true;
    }
    if let Some(index) = value.index {
        if wrote {
            buf.push(',');
        }
        let _ = write!(buf, "\"index\":{}", index);
        wrote = true;
    }
    if let Some(cpal_digest) = &value.cpal_digest {
        if wrote {
            buf.push(',');
        }
        let _ = write!(buf, "\"cpalDigest\":{}", json_escape(cpal_digest));
    }
    buf.push('}');
}

fn write_resolved_color(buf: &mut String, color: &ResolvedColor) {
    buf.push('{');
    if let Some(color_space) = &color.color_space {
        let _ = write!(buf, "\"colorSpace\":{},", json_escape(color_space));
    }
    let _ = write!(
        buf,
        "\"rgba\":[{:.6},{:.6},{:.6},{:.6}]}}",
        color.rgba[0], color.rgba[1], color.rgba[2], color.rgba[3]
    );
}

fn write_glyph_transforms(buf: &mut String, transforms: &[GlyphTransform]) {
    buf.push('[');
    for (idx, transform) in transforms.iter().enumerate() {
        if idx > 0 {
            buf.push(',');
        }
        let _ = write!(
            buf,
            "{{\"xx\":{:.6},\"xy\":{:.6},\"yx\":{:.6},\"yy\":{:.6},\"tx\":{:.6},\"ty\":{:.6}}}",
            transform.xx, transform.xy, transform.yx, transform.yy, transform.tx, transform.ty
        );
    }
    buf.push(']');
}

fn write_glyph_run_diagnostics(buf: &mut String, diagnostics: &GlyphRunDiagnostics) {
    let _ = write!(
        buf,
        "{{\"quality\":{},\"replayEligibility\":{},\"strictVisualEligible\":{},\"maxOriginDeltaPx\":{:.6},\"maxAdvanceDeltaPx\":{:.6},\"maxResidualAfterAdjustmentPx\":{:.6},\"clusterMismatchCount\":{},\"missingGlyphCount\":{},\"usedFallbackFontCount\":{}",
        json_escape(diagnostics.quality.as_str()),
        json_escape(diagnostics.replay_eligibility.as_str()),
        diagnostics.strict_visual_eligible,
        diagnostics.max_origin_delta_px,
        diagnostics.max_advance_delta_px,
        diagnostics.max_residual_after_adjustment_px,
        diagnostics.cluster_mismatch_count,
        diagnostics.missing_glyph_count,
        diagnostics.used_fallback_font_count,
    );
    if let Some(reason) = &diagnostics.reason {
        let _ = write!(buf, ",\"reason\":{}", json_escape(reason));
    }
    buf.push('}');
}

fn write_text_decoration(buf: &mut String, kind: TextDecorationKind, run: &TextRunNode) {
    let (color, shape, underline, emphasis_dot) = match kind {
        TextDecorationKind::Underline => (
            if run.style.underline_color != 0 {
                run.style.underline_color
            } else {
                run.style.color
            },
            run.style.underline_shape,
            run.style.underline,
            0,
        ),
        TextDecorationKind::Strikethrough => (
            if run.style.strike_color != 0 {
                run.style.strike_color
            } else {
                run.style.color
            },
            run.style.strike_shape,
            UnderlineType::None,
            0,
        ),
        TextDecorationKind::EmphasisDot => (
            run.style.color,
            0,
            UnderlineType::None,
            run.style.emphasis_dot,
        ),
    };
    let _ = write!(
        buf,
        "{{\"kind\":{},\"baseline\":{:.3},\"rotation\":{:.3},\"fontSize\":{:.3},\"ratio\":{:.6},\"color\":{},\"shape\":{},\"underline\":{},\"emphasisDot\":{},\"positions\":",
        json_escape(kind.as_str()),
        run.baseline,
        run.rotation,
        run.style.font_size,
        run.style.ratio,
        json_escape(&color_ref_to_css(color)),
        shape,
        json_escape(underline_type_str(underline)),
        emphasis_dot,
    );
    write_text_positions(buf, run);
    buf.push('}');
}

fn write_shape_style(buf: &mut String, style: &ShapeStyle) {
    buf.push('{');
    if let Some(color) = style.fill_color {
        let _ = write!(
            buf,
            "\"fillColor\":{}",
            json_escape(&color_ref_to_css(color))
        );
    } else {
        buf.push_str("\"fillColor\":null");
    }
    if let Some(pattern) = &style.pattern {
        buf.push_str(",\"pattern\":");
        write_pattern_fill(buf, pattern);
    }
    if let Some(color) = style.stroke_color {
        let _ = write!(
            buf,
            ",\"strokeColor\":{}",
            json_escape(&color_ref_to_css(color))
        );
    } else {
        buf.push_str(",\"strokeColor\":null");
    }
    let _ = write!(
        buf,
        ",\"strokeWidth\":{:.3},\"strokeDash\":{},\"opacity\":{:.3}",
        style.stroke_width,
        json_escape(stroke_dash_str(style.stroke_dash)),
        style.opacity,
    );
    if let Some(shadow) = &style.shadow {
        buf.push_str(",\"shadow\":");
        write_shadow_style(buf, shadow);
    }
    buf.push('}');
}

fn write_pattern_fill(buf: &mut String, pattern: &PatternFillInfo) {
    let _ = write!(
        buf,
        "{{\"patternType\":{},\"patternColor\":{},\"backgroundColor\":{}}}",
        pattern.pattern_type,
        json_escape(&color_ref_to_css(pattern.pattern_color)),
        json_escape(&color_ref_to_css(pattern.background_color)),
    );
}

fn write_shadow_style(buf: &mut String, shadow: &ShadowStyle) {
    let _ = write!(
        buf,
        "{{\"shadowType\":{},\"color\":{},\"offsetX\":{:.3},\"offsetY\":{:.3},\"alpha\":{}}}",
        shadow.shadow_type,
        json_escape(&color_ref_to_css(shadow.color)),
        shadow.offset_x,
        shadow.offset_y,
        shadow.alpha,
    );
}

fn write_gradient(buf: &mut String, gradient: &GradientFillInfo) {
    buf.push('{');
    let _ = write!(
        buf,
        "\"gradientType\":{},\"angle\":{},\"centerX\":{},\"centerY\":{},\"colors\":[",
        gradient.gradient_type, gradient.angle, gradient.center_x, gradient.center_y,
    );
    for (idx, color) in gradient.colors.iter().enumerate() {
        if idx > 0 {
            buf.push(',');
        }
        let css = color_ref_to_css(*color);
        buf.push_str(&json_escape(&css));
    }
    buf.push_str("],\"positions\":[");
    for (idx, position) in gradient.positions.iter().enumerate() {
        if idx > 0 {
            buf.push(',');
        }
        let _ = write!(buf, "{:.3}", position);
    }
    buf.push_str("]}");
}

fn write_line_style(buf: &mut String, style: &LineStyle) {
    let _ = write!(
        buf,
        "{{\"color\":{},\"width\":{:.3},\"dash\":{},\"lineType\":{},\"startArrow\":{},\"endArrow\":{},\"startArrowSize\":{},\"endArrowSize\":{}}}",
        json_escape(&color_ref_to_css(style.color)),
        style.width,
        json_escape(stroke_dash_str(style.dash)),
        json_escape(line_render_type_str(style.line_type)),
        json_escape(arrow_style_str(style.start_arrow)),
        json_escape(arrow_style_str(style.end_arrow)),
        style.start_arrow_size,
        style.end_arrow_size,
    );
}

fn write_transform(buf: &mut String, transform: ShapeTransform) {
    let _ = write!(
        buf,
        "{{\"rotation\":{:.3},\"horzFlip\":{},\"vertFlip\":{}}}",
        transform.rotation, transform.horz_flip, transform.vert_flip
    );
}

fn write_path_commands(buf: &mut String, commands: &[PathCommand]) {
    buf.push('[');
    for (idx, command) in commands.iter().enumerate() {
        if idx > 0 {
            buf.push(',');
        }
        match command {
            PathCommand::MoveTo(x, y) => {
                let _ = write!(buf, "{{\"type\":\"moveTo\",\"x\":{:.3},\"y\":{:.3}}}", x, y);
            }
            PathCommand::LineTo(x, y) => {
                let _ = write!(buf, "{{\"type\":\"lineTo\",\"x\":{:.3},\"y\":{:.3}}}", x, y);
            }
            PathCommand::CurveTo(x1, y1, x2, y2, x3, y3) => {
                let _ = write!(
                    buf,
                    "{{\"type\":\"curveTo\",\"x1\":{:.3},\"y1\":{:.3},\"x2\":{:.3},\"y2\":{:.3},\"x3\":{:.3},\"y3\":{:.3}}}",
                    x1, y1, x2, y2, x3, y3
                );
            }
            PathCommand::ArcTo(rx, ry, rotation, large_arc, sweep, x, y) => {
                let _ = write!(
                    buf,
                    "{{\"type\":\"arcTo\",\"rx\":{:.3},\"ry\":{:.3},\"rotation\":{:.3},\"largeArc\":{},\"sweep\":{},\"x\":{:.3},\"y\":{:.3}}}",
                    rx, ry, rotation, large_arc, sweep, x, y
                );
            }
            PathCommand::ClosePath => buf.push_str("{\"type\":\"closePath\"}"),
        }
    }
    buf.push(']');
}

fn underline_type_str(value: UnderlineType) -> &'static str {
    match value {
        UnderlineType::None => "none",
        UnderlineType::Bottom => "bottom",
        UnderlineType::Top => "top",
    }
}

fn stroke_dash_str(value: StrokeDash) -> &'static str {
    match value {
        StrokeDash::Solid => "solid",
        StrokeDash::Dash => "dash",
        StrokeDash::Dot => "dot",
        StrokeDash::DashDot => "dashDot",
        StrokeDash::DashDotDot => "dashDotDot",
    }
}

fn line_render_type_str(value: LineRenderType) -> &'static str {
    match value {
        LineRenderType::Single => "single",
        LineRenderType::Double => "double",
        LineRenderType::ThinThickDouble => "thinThickDouble",
        LineRenderType::ThickThinDouble => "thickThinDouble",
        LineRenderType::ThinThickThinTriple => "thinThickThinTriple",
    }
}

fn arrow_style_str(value: ArrowStyle) -> &'static str {
    match value {
        ArrowStyle::None => "none",
        ArrowStyle::Arrow => "arrow",
        ArrowStyle::ConcaveArrow => "concaveArrow",
        ArrowStyle::OpenDiamond => "openDiamond",
        ArrowStyle::OpenCircle => "openCircle",
        ArrowStyle::OpenSquare => "openSquare",
        ArrowStyle::Diamond => "diamond",
        ArrowStyle::Circle => "circle",
        ArrowStyle::Square => "square",
    }
}

fn image_fill_mode_str(value: ImageFillMode) -> &'static str {
    match value {
        ImageFillMode::TileAll => "tileAll",
        ImageFillMode::TileHorzTop => "tileHorzTop",
        ImageFillMode::TileHorzBottom => "tileHorzBottom",
        ImageFillMode::TileVertLeft => "tileVertLeft",
        ImageFillMode::TileVertRight => "tileVertRight",
        ImageFillMode::FitToSize => "fitToSize",
        ImageFillMode::Center => "center",
        ImageFillMode::CenterTop => "centerTop",
        ImageFillMode::CenterBottom => "centerBottom",
        ImageFillMode::LeftCenter => "leftCenter",
        ImageFillMode::LeftTop => "leftTop",
        ImageFillMode::LeftBottom => "leftBottom",
        ImageFillMode::RightCenter => "rightCenter",
        ImageFillMode::RightTop => "rightTop",
        ImageFillMode::RightBottom => "rightBottom",
        ImageFillMode::None => "none",
    }
}

fn image_effect_str(value: ImageEffect) -> &'static str {
    match value {
        ImageEffect::RealPic => "realPic",
        ImageEffect::GrayScale => "grayScale",
        ImageEffect::BlackWhite => "blackWhite",
        ImageEffect::Pattern8x8 => "pattern8x8",
    }
}

fn text_wrap_str(value: crate::model::shape::TextWrap) -> &'static str {
    use crate::model::shape::TextWrap;
    match value {
        TextWrap::Square => "square",
        TextWrap::Tight => "tight",
        TextWrap::Through => "through",
        TextWrap::TopAndBottom => "topAndBottom",
        TextWrap::BehindText => "behindText",
        TextWrap::InFrontOfText => "inFrontOfText",
    }
}

fn write_render_layer_info(buf: &mut String, layer: RenderLayerInfo) {
    buf.push('{');
    if let Some(text_wrap) = layer.text_wrap {
        let _ = write!(
            buf,
            "\"textWrap\":{}",
            json_escape(text_wrap_str(text_wrap))
        );
    } else {
        buf.push_str("\"textWrap\":null");
    }
    let _ = write!(
        buf,
        ",\"zOrder\":{},\"stableIndex\":{}",
        layer.z_order, layer.stable_index
    );
    buf.push('}');
}

fn render_profile_str(value: RenderProfile) -> &'static str {
    match value {
        RenderProfile::FastPreview => "fastPreview",
        RenderProfile::Screen => "screen",
        RenderProfile::Print => "print",
        RenderProfile::HighQuality => "highQuality",
    }
}

fn form_type_str(value: FormType) -> &'static str {
    match value {
        FormType::PushButton => "pushButton",
        FormType::CheckBox => "checkBox",
        FormType::RadioButton => "radioButton",
        FormType::ComboBox => "comboBox",
        FormType::Edit => "edit",
    }
}

fn json_escape(value: &str) -> String {
    format!("\"{}\"", raw_json_escape(value))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::shape::TextWrap;
    use crate::paint::{
        BitmapGlyphFiltering, BitmapGlyphPayload, BitmapGlyphScalingPolicy, CacheHint, ClipKind,
        ColorGlyphFormat, ColorLayersPayload, ColorPaintGraphNode, ColorPaintGraphNodeKind,
        ColorPaintGraphPayload, ColorPaintSolidPathNode, FontColorGlyphRef, FontFaceKey,
        FontFallbackPolicyId, FontInstanceKey, GlyphCluster, GlyphOutlineFillRule,
        GlyphOutlinePayloadKind, GlyphOutlineStrokeCap, GlyphOutlineStrokeJoin,
        GlyphOutlineStrokeStyle, GlyphRange, GlyphRunDiagnostics, GlyphRunOrientation,
        GlyphRunReplayEligibility, GroupKind, ImageResourceId, LayerAffineTransform,
        LayerGlyphOutlinePaint, LayerGlyphOutlinePath, LayerGlyphRunPaint, LayerNode, LayerPoint,
        LayerVector, PageLayerTree, PaintTextStyle, PaintVariantMeta, ResolvedColor, ScriptTag,
        ShapeKey, ShapingEngineId, SvgGlyphPayload, SvgResourceId, TextDecorationKind,
        TextDirection, TextSourceId, TextSourceRange, TextSourceSpan, TextVariantKind,
        TextVariantQuality, WritingMode,
    };
    use crate::renderer::composer::CharOverlapInfo;
    use crate::renderer::equation::layout::{LayoutBox, LayoutKind};
    use crate::renderer::render_tree::{
        EquationNode, FieldMarkerType, ImageNode, PathNode, PlaceholderNode, RawSvgNode,
        RenderLayerInfo, TextRunNode,
    };
    use serde_json::Value;

    #[test]
    fn serializes_text_and_shape_ops_for_browser_replay() {
        let text = PaintOp::TextRun {
            bbox: BoundingBox::new(10.0, 20.0, 80.0, 18.0),
            run: TextRunNode {
                text: "가A".to_string(),
                style: TextStyle {
                    font_family: "Noto Sans KR".to_string(),
                    font_size: 16.0,
                    color: 0x00010203,
                    bold: true,
                    italic: true,
                    underline: UnderlineType::Bottom,
                    shade_color: 0x0000FFFF,
                    emphasis_dot: 2,
                    ..Default::default()
                },
                char_shape_id: None,
                para_shape_id: None,
                section_index: None,
                para_index: None,
                char_start: None,
                cell_context: None,
                is_para_end: true,
                is_line_break_end: true,
                rotation: 0.0,
                is_vertical: false,
                char_overlap: Some(CharOverlapInfo {
                    border_type: 1,
                    inner_char_size: 90,
                }),
                border_fill_id: 0,
                baseline: 13.0,
                field_marker: FieldMarkerType::FieldBegin,
            },
        };
        let rect = PaintOp::Rectangle {
            bbox: BoundingBox::new(8.0, 18.0, 84.0, 22.0),
            rect: crate::renderer::render_tree::RectangleNode::new(
                4.0,
                ShapeStyle {
                    fill_color: Some(0x00F0F1F2),
                    stroke_color: Some(0x00030405),
                    stroke_width: 1.5,
                    ..Default::default()
                },
                None,
            ),
        };

        let tree = PageLayerTree::new(
            120.0,
            80.0,
            LayerNode::leaf(
                BoundingBox::new(0.0, 0.0, 120.0, 80.0),
                None,
                vec![text, rect],
            ),
        );

        let json = tree.to_json();
        let positions = compute_char_positions(
            "가A",
            &TextStyle {
                font_family: "Noto Sans KR".to_string(),
                font_size: 16.0,
                color: 0x00010203,
                bold: true,
                italic: true,
                underline: UnderlineType::Bottom,
                shade_color: 0x0000FFFF,
                emphasis_dot: 2,
                ..Default::default()
            },
        );
        let positions_json = format!(
            "\"positions\":[{:.3},{:.3},{:.3}]",
            positions[0], positions[1], positions[2]
        );

        assert!(json.contains("\"kind\":\"leaf\""));
        assert!(json.contains(&format!(
            "\"schemaVersion\":{}",
            LAYER_TREE_SCHEMA.schema_version
        )));
        assert!(json.contains(&format!(
            "\"schemaMinorVersion\":{}",
            LAYER_TREE_SCHEMA.schema_minor_version
        )));
        assert!(json.contains(&format!(
            "\"schema\":{{\"major\":{},\"minor\":{}}}",
            LAYER_TREE_SCHEMA.schema_version, LAYER_TREE_SCHEMA.schema_minor_version
        )));
        assert!(json.contains("\"resourceTableVersion\":1"));
        assert!(json.contains("\"resourceTableMinorVersion\":4"));
        assert!(json.contains("\"resourceTable\":{\"major\":1,\"minor\":4}"));
        assert!(json.contains("\"unit\":\"px\""));
        assert!(json.contains("\"coordinateSystem\":\"page-top-left-y-down\""));
        assert!(json.contains("\"profile\":\"screen\""));
        assert!(json.contains("\"buildOptions\":{"));
        assert!(json.contains("\"debugOptions\":{"));
        assert!(json.contains("\"outputOptions\":{"));
        assert!(json.contains("\"clipEnabled\":true"));
        assert!(json.contains("\"type\":\"textRun\""));
        assert!(json.contains("\"textSources\":[{\"id\":0,\"text\":\"가A\""));
        assert!(json.contains("\"source\":{\"id\":0"));
        assert!(json.contains("\"paintStyle\":{"));
        assert!(json.contains("\"placement\":{\"runToPage\":"));
        assert!(json.contains("\"clusterBasis\":\"legacyPosition\""));
        assert!(json.contains("\"clusters\":[{\"sourceRangeUtf8\""));
        assert!(json.contains("\"legacyVisuals\":{"));
        assert!(json.contains("\"layer.optionMetadata\""));
        assert!(json.contains(&positions_json));
        assert!(!json.contains("\"displayText\""));
        assert!(!json.contains("\"displayPositions\""));
        assert!(json.contains("\"isParaEnd\":true"));
        assert!(json.contains("\"isLineBreakEnd\":true"));
        assert!(json.contains("\"fieldMarker\":{\"kind\":\"fieldBegin\"}"));
        assert!(json.contains("\"charOverlap\":{\"borderType\":1,\"innerCharSize\":90}"));
        assert!(json.contains("\"usedFeatures\":[\"text.paintStyle\""));
        assert!(json.contains("\"text.v2.diagnostics\""));
        assert!(json.contains("\"knownFeatures\":[\"fontResources\""));
        assert!(json.contains("\"fontResources\":{\"blobs\":[],\"faces\":[]}"));
        assert!(json.contains("\"textV2\":{\"compatibilityProfile\":\"v1Compat\""));
        assert!(json.contains("\"downgradePath\":\"schemaV1FlattenedTextRunAndGlyphRun\""));
        assert!(json.contains("\"text\":{\"defaultVariant\":\"textRun\""));
        assert!(json.contains("\"fontFamily\":\"Noto Sans KR\""));
        assert!(json.contains("\"italic\":true"));
        assert!(json.contains("\"shadeColor\":\"#ffff00\""));
        assert!(json.contains("\"emphasisDot\":2"));
        assert!(json.contains("\"type\":\"rectangle\""));
        assert!(json.contains("\"cornerRadius\":4.000"));
    }

    #[test]
    fn serializes_display_text_for_pua_text_run() {
        let style = TextStyle {
            font_family: "Noto Sans KR".to_string(),
            font_size: 16.0,
            ..Default::default()
        };
        let text = "\u{F012B}(Signature)";
        let display_text = "(인)(Signature)";
        let source_positions = compute_char_positions(text, &style);
        let display_positions = compute_char_positions(display_text, &style);
        let text_run = PaintOp::TextRun {
            bbox: BoundingBox::new(10.0, 20.0, 80.0, 18.0),
            run: TextRunNode {
                text: text.to_string(),
                style: style.clone(),
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
                baseline: 13.0,
                field_marker: FieldMarkerType::None,
            },
        };
        let tree = PageLayerTree::new(
            120.0,
            80.0,
            LayerNode::leaf(
                BoundingBox::new(0.0, 0.0, 120.0, 80.0),
                None,
                vec![text_run],
            ),
        );

        let json = tree.to_json();
        let source_positions_json = format!(
            "\"positions\":[{}]",
            source_positions
                .iter()
                .map(|position| format!("{:.3}", position))
                .collect::<Vec<_>>()
                .join(",")
        );
        let display_positions_json = format!(
            "\"displayPositions\":[{}]",
            display_positions
                .iter()
                .map(|position| format!("{:.3}", position))
                .collect::<Vec<_>>()
                .join(",")
        );

        assert!(json.contains(&format!("\"text\":\"{}\"", text)));
        assert!(json.contains(&format!("\"displayText\":\"{}\"", display_text)));
        assert!(json.contains(&source_positions_json));
        assert!(json.contains(&display_positions_json));
        assert!(json.contains("\"text.displayText\""));
    }

    #[test]
    fn serializes_empty_display_positions_for_hidden_pua_filler() {
        let text_run = PaintOp::TextRun {
            bbox: BoundingBox::new(10.0, 20.0, 80.0, 18.0),
            run: TextRunNode {
                text: "\u{F081C}".to_string(),
                style: TextStyle {
                    font_family: "Noto Sans KR".to_string(),
                    font_size: 16.0,
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
                baseline: 13.0,
                field_marker: FieldMarkerType::None,
            },
        };
        let tree = PageLayerTree::new(
            120.0,
            80.0,
            LayerNode::leaf(
                BoundingBox::new(0.0, 0.0, 120.0, 80.0),
                None,
                vec![text_run],
            ),
        );

        let json = tree.to_json();

        assert!(json.contains("\"displayText\":\"\""));
        assert!(json.contains("\"displayPositions\":[]"));
    }

    #[test]
    fn serializes_external_text_visual_ops_as_additive_features() {
        let run = TextRunNode {
            text: "A\tB".to_string(),
            style: TextStyle {
                font_family: "Noto Sans".to_string(),
                font_size: 14.0,
                color: 0x00000000,
                underline: UnderlineType::Bottom,
                strikethrough: true,
                emphasis_dot: 1,
                tab_leaders: vec![TabLeaderInfo {
                    start_x: 10.0,
                    end_x: 40.0,
                    fill_type: 3,
                }],
                ..Default::default()
            },
            char_shape_id: None,
            para_shape_id: None,
            section_index: Some(1),
            para_index: Some(2),
            char_start: Some(3),
            cell_context: None,
            is_para_end: true,
            is_line_break_end: false,
            rotation: 0.0,
            is_vertical: false,
            char_overlap: Some(CharOverlapInfo {
                border_type: 2,
                inner_char_size: 80,
            }),
            border_fill_id: 0,
            baseline: 11.0,
            field_marker: FieldMarkerType::FieldEnd,
        };
        let bbox = BoundingBox::new(10.0, 20.0, 40.0, 16.0);
        let tree = PageLayerTree::new(
            120.0,
            80.0,
            LayerNode::leaf(
                BoundingBox::new(0.0, 0.0, 120.0, 80.0),
                None,
                vec![
                    PaintOp::TextRun {
                        bbox,
                        run: run.clone(),
                    },
                    PaintOp::CharOverlap {
                        bbox,
                        run: run.clone(),
                    },
                    PaintOp::TextControlMark {
                        bbox,
                        run: run.clone(),
                    },
                    PaintOp::TabLeader {
                        bbox,
                        run: run.clone(),
                    },
                    PaintOp::TextDecoration {
                        bbox,
                        run: run.clone(),
                        kind: TextDecorationKind::Underline,
                    },
                    PaintOp::TextDecoration {
                        bbox,
                        run,
                        kind: TextDecorationKind::EmphasisDot,
                    },
                ],
            ),
        );

        let json = tree.to_json();

        assert!(json.contains("\"type\":\"charOverlap\""));
        assert!(json.contains("\"type\":\"textControlMark\""));
        assert!(json.contains("\"type\":\"tabLeader\""));
        assert!(json.contains("\"type\":\"textDecoration\""));
        assert!(json.contains("\"kind\":\"underline\""));
        assert!(json.contains("\"kind\":\"emphasisDot\""));
        assert!(json.contains("\"textSources\":[{\"id\":0,\"text\":\"A\\tB\""));
        assert!(json.contains("\"stableSourceKey\":\"section:1/para:2/char:3\""));
        assert!(json.contains("\"marker\":\"fieldEnd\""));
        assert!(json.contains("\"text.charOverlapOp\""));
        assert!(json.contains("\"text.controlMarkOp\""));
        assert!(json.contains("\"text.tabLeaderOp\""));
        assert!(json.contains("\"text.decorationOp\""));
        assert!(json.contains("\"externalizedVisuals\":[\"charOverlap\",\"controlMarks\",\"tabLeaders\",\"decorations\"]"));
        assert!(json.contains("\"legacyVisuals\":{\"charOverlap\":\"mirror\""));
    }

    #[test]
    fn serializes_optional_glyph_run_variant_with_text_run_fallback() {
        let source = TextSourceSpan {
            id: TextSourceId(0),
            utf8_range: TextSourceRange::new(0, 1),
            utf16_range: TextSourceRange::new(0, 1),
            stable_source_key: None,
        };
        let shape_key = ShapeKey {
            font_instance: FontInstanceKey {
                face_key: FontFaceKey("face-0".to_string()),
                size_px: 12.0,
                variations: Vec::new(),
                synthetic_bold: false,
                synthetic_italic: false,
            },
            direction: TextDirection::Ltr,
            writing_mode: WritingMode::HorizontalTb,
            script: Some(ScriptTag("DFLT".to_string())),
            language: None,
            features: Vec::new(),
            shaping_engine: ShapingEngineId("test".to_string()),
            fallback_policy: FontFallbackPolicyId("none".to_string()),
        };
        let text_run = PaintOp::TextRun {
            bbox: BoundingBox::new(0.0, 0.0, 20.0, 20.0),
            run: TextRunNode {
                text: "A".to_string(),
                style: TextStyle {
                    font_family: "Test".to_string(),
                    font_size: 12.0,
                    shade_color: 0x00FF_FFFF,
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
                baseline: 12.0,
                field_marker: FieldMarkerType::None,
            },
        };
        let glyph_run = PaintOp::GlyphRun {
            bbox: BoundingBox::new(0.0, 0.0, 20.0, 20.0),
            run: Box::new(LayerGlyphRunPaint {
                source,
                variant: PaintVariantMeta {
                    equivalence_group: "text-0".to_string(),
                    variant_id: "glyphRun".to_string(),
                    variant_kind: TextVariantKind::GlyphRun,
                    part_index: 0,
                    part_count: 1,
                    is_default_fallback: false,
                    requires: vec!["fontResources".to_string(), "text.glyphRun".to_string()],
                    quality: Some(TextVariantQuality::Exact),
                    anchor_op_id: None,
                    local_paint_order: None,
                },
                paint_style: PaintTextStyle::from(&TextStyle {
                    font_family: "Test".to_string(),
                    font_size: 12.0,
                    shade_color: 0x00FF_FFFF,
                    ..Default::default()
                }),
                shape_key,
                placement: crate::paint::TextRunPlacement {
                    run_to_page: LayerAffineTransform {
                        a: 1.0,
                        b: 0.0,
                        c: 0.0,
                        d: 1.0,
                        e: 0.0,
                        f: 12.0,
                    },
                    baseline_y: 0.0,
                },
                glyph_ids: vec![42],
                positions: vec![LayerPoint { x: 0.0, y: 0.0 }],
                advances: Some(vec![LayerVector { dx: 12.0, dy: 0.0 }]),
                clusters: vec![GlyphCluster {
                    source_range_utf8: TextSourceRange::new(0, 1),
                    source_range_utf16: Some(TextSourceRange::new(0, 1)),
                    text_range_utf8: Some(TextSourceRange::new(0, 1)),
                    glyph_range: GlyphRange::new(0, 1),
                    flags: Vec::new(),
                }],
                direction: TextDirection::Ltr,
                bidi_level: None,
                writing_mode: WritingMode::HorizontalTb,
                orientation: GlyphRunOrientation::Horizontal,
                glyph_transforms: None,
                diagnostics: GlyphRunDiagnostics {
                    quality: TextVariantQuality::Exact,
                    replay_eligibility: GlyphRunReplayEligibility::Portable,
                    strict_visual_eligible: true,
                    max_origin_delta_px: 0.0,
                    max_advance_delta_px: 0.0,
                    max_residual_after_adjustment_px: 0.0,
                    cluster_mismatch_count: 0,
                    missing_glyph_count: 0,
                    used_fallback_font_count: 0,
                    reason: None,
                },
            }),
        };

        let tree = PageLayerTree::new(
            120.0,
            80.0,
            LayerNode::leaf(
                BoundingBox::new(0.0, 0.0, 120.0, 80.0),
                None,
                vec![text_run, glyph_run],
            ),
        );
        let json = tree.to_json();

        assert!(json.contains("\"type\":\"glyphRun\""));
        assert!(json.contains("\"fontResources\":{\"blobs\":[],\"faces\":[]}"));
        assert!(json.contains("\"optionalFeatures\":[\"fontResources\",\"text.glyphRun\"]"));
        assert!(json.contains("\"variants\":[\"textRun\",\"glyphRun\"]"));
        assert!(
            json.contains("\"variant\":{\"equivalenceGroup\":\"text-0\",\"variantId\":\"textRun\"")
        );
        assert!(json.contains("\"variantId\":\"glyphRun\""));
        assert!(json.contains("\"glyphIds\":[42]"));
        assert!(json.contains("\"replayEligibility\":\"portable\""));
        assert!(json.contains("\"strictVisualEligible\":true"));
        assert!(json.contains("\"slotDiagnostics\":[{\"paintOrderSlotId\":\"text-0\""));
        assert!(json.contains("\"strictVariantAvailable\":true"));
    }

    #[test]
    fn serializes_glyph_outline_variant_with_strict_sidecar_contract() {
        let source = TextSourceSpan {
            id: TextSourceId(0),
            utf8_range: TextSourceRange::new(0, 1),
            utf16_range: TextSourceRange::new(0, 1),
            stable_source_key: None,
        };
        let text_run = PaintOp::TextRun {
            bbox: BoundingBox::new(0.0, 0.0, 20.0, 20.0),
            run: TextRunNode {
                text: "A".to_string(),
                style: TextStyle {
                    font_family: "Test".to_string(),
                    font_size: 12.0,
                    shade_color: 0x00FF_FFFF,
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
                baseline: 12.0,
                field_marker: FieldMarkerType::None,
            },
        };
        let outline = PaintOp::GlyphOutline {
            bbox: BoundingBox::new(0.0, 0.0, 20.0, 20.0),
            outline: Box::new(LayerGlyphOutlinePaint {
                source,
                variant: PaintVariantMeta {
                    equivalence_group: "text-0".to_string(),
                    variant_id: "glyphOutline".to_string(),
                    variant_kind: TextVariantKind::GlyphOutline,
                    part_index: 0,
                    part_count: 1,
                    is_default_fallback: false,
                    requires: vec![
                        "text.glyphOutline".to_string(),
                        "text.glyphOutline.strictSidecar".to_string(),
                    ],
                    quality: Some(TextVariantQuality::Exact),
                    anchor_op_id: Some("text-0".to_string()),
                    local_paint_order: Some(0),
                },
                payload_kind: GlyphOutlinePayloadKind::MonochromeFillStroke,
                color_layers: None,
                bitmap_glyph: None,
                svg_glyph: None,
                paint_style: PaintTextStyle::from(&TextStyle {
                    font_family: "Test".to_string(),
                    font_size: 12.0,
                    shade_color: 0x00FF_FFFF,
                    ..Default::default()
                }),
                placement: crate::paint::TextRunPlacement {
                    run_to_page: LayerAffineTransform {
                        a: 1.0,
                        b: 0.0,
                        c: 0.0,
                        d: 1.0,
                        e: 0.0,
                        f: 12.0,
                    },
                    baseline_y: 0.0,
                },
                paths: vec![LayerGlyphOutlinePath {
                    glyph_id: 42,
                    source_range_utf8: TextSourceRange::new(0, 1),
                    glyph_range: GlyphRange::new(0, 1),
                    commands: vec![
                        PathCommand::MoveTo(0.0, 0.0),
                        PathCommand::LineTo(10.0, 0.0),
                        PathCommand::ClosePath,
                    ],
                    fill_rule: GlyphOutlineFillRule::NonZero,
                }],
                stroke: Some(GlyphOutlineStrokeStyle {
                    color: 0x00000000,
                    width: 1.0,
                    join: GlyphOutlineStrokeJoin::Miter,
                    cap: GlyphOutlineStrokeCap::Butt,
                    miter_limit: 2.0,
                    paint_order: crate::paint::GlyphOutlinePaintOrder::FillThenStroke,
                }),
                diagnostics: GlyphRunDiagnostics {
                    quality: TextVariantQuality::Exact,
                    replay_eligibility: GlyphRunReplayEligibility::Portable,
                    strict_visual_eligible: true,
                    max_origin_delta_px: 0.0,
                    max_advance_delta_px: 0.0,
                    max_residual_after_adjustment_px: 0.0,
                    cluster_mismatch_count: 0,
                    missing_glyph_count: 0,
                    used_fallback_font_count: 0,
                    reason: None,
                },
            }),
        };

        let tree = PageLayerTree::new(
            120.0,
            80.0,
            LayerNode::leaf(
                BoundingBox::new(0.0, 0.0, 120.0, 80.0),
                None,
                vec![text_run, outline],
            ),
        );
        let json = tree.to_json();

        assert!(json.contains("\"type\":\"glyphOutline\""));
        assert!(json.contains("\"payloadKind\":\"monochromeFillStroke\""));
        assert!(json.contains("\"anchorOpId\":\"text-0\""));
        assert!(json.contains("\"paths\":[{\"glyphId\":42"));
        assert!(json.contains("\"fillRule\":\"nonzero\""));
        assert!(json.contains("\"stroke\":{\"color\":\"#000000\""));
        assert!(json.contains("\"strictSubset\":true"));
        assert!(json.contains("\"text.glyphOutline\""));
        assert!(json.contains("\"text.glyphOutline.strictSidecar\""));
        assert!(json.contains("\"variants\":[\"textRun\",\"glyphOutline\"]"));
        assert!(json.contains("\"variantKind\":\"glyphOutline\""));
    }

    #[test]
    fn serializes_advanced_glyph_outline_payload_gate_metadata() {
        let outline = PaintOp::GlyphOutline {
            bbox: BoundingBox::new(0.0, 0.0, 20.0, 20.0),
            outline: Box::new(LayerGlyphOutlinePaint {
                source: TextSourceSpan {
                    id: TextSourceId(0),
                    utf8_range: TextSourceRange::new(0, 1),
                    utf16_range: TextSourceRange::new(0, 1),
                    stable_source_key: None,
                },
                variant: PaintVariantMeta {
                    equivalence_group: "text-0".to_string(),
                    variant_id: "glyphOutlineColor".to_string(),
                    variant_kind: TextVariantKind::GlyphOutline,
                    part_index: 0,
                    part_count: 1,
                    is_default_fallback: false,
                    requires: vec![
                        "text.glyphOutline".to_string(),
                        "text.glyphOutline.colorLayers".to_string(),
                        "text.glyphOutline.colorLayers.colrV1".to_string(),
                    ],
                    quality: Some(TextVariantQuality::Exact),
                    anchor_op_id: Some("text-0".to_string()),
                    local_paint_order: Some(0),
                },
                payload_kind: GlyphOutlinePayloadKind::ColorLayers,
                color_layers: Some(ColorLayersPayload {
                    color_format: ColorGlyphFormat::ColrV1,
                    source_font_ref: Some(FontColorGlyphRef {
                        face_key: Some("fixture:resource:face".to_string()),
                        glyph_id: Some(42),
                        palette_index: Some(0),
                        color_format: Some(ColorGlyphFormat::ColrV1),
                    }),
                    palette_ref: None,
                    layers: Vec::new(),
                    paint_graph: Some(ColorPaintGraphPayload {
                        root_node_id: 0,
                        nodes: vec![ColorPaintGraphNode {
                            node_id: 0,
                            kind: ColorPaintGraphNodeKind::SolidPath,
                            solid_path: Some(ColorPaintSolidPathNode {
                                commands: vec![
                                    PathCommand::MoveTo(0.0, 0.0),
                                    PathCommand::LineTo(10.0, 0.0),
                                    PathCommand::ClosePath,
                                ],
                                fill: ResolvedColor {
                                    color_space: Some("sRGB".to_string()),
                                    rgba: [0.0, 0.0, 0.0, 1.0],
                                },
                                fill_rule: GlyphOutlineFillRule::NonZero,
                                source_glyph_id: Some(42),
                                palette_index: Some(0),
                            }),
                            linear_gradient_path: None,
                            radial_gradient_path: None,
                            sweep_gradient_path: None,
                            transform: None,
                            source_range_utf8: Some(TextSourceRange::new(0, 1)),
                            glyph_range: Some(GlyphRange::new(0, 1)),
                            source_font_ref: Some(FontColorGlyphRef {
                                face_key: Some("fixture:resource:face".to_string()),
                                glyph_id: Some(42),
                                palette_index: Some(0),
                                color_format: Some(ColorGlyphFormat::ColrV1),
                            }),
                        }],
                    }),
                    source_range_utf8: Some(TextSourceRange::new(0, 1)),
                    glyph_range: Some(GlyphRange::new(0, 1)),
                }),
                bitmap_glyph: None,
                svg_glyph: None,
                paint_style: PaintTextStyle::from(&TextStyle {
                    font_family: "Test".to_string(),
                    font_size: 12.0,
                    shade_color: 0x00FF_FFFF,
                    ..Default::default()
                }),
                placement: crate::paint::TextRunPlacement {
                    run_to_page: LayerAffineTransform {
                        a: 1.0,
                        b: 0.0,
                        c: 0.0,
                        d: 1.0,
                        e: 0.0,
                        f: 12.0,
                    },
                    baseline_y: 0.0,
                },
                paths: Vec::new(),
                stroke: None,
                diagnostics: GlyphRunDiagnostics {
                    quality: TextVariantQuality::Exact,
                    replay_eligibility: GlyphRunReplayEligibility::Portable,
                    strict_visual_eligible: true,
                    max_origin_delta_px: 0.0,
                    max_advance_delta_px: 0.0,
                    max_residual_after_adjustment_px: 0.0,
                    cluster_mismatch_count: 0,
                    missing_glyph_count: 0,
                    used_fallback_font_count: 0,
                    reason: None,
                },
            }),
        };

        let tree = PageLayerTree::new(
            120.0,
            80.0,
            LayerNode::leaf(BoundingBox::new(0.0, 0.0, 120.0, 80.0), None, vec![outline]),
        );
        let json = tree.to_json();

        assert!(json.contains("\"payloadKind\":\"colorLayers\""));
        assert!(json.contains("\"payloadResourceKey\":\"glyphPayload:colorLayers"));
        assert!(json.contains("\"colorLayers\":{\"colorFormat\":\"colrV1\""));
        assert!(json.contains("\"kind\":\"solidPath\""));
        assert!(json.contains("\"colrv1Stage1GraphContract\":true"));
        assert!(json.contains("\"text.glyphOutline.colorLayers\""));
        assert!(json.contains("\"text.glyphOutline.colorLayers.colrV1\""));
        assert!(json.contains("\"text.glyphOutline.payloadResourceKey\""));
        assert_eq!(
            json.matches("\"text.glyphOutline.payloadResourceDigestKey\"")
                .count(),
            1,
            "color-layer metadata containing ':resource:' must not advertise a resource digest payload feature"
        );
    }

    #[test]
    fn serializes_bitmap_and_svg_glyph_payload_resource_keys() {
        let source = TextSourceSpan {
            id: TextSourceId(0),
            utf8_range: TextSourceRange::new(0, 1),
            utf16_range: TextSourceRange::new(0, 1),
            stable_source_key: None,
        };
        let diagnostics = GlyphRunDiagnostics {
            quality: TextVariantQuality::Exact,
            replay_eligibility: GlyphRunReplayEligibility::Portable,
            strict_visual_eligible: true,
            max_origin_delta_px: 0.0,
            max_advance_delta_px: 0.0,
            max_residual_after_adjustment_px: 0.0,
            cluster_mismatch_count: 0,
            missing_glyph_count: 0,
            used_fallback_font_count: 0,
            reason: None,
        };
        let placement = crate::paint::TextRunPlacement {
            run_to_page: LayerAffineTransform {
                a: 1.0,
                b: 0.0,
                c: 0.0,
                d: 1.0,
                e: 0.0,
                f: 12.0,
            },
            baseline_y: 0.0,
        };
        let text_style = PaintTextStyle::from(&TextStyle {
            font_family: "Test".to_string(),
            font_size: 12.0,
            shade_color: 0x00FF_FFFF,
            ..Default::default()
        });
        let incomplete_bitmap_outline = LayerGlyphOutlinePaint {
            source: source.clone(),
            variant: PaintVariantMeta {
                equivalence_group: "text-invalid".to_string(),
                variant_id: "bitmapGlyphInvalid".to_string(),
                variant_kind: TextVariantKind::GlyphOutline,
                part_index: 0,
                part_count: 1,
                is_default_fallback: false,
                requires: vec!["text.glyphOutline.bitmapGlyph".to_string()],
                quality: Some(TextVariantQuality::Exact),
                anchor_op_id: Some("text-invalid".to_string()),
                local_paint_order: Some(0),
            },
            payload_kind: GlyphOutlinePayloadKind::BitmapGlyph,
            color_layers: None,
            bitmap_glyph: Some(BitmapGlyphPayload {
                image_ref: ImageResourceId(7),
                source_range_utf8: TextSourceRange::new(0, 1),
                glyph_range: GlyphRange::new(0, 1),
                placement: BoundingBox::new(0.0, 0.0, 10.0, 10.0),
                alpha_premultiplied: true,
                scaling_policy: BitmapGlyphScalingPolicy::BackendDefault,
                filtering: BitmapGlyphFiltering::Linear,
                transform_to_run: None,
            }),
            svg_glyph: None,
            paint_style: text_style.clone(),
            placement,
            paths: Vec::new(),
            stroke: None,
            diagnostics: diagnostics.clone(),
        };
        assert!(!incomplete_bitmap_outline.has_payload_resource_key());
        assert!(incomplete_bitmap_outline.payload_resource_key().is_none());
        let mut resources = ResourceArena::default();
        let invalid_image_id = resources.intern_image_bytes(&[1, 2, 3, 4]);
        let mut invalid_bitmap_with_resource = incomplete_bitmap_outline.clone();
        invalid_bitmap_with_resource
            .bitmap_glyph
            .as_mut()
            .unwrap()
            .image_ref = invalid_image_id;
        assert!(!has_payload_resource_digest_key(
            &invalid_bitmap_with_resource,
            &resources
        ));
        let bitmap_outline = PaintOp::GlyphOutline {
            bbox: BoundingBox::new(0.0, 0.0, 20.0, 20.0),
            outline: Box::new(LayerGlyphOutlinePaint {
                source: source.clone(),
                variant: PaintVariantMeta {
                    equivalence_group: "text-0".to_string(),
                    variant_id: "bitmapGlyph".to_string(),
                    variant_kind: TextVariantKind::GlyphOutline,
                    part_index: 0,
                    part_count: 1,
                    is_default_fallback: false,
                    requires: vec!["text.glyphOutline.bitmapGlyph".to_string()],
                    quality: Some(TextVariantQuality::Exact),
                    anchor_op_id: Some("text-0".to_string()),
                    local_paint_order: Some(0),
                },
                payload_kind: GlyphOutlinePayloadKind::BitmapGlyph,
                color_layers: None,
                bitmap_glyph: Some(BitmapGlyphPayload {
                    image_ref: ImageResourceId(0),
                    source_range_utf8: TextSourceRange::new(0, 1),
                    glyph_range: GlyphRange::new(0, 1),
                    placement: BoundingBox::new(0.0, 0.0, 10.0, 10.0),
                    alpha_premultiplied: true,
                    scaling_policy: BitmapGlyphScalingPolicy::SourceExact,
                    filtering: BitmapGlyphFiltering::Linear,
                    transform_to_run: None,
                }),
                svg_glyph: None,
                paint_style: text_style.clone(),
                placement,
                paths: Vec::new(),
                stroke: None,
                diagnostics: diagnostics.clone(),
            }),
        };
        let svg_outline = PaintOp::GlyphOutline {
            bbox: BoundingBox::new(20.0, 0.0, 20.0, 20.0),
            outline: Box::new(LayerGlyphOutlinePaint {
                source,
                variant: PaintVariantMeta {
                    equivalence_group: "text-1".to_string(),
                    variant_id: "svgGlyph".to_string(),
                    variant_kind: TextVariantKind::GlyphOutline,
                    part_index: 0,
                    part_count: 1,
                    is_default_fallback: false,
                    requires: vec!["text.glyphOutline.svgGlyph".to_string()],
                    quality: Some(TextVariantQuality::Exact),
                    anchor_op_id: Some("text-1".to_string()),
                    local_paint_order: Some(0),
                },
                payload_kind: GlyphOutlinePayloadKind::SvgGlyph,
                color_layers: None,
                bitmap_glyph: None,
                svg_glyph: Some(SvgGlyphPayload {
                    svg_ref: SvgResourceId(0),
                    source_range_utf8: TextSourceRange::new(0, 1),
                    glyph_range: GlyphRange::new(0, 1),
                    view_box: BoundingBox::new(0.0, 0.0, 10.0, 10.0),
                    intrinsic_size: Some(LayerVector { dx: 10.0, dy: 10.0 }),
                    static_sanitized: true,
                    script_allowed: false,
                    animation_allowed: false,
                    external_resources_allowed: false,
                    interactivity_allowed: false,
                    transform_to_run: None,
                }),
                paint_style: text_style,
                placement,
                paths: Vec::new(),
                stroke: None,
                diagnostics,
            }),
        };
        let mut tree = PageLayerTree::new(
            120.0,
            80.0,
            LayerNode::leaf(
                BoundingBox::new(0.0, 0.0, 120.0, 80.0),
                None,
                vec![bitmap_outline, svg_outline],
            ),
        );
        let image_bytes = [0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a];
        let svg_fragment = "<path d=\"M0 0H10V10Z\"/>";
        let image_id = tree.resources.intern_image_bytes(&image_bytes);
        let svg_id = tree.resources.intern_svg_fragment(svg_fragment);
        assert_eq!(image_id, ImageResourceId(0));
        assert_eq!(svg_id, SvgResourceId(0));
        let image_resource_key = tree
            .resources
            .image_resource_key(image_id)
            .unwrap()
            .to_string();
        let svg_resource_key = tree.resources.svg_resource_key(svg_id).unwrap().to_string();

        let json = tree.to_json();

        assert!(json.contains("\"schemaMinorVersion\":17"));
        assert!(json.contains("\"payloadResourceKey\":\"glyphPayload:bitmapGlyph:imageRef:0"));
        assert!(json.contains(&format!(":resource:{image_resource_key}\"")));
        assert!(json.contains("\"payloadResourceKey\":\"glyphPayload:svgGlyph:svgRef:0"));
        assert!(json.contains(&format!(":resource:{svg_resource_key}\"")));
        assert!(json.contains("\"vectorResourceId\":0"));
        assert!(json.contains("\"strictVisualContract\":true"));
        assert!(json.contains("\"staticSanitizedContract\":true"));
        assert!(json.contains("\"text.glyphOutline.payloadResourceKey\""));
        assert!(json.contains("\"text.glyphOutline.payloadResourceDigestKey\""));
        assert!(json.contains("\"text.glyphOutline.svgGlyph.vectorResourceId\""));
    }

    #[test]
    fn known_text_features_are_unique() {
        let mut seen = std::collections::BTreeSet::new();
        for feature in KNOWN_TEXT_FEATURES {
            assert!(seen.insert(*feature), "duplicate known feature: {feature}");
        }
    }

    #[test]
    fn serializes_backend_replay_payload_fields() {
        let mut path = PathNode::new(
            vec![
                PathCommand::MoveTo(0.0, 0.0),
                PathCommand::LineTo(10.0, 10.0),
            ],
            ShapeStyle::default(),
            None,
        );
        path.connector_endpoints = Some((1.0, 2.0, 3.0, 4.0));
        path.line_style = Some(LineStyle::default());

        let mut image = ImageNode::new(7, Some(vec![1, 2, 3]));
        image.effect = ImageEffect::BlackWhite;
        image.brightness = -50;
        image.contrast = 70;

        let tree = PageLayerTree::new(
            120.0,
            80.0,
            LayerNode::leaf(
                BoundingBox::new(0.0, 0.0, 120.0, 80.0),
                None,
                vec![
                    PaintOp::Path {
                        bbox: BoundingBox::new(1.0, 2.0, 30.0, 20.0),
                        path,
                    },
                    PaintOp::Image {
                        bbox: BoundingBox::new(3.0, 4.0, 30.0, 20.0),
                        image,
                        resolved: None,
                    },
                    PaintOp::Equation {
                        bbox: BoundingBox::new(5.0, 6.0, 30.0, 20.0),
                        equation: EquationNode {
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
                            note_ref: None,
                        },
                    },
                    PaintOp::Placeholder {
                        bbox: BoundingBox::new(7.0, 8.0, 30.0, 20.0),
                        placeholder: PlaceholderNode {
                            fill_color: 0x00F0F0F0,
                            stroke_color: 0x00000000,
                            label: "OLE".to_string(),
                        },
                    },
                    PaintOp::RawSvg {
                        bbox: BoundingBox::new(9.0, 10.0, 30.0, 20.0),
                        raw: RawSvgNode {
                            svg: "<g><path d=\"M0 0L1 1\"/></g>".to_string(),
                        },
                    },
                ],
            ),
        );

        let json = tree.to_json();

        assert!(json.contains("\"connectorEndpoints\":{\"x1\":1.000"));
        assert!(json.contains("\"lineStyle\":"));
        assert!(json.contains("\"effect\":\"blackWhite\""));
        assert!(json.contains("\"brightness\":-50"));
        assert!(json.contains("\"contrast\":70"));
        assert!(json.contains("\"svgContent\":\"<text>x</text>\""));
        assert!(json.contains("\"type\":\"placeholder\""));
        assert!(json.contains("\"label\":\"OLE\""));
        assert!(json.contains("\"type\":\"rawSvg\""));
        assert!(json.contains("\"svg\":\"<g><path d=\\\"M0 0L1 1\\\"/></g>\""));
    }

    #[test]
    fn serializes_layer_node_metadata() {
        let leaf = LayerNode::leaf(BoundingBox::new(0.0, 0.0, 10.0, 10.0), None, Vec::new());
        let clip = LayerNode::clip_rect(
            BoundingBox::new(0.0, 0.0, 10.0, 10.0),
            None,
            BoundingBox::new(1.0, 1.0, 8.0, 8.0),
            leaf,
            ClipKind::Body,
        );
        let root = LayerNode::group(
            BoundingBox::new(0.0, 0.0, 10.0, 10.0),
            None,
            vec![clip],
            CacheHint::StaticSubtree,
            GroupKind::Column(2),
        )
        .with_layer(Some(RenderLayerInfo::new(
            Some(TextWrap::BehindText),
            7,
            42,
        )));

        let json = PageLayerTree::new(10.0, 10.0, root).to_json();

        assert!(json.contains("\"groupKind\":{\"kind\":\"column\",\"index\":2}"));
        assert!(json.contains("\"cacheHint\":\"staticSubtree\""));
        assert!(json.contains("\"clipKind\":\"body\""));
        assert!(json
            .contains("\"layer\":{\"textWrap\":\"behindText\",\"zOrder\":7,\"stableIndex\":42}"));
    }

    #[test]
    fn serializes_textbox_clip_kind() {
        let leaf = LayerNode::leaf(BoundingBox::new(0.0, 0.0, 10.0, 10.0), None, Vec::new());
        let root = LayerNode::clip_rect(
            BoundingBox::new(0.0, 0.0, 10.0, 10.0),
            None,
            BoundingBox::new(1.0, 1.0, 8.0, 8.0),
            leaf,
            ClipKind::TextBox,
        );

        let json = PageLayerTree::new(10.0, 10.0, root).to_json();

        assert!(json.contains("\"clipKind\":\"textBox\""));
    }

    #[test]
    fn serializes_layer_option_metadata() {
        let root = LayerNode::leaf(BoundingBox::new(0.0, 0.0, 10.0, 10.0), None, Vec::new());
        let json = PageLayerTree::new(10.0, 10.0, root)
            .with_output_options(crate::paint::LayerOutputOptions {
                show_paragraph_marks: true,
                show_control_codes: true,
                show_transparent_borders: true,
                clip_enabled: false,
                debug_overlay: true,
            })
            .to_json();

        let parsed: Value = serde_json::from_str(&json).expect("PageLayerTree JSON");

        assert_eq!(
            parsed["buildOptions"]["showTransparentBorders"].as_bool(),
            Some(true)
        );
        assert_eq!(parsed["buildOptions"]["clipEnabled"].as_bool(), Some(false));
        assert_eq!(parsed["debugOptions"]["debugOverlay"].as_bool(), Some(true));
        assert_eq!(
            parsed["outputOptions"]["showParagraphMarks"].as_bool(),
            Some(true)
        );
        assert_eq!(
            parsed["outputOptions"]["showControlCodes"].as_bool(),
            Some(true)
        );
        assert_eq!(
            parsed["outputOptions"]["showTransparentBorders"].as_bool(),
            Some(true)
        );
        assert_eq!(
            parsed["outputOptions"]["clipEnabled"].as_bool(),
            Some(false)
        );
        assert_eq!(
            parsed["outputOptions"]["debugOverlay"].as_bool(),
            Some(true)
        );
    }
}

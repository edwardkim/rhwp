use std::fmt::Write as _;

use base64::Engine;

use crate::document_core::helpers::{color_ref_to_css, json_escape as raw_json_escape};
use crate::model::control::FormType;
use crate::model::image::ImageEffect;
use crate::model::style::{ImageFillMode, UnderlineType};
use crate::paint::{
    CacheHint, ClipKind, GroupKind, LayerNode, LayerNodeKind, PageLayerTree, PaintOp,
    RenderProfile, TextDecorationKind, TextSourceAnnotation, TextSourceEntry, TextSourceId,
    TextSourceRange, TextSourceSpan, TextSourceTable, LAYER_TREE_SCHEMA,
};
use crate::renderer::layout::compute_char_positions;
use crate::renderer::render_tree::{BoundingBox, FieldMarkerType, ShapeTransform, TextRunNode};
use crate::renderer::{
    ArrowStyle, GradientFillInfo, LineRenderType, LineStyle, PathCommand, PatternFillInfo,
    ShadowStyle, ShapeStyle, StrokeDash, TabLeaderInfo, TextStyle,
};

impl PageLayerTree {
    pub fn to_json(&self) -> String {
        let mut buf = String::with_capacity(32_768);
        buf.push('{');
        let _ = write!(
            buf,
            "\"schemaVersion\":{},\"schemaMinorVersion\":{},\"schema\":{{\"major\":{},\"minor\":{}}},\"resourceTableVersion\":{},\"resourceTableMinorVersion\":{},\"resourceTable\":{{\"major\":{},\"minor\":{}}},\"unit\":{},\"coordinateSystem\":{},\"profile\":{},\"outputOptions\":{{\"showParagraphMarks\":{},\"showControlCodes\":{},\"showTransparentBorders\":{},\"clipEnabled\":{},\"debugOverlay\":{}}},\"pageWidth\":{:.3},\"pageHeight\":{:.3},\"root\":",
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
            self.output_options.show_paragraph_marks,
            self.output_options.show_control_codes,
            self.output_options.show_transparent_borders,
            self.output_options.clip_enabled,
            self.output_options.debug_overlay,
            self.page_width,
            self.page_height
        );
        let mut text_source_state = TextSourceExportState::default();
        self.root.write_json(&mut buf, &mut text_source_state);
        buf.push_str(",\"textSources\":");
        write_text_source_entries(&mut buf, &self.text_sources);
        write_text_export_metadata(&mut buf, &self.root);
        buf.push('}');
        buf
    }
}

fn write_text_export_metadata(buf: &mut String, root: &LayerNode) {
    let externalized_visuals = externalized_text_visuals(root);
    buf.push_str(",\"usedFeatures\":[\"text.paintStyle\",\"text.sourceTable\",\"text.sourceSpan\",\"text.v2.placement\",\"text.v2.clusters\",\"text.projectionKind\",\"text.legacyVisuals\"");
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
    buf.push_str("],\"optionalFeatures\":[],\"knownFeatures\":[\"fontResources\",\"fontResources.blobFaceSplit\",\"text.variantGroups\",\"text.shapeDiagnostics\",\"text.glyphRun\",\"text.outlineGlyph\",\"text.specialVisualOps\",\"text.charOverlapOp\",\"text.controlMarkOp\",\"text.tabLeaderOp\",\"text.decorationOp\",\"text.vertical.mixedPerGlyph\"],\"requiredFeatures\":[],\"text\":{\"defaultVariant\":\"textRun\",\"variants\":[\"textRun\"],\"variantSelection\":\"exclusiveVariantSet\",\"sourceTextPreserved\":true,\"clusterEncoding\":[\"utf8\",\"utf16\"],\"fallbackRequired\":true,\"placementAuthority\":\"compatibilityProjection\",\"externalizedVisuals\":[");
    for (idx, visual) in externalized_visuals.iter().enumerate() {
        if idx > 0 {
            buf.push(',');
        }
        buf.push_str(&json_escape(visual));
    }
    buf.push_str("]}");
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
    fn write_json(&self, buf: &mut String, text_sources: &mut TextSourceExportState) {
        buf.push('{');
        buf.push_str("\"bounds\":");
        write_bbox(buf, self.bounds);
        if let Some(source_node_id) = self.source_node_id {
            let _ = write!(buf, ",\"sourceNodeId\":{}", source_node_id);
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
                    child.write_json(buf, text_sources);
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
                child.write_json(buf, text_sources);
            }
            LayerNodeKind::Leaf { ops } => {
                buf.push_str(",\"kind\":\"leaf\",\"ops\":[");
                let leaf_visuals = LeafTextVisualOps::from_ops(ops);
                for (idx, op) in ops.iter().enumerate() {
                    if idx > 0 {
                        buf.push(',');
                    }
                    op.write_json(buf, text_sources, leaf_visuals);
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
        leaf_visuals: LeafTextVisualOps,
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
                buf.push_str(",\"placement\":");
                write_text_run_placement(buf, *bbox, run);
                buf.push_str(",\"clusters\":");
                write_text_clusters(buf, run);
                buf.push_str(",\"source\":");
                write_text_source_span(buf, &source);
                buf.push_str(",\"style\":");
                write_text_style(buf, &run.style);
                buf.push_str(",\"paintStyle\":");
                write_text_style(buf, &run.style);
                write_text_legacy_visuals(buf, run, leaf_visuals);
                buf.push_str(",\"positions\":");
                write_text_positions(buf, run);
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
            PaintOp::Image { bbox, image } => {
                buf.push('{');
                buf.push_str("\"type\":\"image\",\"bbox\":");
                write_bbox(buf, *bbox);
                if let Some(data) = &image.data {
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

#[derive(Debug, Clone, Copy, Default)]
struct LeafTextVisualOps {
    char_overlap: bool,
    control_marks: bool,
    tab_leaders: bool,
    decorations: bool,
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
                _ => {}
            }
        }
        visuals
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

fn write_text_positions(buf: &mut String, run: &TextRunNode) {
    let positions = compute_char_positions(&run.text, &run.style);
    buf.push('[');
    for (idx, position) in positions.iter().enumerate() {
        if idx > 0 {
            buf.push(',');
        }
        let _ = write!(buf, "{:.3}", position);
    }
    buf.push(']');
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

fn write_text_legacy_visuals(buf: &mut String, run: &TextRunNode, leaf_visuals: LeafTextVisualOps) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::paint::{
        CacheHint, ClipKind, GroupKind, LayerNode, PageLayerTree, TextDecorationKind,
    };
    use crate::renderer::composer::CharOverlapInfo;
    use crate::renderer::equation::layout::{LayoutBox, LayoutKind};
    use crate::renderer::render_tree::{
        EquationNode, FieldMarkerType, ImageNode, PathNode, PlaceholderNode, RawSvgNode,
        TextRunNode,
    };

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
        assert!(json.contains("\"schemaVersion\":1"));
        assert!(json.contains("\"schemaMinorVersion\":8"));
        assert!(json.contains("\"schema\":{\"major\":1,\"minor\":8}"));
        assert!(json.contains("\"resourceTableVersion\":1"));
        assert!(json.contains("\"resourceTableMinorVersion\":2"));
        assert!(json.contains("\"resourceTable\":{\"major\":1,\"minor\":2}"));
        assert!(json.contains("\"unit\":\"px\""));
        assert!(json.contains("\"coordinateSystem\":\"page-top-left-y-down\""));
        assert!(json.contains("\"profile\":\"screen\""));
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
        assert!(json.contains(&positions_json));
        assert!(json.contains("\"isParaEnd\":true"));
        assert!(json.contains("\"isLineBreakEnd\":true"));
        assert!(json.contains("\"fieldMarker\":{\"kind\":\"fieldBegin\"}"));
        assert!(json.contains("\"charOverlap\":{\"borderType\":1,\"innerCharSize\":90}"));
        assert!(json.contains("\"usedFeatures\":[\"text.paintStyle\""));
        assert!(json.contains("\"knownFeatures\":[\"fontResources\""));
        assert!(json.contains("\"text\":{\"defaultVariant\":\"textRun\""));
        assert!(json.contains("\"fontFamily\":\"Noto Sans KR\""));
        assert!(json.contains("\"italic\":true"));
        assert!(json.contains("\"shadeColor\":\"#ffff00\""));
        assert!(json.contains("\"emphasisDot\":2"));
        assert!(json.contains("\"type\":\"rectangle\""));
        assert!(json.contains("\"cornerRadius\":4.000"));
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
        );

        let json = PageLayerTree::new(10.0, 10.0, root).to_json();

        assert!(json.contains("\"groupKind\":{\"kind\":\"column\",\"index\":2}"));
        assert!(json.contains("\"cacheHint\":\"staticSubtree\""));
        assert!(json.contains("\"clipKind\":\"body\""));
    }

    #[test]
    fn serializes_layer_output_options() {
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

        assert!(json.contains("\"showParagraphMarks\":true"));
        assert!(json.contains("\"showControlCodes\":true"));
        assert!(json.contains("\"showTransparentBorders\":true"));
        assert!(json.contains("\"clipEnabled\":false"));
        assert!(json.contains("\"debugOverlay\":true"));
    }
}

fn json_escape(value: &str) -> String {
    format!("\"{}\"", raw_json_escape(value))
}

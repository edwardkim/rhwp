use std::fmt::Write as _;

use base64::Engine;

use crate::document_core::helpers::{color_ref_to_css, json_escape as raw_json_escape};
use crate::model::control::FormType;
use crate::model::image::ImageEffect;
use crate::model::style::{ImageFillMode, UnderlineType};
use crate::paint::{
    CacheHint, ClipKind, GroupKind, LayerNode, LayerNodeKind, PageLayerTree, PaintOp,
    RenderProfile, PAGE_LAYER_TREE_COORDINATE_SYSTEM, PAGE_LAYER_TREE_RESOURCE_TABLE_VERSION,
    PAGE_LAYER_TREE_SCHEMA_VERSION, PAGE_LAYER_TREE_UNIT,
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
            "\"schemaVersion\":{},\"resourceTableVersion\":{},\"unit\":{},\"coordinateSystem\":{},\"profile\":{},\"outputOptions\":{{\"showParagraphMarks\":{},\"showControlCodes\":{},\"showTransparentBorders\":{},\"clipEnabled\":{},\"debugOverlay\":{}}},\"pageWidth\":{:.3},\"pageHeight\":{:.3},\"root\":",
            PAGE_LAYER_TREE_SCHEMA_VERSION,
            PAGE_LAYER_TREE_RESOURCE_TABLE_VERSION,
            json_escape(PAGE_LAYER_TREE_UNIT),
            json_escape(PAGE_LAYER_TREE_COORDINATE_SYSTEM),
            json_escape(render_profile_str(self.profile)),
            self.output_options.show_paragraph_marks,
            self.output_options.show_control_codes,
            self.output_options.show_transparent_borders,
            self.output_options.clip_enabled,
            self.output_options.debug_overlay,
            self.page_width,
            self.page_height
        );
        self.root.write_json(&mut buf);
        buf.push('}');
        buf
    }
}

impl LayerNode {
    fn write_json(&self, buf: &mut String) {
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
                    child.write_json(buf);
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
                child.write_json(buf);
            }
            LayerNodeKind::Leaf { ops } => {
                buf.push_str(",\"kind\":\"leaf\",\"ops\":[");
                for (idx, op) in ops.iter().enumerate() {
                    if idx > 0 {
                        buf.push(',');
                    }
                    op.write_json(buf);
                }
                buf.push(']');
            }
        }
        buf.push('}');
    }
}

impl PaintOp {
    fn write_json(&self, buf: &mut String) {
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
                let _ = write!(
                    buf,
                    ",\"text\":{},\"baseline\":{:.3},\"rotation\":{:.3},\"isVertical\":{}",
                    json_escape(&run.text),
                    run.baseline,
                    run.rotation,
                    run.is_vertical,
                );
                buf.push_str(",\"style\":");
                write_text_style(buf, &run.style);
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
                    let base64_data = base64::engine::general_purpose::STANDARD.encode(&*final_data);
                    let _ = write!(buf, ",\"mime\":\"{}\",\"base64\":{}", final_mime, json_escape(&base64_data));
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
        "\"fontFamily\":{},\"fontSize\":{:.3},\"color\":{},\"bold\":{},\"italic\":{},\"underline\":{},\"strikethrough\":{},\"shadowType\":{},\"shadowColor\":{},\"shadowOffsetX\":{:.3},\"shadowOffsetY\":{:.3},\"underlineColor\":{},\"strikeColor\":{},\"shadeColor\":{},\"emphasisDot\":{}",
        json_escape(&style.font_family),
        style.font_size,
        json_escape(&color_ref_to_css(style.color)),
        style.bold,
        style.italic,
        json_escape(underline_type_str(style.underline)),
        style.strikethrough,
        style.shadow_type,
        json_escape(&color_ref_to_css(style.shadow_color)),
        style.shadow_offset_x,
        style.shadow_offset_y,
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
    use crate::paint::{CacheHint, ClipKind, GroupKind, LayerNode, PageLayerTree};
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
        assert!(json.contains("\"resourceTableVersion\":1"));
        assert!(json.contains("\"unit\":\"px\""));
        assert!(json.contains("\"coordinateSystem\":\"page-top-left\""));
        assert!(json.contains("\"profile\":\"screen\""));
        assert!(json.contains("\"outputOptions\":{"));
        assert!(json.contains("\"clipEnabled\":true"));
        assert!(json.contains("\"type\":\"textRun\""));
        assert!(json.contains(&positions_json));
        assert!(json.contains("\"isParaEnd\":true"));
        assert!(json.contains("\"isLineBreakEnd\":true"));
        assert!(json.contains("\"fieldMarker\":{\"kind\":\"fieldBegin\"}"));
        assert!(json.contains("\"charOverlap\":{\"borderType\":1,\"innerCharSize\":90}"));
        assert!(json.contains("\"fontFamily\":\"Noto Sans KR\""));
        assert!(json.contains("\"italic\":true"));
        assert!(json.contains("\"shadeColor\":\"#ffff00\""));
        assert!(json.contains("\"emphasisDot\":2"));
        assert!(json.contains("\"type\":\"rectangle\""));
        assert!(json.contains("\"cornerRadius\":4.000"));
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

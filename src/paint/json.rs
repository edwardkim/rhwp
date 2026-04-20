use std::fmt::Write as _;

use base64::Engine;

use crate::document_core::helpers::{color_ref_to_css, json_escape as raw_json_escape};
use crate::model::control::FormType;
use crate::model::style::{ImageFillMode, UnderlineType};
use crate::paint::{ClipKind, LayerNode, LayerNodeKind, PageLayerTree, PaintOp};
use crate::renderer::equation::ast::MatrixStyle;
use crate::renderer::equation::layout::{LayoutBox, LayoutKind};
use crate::renderer::equation::symbols::{DecoKind, FontStyleKind};
use crate::renderer::layout::compute_char_positions;
use crate::renderer::render_tree::{BoundingBox, ShapeTransform, TextRunNode};
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
            "\"pageWidth\":{:.6},\"pageHeight\":{:.6},\"root\":",
            self.page_width, self.page_height
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
            LayerNodeKind::Group { children, .. } => {
                buf.push_str(",\"kind\":\"group\",\"children\":[");
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
                let _ = write!(buf, ",\"borderWidth\":{:.6}", background.border_width);
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
                    ",\"text\":{},\"baseline\":{:.6},\"rotation\":{:.6},\"isVertical\":{}",
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
                buf.push('}');
            }
            PaintOp::FootnoteMarker { bbox, marker } => {
                buf.push('{');
                buf.push_str("\"type\":\"footnoteMarker\",\"bbox\":");
                write_bbox(buf, *bbox);
                let _ = write!(
                    buf,
                    ",\"text\":{},\"fontFamily\":{},\"fontSize\":{:.6},\"color\":{}",
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
                    ",\"x1\":{:.6},\"y1\":{:.6},\"x2\":{:.6},\"y2\":{:.6},\"style\":",
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
                    ",\"cornerRadius\":{:.6},\"style\":",
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
                        ",\"connectorEndpoints\":{{\"x1\":{:.6},\"y1\":{:.6},\"x2\":{:.6},\"y2\":{:.6}}}",
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
                    let base64_data = base64::engine::general_purpose::STANDARD.encode(data);
                    let _ = write!(buf, ",\"base64\":{}", json_escape(&base64_data));
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
                        ",\"originalSize\":{{\"width\":{:.6},\"height\":{:.6}}}",
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
                    ",\"color\":{},\"fontSize\":{:.6},\"svgContent\":{},\"layoutBox\":",
                    json_escape(&equation.color_str),
                    equation.font_size,
                    json_escape(&equation.svg_content),
                );
                write_equation_layout_box(buf, &equation.layout_box);
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
        }
    }
}

fn write_bbox(buf: &mut String, bbox: BoundingBox) {
    let _ = write!(
        buf,
        "{{\"x\":{:.6},\"y\":{:.6},\"width\":{:.6},\"height\":{:.6}}}",
        bbox.x, bbox.y, bbox.width, bbox.height
    );
}

fn write_text_style(buf: &mut String, style: &TextStyle) {
    buf.push('{');
    let _ = write!(
        buf,
        "\"fontFamily\":{},\"fontSize\":{:.6},\"color\":{},\"bold\":{},\"italic\":{},\"ratio\":{:.6},\"underline\":{},\"underlineShape\":{},\"strikethrough\":{},\"strikeShape\":{},\"outlineType\":{},\"shadowType\":{},\"shadowColor\":{},\"shadowOffsetX\":{:.6},\"shadowOffsetY\":{:.6},\"emboss\":{},\"engrave\":{},\"emphasisDot\":{},\"underlineColor\":{},\"strikeColor\":{},\"shadeColor\":{}",
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
        style.emphasis_dot,
        json_escape(&color_ref_to_css(style.underline_color)),
        json_escape(&color_ref_to_css(style.strike_color)),
        json_escape(&color_ref_to_css(style.shade_color)),
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
        let _ = write!(buf, "{:.6}", position);
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
            "{{\"startX\":{:.6},\"endX\":{:.6},\"fillType\":{}}}",
            leader.start_x, leader.end_x, leader.fill_type
        );
    }
    buf.push(']');
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
        ",\"strokeWidth\":{:.6},\"strokeDash\":{},\"opacity\":{:.6}",
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
        "{{\"shadowType\":{},\"color\":{},\"offsetX\":{:.6},\"offsetY\":{:.6},\"alpha\":{}}}",
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
        let _ = write!(buf, "{:.6}", position);
    }
    buf.push_str("]}");
}

fn write_line_style(buf: &mut String, style: &LineStyle) {
    let _ = write!(
        buf,
        "{{\"color\":{},\"width\":{:.6},\"dash\":{},\"lineType\":{},\"startArrow\":{},\"endArrow\":{},\"startArrowSize\":{},\"endArrowSize\":{}",
        json_escape(&color_ref_to_css(style.color)),
        style.width,
        json_escape(stroke_dash_str(style.dash)),
        json_escape(line_render_type_str(style.line_type)),
        json_escape(arrow_style_str(style.start_arrow)),
        json_escape(arrow_style_str(style.end_arrow)),
        style.start_arrow_size,
        style.end_arrow_size,
    );
    if let Some(shadow) = &style.shadow {
        buf.push_str(",\"shadow\":");
        write_shadow_style(buf, shadow);
    }
    buf.push('}');
}

fn write_transform(buf: &mut String, transform: ShapeTransform) {
    let _ = write!(
        buf,
        "{{\"rotation\":{:.6},\"horzFlip\":{},\"vertFlip\":{}}}",
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
                let _ = write!(buf, "{{\"type\":\"moveTo\",\"x\":{:.6},\"y\":{:.6}}}", x, y);
            }
            PathCommand::LineTo(x, y) => {
                let _ = write!(buf, "{{\"type\":\"lineTo\",\"x\":{:.6},\"y\":{:.6}}}", x, y);
            }
            PathCommand::CurveTo(x1, y1, x2, y2, x3, y3) => {
                let _ = write!(
                    buf,
                    "{{\"type\":\"curveTo\",\"x1\":{:.6},\"y1\":{:.6},\"x2\":{:.6},\"y2\":{:.6},\"x3\":{:.6},\"y3\":{:.6}}}",
                    x1, y1, x2, y2, x3, y3
                );
            }
            PathCommand::ArcTo(rx, ry, rotation, large_arc, sweep, x, y) => {
                let _ = write!(
                    buf,
                    "{{\"type\":\"arcTo\",\"rx\":{:.6},\"ry\":{:.6},\"rotation\":{:.6},\"largeArc\":{},\"sweep\":{},\"x\":{:.6},\"y\":{:.6}}}",
                    rx, ry, rotation, large_arc, sweep, x, y
                );
            }
            PathCommand::ClosePath => buf.push_str("{\"type\":\"closePath\"}"),
        }
    }
    buf.push(']');
}

fn write_equation_layout_box(buf: &mut String, layout: &LayoutBox) {
    let _ = write!(
        buf,
        "{{\"x\":{:.6},\"y\":{:.6},\"width\":{:.6},\"height\":{:.6},\"baseline\":{:.6},\"kind\":",
        layout.x, layout.y, layout.width, layout.height, layout.baseline,
    );
    write_equation_layout_kind(buf, &layout.kind);
    buf.push('}');
}

fn write_equation_layout_kind(buf: &mut String, kind: &LayoutKind) {
    match kind {
        LayoutKind::Row(children) => {
            buf.push_str("{\"type\":\"row\",\"children\":[");
            for (idx, child) in children.iter().enumerate() {
                if idx > 0 {
                    buf.push(',');
                }
                write_equation_layout_box(buf, child);
            }
            buf.push_str("]}");
        }
        LayoutKind::Text(text) => {
            let _ = write!(buf, "{{\"type\":\"text\",\"text\":{}}}", json_escape(text));
        }
        LayoutKind::Number(text) => {
            let _ = write!(
                buf,
                "{{\"type\":\"number\",\"text\":{}}}",
                json_escape(text)
            );
        }
        LayoutKind::Symbol(text) => {
            let _ = write!(
                buf,
                "{{\"type\":\"symbol\",\"text\":{}}}",
                json_escape(text)
            );
        }
        LayoutKind::MathSymbol(text) => {
            let _ = write!(
                buf,
                "{{\"type\":\"mathSymbol\",\"text\":{}}}",
                json_escape(text)
            );
        }
        LayoutKind::Function(name) => {
            let _ = write!(
                buf,
                "{{\"type\":\"function\",\"name\":{}}}",
                json_escape(name)
            );
        }
        LayoutKind::Fraction { numer, denom } => {
            buf.push_str("{\"type\":\"fraction\",\"numer\":");
            write_equation_layout_box(buf, numer);
            buf.push_str(",\"denom\":");
            write_equation_layout_box(buf, denom);
            buf.push('}');
        }
        LayoutKind::Sqrt { index, body } => {
            buf.push_str("{\"type\":\"sqrt\"");
            if let Some(index) = index {
                buf.push_str(",\"index\":");
                write_equation_layout_box(buf, index);
            }
            buf.push_str(",\"body\":");
            write_equation_layout_box(buf, body);
            buf.push('}');
        }
        LayoutKind::Superscript { base, sup } => {
            buf.push_str("{\"type\":\"superscript\",\"base\":");
            write_equation_layout_box(buf, base);
            buf.push_str(",\"sup\":");
            write_equation_layout_box(buf, sup);
            buf.push('}');
        }
        LayoutKind::Subscript { base, sub } => {
            buf.push_str("{\"type\":\"subscript\",\"base\":");
            write_equation_layout_box(buf, base);
            buf.push_str(",\"sub\":");
            write_equation_layout_box(buf, sub);
            buf.push('}');
        }
        LayoutKind::SubSup { base, sub, sup } => {
            buf.push_str("{\"type\":\"subSup\",\"base\":");
            write_equation_layout_box(buf, base);
            buf.push_str(",\"sub\":");
            write_equation_layout_box(buf, sub);
            buf.push_str(",\"sup\":");
            write_equation_layout_box(buf, sup);
            buf.push('}');
        }
        LayoutKind::BigOp { symbol, sub, sup } => {
            let _ = write!(
                buf,
                "{{\"type\":\"bigOp\",\"symbol\":{}",
                json_escape(symbol)
            );
            if let Some(sub) = sub {
                buf.push_str(",\"sub\":");
                write_equation_layout_box(buf, sub);
            }
            if let Some(sup) = sup {
                buf.push_str(",\"sup\":");
                write_equation_layout_box(buf, sup);
            }
            buf.push('}');
        }
        LayoutKind::Limit { is_upper, sub } => {
            let _ = write!(buf, "{{\"type\":\"limit\",\"isUpper\":{}", is_upper);
            if let Some(sub) = sub {
                buf.push_str(",\"sub\":");
                write_equation_layout_box(buf, sub);
            }
            buf.push('}');
        }
        LayoutKind::Matrix { cells, style } => {
            let _ = write!(
                buf,
                "{{\"type\":\"matrix\",\"style\":{},\"cells\":[",
                json_escape(matrix_style_str(*style))
            );
            for (row_idx, row) in cells.iter().enumerate() {
                if row_idx > 0 {
                    buf.push(',');
                }
                buf.push('[');
                for (cell_idx, cell) in row.iter().enumerate() {
                    if cell_idx > 0 {
                        buf.push(',');
                    }
                    write_equation_layout_box(buf, cell);
                }
                buf.push(']');
            }
            buf.push_str("]}");
        }
        LayoutKind::Rel { arrow, over, under } => {
            buf.push_str("{\"type\":\"rel\",\"arrow\":");
            write_equation_layout_box(buf, arrow);
            buf.push_str(",\"over\":");
            write_equation_layout_box(buf, over);
            if let Some(under) = under {
                buf.push_str(",\"under\":");
                write_equation_layout_box(buf, under);
            }
            buf.push('}');
        }
        LayoutKind::EqAlign { rows } => {
            buf.push_str("{\"type\":\"eqAlign\",\"rows\":[");
            for (idx, (left, right)) in rows.iter().enumerate() {
                if idx > 0 {
                    buf.push(',');
                }
                buf.push_str("{\"left\":");
                write_equation_layout_box(buf, left);
                buf.push_str(",\"right\":");
                write_equation_layout_box(buf, right);
                buf.push('}');
            }
            buf.push_str("]}");
        }
        LayoutKind::Paren { left, right, body } => {
            let _ = write!(
                buf,
                "{{\"type\":\"paren\",\"left\":{},\"right\":{},\"body\":",
                json_escape(left),
                json_escape(right),
            );
            write_equation_layout_box(buf, body);
            buf.push('}');
        }
        LayoutKind::Decoration { kind, body } => {
            let _ = write!(
                buf,
                "{{\"type\":\"decoration\",\"decoration\":{},\"body\":",
                json_escape(deco_kind_str(*kind))
            );
            write_equation_layout_box(buf, body);
            buf.push('}');
        }
        LayoutKind::FontStyle { style, body } => {
            let _ = write!(
                buf,
                "{{\"type\":\"fontStyle\",\"fontStyle\":{},\"body\":",
                json_escape(font_style_kind_str(*style))
            );
            write_equation_layout_box(buf, body);
            buf.push('}');
        }
        LayoutKind::Space(width) => {
            let _ = write!(buf, "{{\"type\":\"space\",\"width\":{:.6}}}", width);
        }
        LayoutKind::Newline => buf.push_str("{\"type\":\"newline\"}"),
        LayoutKind::Empty => buf.push_str("{\"type\":\"empty\"}"),
    }
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

fn matrix_style_str(value: MatrixStyle) -> &'static str {
    match value {
        MatrixStyle::Plain => "plain",
        MatrixStyle::Paren => "paren",
        MatrixStyle::Bracket => "bracket",
        MatrixStyle::Vert => "vert",
    }
}

fn deco_kind_str(value: DecoKind) -> &'static str {
    match value {
        DecoKind::Hat => "hat",
        DecoKind::Check => "check",
        DecoKind::Tilde => "tilde",
        DecoKind::Acute => "acute",
        DecoKind::Grave => "grave",
        DecoKind::Dot => "dot",
        DecoKind::DDot => "dDot",
        DecoKind::Bar => "bar",
        DecoKind::Vec => "vec",
        DecoKind::Dyad => "dyad",
        DecoKind::Under => "under",
        DecoKind::Arch => "arch",
        DecoKind::Underline => "underline",
        DecoKind::Overline => "overline",
        DecoKind::StrikeThrough => "strikeThrough",
    }
}

fn font_style_kind_str(value: FontStyleKind) -> &'static str {
    match value {
        FontStyleKind::Roman => "roman",
        FontStyleKind::Italic => "italic",
        FontStyleKind::Bold => "bold",
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

fn form_type_str(value: FormType) -> &'static str {
    match value {
        FormType::PushButton => "pushButton",
        FormType::CheckBox => "checkBox",
        FormType::RadioButton => "radioButton",
        FormType::ComboBox => "comboBox",
        FormType::Edit => "edit",
    }
}

fn clip_kind_str(value: ClipKind) -> &'static str {
    match value {
        ClipKind::Body => "body",
        ClipKind::TableCell => "tableCell",
        ClipKind::Generic => "generic",
    }
}

#[cfg(test)]
#[allow(clippy::items_after_test_module)]
mod tests {
    use super::*;
    use crate::paint::{ClipKind, LayerNode, PageLayerTree};
    use crate::renderer::render_tree::{EquationNode, TextRunNode};

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
                    underline: UnderlineType::Bottom,
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
                field_marker: Default::default(),
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
        let equation = PaintOp::Equation {
            bbox: BoundingBox::new(12.0, 44.0, 40.0, 16.0),
            equation: EquationNode {
                svg_content: "<text x=\"0\" y=\"12\">x</text>".to_string(),
                layout_box: crate::renderer::equation::layout::LayoutBox {
                    x: 0.0,
                    y: 0.0,
                    width: 10.0,
                    height: 12.0,
                    baseline: 9.0,
                    kind: crate::renderer::equation::layout::LayoutKind::Text("x".to_string()),
                },
                color_str: "#112233".to_string(),
                color: 0x00332211,
                font_size: 14.0,
                section_index: None,
                para_index: None,
                control_index: None,
                cell_index: None,
                cell_para_index: None,
            },
        };

        let tree = PageLayerTree::new(
            120.0,
            80.0,
            LayerNode::leaf(
                BoundingBox::new(0.0, 0.0, 120.0, 80.0),
                None,
                vec![text, rect, equation],
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
                underline: UnderlineType::Bottom,
                ..Default::default()
            },
        );
        let positions_json = format!(
            "\"positions\":[{:.6},{:.6},{:.6}]",
            positions[0], positions[1], positions[2]
        );

        assert!(json.contains("\"kind\":\"leaf\""));
        assert!(json.contains("\"type\":\"textRun\""));
        assert!(json.contains(&positions_json));
        assert!(json.contains("\"fontFamily\":\"Noto Sans KR\""));
        assert!(json.contains("\"type\":\"rectangle\""));
        assert!(json.contains("\"type\":\"equation\""));
        assert!(json.contains("\"svgContent\":\"<text x=\\\"0\\\" y=\\\"12\\\">x</text>\""));
        assert!(json.contains("\"layoutBox\":{\"x\":0.000000,\"y\":0.000000,\"width\":10.000000,\"height\":12.000000,\"baseline\":9.000000,\"kind\":{\"type\":\"text\",\"text\":\"x\"}}"));
        assert!(json.contains("\"cornerRadius\":4.000000"));
    }

    #[test]
    fn serializes_line_shadow_and_connector_arrow_metadata() {
        let line = PaintOp::Line {
            bbox: BoundingBox::new(0.0, 0.0, 24.0, 24.0),
            line: crate::renderer::render_tree::LineNode::new(
                2.0,
                4.0,
                22.0,
                20.0,
                LineStyle {
                    color: 0x000000ff,
                    width: 3.0,
                    dash: StrokeDash::Dash,
                    line_type: LineRenderType::ThinThickDouble,
                    start_arrow: ArrowStyle::Arrow,
                    end_arrow: ArrowStyle::OpenDiamond,
                    start_arrow_size: 2,
                    end_arrow_size: 5,
                    shadow: Some(ShadowStyle {
                        shadow_type: 1,
                        color: 0x00303030,
                        offset_x: 1.5,
                        offset_y: 2.5,
                        alpha: 64,
                    }),
                },
            ),
        };

        let mut path_node = crate::renderer::render_tree::PathNode::new(
            vec![
                PathCommand::MoveTo(4.0, 4.0),
                PathCommand::CurveTo(8.0, 4.0, 16.0, 20.0, 20.0, 20.0),
            ],
            ShapeStyle {
                stroke_color: Some(0x00010203),
                stroke_width: 2.0,
                ..Default::default()
            },
            None,
        );
        path_node.connector_endpoints = Some((4.0, 4.0, 20.0, 20.0));
        path_node.line_style = Some(LineStyle {
            color: 0x00010203,
            width: 2.0,
            dash: StrokeDash::Solid,
            line_type: LineRenderType::Single,
            start_arrow: ArrowStyle::Circle,
            end_arrow: ArrowStyle::Square,
            start_arrow_size: 1,
            end_arrow_size: 8,
            shadow: None,
        });

        let tree = PageLayerTree::new(
            40.0,
            40.0,
            LayerNode::leaf(
                BoundingBox::new(0.0, 0.0, 40.0, 40.0),
                None,
                vec![
                    line,
                    PaintOp::Path {
                        bbox: BoundingBox::new(0.0, 0.0, 24.0, 24.0),
                        path: path_node,
                    },
                ],
            ),
        );

        let json = tree.to_json();
        assert!(json.contains("\"shadow\":{\"shadowType\":1,\"color\":\"#303030\",\"offsetX\":1.500000,\"offsetY\":2.500000,\"alpha\":64}"));
        assert!(json.contains(
            "\"connectorEndpoints\":{\"x1\":4.000000,\"y1\":4.000000,\"x2\":20.000000,\"y2\":20.000000}"
        ));
        assert!(json.contains("\"lineStyle\":{\"color\":\"#030201\",\"width\":2.000000,\"dash\":\"solid\",\"lineType\":\"single\",\"startArrow\":\"circle\",\"endArrow\":\"square\",\"startArrowSize\":1,\"endArrowSize\":8}"));
    }

    #[test]
    fn serializes_clip_kind_for_browser_replay() {
        let tree = PageLayerTree::new(
            40.0,
            40.0,
            LayerNode::clip_rect(
                BoundingBox::new(0.0, 0.0, 40.0, 40.0),
                None,
                BoundingBox::new(1.0, 2.0, 30.0, 20.0),
                LayerNode::leaf(BoundingBox::new(1.0, 2.0, 30.0, 20.0), None, vec![]),
                ClipKind::Body,
            ),
        );

        let json = tree.to_json();
        assert!(json.contains("\"kind\":\"clipRect\""));
        assert!(json.contains("\"clipKind\":\"body\""));
    }
}

fn json_escape(value: &str) -> String {
    format!("\"{}\"", raw_json_escape(value))
}

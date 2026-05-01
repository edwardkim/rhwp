//! HTML DOM 렌더러 (3차 백엔드)
//!
//! 렌더 트리를 HTML 문자열로 변환한다.
//! CSS로 스타일링하여 접근성과 텍스트 선택을 지원한다.

use base64::Engine;
use super::{Renderer, TextStyle, ShapeStyle, LineStyle, PathCommand};
use super::render_tree::{PageRenderTree, RenderNode, RenderNodeType};
use super::layout::compute_char_positions;
use super::svg::{detect_image_mime_type, convert_wmf_to_svg};
use crate::model::style::UnderlineType;

/// HTML 렌더러
pub struct HtmlRenderer {
    /// HTML 출력 버퍼
    output: String,
    /// 페이지 폭
    width: f64,
    /// 페이지 높이
    height: f64,
    /// 문단부호(¶) 표시 여부
    pub show_paragraph_marks: bool,
    /// 조판부호 표시 여부
    pub show_control_codes: bool,
}

impl HtmlRenderer {
    pub fn new() -> Self {
        Self {
            output: String::new(),
            width: 0.0,
            height: 0.0,
            show_paragraph_marks: false,
            show_control_codes: false,
        }
    }

    /// 생성된 HTML 문자열 반환
    pub fn output(&self) -> &str {
        &self.output
    }

    /// 렌더 트리를 HTML로 렌더링
    pub fn render_tree(&mut self, tree: &PageRenderTree) {
        self.render_node(&tree.root);
    }

    /// 개별 노드를 HTML로 렌더링
    fn render_node(&mut self, node: &RenderNode) {
        if !node.visible {
            return;
        }

        match &node.node_type {
            RenderNodeType::Page(page) => {
                self.begin_page(page.width, page.height);
            }
            RenderNodeType::PageBackground(bg) => {
                let bg_color = bg.background_color
                    .map(|c| color_to_css(c))
                    .unwrap_or_else(|| "#ffffff".to_string());
                self.output.push_str(&format!(
                    "<div class=\"page-bg\" style=\"position:absolute;left:{}px;top:{}px;width:{}px;height:{}px;background:{};\"></div>\n",
                    node.bbox.x, node.bbox.y,
                    node.bbox.width, node.bbox.height,
                    bg_color,
                ));
            }
            RenderNodeType::Header => {
                self.output.push_str(&format!(
                    "<header class=\"page-header\" style=\"position:absolute;left:{}px;top:{}px;width:{}px;height:{}px;\">\n",
                    node.bbox.x, node.bbox.y, node.bbox.width, node.bbox.height,
                ));
                for child in &node.children {
                    self.render_node(child);
                }
                self.output.push_str("</header>\n");
                return; // 자식은 이미 처리됨
            }
            RenderNodeType::Footer => {
                self.output.push_str(&format!(
                    "<footer class=\"page-footer\" style=\"position:absolute;left:{}px;top:{}px;width:{}px;height:{}px;\">\n",
                    node.bbox.x, node.bbox.y, node.bbox.width, node.bbox.height,
                ));
                for child in &node.children {
                    self.render_node(child);
                }
                self.output.push_str("</footer>\n");
                return;
            }
            RenderNodeType::Body { .. } => {
                self.output.push_str(&format!(
                    "<div class=\"page-body\" style=\"position:absolute;left:{}px;top:{}px;width:{}px;height:{}px;\">\n",
                    node.bbox.x, node.bbox.y, node.bbox.width, node.bbox.height,
                ));
                for child in &node.children {
                    self.render_node(child);
                }
                self.output.push_str("</div>\n");
                return;
            }
            RenderNodeType::Column(col_idx) => {
                self.output.push_str(&format!(
                    "<div class=\"column column-{}\" style=\"position:absolute;left:{}px;top:{}px;width:{}px;height:{}px;\">\n",
                    col_idx, node.bbox.x, node.bbox.y, node.bbox.width, node.bbox.height,
                ));
                for child in &node.children {
                    self.render_node(child);
                }
                self.output.push_str("</div>\n");
                return;
            }
            RenderNodeType::TextLine(line) => {
                self.output.push_str(&format!(
                    "<div class=\"text-line\" style=\"position:absolute;left:{}px;top:{}px;width:{}px;height:{}px;line-height:{}px;\">\n",
                    node.bbox.x, node.bbox.y, node.bbox.width, node.bbox.height, line.line_height,
                ));
                for child in &node.children {
                    self.render_node(child);
                }
                self.output.push_str("</div>\n");
                return;
            }
            RenderNodeType::TextRun(run) => {
                self.draw_text(&run.text, node.bbox.x, node.bbox.y, &run.style);
                if self.show_paragraph_marks || self.show_control_codes {
                    let font_size = if run.style.font_size > 0.0 { run.style.font_size } else { 12.0 };
                    // 공백·탭 기호
                    if !run.text.is_empty() {
                        let char_positions = compute_char_positions(&run.text, &run.style);
                        let mark_font_size = font_size * 0.5;
                        for (i, c) in run.text.chars().enumerate() {
                            if c == ' ' {
                                let cx = node.bbox.x + char_positions[i];
                                let next_x = if i + 1 < char_positions.len() {
                                    node.bbox.x + char_positions[i + 1]
                                } else {
                                    node.bbox.x + node.bbox.width
                                };
                                let mid_x = (cx + next_x) / 2.0 - mark_font_size * 0.25;
                                self.output.push_str(&format!(
                                    "<span class=\"para-mark\" style=\"position:absolute;left:{}px;top:{}px;font-size:{}px;color:#4A90D9;\">\u{2228}</span>\n",
                                    mid_x, node.bbox.y, mark_font_size,
                                ));
                            } else if c == '\t' {
                                let cx = node.bbox.x + char_positions[i];
                                self.output.push_str(&format!(
                                    "<span class=\"para-mark\" style=\"position:absolute;left:{}px;top:{}px;font-size:{}px;color:#4A90D9;\">\u{2192}</span>\n",
                                    cx, node.bbox.y, mark_font_size,
                                ));
                            }
                        }
                    }
                    // 하드 리턴·강제 줄바꿈 기호
                    if run.is_para_end || run.is_line_break_end {
                        let mark_x = if run.text.is_empty() { node.bbox.x } else { node.bbox.x + node.bbox.width };
                        let mark = if run.is_line_break_end { "\u{2193}" } else { "\u{21B5}" };
                        self.output.push_str(&format!(
                            "<span class=\"para-mark\" style=\"position:absolute;left:{}px;top:{}px;font-size:{}px;color:#4A90D9;\">{}</span>\n",
                            mark_x, node.bbox.y, font_size, mark,
                        ));
                    }
                }
            }
            RenderNodeType::Table(_table) => {
                self.output.push_str(&format!(
                    "<table class=\"hwp-table\" style=\"position:absolute;left:{}px;top:{}px;width:{}px;height:{}px;border-collapse:collapse;\">\n",
                    node.bbox.x, node.bbox.y, node.bbox.width, node.bbox.height,
                ));
                for child in &node.children {
                    self.render_node(child);
                }
                self.output.push_str("</table>\n");
                return;
            }
            RenderNodeType::TableCell(cell) => {
                self.output.push_str(&format!(
                    "<td colspan=\"{}\" rowspan=\"{}\" style=\"width:{}px;height:{}px;\">\n",
                    cell.col_span, cell.row_span, node.bbox.width, node.bbox.height,
                ));
                for child in &node.children {
                    self.render_node(child);
                }
                self.output.push_str("</td>\n");
                return;
            }
            RenderNodeType::Rectangle(rect) => {
                self.draw_rect(
                    node.bbox.x, node.bbox.y,
                    node.bbox.width, node.bbox.height,
                    rect.corner_radius,
                    &rect.style,
                );
            }
            RenderNodeType::Image(img) => {
                if let Some(ref data) = img.data {
                    self.draw_image(data, node.bbox.x, node.bbox.y, node.bbox.width, node.bbox.height);
                } else {
                    self.output.push_str(&format!(
                        "<div class=\"hwp-image\" style=\"position:absolute;left:{}px;top:{}px;width:{}px;height:{}px;background:#eee;\"></div>\n",
                        node.bbox.x, node.bbox.y, node.bbox.width, node.bbox.height,
                    ));
                }
            }
            _ => {}
        }

        // 기본: 자식 노드 렌더링
        for child in &node.children {
            self.render_node(child);
        }

        // 조판부호 개체 마커 (붉은색 대괄호)
        if self.show_control_codes {
            let label = match &node.node_type {
                RenderNodeType::Table(_) => Some("[표]"),
                RenderNodeType::Image(_) => Some("[그림]"),
                RenderNodeType::TextBox => Some("[글상자]"),
                RenderNodeType::Equation(_) => Some("[수식]"),
                RenderNodeType::FormObject(_) => Some("[양식]"),
                RenderNodeType::Header => Some("[머리말]"),
                RenderNodeType::Footer => Some("[꼬리말]"),
                RenderNodeType::FootnoteArea => Some("[각주]"),
                _ => None,
            };
            if let Some(label) = label {
                let fs = 10.0;
                self.output.push_str(&format!(
                    "<span class=\"control-mark\" style=\"position:absolute;left:{}px;top:{}px;font-size:{}px;color:#CC3333;\">{}</span>\n",
                    node.bbox.x, node.bbox.y, fs, label,
                ));
            }
        }

        if matches!(node.node_type, RenderNodeType::Page(_)) {
            self.end_page();
        }
    }
}

impl Renderer for HtmlRenderer {
    fn begin_page(&mut self, width: f64, height: f64) {
        self.width = width;
        self.height = height;
        self.output.clear();
        self.output.push_str(&format!(
            "<div class=\"hwp-page\" style=\"position:relative;width:{}px;height:{}px;overflow:hidden;\">\n",
            width, height,
        ));
    }

    fn end_page(&mut self) {
        self.output.push_str("</div>\n");
    }

    fn draw_text(&mut self, text: &str, x: f64, y: f64, style: &TextStyle) {
        // [Task #509] 한컴은 폰트 지정과 상관없이 PUA 를 자체 처리. 지정 폰트에 글리프
        // 부재 시 한컴 내부 매핑이 발행. rhwp 도 동일 동작 모방 (PR #251 정합).
        let text = &text
            .chars()
            .map(crate::renderer::layout::map_pua_bullet_char)
            .collect::<String>();

        let font_size = if style.font_size > 0.0 { style.font_size } else { 12.0 };
        let color = color_to_css(style.color);
        let font_family = if style.font_family.is_empty() {
            "sans-serif".to_string()
        } else {
            let fallback = super::generic_fallback(&style.font_family);
            format!("'{}', {}", escape_html(&style.font_family), fallback)
        };

        // 위첨자/아래첨자: y좌표·font_size 직접 조정 (absolute 위치이므로 vertical-align 불가)
        let (draw_y, draw_size) = if style.superscript {
            (y - font_size * 0.3, font_size * 0.7)
        } else if style.subscript {
            (y + font_size * 0.15, font_size * 0.7)
        } else {
            (y, font_size)
        };

        let mut css = format!(
            "position:absolute;left:{}px;top:{}px;font-family:{};font-size:{}px;color:{};",
            x, draw_y, font_family, draw_size, color,
        );

        if style.bold {
            css.push_str("font-weight:bold;");
        }
        if style.italic {
            css.push_str("font-style:italic;");
        }
        if !matches!(style.underline, UnderlineType::None) {
            let ul_style = match style.underline_shape {
                1 | 5 => "dashed",
                2 | 6 => "dotted",
                3 | 4 => "dashed",
                7..=9 => "double",
                11 => "wavy",
                _ => "solid",
            };
            let ul_pos = if matches!(style.underline, UnderlineType::Top) {
                "text-underline-position:above;"
            } else {
                ""
            };
            css.push_str(&format!("text-decoration:underline;text-decoration-style:{};{}", ul_style, ul_pos));
        }
        if style.strikethrough {
            let st_style = match style.strike_shape {
                1 | 5 => "dashed",
                2 | 6 => "dotted",
                3 | 4 => "dashed",
                7..=9 => "double",
                11 => "wavy",
                _ => "solid",
            };
            css.push_str(&format!("text-decoration:line-through;text-decoration-style:{};", st_style));
        }
        // 외곽선
        if style.outline_type > 0 {
            css.push_str(&format!(
                "-webkit-text-stroke:1px {};color:transparent;",
                color
            ));
        }
        // 양각
        if style.emboss {
            css.push_str("text-shadow:-1px -1px 0 #999,1px 1px 0 #fff;");
        }
        // 음각
        if style.engrave {
            css.push_str("text-shadow:1px 1px 0 #999,-1px -1px 0 #fff;");
        }

        // 형광펜 배경 (CharShape.shade_color 기반 — 편집기에서 적용한 형광펜)
        let shade_rgb = style.shade_color & 0x00FFFFFF;
        if shade_rgb != 0x00FFFFFF && shade_rgb != 0 {
            css.push_str(&format!("background-color:{};", color_to_css(style.shade_color)));
        }

        let ratio = if style.ratio > 0.0 { style.ratio } else { 1.0 };
        if (ratio - 1.0).abs() > 0.01 {
            css.push_str(&format!("transform:scaleX({:.4});transform-origin:left;", ratio));
        }

        self.output.push_str(&format!(
            "<span class=\"text-run\" style=\"{}\">{}</span>\n",
            css, escape_html(text),
        ));
    }

    fn draw_rect(&mut self, x: f64, y: f64, w: f64, h: f64, corner_radius: f64, style: &ShapeStyle) {
        let mut css = format!(
            "position:absolute;left:{}px;top:{}px;width:{}px;height:{}px;",
            x, y, w, h,
        );

        if corner_radius > 0.0 {
            css.push_str(&format!("border-radius:{}px;", corner_radius));
        }

        if let Some(fill) = style.fill_color {
            css.push_str(&format!("background:{};", color_to_css(fill)));
        }

        if let Some(stroke) = style.stroke_color {
            css.push_str(&format!("border:{}px solid {};", style.stroke_width, color_to_css(stroke)));
        }

        self.output.push_str(&format!(
            "<div class=\"hwp-rect\" style=\"{}\"></div>\n",
            css,
        ));
    }

    fn draw_line(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, _style: &LineStyle) {
        // HTML에서는 SVG 인라인으로 선 표현
        let min_x = x1.min(x2);
        let min_y = y1.min(y2);
        let w = (x2 - x1).abs().max(1.0);
        let h = (y2 - y1).abs().max(1.0);
        self.output.push_str(&format!(
            "<svg class=\"hwp-line\" style=\"position:absolute;left:{}px;top:{}px;width:{}px;height:{}px;\"><line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"black\" stroke-width=\"1\"/></svg>\n",
            min_x, min_y, w, h, x1 - min_x, y1 - min_y, x2 - min_x, y2 - min_y,
        ));
    }

    fn draw_ellipse(&mut self, cx: f64, cy: f64, rx: f64, ry: f64, style: &ShapeStyle) {
        let mut css = format!(
            "position:absolute;left:{}px;top:{}px;width:{}px;height:{}px;border-radius:50%;",
            cx - rx, cy - ry, rx * 2.0, ry * 2.0,
        );

        if let Some(fill) = style.fill_color {
            css.push_str(&format!("background:{};", color_to_css(fill)));
        }

        self.output.push_str(&format!(
            "<div class=\"hwp-ellipse\" style=\"{}\"></div>\n",
            css,
        ));
    }

    fn draw_image(&mut self, data: &[u8], x: f64, y: f64, w: f64, h: f64) {
        let mime_type = detect_image_mime_type(data);
        let (render_data, render_mime): (std::borrow::Cow<[u8]>, &str) = if mime_type == "image/x-wmf" {
            match convert_wmf_to_svg(data) {
                Some(svg_bytes) => (std::borrow::Cow::Owned(svg_bytes), "image/svg+xml"),
                None => (std::borrow::Cow::Borrowed(data), mime_type),
            }
        } else {
            (std::borrow::Cow::Borrowed(data), mime_type)
        };
        let base64_data = base64::engine::general_purpose::STANDARD.encode(&*render_data);
        let data_uri = format!("data:{};base64,{}", render_mime, base64_data);
        self.output.push_str(&format!(
            "<img class=\"hwp-image\" src=\"{}\" style=\"position:absolute;left:{}px;top:{}px;width:{}px;height:{}px;\" />\n",
            data_uri, x, y, w, h,
        ));
    }

    fn draw_path(&mut self, commands: &[PathCommand], style: &ShapeStyle) {
        // HTML에서는 인라인 SVG로 패스 표현
        let mut d = String::new();
        for cmd in commands {
            match cmd {
                PathCommand::MoveTo(x, y) => d.push_str(&format!("M{} {} ", x, y)),
                PathCommand::LineTo(x, y) => d.push_str(&format!("L{} {} ", x, y)),
                PathCommand::CurveTo(x1, y1, x2, y2, x, y) => {
                    d.push_str(&format!("C{} {} {} {} {} {} ", x1, y1, x2, y2, x, y));
                }
                PathCommand::ArcTo(rx, ry, x_rot, large_arc, sweep, x, y) => {
                    d.push_str(&format!("A{} {} {} {} {} {} {} ",
                        rx, ry, x_rot,
                        if *large_arc { 1 } else { 0 },
                        if *sweep { 1 } else { 0 },
                        x, y));
                }
                PathCommand::ClosePath => d.push_str("Z "),
            }
        }

        let fill = style.fill_color
            .map(|c| color_to_css(c))
            .unwrap_or_else(|| "none".to_string());

        self.output.push_str(&format!(
            "<svg class=\"hwp-path\" style=\"position:absolute;\"><path d=\"{}\" fill=\"{}\"/></svg>\n",
            d.trim(), fill,
        ));
    }
}

/// COLORREF (BGR) → CSS 색상 문자열 변환
fn color_to_css(color: u32) -> String {
    let b = (color >> 16) & 0xFF;
    let g = (color >> 8) & 0xFF;
    let r = color & 0xFF;
    format!("#{:02x}{:02x}{:02x}", r, g, b)
}

/// HTML 특수문자 이스케이프
fn escape_html(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => result.push_str("&amp;"),
            '<' => result.push_str("&lt;"),
            '>' => result.push_str("&gt;"),
            '"' => result.push_str("&quot;"),
            // HTML 허용 제어문자: TAB, LF, CR만 통과, 나머지 제거
            '\u{09}' | '\u{0A}' | '\u{0D}' => result.push(c),
            c if c < '\u{0020}' => {} // 제어문자 제거
            _ => result.push(c),
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_html_begin_end_page() {
        let mut renderer = HtmlRenderer::new();
        renderer.begin_page(800.0, 600.0);
        renderer.end_page();
        let output = renderer.output();
        assert!(output.contains("hwp-page"));
        assert!(output.contains("width:800px"));
        assert!(output.ends_with("</div>\n"));
    }

    #[test]
    fn test_html_draw_text() {
        let mut renderer = HtmlRenderer::new();
        renderer.begin_page(800.0, 600.0);
        renderer.draw_text("테스트", 10.0, 20.0, &TextStyle {
            font_size: 14.0,
            bold: true,
            italic: true,
            ..Default::default()
        });
        let output = renderer.output();
        assert!(output.contains("font-weight:bold"));
        assert!(output.contains("font-style:italic"));
    }

    #[test]
    fn test_html_draw_rect() {
        let mut renderer = HtmlRenderer::new();
        renderer.begin_page(800.0, 600.0);
        renderer.draw_rect(0.0, 0.0, 100.0, 50.0, 0.0, &ShapeStyle {
            fill_color: Some(0x00FF0000),
            ..Default::default()
        });
        let output = renderer.output();
        assert!(output.contains("hwp-rect"));
        assert!(output.contains("background:#0000ff")); // BGR → RGB
    }

    #[test]
    fn test_html_escape() {
        assert_eq!(escape_html("<script>alert('xss')</script>"),
            "&lt;script&gt;alert('xss')&lt;/script&gt;");
    }

    #[test]
    fn test_html_render_tree() {
        use super::super::render_tree::*;

        let tree = PageRenderTree::new(0, 800.0, 600.0);
        let mut renderer = HtmlRenderer::new();
        renderer.render_tree(&tree);
        let output = renderer.output();
        assert!(output.contains("hwp-page"));
    }

    #[test]
    fn test_draw_image_png() {
        let mut renderer = HtmlRenderer::new();
        renderer.begin_page(800.0, 600.0);
        // Minimal PNG header (8 bytes)
        let png_data = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00];
        renderer.draw_image(&png_data, 10.0, 20.0, 100.0, 50.0);
        let output = renderer.output();
        assert!(output.contains("<img"));
        assert!(output.contains("data:image/png;base64,"));
        assert!(output.contains("hwp-image"));
    }

    #[test]
    fn test_draw_image_jpeg() {
        let mut renderer = HtmlRenderer::new();
        renderer.begin_page(800.0, 600.0);
        let jpeg_data = [0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, 0x4A, 0x46];
        renderer.draw_image(&jpeg_data, 0.0, 0.0, 200.0, 150.0);
        let output = renderer.output();
        assert!(output.contains("data:image/jpeg;base64,"));
    }

    #[test]
    fn test_draw_image_unknown_format() {
        let mut renderer = HtmlRenderer::new();
        renderer.begin_page(800.0, 600.0);
        let unknown_data = [0x00, 0x01, 0x02, 0x03];
        renderer.draw_image(&unknown_data, 5.0, 5.0, 50.0, 50.0);
        let output = renderer.output();
        assert!(output.contains("data:application/octet-stream;base64,"));
    }
}

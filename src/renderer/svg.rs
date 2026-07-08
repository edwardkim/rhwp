//! SVG 렌더러 (2차 백엔드)
//!
//! 렌더 트리를 SVG 문자열로 변환한다.
//! 정적 출력(인쇄, PDF 변환 등)에 적합하다.

use super::composer::{
    decode_pua_overlap_number, expand_pua_render_text, pua_to_display_text, CharOverlapInfo,
};
use super::form_caption::display_form_caption;
pub(crate) use super::image_resolver::{
    bmp_bytes_to_png_bytes, detect_image_mime_type, pcx_bytes_to_png_bytes,
    real_picture_watermark_bytes_to_hancom_tone_png_bytes,
    real_picture_watermark_fill_bytes_to_hancom_tone_png_bytes,
    watermark_jpeg_bytes_to_hancom_baked_png_bytes,
};
use super::pua_oldhangul::map_pua_old_hangul;
use super::render_tree::{
    BoundingBox, FormObjectNode, ImageNode, PageBackgroundImage, PageRenderTree, RenderNode,
    RenderNodeType, ShapeTransform, LEGACY_IMAGE_WATERMARK_OPACITY,
    REAL_PICTURE_WATERMARK_FILL_OPACITY, REAL_PICTURE_WATERMARK_PAGE_OPACITY,
};
use super::{
    clamp_tab_leader_end_x, GradientFillInfo, LineStyle, PathCommand, PatternFillInfo, Renderer,
    ShapeStyle, StrokeDash, TextStyle,
};

/// Hanyang-PUA 옛한글 코드포인트를 KS X 1026-1:2007 자모 시퀀스로 확장.
/// PUA 가 없으면 원본 문자열 그대로 반환 (allocation 없음).
fn expand_pua_old_hangul(text: &str) -> String {
    if !text.chars().any(|ch| map_pua_old_hangul(ch).is_some()) {
        return text.to_string();
    }
    let mut out = String::with_capacity(text.len() * 2);
    for ch in text.chars() {
        if let Some(jamos) = map_pua_old_hangul(ch) {
            out.extend(jamos.iter().copied());
        } else {
            out.push(ch);
        }
    }
    out
}
use super::layout::{compute_char_positions, split_into_clusters};
use crate::model::control::FormType;
use crate::model::style::{ImageFillMode, UnderlineType};
use base64::Engine;

const TEXT_MARK_CLIP_RIGHT_PAD: f64 = 48.0;

/// SVG 폰트 임베딩 모드
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum FontEmbedMode {
    /// 폰트 임베딩 없음 (CSS font-family 체인만)
    #[default]
    None,
    /// @font-face + local() 참조만 (데이터 미포함)
    Style,
    /// 사용 글자만 서브셋 추출 + base64 임베딩
    Subset,
    /// 전체 폰트 base64 임베딩
    Full,
}

/// SVG 렌더러
pub struct SvgRenderer {
    /// SVG 출력 버퍼
    output: String,
    /// 그라데이션 정의 버퍼 (<defs> 내부)
    defs: Vec<String>,
    /// 그라데이션 ID 카운터
    gradient_counter: u32,
    /// 클립/패턴 ID 카운터
    clip_counter: u32,
    /// <defs> 삽입 위치 (begin_page 후 기록)
    defs_insert_pos: usize,
    /// 페이지 폭
    width: f64,
    /// 페이지 높이
    height: f64,
    /// 문단부호(¶) 표시 여부
    pub show_paragraph_marks: bool,
    /// 조판부호 표시 여부 (개체 마커)
    pub show_control_codes: bool,
    /// 디버그 오버레이 표시 여부
    pub debug_overlay: bool,
    /// 디버그 오버레이용: 문단별 경계 수집 (pi → bbox)
    overlay_para_bounds: std::collections::HashMap<usize, OverlayBounds>,
    /// 디버그 오버레이용: 표 경계 수집
    overlay_table_bounds: Vec<OverlayTableInfo>,
    /// 디버그 오버레이용: 이미지 경계 수집
    overlay_image_bounds: Vec<OverlayImageInfo>,
    /// 디버그 오버레이용: vpos=0 리셋 위치 수집 (문단 첫 줄 제외)
    overlay_vpos_resets: Vec<OverlayVposReset>,
    /// 디버그 오버레이용: 표/머리말/꼬리말 내부 깊이 (셀 내·헤더 문단 제외)
    overlay_skip_depth: u32,
    /// 디버그 오버레이용: 현재 페이지의 메인 섹션 인덱스 (-1이면 미설정)
    overlay_page_section: i32,
    /// defs 내 중복 방지용 ID 집합 (화살표 마커, 이미지 효과 필터 등)
    defs_ids: std::collections::HashSet<String>,
    /// 폰트 임베딩 모드
    pub font_embed_mode: FontEmbedMode,
    /// 추가 폰트 탐색 경로
    pub font_paths: Vec<std::path::PathBuf>,
    /// 사용된 폰트별 codepoint 수집 (font_family → codepoints)
    font_codepoints: std::collections::HashMap<String, std::collections::HashSet<char>>,
}

/// 디버그 오버레이용 문단 경계 정보
struct OverlayBounds {
    section_index: usize,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

/// 디버그 오버레이용 vpos=0 리셋 마커
struct OverlayVposReset {
    section_index: usize,
    para_index: usize,
    line_index: u32,
    /// 줄 시작 y (px)
    y: f64,
    /// 줄 시작 x (px)
    x: f64,
    /// 줄 폭 (px)
    width: f64,
}

/// 디버그 오버레이용 표 정보
struct OverlayTableInfo {
    section_index: usize,
    para_index: usize,
    control_index: usize,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    row_count: u16,
    col_count: u16,
}

/// 디버그 오버레이용 이미지 정보
struct OverlayImageInfo {
    section_index: usize,
    para_index: usize,
    control_index: usize,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

impl SvgRenderer {
    pub fn new() -> Self {
        Self {
            output: String::new(),
            defs: Vec::new(),
            gradient_counter: 0,
            clip_counter: 0,
            defs_insert_pos: 0,
            width: 0.0,
            height: 0.0,
            show_paragraph_marks: false,
            show_control_codes: false,
            debug_overlay: false,
            overlay_para_bounds: std::collections::HashMap::new(),
            overlay_table_bounds: Vec::new(),
            overlay_image_bounds: Vec::new(),
            overlay_vpos_resets: Vec::new(),
            overlay_skip_depth: 0,
            overlay_page_section: -1,
            defs_ids: std::collections::HashSet::new(),
            font_embed_mode: FontEmbedMode::None,
            font_paths: Vec::new(),
            font_codepoints: std::collections::HashMap::new(),
        }
    }

    /// 생성된 SVG 문자열 반환
    pub fn output(&self) -> &str {
        &self.output
    }

    /// 수집된 폰트별 사용 글자 목록 반환
    pub fn font_codepoints(
        &self,
    ) -> &std::collections::HashMap<String, std::collections::HashSet<char>> {
        &self.font_codepoints
    }

    /// 렌더 트리를 SVG로 렌더링
    pub fn render_tree(&mut self, tree: &PageRenderTree) {
        self.render_node(&tree.root);
    }

    /// [Issue #1167/#1197] 노드의 z-order plane 키 (작을수록 먼저=아래).
    /// SVG는 단일 스트림으로 출력하므로 웹/CanvasKit의 multi-layer 합성 순서를 직접
    /// 보존해야 한다:
    /// 페이지 배경(0) → 바탕쪽(1) → BehindText 객체(2) → 일반 Flow 콘텐츠(3)
    /// → InFrontOfText 객체(4).
    /// 페이지 배경(흰 바탕·테두리·배경 워터마크)은 반드시 가장 먼저 그려야 한다.
    /// 그러지 않으면 root 레벨에서 BehindText 워터마크가 PageBackground 보다 앞으로
    /// 정렬되어, 흰 배경 rect 가 워터마크를 덮어버린다(#1167 1차 회귀).
    /// 바탕쪽은 한컴의 "본문 뒤" 배경 성격이므로 BehindText 용지 기준 객체보다
    /// 먼저 그려야 한다. 그렇지 않으면 바탕쪽의 전체 페이지 그림이 #1197의 최종
    /// 표시용 BehindText 표를 다시 덮는다.
    /// #1197부터는 RenderNode.layer 가 있으면 표/도형도 이미지와 같은 plane 계약을 따른다.
    fn node_z_plane(node: &RenderNode) -> u8 {
        if matches!(&node.node_type, RenderNodeType::PageBackground(_)) {
            return 0;
        }
        if matches!(&node.node_type, RenderNodeType::MasterPage) {
            return 1;
        }
        if let Some(layer) = node.layer {
            if let Some(text_wrap) = layer.text_wrap {
                return match text_wrap {
                    crate::model::shape::TextWrap::BehindText => 2,
                    crate::model::shape::TextWrap::InFrontOfText => 4,
                    _ => 3,
                };
            }
        }
        match &node.node_type {
            RenderNodeType::Image(img) => match img.text_wrap {
                Some(crate::model::shape::TextWrap::BehindText) => 2,
                Some(crate::model::shape::TextWrap::InFrontOfText) => 4,
                _ => 3,
            },
            _ => 3,
        }
    }

    fn node_z_sort_key(node: &RenderNode) -> (u8, i32, u32) {
        let layer = node.layer;
        (
            Self::node_z_plane(node),
            layer.map(|l| l.z_order).unwrap_or(0),
            layer.map(|l| l.stable_index).unwrap_or(0),
        )
    }

    /// [Issue #1167/#1197] 자식 중 BehindText/InFrontOfText 객체가 섞여 있어 plane
    /// 재정렬이 필요한지. 대부분의 노드는 Flow 만 가지므로 정렬 비용을 피한다.
    fn children_need_plane_reorder(node: &RenderNode) -> bool {
        node.children.iter().any(|c| Self::node_z_plane(c) != 3)
    }

    /// 개별 노드를 SVG로 렌더링
    fn render_node(&mut self, node: &RenderNode) {
        if !node.visible {
            return;
        }

        match &node.node_type {
            RenderNodeType::Page(page) => {
                self.begin_page(page.width, page.height);
            }
            RenderNodeType::PageBackground(bg) => {
                // 배경색 먼저 (이미지가 투명 부분을 가질 수 있으므로)
                if let Some(color) = bg.background_color {
                    let color_str = color_to_svg(color);
                    self.output.push_str(&format!(
                        "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"{}\"/>\n",
                        node.bbox.x, node.bbox.y, node.bbox.width, node.bbox.height, color_str,
                    ));
                }
                // 그라데이션 (배경색 위에 덮음)
                if let Some(grad) = &bg.gradient {
                    let grad_id = self.create_gradient_def(grad);
                    self.output.push_str(&format!(
                        "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"url(#{})\"/>\n",
                        node.bbox.x, node.bbox.y, node.bbox.width, node.bbox.height, grad_id,
                    ));
                }
                // 이미지 (최상위)
                if let Some(img) = &bg.image {
                    self.render_page_background_image(img, &node.bbox);
                }
            }
            RenderNodeType::TextRun(run) => {
                // 폰트 임베딩: 사용된 폰트/글자 수집
                if self.font_embed_mode != FontEmbedMode::None && !run.style.font_family.is_empty()
                {
                    let codepoints = self
                        .font_codepoints
                        .entry(run.style.font_family.clone())
                        .or_default();
                    for ch in run.text.chars() {
                        if !ch.is_control() {
                            codepoints.insert(ch);
                        }
                    }
                }
                if let Some(ref overlap) = run.char_overlap {
                    // 글자겹침(CharOverlap) 렌더링: 각 문자에 테두리 도형 + 텍스트
                    self.draw_char_overlap(
                        &run.text,
                        &run.style,
                        overlap,
                        node.bbox.x,
                        node.bbox.y,
                        node.bbox.width,
                        node.bbox.height,
                    );
                } else if run.rotation != 0.0 {
                    // 회전 텍스트: bbox 중앙에 중앙 정렬 후 회전
                    let cx = node.bbox.x + node.bbox.width / 2.0;
                    let cy = node.bbox.y + node.bbox.height / 2.0;
                    let color = color_to_svg(run.style.color);
                    let font_size = if run.style.font_size > 0.0 {
                        run.style.font_size
                    } else {
                        12.0
                    };
                    let font_family = if run.style.font_family.is_empty() {
                        "sans-serif".to_string()
                    } else {
                        let fb = super::generic_fallback(&run.style.font_family);
                        format!("{},{}", run.style.font_family, fb)
                    };
                    let mut attrs = format!("font-family=\"{}\" font-size=\"{}\" fill=\"{}\" text-anchor=\"middle\" dominant-baseline=\"central\"",
                        escape_xml(&font_family), font_size, color);
                    if run.style.is_visually_bold() {
                        attrs.push_str(" font-weight=\"bold\"");
                    } else if run.style.is_medium_weight() {
                        attrs.push_str(" font-weight=\"500\"");
                    }
                    if run.style.italic {
                        attrs.push_str(" font-style=\"italic\"");
                    }
                    for c in run.text.chars() {
                        if c == ' ' {
                            continue;
                        }
                        self.output.push_str(&format!(
                            "<text x=\"{}\" y=\"{}\" {} transform=\"rotate({},{},{})\">{}</text>\n",
                            cx,
                            cy,
                            attrs,
                            run.rotation,
                            cx,
                            cy,
                            escape_xml(&c.to_string()),
                        ));
                    }
                } else {
                    self.draw_text(
                        &run.text,
                        node.bbox.x,
                        node.bbox.y + run.baseline,
                        &run.style,
                    );
                }
                if self.show_paragraph_marks || self.show_control_codes {
                    // 조판부호 마커 TextRun은 공백 기호 표시 건너뛰기
                    let is_marker = !matches!(
                        run.field_marker,
                        crate::renderer::render_tree::FieldMarkerType::None
                    );
                    let font_size = if run.style.font_size > 0.0 {
                        run.style.font_size
                    } else {
                        12.0
                    };
                    // 공백·탭 기호: 각 문자 위치에 오버레이
                    if !run.text.is_empty() && !is_marker {
                        let char_positions = compute_char_positions(&run.text, &run.style);
                        let mark_font_size = font_size * 0.5;
                        for (i, c) in run.text.chars().enumerate() {
                            if c == ' ' {
                                let cx = node.bbox.x + char_positions[i];
                                // ∨ 기호를 공백 영역 중앙 하단에 배치
                                let next_x = if i + 1 < char_positions.len() {
                                    node.bbox.x + char_positions[i + 1]
                                } else {
                                    node.bbox.x + node.bbox.width
                                };
                                let mid_x = (cx + next_x) / 2.0 - mark_font_size * 0.25;
                                self.output.push_str(&format!(
                                    "<text x=\"{}\" y=\"{}\" font-size=\"{}\" fill=\"#0066FF\">\u{2228}</text>\n",
                                    mid_x, node.bbox.y + run.baseline, mark_font_size,
                                ));
                            } else if c == '\t' {
                                let cx = node.bbox.x + char_positions[i];
                                self.output.push_str(&format!(
                                    "<text x=\"{}\" y=\"{}\" font-size=\"{}\" fill=\"#0066FF\">\u{2192}</text>\n",
                                    cx, node.bbox.y + run.baseline, mark_font_size,
                                ));
                            }
                        }
                    }
                    // 하드 리턴·강제 줄바꿈 기호
                    if run.is_para_end || run.is_line_break_end {
                        let mark_x = if run.text.is_empty() {
                            node.bbox.x
                        } else {
                            node.bbox.x + node.bbox.width
                        };
                        let mark = if run.is_line_break_end {
                            "\u{2193}"
                        } else {
                            "\u{21B5}"
                        };
                        self.output.push_str(&format!(
                            "<text x=\"{}\" y=\"{}\" font-size=\"{}\" fill=\"#0066FF\">{}</text>\n",
                            mark_x,
                            node.bbox.y + run.baseline,
                            font_size,
                            mark,
                        ));
                    }
                }
            }
            RenderNodeType::FootnoteMarker(marker) => {
                let sup_size = (marker.base_font_size * 0.55).max(7.0);
                let color = color_to_svg(marker.color);
                let font_family = if marker.font_family.is_empty() {
                    "sans-serif"
                } else {
                    &marker.font_family
                };
                let y = node.bbox.y + node.bbox.height * 0.4;
                self.output.push_str(&format!(
                    "<text x=\"{}\" y=\"{}\" font-family=\"{}\" font-size=\"{}\" fill=\"{}\">{}</text>\n",
                    node.bbox.x, y, escape_xml(font_family), sup_size, color, escape_xml(&marker.text),
                ));
            }
            RenderNodeType::Rectangle(rect) => {
                self.open_shape_transform(&rect.transform, &node.bbox);
                self.draw_rect_with_gradient(
                    node.bbox.x,
                    node.bbox.y,
                    node.bbox.width,
                    node.bbox.height,
                    rect.corner_radius,
                    &rect.style,
                    rect.gradient.as_deref(),
                );
            }
            RenderNodeType::Line(line) => {
                self.open_shape_transform(&line.transform, &node.bbox);
                self.draw_line(line.x1, line.y1, line.x2, line.y2, &line.style);
            }
            RenderNodeType::Ellipse(ellipse) => {
                self.open_shape_transform(&ellipse.transform, &node.bbox);
                let cx = node.bbox.x + node.bbox.width / 2.0;
                let cy = node.bbox.y + node.bbox.height / 2.0;
                self.draw_ellipse_with_gradient(
                    cx,
                    cy,
                    node.bbox.width / 2.0,
                    node.bbox.height / 2.0,
                    &ellipse.style,
                    ellipse.gradient.as_deref(),
                );
            }
            RenderNodeType::Image(img) => {
                // [shot 05] 회전 90/270° 시 bbox extent swap — 이중회전 방지.
                let eff_bbox = img.transform.effective_image_bbox(&node.bbox);
                self.open_shape_transform(&img.transform, &eff_bbox);
                self.render_image_node(img, &eff_bbox);
            }
            RenderNodeType::Path(path) => {
                self.open_shape_transform(&path.transform, &node.bbox);
                self.draw_path_with_gradient(&path.commands, &path.style, path.gradient.as_deref());
            }
            RenderNodeType::Equation(eq) => {
                // 수식 SVG 조각을 bbox 위치에 배치
                // HWP 저장 영역(bbox)과 레이아웃 산출 크기(layout_box)가 다를 수 있으므로
                // bbox 너비에 맞춰 스케일링한다. 높이는 줄 높이/여백을 포함한 영역이라
                // 식 자체를 세로로 늘리면 한컴보다 글자가 찌그러진다.
                let scale_x = if eq.layout_box.width > 0.0 && node.bbox.width > 0.0 {
                    node.bbox.width / eq.layout_box.width
                } else {
                    1.0
                };
                let scale_y = 1.0_f64;
                let needs_scale = (scale_x - 1.0).abs() > 0.01 || (scale_y - 1.0).abs() > 0.01;
                if needs_scale {
                    self.output.push_str(&format!(
                        "<g transform=\"translate({},{}) scale({:.4},{:.4})\">\n",
                        node.bbox.x, node.bbox.y, scale_x, scale_y,
                    ));
                } else {
                    self.output.push_str(&format!(
                        "<g transform=\"translate({},{})\">\n",
                        node.bbox.x, node.bbox.y,
                    ));
                }
                self.output.push_str(&eq.svg_content);
                self.output.push_str("</g>\n");
                // 폰트 임베딩: 수식에서 사용된 글자 수집
                if self.font_embed_mode != FontEmbedMode::None {
                    let codepoints = self
                        .font_codepoints
                        .entry("Latin Modern Math".to_string())
                        .or_default();
                    // SVG <text> 요소 내부의 텍스트에서 문자 추출
                    for segment in eq.svg_content.split("</text>") {
                        if let Some(start) = segment.rfind('>') {
                            for ch in segment[start + 1..].chars() {
                                codepoints.insert(ch);
                            }
                        }
                    }
                }
            }
            RenderNodeType::FormObject(form) => {
                self.render_form_object(form, &node.bbox);
            }
            RenderNodeType::RawSvg(r) => {
                // Task #195 단계 8: OOXML 차트 SVG 조각 그대로 삽입
                self.output.push_str(&r.svg);
            }
            RenderNodeType::Placeholder(ph) => {
                // Task #195: 차트/OLE placeholder (점선 테두리 + 중앙 라벨)
                let cx = node.bbox.x + node.bbox.width / 2.0;
                let cy = node.bbox.y + node.bbox.height / 2.0;
                let font_size = (node.bbox.width.min(node.bbox.height) * 0.06).clamp(12.0, 28.0);
                self.output.push_str(&format!(
                    "<rect x=\"{:.2}\" y=\"{:.2}\" width=\"{:.2}\" height=\"{:.2}\" fill=\"{}\" stroke=\"{}\" stroke-width=\"1\" stroke-dasharray=\"6 3\"/>\n",
                    node.bbox.x, node.bbox.y, node.bbox.width, node.bbox.height,
                    color_to_svg(ph.fill_color), color_to_svg(ph.stroke_color),
                ));
                self.output.push_str(&format!(
                    "<text x=\"{:.2}\" y=\"{:.2}\" font-family=\"sans-serif\" font-size=\"{:.1}\" fill=\"{}\" text-anchor=\"middle\" dominant-baseline=\"central\">{}</text>\n",
                    cx, cy, font_size, color_to_svg(ph.stroke_color), escape_xml(&ph.label),
                ));
            }
            RenderNodeType::Body {
                clip_rect: Some(cr),
            } => {
                let clip_id = format!("body-clip-{}", node.id);
                let right_pad = if self.show_paragraph_marks || self.show_control_codes {
                    TEXT_MARK_CLIP_RIGHT_PAD
                } else {
                    0.0
                };
                self.defs.push(format!(
                    "<clipPath id=\"{}\"><rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\"/></clipPath>\n",
                    clip_id,
                    cr.x,
                    cr.y,
                    cr.width + right_pad,
                    cr.height,
                ));
                self.output
                    .push_str(&format!("<g clip-path=\"url(#{})\">", clip_id));
            }
            RenderNodeType::TableCell(ref tc) if tc.clip => {
                let clip_id = format!("cell-clip-{}", node.id);
                self.defs.push(format!(
                    "<clipPath id=\"{}\"><rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\"/></clipPath>\n",
                    clip_id, node.bbox.x, node.bbox.y, node.bbox.width, node.bbox.height,
                ));
                self.output
                    .push_str(&format!("<g clip-path=\"url(#{})\">", clip_id));
            }
            RenderNodeType::TextBox => {
                let clip_id = format!("textbox-clip-{}", node.id);
                self.defs.push(format!(
                    "<clipPath id=\"{}\"><rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\"/></clipPath>\n",
                    clip_id, node.bbox.x, node.bbox.y, node.bbox.width, node.bbox.height,
                ));
                self.output
                    .push_str(&format!("<g clip-path=\"url(#{})\">", clip_id));
            }
            _ => {}
        }

        // 디버그 오버레이: 문단/표 경계 수집 (셀 내부·머리말·꼬리말 제외)
        if self.debug_overlay {
            match &node.node_type {
                RenderNodeType::TextLine(tl) => {
                    if self.overlay_skip_depth == 0 {
                        // section_index가 없는 TextLine은 Shape 내부 등 비본문 요소 — 제외
                        if let (Some(pi), Some(si)) = (tl.para_index, tl.section_index) {
                            // 페이지 메인 섹션 자동 감지 (처음 등장하는 섹션이 메인)
                            if self.overlay_page_section == -1 {
                                self.overlay_page_section = si as i32;
                            }
                            // 페이지 메인 섹션이 아닌 섹션 문단은 오버레이에서 제외
                            // (구역 정의 섹션, 다른 섹션 나누기 문단 등)
                            if si as i32 != self.overlay_page_section {
                                // skip
                            } else {
                                // (section, para) 복합키로 섹션 간 구분
                                let key = si * 100000 + pi;
                                let entry =
                                    self.overlay_para_bounds
                                        .entry(key)
                                        .or_insert(OverlayBounds {
                                            section_index: si,
                                            x: node.bbox.x,
                                            y: node.bbox.y,
                                            width: node.bbox.width,
                                            height: node.bbox.height,
                                        });
                                // 기존 bounds 확장 (여러 줄이 하나의 문단)
                                let min_x = entry.x.min(node.bbox.x);
                                let min_y = entry.y.min(node.bbox.y);
                                let max_x =
                                    (entry.x + entry.width).max(node.bbox.x + node.bbox.width);
                                let max_y =
                                    (entry.y + entry.height).max(node.bbox.y + node.bbox.height);
                                entry.x = min_x;
                                entry.y = min_y;
                                entry.width = max_x - min_x;
                                entry.height = max_y - min_y;

                                // vpos=0 리셋 검출: 문단 첫 줄(line 0) 제외하고 vertical_pos == 0
                                if let (Some(li), Some(vp)) = (tl.line_index, tl.vpos) {
                                    if li > 0 && vp == 0 {
                                        self.overlay_vpos_resets.push(OverlayVposReset {
                                            section_index: si,
                                            para_index: pi,
                                            line_index: li,
                                            y: node.bbox.y,
                                            x: node.bbox.x,
                                            width: node.bbox.width,
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
                RenderNodeType::Table(tbl) => {
                    if let (Some(pi), Some(ci)) = (tbl.para_index, tbl.control_index) {
                        if self.overlay_skip_depth == 0 {
                            let tbl_si = tbl.section_index.unwrap_or(0);
                            // 페이지 메인 섹션 자동 감지
                            if self.overlay_page_section == -1 {
                                self.overlay_page_section = tbl_si as i32;
                            }
                            if tbl_si as i32 == self.overlay_page_section {
                                self.overlay_table_bounds.push(OverlayTableInfo {
                                    section_index: tbl_si,
                                    para_index: pi,
                                    control_index: ci,
                                    x: node.bbox.x,
                                    y: node.bbox.y,
                                    width: node.bbox.width,
                                    height: node.bbox.height,
                                    row_count: tbl.row_count,
                                    col_count: tbl.col_count,
                                });
                                // 표를 포함하는 문단 bounds도 확장 (텍스트 없는 문단 처리)
                                let key = tbl_si * 100000 + pi;
                                let entry =
                                    self.overlay_para_bounds
                                        .entry(key)
                                        .or_insert(OverlayBounds {
                                            section_index: tbl_si,
                                            x: node.bbox.x,
                                            y: node.bbox.y,
                                            width: node.bbox.width,
                                            height: node.bbox.height,
                                        });
                                let min_x = entry.x.min(node.bbox.x);
                                let min_y = entry.y.min(node.bbox.y);
                                let max_x =
                                    (entry.x + entry.width).max(node.bbox.x + node.bbox.width);
                                let max_y =
                                    (entry.y + entry.height).max(node.bbox.y + node.bbox.height);
                                entry.x = min_x;
                                entry.y = min_y;
                                entry.width = max_x - min_x;
                                entry.height = max_y - min_y;
                            }
                        }
                    }
                    self.overlay_skip_depth += 1;
                }
                RenderNodeType::Image(img) => {
                    if let (Some(pi), Some(ci)) = (img.para_index, img.control_index) {
                        if self.overlay_skip_depth == 0 {
                            let img_si = img.section_index.unwrap_or(0);
                            if self.overlay_page_section == -1 {
                                self.overlay_page_section = img_si as i32;
                            }
                            if img_si as i32 == self.overlay_page_section {
                                self.overlay_image_bounds.push(OverlayImageInfo {
                                    section_index: img_si,
                                    para_index: pi,
                                    control_index: ci,
                                    x: node.bbox.x,
                                    y: node.bbox.y,
                                    width: node.bbox.width,
                                    height: node.bbox.height,
                                });
                            }
                        }
                    }
                }
                // 머리말/꼬리말/바탕쪽/각주/텍스트박스/그룹: body 외 영역 제외
                RenderNodeType::Header
                | RenderNodeType::Footer
                | RenderNodeType::MasterPage
                | RenderNodeType::FootnoteArea
                | RenderNodeType::TextBox
                | RenderNodeType::Group(_) => {
                    self.overlay_skip_depth += 1;
                }
                _ => {}
            }
        }

        // [Issue #1167] 자식을 z-order plane 순서로 순회한다.
        // SVG 는 후순위가 위로 합성되므로, BehindText 그림(워터마크)은 본문(Flow)
        // 보다 먼저, InFrontOfText 그림(직인 등)은 본문보다 나중에 그려야 한다.
        // PaintOp replay plane(background → behindText → flow → inFrontOfText)과
        // 동일 의미. 같은 plane 내부는 안정 정렬로 기존 트리 순서를 보존한다.
        // (PNG=native Skia / 웹캔버스=CanvasKit 는 PaintOp replay plane 으로 이미
        //  정정됨 — PR #1163 / #1017. 본 변경은 SVG 경로 정합.)
        if Self::children_need_plane_reorder(node) {
            let mut ordered: Vec<&RenderNode> = node.children.iter().collect();
            ordered.sort_by_key(|c| Self::node_z_sort_key(c));
            for child in ordered {
                self.render_node(child);
            }
        } else {
            for child in &node.children {
                self.render_node(child);
            }
        }

        // 디버그 오버레이: skip 깊이 복원
        if self.debug_overlay {
            match &node.node_type {
                RenderNodeType::Table(_)
                | RenderNodeType::Header
                | RenderNodeType::Footer
                | RenderNodeType::MasterPage
                | RenderNodeType::FootnoteArea
                | RenderNodeType::TextBox
                | RenderNodeType::Group(_) => {
                    self.overlay_skip_depth = self.overlay_skip_depth.saturating_sub(1);
                }
                _ => {}
            }
        }

        // 도형 변환 그룹 종료
        self.close_shape_transform(&node.node_type);

        // 조판부호 개체 마커 (붉은색 대괄호) — 조판부호 ON일 때만
        if self.show_control_codes {
            let label = match &node.node_type {
                RenderNodeType::Table(_) => Some("[표]"),
                RenderNodeType::Image(_) => Some("[그림]"),
                RenderNodeType::TextBox => Some("[글상자]"),
                RenderNodeType::Equation(_) => Some("[수식]"),
                RenderNodeType::Header => Some("[머리말]"),
                RenderNodeType::Footer => Some("[꼬리말]"),
                RenderNodeType::FootnoteArea => Some("[각주]"),
                _ => None,
            };
            if let Some(label) = label {
                let fs = 10.0; // 조판부호 고정 크기
                self.output.push_str(&format!(
                    "<text x=\"{}\" y=\"{}\" font-size=\"{}\" fill=\"#CC3333\">{}</text>\n",
                    node.bbox.x,
                    node.bbox.y + fs,
                    fs,
                    label,
                ));
            }
        }

        // 셀 클리핑 그룹 종료
        if matches!(&node.node_type, RenderNodeType::TableCell(tc) if tc.clip) {
            self.output.push_str("</g>\n");
        }

        // TextBox 클리핑 그룹 종료
        if matches!(node.node_type, RenderNodeType::TextBox) {
            self.output.push_str("</g>\n");
        }

        // Body 클리핑 그룹 종료
        if matches!(node.node_type, RenderNodeType::Body { clip_rect: Some(_) }) {
            self.output.push_str("</g>\n");
        }

        // 페이지 종료 태그
        if matches!(node.node_type, RenderNodeType::Page(_)) {
            self.end_page();
        }
    }

    /// 도형 변환(회전/대칭)이 있으면 `<g transform="...">` 래퍼를 연다.
    fn open_shape_transform(&mut self, transform: &ShapeTransform, bbox: &BoundingBox) {
        if !transform.has_transform() {
            return;
        }
        let cx = bbox.x + bbox.width / 2.0;
        let cy = bbox.y + bbox.height / 2.0;
        let mut parts = Vec::new();
        // [Task #1067] SVG transform 은 left-to-right 적용 (첫 transform 이 마지막 영향).
        // 한컴 정답지 시각 표준: 도형이 자체 좌표계 기준으로 먼저 회전 후 flip 적용.
        // SVG 에서 동일 결과 = "translate(flip) scale(-1,1) rotate(-θ)"
        // (flip 와 함께 회전 시 각도 부호 반전 필요).
        let flip_negate_rotation = transform.horz_flip ^ transform.vert_flip;
        if transform.horz_flip {
            parts.push(format!("translate({},0) scale(-1,1)", cx * 2.0));
        }
        if transform.vert_flip {
            parts.push(format!("translate(0,{}) scale(1,-1)", cy * 2.0));
        }
        if transform.rotation != 0.0 {
            let effective_rotation = if flip_negate_rotation {
                -transform.rotation
            } else {
                transform.rotation
            };
            parts.push(format!("rotate({},{},{})", effective_rotation, cx, cy));
        }
        self.output
            .push_str(&format!("<g transform=\"{}\">\n", parts.join(" ")));
    }

    /// 도형 변환 그룹을 닫는다 (open_shape_transform에 대응).
    fn close_shape_transform(&mut self, node_type: &RenderNodeType) {
        let transform = match node_type {
            RenderNodeType::Rectangle(r) => &r.transform,
            RenderNodeType::Line(l) => &l.transform,
            RenderNodeType::Ellipse(e) => &e.transform,
            RenderNodeType::Image(i) => &i.transform,
            RenderNodeType::Path(p) => &p.transform,
            _ => return,
        };
        if transform.has_transform() {
            self.output.push_str("</g>\n");
        }
    }

    /// 그라데이션 SVG 정의 생성, ID 반환
    fn create_gradient_def(&mut self, grad: &GradientFillInfo) -> String {
        self.gradient_counter += 1;
        let id = format!("grad{}", self.gradient_counter);

        let stops = Self::build_gradient_stops(grad);

        let def = match grad.gradient_type {
            2 => {
                // 원형 (Radial)
                let cx = grad.center_x as f64;
                let cy = grad.center_y as f64;
                format!(
                    "<radialGradient id=\"{}\" cx=\"{}%\" cy=\"{}%\" r=\"50%\" fx=\"{}%\" fy=\"{}%\">\n{}</radialGradient>\n",
                    id, cx, cy, cx, cy, stops,
                )
            }
            _ => {
                // 선형 (Linear) — gradient_type 1(줄무늬), 3(원뿔), 4(사각) 모두 선형으로 근사
                let (x1, y1, x2, y2) = Self::angle_to_svg_coords(grad.angle);
                format!(
                    "<linearGradient id=\"{}\" x1=\"{}%\" y1=\"{}%\" x2=\"{}%\" y2=\"{}%\">\n{}</linearGradient>\n",
                    id, x1, y1, x2, y2, stops,
                )
            }
        };

        self.defs.push(def);
        id
    }

    /// 패턴 채우기 SVG 정의 생성, ID 반환
    fn create_pattern_def(&mut self, info: &PatternFillInfo) -> String {
        self.clip_counter += 1;
        let id = format!("pat{}", self.clip_counter);
        let bg = color_to_svg(info.background_color);
        let fg = color_to_svg(info.pattern_color);
        let sz = 6; // 패턴 타일 크기 (px)

        // HWP 패턴 종류 (0-based, 표 31 참조): 0=가로줄, 1=세로줄, 2=역대각선, 3=대각선, 4=십자, 5=격자
        let lines = match info.pattern_type {
            0 => // 가로줄 (- - - -)
                format!("<rect width=\"{sz}\" height=\"{sz}\" fill=\"{bg}\"/>\
                         <line x1=\"0\" y1=\"3\" x2=\"{sz}\" y2=\"3\" stroke=\"{fg}\" stroke-width=\"1\"/>"),
            1 => // 세로줄 (|||||)
                format!("<rect width=\"{sz}\" height=\"{sz}\" fill=\"{bg}\"/>\
                         <line x1=\"3\" y1=\"0\" x2=\"3\" y2=\"{sz}\" stroke=\"{fg}\" stroke-width=\"1\"/>"),
            2 => // 대각선 (/////)
                format!("<rect width=\"{sz}\" height=\"{sz}\" fill=\"{bg}\"/>\
                         <line x1=\"{sz}\" y1=\"0\" x2=\"0\" y2=\"{sz}\" stroke=\"{fg}\" stroke-width=\"1\"/>"),
            3 => // 역대각선 (\\\\\)
                format!("<rect width=\"{sz}\" height=\"{sz}\" fill=\"{bg}\"/>\
                         <line x1=\"0\" y1=\"0\" x2=\"{sz}\" y2=\"{sz}\" stroke=\"{fg}\" stroke-width=\"1\"/>"),
            4 => // 십자 (+++++)
                format!("<rect width=\"{sz}\" height=\"{sz}\" fill=\"{bg}\"/>\
                         <line x1=\"3\" y1=\"0\" x2=\"3\" y2=\"{sz}\" stroke=\"{fg}\" stroke-width=\"1\"/>\
                         <line x1=\"0\" y1=\"3\" x2=\"{sz}\" y2=\"3\" stroke=\"{fg}\" stroke-width=\"1\"/>"),
            5 => // 격자 (xxxxx)
                format!("<rect width=\"{sz}\" height=\"{sz}\" fill=\"{bg}\"/>\
                         <line x1=\"0\" y1=\"0\" x2=\"{sz}\" y2=\"{sz}\" stroke=\"{fg}\" stroke-width=\"1\"/>\
                         <line x1=\"{sz}\" y1=\"0\" x2=\"0\" y2=\"{sz}\" stroke=\"{fg}\" stroke-width=\"1\"/>"),
            _ => // 알 수 없는 패턴: 단색
                format!("<rect width=\"{sz}\" height=\"{sz}\" fill=\"{bg}\"/>"),
        };

        let def = format!(
            "<pattern id=\"{}\" patternUnits=\"userSpaceOnUse\" width=\"{}\" height=\"{}\">{}</pattern>\n",
            id, sz, sz, lines
        );
        self.defs.push(def);
        id
    }

    /// ShapeStyle에서 SVG fill 속성 문자열 생성
    fn build_fill_attr(
        &mut self,
        style: &ShapeStyle,
        gradient: Option<&GradientFillInfo>,
    ) -> String {
        if let Some(grad) = gradient {
            let grad_id = self.create_gradient_def(grad);
            format!(" fill=\"url(#{})\"", grad_id)
        } else if let Some(ref pat) = style.pattern {
            let pat_id = self.create_pattern_def(pat);
            format!(" fill=\"url(#{})\"", pat_id)
        } else if let Some(fill) = style.fill_color {
            format!(" fill=\"{}\"", color_to_svg(fill))
        } else {
            " fill=\"none\"".to_string()
        }
    }

    /// 화살표 마커 SVG 정의 생성 (중복 시 기존 ID 반환)
    ///
    /// HWP 화살표 크기(0-8): {작은,중간,큰} × {작은,중간,큰} (너비 × 길이)
    /// 선 두께와 길이를 고려하여 마커 크기 결정
    fn ensure_arrow_marker(
        &mut self,
        color: &str,
        stroke_width: f64,
        line_len: f64,
        arrow: &super::ArrowStyle,
        arrow_size: u8,
        is_start: bool,
    ) -> String {
        let type_name = match arrow {
            super::ArrowStyle::Arrow => "arrow",
            super::ArrowStyle::ConcaveArrow => "concave",
            super::ArrowStyle::OpenDiamond => "odiamond",
            super::ArrowStyle::OpenCircle => "ocircle",
            super::ArrowStyle::OpenSquare => "osquare",
            super::ArrowStyle::Diamond => "diamond",
            super::ArrowStyle::Circle => "circle",
            super::ArrowStyle::Square => "square",
            super::ArrowStyle::None => "none",
        };
        let dir = if is_start { "s" } else { "e" };
        let color_id = color.replace('#', "");
        let id = format!("mk-{}-{}-{}-{}", type_name, dir, color_id, arrow_size);

        if !self.defs_ids.insert(id.clone()) {
            return id;
        }

        // HWP 화살표 크기 → 너비/길이 배율
        // arrow_size: 0=작은-작은, 1=작은-중간, 2=작은-큰,
        //             3=중간-작은, 4=중간-중간, 5=중간-큰,
        //             6=큰-작은, 7=큰-중간, 8=큰-큰
        let width_level = arrow_size / 3; // 0=작은, 1=중간, 2=큰
        let length_level = arrow_size % 3; // 0=작은, 1=중간, 2=큰

        // 너비 배율 (선 두께 대비 화살표 높이)
        let width_mult = match width_level {
            0 => 1.5, // 작은: 선 두께의 1.5배
            1 => 2.5, // 중간: 선 두께의 2.5배
            _ => 3.5, // 큰: 선 두께의 3.5배
        };
        // 길이 배율 (화살표 높이 대비 길이)
        let length_mult = match length_level {
            0 => 1.0, // 작은
            1 => 1.5, // 중간
            _ => 2.0, // 큰
        };

        let arrow_h = (stroke_width * width_mult).max(3.0);
        let arrow_w = (arrow_h * length_mult).min(line_len * 0.3); // 선 길이의 30% 이하
        let half_h = arrow_h / 2.0;

        let def = match arrow {
            super::ArrowStyle::Arrow => {
                // 선이 화살표 길이만큼 줄어드므로 refX는 화살표 밑변(base) 위치
                // start: refX=arrow_w (밑변이 줄어든 시작점에 정렬, 팁은 원래 시작점 방향)
                // end:   refX=0 (밑변이 줄어든 끝점에 정렬, 팁은 원래 끝점 방향)
                if is_start {
                    format!(
                        "<marker id=\"{}\" viewBox=\"0 0 {} {}\" refX=\"{}\" refY=\"{}\" markerWidth=\"{}\" markerHeight=\"{}\" orient=\"auto\" markerUnits=\"userSpaceOnUse\">\
                        <path d=\"M {} 0 L 0 {} L {} {}\" fill=\"{}\" stroke=\"none\"/></marker>\n",
                        id, arrow_w, arrow_h, arrow_w, half_h, arrow_w, arrow_h,
                        arrow_w, half_h, arrow_w, arrow_h, color,
                    )
                } else {
                    format!(
                        "<marker id=\"{}\" viewBox=\"0 0 {} {}\" refX=\"0\" refY=\"{}\" markerWidth=\"{}\" markerHeight=\"{}\" orient=\"auto\" markerUnits=\"userSpaceOnUse\">\
                        <path d=\"M 0 0 L {} {} L 0 {}\" fill=\"{}\" stroke=\"none\"/></marker>\n",
                        id, arrow_w, arrow_h, half_h, arrow_w, arrow_h,
                        arrow_w, half_h, arrow_h, color,
                    )
                }
            }
            super::ArrowStyle::ConcaveArrow => {
                let concave = arrow_w * 0.3;
                if is_start {
                    format!(
                        "<marker id=\"{}\" viewBox=\"0 0 {} {}\" refX=\"{}\" refY=\"{}\" markerWidth=\"{}\" markerHeight=\"{}\" orient=\"auto\" markerUnits=\"userSpaceOnUse\">\
                        <path d=\"M {} 0 L 0 {} L {} {} L {} {} Z\" fill=\"{}\" stroke=\"none\"/></marker>\n",
                        id, arrow_w, arrow_h, arrow_w, half_h, arrow_w, arrow_h,
                        arrow_w, half_h, arrow_w, arrow_h, concave, half_h, color,
                    )
                } else {
                    format!(
                        "<marker id=\"{}\" viewBox=\"0 0 {} {}\" refX=\"0\" refY=\"{}\" markerWidth=\"{}\" markerHeight=\"{}\" orient=\"auto\" markerUnits=\"userSpaceOnUse\">\
                        <path d=\"M 0 0 L {} {} L 0 {} L {} {} Z\" fill=\"{}\" stroke=\"none\"/></marker>\n",
                        id, arrow_w, arrow_h, half_h, arrow_w, arrow_h,
                        arrow_w, half_h, arrow_h, arrow_w - concave, half_h, color,
                    )
                }
            }
            super::ArrowStyle::OpenDiamond => {
                let half_w = arrow_w / 2.0;
                let sw = (stroke_width * 0.3).max(0.5);
                let ref_x = if is_start { arrow_w } else { 0.0 };
                format!(
                    "<marker id=\"{}\" viewBox=\"0 0 {} {}\" refX=\"{}\" refY=\"{}\" markerWidth=\"{}\" markerHeight=\"{}\" orient=\"auto\" markerUnits=\"userSpaceOnUse\">\
                    <path d=\"M {} 0 L {} {} L {} {} L 0 {} Z\" fill=\"white\" stroke=\"{}\" stroke-width=\"{}\"/></marker>\n",
                    id, arrow_w, arrow_h, ref_x, half_h, arrow_w, arrow_h,
                    half_w, arrow_w, half_h, half_w, arrow_h, half_h, color, sw,
                )
            }
            super::ArrowStyle::OpenCircle => {
                let half_w = arrow_w / 2.0;
                let rx = half_w * 0.8;
                let ry = half_h * 0.8;
                let sw = (stroke_width * 0.3).max(0.5);
                let ref_x = if is_start { arrow_w } else { 0.0 };
                format!(
                    "<marker id=\"{}\" viewBox=\"0 0 {} {}\" refX=\"{}\" refY=\"{}\" markerWidth=\"{}\" markerHeight=\"{}\" orient=\"auto\" markerUnits=\"userSpaceOnUse\">\
                    <ellipse cx=\"{}\" cy=\"{}\" rx=\"{}\" ry=\"{}\" fill=\"white\" stroke=\"{}\" stroke-width=\"{}\"/></marker>\n",
                    id, arrow_w, arrow_h, ref_x, half_h, arrow_w, arrow_h,
                    half_w, half_h, rx, ry, color, sw,
                )
            }
            super::ArrowStyle::OpenSquare => {
                let sw = (stroke_width * 0.3).max(0.5);
                let ref_x = if is_start { arrow_w } else { 0.0 };
                format!(
                    "<marker id=\"{}\" viewBox=\"0 0 {} {}\" refX=\"{}\" refY=\"{}\" markerWidth=\"{}\" markerHeight=\"{}\" orient=\"auto\" markerUnits=\"userSpaceOnUse\">\
                    <rect x=\"0\" y=\"0\" width=\"{}\" height=\"{}\" fill=\"white\" stroke=\"{}\" stroke-width=\"{}\"/></marker>\n",
                    id, arrow_w, arrow_h, ref_x, half_h, arrow_w, arrow_h,
                    arrow_w, arrow_h, color, sw,
                )
            }
            super::ArrowStyle::Diamond => {
                let half_w = arrow_w / 2.0;
                let ref_x = if is_start { arrow_w } else { 0.0 };
                format!(
                    "<marker id=\"{}\" viewBox=\"0 0 {} {}\" refX=\"{}\" refY=\"{}\" markerWidth=\"{}\" markerHeight=\"{}\" orient=\"auto\" markerUnits=\"userSpaceOnUse\">\
                    <path d=\"M {} 0 L {} {} L {} {} L 0 {} Z\" fill=\"{}\" stroke=\"none\"/></marker>\n",
                    id, arrow_w, arrow_h, ref_x, half_h, arrow_w, arrow_h,
                    half_w, arrow_w, half_h, half_w, arrow_h, half_h, color,
                )
            }
            super::ArrowStyle::Circle => {
                let half_w = arrow_w / 2.0;
                let rx = half_w * 0.8;
                let ry = half_h * 0.8;
                let ref_x = if is_start { arrow_w } else { 0.0 };
                format!(
                    "<marker id=\"{}\" viewBox=\"0 0 {} {}\" refX=\"{}\" refY=\"{}\" markerWidth=\"{}\" markerHeight=\"{}\" orient=\"auto\" markerUnits=\"userSpaceOnUse\">\
                    <ellipse cx=\"{}\" cy=\"{}\" rx=\"{}\" ry=\"{}\" fill=\"{}\" stroke=\"none\"/></marker>\n",
                    id, arrow_w, arrow_h, ref_x, half_h, arrow_w, arrow_h,
                    half_w, half_h, rx, ry, color,
                )
            }
            super::ArrowStyle::Square => {
                let ref_x = if is_start { arrow_w } else { 0.0 };
                format!(
                    "<marker id=\"{}\" viewBox=\"0 0 {} {}\" refX=\"{}\" refY=\"{}\" markerWidth=\"{}\" markerHeight=\"{}\" orient=\"auto\" markerUnits=\"userSpaceOnUse\">\
                    <rect x=\"0\" y=\"0\" width=\"{}\" height=\"{}\" fill=\"{}\" stroke=\"none\"/></marker>\n",
                    id, arrow_w, arrow_h, ref_x, half_h, arrow_w, arrow_h,
                    arrow_w, arrow_h, color,
                )
            }
            super::ArrowStyle::None => return id,
        };

        self.defs.push(def);
        id
    }

    /// 화살표 크기(arrow_w, arrow_h) 계산
    /// ensure_arrow_marker와 동일한 로직으로 화살표 길이를 반환
    fn calc_arrow_dims(stroke_width: f64, line_len: f64, arrow_size: u8) -> (f64, f64) {
        let width_level = arrow_size / 3;
        let length_level = arrow_size % 3;
        let width_mult = match width_level {
            0 => 1.5,
            1 => 2.5,
            _ => 3.5,
        };
        let length_mult = match length_level {
            0 => 1.0,
            1 => 1.5,
            _ => 2.0,
        };
        let arrow_h = (stroke_width * width_mult).max(3.0);
        let arrow_w = (arrow_h * length_mult).min(line_len * 0.3);
        (arrow_w, arrow_h)
    }

    /// 그라데이션 색상 stop 목록 생성
    fn build_gradient_stops(grad: &GradientFillInfo) -> String {
        let mut stops = String::new();
        for (i, &color) in grad.colors.iter().enumerate() {
            let offset = if i < grad.positions.len() {
                grad.positions[i] * 100.0
            } else {
                let n = grad.colors.len();
                if n <= 1 {
                    0.0
                } else {
                    i as f64 / (n - 1) as f64 * 100.0
                }
            };
            stops.push_str(&format!(
                "<stop offset=\"{:.1}%\" stop-color=\"{}\"/>\n",
                offset,
                color_to_svg(color),
            ));
        }
        stops
    }

    /// 그라데이션을 포함한 사각형 그리기 (렌더 트리 전용)
    fn draw_rect_with_gradient(
        &mut self,
        x: f64,
        y: f64,
        w: f64,
        h: f64,
        corner_radius: f64,
        style: &ShapeStyle,
        gradient: Option<&GradientFillInfo>,
    ) {
        let mut attrs = format!("x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\"", x, y, w, h);

        if corner_radius > 0.0 {
            attrs.push_str(&format!(
                " rx=\"{}\" ry=\"{}\"",
                corner_radius, corner_radius
            ));
        }

        attrs.push_str(&self.build_fill_attr(style, gradient));

        if let Some(stroke) = style.stroke_color {
            attrs.push_str(&format!(
                " stroke=\"{}\" stroke-width=\"{}\"",
                color_to_svg(stroke),
                style.stroke_width
            ));
            match style.stroke_dash {
                StrokeDash::Dash => attrs.push_str(" stroke-dasharray=\"6 3\""),
                StrokeDash::Dot => attrs.push_str(" stroke-dasharray=\"2 2\""),
                StrokeDash::DashDot => attrs.push_str(" stroke-dasharray=\"6 3 2 3\""),
                StrokeDash::DashDotDot => attrs.push_str(" stroke-dasharray=\"6 3 2 3 2 3\""),
                _ => {}
            }
        }

        if style.opacity < 1.0 {
            attrs.push_str(&format!(" opacity=\"{:.3}\"", style.opacity));
        }

        self.output.push_str(&format!("<rect {}/>\n", attrs));
    }

    /// 그라데이션을 포함한 타원 그리기 (렌더 트리 전용)
    fn draw_ellipse_with_gradient(
        &mut self,
        cx: f64,
        cy: f64,
        rx: f64,
        ry: f64,
        style: &ShapeStyle,
        gradient: Option<&GradientFillInfo>,
    ) {
        let mut attrs = format!("cx=\"{}\" cy=\"{}\" rx=\"{}\" ry=\"{}\"", cx, cy, rx, ry);

        attrs.push_str(&self.build_fill_attr(style, gradient));

        if let Some(stroke) = style.stroke_color {
            attrs.push_str(&format!(
                " stroke=\"{}\" stroke-width=\"{}\"",
                color_to_svg(stroke),
                style.stroke_width
            ));
        }

        if style.opacity < 1.0 {
            attrs.push_str(&format!(" opacity=\"{:.3}\"", style.opacity));
        }

        self.output.push_str(&format!("<ellipse {}/>\n", attrs));
    }

    /// 그라데이션을 포함한 패스 그리기 (렌더 트리 전용)
    fn draw_path_with_gradient(
        &mut self,
        commands: &[PathCommand],
        style: &ShapeStyle,
        gradient: Option<&GradientFillInfo>,
    ) {
        let mut d = String::new();
        for cmd in commands {
            match cmd {
                PathCommand::MoveTo(x, y) => d.push_str(&format!("M{} {} ", x, y)),
                PathCommand::LineTo(x, y) => d.push_str(&format!("L{} {} ", x, y)),
                PathCommand::CurveTo(x1, y1, x2, y2, x, y) => {
                    d.push_str(&format!("C{} {} {} {} {} {} ", x1, y1, x2, y2, x, y))
                }
                PathCommand::ArcTo(rx, ry, x_rot, large_arc, sweep, x, y) => {
                    d.push_str(&format!(
                        "A{} {} {} {} {} {} {} ",
                        rx,
                        ry,
                        x_rot,
                        if *large_arc { 1 } else { 0 },
                        if *sweep { 1 } else { 0 },
                        x,
                        y
                    ));
                }
                PathCommand::ClosePath => d.push_str("Z "),
            }
        }

        let mut attrs = format!("d=\"{}\"", d.trim());

        attrs.push_str(&self.build_fill_attr(style, gradient));

        if let Some(stroke) = style.stroke_color {
            attrs.push_str(&format!(
                " stroke=\"{}\" stroke-width=\"{}\"",
                color_to_svg(stroke),
                style.stroke_width
            ));
            match style.stroke_dash {
                StrokeDash::Dash => attrs.push_str(" stroke-dasharray=\"6 3\""),
                StrokeDash::Dot => attrs.push_str(" stroke-dasharray=\"2 2\""),
                StrokeDash::DashDot => attrs.push_str(" stroke-dasharray=\"6 3 2 3\""),
                StrokeDash::DashDotDot => attrs.push_str(" stroke-dasharray=\"6 3 2 3 2 3\""),
                _ => {}
            }
        }

        self.output.push_str(&format!("<path {}/>\n", attrs));
    }

    /// HWP 각도(도) → SVG linearGradient 좌표 (x1%, y1%, x2%, y2%) 변환
    fn angle_to_svg_coords(angle: i16) -> (f64, f64, f64, f64) {
        let a = ((angle % 360 + 360) % 360) as f64;
        match a as i32 {
            0 => (0.0, 0.0, 0.0, 100.0),
            45 => (0.0, 0.0, 100.0, 100.0),
            90 => (0.0, 0.0, 100.0, 0.0),
            135 => (0.0, 100.0, 100.0, 0.0),
            180 => (0.0, 100.0, 0.0, 0.0),
            225 => (100.0, 100.0, 0.0, 0.0),
            270 => (100.0, 0.0, 0.0, 0.0),
            315 => (100.0, 0.0, 0.0, 100.0),
            _ => {
                let rad = a.to_radians();
                let sin = rad.sin();
                let cos = rad.cos();
                let x1 = 50.0 - sin * 50.0;
                let y1 = 50.0 - cos * 50.0;
                let x2 = 50.0 + sin * 50.0;
                let y2 = 50.0 + cos * 50.0;
                (x1, y1, x2, y2)
            }
        }
    }

    /// 이중선/삼중선 렌더링: 원래 선에 수직 방향으로 평행선들을 그림
    fn draw_multi_line(
        &mut self,
        x1: f64,
        y1: f64,
        x2: f64,
        y2: f64,
        total_width: f64,
        color: &str,
        line_type: &super::LineRenderType,
    ) {
        let dx = x2 - x1;
        let dy = y2 - y1;
        let len = (dx * dx + dy * dy).sqrt();
        if len < 0.001 {
            return;
        }

        // 수직 방향 단위벡터 (선의 법선)
        let nx = -dy / len;
        let ny = dx / len;

        // (width_ratio, offset_ratio) — offset은 선 중심으로부터의 거리 비율
        let lines: Vec<(f64, f64)> = match line_type {
            super::LineRenderType::Double => {
                // 같은 굵기 이중선: 각 선 30%, 간격 40%
                vec![(0.30, -0.35), (0.30, 0.35)]
            }
            super::LineRenderType::ThickThinDouble => {
                // 굵은선(위)-얇은선(아래): 굵은선 40%, 얇은선 20%, 간격 40%
                vec![(0.4, -0.30), (0.2, 0.40)]
            }
            super::LineRenderType::ThinThickDouble => {
                // 얇은선(위)-굵은선(아래): 얇은선 20%, 굵은선 40%, 간격 40%
                vec![(0.2, -0.40), (0.4, 0.30)]
            }
            super::LineRenderType::ThinThickThinTriple => {
                // 얇은-굵은-얇은 삼중선: 15%, 30%, 15%, 간격 20%×2
                vec![(0.15, -0.425), (0.30, 0.0), (0.15, 0.425)]
            }
            _ => return,
        };

        for (width_ratio, offset_ratio) in &lines {
            let w = total_width * width_ratio;
            let off = total_width * offset_ratio;
            let ox = nx * off;
            let oy = ny * off;
            self.output.push_str(&format!(
                "<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"{}\" stroke-width=\"{}\"/>\n",
                x1 + ox, y1 + oy, x2 + ox, y2 + oy, color, w,
            ));
        }
    }

    /// PageBackground/BorderFill 이미지를 fill_mode에 따라 렌더링한다.
    fn render_page_background_image(&mut self, img: &PageBackgroundImage, bbox: &BoundingBox) {
        // PageBackground RealPic 워터마크 프리셋은 한컴의 색상 있는 배경 워터마크에 맞춰
        // 색감 보정을 PNG 픽셀에 bake한 뒤 반투명으로 합성한다.
        let preserve_color_watermark = img.is_real_picture_watermark_tone_preset();
        // [Issue #1156] 워터마크 판정 = 밝기·대비가 둘 다 0 이 아님 (effect 무관).
        // 한컴은 워터마크 효과 해제 시 밝기·대비를 0/0 으로 되돌린다. 종전의
        // `!RealPic && ...` 조건은 effect=RealPic 배경 워터마크(143E: 70/-50)를
        // 놓쳐 opacity 가 빠지는 회귀를 냈다.
        let is_watermark_image = img.is_watermark();
        let detected_mime = detect_image_mime_type(&img.data);
        // BMP/PCX → PNG 재인코딩 (브라우저 호환성과 PCX white transparency 정합)
        let (render_bytes, render_mime): (std::borrow::Cow<[u8]>, &str) =
            if preserve_color_watermark {
                match real_picture_watermark_bytes_to_hancom_tone_png_bytes(&img.data) {
                    Some(png) => (std::borrow::Cow::Owned(png), "image/png"),
                    None => (
                        std::borrow::Cow::Borrowed(img.data.as_slice()),
                        detected_mime,
                    ),
                }
            } else if detected_mime == "image/bmp" {
                match bmp_bytes_to_png_bytes(&img.data) {
                    Some(png) => (std::borrow::Cow::Owned(png), "image/png"),
                    None => (
                        std::borrow::Cow::Borrowed(img.data.as_slice()),
                        detected_mime,
                    ),
                }
            } else if detected_mime == "image/x-pcx" {
                match pcx_bytes_to_png_bytes(&img.data) {
                    Some(png) => (std::borrow::Cow::Owned(png), "image/png"),
                    None => (
                        std::borrow::Cow::Borrowed(img.data.as_slice()),
                        detected_mime,
                    ),
                }
            } else {
                (
                    std::borrow::Cow::Borrowed(img.data.as_slice()),
                    detected_mime,
                )
            };
        let base64_data = base64::engine::general_purpose::STANDARD.encode(&*render_bytes);
        let data_uri = format!("data:{};base64,{}", render_mime, base64_data);

        let effect_filter_id = if preserve_color_watermark {
            None
        } else {
            self.ensure_image_effect_filter(img.effect)
        };
        if let Some(ref fid) = effect_filter_id {
            self.output
                .push_str(&format!("<g filter=\"url(#{})\">\n", fid));
        }
        let bc_filter_id = if preserve_color_watermark {
            None
        } else {
            self.ensure_brightness_contrast_filter(img.brightness, img.contrast)
        };
        if let Some(ref fid) = bc_filter_id {
            self.output
                .push_str(&format!("<g filter=\"url(#{})\">\n", fid));
        }
        let needs_watermark_opacity = preserve_color_watermark || is_watermark_image;
        if needs_watermark_opacity {
            let opacity = if preserve_color_watermark {
                REAL_PICTURE_WATERMARK_PAGE_OPACITY
            } else {
                LEGACY_IMAGE_WATERMARK_OPACITY
            };
            self.output
                .push_str(&format!("<g opacity=\"{}\">\n", opacity));
        }

        match img.fill_mode {
            ImageFillMode::FitToSize | ImageFillMode::None => {
                self.output.push_str(&format!(
                    "<image x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" preserveAspectRatio=\"none\" href=\"{}\"/>\n",
                    bbox.x, bbox.y, bbox.width, bbox.height, data_uri,
                ));
            }
            ImageFillMode::TileAll => {
                self.render_tiled_image(&render_bytes, &data_uri, bbox, true, true, None);
            }
            ImageFillMode::TileHorzTop | ImageFillMode::TileHorzBottom => {
                self.render_tiled_image(&render_bytes, &data_uri, bbox, true, false, None);
            }
            ImageFillMode::TileVertLeft | ImageFillMode::TileVertRight => {
                self.render_tiled_image(&render_bytes, &data_uri, bbox, false, true, None);
            }
            _ => {
                self.render_positioned_image(&render_bytes, &data_uri, bbox, img.fill_mode, None);
            }
        }

        if needs_watermark_opacity {
            self.output.push_str("</g>\n");
        }
        if bc_filter_id.is_some() {
            self.output.push_str("</g>\n");
        }
        if effect_filter_id.is_some() {
            self.output.push_str("</g>\n");
        }
    }

    /// 이미지 노드를 fill_mode에 따라 렌더링한다.
    fn render_image_node(&mut self, img: &ImageNode, bbox: &super::render_tree::BoundingBox) {
        // [Task #741] 빈 binary 데이터 (외부 file path 그림 등) 도 placeholder 처리.
        // 한컴 한글 2024 viewer 정합 — 외부 file 못 찾는 경우 점선 사각형 + 깨진 image 아이콘.
        let data = match img.data {
            Some(ref d) if !d.is_empty() => d,
            _ => {
                // 이미지 데이터 부재 (None 또는 빈 vec) — placeholder 표시
                self.output.push_str(&format!(
                    "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"#f0f0f0\" stroke=\"#999999\" stroke-dasharray=\"4\"/>\n",
                    bbox.x, bbox.y, bbox.width, bbox.height,
                ));
                // 외부 file path 그림: file path 표시 (가독성)
                if let Some(ref path) = img.external_path {
                    let cx = bbox.x + bbox.width / 2.0;
                    let cy = bbox.y + bbox.height / 2.0;
                    let escaped = path
                        .replace('&', "&amp;")
                        .replace('<', "&lt;")
                        .replace('>', "&gt;");
                    self.output.push_str(&format!(
                        "<text x=\"{}\" y=\"{}\" text-anchor=\"middle\" fill=\"#666666\" font-size=\"10\">[외부: {}]</text>\n",
                        cx, cy, escaped,
                    ));
                }
                return;
            }
        };

        // RealPic 워터마크 프리셋은 한컴의 색상 있는 배경 워터마크에 맞춰
        // 색감을 살린 뒤 반투명으로 합성한다. 표/셀 배경 fill은 쪽 배경보다
        // 더 투명하게 합성되는 샘플이 있어 opacity만 별도 프로파일을 사용한다.
        let preserve_color_watermark = img.is_real_picture_watermark_tone_preset();
        // [Issue #1156] 워터마크 판정 = 밝기·대비가 둘 다 0 이 아님 (effect 무관).
        let is_watermark_image = img.is_watermark();
        let mime_type = detect_image_mime_type(data);

        // WMF → SVG 변환 (브라우저는 WMF를 렌더링할 수 없으므로 SVG로 변환)
        // BMP → PNG 변환 (브라우저는 SVG <image> 내부의 data:image/bmp 미지원)
        // PCX → PNG 변환 (브라우저는 PCX 포맷을 native 렌더링하지 못함, Task #514)
        let (render_data, render_mime, baked_watermark): (std::borrow::Cow<[u8]>, &str, bool) =
            if preserve_color_watermark {
                match real_picture_watermark_fill_bytes_to_hancom_tone_png_bytes(data) {
                    Some(png_bytes) => (std::borrow::Cow::Owned(png_bytes), "image/png", true),
                    None => (std::borrow::Cow::Borrowed(data), mime_type, false),
                }
            } else if mime_type == "image/x-wmf" {
                match convert_wmf_to_svg(data) {
                    Some(svg_bytes) => (std::borrow::Cow::Owned(svg_bytes), "image/svg+xml", false),
                    None => (std::borrow::Cow::Borrowed(data), mime_type, false),
                }
            } else if mime_type == "image/bmp" {
                match bmp_bytes_to_png_bytes(data) {
                    Some(png_bytes) => (std::borrow::Cow::Owned(png_bytes), "image/png", false),
                    None => (std::borrow::Cow::Borrowed(data), mime_type, false),
                }
            } else if mime_type == "image/x-pcx" {
                match pcx_bytes_to_png_bytes(data) {
                    Some(png_bytes) => (std::borrow::Cow::Owned(png_bytes), "image/png", false),
                    None => (std::borrow::Cow::Borrowed(data), mime_type, false),
                }
            } else if is_watermark_image && mime_type == "image/jpeg" {
                match watermark_jpeg_bytes_to_hancom_baked_png_bytes(data) {
                    Some(png_bytes) => (std::borrow::Cow::Owned(png_bytes), "image/png", true),
                    None => (std::borrow::Cow::Borrowed(data), mime_type, false),
                }
            } else {
                (std::borrow::Cow::Borrowed(data), mime_type, false)
            };

        // 그림 효과(그레이스케일/흑백) → SVG 필터 래핑
        let effect_filter_id = if baked_watermark || preserve_color_watermark {
            None
        } else {
            self.ensure_image_effect_filter(img.effect)
        };
        if let Some(ref fid) = effect_filter_id {
            self.output
                .push_str(&format!("<g filter=\"url(#{})\">\n", fid));
        }
        let object_opacity = img.opacity.clamp(0.0, 1.0);
        if object_opacity < 1.0 {
            self.output
                .push_str(&format!("<g opacity=\"{:.3}\">\n", object_opacity));
        }
        // 밝기/대비 → SVG 필터 래핑
        // [Issue #677] 한컴 워터마크 효과 (effect != RealPic 이고 brightness/contrast 가
        // 비-zero) 는 저장값을 그대로 brightness/contrast 필터로 적용한다. JPEG 워터마크는
        // #976의 baked PNG 선보정이 성공하면 런타임 필터를 생략하고, RealPic 색상
        // 워터마크는 #975의 baked PNG 톤 보정으로 처리한다.
        let bc_filter_id = if baked_watermark || preserve_color_watermark {
            None
        } else {
            self.ensure_brightness_contrast_filter(img.brightness, img.contrast)
        };
        if let Some(ref fid) = bc_filter_id {
            self.output
                .push_str(&format!("<g filter=\"url(#{})\">\n", fid));
        }
        // 워터마크 반투명 영역. JPEG baked 워터마크는 이미 한컴 톤으로 픽셀화되어
        // 있으므로 추가 opacity를 적용하지 않는다.
        let needs_watermark_opacity =
            preserve_color_watermark || (is_watermark_image && !baked_watermark);
        if needs_watermark_opacity {
            let opacity = if preserve_color_watermark {
                REAL_PICTURE_WATERMARK_FILL_OPACITY
            } else {
                LEGACY_IMAGE_WATERMARK_OPACITY
            };
            self.output
                .push_str(&format!("<g opacity=\"{}\">\n", opacity));
        }

        let base64_data = base64::engine::general_purpose::STANDARD.encode(&*render_data);
        let data_uri = format!("data:{};base64,{}", render_mime, base64_data);

        let fill_mode = img.fill_mode.unwrap_or(ImageFillMode::FitToSize);

        match fill_mode {
            ImageFillMode::FitToSize => {
                // 그림 자르기: crop이 있으면 원본 이미지의 일부만 표시
                if let Some((cl, ct, cr, cb)) = img.crop {
                    if let Some((img_w, img_h)) = parse_image_dimensions(&render_data) {
                        let img_w = img_w as f64;
                        let img_h = img_h as f64;
                        let (src_x, src_y, src_w, src_h) = compute_image_crop_src(
                            (cl, ct, cr, cb),
                            img.original_size_hu,
                            img_w,
                            img_h,
                        );
                        // 전체 이미지 대비 잘림이 있는지 확인
                        let is_cropped = src_x > 0.5
                            || src_y > 0.5
                            || (src_w - img_w).abs() > 1.0
                            || (src_h - img_h).abs() > 1.0;
                        if is_cropped {
                            // SVG: 중첩 svg + viewBox로 crop 영역만 표시
                            self.output.push_str(&format!(
                                "<svg x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" viewBox=\"{} {} {} {}\" preserveAspectRatio=\"none\">\
                                <image width=\"{}\" height=\"{}\" preserveAspectRatio=\"none\" href=\"{}\"/></svg>\n",
                                bbox.x, bbox.y, bbox.width, bbox.height,
                                src_x, src_y, src_w, src_h,
                                img_w, img_h, data_uri,
                            ));
                        } else {
                            self.output.push_str(&format!(
                                "<image x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" preserveAspectRatio=\"none\" href=\"{}\"/>\n",
                                bbox.x, bbox.y, bbox.width, bbox.height, data_uri,
                            ));
                        }
                    } else {
                        // 이미지 크기 파싱 실패 → crop 무시
                        self.output.push_str(&format!(
                            "<image x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" preserveAspectRatio=\"none\" href=\"{}\"/>\n",
                            bbox.x, bbox.y, bbox.width, bbox.height, data_uri,
                        ));
                    }
                } else {
                    // crop 없음: 기존 동작
                    self.output.push_str(&format!(
                        "<image x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" preserveAspectRatio=\"none\" href=\"{}\"/>\n",
                        bbox.x, bbox.y, bbox.width, bbox.height, data_uri,
                    ));
                }
            }
            ImageFillMode::TileAll => {
                // 바둑판식으로-모두: 원래 크기로 전체 타일링
                self.render_tiled_image(
                    &render_data,
                    &data_uri,
                    bbox,
                    true,
                    true,
                    img.original_size,
                );
            }
            ImageFillMode::TileHorzTop | ImageFillMode::TileHorzBottom => {
                // 바둑판식으로-가로: 가로 방향만 타일링 (위 또는 아래 기준)
                self.render_tiled_image(
                    &render_data,
                    &data_uri,
                    bbox,
                    true,
                    false,
                    img.original_size,
                );
            }
            ImageFillMode::TileVertLeft | ImageFillMode::TileVertRight => {
                // 바둑판식으로-세로: 세로 방향만 타일링 (왼쪽 또는 오른쪽 기준)
                self.render_tiled_image(
                    &render_data,
                    &data_uri,
                    bbox,
                    false,
                    true,
                    img.original_size,
                );
            }
            _ => {
                // 배치 모드: 원래 크기대로 지정 위치에 배치
                self.render_positioned_image(
                    &render_data,
                    &data_uri,
                    bbox,
                    fill_mode,
                    img.original_size,
                );
            }
        }

        if needs_watermark_opacity {
            self.output.push_str("</g>\n");
        }
        if bc_filter_id.is_some() {
            self.output.push_str("</g>\n");
        }
        if object_opacity < 1.0 {
            self.output.push_str("</g>\n");
        }
        if effect_filter_id.is_some() {
            self.output.push_str("</g>\n");
        }
    }

    /// 그림 효과(ImageEffect)에 해당하는 SVG 필터를 defs에 보장하고 ID를 반환한다.
    /// RealPic(기본)은 필터가 필요 없으므로 None 반환.
    fn ensure_image_effect_filter(
        &mut self,
        effect: crate::model::image::ImageEffect,
    ) -> Option<String> {
        use crate::model::image::ImageEffect;
        let (id, def) = match effect {
            ImageEffect::RealPic => return None,
            ImageEffect::GrayScale => (
                "rhwp-img-grayscale",
                "<filter id=\"rhwp-img-grayscale\"><feColorMatrix type=\"matrix\" values=\"\
                    0.299 0.587 0.114 0 0 \
                    0.299 0.587 0.114 0 0 \
                    0.299 0.587 0.114 0 0 \
                    0 0 0 1 0\"/></filter>\n",
            ),
            ImageEffect::BlackWhite => (
                "rhwp-img-blackwhite",
                "<filter id=\"rhwp-img-blackwhite\">\
                    <feColorMatrix type=\"matrix\" values=\"\
                        0.299 0.587 0.114 0 0 \
                        0.299 0.587 0.114 0 0 \
                        0.299 0.587 0.114 0 0 \
                        0 0 0 1 0\"/>\
                    <feComponentTransfer>\
                        <feFuncR type=\"discrete\" tableValues=\"0 1\"/>\
                        <feFuncG type=\"discrete\" tableValues=\"0 1\"/>\
                        <feFuncB type=\"discrete\" tableValues=\"0 1\"/>\
                    </feComponentTransfer>\
                </filter>\n",
            ),
            // Pattern8x8은 SVG 필터로 표현하기 어려워 그레이스케일로 폴백
            ImageEffect::Pattern8x8 => (
                "rhwp-img-grayscale",
                "<filter id=\"rhwp-img-grayscale\"><feColorMatrix type=\"matrix\" values=\"\
                    0.299 0.587 0.114 0 0 \
                    0.299 0.587 0.114 0 0 \
                    0.299 0.587 0.114 0 0 \
                    0 0 0 1 0\"/></filter>\n",
            ),
        };
        if self.defs_ids.insert(id.to_string()) {
            self.defs.push(def.to_string());
        }
        Some(id.to_string())
    }

    /// 밝기/대비 조정용 SVG 필터를 defs에 보장하고 ID를 반환한다.
    /// 둘 다 0이면 필터 불필요 → None 반환.
    /// HWP 스펙은 brightness/contrast 를 -100..=100 으로 정의하므로 손상된 입력에 대비해 clamp 한다.
    fn ensure_brightness_contrast_filter(
        &mut self,
        brightness: i8,
        contrast: i8,
    ) -> Option<String> {
        let brightness = brightness.clamp(-100, 100);
        let contrast = contrast.clamp(-100, 100);
        if brightness == 0 && contrast == 0 {
            return None;
        }

        let id = format!("rhwp-img-bc-b{}c{}", brightness, contrast);

        // 밝기: intercept 오프셋으로 구현 (slope=1, intercept=brightness/100)
        // 대비: slope 조정으로 구현 (slope=(100+contrast)/100, intercept=0.5-0.5*slope)
        // 둘을 합성: slope=contrast_slope, intercept=contrast_intercept + brightness_offset
        let b = brightness as f64 / 100.0;
        let slope = (100.0 + contrast as f64) / 100.0;
        let intercept = (0.5 - 0.5 * slope) + b;

        let def = format!(
            "<filter id=\"{id}\">\
                <feComponentTransfer>\
                    <feFuncR type=\"linear\" slope=\"{slope:.4}\" intercept=\"{intercept:.4}\"/>\
                    <feFuncG type=\"linear\" slope=\"{slope:.4}\" intercept=\"{intercept:.4}\"/>\
                    <feFuncB type=\"linear\" slope=\"{slope:.4}\" intercept=\"{intercept:.4}\"/>\
                </feComponentTransfer>\
            </filter>\n"
        );
        if self.defs_ids.insert(id.clone()) {
            self.defs.push(def);
        }
        Some(id)
    }

    /// 이미지를 원래 크기로 지정 위치에 배치 (배치 모드)
    fn render_positioned_image(
        &mut self,
        data: &[u8],
        data_uri: &str,
        bbox: &super::render_tree::BoundingBox,
        fill_mode: ImageFillMode,
        original_size: Option<(f64, f64)>,
    ) {
        // 원본 크기: HWP shape_attr 기반(우선) 또는 이미지 픽셀 크기(폴백)
        let (img_width, img_height) = if let Some((ow, oh)) = original_size {
            (ow, oh)
        } else {
            match parse_image_dimensions(data) {
                Some((w, h)) => (w as f64, h as f64),
                None => {
                    // 크기 파싱 실패 시 meet으로 폴백
                    let par = match fill_mode {
                        ImageFillMode::Center => "xMidYMid meet",
                        ImageFillMode::CenterTop => "xMidYMin meet",
                        ImageFillMode::CenterBottom => "xMidYMax meet",
                        ImageFillMode::LeftCenter => "xMinYMid meet",
                        ImageFillMode::LeftTop => "xMinYMin meet",
                        ImageFillMode::LeftBottom => "xMinYMax meet",
                        ImageFillMode::RightCenter => "xMaxYMid meet",
                        ImageFillMode::RightTop => "xMaxYMin meet",
                        ImageFillMode::RightBottom => "xMaxYMax meet",
                        _ => "xMidYMid meet",
                    };
                    self.output.push_str(&format!(
                        "<image x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" preserveAspectRatio=\"{}\" href=\"{}\"/>\n",
                        bbox.x, bbox.y, bbox.width, bbox.height, par, data_uri,
                    ));
                    return;
                }
            }
        };

        // 배치 위치 계산
        let (ix, iy) = match fill_mode {
            ImageFillMode::LeftTop => (bbox.x, bbox.y),
            ImageFillMode::CenterTop => (bbox.x + (bbox.width - img_width) / 2.0, bbox.y),
            ImageFillMode::RightTop => (bbox.x + bbox.width - img_width, bbox.y),
            ImageFillMode::LeftCenter => (bbox.x, bbox.y + (bbox.height - img_height) / 2.0),
            ImageFillMode::Center => (
                bbox.x + (bbox.width - img_width) / 2.0,
                bbox.y + (bbox.height - img_height) / 2.0,
            ),
            ImageFillMode::RightCenter => (
                bbox.x + bbox.width - img_width,
                bbox.y + (bbox.height - img_height) / 2.0,
            ),
            ImageFillMode::LeftBottom => (bbox.x, bbox.y + bbox.height - img_height),
            ImageFillMode::CenterBottom => (
                bbox.x + (bbox.width - img_width) / 2.0,
                bbox.y + bbox.height - img_height,
            ),
            ImageFillMode::RightBottom => (
                bbox.x + bbox.width - img_width,
                bbox.y + bbox.height - img_height,
            ),
            _ => (bbox.x, bbox.y),
        };

        // 도형 영역으로 클리핑
        let clip_id = format!("fill-clip-{}", self.next_clip_id());
        self.defs.push(format!(
            "<clipPath id=\"{}\"><rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\"/></clipPath>\n",
            clip_id, bbox.x, bbox.y, bbox.width, bbox.height,
        ));
        self.output.push_str(&format!(
            "<g clip-path=\"url(#{})\"><image x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" preserveAspectRatio=\"none\" href=\"{}\"/></g>\n",
            clip_id, ix, iy, img_width, img_height, data_uri,
        ));
    }

    /// 이미지를 타일링 모드로 렌더링
    fn render_tiled_image(
        &mut self,
        data: &[u8],
        data_uri: &str,
        bbox: &super::render_tree::BoundingBox,
        tile_h: bool,
        tile_v: bool,
        original_size: Option<(f64, f64)>,
    ) {
        // 원본 크기: HWP shape_attr 기반(우선) 또는 이미지 픽셀 크기(폴백)
        let (img_width, img_height) = if let Some((ow, oh)) = original_size {
            (ow, oh)
        } else {
            match parse_image_dimensions(data) {
                Some((w, h)) => (w as f64, h as f64),
                None => {
                    // 크기 파싱 실패 시 전체 채우기로 폴백
                    self.output.push_str(&format!(
                        "<image x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" preserveAspectRatio=\"none\" href=\"{}\"/>\n",
                        bbox.x, bbox.y, bbox.width, bbox.height, data_uri,
                    ));
                    return;
                }
            }
        };

        let pat_id = format!("tile-pat-{}", self.next_clip_id());
        let pat_w = if tile_h { img_width } else { bbox.width };
        let pat_h = if tile_v { img_height } else { bbox.height };

        self.defs.push(format!(
            "<pattern id=\"{}\" x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" patternUnits=\"userSpaceOnUse\">\
             <image width=\"{}\" height=\"{}\" preserveAspectRatio=\"none\" href=\"{}\"/>\
             </pattern>\n",
            pat_id, bbox.x, bbox.y, pat_w, pat_h,
            img_width, img_height, data_uri,
        ));
        self.output.push_str(&format!(
            "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"url(#{})\"/>\n",
            bbox.x, bbox.y, bbox.width, bbox.height, pat_id,
        ));
    }

    /// 고유 클립/패턴 ID 생성
    fn next_clip_id(&mut self) -> u32 {
        self.clip_counter += 1;
        self.clip_counter
    }

    /// 글자겹침(CharOverlap) 렌더링
    ///
    /// 각 문자를 테두리 도형(원/사각형) 안에 중앙 배치하여 렌더링한다.
    /// border_type: 0=없음, 1=원, 2=반전원, 3=사각형, 4=반전사각형
    /// 반전: 도형 채움(검정) + 흰 글자, 일반: 도형 테두리(검정) + 검정 글자
    ///
    /// 다자리 PUA 숫자 (2~3자리): 모든 문자를 하나의 원/사각형 안에 합쳐서 렌더링.
    /// border_type=0이고 PUA 겹침 숫자이면 원형(circle)으로 자동 렌더링.
    /// 한컴 방식: 장평 조절로 좁은 숫자를 하나의 도형 안에 배치.
    fn draw_char_overlap(
        &mut self,
        text: &str,
        style: &TextStyle,
        overlap: &CharOverlapInfo,
        bbox_x: f64,
        bbox_y: f64,
        bbox_w: f64,
        bbox_h: f64,
    ) {
        let font_size = if style.font_size > 0.0 {
            style.font_size
        } else {
            12.0
        };
        let chars: Vec<char> = text.chars().collect();
        if chars.is_empty() {
            return;
        }

        // PUA 다자리 숫자 디코딩 시도
        if let Some(number_str) = decode_pua_overlap_number(&chars) {
            self.draw_char_overlap_combined(
                style,
                overlap,
                &number_str,
                bbox_x,
                bbox_y,
                bbox_w,
                bbox_h,
            );
            return;
        }

        // 일반 CharOverlap 처리. 디코딩되지 않는 다중 PUA 조합도 한 컨트롤 안에서
        // 같은 중심에 겹쳐 그린다. table-vpos-01의 10/11/12 마커는
        // U+F02BA + U+F02C3/C4/C5 조합으로 저장되며, 나란히 그리면 숫자가
        // 사각형 밖으로 밀린다.
        let box_size = font_size;

        let is_reversed = overlap.border_type == 2 || overlap.border_type == 4;
        let is_circle = overlap.border_type == 1 || overlap.border_type == 2;
        let is_rect = overlap.border_type == 3 || overlap.border_type == 4;

        // inner_char_size 해석:
        //   > 0 → percent ratio (HWPX 양수 case 보존: 50 = 0.5)
        //   < 0 → 10% step 축소 (한컴 정합: charSz=-3 → 1.0 + (-3)×0.10 = 0.70, 13pt→9.1pt)
        //   == 0 → 기본 100%
        let size_ratio = if overlap.inner_char_size > 0 {
            overlap.inner_char_size as f64 / 100.0
        } else if overlap.inner_char_size < 0 {
            1.0 + overlap.inner_char_size as f64 * 0.10
        } else {
            1.0
        };
        let inner_font_size = font_size * size_ratio;

        // 한컴은 동그라미 테두리도 글자색과 동일 색상으로 그림 (raw PDF 0 0 1 RG/rg).
        // reversed(반전)는 기존대로 검정 채움 + 흰 글자.
        let glyph_color = color_to_svg(style.color);
        let fill_color = if is_reversed { "#000000" } else { "none" };
        let stroke_color: &str = if is_reversed { "#000000" } else { &glyph_color };
        let text_color: &str = if is_reversed { "#FFFFFF" } else { &glyph_color };

        let font_family_str = if style.font_family.is_empty() {
            "sans-serif".to_string()
        } else {
            let fb = super::generic_fallback(&style.font_family);
            format!("{},{}", style.font_family, fb)
        };
        let mut font_attrs = format!(
            "font-family=\"{}\" font-size=\"{:.2}\"",
            escape_xml(&font_family_str),
            inner_font_size
        );
        if style.is_visually_bold() {
            font_attrs.push_str(" font-weight=\"bold\"");
        } else if style.is_medium_weight() {
            font_attrs.push_str(" font-weight=\"500\"");
        }
        if style.italic {
            font_attrs.push_str(" font-style=\"italic\"");
        }

        if chars.len() > 1 {
            let cx = bbox_x + bbox_w / 2.0;
            let cy = bbox_y + bbox_h / 2.0;

            if is_circle {
                let ry = box_size / 2.0;
                let rx = ry * 0.85;
                self.output.push_str(&format!(
                    "<ellipse cx=\"{:.2}\" cy=\"{:.2}\" rx=\"{:.2}\" ry=\"{:.2}\" fill=\"{}\" stroke=\"{}\" stroke-width=\"0.8\"/>\n",
                    cx, cy, rx, ry, fill_color, stroke_color,
                ));
            } else if is_rect {
                let rx = cx - box_size / 2.0;
                let ry = cy - box_size / 2.0;
                self.output.push_str(&format!(
                    "<rect x=\"{:.2}\" y=\"{:.2}\" width=\"{:.2}\" height=\"{:.2}\" fill=\"{}\" stroke=\"{}\" stroke-width=\"0.8\"/>\n",
                    rx, ry, box_size, box_size, fill_color, stroke_color,
                ));
            }

            for ch in chars.iter() {
                let display_str = {
                    let cp = *ch as u32;
                    if (0x2460..=0x2473).contains(&cp) {
                        format!("{}", cp - 0x2460 + 1)
                    } else if let Some(s) = pua_to_display_text(*ch) {
                        s
                    } else {
                        ch.to_string()
                    }
                };
                self.output.push_str(&format!(
                    "<text x=\"{:.2}\" y=\"{:.2}\" fill=\"{}\" {} text-anchor=\"middle\" dominant-baseline=\"central\">{}</text>\n",
                    cx, cy, text_color, font_attrs, escape_xml(&display_str),
                ));
            }
            return;
        }

        for (i, ch) in chars.iter().enumerate() {
            let display_str = {
                let cp = *ch as u32;
                if (0x2460..=0x2473).contains(&cp) {
                    format!("{}", cp - 0x2460 + 1)
                } else if let Some(s) = pua_to_display_text(*ch) {
                    s
                } else {
                    ch.to_string()
                }
            };

            let cx = bbox_x + i as f64 * box_size + box_size / 2.0;
            let cy = bbox_y + bbox_h / 2.0;

            if is_circle {
                // 한컴 글자겹침은 세로로 긴 타원 (h/w ≈ 1.18). 한글 글리프 비율과 정합.
                let ry = box_size / 2.0;
                let rx = ry * 0.85;
                self.output.push_str(&format!(
                    "<ellipse cx=\"{:.2}\" cy=\"{:.2}\" rx=\"{:.2}\" ry=\"{:.2}\" fill=\"{}\" stroke=\"{}\" stroke-width=\"0.8\"/>\n",
                    cx, cy, rx, ry, fill_color, stroke_color,
                ));
            } else if is_rect {
                let rx = cx - box_size / 2.0;
                let ry = cy - box_size / 2.0;
                self.output.push_str(&format!(
                    "<rect x=\"{:.2}\" y=\"{:.2}\" width=\"{:.2}\" height=\"{:.2}\" fill=\"{}\" stroke=\"{}\" stroke-width=\"0.8\"/>\n",
                    rx, ry, box_size, box_size, fill_color, stroke_color,
                ));
            }

            self.output.push_str(&format!(
                "<text x=\"{:.2}\" y=\"{:.2}\" fill=\"{}\" {} text-anchor=\"middle\" dominant-baseline=\"central\">{}</text>\n",
                cx, cy, text_color, font_attrs, escape_xml(&display_str),
            ));
        }
    }

    /// PUA 다자리 숫자를 하나의 도형 안에 합쳐서 렌더링
    ///
    /// border_type=0이면 원형으로 자동 렌더링 (PUA 겹침 숫자는 원래 원문자)
    /// 장평 조절: textLength 속성으로 숫자 문자열을 도형 내부 폭에 맞춤
    fn draw_char_overlap_combined(
        &mut self,
        style: &TextStyle,
        overlap: &CharOverlapInfo,
        number_str: &str,
        bbox_x: f64,
        bbox_y: f64,
        bbox_w: f64,
        bbox_h: f64,
    ) {
        let font_size = if style.font_size > 0.0 {
            style.font_size
        } else {
            12.0
        };
        let box_size = font_size;

        // border_type=0이고 PUA 숫자이면 원형으로 자동 렌더링
        let effective_border = if overlap.border_type == 0 {
            1u8
        } else {
            overlap.border_type
        };
        let is_reversed = effective_border == 2 || effective_border == 4;
        let is_circle = effective_border == 1 || effective_border == 2;
        let is_rect = effective_border == 3 || effective_border == 4;

        // inner_char_size 해석 (draw_char_overlap와 동일 — 음수=10% step 축소)
        let size_ratio = if overlap.inner_char_size > 0 {
            overlap.inner_char_size as f64 / 100.0
        } else if overlap.inner_char_size < 0 {
            1.0 + overlap.inner_char_size as f64 * 0.10
        } else {
            1.0
        };
        let inner_font_size = font_size * size_ratio;

        let glyph_color = color_to_svg(style.color);
        let fill_color = if is_reversed { "#000000" } else { "none" };
        let stroke_color: &str = if is_reversed { "#000000" } else { &glyph_color };
        let text_color: &str = if is_reversed { "#FFFFFF" } else { &glyph_color };

        let font_family_str = if style.font_family.is_empty() {
            "sans-serif".to_string()
        } else {
            let fb = super::generic_fallback(&style.font_family);
            format!("{},{}", style.font_family, fb)
        };
        let mut font_attrs = format!(
            "font-family=\"{}\" font-size=\"{:.2}\"",
            escape_xml(&font_family_str),
            inner_font_size
        );
        if style.is_visually_bold() {
            font_attrs.push_str(" font-weight=\"bold\"");
        } else if style.is_medium_weight() {
            font_attrs.push_str(" font-weight=\"500\"");
        }
        if style.italic {
            font_attrs.push_str(" font-style=\"italic\"");
        }

        let cx = bbox_x + box_size / 2.0;
        let cy = bbox_y + bbox_h / 2.0;

        // 도형 렌더링 — 세로로 긴 타원 (한컴 정합, rx=ry*0.85)
        if is_circle {
            let ry = box_size / 2.0;
            let rx = ry * 0.85;
            self.output.push_str(&format!(
                "<ellipse cx=\"{:.2}\" cy=\"{:.2}\" rx=\"{:.2}\" ry=\"{:.2}\" fill=\"{}\" stroke=\"{}\" stroke-width=\"0.8\"/>\n",
                cx, cy, rx, ry, fill_color, stroke_color,
            ));
        } else if is_rect {
            let rx = cx - box_size / 2.0;
            let ry = cy - box_size / 2.0;
            self.output.push_str(&format!(
                "<rect x=\"{:.2}\" y=\"{:.2}\" width=\"{:.2}\" height=\"{:.2}\" fill=\"{}\" stroke=\"{}\" stroke-width=\"0.8\"/>\n",
                rx, ry, box_size, box_size, fill_color, stroke_color,
            ));
        }

        // 장평 조절: 숫자 자릿수에 따라 textLength로 폭 압축
        let text_width = box_size * 0.7; // 도형 내부 여백 고려
                                         // 다자리 숫자는 baseline을 살짝 올려 시각적 중앙 맞춤
        let text_y = cy - font_size * 0.08;
        self.output.push_str(&format!(
            "<text x=\"{:.2}\" y=\"{:.2}\" fill=\"{}\" {} text-anchor=\"middle\" dominant-baseline=\"central\" textLength=\"{:.2}\" lengthAdjust=\"spacingAndGlyphs\">{}</text>\n",
            cx, text_y, text_color, font_attrs, text_width, escape_xml(number_str),
        ));
    }

    /// 선 모양(shape)에 따라 SVG line/group을 출력한다.
    /// shape: 0=실선, 1=긴점선, 2=점선, 3=일점쇄선, 4=이점쇄선, 5=긴파선,
    ///        6=원형점, 7=이중선, 8=가는+굵은, 9=굵은+가는, 10=삼중선
    fn draw_line_shape(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, color: &str, shape: u8) {
        match shape {
            7 => {
                // 이중선
                self.output.push_str(&format!(
                    "<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"{}\" stroke-width=\"0.7\"/>\n",
                    x1, y1 - 1.0, x2, y2 - 1.0, color));
                self.output.push_str(&format!(
                    "<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"{}\" stroke-width=\"0.7\"/>\n",
                    x1, y1 + 1.0, x2, y2 + 1.0, color));
            }
            8 => {
                // 가는+굵은 이중선
                self.output.push_str(&format!(
                    "<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"{}\" stroke-width=\"0.5\"/>\n",
                    x1, y1 - 1.2, x2, y2 - 1.2, color));
                self.output.push_str(&format!(
                    "<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"{}\" stroke-width=\"1.2\"/>\n",
                    x1, y1 + 0.8, x2, y2 + 0.8, color));
            }
            9 => {
                // 굵은+가는 이중선
                self.output.push_str(&format!(
                    "<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"{}\" stroke-width=\"1.2\"/>\n",
                    x1, y1 - 0.8, x2, y2 - 0.8, color));
                self.output.push_str(&format!(
                    "<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"{}\" stroke-width=\"0.5\"/>\n",
                    x1, y1 + 1.2, x2, y2 + 1.2, color));
            }
            10 => {
                // 삼중선
                self.output.push_str(&format!(
                    "<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"{}\" stroke-width=\"0.5\"/>\n",
                    x1, y1 - 1.5, x2, y2 - 1.5, color));
                self.output.push_str(&format!(
                    "<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"{}\" stroke-width=\"0.5\"/>\n",
                    x1, y1, x2, y2, color));
                self.output.push_str(&format!(
                    "<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"{}\" stroke-width=\"0.5\"/>\n",
                    x1, y1 + 1.5, x2, y2 + 1.5, color));
            }
            11 => {
                // 물결선
                let wave_h = 1.5;
                let wave_w = 6.0;
                let mut d = format!("M{:.2},{:.2}", x1, y1);
                let mut cx = x1;
                let mut up = true;
                while cx < x2 {
                    let next = (cx + wave_w).min(x2);
                    let cy = if up { y1 - wave_h } else { y1 + wave_h };
                    d.push_str(&format!(
                        " Q{:.2},{:.2} {:.2},{:.2}",
                        (cx + next) / 2.0,
                        cy,
                        next,
                        y1
                    ));
                    cx = next;
                    up = !up;
                }
                self.output.push_str(&format!(
                    "<path d=\"{}\" fill=\"none\" stroke=\"{}\" stroke-width=\"0.7\"/>\n",
                    d, color
                ));
            }
            12 => {
                // 이중물결선
                for offset in [-1.0f64, 1.0] {
                    let wy = y1 + offset;
                    let wave_h = 1.2;
                    let wave_w = 6.0;
                    let mut d = format!("M{:.2},{:.2}", x1, wy);
                    let mut cx = x1;
                    let mut up = true;
                    while cx < x2 {
                        let next = (cx + wave_w).min(x2);
                        let cy = if up { wy - wave_h } else { wy + wave_h };
                        d.push_str(&format!(
                            " Q{:.2},{:.2} {:.2},{:.2}",
                            (cx + next) / 2.0,
                            cy,
                            next,
                            wy
                        ));
                        cx = next;
                        up = !up;
                    }
                    self.output.push_str(&format!(
                        "<path d=\"{}\" fill=\"none\" stroke=\"{}\" stroke-width=\"0.5\"/>\n",
                        d, color
                    ));
                }
            }
            _ => {
                // 단선 (dasharray로 모양 표현)
                // 0=실선, 1=파선, 2=점선, 3=일점쇄선, 4=이점쇄선, 5=긴파선, 6=원형점선
                let dasharray = match shape {
                    1 => " stroke-dasharray=\"3 3\"",
                    2 => " stroke-dasharray=\"1 2\"",
                    3 => " stroke-dasharray=\"6 2 1 2\"",
                    4 => " stroke-dasharray=\"6 2 1 2 1 2\"",
                    5 => " stroke-dasharray=\"8 4\"",
                    6 => " stroke-dasharray=\"0.1 2.5\" stroke-linecap=\"round\"",
                    _ => "", // 0=실선
                };
                self.output.push_str(&format!(
                    "<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"{}\" stroke-width=\"1\"{}/>\n",
                    x1, y1, x2, y2, color, dasharray));
            }
        }
    }

    /// 양식 개체 SVG 렌더링
    fn render_form_object(&mut self, form: &FormObjectNode, bbox: &BoundingBox) {
        let x = bbox.x;
        let y = bbox.y;
        let w = bbox.width;
        let h = bbox.height;

        match form.form_type {
            FormType::PushButton => {
                // 3D 버튼 (웹 환경 비활성 — 회색 스타일)
                self.output.push_str(&format!(
                    "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"#d0d0d0\" stroke=\"#a0a0a0\" stroke-width=\"0.5\"/>\n",
                    x, y, w, h));
                // 캡션 텍스트 (회색, 중앙)
                if !form.caption.is_empty() {
                    let caption = display_form_caption(&form.caption);
                    let font_size = (h * 0.55).min(12.0).max(7.0);
                    self.output.push_str(&format!(
                        "<text x=\"{}\" y=\"{}\" font-size=\"{:.1}\" fill=\"#808080\" text-anchor=\"middle\" dominant-baseline=\"central\" font-family=\"'맑은 고딕',sans-serif\">{}</text>\n",
                        x + w / 2.0, y + h / 2.0, font_size, escape_xml(caption.as_ref())));
                }
            }
            FormType::CheckBox => {
                // 체크박스: □/☑ + 캡션
                let box_size = (h * 0.7).min(13.0);
                let box_y = y + (h - box_size) / 2.0;
                let box_x = x + 2.0;
                self.output.push_str(&format!(
                    "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"white\" stroke=\"#606060\" stroke-width=\"0.8\"/>\n",
                    box_x, box_y, box_size, box_size));
                if form.value != 0 {
                    // 체크 마크 (✓)
                    let cx = box_x + box_size * 0.2;
                    let cy = box_y + box_size * 0.55;
                    let mx = box_x + box_size * 0.45;
                    let my = box_y + box_size * 0.8;
                    let ex = box_x + box_size * 0.85;
                    let ey = box_y + box_size * 0.2;
                    self.output.push_str(&format!(
                        "<polyline points=\"{},{} {},{} {},{}\" fill=\"none\" stroke=\"#000000\" stroke-width=\"1.5\"/>\n",
                        cx, cy, mx, my, ex, ey));
                }
                // 캡션
                if !form.caption.is_empty() {
                    let caption = display_form_caption(&form.caption);
                    let text_x = box_x + box_size + 3.0;
                    let font_size = (h * 0.55).min(12.0).max(7.0);
                    self.output.push_str(&format!(
                        "<text x=\"{}\" y=\"{}\" font-size=\"{:.1}\" fill=\"{}\" dominant-baseline=\"central\" font-family=\"'맑은 고딕',sans-serif\">{}</text>\n",
                        text_x, y + h / 2.0, font_size, form.fore_color, escape_xml(caption.as_ref())));
                }
            }
            FormType::RadioButton => {
                // 라디오: ○/◉ + 캡션
                let r = (h * 0.3).min(6.5);
                let cx = x + 2.0 + r;
                let cy = y + h / 2.0;
                self.output.push_str(&format!(
                    "<circle cx=\"{}\" cy=\"{}\" r=\"{}\" fill=\"white\" stroke=\"#606060\" stroke-width=\"0.8\"/>\n",
                    cx, cy, r));
                if form.value != 0 {
                    self.output.push_str(&format!(
                        "<circle cx=\"{}\" cy=\"{}\" r=\"{}\" fill=\"#000000\"/>\n",
                        cx,
                        cy,
                        r * 0.5
                    ));
                }
                // 캡션
                if !form.caption.is_empty() {
                    let caption = display_form_caption(&form.caption);
                    let text_x = cx + r + 3.0;
                    let font_size = (h * 0.55).min(12.0).max(7.0);
                    self.output.push_str(&format!(
                        "<text x=\"{}\" y=\"{}\" font-size=\"{:.1}\" fill=\"{}\" dominant-baseline=\"central\" font-family=\"'맑은 고딕',sans-serif\">{}</text>\n",
                        text_x, y + h / 2.0, font_size, form.fore_color, escape_xml(caption.as_ref())));
                }
            }
            FormType::ComboBox => {
                // 콤보박스: 입력 영역 + 드롭다운 버튼(▼)
                let btn_w = (h * 0.8).min(16.0);
                self.output.push_str(&format!(
                    "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"white\" stroke=\"#a0a0a0\" stroke-width=\"0.8\"/>\n",
                    x, y, w, h));
                // 드롭다운 버튼
                self.output.push_str(&format!(
                    "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"#e0e0e0\" stroke=\"#a0a0a0\" stroke-width=\"0.5\"/>\n",
                    x + w - btn_w, y, btn_w, h));
                // ▼ 화살표
                let arrow_cx = x + w - btn_w / 2.0;
                let arrow_cy = y + h / 2.0;
                let arrow_size = (h * 0.2).min(4.0);
                self.output.push_str(&format!(
                    "<polygon points=\"{},{} {},{} {},{}\" fill=\"#404040\"/>\n",
                    arrow_cx - arrow_size,
                    arrow_cy - arrow_size * 0.5,
                    arrow_cx + arrow_size,
                    arrow_cy - arrow_size * 0.5,
                    arrow_cx,
                    arrow_cy + arrow_size * 0.5
                ));
                // 텍스트
                if !form.text.is_empty() {
                    let font_size = (h * 0.55).min(12.0).max(7.0);
                    self.output.push_str(&format!(
                        "<text x=\"{}\" y=\"{}\" font-size=\"{:.1}\" fill=\"{}\" dominant-baseline=\"central\" font-family=\"'맑은 고딕',sans-serif\">{}</text>\n",
                        x + 3.0, y + h / 2.0, font_size, form.fore_color, escape_xml(&form.text)));
                }
            }
            FormType::Edit => {
                // 입력 상자: 테두리 사각형 + 내부 텍스트
                self.output.push_str(&format!(
                    "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"white\" stroke=\"#a0a0a0\" stroke-width=\"0.8\"/>\n",
                    x, y, w, h));
                if !form.text.is_empty() {
                    let font_size = (h * 0.55).min(12.0).max(7.0);
                    self.output.push_str(&format!(
                        "<text x=\"{}\" y=\"{}\" font-size=\"{:.1}\" fill=\"{}\" dominant-baseline=\"central\" font-family=\"'맑은 고딕',sans-serif\">{}</text>\n",
                        x + 3.0, y + h / 2.0, font_size, form.fore_color, escape_xml(&form.text)));
                }
            }
        }
    }

    fn place_debug_label(
        occupied: &mut Vec<(f64, f64, f64, f64)>,
        x: f64,
        preferred_y: f64,
        width: f64,
        height: f64,
        page_height: f64,
    ) -> f64 {
        fn overlaps(a: (f64, f64, f64, f64), b: (f64, f64, f64, f64)) -> bool {
            let pad = 1.0;
            let (ax, ay, aw, ah) = a;
            let (bx, by, bw, bh) = b;
            ax < bx + bw + pad && ax + aw + pad > bx && ay < by + bh + pad && ay + ah + pad > by
        }

        let min_y = 0.0;
        let max_y = (page_height - height).max(0.0);
        let preferred_y = preferred_y.clamp(min_y, max_y);
        let step = height + 2.0;

        for i in 0..64 {
            let distance = step * i as f64;
            for offset in if i == 0 {
                [0.0, f64::NAN]
            } else {
                [-distance, distance]
            } {
                if offset.is_nan() {
                    continue;
                }
                let candidate_y = preferred_y + offset;
                if candidate_y < min_y || candidate_y > max_y {
                    continue;
                }
                let candidate = (x, candidate_y, width, height);
                if !occupied
                    .iter()
                    .any(|&(ox, oy, ow, oh)| overlaps(candidate, (ox, oy, ow, oh)))
                {
                    occupied.push(candidate);
                    return candidate_y;
                }
            }
        }

        occupied.push((x, preferred_y, width, height));
        preferred_y
    }

    /// 디버그 오버레이: 문단/표 경계와 인덱스 라벨을 렌더링
    fn render_debug_overlay(&mut self) {
        self.output
            .push_str("<g id=\"debug-overlay\" opacity=\"0.7\">\n");

        // 색상 팔레트: 문단별 교대 색상
        let colors = [
            "#FF6B6B", "#4ECDC4", "#45B7D1", "#96CEB4", "#FFEAA7", "#DDA0DD", "#98D8C8", "#F7DC6F",
        ];
        let mut occupied_labels = Vec::new();

        // 문단 경계 렌더링
        let mut sorted_paras: Vec<_> = self.overlay_para_bounds.iter().collect();
        sorted_paras.sort_by_key(|&(pi, _)| *pi);

        for (key, bounds) in &sorted_paras {
            let pi = **key % 100000;
            let si = bounds.section_index;
            let color = colors[pi % colors.len()];
            let label = format!("s{}:pi={} y={:.1}", si, pi, bounds.y);
            // 경계 사각형 (점선)
            self.output.push_str(&format!(
                "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"none\" stroke=\"{}\" stroke-width=\"0.5\" stroke-dasharray=\"3,2\"/>\n",
                bounds.x, bounds.y, bounds.width, bounds.height, color,
            ));
            // 라벨 (좌측 상단)
            let label_w = label.len() as f64 * 5.0 + 4.0;
            let label_h = 10.0;
            let label_y = Self::place_debug_label(
                &mut occupied_labels,
                bounds.x,
                bounds.y - label_h,
                label_w,
                label_h,
                self.height,
            );
            self.output.push_str(&format!(
                "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"10\" fill=\"{}\" rx=\"2\"/>\n",
                bounds.x, label_y, label_w, color,
            ));
            self.output.push_str(&format!(
                "<text x=\"{}\" y=\"{}\" font-family=\"monospace\" font-size=\"8\" fill=\"#fff\" font-weight=\"bold\">{}</text>\n",
                bounds.x + 2.0,
                label_y + label_h - 2.0,
                label,
            ));
        }

        // 표 경계 렌더링
        let table_bounds = std::mem::take(&mut self.overlay_table_bounds);
        for tbl in &table_bounds {
            // 표 경계 (빨간 점선)
            self.output.push_str(&format!(
                "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"none\" stroke=\"#E74C3C\" stroke-width=\"1.0\" stroke-dasharray=\"5,3\"/>\n",
                tbl.x, tbl.y, tbl.width, tbl.height,
            ));
            // 표 라벨 (우측 상단)
            let label = format!(
                "s{}:pi={} ci={} {}x{} y={:.1}",
                tbl.section_index,
                tbl.para_index,
                tbl.control_index,
                tbl.row_count,
                tbl.col_count,
                tbl.y
            );
            let label_w = label.len() as f64 * 5.0 + 4.0;
            let label_x = (tbl.x + tbl.width - label_w).max(tbl.x);
            let label_h = 11.0;
            let label_y = Self::place_debug_label(
                &mut occupied_labels,
                label_x,
                tbl.y - label_h,
                label_w,
                label_h,
                self.height,
            );
            self.output.push_str(&format!(
                "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"11\" fill=\"#E74C3C\" rx=\"2\"/>\n",
                label_x, label_y, label_w,
            ));
            self.output.push_str(&format!(
                "<text x=\"{}\" y=\"{}\" font-family=\"monospace\" font-size=\"8\" fill=\"#fff\" font-weight=\"bold\">{}</text>\n",
                label_x + 2.0,
                label_y + label_h - 2.0,
                label,
            ));
        }
        self.overlay_table_bounds = table_bounds;

        // 이미지 경계 렌더링
        let image_bounds = std::mem::take(&mut self.overlay_image_bounds);
        for img in &image_bounds {
            self.output.push_str(&format!(
                "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"none\" stroke=\"#7B61FF\" stroke-width=\"1.2\" stroke-dasharray=\"4,2\"/>\n",
                img.x, img.y, img.width, img.height,
            ));
            let label = format!(
                "s{}:pi={} ci={} image y={:.1}",
                img.section_index, img.para_index, img.control_index, img.y
            );
            let label_w = label.len() as f64 * 5.0 + 4.0;
            let label_h = 11.0;
            let label_y = Self::place_debug_label(
                &mut occupied_labels,
                img.x,
                img.y - label_h,
                label_w,
                label_h,
                self.height,
            );
            self.output.push_str(&format!(
                "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"11\" fill=\"#7B61FF\" rx=\"2\"/>\n",
                img.x, label_y, label_w,
            ));
            self.output.push_str(&format!(
                "<text x=\"{}\" y=\"{}\" font-family=\"monospace\" font-size=\"8\" fill=\"#fff\" font-weight=\"bold\">{}</text>\n",
                img.x + 2.0,
                label_y + label_h - 2.0,
                label,
            ));
        }
        self.overlay_image_bounds = image_bounds;

        // vpos=0 리셋 위치 마커 (앰버 가로 점선 + 라벨)
        let vpos_resets = std::mem::take(&mut self.overlay_vpos_resets);
        for rs in &vpos_resets {
            // 노란 가로선 (점선)
            self.output.push_str(&format!(
                "<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"#FFB300\" stroke-width=\"1.5\" stroke-dasharray=\"6,3\"/>\n",
                rs.x, rs.y, rs.x + rs.width, rs.y,
            ));
            // 라벨 (좌측, 가로선 위)
            let label = format!(
                "vpos-reset s{}:pi={}:line={}",
                rs.section_index, rs.para_index, rs.line_index
            );
            let label_w = label.len() as f64 * 5.0 + 4.0;
            let label_h = 11.0;
            let label_y = Self::place_debug_label(
                &mut occupied_labels,
                rs.x,
                rs.y - label_h,
                label_w,
                label_h,
                self.height,
            );
            self.output.push_str(&format!(
                "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"11\" fill=\"#FFB300\" rx=\"2\"/>\n",
                rs.x, label_y, label_w,
            ));
            self.output.push_str(&format!(
                "<text x=\"{}\" y=\"{}\" font-family=\"monospace\" font-size=\"8\" fill=\"#000\" font-weight=\"bold\">{}</text>\n",
                rs.x + 2.0,
                label_y + label_h - 2.0,
                label,
            ));
        }
        self.overlay_vpos_resets = vpos_resets;

        self.output.push_str("</g>\n");
    }
}

impl Renderer for SvgRenderer {
    fn begin_page(&mut self, width: f64, height: f64) {
        self.width = width;
        self.height = height;
        self.output.clear();
        self.defs.clear();
        self.defs_ids.clear();
        self.gradient_counter = 0;
        self.overlay_para_bounds.clear();
        self.overlay_table_bounds.clear();
        self.overlay_image_bounds.clear();
        self.overlay_vpos_resets.clear();
        self.overlay_skip_depth = 0;
        self.overlay_page_section = -1;
        // xmlns:xlink 필수: SVG 가 <img> 로 로드될 때(예: blob URL 미리보기)
        // 엄격한 XML 파싱으로 인해 xmlns:xlink 미선언 시 <image xlink:href=...> 가 무시됨.
        self.output.push_str(&format!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" xmlns:xlink=\"http://www.w3.org/1999/xlink\" width=\"{}\" height=\"{}\" viewBox=\"0 0 {} {}\">\n",
            width, height, width, height,
        ));
        self.defs_insert_pos = self.output.len();
    }

    fn end_page(&mut self) {
        // 디버그 오버레이 출력
        if self.debug_overlay {
            self.render_debug_overlay();
        }

        if !self.defs.is_empty() {
            let mut defs_block = String::from("<defs>\n");
            for def in &self.defs {
                defs_block.push_str(def);
            }
            defs_block.push_str("</defs>\n");
            self.output.insert_str(self.defs_insert_pos, &defs_block);
        }
        self.output.push_str("</svg>\n");
    }

    fn draw_text(&mut self, text: &str, x: f64, y: f64, style: &TextStyle) {
        // [Task #1067] inline 컨트롤 placeholder (U+FFFC OBJECT REPLACEMENT CHARACTER) 를
        // 보이지 않게 처리. HWP/HWPX 의 inline 도형/표/그림 등 treat_as_char 컨트롤이
        // paragraph text 자체에 U+FFFC 로 표현됨 — 도형 path 는 별도 emit 되므로 본
        // placeholder character 는 시각적으로 invisible 해야 한다. 한컴 정답지 정합.
        let text: String = text.chars().filter(|&c| c != '\u{FFFC}').collect();
        if text.is_empty() {
            return;
        }
        // [Task #509] 한컴은 폰트 지정과 상관없이 PUA 를 자체 처리. 지정 폰트에 글리프
        // 부재 시 한컴 내부 매핑이 발행. rhwp 도 동일 동작 모방 — 일반 텍스트도 PUA
        // 변환 적용 (PR #251 정합). 매핑 표는 한컴 PDF 정답지 기준.
        let text = &expand_pua_render_text(&text);
        // [Task #528] Hanyang-PUA 옛한글 → KS X 1026-1:2007 자모 시퀀스.
        // 한/글 2010 이전 옛한글 PUA 인코딩을 표준 자모로 변환 (KTUG 매핑).
        let text = &expand_pua_old_hangul(text);

        let color = color_to_svg(style.color);
        let base_font_size = if style.font_size > 0.0 {
            style.font_size
        } else {
            12.0
        };
        // 위첨자/아래첨자는 레이아웃 advance 는 원래 run 기준으로 유지하고,
        // 실제 SVG glyph 크기와 baseline 만 Canvas/HTML 출력과 동일하게 조정한다.
        let (font_size, y) = if style.superscript {
            (base_font_size * 0.7, y - base_font_size * 0.3)
        } else if style.subscript {
            (base_font_size * 0.7, y + base_font_size * 0.15)
        } else {
            (base_font_size, y)
        };
        let font_family = if style.font_family.is_empty() {
            "sans-serif".to_string()
        } else {
            let fb = super::generic_fallback(&style.font_family);
            format!("{},{}", style.font_family, fb)
        };

        let ratio = if style.ratio > 0.0 { style.ratio } else { 1.0 };
        let has_ratio = (ratio - 1.0).abs() > 0.01;

        // 공통 스타일 속성 구성 (fill 제외 — 그림자/원본에서 각각 설정)
        let mut base_attrs = format!(
            "font-family=\"{}\" font-size=\"{}\"",
            escape_xml(&font_family),
            font_size,
        );
        if style.is_visually_bold() {
            base_attrs.push_str(" font-weight=\"bold\"");
        } else if style.is_medium_weight() {
            base_attrs.push_str(" font-weight=\"500\"");
        }
        if style.italic {
            base_attrs.push_str(" font-style=\"italic\"");
        }

        // 클러스터 단위 렌더링: 옛한글 자모 조합 시퀀스를 하나의 <text>로 묶음
        let char_positions = compute_char_positions(text, style);
        let clusters = split_into_clusters(text);

        // 형광펜 배경 (CharShape.shade_color 기반 — web_canvas.rs와 동일 로직)
        let shade_rgb = style.shade_color & 0x00FFFFFF;
        if shade_rgb != 0x00FFFFFF && shade_rgb != 0 {
            let text_width = *char_positions.last().unwrap_or(&0.0);
            if text_width > 0.0 {
                self.output.push_str(&format!(
                    "<rect x=\"{:.4}\" y=\"{:.4}\" width=\"{:.4}\" height=\"{:.4}\" fill=\"{}\"/>\n",
                    x, y - font_size, text_width, font_size * 1.2,
                    color_to_svg(style.shade_color),
                ));
            }
        }

        // Task #257: `·`(U+00B7) 를 <text> 대신 <circle> 로 렌더한다.
        //
        // 폰트 대체(휴먼명조→Batang 등)로 각 폰트의 `·` 글리프 LSB 와 글리프
        // 폭이 달라, rhwp 의 metric DB 기반 advance 계산과 실제 브라우저 렌더
        // 위치가 어긋난다. 한글 문서에서 `·` 의 시각적 의미는 "두 글자 사이의
        // 중앙 점" 이므로 폰트 비의존 벡터 도형으로 직접 그린다.
        //
        //   cx = advance box 수평 중앙
        //   cy = baseline(y) − font_size × 0.35  (CJK x-height 중앙 근사)
        //   r  = font_size × 0.08               (PDF 관찰치 기준)
        let cluster_advance = |char_idx: usize, cluster_str: &str| -> f64 {
            let n = cluster_str.chars().count();
            let end = char_idx + n;
            if end < char_positions.len() {
                char_positions[end] - char_positions[char_idx]
            } else {
                0.0
            }
        };
        let is_middle_dot = |cluster_str: &str| cluster_str == "\u{00B7}";
        let dot_radius = font_size * 0.08;
        let dot_cy_offset = -font_size * 0.35;

        // Task #352: 3+ 연속 '-' 시퀀스(빈칸/leader) 를 단일 가로선으로 대체.
        // Stage 2 가 advance 를 좁히면 글리프 폭이 advance 를 초과해 시각상
        // 겹치므로 글리프 출력은 스킵하고 라인으로 통합. 가운데점 패턴과 동일.
        // 단, 같은 run 에 underline 이 설정된 경우 underline 이 빈칸의 시각
        // representation 을 담당하므로 dash leader 라인은 생략 (이중선 방지).
        let suppress_dash_leader_line = !matches!(style.underline, UnderlineType::None);
        let dash_run_groups: Vec<(usize, usize)> = {
            let mut groups = Vec::new();
            let mut run_start: Option<usize> = None;
            for (idx, (_, cs)) in clusters.iter().enumerate() {
                if cs == "-" {
                    if run_start.is_none() {
                        run_start = Some(idx);
                    }
                } else if let Some(s) = run_start.take() {
                    if idx - s >= 3 {
                        groups.push((s, idx));
                    }
                }
            }
            if let Some(s) = run_start {
                if clusters.len() - s >= 3 {
                    groups.push((s, clusters.len()));
                }
            }
            groups
        };
        let dash_line_y_offset = -font_size * 0.32; // baseline 기준 dash 중앙선 근사
        let dash_line_stroke_w = (font_size * 0.07).max(0.5);
        let cluster_in_dash_run = |cluster_idx: usize| -> Option<(f64, f64)> {
            // 첫 cluster 위치라면 (line_x1, line_x2) 반환, 외 None
            for &(s, e) in &dash_run_groups {
                if cluster_idx == s {
                    let start_char_idx = clusters[s].0;
                    let last = &clusters[e - 1];
                    let end_char_idx = last.0 + last.1.chars().count();
                    let x1 = char_positions.get(start_char_idx).copied().unwrap_or(0.0);
                    let x2 = char_positions
                        .get(end_char_idx)
                        .copied()
                        .unwrap_or_else(|| *char_positions.last().unwrap_or(&0.0));
                    return Some((x1, x2));
                }
                if cluster_idx > s && cluster_idx < e {
                    // run 내부 dash: 라인은 한 번만 그리고 글리프 출력은 모두 스킵
                    return Some((f64::NAN, f64::NAN));
                }
            }
            None
        };

        // 그림자 렌더링 (원본 아래에 오프셋된 그림자색 텍스트)
        if style.shadow_type > 0 {
            let shadow_color = color_to_svg(style.shadow_color);
            let shadow_attrs = format!("{} fill=\"{}\"", base_attrs, shadow_color);
            let dx = style.shadow_offset_x;
            let dy = style.shadow_offset_y;
            for (cluster_idx, (char_idx, cluster_str)) in clusters.iter().enumerate() {
                if cluster_str == " " || cluster_str == "\t" {
                    continue;
                }
                // Task #352: dash leader 시퀀스는 글리프 스킵, 필요 시 라인 1 회
                if let Some((x1_rel, x2_rel)) = cluster_in_dash_run(cluster_idx) {
                    if x1_rel.is_finite() && !suppress_dash_leader_line {
                        let line_y = y + dash_line_y_offset + dy;
                        self.output.push_str(&format!(
                            "<line x1=\"{:.4}\" y1=\"{:.4}\" x2=\"{:.4}\" y2=\"{:.4}\" stroke=\"{}\" stroke-width=\"{:.4}\"/>\n",
                            x + x1_rel + dx, line_y, x + x2_rel + dx, line_y, shadow_color, dash_line_stroke_w,
                        ));
                    }
                    continue;
                }
                if is_middle_dot(cluster_str) {
                    let adv = cluster_advance(*char_idx, cluster_str);
                    let cx = x + char_positions[*char_idx] + adv / 2.0 + dx;
                    let cy = y + dot_cy_offset + dy;
                    self.output.push_str(&format!(
                        "<circle cx=\"{:.4}\" cy=\"{:.4}\" r=\"{:.4}\" fill=\"{}\"/>\n",
                        cx, cy, dot_radius, shadow_color,
                    ));
                    continue;
                }
                let char_x = x + char_positions[*char_idx] + dx;
                let char_y = y + dy;
                let length_attrs = svg_text_length_attrs(
                    cluster_str,
                    cluster_advance(*char_idx, cluster_str),
                    ratio,
                );
                if has_ratio {
                    self.output.push_str(&format!(
                        "<text transform=\"translate({},{}) scale({:.4},1)\" {}{}>{}</text>\n",
                        char_x,
                        char_y,
                        ratio,
                        shadow_attrs,
                        length_attrs,
                        escape_xml(cluster_str),
                    ));
                } else {
                    self.output.push_str(&format!(
                        "<text x=\"{}\" y=\"{}\" {}{}>{}</text>\n",
                        char_x,
                        char_y,
                        shadow_attrs,
                        length_attrs,
                        escape_xml(cluster_str),
                    ));
                }
            }
        }

        // 원본 텍스트 렌더링
        let common_attrs = format!("{} fill=\"{}\"", base_attrs, color);
        for (cluster_idx, (char_idx, cluster_str)) in clusters.iter().enumerate() {
            if cluster_str == " " || cluster_str == "\t" {
                continue;
            }
            // Task #352: dash leader 시퀀스는 글리프 스킵, 필요 시 라인 1 회
            if let Some((x1_rel, x2_rel)) = cluster_in_dash_run(cluster_idx) {
                if x1_rel.is_finite() && !suppress_dash_leader_line {
                    let line_y = y + dash_line_y_offset;
                    self.output.push_str(&format!(
                        "<line x1=\"{:.4}\" y1=\"{:.4}\" x2=\"{:.4}\" y2=\"{:.4}\" stroke=\"{}\" stroke-width=\"{:.4}\"/>\n",
                        x + x1_rel, line_y, x + x2_rel, line_y, color, dash_line_stroke_w,
                    ));
                }
                continue;
            }
            if is_middle_dot(cluster_str) {
                let adv = cluster_advance(*char_idx, cluster_str);
                let cx = x + char_positions[*char_idx] + adv / 2.0;
                let cy = y + dot_cy_offset;
                self.output.push_str(&format!(
                    "<circle cx=\"{:.4}\" cy=\"{:.4}\" r=\"{:.4}\" fill=\"{}\"/>\n",
                    cx, cy, dot_radius, color,
                ));
                continue;
            }
            let char_x = x + char_positions[*char_idx];
            let length_attrs =
                svg_text_length_attrs(cluster_str, cluster_advance(*char_idx, cluster_str), ratio);

            if has_ratio {
                self.output.push_str(&format!(
                    "<text transform=\"translate({},{}) scale({:.4},1)\" {}{}>{}</text>\n",
                    char_x,
                    y,
                    ratio,
                    common_attrs,
                    length_attrs,
                    escape_xml(cluster_str),
                ));
            } else {
                self.output.push_str(&format!(
                    "<text x=\"{}\" y=\"{}\" {}{}>{}</text>\n",
                    char_x,
                    y,
                    common_attrs,
                    length_attrs,
                    escape_xml(cluster_str),
                ));
            }
        }

        // 밑줄 처리
        if !matches!(style.underline, UnderlineType::None) {
            let text_width = *char_positions.last().unwrap_or(&0.0);
            let ul_color = if style.underline_color != 0 {
                color_to_svg(style.underline_color)
            } else {
                color.to_string()
            };
            let ul_y = match style.underline {
                UnderlineType::Top => y - font_size + 1.0,
                _ => y + 2.0,
            };
            self.draw_line_shape(
                x,
                ul_y,
                x + text_width,
                ul_y,
                &ul_color,
                style.underline_shape,
            );
        }

        // 취소선 처리
        if style.strikethrough {
            let text_width = *char_positions.last().unwrap_or(&0.0);
            let strike_y = y - font_size * 0.3;
            let st_color = if style.strike_color != 0 {
                color_to_svg(style.strike_color)
            } else {
                color.to_string()
            };
            self.draw_line_shape(
                x,
                strike_y,
                x + text_width,
                strike_y,
                &st_color,
                style.strike_shape,
            );
        }

        // 강조점 처리
        if style.emphasis_dot > 0 {
            let dot_char = match style.emphasis_dot {
                1 => "●",
                2 => "○",
                3 => "ˇ",
                4 => "˜",
                5 => "･",
                6 => "˸",
                _ => "",
            };
            if !dot_char.is_empty() {
                let dot_size = font_size * 0.3;
                let dot_y = y - font_size * 1.05;
                for &cx in &char_positions[..char_positions.len().saturating_sub(1)] {
                    let dot_x = x + cx + (font_size * style.ratio * 0.5);
                    self.output.push_str(&format!(
                        "<text x=\"{}\" y=\"{}\" font-size=\"{}\" text-anchor=\"middle\" fill=\"{}\">{}</text>\n",
                        dot_x, dot_y, dot_size, color, dot_char,
                    ));
                }
            }
        }

        // 탭 리더(채움 기호) 렌더링
        for leader in &style.tab_leaders {
            if leader.fill_type == 0 {
                continue;
            }
            let lx1 = x + leader.start_x;
            let leader_end_x = clamp_tab_leader_end_x(text, &char_positions, leader, font_size);
            let lx2 = x + leader_end_x;
            let ly = y - font_size * 0.35; // 글자 세로 중앙 (베이스라인에서 x-height 절반)
                                           // 채울 모양 12종: 0=없음, 1=실선, 2=파선, 3=점선, 4=일점쇄선,
                                           // 5=이점쇄선, 6=긴파선, 7=원형점선, 8=이중실선,
                                           // 9=얇고굵은이중선, 10=굵고얇은이중선, 11=얇고굵고얇은삼중선
            match leader.fill_type {
                1 => {
                    // 실선
                    self.output.push_str(&format!(
                        "<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"{}\" stroke-width=\"0.5\"/>\n",
                        lx1, ly, lx2, ly, color,
                    ));
                }
                2 => {
                    // 파선 - - -
                    self.output.push_str(&format!(
                        "<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"{}\" stroke-width=\"0.5\" stroke-dasharray=\"3 3\"/>\n",
                        lx1, ly, lx2, ly, color,
                    ));
                }
                3 => {
                    // 점선 ··· — round cap으로 원형 점 표현 (한컴 동등)
                    self.output.push_str(&format!(
                        "<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"{}\" stroke-width=\"1.0\" stroke-dasharray=\"0.1 3\" stroke-linecap=\"round\"/>\n",
                        lx1, ly, lx2, ly, color,
                    ));
                }
                4 => {
                    // 일점쇄선 -·-·
                    self.output.push_str(&format!(
                        "<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"{}\" stroke-width=\"0.5\" stroke-dasharray=\"6 2 1 2\"/>\n",
                        lx1, ly, lx2, ly, color,
                    ));
                }
                5 => {
                    // 이점쇄선 -··-··
                    self.output.push_str(&format!(
                        "<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"{}\" stroke-width=\"0.5\" stroke-dasharray=\"6 2 1 2 1 2\"/>\n",
                        lx1, ly, lx2, ly, color,
                    ));
                }
                6 => {
                    // 긴파선 ── ──
                    self.output.push_str(&format!(
                        "<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"{}\" stroke-width=\"0.5\" stroke-dasharray=\"8 4\"/>\n",
                        lx1, ly, lx2, ly, color,
                    ));
                }
                7 => {
                    // 원형점선 ●●●
                    self.output.push_str(&format!(
                        "<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"{}\" stroke-width=\"0.7\" stroke-dasharray=\"0.1 2.5\" stroke-linecap=\"round\"/>\n",
                        lx1, ly, lx2, ly, color,
                    ));
                }
                8 => {
                    // 이중실선 ═══
                    self.output.push_str(&format!(
                        "<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"{}\" stroke-width=\"0.3\"/>\n\
                         <line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"{}\" stroke-width=\"0.3\"/>\n",
                        lx1, ly - 1.0, lx2, ly - 1.0, color,
                        lx1, ly + 1.0, lx2, ly + 1.0, color,
                    ));
                }
                9 => {
                    // 얇고 굵은 이중선
                    self.output.push_str(&format!(
                        "<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"{}\" stroke-width=\"0.3\"/>\n\
                         <line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"{}\" stroke-width=\"0.8\"/>\n",
                        lx1, ly - 1.2, lx2, ly - 1.2, color,
                        lx1, ly + 0.8, lx2, ly + 0.8, color,
                    ));
                }
                10 => {
                    // 굵고 얇은 이중선
                    self.output.push_str(&format!(
                        "<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"{}\" stroke-width=\"0.8\"/>\n\
                         <line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"{}\" stroke-width=\"0.3\"/>\n",
                        lx1, ly - 0.8, lx2, ly - 0.8, color,
                        lx1, ly + 1.2, lx2, ly + 1.2, color,
                    ));
                }
                11 => {
                    // 얇고 굵고 얇은 삼중선
                    self.output.push_str(&format!(
                        "<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"{}\" stroke-width=\"0.3\"/>\n\
                         <line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"{}\" stroke-width=\"0.8\"/>\n\
                         <line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"{}\" stroke-width=\"0.3\"/>\n",
                        lx1, ly - 2.0, lx2, ly - 2.0, color,
                        lx1, ly, lx2, ly, color,
                        lx1, ly + 2.0, lx2, ly + 2.0, color,
                    ));
                }
                _ => {
                    // 알 수 없는 타입: 점선 폴백
                    self.output.push_str(&format!(
                        "<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"{}\" stroke-width=\"0.5\" stroke-dasharray=\"1 2\"/>\n",
                        lx1, ly, lx2, ly, color,
                    ));
                }
            }
        }
    }

    fn draw_rect(
        &mut self,
        x: f64,
        y: f64,
        w: f64,
        h: f64,
        corner_radius: f64,
        style: &ShapeStyle,
    ) {
        self.draw_rect_with_gradient(x, y, w, h, corner_radius, style, None);
    }

    fn draw_line(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, style: &LineStyle) {
        let color = color_to_svg(style.color);
        let width = if style.width > 0.0 { style.width } else { 1.0 };

        // 이중선/삼중선 처리: 여러 평행선으로 렌더링
        match style.line_type {
            super::LineRenderType::Double
            | super::LineRenderType::ThinThickDouble
            | super::LineRenderType::ThickThinDouble
            | super::LineRenderType::ThinThickThinTriple => {
                self.draw_multi_line(x1, y1, x2, y2, width, &color, &style.line_type);
                return;
            }
            _ => {}
        }

        let dx = x2 - x1;
        let dy = y2 - y1;
        let line_len = (dx * dx + dy * dy).sqrt();

        // 화살표 머리 크기만큼 선 끝점 조정
        // 선이 화살표 머리 안으로 침범하지 않도록 줄임
        let mut lx1 = x1;
        let mut ly1 = y1;
        let mut lx2 = x2;
        let mut ly2 = y2;
        let mut marker_start_attr = String::new();
        let mut marker_end_attr = String::new();

        if line_len > 0.0 {
            let ux = dx / line_len; // 단위 벡터
            let uy = dy / line_len;

            if style.start_arrow != super::ArrowStyle::None {
                let (arrow_w, _) = Self::calc_arrow_dims(width, line_len, style.start_arrow_size);
                let marker_id = self.ensure_arrow_marker(
                    &color,
                    width,
                    line_len,
                    &style.start_arrow,
                    style.start_arrow_size,
                    true,
                );
                marker_start_attr = format!(" marker-start=\"url(#{})\"", marker_id);
                // 시작점을 화살표 길이만큼 전진
                lx1 += ux * arrow_w;
                ly1 += uy * arrow_w;
            }
            if style.end_arrow != super::ArrowStyle::None {
                let (arrow_w, _) = Self::calc_arrow_dims(width, line_len, style.end_arrow_size);
                let marker_id = self.ensure_arrow_marker(
                    &color,
                    width,
                    line_len,
                    &style.end_arrow,
                    style.end_arrow_size,
                    false,
                );
                marker_end_attr = format!(" marker-end=\"url(#{})\"", marker_id);
                // 끝점을 화살표 길이만큼 후퇴
                lx2 -= ux * arrow_w;
                ly2 -= uy * arrow_w;
            }
        }

        let mut attrs = format!(
            "x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"{}\" stroke-width=\"{}\"",
            lx1, ly1, lx2, ly2, color, width,
        );
        match style.dash {
            super::StrokeDash::Dash => attrs.push_str(" stroke-dasharray=\"6 3\""),
            super::StrokeDash::Dot => attrs.push_str(" stroke-dasharray=\"2 2\""),
            super::StrokeDash::DashDot => attrs.push_str(" stroke-dasharray=\"6 3 2 3\""),
            super::StrokeDash::DashDotDot => attrs.push_str(" stroke-dasharray=\"6 3 2 3 2 3\""),
            _ => {} // Solid
        }
        attrs.push_str(&marker_start_attr);
        attrs.push_str(&marker_end_attr);
        self.output.push_str(&format!("<line {}/>\n", attrs));
    }

    fn draw_ellipse(&mut self, cx: f64, cy: f64, rx: f64, ry: f64, style: &ShapeStyle) {
        self.draw_ellipse_with_gradient(cx, cy, rx, ry, style, None);
    }

    fn draw_image(&mut self, data: &[u8], x: f64, y: f64, w: f64, h: f64) {
        let mime_type = detect_image_mime_type(data);
        let (render_data, render_mime): (std::borrow::Cow<[u8]>, &str) =
            if mime_type == "image/x-wmf" {
                match convert_wmf_to_svg(data) {
                    Some(svg_bytes) => (std::borrow::Cow::Owned(svg_bytes), "image/svg+xml"),
                    None => (std::borrow::Cow::Borrowed(data), mime_type),
                }
            } else if mime_type == "image/x-pcx" {
                match pcx_bytes_to_png_bytes(data) {
                    Some(png_bytes) => (std::borrow::Cow::Owned(png_bytes), "image/png"),
                    None => (std::borrow::Cow::Borrowed(data), mime_type),
                }
            } else {
                (std::borrow::Cow::Borrowed(data), mime_type)
            };
        let base64_data = base64::engine::general_purpose::STANDARD.encode(&*render_data);
        let data_uri = format!("data:{};base64,{}", render_mime, base64_data);
        self.output.push_str(&format!(
            "<image x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" preserveAspectRatio=\"none\" href=\"{}\"/>\n",
            x, y, w, h, data_uri,
        ));
    }

    fn draw_path(&mut self, commands: &[PathCommand], style: &ShapeStyle) {
        self.draw_path_with_gradient(commands, style, None);
    }
}

/// COLORREF (BGR) → SVG 색상 문자열 변환
fn color_to_svg(color: u32) -> String {
    let b = (color >> 16) & 0xFF;
    let g = (color >> 8) & 0xFF;
    let r = color & 0xFF;
    format!("#{:02x}{:02x}{:02x}", r, g, b)
}

fn svg_text_length_attrs(cluster_str: &str, cluster_advance: f64, scale_x: f64) -> String {
    if !cluster_str.chars().any(|ch| ch.is_ascii_alphanumeric()) {
        return String::new();
    }
    if !cluster_advance.is_finite() || cluster_advance <= 0.0 {
        return String::new();
    }
    let scale_x = if scale_x.is_finite() && scale_x.abs() > 0.0001 {
        scale_x.abs()
    } else {
        1.0
    };
    let text_length = cluster_advance / scale_x;
    format!(
        " textLength=\"{:.4}\" lengthAdjust=\"spacingAndGlyphs\"",
        text_length
    )
}

/// XML 특수문자 이스케이프
fn escape_xml(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => result.push_str("&amp;"),
            '<' => result.push_str("&lt;"),
            '>' => result.push_str("&gt;"),
            '"' => result.push_str("&quot;"),
            '\'' => result.push_str("&apos;"),
            // XML 1.0 허용 문자: #x9 | #xA | #xD | [#x20-#xD7FF] | [#xE000-#xFFFD] | [#x10000-#x10FFFF]
            // 그 외(제어문자, U+FFFE, U+FFFF 등)는 제거
            '\u{09}' | '\u{0A}' | '\u{0D}' => result.push(c),
            '\u{20}'..='\u{D7FF}' | '\u{E000}'..='\u{FFFD}' | '\u{10000}'..='\u{10FFFF}' => {
                result.push(c)
            }
            _ => {} // XML 무효 문자 제거
        }
    }
    result
}

/// WMF 바이트를 SVG로 변환한다. 실패 시 None 반환.
pub(crate) fn convert_wmf_to_svg(data: &[u8]) -> Option<Vec<u8>> {
    use crate::wmf::converter::{SVGPlayer, WMFConverter};
    let player = SVGPlayer::new();
    let converter = WMFConverter::new(data, player);
    converter.run().ok()
}

/// 이미지 데이터에서 픽셀 크기(width, height)를 파싱한다.
/// HWP `pic.crop` (HWPUNIT) 로부터 SVG `viewBox` 에 쓸 원본 픽셀 단위
/// source rect (x, y, w, h) 를 계산한다.
///
/// [Task #477] HWP 표준 룰: 1 inch = 7200 HU = 96 px → **75 HU/px** (DPI 96).
/// 한컴이 BinData 에 저장하는 image 의 표준 DPI 이며, crop 좌표 (HU) 와 image
/// 픽셀의 변환은 이 표준 scale 로 항상 정합한다.
///
/// `original_size_hu` 인자는 라운드트립 보존 메타로만 유지하며 계산에는 사용하지
/// 않는다 (Task #430 이 도입했던 `orig/img_w` scale 은 일부 케이스에서 결함을
/// 유발 — k-water-rfp pi=31 등에서 image 좌측만 표시되는 회귀).
pub(crate) fn compute_image_crop_src(
    crop_hu: (i32, i32, i32, i32),
    _original_size_hu: Option<(u32, u32)>,
    _img_w_px: f64,
    _img_h_px: f64,
) -> (f64, f64, f64, f64) {
    let (cl, ct, cr, cb) = crop_hu;
    // HWP 표준 DPI 96 = 75 HU/px
    const HU_PER_PX: f64 = 75.0;
    let scale_x = HU_PER_PX;
    let scale_y = HU_PER_PX;
    let src_x = cl as f64 / scale_x;
    let src_y = ct as f64 / scale_y;
    let src_w = (cr - cl) as f64 / scale_x;
    let src_h = (cb - ct) as f64 / scale_y;
    (src_x, src_y, src_w, src_h)
}

fn parse_image_dimensions(data: &[u8]) -> Option<(u32, u32)> {
    if data.len() < 24 {
        return None;
    }

    // PNG: IHDR 청크에서 크기 읽기 (바이트 16-23)
    if data.starts_with(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]) {
        let w = u32::from_be_bytes([data[16], data[17], data[18], data[19]]);
        let h = u32::from_be_bytes([data[20], data[21], data[22], data[23]]);
        return Some((w, h));
    }

    // JPEG: SOF 마커에서 크기 읽기
    if data.starts_with(&[0xFF, 0xD8, 0xFF]) {
        let mut i = 2;
        while i + 9 < data.len() {
            if data[i] != 0xFF {
                i += 1;
                continue;
            }
            let marker = data[i + 1];
            // SOF0-SOF3 (0xC0-0xC3), SOF5-SOF7 (0xC5-0xC7),
            // SOF9-SOF11 (0xC9-0xCB), SOF13-SOF15 (0xCD-0xCF)
            if (marker >= 0xC0 && marker <= 0xCF)
                && marker != 0xC4
                && marker != 0xC8
                && marker != 0xCC
            {
                let h = u16::from_be_bytes([data[i + 5], data[i + 6]]) as u32;
                let w = u16::from_be_bytes([data[i + 7], data[i + 8]]) as u32;
                if w > 0 && h > 0 {
                    return Some((w, h));
                }
            }
            let seg_len = u16::from_be_bytes([data[i + 2], data[i + 3]]) as usize;
            i += 2 + seg_len;
        }
        return None;
    }

    // GIF: 바이트 6-9
    if data.starts_with(b"GIF87a") || data.starts_with(b"GIF89a") {
        let w = u16::from_le_bytes([data[6], data[7]]) as u32;
        let h = u16::from_le_bytes([data[8], data[9]]) as u32;
        return Some((w, h));
    }

    // BMP: 바이트 18-25
    if data.starts_with(&[0x42, 0x4D]) && data.len() >= 26 {
        let w = u32::from_le_bytes([data[18], data[19], data[20], data[21]]);
        let h = i32::from_le_bytes([data[22], data[23], data[24], data[25]]);
        return Some((w, h.unsigned_abs()));
    }

    None
}

/// 폰트명 → local() 별칭 매핑 (한글명 + 영문명)
fn font_local_aliases(font_family: &str) -> Vec<&'static str> {
    match font_family {
        "함초롬바탕" => vec!["함초롬바탕", "HCR Batang"],
        "함초롬돋움" => vec!["함초롬돋움", "HCR Dotum"],
        "함초롱바탕" => vec!["함초롱바탕", "HCR Batang"],
        "함초롱돋움" => vec!["함초롱돋움", "HCR Dotum"],
        "한컴바탕" => vec!["한컴바탕", "함초롬바탕", "HCR Batang"],
        "한컴돋움" => vec!["한컴돋움", "함초롬돋움", "HCR Dotum"],
        "맑은 고딕" => vec!["맑은 고딕", "Malgun Gothic"],
        "바탕" => vec!["바탕", "Batang"],
        "돋움" => vec!["돋움", "Dotum"],
        "굴림" => vec!["굴림", "Gulim"],
        "굴림체" => vec!["굴림체", "GulimChe"],
        "바탕체" => vec!["바탕체", "BatangChe"],
        "궁서" => vec!["궁서", "Gungsuh"],
        "궁서체" => vec!["궁서체", "GungsuhChe"],
        _ => vec![],
    }
}

/// 폰트명 → 알려진 파일명 매핑 (HWP/한컴/MS 폰트)
fn known_font_filenames(font_name: &str) -> Vec<&'static str> {
    match font_name {
        "함초롬바탕" | "함초롱바탕" | "한컴바탕" => {
            vec!["hamchob-r.ttf", "HBATANG.TTF"]
        }
        "함초롬돋움" | "함초롱돋움" | "한컴돋움" => {
            vec!["hamchod-r.ttf", "HDOTUM.TTF"]
        }
        "HY헤드라인M" | "HYHeadLine M" => vec!["H2HDRM.TTF"],
        "HY견고딕" | "HYGothic-Extra" => vec!["HYGTRE.TTF"],
        "HY그래픽" | "HYGraphic-Medium" => vec!["HYGPRM.TTF"],
        "HY견명조" | "HYMyeongJo-Extra" => vec!["HYMJRE.TTF"],
        "HY신명조" => vec!["HYSNMJ.TTF", "hamchob-r.ttf"],
        "Latin Modern Math" => vec![
            "latinmodern-math.otf",
            "LatinModernMath-Regular.otf",
            "lmmath-regular.otf",
        ],
        "맑은 고딕" | "Malgun Gothic" => vec!["malgun.ttf", "MalgunGothic.ttf"],
        "바탕" | "Batang" => vec!["batang.ttc", "BATANG.TTC", "hamchob-r.ttf"],
        "돋움" | "Dotum" => vec!["dotum.ttc", "DOTUM.TTC", "hamchod-r.ttf"],
        "굴림" | "Gulim" => vec!["gulim.ttc", "GULIM.TTC", "hamchod-r.ttf"],
        "궁서" | "Gungsuh" => vec!["gungsuh.ttc", "GUNGSUH.TTC", "hamchob-r.ttf"],
        "굴림체" | "GulimChe" => vec!["gulim.ttc", "hamchod-r.ttf"],
        "바탕체" | "BatangChe" => vec!["batang.ttc", "hamchob-r.ttf"],
        "휴먼명조" => vec!["HYMJRE.TTF", "hamchob-r.ttf"],
        "새바탕" | "새돋움" | "새굴림" | "새궁서" => {
            vec!["hamchob-r.ttf", "hamchod-r.ttf"]
        }
        _ => vec![],
    }
}

/// Task #1224: 한국어 고딕(돋움/고딕/굴림) 계열의 오픈소스 대체 폰트 파일명.
///
/// 한컴/MS 저작권 고딕(한컴돋움·Haansoft Dotum·맑은 고딕·돋움·굴림 등) 파일 부재 시
/// 임베딩의 **최후 후보**로 사용한다. 현 폴백(Noto Sans CJK KR Regular)은 한컴 돋움보다
/// 획이 +43% 두꺼워(페이지 밀도 0.378 vs 0.265) 본문이 과도하게 굵게 렌더되므로, 획 두께가
/// 한컴 돋움에 근접한 Noto Sans KR ExtraLight(rsvg 페이지 밀도 0.277)로 교정한다
/// (`ttfs/opensource/`).
///
/// serif(바탕/명조/궁서)·라틴 폰트에는 적용하지 않는다(시각 정합과 무관). 실제 저작권
/// 폰트가 탐색 경로에 있으면 그쪽이 우선한다(대체는 탐색 경로 말단의 `ttfs/opensource/`).
///
/// 주의: 현 임베딩 subsetter(typst, PDF용)는 cmap 을 제거하므로 @font-face 임베딩은
/// 브라우저 `<text>` 매핑에 무효. 본 대체의 실효 경로는 **폴백 체인 + fontconfig/웹폰트로
/// 설치된 ExtraLight** 이다(Task #1224 보고서 참조).
#[cfg(not(target_arch = "wasm32"))]
fn korean_gothic_substitute(font_name: &str) -> Option<&'static str> {
    let lower = font_name.to_ascii_lowercase();
    let is_gothic = font_name.contains("돋움")
        || font_name.contains("고딕")
        || font_name.contains("굴림")
        || lower.contains("dotum")
        || lower.contains("gothic")
        || lower.contains("gulim");
    if is_gothic {
        Some("NotoSansKR-ExtraLight.ttf")
    } else {
        None
    }
}

/// 폰트명으로 TTF/OTF 파일을 탐색한다.
#[cfg(not(target_arch = "wasm32"))]
fn find_font_file(
    font_name: &str,
    extra_paths: &[std::path::PathBuf],
) -> Option<std::path::PathBuf> {
    use std::path::Path;

    // 폰트명 → 파일명 후보 생성
    let candidates: Vec<String> = {
        let mut files: Vec<String> = known_font_filenames(font_name)
            .iter()
            .map(|s| s.to_string())
            .collect();
        let aliases = font_local_aliases(font_name);
        let mut names = vec![font_name.to_string()];
        for a in &aliases {
            names.push(a.to_string());
        }
        for name in &names {
            let clean = name.replace(' ', "");
            files.push(format!("{}.ttf", name));
            files.push(format!("{}.otf", name));
            files.push(format!("{}.ttc", name));
            files.push(format!("{}.TTF", name));
            if clean != *name {
                files.push(format!("{}.ttf", clean));
                files.push(format!("{}.otf", clean));
                files.push(format!("{}.ttc", clean));
            }
        }
        // Task #1224: 고딕 계열은 오픈소스 대체(Noto Sans KR ExtraLight)를 최후 후보로 추가.
        // 실제 저작권 폰트가 앞선 탐색 경로에 있으면 그쪽이 우선하므로, 대체는
        // 탐색 경로 말단(ttfs/opensource)에서만 매칭된다.
        if let Some(sub) = korean_gothic_substitute(font_name) {
            files.push(sub.to_string());
        }
        files
    };

    // 탐색 경로 (우선순위 순)
    let mut search_dirs: Vec<std::path::PathBuf> = extra_paths.to_vec();
    for dir in &["ttfs/hwp", "ttfs/windows", "ttfs"] {
        search_dirs.push(Path::new(dir).to_path_buf());
    }
    // 시스템 폰트 경로
    #[cfg(target_os = "macos")]
    {
        search_dirs.push(Path::new("/Library/Fonts").to_path_buf());
        search_dirs.push(Path::new("/System/Library/Fonts").to_path_buf());
        search_dirs.push(Path::new("/System/Library/Fonts/Supplemental").to_path_buf());
    }
    #[cfg(target_os = "linux")]
    {
        search_dirs.push(Path::new("/usr/share/fonts").to_path_buf());
        search_dirs.push(Path::new("/usr/local/share/fonts").to_path_buf());
    }
    #[cfg(target_os = "windows")]
    {
        search_dirs.push(Path::new("C:\\Windows\\Fonts").to_path_buf());
    }
    // WSL Windows 폰트
    if Path::new("/mnt/c/Windows/Fonts").exists() {
        search_dirs.push(Path::new("/mnt/c/Windows/Fonts").to_path_buf());
    }
    // Task #1224: 오픈소스 번들 대체 폰트 경로 — **최후 탐색**(실제 저작권/시스템 폰트가
    // 항상 우선). 고딕 계열의 Noto Sans KR ExtraLight 대체가 여기서만 매칭된다.
    search_dirs.push(Path::new("ttfs/opensource").to_path_buf());

    for dir in &search_dirs {
        if !dir.exists() {
            continue;
        }
        for candidate in &candidates {
            let path = dir.join(candidate);
            if path.exists() {
                return Some(path);
            }
        }
    }
    None
}

/// SvgRenderer의 수집된 폰트 정보를 기반으로 @font-face CSS를 생성한다.
#[cfg(not(target_arch = "wasm32"))]
pub fn generate_font_style(renderer: &SvgRenderer, font_paths: &[std::path::PathBuf]) -> String {
    let codepoints = renderer.font_codepoints();
    if codepoints.is_empty() {
        return String::new();
    }

    let mut css = String::new();

    match renderer.font_embed_mode {
        FontEmbedMode::Style => {
            for font_name in codepoints.keys() {
                let aliases = font_local_aliases(font_name);
                let src = if aliases.is_empty() {
                    format!("local(\"{}\")", font_name)
                } else {
                    aliases
                        .iter()
                        .map(|a| format!("local(\"{}\")", a))
                        .collect::<Vec<_>>()
                        .join(", ")
                };
                css.push_str(&format!(
                    "@font-face {{ font-family: \"{}\"; src: {}; }}\n",
                    font_name, src,
                ));
            }
        }
        FontEmbedMode::Subset => {
            for (font_name, chars) in codepoints.iter() {
                if let Some(font_path) = find_font_file(font_name, font_paths) {
                    if let Ok(font_data) = std::fs::read(&font_path) {
                        // codepoint → glyph ID 변환 (ttf-parser cmap 사용)
                        let mut remapper = subsetter::GlyphRemapper::new();
                        if let Ok(face) = ttf_parser::Face::parse(&font_data, 0) {
                            // glyph 0 (.notdef) 항상 포함
                            remapper.remap(0);
                            for ch in chars {
                                if let Some(gid) = face.glyph_index(*ch) {
                                    remapper.remap(gid.0);
                                }
                            }
                        }
                        // 서브셋 추출
                        match subsetter::subset(&font_data, 0, &remapper) {
                            Ok(subset_data) => {
                                let b64 =
                                    base64::engine::general_purpose::STANDARD.encode(&subset_data);
                                css.push_str(&format!(
                                    "@font-face {{ font-family: \"{}\"; src: url(\"data:font/opentype;base64,{}\") format(\"opentype\"); }}\n",
                                    font_name, b64,
                                ));
                                eprintln!(
                                    "  [font-embed] {} → 서브셋 {:.1}KB ({}글자, 원본 {:.1}KB)",
                                    font_name,
                                    subset_data.len() as f64 / 1024.0,
                                    chars.len(),
                                    font_data.len() as f64 / 1024.0
                                );
                                continue;
                            }
                            Err(e) => {
                                eprintln!(
                                    "  [font-embed] {} 서브셋 실패: {} → local() 폴백",
                                    font_name, e
                                );
                            }
                        }
                    }
                }
                // 폰트 파일 없거나 서브셋 실패 → local() 폴백
                let aliases = font_local_aliases(font_name);
                let src = if aliases.is_empty() {
                    format!("local(\"{}\")", font_name)
                } else {
                    aliases
                        .iter()
                        .map(|a| format!("local(\"{}\")", a))
                        .collect::<Vec<_>>()
                        .join(", ")
                };
                css.push_str(&format!(
                    "@font-face {{ font-family: \"{}\"; src: {}; }}\n",
                    font_name, src,
                ));
            }
        }
        FontEmbedMode::Full => {
            for font_name in codepoints.keys() {
                if let Some(font_path) = find_font_file(font_name, font_paths) {
                    if let Ok(font_data) = std::fs::read(&font_path) {
                        let b64 = base64::engine::general_purpose::STANDARD.encode(&font_data);
                        css.push_str(&format!(
                            "@font-face {{ font-family: \"{}\"; src: url(\"data:font/opentype;base64,{}\") format(\"opentype\"); }}\n",
                            font_name, b64,
                        ));
                        eprintln!(
                            "  [font-embed] {} → 전체 {:.1}KB",
                            font_name,
                            font_data.len() as f64 / 1024.0
                        );
                        continue;
                    }
                }
                // 폰트 파일 없음 → local() 폴백
                let aliases = font_local_aliases(font_name);
                let src = if aliases.is_empty() {
                    format!("local(\"{}\")", font_name)
                } else {
                    aliases
                        .iter()
                        .map(|a| format!("local(\"{}\")", a))
                        .collect::<Vec<_>>()
                        .join(", ")
                };
                css.push_str(&format!(
                    "@font-face {{ font-family: \"{}\"; src: {}; }}\n",
                    font_name, src,
                ));
            }
        }
        FontEmbedMode::None => {}
    }

    css
}

#[cfg(test)]
mod tests;

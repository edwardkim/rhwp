//! 메모 렌더링 레이아웃 (--show-memos)
//!
//! 본문 조판 중 수집된 메모 앵커([`MemoAnchor`])를 페이지 우측 여백 바깥에
//! 메모 박스로 배치한다. 한컴 편집기의 "메모 보기" 화면 구성을 따른다:
//! 앵커 밑줄 + 연결선 + 우측 메모 박스(작성자 라벨 + 본문 문단).
//!
//! 한컴 PDF 변환본(권위 자료)에는 메모가 출력되지 않으므로 이 경로는
//! 옵트인 플래그(`--show-memos`)가 켜졌을 때만 동작한다 — 기본 출력 불변.

use super::super::composer::{compose_paragraph, recompose_for_cell_width};
use super::super::page_layout::{LayoutRect, PageLayoutInfo};
use super::super::render_tree::*;
use super::super::style_resolver::ResolvedStyleSet;
use super::super::{hwpunit_to_px, LineStyle, ShapeStyle};
use super::text_measurement::resolved_to_text_style;
use super::LayoutEngine;
use crate::model::paragraph::Paragraph;

/// 본문 조판 중 수집되는 메모 앵커 정보.
/// 필드 시작 문자가 놓인 줄의 좌표와 메모 본문(subList) 사본을 담는다.
pub(crate) struct MemoAnchor {
    pub section_index: usize,
    pub para_index: usize,
    pub control_idx: usize,
    /// 앵커(필드 시작 문자) x 좌표
    pub x: f64,
    /// 앵커 밑줄 끝 x (필드 끝이 같은 줄이면 끝 문자, 아니면 줄 끝)
    pub end_x: f64,
    /// 앵커 줄 상단 y
    pub y: f64,
    /// 앵커 줄 높이
    pub line_height: f64,
    /// 메모 번호 (fieldBegin Number 파라미터)
    #[allow(dead_code)]
    pub memo_index: u32,
    /// 작성자 (MEMO command 6번째 성분)
    pub author: Option<String>,
    /// 메모 본문 문단 (fieldBegin subList)
    pub paragraphs: Vec<Paragraph>,
}

/// 페이지 우측 끝과 메모 박스 사이 간격 (px)
const MEMO_GUTTER_GAP: f64 = 12.0;
/// 메모 박스 내부 패딩 (px)
const MEMO_BOX_PADDING: f64 = 6.0;
/// 메모 박스 사이 세로 간격 (px)
const MEMO_BOX_VGAP: f64 = 8.0;
/// 앵커 밑줄 두께 (px)
const ANCHOR_UNDERLINE_H: f64 = 2.0;

impl LayoutEngine {
    /// 수집된 메모 앵커를 소비해 우측 여백 밖에 메모 박스를 배치한다.
    /// build_render_tree 마지막(모든 콘텐츠 위)에 호출된다.
    pub(crate) fn build_memo_areas(
        &self,
        tree: &mut PageRenderTree,
        styles: &ResolvedStyleSet,
        layout: &PageLayoutInfo,
    ) {
        if !self.show_memos.get() {
            return;
        }
        let mut anchors = std::mem::take(&mut *self.memo_anchors.borrow_mut());
        if anchors.is_empty() {
            return;
        }

        // 표 측정 등 사전 패스에서 같은 필드가 중복 수집될 수 있으므로
        // (section, para, control) 별 마지막 수집(최종 배치 좌표)만 유지한다.
        {
            let mut seen = std::collections::HashSet::new();
            let mut kept: Vec<MemoAnchor> = Vec::with_capacity(anchors.len());
            for a in anchors.into_iter().rev() {
                if seen.insert((a.section_index, a.para_index, a.control_idx)) {
                    kept.push(a);
                }
            }
            kept.reverse();
            anchors = kept;
        }
        // 앵커 세로 순서대로 박스를 쌓는다.
        anchors.sort_by(|a, b| a.y.total_cmp(&b.y).then(a.x.total_cmp(&b.x)));

        let shape = self
            .memo_shapes
            .borrow()
            .first()
            .cloned()
            .unwrap_or_default();
        let box_w = hwpunit_to_px(shape.width as i32, self.dpi).max(80.0);
        let inner_w = box_w - 2.0 * MEMO_BOX_PADDING;
        let box_x = layout.page_width + MEMO_GUTTER_GAP;
        let border_w = (shape.line_width as f64 * 0.25).max(0.75);

        let mut prev_bottom = f64::NEG_INFINITY;
        for anchor in &anchors {
            let memo_node_id = tree.next_id();
            let mut memo_node = RenderNode::new(
                memo_node_id,
                RenderNodeType::MemoArea,
                BoundingBox::new(box_x, 0.0, box_w, 0.0),
            );

            // (1) 내용을 y=0 기준으로 조판해 높이를 확정한 뒤 최종 y 로 이동한다.
            let content_area = LayoutRect {
                x: box_x + MEMO_BOX_PADDING,
                y: 0.0,
                width: inner_w,
                height: layout.page_height,
            };
            let mut content_y = 0.0_f64;

            // 작성자 라벨 (박스 상단, 본문보다 작게)
            if let Some(author) = &anchor.author {
                content_y = self.layout_memo_author_label(
                    tree,
                    &mut memo_node,
                    author,
                    anchor,
                    styles,
                    &content_area,
                );
            }

            // 메모 본문. subList 의 lineseg 는 한컴 화면 메모 창 폭 기준이라
            // 박스 폭과 무관하므로 비우고 박스 내부 폭으로 재조판한다.
            for (p_idx, para) in anchor.paragraphs.iter().enumerate() {
                let mut p = para.clone();
                p.line_segs.clear();
                let mut composed = compose_paragraph(&p);
                recompose_for_cell_width(&mut composed, &p, inner_w, styles);
                let line_count = composed.lines.len();
                content_y = self.layout_composed_paragraph(
                    tree,
                    &mut memo_node,
                    &composed,
                    styles,
                    &content_area,
                    content_y,
                    0,
                    line_count,
                    // 히트테스트 식별용 sentinel (각주의 usize::MAX-2000 스킴과 구분)
                    anchor.section_index,
                    usize::MAX - 3000 - p_idx,
                    None,
                    false,
                    false,
                    0.0,
                    None,
                    None,
                    None,
                    None,
                );
            }
            let content_h = content_y.max(12.0);
            let box_h = content_h + 2.0 * MEMO_BOX_PADDING;

            // (2) 박스 y 확정: 앵커 줄 상단에 맞추되 앞 박스와 겹치지 않게 내리고,
            //     페이지 하단을 넘으면 가능한 범위에서 끌어올린다.
            let min_y = if prev_bottom.is_finite() {
                prev_bottom + MEMO_BOX_VGAP
            } else {
                0.0
            };
            let mut box_y = anchor.y.max(min_y);
            if box_y + box_h > layout.page_height {
                box_y = (layout.page_height - box_h).max(min_y).max(0.0);
            }
            prev_bottom = box_y + box_h;

            // (3) 조판된 내용을 최종 위치로 이동
            let dy = box_y + MEMO_BOX_PADDING;
            for child in &mut memo_node.children {
                translate_node_y(child, dy);
            }
            memo_node.bbox = BoundingBox::new(box_x, box_y, box_w, box_h);

            // (4) 배경 박스 (memoPr fillColor/lineColor) — children 맨 앞에 삽입
            let bg_style = ShapeStyle {
                fill_color: Some(shape.fill_color),
                stroke_color: Some(shape.line_color),
                stroke_width: border_w,
                ..ShapeStyle::default()
            };
            let bg_id = tree.next_id();
            let bg_node = RenderNode::new(
                bg_id,
                RenderNodeType::Rectangle(RectangleNode::new(2.0, bg_style, None)),
                BoundingBox::new(box_x, box_y, box_w, box_h),
            );
            memo_node.children.insert(0, bg_node);

            // (5) 앵커 밑줄 (본문 텍스트를 가리지 않도록 줄 하단에 fillColor 밴드)
            let underline_w = (anchor.end_x - anchor.x).max(6.0);
            let underline_style = ShapeStyle {
                fill_color: Some(shape.fill_color),
                ..ShapeStyle::default()
            };
            let ul_id = tree.next_id();
            let underline_node = RenderNode::new(
                ul_id,
                RenderNodeType::Rectangle(RectangleNode::new(0.0, underline_style, None)),
                BoundingBox::new(
                    anchor.x,
                    anchor.y + anchor.line_height - ANCHOR_UNDERLINE_H,
                    underline_w,
                    ANCHOR_UNDERLINE_H,
                ),
            );
            tree.root.children.push(underline_node);

            // (6) 연결선: 앵커 밑줄 끝 → 박스 좌측 상단
            let conn_id = tree.next_id();
            let conn_node = RenderNode::new(
                conn_id,
                RenderNodeType::Line(LineNode::new(
                    anchor.x + underline_w,
                    anchor.y + anchor.line_height - ANCHOR_UNDERLINE_H / 2.0,
                    box_x,
                    box_y + MEMO_BOX_PADDING + 4.0,
                    LineStyle {
                        color: shape.line_color,
                        width: 0.75,
                        ..LineStyle::default()
                    },
                )),
                BoundingBox::new(
                    anchor.x + underline_w,
                    (anchor.y + anchor.line_height).min(box_y),
                    (box_x - anchor.x - underline_w).abs(),
                    (box_y - anchor.y).abs().max(1.0),
                ),
            );
            tree.root.children.push(conn_node);

            tree.root.children.push(memo_node);
        }
    }

    /// 메모 박스 상단의 작성자 라벨 한 줄을 배치하고 다음 y 를 반환한다.
    fn layout_memo_author_label(
        &self,
        tree: &mut PageRenderTree,
        memo_node: &mut RenderNode,
        author: &str,
        anchor: &MemoAnchor,
        styles: &ResolvedStyleSet,
        content_area: &LayoutRect,
    ) -> f64 {
        let base_cs_id = anchor
            .paragraphs
            .first()
            .and_then(|p| p.char_shapes.first())
            .map(|cs| cs.char_shape_id as u32)
            .unwrap_or(0);
        let mut style = resolved_to_text_style(styles, base_cs_id, 0);
        style.font_size = (style.font_size * 0.85).max(8.0);
        style.bold = true;

        let label_h = style.font_size * 1.5;
        let baseline = style.font_size * 1.15;
        let line_id = tree.next_id();
        let mut line_node = RenderNode::new(
            line_id,
            RenderNodeType::TextLine(TextLineNode::new(label_h, baseline)),
            BoundingBox::new(content_area.x, 0.0, content_area.width, label_h),
        );
        let run_id = tree.next_id();
        let run_node = RenderNode::new(
            run_id,
            RenderNodeType::TextRun(TextRunNode {
                text: author.to_string(),
                style,
                char_shape_id: None,
                para_shape_id: None,
                section_index: Some(anchor.section_index),
                para_index: Some(usize::MAX - 3000),
                char_start: None,
                cell_context: None,
                is_para_end: false,
                is_line_break_end: false,
                rotation: 0.0,
                is_vertical: false,
                char_overlap: None,
                border_fill_id: 0,
                baseline,
                field_marker: FieldMarkerType::None,
            }),
            BoundingBox::new(content_area.x, 0.0, content_area.width, label_h),
        );
        line_node.children.push(run_node);
        memo_node.children.push(line_node);
        label_h + 2.0
    }
}

/// 노드와 그 자식들의 y 좌표를 dy 만큼 이동한다 (메모 본문 배치용).
/// bbox 외에 자체 좌표를 갖는 노드(Line)도 함께 이동한다.
fn translate_node_y(node: &mut RenderNode, dy: f64) {
    node.bbox.y += dy;
    if let RenderNodeType::Line(line) = &mut node.node_type {
        line.y1 += dy;
        line.y2 += dy;
    }
    for child in &mut node.children {
        translate_node_y(child, dy);
    }
}

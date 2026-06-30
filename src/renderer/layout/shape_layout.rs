//! 도형/글상자/그룹 개체 레이아웃

use super::super::composer::{compose_paragraph, reflow_line_segs, ComposedParagraph};
use super::super::page_layout::LayoutRect;
use super::super::pagination::PageItem;
use super::super::render_tree::*;
use super::super::style_resolver::ResolvedStyleSet;
use super::super::{hwpunit_to_px, px_to_hwpunit, PathCommand, ShapeStyle, TextStyle};
use super::text_measurement::{
    estimate_text_width, is_cjk_char, is_vertical_rotate_char, resolved_to_text_style,
    vertical_substitute_char,
};
use super::utils::{
    drawing_to_line_style, drawing_to_shape_style, extract_shape_transform, find_bin_data,
};
use super::LayoutEngine;
use super::{CellContext, CellPathEntry};
use crate::model::bin_data::BinDataContent;
use crate::model::control::Control;
use crate::model::paragraph::Paragraph;
use crate::model::shape::{CommonObjAttr, DrawingObjAttr, TextBox};
use crate::model::shape::{HorzAlign, HorzRelTo, VertAlign, VertRelTo};
use crate::model::style::{Alignment, FillType};

/// 글상자에 공백이 아닌 실제 텍스트가 한 글자라도 있는지.
fn textbox_has_visible_text(text_box: &TextBox) -> bool {
    text_box
        .paragraphs
        .iter()
        .any(|para| para.text.chars().any(|ch| !ch.is_whitespace()))
}

fn textbox_contains_non_tac_picture(text_box: &TextBox) -> bool {
    text_box.paragraphs.iter().any(|para| {
        para.controls
            .iter()
            .any(|control| matches!(control, Control::Picture(pic) if !pic.common.treat_as_char))
    })
}

/// 평탄화된 HWPX 그룹(matrix group) 자식의 "글상자 보조선"(검정 얇은 SOLID 테두리)은
/// 한컴 실물에서 인쇄되지 않는다(편람 장 표지 "행정업무 운영 개요" 제목/목록 글상자).
/// 오탐 방지를 위해 매우 좁게 한정: 그룹 자식(group_level>0) + 회전/전단 없음 + 검정
/// (color==0) 얇은(0<width<=40 HWPUNIT) SOLID(line_type==1) 테두리 + 캡션 없음 +
/// (a) 채우기 없는 텍스트 전용 글상자 또는 (b) 흰색 단색 마스크 박스.
fn should_suppress_group_child_construction_stroke(drawing: &DrawingObjAttr) -> bool {
    if drawing.caption.is_some() {
        return false;
    }
    let sa = &drawing.shape_attr;
    let has_rotation_or_shear = sa.render_b.abs() > 1e-6 || sa.render_c.abs() > 1e-6;
    if sa.group_level == 0 || has_rotation_or_shear {
        return false;
    }
    let line = &drawing.border_line;
    let line_type = line.attr & 0x3f;
    if line_type != 1 || line.color != 0 || line.width <= 0 || line.width > 40 {
        return false;
    }
    let text_only_box = drawing
        .text_box
        .as_ref()
        .is_some_and(textbox_has_visible_text)
        && drawing.fill.fill_type == FillType::None
        && drawing.fill.gradient.is_none()
        && drawing.fill.image.is_none();
    if text_only_box {
        return true;
    }
    drawing.text_box.is_none()
        && drawing.fill.fill_type == FillType::Solid
        && drawing.fill.gradient.is_none()
        && drawing.fill.image.is_none()
        && drawing
            .fill
            .solid
            .is_some_and(|solid| solid.background_color == 0x00ff_ffff && solid.pattern_type <= 0)
}

fn push_placeholder_render_node(
    tree: &mut PageRenderTree,
    parent: &mut RenderNode,
    bbox: BoundingBox,
    fill_color: u32,
    stroke_color: u32,
    label: String,
) {
    let node_id = tree.next_id();
    let node = RenderNode::new(
        node_id,
        RenderNodeType::Placeholder(crate::renderer::render_tree::PlaceholderNode {
            fill_color,
            stroke_color,
            label,
        }),
        bbox,
    );
    parent.children.push(node);
}

fn push_raw_svg_render_node(
    tree: &mut PageRenderTree,
    parent: &mut RenderNode,
    bbox: BoundingBox,
    svg: String,
) {
    let node_id = tree.next_id();
    let node = RenderNode::new(
        node_id,
        RenderNodeType::RawSvg(crate::renderer::render_tree::RawSvgNode { svg }),
        bbox,
    );
    parent.children.push(node);
}

fn ole_chart_fallback_label(message: impl AsRef<str>, bin_data_id: u32) -> String {
    format!("{} (BinData #{bin_data_id})", message.as_ref())
}

fn measure_composed_text_range_width(
    composed: &ComposedParagraph,
    styles: &ResolvedStyleSet,
    start: usize,
    end: usize,
    space_advance_override: Option<f64>,
) -> f64 {
    if start >= end {
        return 0.0;
    }

    let tab_width = styles
        .para_styles
        .get(composed.para_style_id as usize)
        .map(|s| s.default_tab_width)
        .unwrap_or(0.0);
    let mut width = 0.0;

    for line in &composed.lines {
        let mut run_start = line.char_start;
        for run in &line.runs {
            let run_len = run.text.chars().count();
            let run_end = run_start + run_len;
            let seg_start = start.max(run_start);
            let seg_end = end.min(run_end);

            if seg_start < seg_end {
                let seg_text: String = run
                    .text
                    .chars()
                    .skip(seg_start - run_start)
                    .take(seg_end - seg_start)
                    .collect();
                if let Some(space_advance) = space_advance_override {
                    let space_count = seg_text.chars().filter(|&ch| ch == ' ').count();
                    if space_count > 0 && seg_text.chars().all(|ch| ch == ' ') {
                        width += space_advance * space_count as f64;
                        run_start = run_end;
                        continue;
                    }
                }
                let mut style = resolved_to_text_style(styles, run.char_style_id, run.lang_index);
                style.default_tab_width = tab_width;
                width += estimate_text_width(&seg_text, &style);
            }

            run_start = run_end;
        }
    }

    width
}

fn textbox_tac_space_advance_override(
    para: &Paragraph,
    composed: &ComposedParagraph,
    total_inline_width: f64,
    alignment: Alignment,
    dpi: f64,
) -> Option<f64> {
    if !matches!(alignment, Alignment::Left) || total_inline_width <= 0.0 {
        return None;
    }

    let first_line = composed.lines.first()?;
    let mut space_count = 0usize;
    let mut has_non_space_text = false;
    for run in &first_line.runs {
        for ch in run.text.chars() {
            if ch == ' ' {
                space_count += 1;
            } else {
                has_non_space_text = true;
            }
        }
    }
    if space_count == 0 || has_non_space_text {
        return None;
    }

    let line_width_hu = para
        .line_segs
        .first()
        .map(|seg| seg.segment_width)
        .unwrap_or(first_line.segment_width);
    if line_width_hu <= 0 {
        return None;
    }

    // HWPX 글상자 안의 TAC 그림 사이 공백은 직접 run charPr의 음수 자간으로
    // 압축하지 않고, 문단 스타일(예: 바탕글)로 이미 조판된 lineSeg 폭을 따른다.
    // 순수 공백 + TAC 컨트롤만 있는 라인은 lineSeg.horzsize가 한컴의 공백
    // advance 계약값이므로, 남은 폭을 공백 개수로 나눠 사용한다.
    let line_width = hwpunit_to_px(line_width_hu, dpi);
    let remaining = line_width - total_inline_width;
    if remaining > 0.0 {
        Some(remaining / space_count as f64)
    } else {
        None
    }
}

fn matrix_textbox_lines_need_reflow(para: &Paragraph) -> bool {
    para.controls.is_empty()
        && !para.text.is_empty()
        && !para.text.contains('\n')
        && para.line_segs.len() > 1
}

fn matrix_textbox_lines_overflow_height(para: &Paragraph, available_height: f64, dpi: f64) -> bool {
    para.line_segs.last().is_some_and(|seg| {
        hwpunit_to_px(seg.vertical_pos.saturating_add(seg.line_height), dpi)
            > available_height + 0.5
    })
}

fn should_reflow_matrix_textbox_lines(
    matrix_positioned: bool,
    drawing: &DrawingObjAttr,
    text_box: &TextBox,
    text_direction: u8,
    available_width: f64,
    available_height: f64,
    dpi: f64,
) -> bool {
    if text_direction != 0 || available_width <= 0.0 || available_height <= 0.0 {
        return false;
    }

    if !matrix_positioned {
        return false;
    }

    let sa = &drawing.shape_attr;
    let has_axis_scale =
        (sa.render_sx.abs() - 1.0).abs() > 0.001 || (sa.render_sy.abs() - 1.0).abs() > 0.001;
    let has_rotation_or_shear = sa.render_b.abs() > 1e-6 || sa.render_c.abs() > 1e-6;
    let compressed_group_child = sa.group_level > 0
        && sa.render_sx > 0.0
        && sa.render_sy > 0.0
        && (sa.render_sx < 0.99 || sa.render_sy < 0.99);
    has_axis_scale
        && !has_rotation_or_shear
        && text_box.paragraphs.iter().any(|para| {
            matrix_textbox_lines_need_reflow(para)
                && (compressed_group_child
                    || matrix_textbox_lines_overflow_height(para, available_height, dpi))
        })
}

fn reflow_matrix_textbox_para(
    para: &mut Paragraph,
    available_width: f64,
    _available_height: f64,
    styles: &ResolvedStyleSet,
    dpi: f64,
) {
    if !matrix_textbox_lines_need_reflow(para) {
        return;
    }

    const MATRIX_TEXT_FIT_TOLERANCE_PX: f64 = 3.0;
    let composed = compose_paragraph(para);
    let text_len = para.text.chars().count();
    let full_width = measure_composed_text_range_width(&composed, styles, 0, text_len, None);

    if full_width <= available_width + MATRIX_TEXT_FIT_TOLERANCE_PX {
        if let Some(first_seg) = para.line_segs.first().cloned() {
            let mut line_seg = first_seg;
            line_seg.text_start = 0;
            line_seg.segment_width = px_to_hwpunit(available_width, dpi);
            para.line_segs = vec![line_seg];
            return;
        }
    }

    reflow_line_segs(para, available_width, styles, dpi);
}

impl LayoutEngine {
    pub(crate) fn scan_textbox_overflow(
        &self,
        paragraphs: &[Paragraph],
        shape_items: &[(i32, usize, usize, f64, Alignment)],
    ) -> std::collections::HashMap<(usize, usize), Vec<Paragraph>> {
        use crate::model::shape::ShapeObject;

        // 1단계: 오버플로우 문단 수집 (소스 텍스트박스에서)
        let mut overflow_paras: Vec<(i32, Vec<Paragraph>)> = Vec::new(); // (target_sw, paragraphs)
                                                                         // 빈 텍스트박스 수집 (타겟 후보)
        let mut empty_targets: Vec<(usize, usize, i32)> = Vec::new(); // (para_idx, ctrl_idx, inner_sw)

        for &(_, pi, ci, _, _) in shape_items {
            let para = match paragraphs.get(pi) {
                Some(p) => p,
                None => continue,
            };
            let ctrl = match para.controls.get(ci) {
                Some(c) => c,
                None => continue,
            };
            let drawing = match ctrl {
                Control::Shape(s) => match s.as_ref() {
                    ShapeObject::Rectangle(r) => &r.drawing,
                    _ => continue,
                },
                _ => continue,
            };
            let tb = match &drawing.text_box {
                Some(tb) => tb,
                None => continue,
            };

            // 텍스트가 있는 문단 수
            let has_text = tb.paragraphs.iter().any(|p| !p.text.is_empty());
            if !has_text {
                // 빈 텍스트박스: 첫 문단의 line_seg sw를 inner_sw로 사용하거나 계산
                // 실제로는 오버플로우 문단이 렌더링될 때 sw를 사용
                let inner_sw = tb
                    .paragraphs
                    .first()
                    .and_then(|p| p.line_segs.first())
                    .map(|ls| ls.segment_width)
                    .unwrap_or(0);
                empty_targets.push((pi, ci, inner_sw));
                continue;
            }

            // 오버플로우 감지: 첫 문단의 sw와 다른 sw를 가진 문단 찾기
            let first_sw = tb
                .paragraphs
                .first()
                .and_then(|p| p.line_segs.first())
                .map(|ls| ls.segment_width)
                .unwrap_or(0);
            let mut max_vpos_end: i32 = 0;
            let mut overflow_idx: Option<usize> = None;
            for (tpi, tp) in tb.paragraphs.iter().enumerate() {
                if let Some(first_ls) = tp.line_segs.first() {
                    if tpi > 0
                        && first_ls.segment_width != first_sw
                        && first_ls.vertical_pos < max_vpos_end
                    {
                        overflow_idx = Some(tpi);
                        break;
                    }
                    if let Some(last_ls) = tp.line_segs.last() {
                        let end = last_ls.vertical_pos + last_ls.line_height;
                        if end > max_vpos_end {
                            max_vpos_end = end;
                        }
                    }
                }
            }
            if let Some(oi) = overflow_idx {
                let target_sw = tb.paragraphs[oi]
                    .line_segs
                    .first()
                    .map(|ls| ls.segment_width)
                    .unwrap_or(0);
                let overflow: Vec<Paragraph> = tb.paragraphs[oi..].to_vec();
                overflow_paras.push((target_sw, overflow));
            }
        }

        // 2단계: 오버플로우 문단을 빈 텍스트박스에 매핑 (sw 매칭)
        let mut result = std::collections::HashMap::new();
        for (target_sw, paras) in overflow_paras {
            // sw가 가장 가까운 빈 텍스트박스 찾기
            let best = empty_targets
                .iter()
                .enumerate()
                .min_by_key(|(_, (_, _, esw))| (target_sw - *esw).abs());
            if let Some((idx, &(pi, ci, _))) = best {
                result.insert((pi, ci), paras);
                empty_targets.remove(idx);
            }
        }
        result
    }

    /// 도형(Shape) 레이아웃 - 시각 요소 + 글상자(TextBox) 포함
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn layout_shape(
        &self,
        tree: &mut PageRenderTree,
        parent: &mut RenderNode,
        paragraphs: &[Paragraph],
        para_index: usize,
        control_index: usize,
        section_index: usize,
        styles: &ResolvedStyleSet,
        col_area: &LayoutRect,
        body_area: &LayoutRect,
        paper_area: &LayoutRect,
        para_y: f64,
        alignment: Alignment,
        bin_data_content: &[BinDataContent],
        overflow_map: &std::collections::HashMap<(usize, usize), Vec<Paragraph>>,
        clamp_negative_para_offset: bool,
    ) {
        use crate::model::shape::ShapeObject;

        let para = match paragraphs.get(para_index) {
            Some(p) => p,
            None => return,
        };

        let ctrl = match para.controls.get(control_index) {
            Some(c) => c,
            None => return,
        };

        // 수식 컨트롤 처리
        if let Control::Equation(eq) = ctrl {
            // 인라인 좌표가 등록되어 있으면 paragraph_layout에서 이미 렌더링됨 → 스킵
            let inline_pos =
                tree.get_inline_shape_position(section_index, para_index, control_index, None);
            if inline_pos.is_some() {
                return;
            }

            // 인라인 좌표 없으면 기존 방식 (정렬 기반 단독 배치)
            let eq_w = hwpunit_to_px(eq.common.width as i32, self.dpi);
            let eq_h = hwpunit_to_px(eq.common.height as i32, self.dpi);
            let eq_x = match alignment {
                Alignment::Center | Alignment::Distribute => {
                    col_area.x + (col_area.width - eq_w).max(0.0) / 2.0
                }
                Alignment::Right => col_area.x + (col_area.width - eq_w).max(0.0),
                _ => col_area.x,
            };
            let eq_y = para_y;

            // 수식 스크립트 → AST → 레이아웃 → SVG 조각
            let tokens = super::super::equation::tokenizer::tokenize(&eq.script);
            let ast = super::super::equation::parser::EqParser::new(tokens).parse();
            let font_size_px = hwpunit_to_px(eq.font_size as i32, self.dpi);
            let layout_box =
                super::super::equation::layout::EqLayout::new(font_size_px).layout(&ast);
            let color_str = super::super::equation::svg_render::eq_color_to_svg(eq.color);
            let svg_content = super::super::equation::svg_render::render_equation_svg(
                &layout_box,
                &color_str,
                font_size_px,
            );

            let eq_node = RenderNode::new(
                tree.next_id(),
                RenderNodeType::Equation(EquationNode {
                    svg_content,
                    layout_box,
                    color_str,
                    color: eq.color,
                    font_size: font_size_px,
                    section_index: Some(section_index),
                    para_index: Some(para_index),
                    control_index: Some(control_index),
                    cell_index: None,
                    cell_para_index: None,
                    note_ref: None,
                }),
                BoundingBox::new(eq_x, eq_y, eq_w, eq_h),
            );
            parent.children.push(eq_node);
            return;
        }

        let shape = match ctrl {
            Control::Shape(s) => s.as_ref(),
            _ => return,
        };

        let common = shape.common();
        let adjusted_common;
        let common_for_position = if clamp_negative_para_offset
            && !common.treat_as_char
            && matches!(common.vert_rel_to, crate::model::shape::VertRelTo::Para)
            && matches!(
                common.vert_align,
                crate::model::shape::VertAlign::Top | crate::model::shape::VertAlign::Inside
            )
            && (common.vertical_offset as i32) < 0
        {
            adjusted_common = {
                let mut common = common.clone();
                common.vertical_offset = 0;
                common
            };
            &adjusted_common
        } else {
            common
        };

        let (mut shape_w, mut shape_h) =
            self.resolve_object_size(common, col_area, body_area, paper_area);

        if shape
            .drawing()
            .and_then(|drawing| drawing.text_box.as_ref())
            .is_some_and(textbox_contains_non_tac_picture)
        {
            let sa = shape.shape_attr();
            let cur_w = hwpunit_to_px(sa.current_width as i32, self.dpi);
            let cur_h = hwpunit_to_px(sa.current_height as i32, self.dpi);
            if cur_w > shape_w && cur_w > 0.0 {
                shape_w = cur_w;
            }
            if cur_h > shape_h && cur_h > 0.0 {
                shape_h = cur_h;
            }
        }

        // 문단 여백 반영: Para 기준 위치 지정 시 문단의 왼쪽/오른쪽 여백 고려
        let composed_para = paragraphs.get(para_index).and_then(|_| {
            // composed 데이터가 없으므로 paragraphs에서 직접 para_shape_id 사용
            let pid = para.para_shape_id as usize;
            styles.para_styles.get(pid)
        });
        let para_margin_left = composed_para.map(|ps| ps.margin_left).unwrap_or(0.0);
        let para_margin_right = composed_para.map(|ps| ps.margin_right).unwrap_or(0.0);

        // 인라인 Shape: paragraph_layout에서 계산된 좌표가 있으면 사용
        let inline_pos = if common.treat_as_char {
            tree.get_inline_shape_position(section_index, para_index, control_index, None)
        } else {
            None
        };
        // [Issue #476] treat_as_char Shape는 paragraph_layout이 inline_pos 등록 후
        // 본 함수가 그려야 한다. inline_pos 가 없는 경우는 paginator 가 PageItem::Shape 를
        // 잘못된 페이지(박스가 속한 line이 라우팅되지 않은 페이지)에 등록한 결과이며,
        // compute_object_position fallback 으로 그리면 절대 좌표(예: 문단 오프셋=0,0)
        // 기준의 잘못된 위치에 박스가 출현한다 (= 다른 paragraph 영역에 침범).
        // 본질 수정 전까지(paginator A 단계) fallback 그리기를 차단한다.
        if common.treat_as_char && inline_pos.is_none() {
            if std::env::var("RHWP_DEBUG_LAYOUT").is_ok() {
                eprintln!(
                    "[#476 skip] inline Shape without inline_pos: sec={} para={} ci={}",
                    section_index, para_index, control_index
                );
            }
            return;
        }

        // 통합 좌표 계산 (layout_body_picture와 동일 로직)
        let shape_container = LayoutRect {
            x: col_area.x + para_margin_left,
            y: para_y,
            width: col_area.width - para_margin_left - para_margin_right,
            height: col_area.height - (para_y - col_area.y).max(0.0),
        };
        let (shape_x, shape_y) = if let Some((ix, iy)) = inline_pos {
            (ix, iy)
        } else {
            self.compute_object_position(
                common_for_position,
                shape_w,
                shape_h,
                &shape_container,
                col_area,
                body_area,
                paper_area,
                para_y,
                alignment,
            )
        };

        // 캡션 높이 및 간격 계산
        let drawing = shape.drawing();
        let caption_opt = drawing.and_then(|d| d.caption.clone()).or_else(|| {
            if let ShapeObject::Group(g) = shape {
                g.caption.clone()
            } else {
                None
            }
        });
        let caption = caption_opt.as_ref();
        let caption_height = self.calculate_caption_height(&caption_opt, styles);
        let caption_spacing = caption
            .map(|c| hwpunit_to_px(c.spacing as i32, self.dpi))
            .unwrap_or(0.0);

        use crate::model::shape::CaptionDirection;

        // 캡션 방향에 따라 도형 위치 오프셋 계산
        let (caption_top_offset, caption_left_offset) = if let Some(c) = caption {
            match c.direction {
                CaptionDirection::Top => (caption_height + caption_spacing, 0.0),
                CaptionDirection::Left => {
                    let cw = hwpunit_to_px(c.width as i32, self.dpi);
                    (0.0, cw + caption_spacing)
                }
                _ => (0.0, 0.0),
            }
        } else {
            (0.0, 0.0)
        };
        let adjusted_shape_x = shape_x + caption_left_offset;
        let adjusted_shape_y = shape_y + caption_top_offset;

        // 도형 타입별 렌더 노드 생성
        self.layout_shape_object(
            tree,
            parent,
            shape,
            adjusted_shape_x,
            adjusted_shape_y,
            shape_w,
            shape_h,
            section_index,
            para_index,
            control_index,
            styles,
            bin_data_content,
            overflow_map,
            &[],
            None, // [Task #1138] 본문 도형 — 셀 정보 없음
            false,
        );

        // 캡션 렌더링
        if let Some(caption) = caption {
            use crate::model::shape::CaptionVertAlign;
            let (cap_x, cap_w, cap_y) = match caption.direction {
                CaptionDirection::Top => (adjusted_shape_x, shape_w, shape_y),
                CaptionDirection::Bottom => (
                    adjusted_shape_x,
                    shape_w,
                    adjusted_shape_y + shape_h + caption_spacing,
                ),
                CaptionDirection::Left | CaptionDirection::Right => {
                    let cw = hwpunit_to_px(caption.width as i32, self.dpi);
                    let cx = if caption.direction == CaptionDirection::Left {
                        shape_x
                    } else {
                        adjusted_shape_x + shape_w + caption_spacing
                    };
                    // Left/Right 캡션의 세로 정렬
                    let cy = match caption.vert_align {
                        CaptionVertAlign::Top => adjusted_shape_y,
                        CaptionVertAlign::Center => {
                            adjusted_shape_y + (shape_h - caption_height).max(0.0) / 2.0
                        }
                        CaptionVertAlign::Bottom => {
                            adjusted_shape_y + (shape_h - caption_height).max(0.0)
                        }
                    };
                    (cx, cw, cy)
                }
            };
            self.layout_caption(
                tree,
                parent,
                caption,
                styles,
                col_area,
                cap_x,
                cap_w,
                cap_y,
                &mut self.auto_counter.borrow_mut(),
                bin_data_content,
                None,
            );
        }
    }

    /// 회전이 있는 그룹 자식 도형을 전체 아핀 변환으로 렌더링한다.
    /// HWPX/HWP renderingInfo is already composed within the top-level group
    /// coordinate system. Apply only that top-level group anchor; nested group
    /// bboxes are structural and must not be applied as another translation.
    /// sa: 자식의 ShapeComponentAttr (아핀 행렬 포함)
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn layout_group_child_affine(
        &self,
        tree: &mut PageRenderTree,
        parent: &mut RenderNode,
        child: &crate::model::shape::ShapeObject,
        group_origin_x: f64,
        group_origin_y: f64,
        sa: &crate::model::shape::ShapeComponentAttr,
        section_index: usize,
        para_index: usize,
        control_index: usize,
        styles: &ResolvedStyleSet,
        bin_data_content: &[BinDataContent],
        parent_cell_path: &[CellPathEntry],
    ) {
        use crate::model::shape::ShapeObject;

        // 아핀 변환: (x', y') = (a*x + b*y + tx, c*x + d*y + ty)
        // x, y는 원본 좌표(HWP units), 결과는 그룹 로컬 좌표(HWP units)
        let a = sa.render_sx;
        let b = sa.render_b;
        let tx = sa.render_tx;
        let c = sa.render_c;
        let d = sa.render_sy;
        let ty = sa.render_ty;

        // HWP 좌표를 아핀 변환 후 페이지 절대 좌표(px)로 변환하는 헬퍼.
        let transform_pt = |ox: f64, oy: f64| -> (f64, f64) {
            let gx = a * ox + b * oy + tx;
            let gy = c * ox + d * oy + ty;
            (
                group_origin_x + hwpunit_to_px(gx as i32, self.dpi),
                group_origin_y + hwpunit_to_px(gy as i32, self.dpi),
            )
        };

        match child {
            ShapeObject::Polygon(poly) => {
                let (style, gradient) = drawing_to_shape_style(&poly.drawing);
                let mut commands = Vec::new();
                let mut min_x = f64::MAX;
                let mut min_y = f64::MAX;
                let mut max_x = f64::MIN;
                let mut max_y = f64::MIN;
                for (i, pt) in poly.points.iter().enumerate() {
                    let (px, py) = transform_pt(pt.x as f64, pt.y as f64);
                    min_x = min_x.min(px);
                    min_y = min_y.min(py);
                    max_x = max_x.max(px);
                    max_y = max_y.max(py);
                    if i == 0 {
                        commands.push(PathCommand::MoveTo(px, py));
                    } else {
                        commands.push(PathCommand::LineTo(px, py));
                    }
                }
                if poly.points.len() >= 3 {
                    let first = &poly.points[0];
                    let last = &poly.points[poly.points.len() - 1];
                    if first.x == last.x && first.y == last.y {
                        commands.push(PathCommand::ClosePath);
                    }
                }
                let bbox_w = (max_x - min_x).max(0.0);
                let bbox_h = (max_y - min_y).max(0.0);
                let node_id = tree.next_id();
                let node = RenderNode::new(
                    node_id,
                    RenderNodeType::Path(PathNode::new(commands, style, gradient)),
                    BoundingBox::new(min_x, min_y, bbox_w, bbox_h),
                );
                parent.children.push(node);
            }
            ShapeObject::Line(line) => {
                let (style, gradient) = drawing_to_shape_style(&line.drawing);
                let (x1, y1) = transform_pt(line.start.x as f64, line.start.y as f64);
                let (x2, y2) = transform_pt(line.end.x as f64, line.end.y as f64);
                let min_x = x1.min(x2);
                let min_y = y1.min(y2);
                let commands = vec![PathCommand::MoveTo(x1, y1), PathCommand::LineTo(x2, y2)];
                let node_id = tree.next_id();
                let node = RenderNode::new(
                    node_id,
                    RenderNodeType::Path(PathNode::new(commands, style, gradient)),
                    BoundingBox::new(min_x, min_y, (x2 - x1).abs(), (y2 - y1).abs()),
                );
                parent.children.push(node);
            }
            ShapeObject::Curve(curve) => {
                let (style, gradient) = drawing_to_shape_style(&curve.drawing);
                let mut commands = Vec::new();
                let mut min_x = f64::MAX;
                let mut min_y = f64::MAX;
                let mut max_x = f64::MIN;
                let mut max_y = f64::MIN;
                let points = &curve.points;
                if !points.is_empty() {
                    let (px, py) = transform_pt(points[0].x as f64, points[0].y as f64);
                    commands.push(PathCommand::MoveTo(px, py));
                    min_x = min_x.min(px);
                    min_y = min_y.min(py);
                    max_x = max_x.max(px);
                    max_y = max_y.max(py);
                    let mut i = 1;
                    while i + 2 < points.len() {
                        let (cx1, cy1) = transform_pt(points[i].x as f64, points[i].y as f64);
                        let (cx2, cy2) =
                            transform_pt(points[i + 1].x as f64, points[i + 1].y as f64);
                        let (ex, ey) = transform_pt(points[i + 2].x as f64, points[i + 2].y as f64);
                        commands.push(PathCommand::CurveTo(cx1, cy1, cx2, cy2, ex, ey));
                        for &(px, py) in &[(cx1, cy1), (cx2, cy2), (ex, ey)] {
                            min_x = min_x.min(px);
                            min_y = min_y.min(py);
                            max_x = max_x.max(px);
                            max_y = max_y.max(py);
                        }
                        i += 3;
                    }
                }
                let bbox_w = (max_x - min_x).max(0.0);
                let bbox_h = (max_y - min_y).max(0.0);
                let node_id = tree.next_id();
                let node = RenderNode::new(
                    node_id,
                    RenderNodeType::Path(PathNode::new(commands, style, gradient)),
                    BoundingBox::new(min_x, min_y, bbox_w, bbox_h),
                );
                parent.children.push(node);
            }
            ShapeObject::Rectangle(rect) => {
                // 회전된 사각형: 4꼭짓점을 아핀 변환하여 다각형으로 렌더링
                let ow = sa.original_width as f64;
                let oh = sa.original_height as f64;
                let corners = [(0.0, 0.0), (ow, 0.0), (ow, oh), (0.0, oh)];
                let (style, gradient) = drawing_to_shape_style(&rect.drawing);
                let mut commands = Vec::new();
                let mut min_x = f64::MAX;
                let mut min_y = f64::MAX;
                let mut max_x = f64::MIN;
                let mut max_y = f64::MIN;
                for (i, &(ox, oy)) in corners.iter().enumerate() {
                    let (px, py) = transform_pt(ox, oy);
                    min_x = min_x.min(px);
                    min_y = min_y.min(py);
                    max_x = max_x.max(px);
                    max_y = max_y.max(py);
                    if i == 0 {
                        commands.push(PathCommand::MoveTo(px, py));
                    } else {
                        commands.push(PathCommand::LineTo(px, py));
                    }
                }
                commands.push(PathCommand::ClosePath);
                let node_id = tree.next_id();
                let node = RenderNode::new(
                    node_id,
                    RenderNodeType::Path(PathNode::new(commands, style, gradient)),
                    BoundingBox::new(
                        min_x,
                        min_y,
                        (max_x - min_x).max(0.0),
                        (max_y - min_y).max(0.0),
                    ),
                );
                parent.children.push(node);
            }
            _ => {
                // 그 외 도형(타원, 호, 그림 등): 기본 AABB 방식 폴백
                let (p0x, p0y) = transform_pt(0.0, 0.0);
                let (p1x, p1y) = transform_pt(sa.original_width as f64, sa.original_height as f64);
                let child_x = p0x.min(p1x);
                let child_y = p0y.min(p1y);
                let child_w = (p1x - p0x).abs();
                let child_h = (p1y - p0y).abs();
                let empty_map = std::collections::HashMap::new();
                self.layout_shape_object(
                    tree,
                    parent,
                    child,
                    child_x,
                    child_y,
                    child_w,
                    child_h,
                    section_index,
                    para_index,
                    control_index,
                    styles,
                    bin_data_content,
                    &empty_map,
                    parent_cell_path,
                    None, // [Task #1138] TODO: layout_group_child_affine 에 cell ctx propagate (별도 후속)
                    true,
                );
            }
        }
    }

    /// 개별 ShapeObject를 렌더 노드로 변환한다.
    /// base_x/y는 도형의 절대 좌표, w/h는 도형 크기.
    /// parent_cell_path: 중첩 글상자/표에서 상위 경로 (최상위 도형이면 빈 슬라이스)
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn layout_shape_object(
        &self,
        tree: &mut PageRenderTree,
        parent: &mut RenderNode,
        shape: &crate::model::shape::ShapeObject,
        base_x: f64,
        base_y: f64,
        w: f64,
        h: f64,
        section_index: usize,
        para_index: usize,
        control_index: usize,
        styles: &ResolvedStyleSet,
        bin_data_content: &[BinDataContent],
        overflow_map: &std::collections::HashMap<(usize, usize), Vec<Paragraph>>,
        parent_cell_path: &[CellPathEntry],
        // [Task #1138] 표 셀 내 도형인 경우: (cell_idx, cell_para_idx, outer_table_ctrl_idx)
        table_cell_ref: Option<(usize, usize, usize)>,
        // 그룹 자식은 renderingInfo 행렬로 이미 위치/대칭이 반영되어 있으므로
        // ShapeComponentAttr의 flip/rotation을 다시 SVG transform으로 적용하지 않는다.
        matrix_positioned: bool,
    ) {
        self.layout_shape_object_with_group_origin(
            tree,
            parent,
            shape,
            base_x,
            base_y,
            w,
            h,
            section_index,
            para_index,
            control_index,
            styles,
            bin_data_content,
            overflow_map,
            parent_cell_path,
            table_cell_ref,
            matrix_positioned,
            None,
        );
    }

    #[allow(clippy::too_many_arguments)]
    fn layout_shape_object_with_group_origin(
        &self,
        tree: &mut PageRenderTree,
        parent: &mut RenderNode,
        shape: &crate::model::shape::ShapeObject,
        base_x: f64,
        base_y: f64,
        w: f64,
        h: f64,
        section_index: usize,
        para_index: usize,
        control_index: usize,
        styles: &ResolvedStyleSet,
        bin_data_content: &[BinDataContent],
        overflow_map: &std::collections::HashMap<(usize, usize), Vec<Paragraph>>,
        parent_cell_path: &[CellPathEntry],
        // [Task #1138] 표 셀 내 도형인 경우: (cell_idx, cell_para_idx, outer_table_ctrl_idx)
        table_cell_ref: Option<(usize, usize, usize)>,
        // 그룹 자식은 renderingInfo 행렬로 이미 위치/대칭이 반영되어 있으므로
        // ShapeComponentAttr의 flip/rotation을 다시 SVG transform으로 적용하지 않는다.
        matrix_positioned: bool,
        inherited_group_origin: Option<(f64, f64)>,
    ) {
        use crate::model::shape::ShapeObject;

        // 공통: 회전/대칭 정보 추출
        let transform = if matrix_positioned {
            ShapeTransform::default()
        } else {
            extract_shape_transform(shape.shape_attr())
        };

        // [Task #1138] 표 셀 내 도형 식별을 위한 cell 정보 추출 (helper)
        let cell_index = table_cell_ref.map(|(ci, _, _)| ci);
        let cell_para_index = table_cell_ref.map(|(_, cpi, _)| cpi);
        let outer_table_control_index = table_cell_ref.map(|(_, _, otci)| otci);

        // 회전/대칭이 있으면 current 크기로 중앙 배치
        // 그렇지 않으면 호출자가 전달한 w, h를 그대로 사용
        // (그룹 자식은 호출자가 render_sx/sy로 스케일 적용 후 w,h 전달)
        let (render_x, render_y, render_w, render_h) = if transform.has_transform() {
            let sa = shape.shape_attr();
            let cur_w = hwpunit_to_px(sa.current_width as i32, self.dpi);
            let cur_h = hwpunit_to_px(sa.current_height as i32, self.dpi);
            if cur_w > 0.0 && cur_h > 0.0 {
                let cx = base_x + w / 2.0;
                let cy = base_y + h / 2.0;
                (cx - cur_w / 2.0, cy - cur_h / 2.0, cur_w, cur_h)
            } else {
                (base_x, base_y, w, h)
            }
        } else {
            (base_x, base_y, w, h)
        };

        match shape {
            ShapeObject::Rectangle(rect) => {
                let (mut style, gradient) = drawing_to_shape_style(&rect.drawing);
                // 평탄화된 그룹 자식 글상자의 비인쇄 보조선(검정 얇은 SOLID)을 억제한다.
                if should_suppress_group_child_construction_stroke(&rect.drawing) {
                    style.stroke_color = None;
                    style.stroke_width = 0.0;
                }
                let round_px = if rect.round_rate > 0 {
                    (rect.round_rate as f64 / 100.0) * render_w.min(render_h) / 2.0
                } else {
                    0.0
                };
                let node_id = tree.next_id();
                let mut node = RenderNode::new(
                    node_id,
                    RenderNodeType::Rectangle(RectangleNode {
                        section_index: Some(section_index),
                        para_index: Some(para_index),
                        control_index: Some(control_index),
                        transform,
                        cell_index,
                        cell_para_index,
                        outer_table_control_index,
                        ..RectangleNode::new(round_px, style, gradient)
                    }),
                    BoundingBox::new(render_x, render_y, render_w, render_h),
                );
                // 이미지 채우기가 있으면 자식으로 이미지 노드 추가
                self.add_image_fill_node(
                    tree,
                    &mut node,
                    &rect.drawing,
                    render_x,
                    render_y,
                    render_w,
                    render_h,
                    bin_data_content,
                );
                // TextBox가 있으면 자식으로 텍스트 레이아웃
                self.layout_textbox_content(
                    tree,
                    &mut node,
                    &rect.drawing,
                    render_x,
                    render_y,
                    render_w,
                    render_h,
                    section_index,
                    para_index,
                    control_index,
                    styles,
                    bin_data_content,
                    overflow_map,
                    parent_cell_path,
                    shape.common().treat_as_char,
                    matrix_positioned,
                );
                parent.children.push(node);
            }
            ShapeObject::Line(line) => {
                let sa = &line.drawing.shape_attr;
                let sx = if sa.original_width > 0 {
                    render_w / hwpunit_to_px(sa.original_width as i32, self.dpi)
                } else {
                    1.0
                };
                let sy = if sa.original_height > 0 {
                    render_h / hwpunit_to_px(sa.original_height as i32, self.dpi)
                } else {
                    1.0
                };

                // 연결선: 제어점이 있으면 Path로, 없으면 Line으로 렌더링
                if let Some(ref conn) = line.connector {
                    if !conn.control_points.is_empty() {
                        let mut line_style = drawing_to_line_style(&line.drawing);
                        // 연결선 화살표: LinkLineType → ArrowStyle
                        use crate::model::shape::LinkLineType;
                        match conn.link_type {
                            LinkLineType::StraightOneWay
                            | LinkLineType::StrokeOneWay
                            | LinkLineType::ArcOneWay => {
                                line_style.end_arrow = super::super::ArrowStyle::Arrow;
                                line_style.end_arrow_size = 4; // 중간 크기
                            }
                            LinkLineType::StraightBoth
                            | LinkLineType::StrokeBoth
                            | LinkLineType::ArcBoth => {
                                line_style.start_arrow = super::super::ArrowStyle::Arrow;
                                line_style.start_arrow_size = 4;
                                line_style.end_arrow = super::super::ArrowStyle::Arrow;
                                line_style.end_arrow_size = 4;
                            }
                            _ => {}
                        }
                        // 제어점으로 경로 생성
                        let mut commands = Vec::new();
                        let conn_x1 = render_x + hwpunit_to_px(line.start.x, self.dpi) * sx;
                        let conn_y1 = render_y + hwpunit_to_px(line.start.y, self.dpi) * sy;
                        let conn_x2 = render_x + hwpunit_to_px(line.end.x, self.dpi) * sx;
                        let conn_y2 = render_y + hwpunit_to_px(line.end.y, self.dpi) * sy;
                        let connector_point_xy =
                            |cp: &crate::model::shape::ConnectorControlPoint| {
                                (
                                    render_x + hwpunit_to_px(cp.x, self.dpi) * sx,
                                    render_y + hwpunit_to_px(cp.y, self.dpi) * sy,
                                )
                            };
                        let first_control_is_start = conn
                            .control_points
                            .first()
                            .map(|cp| cp.point_type == 3)
                            .unwrap_or(false);
                        let last_control_is_end = conn
                            .control_points
                            .last()
                            .map(|cp| cp.point_type == 26)
                            .unwrap_or(false);
                        let (path_start_x, path_start_y) = if first_control_is_start {
                            connector_point_xy(&conn.control_points[0])
                        } else {
                            (conn_x1, conn_y1)
                        };
                        let mut path_end_x = conn_x2;
                        let mut path_end_y = conn_y2;
                        commands.push(PathCommand::MoveTo(path_start_x, path_start_y));

                        if conn.link_type.is_arc() {
                            // 곡선 연결선: 제어점(type=2)을 bezier 제어점으로, 나머지는 앵커로 사용
                            let cps = &conn.control_points;
                            let end_x = conn_x2;
                            let end_y = conn_y2;
                            // type=2인 제어점만 추출
                            let ctrl_pts: Vec<(f64, f64)> = cps
                                .iter()
                                .filter(|cp| cp.point_type == 2)
                                .map(|cp| {
                                    (
                                        render_x + hwpunit_to_px(cp.x, self.dpi) * sx,
                                        render_y + hwpunit_to_px(cp.y, self.dpi) * sy,
                                    )
                                })
                                .collect();
                            match ctrl_pts.len() {
                                0 => {
                                    // 제어점 없음 → 직선
                                    commands.push(PathCommand::LineTo(end_x, end_y));
                                }
                                1 => {
                                    // 제어점 1개 → quadratic bezier (cubic으로 변환)
                                    let (qx, qy) = ctrl_pts[0];
                                    let (sx0, sy0) = match commands.last() {
                                        Some(PathCommand::MoveTo(x, y)) => (*x, *y),
                                        _ => (render_x, render_y),
                                    };
                                    // Q→C 변환: C = (S + 2*Q)/3, (2*Q + E)/3, E
                                    let cx1 = (sx0 + 2.0 * qx) / 3.0;
                                    let cy1 = (sy0 + 2.0 * qy) / 3.0;
                                    let cx2 = (2.0 * qx + end_x) / 3.0;
                                    let cy2 = (2.0 * qy + end_y) / 3.0;
                                    commands.push(PathCommand::CurveTo(
                                        cx1, cy1, cx2, cy2, end_x, end_y,
                                    ));
                                }
                                2 => {
                                    // 제어점 2개 → cubic bezier
                                    let (cx1, cy1) = ctrl_pts[0];
                                    let (cx2, cy2) = ctrl_pts[1];
                                    commands.push(PathCommand::CurveTo(
                                        cx1, cy1, cx2, cy2, end_x, end_y,
                                    ));
                                }
                                _ => {
                                    // 3개 이상 → 여러 cubic bezier 세그먼트
                                    let mut i = 0;
                                    while i + 1 < ctrl_pts.len() {
                                        let (cx1, cy1) = ctrl_pts[i];
                                        let (cx2, cy2) = ctrl_pts[i + 1];
                                        let (ex, ey) = if i + 2 < ctrl_pts.len() {
                                            (
                                                (cx2 + ctrl_pts[i + 2].0) / 2.0,
                                                (cy2 + ctrl_pts[i + 2].1) / 2.0,
                                            )
                                        } else {
                                            (end_x, end_y)
                                        };
                                        commands
                                            .push(PathCommand::CurveTo(cx1, cy1, cx2, cy2, ex, ey));
                                        i += 2;
                                    }
                                }
                            }
                        } else {
                            for cp in conn.control_points.iter().skip(if first_control_is_start {
                                1
                            } else {
                                0
                            }) {
                                let (cpx, cpy) = connector_point_xy(cp);
                                commands.push(PathCommand::LineTo(cpx, cpy));
                                path_end_x = cpx;
                                path_end_y = cpy;
                            }
                            if !last_control_is_end {
                                commands.push(PathCommand::LineTo(conn_x2, conn_y2));
                                path_end_x = conn_x2;
                                path_end_y = conn_y2;
                            }
                        }

                        let style = ShapeStyle {
                            stroke_color: Some(line_style.color),
                            stroke_width: line_style.width,
                            stroke_dash: line_style.dash.clone(),
                            fill_color: None,
                            ..Default::default()
                        };
                        let node_id = tree.next_id();
                        let mut path_node = PathNode::new(commands, style, None);
                        path_node.section_index = Some(section_index);
                        path_node.para_index = Some(para_index);
                        path_node.control_index = Some(control_index);
                        path_node.transform = transform;
                        path_node.cell_index = cell_index;
                        path_node.cell_para_index = cell_para_index;
                        path_node.outer_table_control_index = outer_table_control_index;
                        // 연결선: 시작/끝 좌표 (선 선택 방식용) + 화살표
                        path_node.connector_endpoints =
                            Some((path_start_x, path_start_y, path_end_x, path_end_y));
                        if line_style.start_arrow != super::super::ArrowStyle::None
                            || line_style.end_arrow != super::super::ArrowStyle::None
                        {
                            path_node.line_style = Some(line_style);
                        }
                        let node = RenderNode::new(
                            node_id,
                            RenderNodeType::Path(path_node),
                            BoundingBox::new(render_x, render_y, render_w, render_h),
                        );
                        parent.children.push(node);
                    } else {
                        // 제어점 없는 연결선 → 직선으로 렌더링
                        let mut line_style = drawing_to_line_style(&line.drawing);
                        use crate::model::shape::LinkLineType;
                        match conn.link_type {
                            LinkLineType::StraightOneWay
                            | LinkLineType::StrokeOneWay
                            | LinkLineType::ArcOneWay => {
                                line_style.end_arrow = super::super::ArrowStyle::Arrow;
                                line_style.end_arrow_size = 4;
                            }
                            LinkLineType::StraightBoth
                            | LinkLineType::StrokeBoth
                            | LinkLineType::ArcBoth => {
                                line_style.start_arrow = super::super::ArrowStyle::Arrow;
                                line_style.start_arrow_size = 4;
                                line_style.end_arrow = super::super::ArrowStyle::Arrow;
                                line_style.end_arrow_size = 4;
                            }
                            _ => {}
                        }
                        let x1 = render_x + hwpunit_to_px(line.start.x, self.dpi) * sx;
                        let y1 = render_y + hwpunit_to_px(line.start.y, self.dpi) * sy;
                        let x2 = render_x + hwpunit_to_px(line.end.x, self.dpi) * sx;
                        let y2 = render_y + hwpunit_to_px(line.end.y, self.dpi) * sy;
                        let node_id = tree.next_id();
                        let mut line_node = LineNode::new(x1, y1, x2, y2, line_style);
                        line_node.section_index = Some(section_index);
                        line_node.para_index = Some(para_index);
                        line_node.control_index = Some(control_index);
                        line_node.transform = transform;
                        line_node.cell_index = cell_index;
                        line_node.cell_para_index = cell_para_index;
                        line_node.outer_table_control_index = outer_table_control_index;
                        let node = RenderNode::new(
                            node_id,
                            RenderNodeType::Line(line_node),
                            BoundingBox::new(render_x, render_y, render_w, render_h),
                        );
                        parent.children.push(node);
                    }
                } else {
                    // 일반 직선
                    let line_style = drawing_to_line_style(&line.drawing);
                    let x1 = render_x + hwpunit_to_px(line.start.x, self.dpi) * sx;
                    let y1 = render_y + hwpunit_to_px(line.start.y, self.dpi) * sy;
                    let x2 = render_x + hwpunit_to_px(line.end.x, self.dpi) * sx;
                    let y2 = render_y + hwpunit_to_px(line.end.y, self.dpi) * sy;
                    let node_id = tree.next_id();
                    let mut line_node = LineNode::new(x1, y1, x2, y2, line_style);
                    line_node.section_index = Some(section_index);
                    line_node.para_index = Some(para_index);
                    line_node.control_index = Some(control_index);
                    line_node.transform = transform;
                    line_node.cell_index = cell_index;
                    line_node.cell_para_index = cell_para_index;
                    line_node.outer_table_control_index = outer_table_control_index;
                    let node = RenderNode::new(
                        node_id,
                        RenderNodeType::Line(line_node),
                        BoundingBox::new(render_x, render_y, render_w, render_h),
                    );
                    parent.children.push(node);
                }
            }
            ShapeObject::Ellipse(ellipse) => {
                let (style, gradient) = drawing_to_shape_style(&ellipse.drawing);
                let node_id = tree.next_id();
                let mut ell_node = EllipseNode::new(style, gradient);
                ell_node.section_index = Some(section_index);
                ell_node.para_index = Some(para_index);
                ell_node.control_index = Some(control_index);
                ell_node.transform = transform;
                ell_node.cell_index = cell_index;
                ell_node.cell_para_index = cell_para_index;
                ell_node.outer_table_control_index = outer_table_control_index;
                let mut node = RenderNode::new(
                    node_id,
                    RenderNodeType::Ellipse(ell_node),
                    BoundingBox::new(render_x, render_y, render_w, render_h),
                );
                self.add_image_fill_node(
                    tree,
                    &mut node,
                    &ellipse.drawing,
                    render_x,
                    render_y,
                    render_w,
                    render_h,
                    bin_data_content,
                );
                let empty_map = std::collections::HashMap::new();
                self.layout_textbox_content(
                    tree,
                    &mut node,
                    &ellipse.drawing,
                    render_x,
                    render_y,
                    render_w,
                    render_h,
                    section_index,
                    para_index,
                    control_index,
                    styles,
                    bin_data_content,
                    &empty_map,
                    parent_cell_path,
                    shape.common().treat_as_char,
                    matrix_positioned,
                );
                parent.children.push(node);
            }
            ShapeObject::Arc(arc) => {
                let (style, gradient) = drawing_to_shape_style(&arc.drawing);
                // 호(Arc) 좌표 계산: center, axis1, axis2를 렌더 좌표로 변환
                let sa = &arc.drawing.shape_attr;
                let sx = if sa.original_width > 0 {
                    render_w / hwpunit_to_px(sa.original_width as i32, self.dpi)
                } else {
                    1.0
                };
                let sy = if sa.original_height > 0 {
                    render_h / hwpunit_to_px(sa.original_height as i32, self.dpi)
                } else {
                    1.0
                };

                let cx = render_x + hwpunit_to_px(arc.center.x, self.dpi) * sx;
                let cy = render_y + hwpunit_to_px(arc.center.y, self.dpi) * sy;
                let ax1 = render_x + hwpunit_to_px(arc.axis1.x, self.dpi) * sx;
                let ay1 = render_y + hwpunit_to_px(arc.axis1.y, self.dpi) * sy;
                let ax2 = render_x + hwpunit_to_px(arc.axis2.x, self.dpi) * sx;
                let ay2 = render_y + hwpunit_to_px(arc.axis2.y, self.dpi) * sy;

                // center→axis 벡터
                let dx1 = ax1 - cx;
                let dy1 = ay1 - cy;
                let dx2 = ax2 - cx;
                let dy2 = ay2 - cy;

                // 타원 반지름을 axis 좌표 기반으로 계산
                // axis1/axis2가 정확히 축 위에 있으면 각각의 축 반지름 사용
                let (ell_rx, ell_ry) = {
                    let r1 = (dx1 * dx1 + dy1 * dy1).sqrt();
                    let r2 = (dx2 * dx2 + dy2 * dy2).sqrt();
                    if r1 > 0.1 && r2 > 0.1 {
                        // axis1, axis2가 직교축 위에 있다면 rx, ry 분리 가능
                        let a1 = dy1.atan2(dx1);
                        let a2 = dy2.atan2(dx2);
                        let a1_abs = a1.abs();
                        let a2_abs = a2.abs();
                        // axis1이 y축 근처(90° 또는 -90°)이고 axis2가 x축 근처(0° 또는 180°)
                        if (a1_abs - std::f64::consts::FRAC_PI_2).abs() < 0.3 && a2_abs < 0.3 {
                            (r2, r1) // axis2→rx, axis1→ry
                        } else if a1_abs < 0.3 && (a2_abs - std::f64::consts::FRAC_PI_2).abs() < 0.3
                        {
                            (r1, r2) // axis1→rx, axis2→ry
                        } else {
                            (r1.max(r2), r1.min(r2))
                        }
                    } else {
                        (render_w / 2.0, render_h / 2.0)
                    }
                };

                // 각도 계산 (SVG 좌표계: Y-down)
                let angle1 = dy1.atan2(dx1);
                let angle2 = dy2.atan2(dx2);

                // axis1→axis2 방향: 반시계 방향(SVG 좌표) = sweep=0
                // 각도 차이로 large_arc 결정
                let mut sweep_angle = angle1 - angle2;
                if sweep_angle < 0.0 {
                    sweep_angle += 2.0 * std::f64::consts::PI;
                }
                let large_arc = sweep_angle > std::f64::consts::PI;

                let mut commands = Vec::new();
                commands.push(PathCommand::MoveTo(ax1, ay1));
                commands.push(PathCommand::ArcTo(
                    ell_rx, ell_ry, 0.0, large_arc, false, ax2, ay2,
                ));

                match arc.arc_type {
                    1 => {
                        // 부채꼴 (Pie/CircularSector): 호 → 중심 → 시작점 → 닫기
                        commands.push(PathCommand::LineTo(cx, cy));
                        commands.push(PathCommand::ClosePath);
                    }
                    2 => {
                        // 활 (Bow/Chord): 호 끝점 → 시작점 직선 → 닫기
                        commands.push(PathCommand::ClosePath);
                    }
                    _ => {
                        // 호 (Arc): 열린 곡선 (닫지 않음)
                    }
                }

                let node_id = tree.next_id();
                let mut path_node = PathNode::new(commands, style, gradient);
                path_node.section_index = Some(section_index);
                path_node.para_index = Some(para_index);
                path_node.control_index = Some(control_index);
                path_node.transform = transform;
                path_node.cell_index = cell_index;
                path_node.cell_para_index = cell_para_index;
                path_node.outer_table_control_index = outer_table_control_index;
                let node = RenderNode::new(
                    node_id,
                    RenderNodeType::Path(path_node),
                    BoundingBox::new(render_x, render_y, render_w, render_h),
                );
                parent.children.push(node);
            }
            ShapeObject::Polygon(poly) => {
                let (style, gradient) = drawing_to_shape_style(&poly.drawing);
                // 꼭짓점 좌표를 PathCommand로 변환 (원본→렌더 스케일링 적용)
                let sa = &poly.drawing.shape_attr;
                let sx = if sa.original_width > 0 {
                    render_w / hwpunit_to_px(sa.original_width as i32, self.dpi)
                } else {
                    1.0
                };
                let sy = if sa.original_height > 0 {
                    render_h / hwpunit_to_px(sa.original_height as i32, self.dpi)
                } else {
                    1.0
                };
                let mut commands = Vec::new();
                for (i, pt) in poly.points.iter().enumerate() {
                    let px = render_x + hwpunit_to_px(pt.x, self.dpi) * sx;
                    let py = render_y + hwpunit_to_px(pt.y, self.dpi) * sy;
                    if i == 0 {
                        commands.push(PathCommand::MoveTo(px, py));
                    } else {
                        commands.push(PathCommand::LineTo(px, py));
                    }
                }
                // 첫 점과 끝 점이 같으면 닫힌 다각형, 다르면 열린 폴리라인
                if poly.points.len() >= 3 {
                    let first = &poly.points[0];
                    let last = &poly.points[poly.points.len() - 1];
                    if first.x == last.x && first.y == last.y {
                        commands.push(PathCommand::ClosePath);
                    }
                }
                let node_id = tree.next_id();
                let mut path_node = PathNode::new(commands, style, gradient);
                path_node.section_index = Some(section_index);
                path_node.para_index = Some(para_index);
                path_node.control_index = Some(control_index);
                path_node.transform = transform;
                path_node.cell_index = cell_index;
                path_node.cell_para_index = cell_para_index;
                path_node.outer_table_control_index = outer_table_control_index;
                let mut node = RenderNode::new(
                    node_id,
                    RenderNodeType::Path(path_node),
                    BoundingBox::new(render_x, render_y, render_w, render_h),
                );
                self.add_image_fill_node(
                    tree,
                    &mut node,
                    &poly.drawing,
                    render_x,
                    render_y,
                    render_w,
                    render_h,
                    bin_data_content,
                );
                let empty_map = std::collections::HashMap::new();
                self.layout_textbox_content(
                    tree,
                    &mut node,
                    &poly.drawing,
                    base_x,
                    base_y,
                    w,
                    h,
                    section_index,
                    para_index,
                    control_index,
                    styles,
                    bin_data_content,
                    &empty_map,
                    parent_cell_path,
                    shape.common().treat_as_char,
                    matrix_positioned,
                );
                parent.children.push(node);
            }
            ShapeObject::Curve(curve) => {
                let (style, gradient) = drawing_to_shape_style(&curve.drawing);
                let sa = &curve.drawing.shape_attr;
                let sx = if sa.original_width > 0 {
                    render_w / hwpunit_to_px(sa.original_width as i32, self.dpi)
                } else {
                    1.0
                };
                let sy = if sa.original_height > 0 {
                    render_h / hwpunit_to_px(sa.original_height as i32, self.dpi)
                } else {
                    1.0
                };
                let commands =
                    self.curve_to_path_commands_scaled(curve, render_x, render_y, sx, sy);
                let node_id = tree.next_id();
                let mut path_node = PathNode::new(commands, style, gradient);
                path_node.section_index = Some(section_index);
                path_node.para_index = Some(para_index);
                path_node.control_index = Some(control_index);
                path_node.transform = transform;
                path_node.cell_index = cell_index;
                path_node.cell_para_index = cell_para_index;
                path_node.outer_table_control_index = outer_table_control_index;
                let mut node = RenderNode::new(
                    node_id,
                    RenderNodeType::Path(path_node),
                    BoundingBox::new(render_x, render_y, render_w, render_h),
                );
                self.add_image_fill_node(
                    tree,
                    &mut node,
                    &curve.drawing,
                    render_x,
                    render_y,
                    render_w,
                    render_h,
                    bin_data_content,
                );
                let empty_map = std::collections::HashMap::new();
                self.layout_textbox_content(
                    tree,
                    &mut node,
                    &curve.drawing,
                    base_x,
                    base_y,
                    w,
                    h,
                    section_index,
                    para_index,
                    control_index,
                    styles,
                    bin_data_content,
                    &empty_map,
                    parent_cell_path,
                    shape.common().treat_as_char,
                    matrix_positioned,
                );
                parent.children.push(node);
            }
            ShapeObject::Group(group) => {
                // 묶음 개체: Group 컨테이너 노드로 감싸서 hittest 시 하나의 개체로 선택되도록 함
                let group_id = tree.next_id();
                let group_origin = inherited_group_origin.unwrap_or({
                    if group.shape_attr.group_level == 0 {
                        (base_x, base_y)
                    } else {
                        (0.0, 0.0)
                    }
                });
                let mut group_node = RenderNode::new(
                    group_id,
                    RenderNodeType::Group(GroupNode {
                        section_index: Some(section_index),
                        para_index: Some(para_index),
                        control_index: Some(control_index),
                    }),
                    BoundingBox::new(base_x, base_y, w, h),
                );
                for (_ci, child) in group.children.iter().enumerate() {
                    let sa = child.shape_attr();
                    let has_rotation = sa.render_b.abs() > 1e-6 || sa.render_c.abs() > 1e-6;

                    if has_rotation {
                        self.layout_group_child_affine(
                            tree,
                            &mut group_node,
                            child,
                            group_origin.0,
                            group_origin.1,
                            sa,
                            section_index,
                            para_index,
                            control_index,
                            styles,
                            bin_data_content,
                            parent_cell_path,
                        );
                    } else {
                        // render_tx/ty와 render_sx/sy에는 top-level group 로컬 좌표계
                        // 안에서 그룹 스케일/이동/flip이 합성되어 있다. top-level group
                        // anchor만 더하고, nested group bbox는 다시 더하지 않는다. 음수
                        // scale이면 tx는 오른쪽/아래쪽 모서리일 수 있으므로 두 변환
                        // 모서리의 min/max로 실제 bbox를 만든다.
                        let x0 = sa.render_tx;
                        let y0 = sa.render_ty;
                        let x1 = sa.render_tx + sa.original_width as f64 * sa.render_sx;
                        let y1 = sa.render_ty + sa.original_height as f64 * sa.render_sy;
                        let child_x = group_origin.0 + hwpunit_to_px(x0.min(x1) as i32, self.dpi);
                        let child_y = group_origin.1 + hwpunit_to_px(y0.min(y1) as i32, self.dpi);
                        let child_w = hwpunit_to_px((x1 - x0).abs() as i32, self.dpi);
                        let child_h = hwpunit_to_px((y1 - y0).abs() as i32, self.dpi);
                        let empty_map = std::collections::HashMap::new();
                        self.layout_shape_object_with_group_origin(
                            tree,
                            &mut group_node,
                            child,
                            child_x,
                            child_y,
                            child_w,
                            child_h,
                            section_index,
                            para_index,
                            control_index,
                            styles,
                            bin_data_content,
                            &empty_map,
                            parent_cell_path,
                            table_cell_ref, // [Task #1138] 그룹 자식 — 부모와 같은 셀 컨텍스트
                            true,
                            Some(group_origin),
                        );
                    }
                }
                parent.children.push(group_node);
            }
            ShapeObject::Picture(pic) => {
                // 그룹 내 그림: common이 비어있으므로 w, h(shape_attr 기반)를 직접 사용
                let bin_data_id = pic.image_attr.bin_data_id;
                let image_data =
                    find_bin_data(bin_data_content, bin_data_id).map(|c| c.data.clone());
                let img_id = tree.next_id();
                let img_node = RenderNode::new(
                    img_id,
                    RenderNodeType::Image(ImageNode {
                        transform,
                        effect: pic.image_attr.effect,
                        brightness: pic.image_attr.brightness,
                        contrast: pic.image_attr.contrast,
                        opacity: pic.image_attr.opacity(),
                        text_wrap: Some(pic.common.text_wrap),
                        external_path: pic.image_attr.external_path.clone(),
                        ..ImageNode::new(bin_data_id, image_data)
                    }),
                    BoundingBox::new(render_x, render_y, render_w, render_h),
                );
                parent.children.push(img_node);
            }
            ShapeObject::Chart(chart) => {
                // Task #195: 차트 placeholder (점선 테두리 + "차트" 라벨)
                let _chart = chart;
                let node_id = tree.next_id();
                let node = RenderNode::new(
                    node_id,
                    RenderNodeType::Placeholder(crate::renderer::render_tree::PlaceholderNode {
                        fill_color: 0xFFE8F0FE,
                        stroke_color: 0xFF4A90E2,
                        label: "차트 (Chart)".to_string(),
                    }),
                    BoundingBox::new(render_x, render_y, render_w, render_h),
                );
                parent.children.push(node);
            }
            ShapeObject::Ole(ole) => {
                // Task #195 단계 8: BinData에서 OOXML 차트 시도 → 성공 시 네이티브 SVG 렌더
                let mut rendered = false;
                if let Some(content) = find_bin_data(bin_data_content, ole.bin_data_id as u16) {
                    // HWPX에서 주입된 OOXML 차트 XML 직접 경로 (CFB 컨테이너 없음)
                    if content.extension == "ooxml_chart" {
                        if let Some(chart) = crate::ooxml_chart::OoxmlChart::parse(&content.data) {
                            let svg_fragment =
                                chart.render_svg(render_x, render_y, render_w, render_h);
                            let node_id = tree.next_id();
                            let node = RenderNode::new(
                                node_id,
                                RenderNodeType::RawSvg(crate::renderer::render_tree::RawSvgNode {
                                    svg: svg_fragment,
                                }),
                                BoundingBox::new(render_x, render_y, render_w, render_h),
                            );
                            parent.children.push(node);
                            rendered = true;
                        }
                    }
                    if !rendered {
                        if let Some(container) =
                            crate::parser::ole_container::parse_ole_container(&content.data)
                        {
                            if let Some(ooxml_bytes) = container.ooxml_chart.as_ref() {
                                if let Some(chart) =
                                    crate::ooxml_chart::OoxmlChart::parse(ooxml_bytes)
                                {
                                    let svg_fragment =
                                        chart.render_svg(render_x, render_y, render_w, render_h);
                                    push_raw_svg_render_node(
                                        tree,
                                        parent,
                                        BoundingBox::new(render_x, render_y, render_w, render_h),
                                        svg_fragment,
                                    );
                                    rendered = true;
                                }
                            }

                            // Issue #1251: legacy HWP chart `Contents` stream.
                            if !rendered {
                                if let Some(raw_contents) = container.raw_contents.as_ref() {
                                    match crate::ole_chart::parse_ole_chart_contents(raw_contents) {
                                        Ok(ole_chart) => {
                                            let svg_fragment =
                                                crate::ole_chart::render_ole_chart_svg_fragment(
                                                    &ole_chart,
                                                    render_x,
                                                    render_y,
                                                    render_w,
                                                    render_h,
                                                    ole.bin_data_id,
                                                );
                                            push_raw_svg_render_node(
                                                tree,
                                                parent,
                                                BoundingBox::new(
                                                    render_x, render_y, render_w, render_h,
                                                ),
                                                svg_fragment,
                                            );
                                            rendered = true;
                                        }
                                        Err(error) => {
                                            push_placeholder_render_node(
                                                tree,
                                                parent,
                                                BoundingBox::new(
                                                    render_x, render_y, render_w, render_h,
                                                ),
                                                0xFFFFF4E5,
                                                0xFFB45F06,
                                                ole_chart_fallback_label(
                                                    error.stable_message(),
                                                    ole.bin_data_id,
                                                ),
                                            );
                                            rendered = true;
                                        }
                                    }
                                }
                            }

                            // Task #195 단계 14: OOXML 차트 부재 시 EMF 네이티브 SVG 폴백
                            if !rendered {
                                if let Some(emf_bytes) = container.preview_emf.as_ref() {
                                    let render_rect = (
                                        render_x as f32,
                                        render_y as f32,
                                        render_w as f32,
                                        render_h as f32,
                                    );
                                    if let Ok(svg_fragment) =
                                        crate::emf::convert_to_svg(emf_bytes, render_rect)
                                    {
                                        let node_id = tree.next_id();
                                        let node = RenderNode::new(
                                            node_id,
                                            RenderNodeType::RawSvg(
                                                crate::renderer::render_tree::RawSvgNode {
                                                    svg: svg_fragment,
                                                },
                                            ),
                                            BoundingBox::new(
                                                render_x, render_y, render_w, render_h,
                                            ),
                                        );
                                        parent.children.push(node);
                                        rendered = true;
                                    }
                                }
                            }

                            // 네이티브 임베딩 이미지(BMP/PNG/JPEG/GIF) 폴백
                            if !rendered {
                                if let Some((kind, bytes)) = container.native_image.as_ref() {
                                    use base64::Engine;
                                    // BMP → PNG 재인코딩 (SVG <image>는 data:image/bmp 미지원)
                                    let (render_bytes, render_mime): (
                                        std::borrow::Cow<[u8]>,
                                        &str,
                                    ) = if kind.mime() == "image/bmp" {
                                        match crate::renderer::svg::bmp_bytes_to_png_bytes(bytes) {
                                            Some(png) => {
                                                (std::borrow::Cow::Owned(png), "image/png")
                                            }
                                            None => (
                                                std::borrow::Cow::Borrowed(bytes.as_slice()),
                                                kind.mime(),
                                            ),
                                        }
                                    } else {
                                        (std::borrow::Cow::Borrowed(bytes.as_slice()), kind.mime())
                                    };
                                    let b64 = base64::engine::general_purpose::STANDARD
                                        .encode(&*render_bytes);
                                    let href = format!("data:{};base64,{}", render_mime, b64);
                                    let svg_fragment = format!(
                                    "<image x=\"{:.2}\" y=\"{:.2}\" width=\"{:.2}\" height=\"{:.2}\" preserveAspectRatio=\"xMidYMid meet\" xlink:href=\"{}\" href=\"{}\"/>",
                                    render_x, render_y, render_w, render_h, href, href
                                );
                                    let node_id = tree.next_id();
                                    let node = RenderNode::new(
                                        node_id,
                                        RenderNodeType::RawSvg(
                                            crate::renderer::render_tree::RawSvgNode {
                                                svg: svg_fragment,
                                            },
                                        ),
                                        BoundingBox::new(render_x, render_y, render_w, render_h),
                                    );
                                    parent.children.push(node);
                                    rendered = true;
                                }
                            }
                        }
                    } // !rendered (CFB path)
                }

                if !rendered {
                    // 폴백: placeholder
                    let label = format!("OLE 개체 (BinData #{})", ole.bin_data_id);
                    let node_id = tree.next_id();
                    let node = RenderNode::new(
                        node_id,
                        RenderNodeType::Placeholder(
                            crate::renderer::render_tree::PlaceholderNode {
                                fill_color: 0xFFF0F0F0,
                                stroke_color: 0xFF707070,
                                label,
                            },
                        ),
                        BoundingBox::new(render_x, render_y, render_w, render_h),
                    );
                    parent.children.push(node);
                }
            }
        }
    }

    /// 도형의 이미지 채우기를 자식 이미지 노드로 추가한다.
    pub(crate) fn add_image_fill_node(
        &self,
        tree: &mut PageRenderTree,
        parent: &mut RenderNode,
        drawing: &crate::model::shape::DrawingObjAttr,
        base_x: f64,
        base_y: f64,
        w: f64,
        h: f64,
        bin_data_content: &[BinDataContent],
    ) {
        use crate::model::style::FillType;
        if drawing.fill.fill_type == FillType::Image {
            if let Some(ref img_fill) = drawing.fill.image {
                let bin_data_id = img_fill.bin_data_id;
                let image_data =
                    find_bin_data(bin_data_content, bin_data_id).map(|c| c.data.clone());
                // 이미지 원본 크기: shape_attr의 original_width/height (HWPUNIT)
                let original_size = {
                    let ow = drawing.shape_attr.original_width;
                    let oh = drawing.shape_attr.original_height;
                    if ow > 0 && oh > 0 {
                        Some((
                            hwpunit_to_px(ow as i32, self.dpi),
                            hwpunit_to_px(oh as i32, self.dpi),
                        ))
                    } else {
                        None
                    }
                };

                let img_id = tree.next_id();
                let img_node = RenderNode::new(
                    img_id,
                    RenderNodeType::Image(ImageNode {
                        fill_mode: Some(img_fill.fill_mode),
                        original_size,
                        brightness: img_fill.brightness,
                        contrast: img_fill.contrast,
                        effect: match img_fill.effect {
                            1 => crate::model::image::ImageEffect::GrayScale,
                            2 => crate::model::image::ImageEffect::BlackWhite,
                            3 => crate::model::image::ImageEffect::Pattern8x8,
                            _ => crate::model::image::ImageEffect::RealPic,
                        },
                        ..ImageNode::new(bin_data_id, image_data)
                    }),
                    BoundingBox::new(base_x, base_y, w, h),
                );
                parent.children.push(img_node);
            }
        }
    }

    fn max_descendant_bottom(node: &RenderNode) -> Option<f64> {
        node.children.iter().fold(None, |acc, child| {
            let child_bottom = child.bbox.y + child.bbox.height;
            let descendant_bottom = Self::max_descendant_bottom(child).unwrap_or(child_bottom);
            Some(acc.map_or(descendant_bottom, |current: f64| {
                current.max(descendant_bottom)
            }))
        })
    }

    fn expand_inline_textbox_to_content(
        shape_node: &mut RenderNode,
        textbox_node: &mut RenderNode,
        bottom_padding: f64,
    ) {
        let Some(content_bottom) = Self::max_descendant_bottom(textbox_node) else {
            return;
        };

        let textbox_bottom = textbox_node.bbox.y + textbox_node.bbox.height;
        if content_bottom > textbox_bottom {
            textbox_node.bbox.height = content_bottom - textbox_node.bbox.y;
        }

        let shape_bottom = shape_node.bbox.y + shape_node.bbox.height;
        let required_shape_bottom = content_bottom + bottom_padding;
        if required_shape_bottom > shape_bottom {
            shape_node.bbox.height = required_shape_bottom - shape_node.bbox.y;
        }
    }

    /// 도형 내 TextBox 콘텐츠를 레이아웃한다.
    /// parent_cell_path: 상위 글상자/표의 경로 (최상위이면 빈 슬라이스)
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn layout_textbox_content(
        &self,
        tree: &mut PageRenderTree,
        shape_node: &mut RenderNode,
        drawing: &crate::model::shape::DrawingObjAttr,
        base_x: f64,
        base_y: f64,
        w: f64,
        h: f64,
        section_index: usize,
        para_index: usize,
        control_index: usize,
        styles: &ResolvedStyleSet,
        bin_data_content: &[BinDataContent],
        overflow_map: &std::collections::HashMap<(usize, usize), Vec<Paragraph>>,
        parent_cell_path: &[CellPathEntry],
        parent_treat_as_char: bool,
        matrix_positioned: bool,
    ) {
        let text_box = match &drawing.text_box {
            Some(tb) => tb,
            None => return,
        };

        let margin_left = hwpunit_to_px(text_box.margin_left as i32, self.dpi);
        let margin_right = hwpunit_to_px(text_box.margin_right as i32, self.dpi);
        let margin_top = hwpunit_to_px(text_box.margin_top as i32, self.dpi);
        let margin_bottom = hwpunit_to_px(text_box.margin_bottom as i32, self.dpi);

        let inner_area = LayoutRect {
            x: base_x + margin_left,
            y: base_y + margin_top,
            width: (w - margin_left - margin_right).max(0.0),
            height: (h - margin_top - margin_bottom).max(0.0),
        };
        let mut textbox_node = RenderNode::new(
            tree.next_id(),
            RenderNodeType::TextBox,
            BoundingBox::new(
                inner_area.x,
                inner_area.y,
                inner_area.width,
                inner_area.height,
            ),
        );

        // [Task #874 #3 / #930] shortcut.hwp 1 페이지 우측하단 자동번호 "1" 시각 정합:
        // 도형이 원본 (sa.original_*) 대비 확대된 마스터 페이지 글상자는 한컴이 내부
        // 글꼴을 축소 렌더링한다. base_size 가 25400 (254 pt) 인 자동번호 글상자가 현재
        // 좌표계에서 그대로 339 px 로 그려지면 본문 우측 단축키 행과 겹친다.
        //
        // [#930] 발동 조건을 두 축 모두 1.5× 초과 (= min_ratio > 1.5, 등방 확대) 로
        // 좁힌다. table-in-tbox.hwp 2 페이지처럼 한 축만 강하게 늘어난 이방 확��
        // 글상자 (sx≈1.07, sy≈8.2) 는 내부 문단이 이미 current 박스 기준으로 조판된
        // 일반 본문이므로 축소하면 안 된다 (글꼴이 1/8 로 깨짐).
        //
        // [#930] 축소 계수는 2.0 / max_ratio. #874 의 1.0 / max_ratio 는 한컴 2022
        // PDF 측정 대비 자동번호를 약 2× 과축소했다 (글리프 93 px vs PDF 187 px).
        // 단일 샘플 (shortcut.hwp 자동번호) 기반 경험적 보정이므로, 다른 등방 확대
        // 글상자 샘플이 확보되면 재검증한다.
        let sa = &drawing.shape_attr;
        let local_styles_scaled: Option<ResolvedStyleSet> = {
            let positive_current_ratio = |current: u32, original: u32| -> f64 {
                let signed_current = current as i32;
                if original > 0 && signed_current > 0 {
                    signed_current as f64 / original as f64
                } else {
                    1.0
                }
            };
            let sw_ratio = positive_current_ratio(sa.current_width, sa.original_width);
            let sh_ratio = positive_current_ratio(sa.current_height, sa.original_height);
            let max_ratio = sw_ratio.max(sh_ratio);
            let min_ratio = sw_ratio.min(sh_ratio);
            // [Task #928] 인라인 도형 (treat_as_char=true) 은 폰트 자동 축소 적용 안 함.
            // exam_kor 5p 다이어그램의 사각형 [A 단계] 가 original=(2925, 975) HU →
            // current=(6518, 1983) HU 로 2.2배 확대된 채 IR 인코딩되어 있으나 한컴은
            // 폰트를 11.5pt 그대로 유지한다. 본 ratio 축소 (#874 #3) 가 잘못 발동하여
            // 폰트가 6.88px 로 축소되던 회귀를 차단. shortcut.hwp 마스터 페이지
            // 글상자는 tac=false 라 ratio 적용 유지.
            //
            // [PR #1276] HWPX 바탕쪽 하단 글상자처럼 current size 가 음수 HWPUNIT 을
            // u32 로 보존한 값이면(예: curSz height 4294965455 = -1841) 확대율로
            // 사용하지 않는다. 이를 양수로 해석하면 sh_ratio 가 수백만 배가 되어
            // 내부 글꼴이 0px 에 가깝게 붕괴한다.
            if min_ratio > 1.5 && !parent_treat_as_char {
                let inv = (2.0 / max_ratio).min(1.0);
                let mut local = styles.clone();
                for cs in local.char_styles.iter_mut() {
                    cs.font_size *= inv;
                    cs.letter_spacing *= inv;
                    for ls in cs.letter_spacings.iter_mut() {
                        *ls *= inv;
                    }
                }
                Some(local)
            } else {
                None
            }
        };
        let styles: &ResolvedStyleSet = local_styles_scaled.as_ref().unwrap_or(styles);

        // 세로쓰기 판정: 글상자 list_attr bit 0~2 = text_direction
        // (0=가로, 1=영문 눕힘, 2=영문 세움)
        // 주의: 테이블 셀은 bit 16~18이지만 글상자 LIST_HEADER는 bit 0~2
        let text_direction = (text_box.list_attr & 0x07) as u8;
        let reflowed_textbox_paragraphs = if should_reflow_matrix_textbox_lines(
            matrix_positioned,
            drawing,
            text_box,
            text_direction,
            inner_area.width,
            inner_area.height,
            self.dpi,
        ) {
            let mut paragraphs = text_box.paragraphs.clone();
            for para in paragraphs
                .iter_mut()
                .filter(|para| matrix_textbox_lines_need_reflow(para))
            {
                reflow_matrix_textbox_para(
                    para,
                    inner_area.width,
                    inner_area.height,
                    styles,
                    self.dpi,
                );
            }
            Some(paragraphs)
        } else {
            None
        };
        let textbox_paragraphs = reflowed_textbox_paragraphs
            .as_deref()
            .unwrap_or(&text_box.paragraphs);

        // 빈 텍스트박스에 오버플로우 문단이 매핑되어 있는지 확인 (가로/세로 공통)
        let key = (para_index, control_index);
        if let Some(overflow_paras) = overflow_map.get(&key) {
            if text_direction != 0 {
                // 세로쓰기 오버플로우 수신: 오버플로우 문단을 세로 레이아웃으로 렌더링
                self.layout_vertical_textbox_text_with_paras(
                    tree,
                    &mut textbox_node,
                    overflow_paras,
                    text_box,
                    styles,
                    &inner_area,
                    text_direction,
                    section_index,
                    para_index,
                    control_index,
                    parent_cell_path,
                );
            } else {
                // 이 텍스트박스는 연결된 텍스트박스의 타겟: 오버플로우 문단 렌더링
                let composed_paras: Vec<_> = overflow_paras
                    .iter()
                    .map(|p| compose_paragraph(p))
                    .collect();
                let mut para_y = inner_area.y;
                for (tb_para_idx, composed) in composed_paras.iter().enumerate() {
                    let para = &overflow_paras[tb_para_idx];
                    if let Some(first_ls) = para.line_segs.first() {
                        let vpos_y = inner_area.y + hwpunit_to_px(first_ls.vertical_pos, self.dpi);
                        para_y = vpos_y.max(para_y);
                    }
                    let para_col_area = LayoutRect {
                        y: para_y,
                        ..inner_area
                    };
                    let cell_ctx = CellContext {
                        parent_para_index: para_index,
                        path: {
                            let mut p = parent_cell_path.to_vec();
                            p.push(CellPathEntry {
                                control_index,
                                cell_index: 0,
                                cell_para_index: tb_para_idx,
                                text_direction: 0,
                            });
                            p
                        },
                    };
                    let is_last_para = tb_para_idx + 1 == composed_paras.len();
                    para_y = self.layout_composed_paragraph(
                        tree,
                        &mut textbox_node,
                        composed,
                        styles,
                        &para_col_area,
                        para_y,
                        0,
                        composed.lines.len(),
                        section_index,
                        tb_para_idx,
                        Some(cell_ctx),
                        true,
                        is_last_para,
                        0.0,
                        None,
                        Some(para),
                        None,
                        None, // 도형 컨텍스트 — wrap zone 무관
                    );
                }
            }
            if !textbox_node.children.is_empty() {
                if parent_treat_as_char {
                    Self::expand_inline_textbox_to_content(
                        shape_node,
                        &mut textbox_node,
                        margin_bottom,
                    );
                }
                shape_node.children.push(textbox_node);
            }
            return;
        }

        // 오버플로우 감지 (가로/세로 공통): 텍스트박스 내 문단의 line_segs에서
        // vpos가 리셋(이전 문단보다 감소)되고 sw가 변경되면
        // 해당 문단은 다른 텍스트박스(연결된 글상자)에 속함
        let first_sw = textbox_paragraphs
            .first()
            .and_then(|p| p.line_segs.first())
            .map(|ls| ls.segment_width)
            .unwrap_or(0);
        let mut max_vpos_end: i32 = 0;
        let mut overflow_start_idx: Option<usize> = None;
        for (pi, para) in textbox_paragraphs.iter().enumerate() {
            if let Some(first_ls) = para.line_segs.first() {
                let sw = first_ls.segment_width;
                let vpos = first_ls.vertical_pos;
                // sw가 변경되고 vpos가 이전 최대치보다 작으면 오버플로우
                if pi > 0 && sw != first_sw && vpos < max_vpos_end {
                    overflow_start_idx = Some(pi);
                    break;
                }
                // 이 문단의 마지막 line_seg의 끝 위치 추적
                if let Some(last_ls) = para.line_segs.last() {
                    let end = last_ls.vertical_pos + last_ls.line_height;
                    if end > max_vpos_end {
                        max_vpos_end = end;
                    }
                }
            }
        }

        // 현재 텍스트박스에 속하는 문단만 사용
        let para_count = overflow_start_idx.unwrap_or(textbox_paragraphs.len());

        // 세로쓰기: 오버플로우 감지 후 세로 레이아웃으로 분기
        if text_direction != 0 {
            self.layout_vertical_textbox_text_with_paras(
                tree,
                &mut textbox_node,
                &textbox_paragraphs[..para_count],
                text_box,
                styles,
                &inner_area,
                text_direction,
                section_index,
                para_index,
                control_index,
                parent_cell_path,
            );
            if !textbox_node.children.is_empty() {
                if parent_treat_as_char {
                    Self::expand_inline_textbox_to_content(
                        shape_node,
                        &mut textbox_node,
                        margin_bottom,
                    );
                }
                shape_node.children.push(textbox_node);
            }
            return;
        }

        let mut composed_paras: Vec<_> = textbox_paragraphs[..para_count]
            .iter()
            .map(|p| compose_paragraph(p))
            .collect();

        // AutoNumber(Page) 치환: 글상자 안의 쪽번호 필드를 현재 페이지 번호로 변환
        let current_pn = self.current_page_number.get();
        if current_pn > 0 {
            for (pi, para) in textbox_paragraphs[..para_count].iter().enumerate() {
                if para.controls.iter().any(|c| {
                    matches!(c, crate::model::control::Control::AutoNumber(an)
                        if an.number_type == crate::model::control::AutoNumberType::Page)
                }) {
                    if let Some(comp) = composed_paras.get_mut(pi) {
                        self.substitute_page_auto_numbers_in_composed(para, comp, current_pn);
                    }
                }
            }
        }

        // 세로 정렬: 전체 콘텐츠 높이를 계산하여 center/bottom 오프셋 적용
        let vert_offset = {
            use crate::model::table::VerticalAlign;
            match text_box.vertical_align {
                VerticalAlign::Center | VerticalAlign::Bottom => {
                    // 전체 콘텐츠 높이 = line_seg 끝 위치와 글상자 내부 non-TAC 개체 높이의 최대값.
                    //
                    // HWPX hy-002처럼 글상자 문단이 빈 line_seg + non-TAC 그림 하나로 구성되면
                    // line_seg 높이만으로 CENTER 오프셋을 계산할 수 없다. 그렇게 하면 그림이 실제
                    // 콘텐츠 높이에서 빠지고, 아래 picture container가 오프셋 이후 남은 높이로
                    // 줄어들어 한컴보다 과도하게 축소된다.
                    let mut total_content_height = textbox_paragraphs[..para_count]
                        .iter()
                        .flat_map(|p| p.line_segs.last())
                        .map(|ls| hwpunit_to_px(ls.vertical_pos + ls.line_height, self.dpi))
                        .last()
                        .unwrap_or(0.0);

                    for para in &textbox_paragraphs[..para_count] {
                        let para_vpos = para
                            .line_segs
                            .first()
                            .map(|ls| ls.vertical_pos)
                            .unwrap_or(0);
                        for ctrl in &para.controls {
                            let common = match ctrl {
                                Control::Picture(pic) if !pic.common.treat_as_char => {
                                    Some(&pic.common)
                                }
                                Control::Shape(shape) if !shape.as_ref().common().treat_as_char => {
                                    Some(shape.as_ref().common())
                                }
                                _ => None,
                            };

                            if let Some(common) = common {
                                let top = para_vpos.saturating_add(common.vertical_offset as i32);
                                let bottom = top.saturating_add(common.height as i32);
                                total_content_height =
                                    total_content_height.max(hwpunit_to_px(bottom, self.dpi));
                            }
                        }
                    }

                    let free_space = (inner_area.height - total_content_height).max(0.0);
                    match text_box.vertical_align {
                        VerticalAlign::Center => free_space / 2.0,
                        VerticalAlign::Bottom => free_space,
                        _ => 0.0,
                    }
                }
                _ => 0.0,
            }
        };

        let mut para_y = inner_area.y + vert_offset;
        for (tb_para_idx, composed) in composed_paras.iter().enumerate() {
            // vpos 기반 수직 위치: 원본 HWP 파일에서는 vertical_pos가 누적 절대값,
            // 편집 후 reflow된 문단은 vertical_pos=0이므로 incremental para_y와 비교하여
            // 더 큰 값 사용 (원본 호환 + 편집 후 정상 배치 모두 지원)
            let para = &textbox_paragraphs[tb_para_idx];
            if let Some(first_ls) = para.line_segs.first() {
                let vpos_y =
                    inner_area.y + vert_offset + hwpunit_to_px(first_ls.vertical_pos, self.dpi);
                para_y = vpos_y.max(para_y);
            }
            // 인라인(treat_as_char) 컨트롤의 총 폭 계산
            let tb_inline_width: f64 = para
                .controls
                .iter()
                .map(|ctrl| match ctrl {
                    Control::Picture(pic) if pic.common.treat_as_char => {
                        hwpunit_to_px(pic.common.width as i32, self.dpi)
                    }
                    Control::Shape(shape) if shape.common().treat_as_char => {
                        hwpunit_to_px(shape.common().width as i32, self.dpi)
                    }
                    _ => 0.0,
                })
                .sum();
            let para_col_area = LayoutRect {
                y: para_y,
                ..inner_area
            };
            let cell_ctx = CellContext {
                parent_para_index: para_index,
                path: {
                    let mut p = parent_cell_path.to_vec();
                    p.push(CellPathEntry {
                        control_index,
                        cell_index: 0,
                        cell_para_index: tb_para_idx,
                        text_direction: 0,
                    });
                    p
                },
            };
            let is_last_para = tb_para_idx + 1 == composed_paras.len();
            para_y = self.layout_composed_paragraph(
                tree,
                &mut textbox_node,
                composed,
                styles,
                &para_col_area,
                para_y,
                0,
                composed.lines.len(),
                section_index,
                tb_para_idx,
                Some(cell_ctx),
                true,
                is_last_para,
                tb_inline_width,
                None,
                Some(para),
                None,
                None, // 도형 컨텍스트 — wrap zone 무관
            );
        }

        // 텍스트 박스 내 인라인 도형 컨트롤 렌더링
        // treat_as_char 도형은 텍스트 흐름 내 문단 시작 위치에 배치
        // 오버플로우된 문단은 제외 (다른 텍스트박스에서 처리)
        use crate::model::shape::ShapeObject;
        let mut inline_y = inner_area.y + vert_offset; // 텍스트 영역 시작 위치
        for (pi, para) in textbox_paragraphs[..para_count].iter().enumerate() {
            // 이 문단에 해당하는 composed 문단의 시작 y 위치 계산
            let para_start_y = if pi < composed_paras.len() {
                if let Some(first_seg) = para.line_segs.first() {
                    inner_area.y + vert_offset + hwpunit_to_px(first_seg.vertical_pos, self.dpi)
                } else {
                    inline_y
                }
            } else {
                inline_y
            };

            // 문단 정렬 조회
            let para_alignment = if pi < composed_paras.len() {
                let para_style_id = composed_paras[pi].para_style_id as usize;
                styles
                    .para_styles
                    .get(para_style_id)
                    .map(|s| s.alignment)
                    .unwrap_or(Alignment::Left)
            } else {
                Alignment::Left
            };

            // 문단 시작 위치로 inline_y 갱신 (이전 문단보다 앞으로 가지 않도록)
            inline_y = inline_y.max(para_start_y);

            // 인라인(treat_as_char) 컨트롤의 총 너비를 먼저 계산하여 정렬에 사용
            let mut total_inline_width = 0.0f64;
            let mut max_inline_height = 0.0f64;
            for ctrl in &para.controls {
                match ctrl {
                    Control::Shape(shape) => {
                        let child_common = shape.as_ref().common();
                        if child_common.treat_as_char {
                            total_inline_width +=
                                hwpunit_to_px(child_common.width as i32, self.dpi);
                            max_inline_height = max_inline_height
                                .max(hwpunit_to_px(child_common.height as i32, self.dpi));
                        }
                    }
                    Control::Picture(pic) => {
                        if pic.common.treat_as_char {
                            total_inline_width += hwpunit_to_px(pic.common.width as i32, self.dpi);
                            max_inline_height = max_inline_height
                                .max(hwpunit_to_px(pic.common.height as i32, self.dpi));
                        }
                    }
                    Control::Equation(eq) => {
                        total_inline_width += hwpunit_to_px(eq.common.width as i32, self.dpi);
                        max_inline_height =
                            max_inline_height.max(hwpunit_to_px(eq.common.height as i32, self.dpi));
                    }
                    _ => {}
                }
            }

            // 인라인 컨트롤의 시작 x 위치 (정렬 기반)
            // 이미지+텍스트 전체 폭을 기준으로 정렬 (함께 센터링)
            let first_line_text_width: f64 = if total_inline_width > 0.0
                && pi < composed_paras.len()
            {
                if let Some(first_line) = composed_paras[pi].lines.first() {
                    let tab_width = styles
                        .para_styles
                        .get(composed_paras[pi].para_style_id as usize)
                        .map(|s| s.default_tab_width)
                        .unwrap_or(0.0);
                    first_line
                        .runs
                        .iter()
                        .map(|run| {
                            let mut ts =
                                resolved_to_text_style(styles, run.char_style_id, run.lang_index);
                            ts.default_tab_width = tab_width;
                            estimate_text_width(&run.text, &ts)
                        })
                        .sum()
                } else {
                    0.0
                }
            } else {
                0.0
            };
            let total_line_width = total_inline_width + first_line_text_width;
            let mut inline_x = match para_alignment {
                Alignment::Center | Alignment::Distribute => {
                    inner_area.x + (inner_area.width - total_line_width).max(0.0) / 2.0
                }
                Alignment::Right => inner_area.x + (inner_area.width - total_line_width).max(0.0),
                _ => inner_area.x,
            };

            let space_advance_override = if pi < composed_paras.len() {
                textbox_tac_space_advance_override(
                    para,
                    &composed_paras[pi],
                    total_inline_width,
                    para_alignment,
                    self.dpi,
                )
            } else {
                None
            };

            let control_text_positions = para.control_text_positions();
            let mut text_cursor = 0usize;

            for (ctrl_idx_in_para, ctrl) in para.controls.iter().enumerate() {
                let ctrl_text_pos = control_text_positions
                    .get(ctrl_idx_in_para)
                    .copied()
                    .unwrap_or(text_cursor);
                let mut advance_to_control = |inline_x: &mut f64| {
                    if ctrl_text_pos > text_cursor && pi < composed_paras.len() {
                        *inline_x += measure_composed_text_range_width(
                            &composed_paras[pi],
                            styles,
                            text_cursor,
                            ctrl_text_pos,
                            space_advance_override,
                        );
                        text_cursor = ctrl_text_pos;
                    }
                };

                match ctrl {
                    Control::Shape(shape) => {
                        let child_common = shape.as_ref().common();

                        let child_w = hwpunit_to_px(child_common.width as i32, self.dpi);
                        let child_h = hwpunit_to_px(child_common.height as i32, self.dpi);

                        let (child_x, child_y) = if child_common.treat_as_char {
                            // 인라인 도형: 텍스트 내 삽입 위치까지의 advance를 반영하여 배치
                            advance_to_control(&mut inline_x);
                            let x = inline_x;
                            inline_x += child_w;
                            (x, para_start_y)
                        } else {
                            // 절대 위치 도형
                            (
                                base_x
                                    + hwpunit_to_px(
                                        child_common.horizontal_offset as i32,
                                        self.dpi,
                                    ),
                                base_y
                                    + hwpunit_to_px(child_common.vertical_offset as i32, self.dpi),
                            )
                        };

                        // 중첩 도형: 현재 글상자의 경로 엔트리를 추가하여 자식에 전달
                        let mut nested_parent_path = parent_cell_path.to_vec();
                        nested_parent_path.push(CellPathEntry {
                            control_index,
                            cell_index: 0,
                            cell_para_index: pi,
                            text_direction: 0,
                        });
                        let empty_map = std::collections::HashMap::new();
                        self.layout_shape_object(
                            tree,
                            &mut textbox_node,
                            shape.as_ref(),
                            child_x,
                            child_y,
                            child_w,
                            child_h,
                            section_index,
                            para_index,
                            ctrl_idx_in_para,
                            styles,
                            bin_data_content,
                            &empty_map,
                            &nested_parent_path,
                            None, // [Task #1138] TODO: layout_textbox_content 에 cell ctx propagate (별도 후속)
                            false,
                        );
                    }
                    Control::Picture(pic) => {
                        // [Task #1171] 글상자 내부 picture — cellPath sentinel(cell_index=0)을
                        // 빌드해 ImageNode.cell_context 로 전달. text-run(1963행)/표 경로와 동일 패턴.
                        // 식별자: (바깥 Shape control_index, cell_index=0, 글상자 문단 pi). innerControlIdx 는
                        // layout_picture 의 control_index 인자(= ctrl_idx_in_para)로 별도 전달된다.
                        let pic_cell_ctx = CellContext {
                            parent_para_index: para_index,
                            path: {
                                let mut p = parent_cell_path.to_vec();
                                p.push(CellPathEntry {
                                    control_index,
                                    cell_index: 0,
                                    cell_para_index: pi,
                                    text_direction: 0,
                                });
                                p
                            },
                        };
                        if pic.common.treat_as_char {
                            // 인라인 이미지: 텍스트 내 삽입 위치까지의 advance를 반영하여 배치
                            advance_to_control(&mut inline_x);
                            let pic_w = hwpunit_to_px(pic.common.width as i32, self.dpi);
                            let pic_h = hwpunit_to_px(pic.common.height as i32, self.dpi);
                            // [Task #477] 도형 컨테이너 폭 초과 시 비율 유지 클램프
                            let clamped_w = pic_w.min(inner_area.width);
                            let clamped_h = if pic_w > 0.0 {
                                pic_h * (clamped_w / pic_w)
                            } else {
                                pic_h
                            };
                            let pic_container = LayoutRect {
                                x: inline_x,
                                y: inline_y,
                                width: clamped_w,
                                height: clamped_h,
                            };
                            // [Task #1151 v4] 글상자 내부 picture (inline) — section/para/control
                            // 전달하여 findPictureAtClick 의 secIdx undefined skip 차단.
                            self.layout_picture(
                                tree,
                                &mut textbox_node,
                                pic,
                                &pic_container,
                                bin_data_content,
                                Alignment::Left,
                                Some(section_index),
                                Some(para_index),
                                Some(ctrl_idx_in_para),
                                Some(&pic_cell_ctx),
                            );
                            inline_x += clamped_w;
                        } else {
                            // 절대 위치 이미지
                            let pic_container = LayoutRect {
                                x: inner_area.x,
                                y: inline_y,
                                width: inner_area.width,
                                height: (inner_area.height - (inline_y - inner_area.y)).max(0.0),
                            };
                            // [Task #1151 v4] 글상자 내부 picture (absolute) — section/para/control 전달.
                            self.layout_picture(
                                tree,
                                &mut textbox_node,
                                pic,
                                &pic_container,
                                bin_data_content,
                                para_alignment,
                                Some(section_index),
                                Some(para_index),
                                Some(ctrl_idx_in_para),
                                Some(&pic_cell_ctx),
                            );
                            let pic_h = hwpunit_to_px(pic.common.height as i32, self.dpi);
                            max_inline_height = max_inline_height.max(pic_h);
                        }
                    }
                    Control::Equation(eq) => {
                        // 글상자 내 수식: 항상 글자처럼 인라인 배치
                        let eq_w = hwpunit_to_px(eq.common.width as i32, self.dpi);
                        let eq_h = hwpunit_to_px(eq.common.height as i32, self.dpi);
                        // [Task #962] 글상자 내부 paragraph 의 inline equation 은
                        // paragraph_layout 가 layout_composed_paragraph 경로에서 정확한
                        // gap 위치 (text 사이) 에 emit 한다. 본 두번째 loop 는
                        // paragraph_layout 미지원 이전의 legacy fallback. paragraph_layout
                        // 가 이미 inline_shape_position 으로 등록한 경우 중복 emit 차단.
                        // (시험지 page 2 문14 <보기> textbox 의 6개 inline 수식이
                        //  paragraph_layout + 본 분기 양쪽에서 각각 emit → 중복).
                        let equiv_cell_ctx = CellContext {
                            parent_para_index: para_index,
                            path: {
                                let mut p = parent_cell_path.to_vec();
                                p.push(CellPathEntry {
                                    control_index,
                                    cell_index: 0,
                                    cell_para_index: pi,
                                    text_direction: 0,
                                });
                                p
                            },
                        };
                        if tree
                            .get_inline_shape_position(
                                section_index,
                                pi,
                                ctrl_idx_in_para,
                                Some(&equiv_cell_ctx),
                            )
                            .is_some()
                        {
                            // paragraph_layout 가 이미 emit — inline_x 만 advance
                            inline_x += eq_w;
                        } else {
                            let (eq_x, eq_y) = {
                                let x = inline_x;
                                inline_x += eq_w;
                                (x, para_start_y)
                            };

                            let tokens = super::super::equation::tokenizer::tokenize(&eq.script);
                            let ast = super::super::equation::parser::EqParser::new(tokens).parse();
                            let font_size_px = hwpunit_to_px(eq.font_size as i32, self.dpi);
                            let layout_box =
                                super::super::equation::layout::EqLayout::new(font_size_px)
                                    .layout(&ast);
                            let color_str =
                                super::super::equation::svg_render::eq_color_to_svg(eq.color);
                            let svg_content =
                                super::super::equation::svg_render::render_equation_svg(
                                    &layout_box,
                                    &color_str,
                                    font_size_px,
                                );

                            let eq_node = RenderNode::new(
                                tree.next_id(),
                                RenderNodeType::Equation(EquationNode {
                                    svg_content,
                                    layout_box,
                                    color_str,
                                    color: eq.color,
                                    font_size: font_size_px,
                                    section_index: Some(section_index),
                                    para_index: Some(para_index),
                                    control_index: Some(ctrl_idx_in_para),
                                    cell_index: None,
                                    cell_para_index: None,
                                    note_ref: None,
                                }),
                                BoundingBox::new(eq_x, eq_y, eq_w, eq_h),
                            );
                            textbox_node.children.push(eq_node);
                        }
                    }
                    Control::Table(table) => {
                        // TextBox 내 인라인 표 렌더링
                        // 현재 글상자의 경로 + 이 문단의 표 컨트롤 인덱스를 전달
                        let mut table_enclosing_path = parent_cell_path.to_vec();
                        table_enclosing_path.push(CellPathEntry {
                            control_index,
                            cell_index: 0,
                            cell_para_index: pi,
                            text_direction: 0,
                        });
                        // 호스트 문단의 정렬 속성
                        let host_align = styles
                            .para_styles
                            .get(para.para_shape_id as usize)
                            .map(|ps| ps.alignment)
                            .unwrap_or(Alignment::Left);
                        inline_y = self.layout_embedded_table(
                            tree,
                            &mut textbox_node,
                            table,
                            styles,
                            &inner_area,
                            para_start_y,
                            Some((
                                section_index,
                                para_index,
                                &table_enclosing_path,
                                ctrl_idx_in_para,
                            )),
                            bin_data_content,
                            host_align,
                        );
                    }
                    _ => {}
                }
            }
            // 인라인 컨트롤의 최대 높이만큼 y 전진
            if max_inline_height > 0.0 {
                inline_y += max_inline_height;
            }
        }
        if !textbox_node.children.is_empty() {
            if parent_treat_as_char {
                Self::expand_inline_textbox_to_content(
                    shape_node,
                    &mut textbox_node,
                    margin_bottom,
                );
            }
            shape_node.children.push(textbox_node);
        }
    }

    /// 글상자 세로쓰기 레이아웃
    ///
    /// 텍스트 방향: 위→아래, 열(column)은 오른쪽→왼쪽
    /// text_direction: 1=영문 눕힘(회전), 2=영문 세움(직립)
    /// 정렬 매핑: Top→오른쪽(첫 열), Center→중앙, Bottom→왼쪽
    #[allow(clippy::too_many_arguments)]
    fn layout_vertical_textbox_text_with_paras(
        &self,
        tree: &mut PageRenderTree,
        shape_node: &mut RenderNode,
        paragraphs: &[Paragraph],
        text_box: &crate::model::shape::TextBox,
        styles: &ResolvedStyleSet,
        inner_area: &LayoutRect,
        text_direction: u8,
        section_index: usize,
        para_index: usize,
        control_index: usize,
        parent_cell_path: &[CellPathEntry],
    ) {
        use super::super::composer::compose_paragraph;

        // 1. line_seg 기반으로 composed lines를 열(column)로 변환
        struct CharInfo {
            ch: char,
            style: TextStyle,
            char_style_id: u32,
            para_style_id: u16,
            cell_para_index: usize,
            char_offset: usize,
            is_para_end: bool,
        }

        struct ColumnInfo {
            start_idx: usize,
            end_idx: usize,
            col_width: f64, // line_height + line_spacing (px), 마지막 칼럼은 line_height만
            col_spacing: f64, // 항상 0 (line_spacing이 col_width에 흡수됨)
            total_height: f64,
            alignment: Alignment,
            absorbed_spacing: f64, // 흡수된 line_spacing (px) — 마지막 칼럼 후처리용
        }

        let composed_paras: Vec<_> = paragraphs.iter().map(|p| compose_paragraph(p)).collect();

        let get_alignment = |para_style_id: u16| -> Alignment {
            styles
                .para_styles
                .get(para_style_id as usize)
                .map(|s| s.alignment)
                .unwrap_or(Alignment::Left)
        };

        let mut chars: Vec<CharInfo> = Vec::new();
        let mut columns: Vec<ColumnInfo> = Vec::new();

        for (cp_idx, composed) in composed_paras.iter().enumerate() {
            let para = &paragraphs[cp_idx];
            let alignment = get_alignment(composed.para_style_id);

            if composed.lines.is_empty() {
                // 빈 문단: 빈 열 추가
                // 칼럼 너비 = line_height + line_spacing (전체 피치를 칼럼에 흡수)
                let ls = para.line_segs.first();
                let spacing = ls
                    .map(|l| hwpunit_to_px(l.line_spacing, self.dpi))
                    .unwrap_or(0.0);
                columns.push(ColumnInfo {
                    start_idx: chars.len(),
                    end_idx: chars.len(),
                    col_width: ls
                        .map(|l| hwpunit_to_px(l.line_height + l.line_spacing, self.dpi))
                        .unwrap_or(13.0),
                    col_spacing: 0.0,
                    total_height: 0.0,
                    alignment,
                    absorbed_spacing: spacing,
                });
                continue;
            }

            let mut char_offset = 0usize;
            for (line_idx, line) in composed.lines.iter().enumerate() {
                let ls = para.line_segs.get(line_idx);
                // 칼럼 너비 = line_height + line_spacing (전체 피치 흡수)
                let col_width = ls
                    .map(|l| hwpunit_to_px(l.line_height + l.line_spacing, self.dpi))
                    .unwrap_or(13.0);
                let col_spacing = 0.0;
                let absorbed_spacing = ls
                    .map(|l| hwpunit_to_px(l.line_spacing, self.dpi))
                    .unwrap_or(0.0);

                let col_start = chars.len();
                let mut col_height = 0.0;

                for run in &line.runs {
                    let text_style =
                        resolved_to_text_style(styles, run.char_style_id, run.lang_index);
                    for ch in run.text.chars() {
                        if ch == '\n' || ch == '\r' {
                            char_offset += 1;
                            continue;
                        }
                        let is_rotate = is_vertical_rotate_char(ch);
                        let needs_rotation = is_rotate || (text_direction == 1 && !is_cjk_char(ch));
                        // 세로쓰기에서 구두점/기호만 반칸 advance (영문/숫자는 캐릭터 높이)
                        let half_advance =
                            needs_rotation || (!is_cjk_char(ch) && !ch.is_ascii_alphanumeric());
                        let advance = if half_advance {
                            text_style.font_size * 0.5
                        } else {
                            text_style.font_size
                        };
                        chars.push(CharInfo {
                            ch,
                            style: text_style.clone(),
                            char_style_id: run.char_style_id,
                            para_style_id: composed.para_style_id,
                            cell_para_index: cp_idx,
                            char_offset,
                            is_para_end: false,
                        });
                        col_height += advance;
                        char_offset += 1;
                    }
                }

                if line_idx == composed.lines.len() - 1 {
                    if let Some(last) = chars.last_mut() {
                        if last.cell_para_index == cp_idx {
                            last.is_para_end = true;
                        }
                    }
                }

                columns.push(ColumnInfo {
                    start_idx: col_start,
                    end_idx: chars.len(),
                    col_width,
                    col_spacing,
                    total_height: col_height,
                    alignment,
                    absorbed_spacing,
                });
            }
        }

        if chars.is_empty() && columns.iter().all(|c| c.start_idx == c.end_idx) {
            return;
        }

        // 마지막 칼럼은 뒤에 간격이 불필요하므로 흡수된 line_spacing분 제거
        if let Some(last_col) = columns.last_mut() {
            last_col.col_width -= last_col.absorbed_spacing;
        }

        // 2. 열 배치 x좌표 계산 (오른쪽→왼쪽)
        let total_cols_width: f64 = if columns.is_empty() {
            0.0
        } else {
            columns.iter().map(|c| c.col_width).sum::<f64>()
                + columns[..columns.len() - 1]
                    .iter()
                    .map(|c| c.col_spacing)
                    .sum::<f64>()
        };

        use crate::model::table::VerticalAlign;
        // 열이 셀보다 넓으면 첫 열이 오른쪽 가장자리에서 시작하도록 클램핑
        let right_aligned = inner_area.x + inner_area.width - total_cols_width;
        let cols_x_start = match text_box.vertical_align {
            VerticalAlign::Top => right_aligned,
            VerticalAlign::Center => {
                let centered = inner_area.x + (inner_area.width - total_cols_width) / 2.0;
                centered.min(right_aligned)
            }
            VerticalAlign::Bottom => inner_area.x.min(right_aligned),
        };

        // 3. 각 글자를 TextLine + TextRun 노드로 생성
        let mut col_x = cols_x_start + total_cols_width;

        for col in &columns {
            col_x -= col.col_width;

            let free_space = (inner_area.height - col.total_height).max(0.0);
            let y_start = inner_area.y
                + match col.alignment {
                    Alignment::Center | Alignment::Distribute => free_space / 2.0,
                    Alignment::Right => free_space,
                    _ => 0.0,
                };
            let mut char_y = y_start;
            let col_bottom = inner_area.y + inner_area.height;

            for i in col.start_idx..col.end_idx {
                let ci = &chars[i];
                let is_rotate = is_vertical_rotate_char(ci.ch);
                let needs_rotation = is_rotate || (text_direction == 1 && !is_cjk_char(ci.ch));
                // 세로쓰기에서 구두점/기호만 반칸 advance (영문/숫자는 캐릭터 높이)
                let half_advance =
                    needs_rotation || (!is_cjk_char(ci.ch) && !ci.ch.is_ascii_alphanumeric());
                let advance = if half_advance {
                    ci.style.font_size * 0.5
                } else {
                    ci.style.font_size
                };

                // 열 높이 초과 시 렌더링 중단
                if char_y + advance > col_bottom + 0.5 {
                    break;
                }

                let char_width = if is_cjk_char(ci.ch) || is_rotate {
                    ci.style.font_size
                } else if needs_rotation {
                    ci.style.font_size
                } else {
                    ci.style.font_size * 0.5
                };

                let char_x = col_x + (col.col_width - char_width) / 2.0;
                // 기호 대체: 세로 형태 Unicode가 있으면 대체 문자를 사용 (회전 불필요)
                let (render_ch, rotation) = if needs_rotation {
                    if let Some(sub) = vertical_substitute_char(ci.ch) {
                        (sub, 0.0)
                    } else {
                        (ci.ch, 90.0)
                    }
                } else {
                    (ci.ch, 0.0)
                };

                let cell_ctx = CellContext {
                    parent_para_index: para_index,
                    path: {
                        let mut p = parent_cell_path.to_vec();
                        p.push(CellPathEntry {
                            control_index,
                            cell_index: 0,
                            cell_para_index: ci.cell_para_index,
                            text_direction,
                        });
                        p
                    },
                };

                let line_id = tree.next_id();
                let mut line_node = RenderNode::new(
                    line_id,
                    RenderNodeType::TextLine(TextLineNode::new(advance, advance * 0.85)),
                    BoundingBox::new(char_x, char_y, char_width, advance),
                );

                let run_id = tree.next_id();
                let run_node = RenderNode::new(
                    run_id,
                    RenderNodeType::TextRun(TextRunNode {
                        text: render_ch.to_string(),
                        style: ci.style.clone(),
                        char_shape_id: Some(ci.char_style_id),
                        para_shape_id: Some(ci.para_style_id),
                        section_index: Some(section_index),
                        para_index: Some(ci.cell_para_index),
                        char_start: Some(ci.char_offset),
                        cell_context: Some(cell_ctx),
                        is_para_end: ci.is_para_end,
                        is_line_break_end: false,
                        rotation,
                        is_vertical: true,
                        char_overlap: None,
                        border_fill_id: styles
                            .char_styles
                            .get(ci.char_style_id as usize)
                            .map(|cs| cs.border_fill_id)
                            .unwrap_or(0),
                        baseline: advance * 0.85,
                        field_marker: FieldMarkerType::None,
                    }),
                    BoundingBox::new(char_x, char_y, char_width, advance),
                );

                line_node.children.push(run_node);
                shape_node.children.push(line_node);

                char_y += advance;
            }

            col_x -= col.col_spacing;
        }
    }

    pub(crate) fn curve_to_path_commands_scaled(
        &self,
        curve: &crate::model::shape::CurveShape,
        base_x: f64,
        base_y: f64,
        sx: f64,
        sy: f64,
    ) -> Vec<PathCommand> {
        let mut commands = Vec::new();
        if curve.points.is_empty() {
            return commands;
        }

        let pts: Vec<(f64, f64)> = curve
            .points
            .iter()
            .map(|p| {
                (
                    base_x + hwpunit_to_px(p.x, self.dpi) * sx,
                    base_y + hwpunit_to_px(p.y, self.dpi) * sy,
                )
            })
            .collect();

        commands.push(PathCommand::MoveTo(pts[0].0, pts[0].1));

        let mut i = 1;
        let mut seg_idx = 0;
        while i < pts.len() {
            let seg_type = curve.segment_types.get(seg_idx).copied().unwrap_or(0);
            if seg_type == 1 && i + 2 < pts.len() {
                // 베지어 곡선: 제어점 2개 + 끝점 1개
                commands.push(PathCommand::CurveTo(
                    pts[i].0,
                    pts[i].1,
                    pts[i + 1].0,
                    pts[i + 1].1,
                    pts[i + 2].0,
                    pts[i + 2].1,
                ));
                i += 3;
            } else {
                // 직선
                commands.push(PathCommand::LineTo(pts[i].0, pts[i].1));
                i += 1;
            }
            seg_idx += 1;
        }

        commands
    }

    /// TopAndBottom 모드 글상자들의 앵커 문단별 예약 높이 계산
    /// 각 (앵커 문단 인덱스, 글상자 하단 y) 쌍을 반환
    /// 앵커 문단 이전 문단은 글상자 위에, 앵커 문단부터는 글상자 아래에 배치됨
    pub(crate) fn calculate_shape_reserved_heights(
        &self,
        paragraphs: &[Paragraph],
        items: &[PageItem],
        col_area: &LayoutRect,
        body_area: &LayoutRect,
    ) -> Vec<(usize, f64)> {
        use crate::model::shape::TextWrap;

        let mut result: Vec<(usize, f64)> = Vec::new();

        for item in items {
            let (para_index, control_index) = match item {
                PageItem::Shape {
                    para_index,
                    control_index,
                } => (para_index, control_index),
                PageItem::Table {
                    para_index,
                    control_index,
                } => (para_index, control_index),
                _ => continue,
            };
            {
                let para = match paragraphs.get(*para_index) {
                    Some(p) => p,
                    None => continue,
                };
                let ctrl = match para.controls.get(*control_index) {
                    Some(c) => c,
                    None => continue,
                };
                let common = match ctrl {
                    Control::Shape(s) => s.common(),
                    Control::Table(t) if !t.common.treat_as_char => &t.common,
                    Control::Picture(p) if !p.common.treat_as_char => &p.common,
                    _ => continue,
                };

                // TopAndBottom 모드만 본문 밀어내기 처리
                if !matches!(common.text_wrap, TextWrap::TopAndBottom) {
                    continue;
                }

                // 글자처럼 취급(인라인)은 LineSeg가 이미 높이를 포함하므로 예약 불필요
                if common.treat_as_char {
                    continue;
                }

                // vert=Para: 문단 상대 위치 표/도형은 shape_reserved 제외
                // 이 개체들은 앵커 문단의 y_offset에 따라 배치되므로
                // 미리 공간을 예약하면 y_offset이 이중으로 밀려남
                if matches!(common.vert_rel_to, crate::model::shape::VertRelTo::Para) {
                    continue;
                }

                // 수평 겹침 확인
                if !self.check_horizontal_overlap(common, col_area, body_area) {
                    continue;
                }

                // 하단 y 좌표 계산
                // Table: common.height는 실제 렌더링 높이(셀 콘텐츠 포함)보다 작을 수 있음
                // 셀 내 LINE_SEG 기반 실제 높이를 사용하여 보정
                let effective_common = if let Control::Table(t) = ctrl {
                    let measured_h = self.measure_table_actual_height(t);
                    if measured_h > common.height {
                        let mut c = common.clone();
                        c.height = measured_h;
                        Some(c)
                    } else {
                        None
                    }
                } else {
                    None
                };
                let effective_ref = effective_common.as_ref().unwrap_or(common);
                let (bottom_y, shape_y) =
                    self.calc_shape_bottom_y(effective_ref, col_area, body_area);

                // 본문 시작 근처만 고려 (페이지 하단 개체는 제외)
                let threshold_y = col_area.y + col_area.height / 3.0;
                if shape_y > threshold_y {
                    continue;
                }

                // 같은 앵커 문단에 여러 글상자가 있으면 최대 하단 y 사용
                if let Some(existing) = result.iter_mut().find(|(pi, _)| *pi == *para_index) {
                    if bottom_y > existing.1 {
                        existing.1 = bottom_y;
                    }
                } else {
                    result.push((*para_index, bottom_y));
                }
            }
        }

        result
    }

    /// TopAndBottom 개체의 하단 y 좌표와 상단 y 좌표를 계산 (pub(crate) — Task #901 Stage 8 flow-around)
    pub(crate) fn calc_shape_bottom_y(
        &self,
        common: &CommonObjAttr,
        col_area: &LayoutRect,
        body_area: &LayoutRect,
    ) -> (f64, f64) {
        use crate::model::shape::{VertAlign, VertRelTo};
        let v_offset = hwpunit_to_px(common.vertical_offset as i32, self.dpi);
        let shape_h = hwpunit_to_px(common.height as i32, self.dpi);
        let (ref_y, ref_h) = match common.vert_rel_to {
            VertRelTo::Paper => (0.0_f64, body_area.y + body_area.height + body_area.y),
            VertRelTo::Page => (body_area.y, body_area.height),
            VertRelTo::Para => (col_area.y, col_area.height),
        };
        let shape_y = match common.vert_align {
            VertAlign::Top | VertAlign::Inside => ref_y + v_offset,
            VertAlign::Center => ref_y + (ref_h - shape_h) / 2.0 + v_offset,
            VertAlign::Bottom | VertAlign::Outside => ref_y + ref_h - shape_h - v_offset,
        };
        // 바깥 여백(margin.bottom) 포함 — 본문이 여백 이후부터 시작
        let margin_bottom = hwpunit_to_px(common.margin.bottom as i32, self.dpi);
        (shape_y + shape_h + margin_bottom, shape_y)
    }

    /// 다단 레이아웃에서 body_area 전체에 걸치는 TopAndBottom 개체의 예약 높이 계산
    /// 모든 단의 items를 순회하여 body_area 너비와 동일하거나 큰 개체만 반환
    pub(crate) fn calculate_body_wide_shape_reserved(
        &self,
        paragraphs: &[Paragraph],
        column_contents: &[super::super::pagination::ColumnContent],
        body_area: &LayoutRect,
    ) -> Vec<(usize, f64)> {
        use crate::model::shape::TextWrap;

        let mut result: Vec<(usize, f64)> = Vec::new();

        for col_content in column_contents {
            for item in &col_content.items {
                let (para_index, control_index) = match item {
                    super::super::pagination::PageItem::Shape {
                        para_index,
                        control_index,
                    } => (para_index, control_index),
                    super::super::pagination::PageItem::Table {
                        para_index,
                        control_index,
                    } => (para_index, control_index),
                    _ => continue,
                };
                let para = match paragraphs.get(*para_index) {
                    Some(p) => p,
                    None => continue,
                };
                let ctrl = match para.controls.get(*control_index) {
                    Some(c) => c,
                    None => continue,
                };
                let common = match ctrl {
                    Control::Shape(s) => s.common(),
                    Control::Table(t) if !t.common.treat_as_char => &t.common,
                    Control::Picture(p) if !p.common.treat_as_char => &p.common,
                    _ => continue,
                };
                if !matches!(common.text_wrap, TextWrap::TopAndBottom) || common.treat_as_char {
                    continue;
                }
                // Task #321 v3 정밀화 (#326): Paper(용지) 기준 도형 중 본문과 겹치지 않는
                // (= 머리말 영역에만 위치하는) 도형만 col 1+ reserve 에서 제외.
                // 본문과 겹치는 Paper 도형은 col 1 시작에 영향 → 제외 안 함.
                if matches!(common.vert_rel_to, crate::model::shape::VertRelTo::Paper) {
                    let shape_top = hwpunit_to_px(common.vertical_offset as i32, self.dpi);
                    let shape_bottom = shape_top + hwpunit_to_px(common.height as i32, self.dpi);
                    if shape_bottom <= body_area.y {
                        continue;
                    }
                }
                // body_area 너비의 80% 이상 차지하는 개체만 (2단에 걸치는 개체)
                let shape_w = hwpunit_to_px(common.width as i32, self.dpi);
                if shape_w < body_area.width * 0.8 {
                    continue;
                }
                let (bottom_y, shape_y) = self.calc_shape_bottom_y(common, body_area, body_area);
                let threshold_y = body_area.y + body_area.height / 3.0;
                if shape_y > threshold_y {
                    continue;
                }
                if let Some(existing) = result.iter_mut().find(|(pi, _)| *pi == *para_index) {
                    if bottom_y > existing.1 {
                        existing.1 = bottom_y;
                    }
                } else {
                    result.push((*para_index, bottom_y));
                }
            }
        }

        result
    }

    /// 표의 실제 렌더링 높이를 계산 (셀 콘텐츠 + 패딩 포함)
    fn measure_table_actual_height(&self, table: &crate::model::table::Table) -> u32 {
        let row_count = table.row_count as usize;
        if row_count == 0 {
            return table.common.height;
        }
        let mut row_heights = vec![0u32; row_count];
        for cell in &table.cells {
            if cell.row_span == 1 && (cell.row as usize) < row_count {
                let r = cell.row as usize;
                if cell.height < 0x80000000 && cell.height > row_heights[r] {
                    row_heights[r] = cell.height;
                }
            }
        }
        // 셀 내 LINE_SEG 기반 콘텐츠 높이로 보정
        for cell in &table.cells {
            if cell.row_span == 1 && (cell.row as usize) < row_count {
                let r = cell.row as usize;
                let (pad_top, pad_bottom) = if cell.apply_inner_margin {
                    (cell.padding.top as u32, cell.padding.bottom as u32)
                } else {
                    (table.padding.top as u32, table.padding.bottom as u32)
                };
                let content_h: i32 = cell
                    .paragraphs
                    .iter()
                    .flat_map(|p| p.line_segs.last())
                    .map(|s| s.vertical_pos + s.line_height)
                    .max()
                    .unwrap_or(0);
                let required = content_h as u32 + pad_top + pad_bottom;
                if required > row_heights[r] {
                    row_heights[r] = required;
                }
            }
        }
        let total: u32 = row_heights.iter().sum();
        total.max(table.common.height)
    }

    /// 개체가 단 영역과 수평으로 겹치는지 확인
    fn check_horizontal_overlap(
        &self,
        common: &CommonObjAttr,
        col_area: &LayoutRect,
        body_area: &LayoutRect,
    ) -> bool {
        use crate::model::shape::{HorzAlign, HorzRelTo};
        let h_offset = hwpunit_to_px(common.horizontal_offset as i32, self.dpi);
        let shape_w = hwpunit_to_px(common.width as i32, self.dpi);
        let (ref_x, ref_w) = match common.horz_rel_to {
            HorzRelTo::Paper => (0.0_f64, body_area.x + body_area.width + body_area.x),
            HorzRelTo::Page => (body_area.x, body_area.width),
            HorzRelTo::Column => (col_area.x, col_area.width),
            _ => (col_area.x, col_area.width),
        };
        let shape_x = match common.horz_align {
            HorzAlign::Left | HorzAlign::Inside => ref_x + h_offset,
            HorzAlign::Center => ref_x + (ref_w - shape_w) / 2.0 + h_offset,
            HorzAlign::Right | HorzAlign::Outside => ref_x + ref_w - shape_w - h_offset,
        };
        let shape_right = shape_x + shape_w;
        let col_right = col_area.x + col_area.width;
        shape_right >= col_area.x && shape_x <= col_right
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::paragraph::{CharShapeRef, LineSeg};
    use crate::renderer::style_resolver::{ResolvedCharStyle, ResolvedParaStyle};

    fn line_seg(text_start: u32, vertical_pos: i32) -> LineSeg {
        LineSeg {
            text_start,
            vertical_pos,
            line_height: 2000,
            text_height: 2000,
            baseline_distance: 1700,
            line_spacing: 1200,
            segment_width: 16856,
            tag: LineSeg::TAG_SINGLE_SEGMENT_LINE,
            ..Default::default()
        }
    }

    #[test]
    fn matrix_textbox_para_collapses_imported_lines_that_overflow_height() {
        let mut para = Paragraph {
            char_count: 2,
            text: "AB".to_string(),
            char_offsets: vec![0, 1],
            char_shapes: vec![CharShapeRef {
                start_pos: 0,
                char_shape_id: 0,
            }],
            line_segs: vec![line_seg(0, 0), line_seg(1, 3200)],
            ..Default::default()
        };
        let mut styles = ResolvedStyleSet::default();
        styles.char_styles.push(ResolvedCharStyle {
            font_family: "Arial".to_string(),
            font_families: vec!["Arial".to_string(); 7],
            font_size: 20.0,
            ..Default::default()
        });
        styles.para_styles.push(ResolvedParaStyle::default());

        reflow_matrix_textbox_para(&mut para, 80.0, 30.0, &styles, 96.0);

        assert_eq!(para.line_segs.len(), 1);
        assert_eq!(para.line_segs[0].text_start, 0);
        assert_eq!(para.line_segs[0].segment_width, px_to_hwpunit(80.0, 96.0));
    }
}

//! 그림/캡션 레이아웃 + 각주 영역 레이아웃

use crate::model::paragraph::Paragraph;
use crate::model::style::Alignment;
use crate::model::shape::{Caption, CaptionDirection, CommonObjAttr, HorzAlign, HorzRelTo, TextWrap, VertAlign, VertRelTo};
use crate::model::footnote::{FootnoteShape, NumberFormat};
use super::super::pagination::{FootnoteRef, FootnoteSource};
use crate::model::control::Control;
use crate::model::bin_data::BinDataContent;
use super::super::render_tree::*;
use super::super::page_layout::LayoutRect;
use super::super::composer::{compose_paragraph, ComposedParagraph};
use super::super::style_resolver::ResolvedStyleSet;
use super::super::{hwpunit_to_px, StrokeDash, LineStyle, TextStyle, AutoNumberCounter, format_number, NumberFormat as NumFmt};
use super::LayoutEngine;
use super::border_rendering::border_width_to_px;
use super::utils::find_bin_data;
use super::text_measurement::{resolved_to_text_style, estimate_text_width};

impl LayoutEngine {
    pub(crate) fn layout_picture(
        &self,
        tree: &mut PageRenderTree,
        parent_node: &mut RenderNode,
        picture: &crate::model::image::Picture,
        container: &LayoutRect,
        bin_data_content: &[BinDataContent],
        alignment: Alignment,
        section_index: Option<usize>,
        para_index: Option<usize>,
        control_index: Option<usize>,
    ) {
        // 그림 크기 (HWPUNIT → 픽셀)
        // CommonObjAttr의 width/height가 개체의 실제 표시 크기
        let mut pic_width = hwpunit_to_px(picture.common.width as i32, self.dpi);
        let mut pic_height = hwpunit_to_px(picture.common.height as i32, self.dpi);

        // 컨테이너 초과 시 비율 유지하며 축소 (표 셀 등)
        if container.width > 0.0 && pic_width > container.width {
            let scale = container.width / pic_width;
            pic_width = container.width;
            pic_height *= scale;
        }
        if container.height > 0.0 && pic_height > container.height {
            let scale = container.height / pic_height;
            pic_height = container.height;
            pic_width *= scale;
        }

        // 그림 위치: non-TAC 이미지는 common 속성의 offset 적용
        // 머리말/꼬리말에서 vert=Paper는 상단여백(header area) 기준
        let (pic_x, pic_y) = if !picture.common.treat_as_char {
            let h_offset = hwpunit_to_px(picture.common.horizontal_offset as i32, self.dpi);
            let v_offset = hwpunit_to_px(picture.common.vertical_offset as i32, self.dpi);
            let x = match picture.common.horz_align {
                HorzAlign::Left | HorzAlign::Inside => container.x + h_offset,
                HorzAlign::Center => container.x + (container.width - pic_width) / 2.0 + h_offset,
                HorzAlign::Right | HorzAlign::Outside => container.x + container.width - pic_width - h_offset,
            };
            let y = match picture.common.vert_align {
                VertAlign::Top | VertAlign::Inside => container.y + v_offset,
                VertAlign::Center => container.y + (container.height - pic_height) / 2.0 + v_offset,
                VertAlign::Bottom | VertAlign::Outside => container.y + container.height - pic_height - v_offset,
            };
            (x, y)
        } else {
            let x = match alignment {
                Alignment::Center | Alignment::Distribute => {
                    container.x + (container.width - pic_width).max(0.0) / 2.0
                }
                Alignment::Right => {
                    container.x + (container.width - pic_width).max(0.0)
                }
                _ => container.x,
            };
            (x, container.y)
        };

        // BinData에서 이미지 데이터 찾기 (bin_data_id는 1-indexed 순번)
        let bin_data_id = picture.image_attr.bin_data_id;
        let image_data = find_bin_data(bin_data_content, bin_data_id)
            .map(|c| c.data.clone());

        // 그림 자르기: crop 좌표를 그대로 저장 (렌더러에서 이미지 px 크기와 비교)
        let crop = {
            let c = &picture.crop;
            if c.right > c.left && c.bottom > c.top && (c.left != 0 || c.top != 0 || c.right != 0 || c.bottom != 0) {
                Some((c.left, c.top, c.right, c.bottom))
            } else {
                None
            }
        };

        // 원본 이미지 크기(HU) — crop 좌표 보정용
        let original_size_hu = if picture.shape_attr.original_width > 0
            && picture.shape_attr.original_height > 0
        {
            Some((picture.shape_attr.original_width, picture.shape_attr.original_height))
        } else {
            None
        };

        // 이미지 노드 생성
        let img_id = tree.next_id();
        let img_node = RenderNode::new(
            img_id,
            RenderNodeType::Image(ImageNode {
                section_index,
                para_index,
                control_index,
                crop,
                original_size_hu,
                effect: picture.image_attr.effect,
                brightness: picture.image_attr.brightness,
                contrast: picture.image_attr.contrast,
                ..ImageNode::new(bin_data_id, image_data)
            }),
            BoundingBox::new(pic_x, pic_y, pic_width, pic_height),
        );

        parent_node.children.push(img_node);

        // 그림 테두리(선) 렌더링
        self.render_picture_border(tree, parent_node, picture, pic_x, pic_y, pic_width, pic_height);
    }

    /// 개체(Picture/Shape)의 절대 좌표 (x, y)를 계산한다.
    /// HWP 스펙에 따른 VertRelTo/HorzRelTo/VertAlign/HorzAlign/treat_as_char 처리를 통합한 단일 함수.
    /// 상대 크기(WidthCriterion/HeightCriterion)를 적용하여 실제 px 크기를 반환한다.
    pub(crate) fn resolve_object_size(
        &self,
        common: &CommonObjAttr,
        col_area: &LayoutRect,
        body_area: &LayoutRect,
        paper_area: &LayoutRect,
    ) -> (f64, f64) {
        use crate::model::shape::SizeCriterion;

        let raw_w = common.width as f64;
        let raw_h = common.height as f64;

        let obj_width = match common.width_criterion {
            SizeCriterion::Absolute => hwpunit_to_px(common.width as i32, self.dpi),
            SizeCriterion::Paper => paper_area.width * raw_w / 10000.0,
            SizeCriterion::Page => body_area.width * raw_w / 10000.0,
            SizeCriterion::Column => col_area.width * raw_w / 10000.0,
            SizeCriterion::Para => col_area.width * raw_w / 10000.0,
        };

        let obj_height = match common.height_criterion {
            SizeCriterion::Absolute => hwpunit_to_px(common.height as i32, self.dpi),
            SizeCriterion::Paper => paper_area.height * raw_h / 10000.0,
            SizeCriterion::Page => body_area.height * raw_h / 10000.0,
            _ => hwpunit_to_px(common.height as i32, self.dpi),
        };

        (obj_width, obj_height)
    }

    pub(crate) fn compute_object_position(
        &self,
        common: &CommonObjAttr,
        obj_width: f64,
        obj_height: f64,
        container: &LayoutRect,
        col_area: &LayoutRect,
        body_area: &LayoutRect,
        paper_area: &LayoutRect,
        para_y: f64,
        alignment: Alignment,
    ) -> (f64, f64) {
        let h_offset = hwpunit_to_px(common.horizontal_offset as i32, self.dpi);
        let v_offset = hwpunit_to_px(common.vertical_offset as i32, self.dpi);

        let x = if common.treat_as_char {
            match alignment {
                Alignment::Center | Alignment::Distribute => {
                    container.x + (container.width - obj_width).max(0.0) / 2.0
                }
                Alignment::Right => {
                    container.x + (container.width - obj_width).max(0.0)
                }
                _ => container.x,
            }
        } else {
            // 가로 기준 영역 결정
            let (ref_x, ref_w) = match common.horz_rel_to {
                HorzRelTo::Paper => (paper_area.x, paper_area.width),
                HorzRelTo::Page => (body_area.x, body_area.width),
                HorzRelTo::Column => (col_area.x, col_area.width),
                HorzRelTo::Para => (container.x, container.width),
            };
            // 가로 정렬 방식 적용
            match common.horz_align {
                HorzAlign::Left | HorzAlign::Inside => ref_x + h_offset,
                HorzAlign::Center => ref_x + (ref_w - obj_width) / 2.0 + h_offset,
                HorzAlign::Right | HorzAlign::Outside => ref_x + ref_w - obj_width - h_offset,
            }
        };

        let y = if common.treat_as_char {
            para_y
        } else {
            // 세로 기준 영역 결정
            let (ref_y, ref_h) = match common.vert_rel_to {
                VertRelTo::Paper => (paper_area.y, paper_area.height),
                VertRelTo::Page => (body_area.y, body_area.height),
                VertRelTo::Para => (para_y, container.height),
            };
            // 세로 정렬 방식 적용
            match common.vert_align {
                VertAlign::Top | VertAlign::Inside => ref_y + v_offset,
                VertAlign::Center => ref_y + (ref_h - obj_height) / 2.0 + v_offset,
                VertAlign::Bottom | VertAlign::Outside => ref_y + ref_h - obj_height - v_offset,
            }
        };

        (x, y)
    }

    /// 본문 그림(Picture) 개체를 레이아웃하고 업데이트된 y_offset을 반환한다.
    pub(crate) fn layout_body_picture(
        &self,
        tree: &mut PageRenderTree,
        parent_node: &mut RenderNode,
        picture: &crate::model::image::Picture,
        container: &LayoutRect,
        col_area: &LayoutRect,
        body_area: &LayoutRect,
        paper_area: &LayoutRect,
        bin_data_content: &[BinDataContent],
        styles: &ResolvedStyleSet,
        alignment: Alignment,
        y_offset: f64,
        section_index: usize,
        para_index: usize,
        control_index: usize,
    ) -> f64 {
        // 그림 크기 (HWPUNIT → 픽셀)
        let pic_width = hwpunit_to_px(picture.common.width as i32, self.dpi);
        let pic_height = hwpunit_to_px(picture.common.height as i32, self.dpi);

        // 캡션 높이 및 간격 계산
        let caption_height = self.calculate_caption_height(&picture.caption, styles);
        let caption_spacing = if let Some(ref caption) = picture.caption {
            hwpunit_to_px(caption.spacing as i32, self.dpi)
        } else {
            0.0
        };

        // 캡션을 포함한 전체 크기 계산 (위치 결정에 사용)
        let (total_width, total_height) = if let Some(ref caption) = picture.caption {
            match caption.direction {
                CaptionDirection::Top | CaptionDirection::Bottom => {
                    (pic_width, pic_height + caption_height + caption_spacing)
                }
                CaptionDirection::Left | CaptionDirection::Right => {
                    let cw = hwpunit_to_px(caption.width as i32, self.dpi);
                    (pic_width + cw + caption_spacing, pic_height)
                }
            }
        } else {
            (pic_width, pic_height)
        };

        // 통합 좌표 계산 (캡션 포함 전체 크기 기준)
        let (pic_x, base_y) = self.compute_object_position(
            &picture.common, total_width, total_height, container, col_area, body_area, paper_area, y_offset, alignment,
        );

        // 캡션 방향에 따라 그림 위치 오프셋 계산
        let (caption_top_offset, caption_left_offset) = if let Some(ref caption) = picture.caption {
            match caption.direction {
                CaptionDirection::Top => (caption_height + caption_spacing, 0.0),
                CaptionDirection::Left => {
                    let cw = hwpunit_to_px(caption.width as i32, self.dpi);
                    (0.0, cw + caption_spacing)
                }
                _ => (0.0, 0.0),
            }
        } else {
            (0.0, 0.0)
        };

        let adjusted_pic_x = pic_x + caption_left_offset;
        let pic_y = base_y + caption_top_offset;

        // BinData에서 이미지 데이터 찾기 (bin_data_id는 1-indexed 순번)
        let bin_data_id = picture.image_attr.bin_data_id;
        let image_data = find_bin_data(bin_data_content, bin_data_id)
            .map(|c| c.data.clone());

        // 그림 자르기
        let crop = {
            let c = &picture.crop;
            if c.right > c.left && c.bottom > c.top {
                Some((c.left, c.top, c.right, c.bottom))
            } else {
                None
            }
        };

        // 원본 이미지 크기(HU)
        let original_size_hu = if picture.shape_attr.original_width > 0
            && picture.shape_attr.original_height > 0
        {
            Some((picture.shape_attr.original_width, picture.shape_attr.original_height))
        } else {
            None
        };

        // 이미지 노드 생성
        let img_id = tree.next_id();
        let img_node = RenderNode::new(
            img_id,
            RenderNodeType::Image(ImageNode {
                section_index: Some(section_index),
                para_index: Some(para_index),
                control_index: Some(control_index),
                crop,
                original_size_hu,
                effect: picture.image_attr.effect,
                brightness: picture.image_attr.brightness,
                contrast: picture.image_attr.contrast,
                ..ImageNode::new(bin_data_id, image_data)
            }),
            BoundingBox::new(adjusted_pic_x, pic_y, pic_width, pic_height),
        );

        parent_node.children.push(img_node);

        // 그림 테두리(선) 렌더링
        self.render_picture_border(tree, parent_node, picture, adjusted_pic_x, pic_y, pic_width, pic_height);

        // 캡션 렌더링
        if let Some(ref caption) = picture.caption {
            use crate::model::shape::CaptionVertAlign;
            let (cap_x, cap_w, cap_y) = match caption.direction {
                CaptionDirection::Top => (adjusted_pic_x, pic_width, base_y),
                CaptionDirection::Bottom => (adjusted_pic_x, pic_width, pic_y + pic_height + caption_spacing),
                CaptionDirection::Left | CaptionDirection::Right => {
                    let cw = hwpunit_to_px(caption.width as i32, self.dpi);
                    let cx = if caption.direction == CaptionDirection::Left {
                        pic_x
                    } else {
                        adjusted_pic_x + pic_width + caption_spacing
                    };
                    let cy = match caption.vert_align {
                        CaptionVertAlign::Top => pic_y,
                        CaptionVertAlign::Center => pic_y + (pic_height - caption_height).max(0.0) / 2.0,
                        CaptionVertAlign::Bottom => pic_y + (pic_height - caption_height).max(0.0),
                    };
                    (cx, cw, cy)
                }
            };

            let cell_ctx = super::CellContext {
                parent_para_index: para_index,
                path: vec![super::CellPathEntry {
                    control_index,
                    cell_index: 0,
                    cell_para_index: 0,
                    text_direction: 0,
                }],
            };
            self.layout_caption(
                tree, parent_node, caption, styles, col_area,
                cap_x, cap_w, cap_y,
                &mut self.auto_counter.borrow_mut(),
                Some(cell_ctx),
            );
        }

        // y_offset 업데이트: Para 기준 그림만 높이만큼 진행
        // Page/Paper 기준 그림은 플로팅이므로 y_offset 변경 없음
        // Task #347: 글뒤로/글앞으로 그림은 본문 흐름을 점유하지 않으므로 y 미진행.
        let total_height = pic_height + caption_height + if caption_height > 0.0 { caption_spacing } else { 0.0 };
        match (picture.common.vert_rel_to, picture.common.text_wrap) {
            (VertRelTo::Para, TextWrap::BehindText | TextWrap::InFrontOfText) => y_offset,
            (VertRelTo::Para, _) => y_offset + total_height,
            (VertRelTo::Page | VertRelTo::Paper, _) => y_offset,
        }
    }

    /// 캡션의 총 높이를 계산한다.
    pub(crate) fn calculate_caption_height(
        &self,
        caption: &Option<Caption>,
        _styles: &ResolvedStyleSet,
    ) -> f64 {
        let caption = match caption {
            Some(c) => c,
            None => return 0.0,
        };

        if caption.paragraphs.is_empty() {
            return 0.0;
        }

        let mut total_height = 0.0;
        for para in &caption.paragraphs {
            let composed = compose_paragraph(para);
            if composed.lines.is_empty() {
                total_height += hwpunit_to_px(400, self.dpi); // 기본 줄 높이
            } else {
                for (i, line) in composed.lines.iter().enumerate() {
                    let line_h = hwpunit_to_px(line.line_height, self.dpi);
                    let spacing = if i < composed.lines.len() - 1 {
                        hwpunit_to_px(line.line_spacing, self.dpi)
                    } else {
                        0.0 // 마지막 줄은 line_spacing 제외
                    };
                    total_height += line_h + spacing;
                }
            }
        }

        total_height
    }

    /// 캡션을 레이아웃한다.
    pub(crate) fn layout_caption(
        &self,
        tree: &mut PageRenderTree,
        parent_node: &mut RenderNode,
        caption: &Caption,
        styles: &ResolvedStyleSet,
        _col_area: &LayoutRect,
        content_x: f64,
        content_width: f64,
        y_start: f64,
        auto_counter: &mut AutoNumberCounter,
        cell_ctx: Option<super::CellContext>,
    ) {
        if caption.paragraphs.is_empty() {
            return;
        }

        let caption_area = LayoutRect {
            x: content_x,
            y: y_start,
            width: content_width,
            height: 0.0, // 높이는 동적
        };

        let mut para_y = y_start;
        for (pi, para) in caption.paragraphs.iter().enumerate() {
            // 먼저 문단을 조합
            let mut composed = compose_paragraph(para);

            // AutoNumber 컨트롤 처리: 조합된 텍스트에 번호 삽입
            self.apply_auto_numbers_to_composed(&mut composed, para, auto_counter);

            // cell_ctx에 cell_para_index 갱신
            let ctx = cell_ctx.as_ref().map(|c| {
                let mut cc = c.clone();
                if let Some(last) = cc.path.last_mut() {
                    last.cell_para_index = pi;
                }
                cc
            });

            para_y = self.layout_composed_paragraph(
                tree,
                parent_node,
                &composed,
                styles,
                &caption_area,
                para_y,
                0,
                composed.lines.len(),
                0, 0, ctx, false, 0.0, None, None, None,
            );
        }
    }

    pub(crate) fn estimate_footnote_area_height(
        &self,
        footnotes: &[FootnoteRef],
        paragraphs: &[Paragraph],
        shape: &FootnoteShape,
    ) -> f64 {
        if footnotes.is_empty() {
            return 0.0;
        }
        let mut total = 0.0;

        // 구분선 위 여백 + 구분선 + 아래 여백
        total += hwpunit_to_px(shape.separator_margin_top as i32, self.dpi);
        total += border_width_to_px(shape.separator_line_width).max(0.5);
        total += hwpunit_to_px(shape.separator_margin_bottom as i32, self.dpi);

        // 각 각주의 문단 높이 (LineSeg.line_height는 HWP에서 줄간격 이미 반영됨)
        for (i, fn_ref) in footnotes.iter().enumerate() {
            let fn_paras = get_footnote_paragraphs(fn_ref, paragraphs);
            for para in fn_paras {
                if para.line_segs.is_empty() {
                    total += hwpunit_to_px(400, self.dpi);
                } else {
                    for seg in &para.line_segs {
                        total += hwpunit_to_px(seg.line_height, self.dpi);
                    }
                }
            }
            // 각주 간 간격
            if i + 1 < footnotes.len() {
                total += hwpunit_to_px(shape.note_spacing as i32, self.dpi);
            }
        }
        total
    }

    /// 각주 영역 레이아웃 (구분선 + 각주 문단들)
    pub(crate) fn layout_footnote_area(
        &self,
        tree: &mut PageRenderTree,
        fn_node: &mut RenderNode,
        footnotes: &[FootnoteRef],
        paragraphs: &[Paragraph],
        styles: &ResolvedStyleSet,
        fn_area: &LayoutRect,
        shape: &FootnoteShape,
    ) {
        let mut y = fn_area.y;

        // (1) 구분선 위 여백
        y += hwpunit_to_px(shape.separator_margin_top as i32, self.dpi);

        // (2) 구분선
        let sep_length = if shape.separator_length > 0 {
            // separator_length는 HWP 단위로 페이지 폭의 비율
            let fraction = shape.separator_length as f64 / 50000.0;
            fn_area.width * fraction.min(1.0)
        } else {
            fn_area.width / 3.0 // 기본값: 1/3 폭
        };
        let line_width = border_width_to_px(shape.separator_line_width).max(0.5);

        let sep_id = tree.next_id();
        let sep_node = RenderNode::new(
            sep_id,
            RenderNodeType::Line(LineNode::new(
                fn_area.x, y, fn_area.x + sep_length, y,
                LineStyle {
                    color: shape.separator_color,
                    width: line_width,
                    dash: StrokeDash::Solid,
                    ..Default::default()
                },
            )),
            BoundingBox::new(fn_area.x, y - line_width / 2.0, sep_length, line_width),
        );
        fn_node.children.push(sep_node);
        y += line_width;

        // (3) 구분선 아래 여백
        y += hwpunit_to_px(shape.separator_margin_bottom as i32, self.dpi);

        // (4) 각 각주 렌더링
        // 각주 TextRun에 마커를 인코딩하여 히트테스트에서 식별 가능하도록 함
        // section_index = footnote_index (footnotes 배열 인덱스)
        // para_index = usize::MAX - 2000 - fn_para_idx (각주 내 문단 인덱스)
        for (i, fn_ref) in footnotes.iter().enumerate() {
            let fn_paras = get_footnote_paragraphs(fn_ref, paragraphs);
            let number_text = format_footnote_number(fn_ref.number, &shape.number_format, shape.suffix_char);

            for (p_idx, para) in fn_paras.iter().enumerate() {
                let composed = compose_paragraph(para);
                let marker_section = i; // footnote_index
                let marker_para = usize::MAX - 2000 - p_idx; // 각주 내 문단 인덱스
                // 각주 번호 스타일용 기본 char_shape_id (빈/비빈 문단 모두 동일)
                let base_cs_id = para.char_shapes.first()
                    .map(|cs| cs.char_shape_id as u32)
                    .unwrap_or(composed.para_style_id as u32);

                if p_idx == 0 {
                    // 첫 문단: 각주 번호를 텍스트 앞에 삽입
                    y = self.layout_footnote_paragraph_with_number(
                        tree, fn_node, &composed, styles, fn_area, y, &number_text,
                        marker_section, marker_para, base_cs_id,
                    );
                } else {
                    y = self.layout_composed_paragraph(
                        tree, fn_node, &composed, styles, fn_area, y, 0, composed.lines.len(),
                        marker_section, marker_para, None, false, 0.0, None, None, None,
                    );
                }
            }

            // 각주 간 간격
            if i + 1 < footnotes.len() {
                y += hwpunit_to_px(shape.note_spacing as i32, self.dpi);
            }
        }
    }

    /// 각주 번호를 앞에 붙여 문단을 레이아웃
    /// marker_section: footnote_index, marker_para: 각주 내 문단 마커 (usize::MAX - 2000 - fn_para_idx)
    /// base_cs_id: 번호 스타일 결정용 기본 char_shape_id (문단의 char_shapes[0])
    pub(crate) fn layout_footnote_paragraph_with_number(
        &self,
        tree: &mut PageRenderTree,
        parent: &mut RenderNode,
        composed: &ComposedParagraph,
        styles: &ResolvedStyleSet,
        area: &LayoutRect,
        y_start: f64,
        number_text: &str,
        marker_section: usize,
        marker_para: usize,
        base_cs_id: u32,
    ) -> f64 {
        let mut y = y_start;

        for (line_idx, comp_line) in composed.lines.iter().enumerate() {
            // LineSeg.line_height는 HWP에서 줄간격이 이미 반영된 값
            let line_height = hwpunit_to_px(comp_line.line_height, self.dpi);
            let baseline = hwpunit_to_px(comp_line.baseline_distance, self.dpi);

            let line_id = tree.next_id();
            let mut line_node = RenderNode::new(
                line_id,
                RenderNodeType::TextLine(TextLineNode::new(line_height, baseline)),
                BoundingBox::new(area.x, y, area.width, line_height),
            );

            let mut x = area.x;

            // 첫 줄에 각주 번호 삽입
            if line_idx == 0 {
                // 각주 번호 스타일: 문단의 기본 char_shape로 고정 (크기 약간 축소)
                // 빈/비빈 문단 모두 동일한 base_cs_id 사용 → 리렌더링 시 폰트·폭 변동 방지
                let base_style = {
                    let mut ts = resolved_to_text_style(styles, base_cs_id, 0);
                    ts.font_size = (ts.font_size * 0.9).max(8.0);
                    ts
                };

                let num_width = estimate_text_width(number_text, &base_style);
                let num_id = tree.next_id();
                let num_node = RenderNode::new(
                    num_id,
                    RenderNodeType::TextRun(TextRunNode {
                        text: number_text.to_string(),
                        style: base_style,
                        char_shape_id: None,
                        para_shape_id: None,
                        section_index: Some(marker_section),
                        para_index: Some(marker_para),
                        char_start: None, // 번호 run은 char_start 없음
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
                    BoundingBox::new(x, y, num_width, line_height),
                );
                line_node.children.push(num_node);
                x += num_width;
            }

            // 원본 TextRun들
            let mut char_offset = comp_line.char_start;
            for run in &comp_line.runs {
                let text_style = resolved_to_text_style(styles, run.char_style_id, run.lang_index);
                let width = estimate_text_width(&run.text, &text_style);

                let run_id = tree.next_id();
                let run_node = RenderNode::new(
                    run_id,
                    RenderNodeType::TextRun(TextRunNode {
                        text: run.text.clone(),
                        style: text_style,
                        char_shape_id: None,
                        para_shape_id: None,
                        section_index: Some(marker_section),
                        para_index: Some(marker_para),
                        char_start: Some(char_offset),
                        cell_context: None,
                        is_para_end: false,
                        is_line_break_end: false,
                        rotation: 0.0,
                        is_vertical: false,
                        char_overlap: run.char_overlap.clone(),
                        border_fill_id: 0,
                        baseline,
                        field_marker: FieldMarkerType::None,
                    }),
                    BoundingBox::new(x, y, width, line_height),
                );
                line_node.children.push(run_node);
                x += width;
                char_offset += run.text.chars().count();
            }

            parent.children.push(line_node);
            // [Issue #483] trailing line_spacing 추가 — layout_composed_paragraph:2560 과 정합.
            // 누락 시 multi-paragraph 각주의 paragraph 간 gap 이 line_spacing(예: 360 HU = 4.8px)
            // 만큼 좁아져 한컴 vpos 의도와 어긋남.
            let line_spacing_px = hwpunit_to_px(comp_line.line_spacing, self.dpi);
            y += line_height + line_spacing_px;
        }

        // 빈 문단 fallback
        if composed.lines.is_empty() {
            let default_height = hwpunit_to_px(400, self.dpi);
            let line_id = tree.next_id();
            let line_node = RenderNode::new(
                line_id,
                RenderNodeType::TextLine(TextLineNode::new(default_height, default_height * 0.8)),
                BoundingBox::new(area.x, y, area.width, default_height),
            );
            parent.children.push(line_node);
            y += default_height;
        }

        y
    }

    /// 문단 내 각주/미주 컨트롤에 대해 윗첨자 참조 번호를 렌더링한다.
    ///
    /// 마지막 TextLine의 마지막 TextRun 우측에 윗첨자 번호를 추가한다.
    pub(crate) fn add_footnote_superscripts(
        &self,
        tree: &mut PageRenderTree,
        parent: &mut RenderNode,
        para: &Paragraph,
        _styles: &ResolvedStyleSet,
    ) {
        // layout_composed_paragraph에서 이미 인라인 FootnoteMarker를 삽입한 경우 건너뜀
        let has_inline_markers = parent.children.iter().any(|line| {
            line.children.iter().any(|n| matches!(n.node_type, RenderNodeType::FootnoteMarker(_)))
        });
        if has_inline_markers {
            return;
        }

        // 각주/미주의 (번호, 텍스트 위치) 수집 — ComposedParagraph에서 미리 계산된 위치 사용
        // 폴백: find_control_text_positions로 직접 계산
        let ctrl_positions = crate::document_core::helpers::find_control_text_positions(para);
        let mut footnotes: Vec<(u16, usize)> = Vec::new();
        for (ci, ctrl) in para.controls.iter().enumerate() {
            let num = match ctrl {
                Control::Footnote(fn_ctrl) => Some(fn_ctrl.number),
                Control::Endnote(en_ctrl) => Some(en_ctrl.number),
                _ => None,
            };
            if let Some(n) = num {
                let pos = ctrl_positions.get(ci).copied().unwrap_or(usize::MAX);
                footnotes.push((n, pos));
            }
        }

        if footnotes.is_empty() {
            return;
        }

        // 각 각주 위첨자를 렌더링: char_start 기반으로 정확한 TextRun 위치에 삽입
        for (_fn_idx, (num, char_pos)) in footnotes.iter().enumerate() {
            let mut target_line_idx: Option<usize> = None;
            let mut insert_x = 0.0;
            let mut line_height = 18.0;
            let mut line_y = 0.0;
            let mut base_font_size = 12.0_f64;
            let mut base_font_family = "sans-serif".to_string();

            // TextRun의 char_start로 각주 위치 찾기
            if *char_pos < usize::MAX {
                'outer: for (li, line_node) in parent.children.iter().enumerate() {
                    if !matches!(line_node.node_type, RenderNodeType::TextLine(_)) { continue; }
                    // 이 줄의 char_start 범위 확인: 첫 run의 char_start ~ 마지막 run의 (char_start + len)
                    let mut line_min_cs = usize::MAX;
                    let mut line_max_end = 0usize;
                    for run_node in &line_node.children {
                        if let RenderNodeType::TextRun(ref run) = run_node.node_type {
                            if let Some(cs) = run.char_start {
                                line_min_cs = line_min_cs.min(cs);
                                line_max_end = line_max_end.max(cs + run.text.chars().count());
                            }
                        }
                    }
                    // 각주 위치가 이 줄에 포함되지 않으면 다음 줄
                    if *char_pos > line_max_end || line_min_cs == usize::MAX { continue; }

                    for run_node in &line_node.children {
                        if let RenderNodeType::TextRun(ref run) = run_node.node_type {
                            if let Some(cs) = run.char_start {
                                let run_len = run.text.chars().count();
                                let run_end = cs + run_len;
                                if *char_pos >= cs && *char_pos <= run_end {
                                    let chars_before = char_pos - cs;
                                    let partial_text: String = run.text.chars().take(chars_before).collect();
                                    let partial_width = estimate_text_width(&partial_text, &run.style);
                                    insert_x = run_node.bbox.x + partial_width;
                                    line_height = line_node.bbox.height;
                                    line_y = line_node.bbox.y;
                                    base_font_size = run.style.font_size;
                                    base_font_family = run.style.font_family.clone();
                                    target_line_idx = Some(li);
                                    break 'outer;
                                }
                            }
                        }
                    }
                }
            }

            // 최종 폴백: 마지막 TextLine 끝
            if target_line_idx.is_none() {
                if let Some(li) = parent.children.iter().rposition(|n| matches!(n.node_type, RenderNodeType::TextLine(_))) {
                    let line = &parent.children[li];
                    insert_x = line.children.last().map(|c| c.bbox.x + c.bbox.width).unwrap_or(line.bbox.x);
                    line_height = line.bbox.height;
                    line_y = line.bbox.y;
                    if let Some(last_run) = line.children.last() {
                        if let RenderNodeType::TextRun(ref run) = last_run.node_type {
                            base_font_size = run.style.font_size;
                            base_font_family = run.style.font_family.clone();
                        }
                    }
                    target_line_idx = Some(li);
                }
            }

            if let Some(line_idx) = target_line_idx {
                let sup_font_size = (base_font_size * 0.6).max(7.0);
                let sup_y_offset = line_height * 0.35;
                let number_text = format!("{})", num);
                let style = TextStyle {
                    font_size: sup_font_size,
                    font_family: base_font_family,
                    ..Default::default()
                };
                let width = estimate_text_width(&number_text, &style);

                let run_id = tree.next_id();
                let run_node = RenderNode::new(
                    run_id,
                    RenderNodeType::TextRun(TextRunNode {
                        text: number_text,
                        style,
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
                        baseline: line_height,
                        field_marker: FieldMarkerType::None,
                    }),
                    BoundingBox::new(insert_x, line_y - sup_y_offset, width, line_height),
                );

                let line_mut = &mut parent.children[line_idx];
                line_mut.children.push(run_node);
            }
        }
    }
}

fn get_footnote_paragraphs<'a>(
    fn_ref: &FootnoteRef,
    paragraphs: &'a [Paragraph],
) -> &'a [Paragraph] {
    match &fn_ref.source {
        FootnoteSource::Body { para_index, control_index } => {
            if let Some(para) = paragraphs.get(*para_index) {
                if let Some(Control::Footnote(footnote)) = para.controls.get(*control_index) {
                    return &footnote.paragraphs;
                }
            }
            &[]
        }
        FootnoteSource::TableCell {
            para_index,
            table_control_index,
            cell_index,
            cell_para_index,
            cell_control_index,
        } => {
            if let Some(para) = paragraphs.get(*para_index) {
                if let Some(Control::Table(table)) = para.controls.get(*table_control_index) {
                    if let Some(cell) = table.cells.get(*cell_index) {
                        if let Some(cp) = cell.paragraphs.get(*cell_para_index) {
                            if let Some(Control::Footnote(footnote)) = cp.controls.get(*cell_control_index) {
                                return &footnote.paragraphs;
                            }
                        }
                    }
                }
            }
            &[]
        }
        FootnoteSource::ShapeTextBox {
            para_index,
            shape_control_index,
            tb_para_index,
            tb_control_index,
        } => {
            if let Some(para) = paragraphs.get(*para_index) {
                if let Some(Control::Shape(shape_obj)) = para.controls.get(*shape_control_index) {
                    if let Some(text_box) = shape_obj.drawing().and_then(|d| d.text_box.as_ref()) {
                        if let Some(tp) = text_box.paragraphs.get(*tb_para_index) {
                            if let Some(Control::Footnote(footnote)) = tp.controls.get(*tb_control_index) {
                                return &footnote.paragraphs;
                            }
                        }
                    }
                }
            }
            &[]
        }
    }
}

/// 각주 번호 포맷 (NumberFormat에 따른 변환)
fn format_footnote_number(number: u16, format: &NumberFormat, suffix: char) -> String {
    let num_str = match format {
        NumberFormat::Digit => number.to_string(),
        NumberFormat::CircledDigit => {
            // ① ~ ⑳
            if number >= 1 && number <= 20 {
                char::from_u32(0x2460 + (number - 1) as u32)
                    .map(|c| c.to_string())
                    .unwrap_or_else(|| number.to_string())
            } else {
                number.to_string()
            }
        }
        NumberFormat::LowerAlpha => {
            if number >= 1 && number <= 26 {
                char::from_u32(b'a' as u32 + (number - 1) as u32)
                    .map(|c| c.to_string())
                    .unwrap_or_else(|| number.to_string())
            } else {
                number.to_string()
            }
        }
        NumberFormat::UpperAlpha => {
            if number >= 1 && number <= 26 {
                char::from_u32(b'A' as u32 + (number - 1) as u32)
                    .map(|c| c.to_string())
                    .unwrap_or_else(|| number.to_string())
            } else {
                number.to_string()
            }
        }
        _ => number.to_string(), // 기타 형식은 숫자로 fallback
    };

    let suffix_str = if suffix != '\0' {
        suffix.to_string()
    } else {
        ")".to_string()
    };

    format!("{}{} ", num_str, suffix_str)
}

impl LayoutEngine {
    /// 그림 테두리(선) 렌더링
    /// border_attr의 bit 0~5가 선 종류, border_width가 두께 (0이면 기본 0.1mm)
    pub(crate) fn render_picture_border(
        &self,
        tree: &mut PageRenderTree,
        parent: &mut RenderNode,
        picture: &crate::model::image::Picture,
        x: f64, y: f64, w: f64, h: f64,
    ) {
        let line_type = picture.border_attr.attr & 0x3F;
        // 선 종류 0 = 없음
        if line_type == 0 {
            return;
        }
        let border_w = if picture.border_width > 0 {
            hwpunit_to_px(picture.border_width, self.dpi)
        } else {
            // 기본 선 두께 0.1mm
            0.1 / 25.4 * self.dpi
        };
        let stroke_dash = match line_type {
            2 => super::super::StrokeDash::Dot,      // 점선
            3 => super::super::StrokeDash::Dash,      // 긴 점선 (파선)
            4 => super::super::StrokeDash::DashDot,   // 일점쇄선
            5 => super::super::StrokeDash::DashDotDot, // 이점쇄선
            _ => super::super::StrokeDash::Solid,      // 1=실선, 기타
        };
        let style = super::super::ShapeStyle {
            fill_color: None,
            pattern: None,
            stroke_color: Some(picture.border_color),
            stroke_width: border_w,
            stroke_dash,
            opacity: 1.0,
            shadow: None,
        };
        let border_id = tree.next_id();
        let border_node = RenderNode::new(
            border_id,
            RenderNodeType::Rectangle(RectangleNode::new(0.0, style, None)),
            BoundingBox::new(x, y, w, h),
        );
        parent.children.push(border_node);
    }
}

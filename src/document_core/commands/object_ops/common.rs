//! 공통 개체 속성/헬퍼 + 새 번호 (object_ops 분할, #1904).

use super::MIN_SHAPE_SIZE;
use crate::document_core::helpers::{get_textbox_from_shape, get_textbox_from_shape_mut};
use crate::document_core::DocumentCore;
use crate::error::HwpError;
use crate::model::control::Control;
use crate::model::event::DocumentEvent;
use crate::model::paragraph::Paragraph;
use crate::model::shape::{common_obj_offsets, ShapeObject};

impl DocumentCore {
    const COMMON_OBJ_ATTR_KNOWN_MASK: u32 = 0x01
        | (0x03 << 3)
        | (0x07 << 5)
        | (0x03 << 8)
        | (0x07 << 10)
        | (1 << 13)
        | (1 << 14)
        | (0x07 << 15)
        | (0x03 << 18)
        | (1 << 20)
        | (0x07 << 21)
        | (0x03 << 24)
        | (1 << 26)
        | (1 << 28);
    pub(crate) fn sync_common_obj_attr_known_bits(c: &mut crate::model::shape::CommonObjAttr) {
        let packed =
            crate::document_core::converters::common_obj_attr_writer::pack_common_attr_bits(c);
        c.attr = (c.attr & !Self::COMMON_OBJ_ATTR_KNOWN_MASK)
            | (packed & Self::COMMON_OBJ_ATTR_KNOWN_MASK);
    }
    pub(crate) fn is_structure_only_empty_paragraph(para: &Paragraph) -> bool {
        para.text.is_empty()
            && !para.controls.is_empty()
            && para
                .controls
                .iter()
                .all(|ctrl| matches!(ctrl, Control::SectionDef(_) | Control::ColumnDef(_)))
    }
    /// 컨트롤 삭제 후 문단의 line_segs를 재계산한다.
    ///
    /// 그림/도형 삭제 시 문단의 line_segs에 컨트롤 높이가 그대로 남아,
    /// 레이아웃이 갱신되지 않는 문제를 방지한다.
    pub(crate) fn reflow_paragraph_line_segs_after_control_delete(
        para: &mut Paragraph,
        styles: &crate::renderer::style_resolver::ResolvedStyleSet,
        dpi: f64,
    ) {
        // 남은 컨트롤 중 가장 큰 높이 계산
        let max_remaining_ctrl_height = para
            .controls
            .iter()
            .map(|ctrl| match ctrl {
                Control::Picture(pic) => pic.common.height as i32,
                Control::Shape(shape) => shape.common().height as i32,
                Control::Equation(eq) => eq.common.height as i32,
                _ => 0,
            })
            .max()
            .unwrap_or(0);

        if max_remaining_ctrl_height > 0 {
            // 아직 컨트롤이 남아있으면 가장 큰 컨트롤 높이로 설정
            if let Some(ls) = para.line_segs.first_mut() {
                ls.line_height = max_remaining_ctrl_height;
                ls.text_height = max_remaining_ctrl_height;
                ls.baseline_distance = (max_remaining_ctrl_height * 850) / 1000;
            }
        } else if para.text.is_empty() {
            // 텍스트도 컨트롤도 없음 → 기본 텍스트 높이로 리셋
            if let Some(ls) = para.line_segs.first_mut() {
                ls.line_height = 1000;
                ls.text_height = 1000;
                ls.baseline_distance = 850;
                ls.line_spacing = 600;
            }
        } else {
            // 텍스트가 있으면 reflow_line_segs로 재계산
            let seg_width = para.line_segs.first().map(|s| s.segment_width).unwrap_or(0);
            let available_width_px = crate::renderer::hwpunit_to_px(seg_width, dpi);
            crate::renderer::composer::reflow_line_segs(para, available_width_px, styles, dpi);
        }
    }
    /// CommonObjAttr → JSON 문자열 (Shape/Picture 공용 속성)
    pub(crate) fn common_obj_attr_to_json(c: &crate::model::shape::CommonObjAttr) -> String {
        let vert_rel = match c.vert_rel_to {
            crate::model::shape::VertRelTo::Paper => "Paper",
            crate::model::shape::VertRelTo::Page => "Page",
            crate::model::shape::VertRelTo::Para => "Para",
        };
        let vert_align = match c.vert_align {
            crate::model::shape::VertAlign::Top => "Top",
            crate::model::shape::VertAlign::Center => "Center",
            crate::model::shape::VertAlign::Bottom => "Bottom",
            crate::model::shape::VertAlign::Inside => "Inside",
            crate::model::shape::VertAlign::Outside => "Outside",
        };
        let horz_rel = match c.horz_rel_to {
            crate::model::shape::HorzRelTo::Paper => "Paper",
            crate::model::shape::HorzRelTo::Page => "Page",
            crate::model::shape::HorzRelTo::Column => "Column",
            crate::model::shape::HorzRelTo::Para => "Para",
        };
        let horz_align = match c.horz_align {
            crate::model::shape::HorzAlign::Left => "Left",
            crate::model::shape::HorzAlign::Center => "Center",
            crate::model::shape::HorzAlign::Right => "Right",
            crate::model::shape::HorzAlign::Inside => "Inside",
            crate::model::shape::HorzAlign::Outside => "Outside",
        };
        let text_wrap = match c.text_wrap {
            crate::model::shape::TextWrap::Square => "Square",
            crate::model::shape::TextWrap::Tight => "Tight",
            crate::model::shape::TextWrap::Through => "Through",
            crate::model::shape::TextWrap::TopAndBottom => "TopAndBottom",
            crate::model::shape::TextWrap::BehindText => "BehindText",
            crate::model::shape::TextWrap::InFrontOfText => "InFrontOfText",
        };
        let desc_escaped = crate::document_core::helpers::json_escape(&c.description);
        format!(
            "\"width\":{},\"height\":{},\"treatAsChar\":{},\
             \"vertRelTo\":\"{}\",\"vertAlign\":\"{}\",\
             \"horzRelTo\":\"{}\",\"horzAlign\":\"{}\",\
             \"vertOffset\":{},\"horzOffset\":{},\
             \"textWrap\":\"{}\",\"restrictInPage\":{},\"allowOverlap\":{},\"sizeProtect\":{},\
             \"zOrder\":{},\"instanceId\":{},\
             \"outerMarginLeft\":{},\"outerMarginTop\":{},\
             \"outerMarginRight\":{},\"outerMarginBottom\":{},\
             \"description\":\"{}\"",
            c.width,
            c.height,
            c.treat_as_char,
            vert_rel,
            vert_align,
            horz_rel,
            horz_align,
            c.vertical_offset,
            c.horizontal_offset,
            text_wrap,
            c.flow_with_text,
            c.allow_overlap,
            c.size_protect,
            c.z_order,
            c.instance_id,
            c.margin.left,
            c.margin.top,
            c.margin.right,
            c.margin.bottom,
            desc_escaped,
        )
    }
    /// JSON → CommonObjAttr 필드 업데이트 (Shape/Picture 공용)
    pub(crate) fn apply_common_obj_attr_from_json(
        c: &mut crate::model::shape::CommonObjAttr,
        props_json: &str,
    ) {
        use crate::document_core::helpers::{json_bool, json_i16, json_str, json_u32};

        if let Some(w) = json_u32(props_json, "width") {
            c.width = w.max(MIN_SHAPE_SIZE);
        }
        if let Some(h) = json_u32(props_json, "height") {
            c.height = h.max(MIN_SHAPE_SIZE);
        }
        if let Some(tac) = json_bool(props_json, "treatAsChar") {
            c.treat_as_char = tac;
            if tac {
                c.attr |= 0x01;
            } else {
                c.attr &= !0x01;
            }
        }
        if let Some(v) = json_str(props_json, "vertRelTo") {
            c.vert_rel_to = match v.as_str() {
                "Paper" => crate::model::shape::VertRelTo::Paper,
                "Page" => crate::model::shape::VertRelTo::Page,
                "Para" => crate::model::shape::VertRelTo::Para,
                _ => c.vert_rel_to,
            };
        }
        if let Some(v) = json_str(props_json, "horzRelTo") {
            c.horz_rel_to = match v.as_str() {
                "Paper" => crate::model::shape::HorzRelTo::Paper,
                "Page" => crate::model::shape::HorzRelTo::Page,
                "Column" => crate::model::shape::HorzRelTo::Column,
                "Para" => crate::model::shape::HorzRelTo::Para,
                _ => c.horz_rel_to,
            };
        }
        if let Some(v) = json_str(props_json, "vertAlign") {
            c.vert_align = match v.as_str() {
                "Top" => crate::model::shape::VertAlign::Top,
                "Center" => crate::model::shape::VertAlign::Center,
                "Bottom" => crate::model::shape::VertAlign::Bottom,
                _ => c.vert_align,
            };
        }
        if let Some(v) = json_str(props_json, "horzAlign") {
            c.horz_align = match v.as_str() {
                "Left" => crate::model::shape::HorzAlign::Left,
                "Center" => crate::model::shape::HorzAlign::Center,
                "Right" => crate::model::shape::HorzAlign::Right,
                _ => c.horz_align,
            };
        }
        if let Some(v) = json_str(props_json, "textWrap") {
            c.text_wrap = match v.as_str() {
                "Square" => crate::model::shape::TextWrap::Square,
                "Tight" => crate::model::shape::TextWrap::Tight,
                "Through" => crate::model::shape::TextWrap::Through,
                "TopAndBottom" => crate::model::shape::TextWrap::TopAndBottom,
                "BehindText" => crate::model::shape::TextWrap::BehindText,
                "InFrontOfText" => crate::model::shape::TextWrap::InFrontOfText,
                _ => c.text_wrap,
            };
        }
        if let Some(v) = json_bool(props_json, "restrictInPage") {
            c.flow_with_text = v;
            if v {
                c.attr |= 1 << 13;
                c.allow_overlap = false;
                c.attr &= !(1 << 14);
            } else {
                c.attr &= !(1 << 13);
            }
        }
        if let Some(v) = json_bool(props_json, "allowOverlap") {
            c.allow_overlap = v;
            if v {
                c.attr |= 1 << 14;
            } else {
                c.attr &= !(1 << 14);
            }
        }
        if let Some(v) = json_bool(props_json, "sizeProtect") {
            c.size_protect = v;
            if v {
                c.attr |= 1 << 20;
            } else {
                c.attr &= !(1 << 20);
            }
        }
        if c.flow_with_text {
            c.allow_overlap = false;
            c.attr &= !(1 << 14);
        }
        if let Some(v) = json_u32(props_json, "vertOffset") {
            c.vertical_offset = v;
        }
        if let Some(v) = json_u32(props_json, "horzOffset") {
            c.horizontal_offset = v;
        }
        if let Some(v) = json_str(props_json, "description") {
            c.description = v;
        }
        if let Some(v) = json_i16(props_json, "outerMarginLeft") {
            c.margin.left = v;
        }
        if let Some(v) = json_i16(props_json, "outerMarginTop") {
            c.margin.top = v;
        }
        if let Some(v) = json_i16(props_json, "outerMarginRight") {
            c.margin.right = v;
        }
        if let Some(v) = json_i16(props_json, "outerMarginBottom") {
            c.margin.bottom = v;
        }
        Self::sync_common_obj_attr_known_bits(c);
    }
    /// 직선 끝점 이동: 글로벌 좌표(HWPUNIT)로 시작/끝점을 직접 설정
    pub fn move_line_endpoint_native(
        &mut self,
        section_idx: usize,
        para_idx: usize,
        control_idx: usize,
        start_x: i32,
        start_y: i32,
        end_x: i32,
        end_y: i32,
    ) -> Result<String, HwpError> {
        let section = self
            .document
            .sections
            .get_mut(section_idx)
            .ok_or_else(|| HwpError::RenderError("구역 범위 초과".to_string()))?;
        let para = section
            .paragraphs
            .get_mut(para_idx)
            .ok_or_else(|| HwpError::RenderError("문단 범위 초과".to_string()))?;
        let ctrl = para
            .controls
            .get_mut(control_idx)
            .ok_or_else(|| HwpError::RenderError("컨트롤 범위 초과".to_string()))?;
        let line = match ctrl {
            Control::Shape(ref mut s) => match s.as_mut() {
                ShapeObject::Line(ref mut l) => l,
                _ => return Err(HwpError::RenderError("직선이 아닙니다".to_string())),
            },
            _ => return Err(HwpError::RenderError("Shape이 아닙니다".to_string())),
        };

        let min_x = start_x.min(end_x);
        let min_y = start_y.min(end_y);
        let w = (start_x - end_x).abs().max(1);
        let h = (start_y - end_y).abs().max(0);

        line.common.horizontal_offset = min_x as u32;
        line.common.vertical_offset = min_y as u32;
        line.common.width = w as u32;
        line.common.height = h.max(1) as u32;
        line.start.x = start_x - min_x;
        line.start.y = start_y - min_y;
        line.end.x = end_x - min_x;
        line.end.y = end_y - min_y;

        line.drawing.shape_attr.current_width = w as u32;
        line.drawing.shape_attr.original_width = w as u32;
        line.drawing.shape_attr.current_height = h.max(1) as u32;
        line.drawing.shape_attr.original_height = h.max(1) as u32;
        line.drawing.shape_attr.rotation_center.x = w / 2;
        line.drawing.shape_attr.rotation_center.y = h / 2;
        line.drawing.shape_attr.raw_rendering = Vec::new();

        section.raw_stream = None;
        self.recompose_section(section_idx);
        self.paginate_if_needed();
        self.update_connectors_in_section(section_idx);

        Ok("{\"ok\":true}".to_string())
    }
    pub(crate) fn first_char_or_nul(value: &str) -> char {
        value.chars().next().unwrap_or('\0')
    }
    pub(crate) fn hwpunit16_from_json(json: &str, key: &str) -> Option<i16> {
        crate::document_core::helpers::json_i32(json, key)
            .map(|v| v.clamp(i16::MIN as i32, i16::MAX as i32) as i16)
    }
}

impl crate::document_core::DocumentCore {
    pub fn insert_new_number_native(
        &mut self,
        section_idx: usize,
        para_idx: usize,
        char_offset: usize,
        start_num: u16,
    ) -> Result<String, crate::error::HwpError> {
        use crate::error::HwpError;
        use crate::model::control::{AutoNumberType, Control, NewNumber};

        if section_idx >= self.document.sections.len() {
            return Err(HwpError::RenderError(format!(
                "구역 인덱스 {} 범위 초과",
                section_idx
            )));
        }
        if para_idx >= self.document.sections[section_idx].paragraphs.len() {
            return Err(HwpError::RenderError(format!(
                "문단 인덱스 {} 범위 초과",
                para_idx
            )));
        }

        let new_number = NewNumber {
            number_type: AutoNumberType::Page,
            number: start_num,
        };

        self.document.sections[section_idx].raw_stream = None;
        let paragraph = &mut self.document.sections[section_idx].paragraphs[para_idx];

        let insert_idx = {
            let positions = crate::document_core::helpers::find_control_text_positions(paragraph);
            let mut idx = paragraph.controls.len();
            for (i, &pos) in positions.iter().enumerate() {
                if pos > char_offset {
                    idx = i;
                    break;
                }
            }
            idx
        };

        paragraph
            .controls
            .insert(insert_idx, Control::NewNumber(new_number));
        paragraph.ctrl_data_records.insert(insert_idx, None);

        if !paragraph.char_offsets.is_empty() {
            let text_len = paragraph.text.chars().count();
            let safe_offset = char_offset.min(text_len);
            for co in paragraph.char_offsets[safe_offset..].iter_mut() {
                *co += 8;
            }
        }
        paragraph.char_count += 8;
        paragraph.control_mask |= 1u32 << 0x0012;
        paragraph.has_para_text = true;

        self.reflow_paragraph(section_idx, para_idx);
        self.recompose_section(section_idx);
        self.paginate_if_needed();
        self.invalidate_page_tree_cache();

        Ok(crate::document_core::helpers::json_ok_with(&format!(
            "\"controlIdx\":{}",
            insert_idx
        )))
    }
}

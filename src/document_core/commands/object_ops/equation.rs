//! 수식 native 명령 (object_ops 분할, #1904).

use super::MIN_SHAPE_SIZE;
use crate::document_core::helpers::{get_textbox_from_shape, get_textbox_from_shape_mut};
use crate::document_core::DocumentCore;
use crate::error::HwpError;
use crate::model::control::Control;
use crate::model::event::DocumentEvent;
use crate::model::paragraph::Paragraph;
use crate::model::shape::{common_obj_offsets, ShapeObject};

impl DocumentCore {
    /// 수식 컨트롤의 속성을 조회한다 (네이티브).
    /// 표 셀 내 또는 본문의 수식 컨트롤을 찾아 불변 참조를 반환한다.
    fn find_equation_ref(
        &self,
        section_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
        cell_idx: Option<usize>,
        cell_para_idx: Option<usize>,
    ) -> Result<&crate::model::control::Equation, HwpError> {
        let section = self.document.sections.get(section_idx).ok_or_else(|| {
            HwpError::RenderError(format!("구역 인덱스 {} 범위 초과", section_idx))
        })?;

        let ctrl = if let (Some(ci), Some(cpi)) = (cell_idx, cell_para_idx) {
            // 표 셀 내 수식
            let para = section.paragraphs.get(parent_para_idx).ok_or_else(|| {
                HwpError::RenderError(format!("문단 인덱스 {} 범위 초과", parent_para_idx))
            })?;
            let table = match para.controls.get(control_idx) {
                Some(Control::Table(t)) => t,
                _ => {
                    return Err(HwpError::RenderError(
                        "지정된 컨트롤이 표가 아닙니다".to_string(),
                    ))
                }
            };
            let cell = table
                .cells
                .get(ci)
                .ok_or_else(|| HwpError::RenderError(format!("셀 인덱스 {} 범위 초과", ci)))?;
            let cell_para = cell.paragraphs.get(cpi).ok_or_else(|| {
                HwpError::RenderError(format!("셀 문단 인덱스 {} 범위 초과", cpi))
            })?;
            // 셀 문단의 첫 번째 수식 컨트롤을 찾는다
            cell_para
                .controls
                .iter()
                .find(|c| matches!(c, Control::Equation(_)))
                .ok_or_else(|| {
                    HwpError::RenderError("셀 문단에 수식 컨트롤이 없습니다".to_string())
                })?
        } else {
            // 본문 수식
            let para = section.paragraphs.get(parent_para_idx).ok_or_else(|| {
                HwpError::RenderError(format!("문단 인덱스 {} 범위 초과", parent_para_idx))
            })?;
            para.controls.get(control_idx).ok_or_else(|| {
                HwpError::RenderError(format!("컨트롤 인덱스 {} 범위 초과", control_idx))
            })?
        };

        match ctrl {
            Control::Equation(e) => Ok(e),
            _ => Err(HwpError::RenderError(
                "지정된 컨트롤이 수식이 아닙니다".to_string(),
            )),
        }
    }
    /// 표 셀 내 또는 본문의 수식 컨트롤을 찾아 가변 참조를 반환한다.
    fn find_equation_mut(
        &mut self,
        section_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
        cell_idx: Option<usize>,
        cell_para_idx: Option<usize>,
    ) -> Result<&mut crate::model::control::Equation, HwpError> {
        let section = self.document.sections.get_mut(section_idx).ok_or_else(|| {
            HwpError::RenderError(format!("구역 인덱스 {} 범위 초과", section_idx))
        })?;

        let ctrl = if let (Some(ci), Some(cpi)) = (cell_idx, cell_para_idx) {
            // 표 셀 내 수식
            let para = section.paragraphs.get_mut(parent_para_idx).ok_or_else(|| {
                HwpError::RenderError(format!("문단 인덱스 {} 범위 초과", parent_para_idx))
            })?;
            let table = match para.controls.get_mut(control_idx) {
                Some(Control::Table(t)) => t,
                _ => {
                    return Err(HwpError::RenderError(
                        "지정된 컨트롤이 표가 아닙니다".to_string(),
                    ))
                }
            };
            let cell = table
                .cells
                .get_mut(ci)
                .ok_or_else(|| HwpError::RenderError(format!("셀 인덱스 {} 범위 초과", ci)))?;
            let cell_para = cell.paragraphs.get_mut(cpi).ok_or_else(|| {
                HwpError::RenderError(format!("셀 문단 인덱스 {} 범위 초과", cpi))
            })?;
            cell_para
                .controls
                .iter_mut()
                .find(|c| matches!(c, Control::Equation(_)))
                .ok_or_else(|| {
                    HwpError::RenderError("셀 문단에 수식 컨트롤이 없습니다".to_string())
                })?
        } else {
            // 본문 수식
            let para = section.paragraphs.get_mut(parent_para_idx).ok_or_else(|| {
                HwpError::RenderError(format!("문단 인덱스 {} 범위 초과", parent_para_idx))
            })?;
            para.controls.get_mut(control_idx).ok_or_else(|| {
                HwpError::RenderError(format!("컨트롤 인덱스 {} 범위 초과", control_idx))
            })?
        };

        match ctrl {
            Control::Equation(e) => Ok(e),
            _ => Err(HwpError::RenderError(
                "지정된 컨트롤이 수식이 아닙니다".to_string(),
            )),
        }
    }
    pub(crate) fn equation_properties_json(eq: &crate::model::control::Equation) -> String {
        let common_json = Self::common_obj_attr_to_json(&eq.common);
        let script_escaped = crate::document_core::helpers::json_escape(&eq.script);
        let font_name_escaped = crate::document_core::helpers::json_escape(&eq.font_name);

        format!(
            concat!(
                "{{{},\"script\":\"{}\",\"fontSize\":{},\"color\":{},",
                "\"baseline\":{},\"fontName\":\"{}\",",
                "\"hasCaption\":false,\"captionDirection\":\"None\",",
                "\"captionWidth\":0,\"captionSpacing\":0}}"
            ),
            common_json, script_escaped, eq.font_size, eq.color, eq.baseline, font_name_escaped,
        )
    }
    pub(crate) fn apply_equation_properties(
        eq: &mut crate::model::control::Equation,
        dpi: f64,
        props_json: &str,
    ) {
        use crate::document_core::helpers::{json_i32, json_str, json_u32};
        use crate::renderer::equation::layout::EqLayout;
        use crate::renderer::equation::parser::EqParser;
        use crate::renderer::equation::tokenizer::tokenize;
        use crate::renderer::hwpunit_to_px;

        if let Some(s) = json_str(props_json, "script") {
            eq.script = s;
        }
        if let Some(fs) = json_u32(props_json, "fontSize") {
            eq.font_size = fs;
        }
        if let Some(c) = json_u32(props_json, "color") {
            eq.color = c;
        }
        if let Some(bl) = json_i32(props_json, "baseline") {
            eq.baseline = bl as i16;
        }
        if let Some(fn_) = json_str(props_json, "fontName") {
            eq.font_name = fn_;
        }
        Self::apply_common_obj_attr_from_json(&mut eq.common, props_json);

        let font_size_px = hwpunit_to_px(eq.font_size as i32, dpi);
        let tokens = tokenize(&eq.script);
        let ast = EqParser::new(tokens).parse();
        let layout_box = EqLayout::new(font_size_px).layout(&ast);
        let new_w = crate::renderer::px_to_hwpunit(layout_box.width, dpi).max(0) as u32;
        let new_h = crate::renderer::px_to_hwpunit(layout_box.height, dpi).max(0) as u32;
        eq.common.width = new_w;
        eq.common.height = new_h;
    }
    pub fn get_equation_properties_native(
        &self,
        section_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
        cell_idx: Option<usize>,
        cell_para_idx: Option<usize>,
    ) -> Result<String, HwpError> {
        let eq = self.find_equation_ref(
            section_idx,
            parent_para_idx,
            control_idx,
            cell_idx,
            cell_para_idx,
        )?;

        Ok(Self::equation_properties_json(eq))
    }
    /// 수식 컨트롤의 속성을 변경한다 (네이티브).
    pub fn set_equation_properties_native(
        &mut self,
        section_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
        cell_idx: Option<usize>,
        cell_para_idx: Option<usize>,
        props_json: &str,
    ) -> Result<String, HwpError> {
        let dpi = self.dpi;
        let eq = self.find_equation_mut(
            section_idx,
            parent_para_idx,
            control_idx,
            cell_idx,
            cell_para_idx,
        )?;
        Self::apply_equation_properties(eq, dpi, props_json);

        // 표 셀 내 수식인 경우 표 dirty 플래그 설정
        if cell_idx.is_some() {
            if let Some(Control::Table(t)) = self.document.sections[section_idx].paragraphs
                [parent_para_idx]
                .controls
                .get_mut(control_idx)
            {
                t.dirty = true;
            }
        }

        // 재조판
        let section = &mut self.document.sections[section_idx];
        section.raw_stream = None;
        self.recompose_section(section_idx);
        self.paginate_if_needed();

        Ok(crate::document_core::helpers::json_ok())
    }
    /// 수식 스크립트를 SVG로 렌더링하여 반환한다 (미리보기 전용).
    pub fn render_equation_preview_native(
        &self,
        script: &str,
        font_size_hwpunit: u32,
        color: u32,
    ) -> Result<String, HwpError> {
        use crate::renderer::equation::layout::EqLayout;
        use crate::renderer::equation::parser::EqParser;
        use crate::renderer::equation::svg_render::{eq_color_to_svg, render_equation_svg};
        use crate::renderer::equation::tokenizer::tokenize;

        let font_size_px = crate::renderer::hwpunit_to_px(font_size_hwpunit as i32, self.dpi);
        let tokens = tokenize(script);
        let ast = EqParser::new(tokens).parse();
        let layout_box = EqLayout::new(font_size_px).layout(&ast);
        let color_str = eq_color_to_svg(color);
        let svg_fragment = render_equation_svg(&layout_box, &color_str, font_size_px);

        let w = layout_box.width;
        let h = layout_box.height;
        let svg = format!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {:.2} {:.2}\" width=\"{:.2}\" height=\"{:.2}\">{}</svg>",
            w, h, w, h, svg_fragment,
        );
        Ok(svg)
    }
    /// 수식(Equation) 컨트롤을 문단에서 삭제한다.
    pub fn delete_equation_control_native(
        &mut self,
        section_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
    ) -> Result<String, HwpError> {
        if section_idx >= self.document.sections.len() {
            return Err(HwpError::RenderError(format!(
                "구역 인덱스 {} 범위 초과",
                section_idx
            )));
        }
        let section = &mut self.document.sections[section_idx];
        if parent_para_idx >= section.paragraphs.len() {
            return Err(HwpError::RenderError(format!(
                "문단 인덱스 {} 범위 초과",
                parent_para_idx
            )));
        }
        let para = &mut section.paragraphs[parent_para_idx];
        if control_idx >= para.controls.len() {
            return Err(HwpError::RenderError(format!(
                "컨트롤 인덱스 {} 범위 초과",
                control_idx
            )));
        }
        if !matches!(&para.controls[control_idx], Control::Equation(_)) {
            return Err(HwpError::RenderError(
                "지정된 컨트롤이 수식이 아닙니다".to_string(),
            ));
        }

        let text_chars: Vec<char> = para.text.chars().collect();
        let mut ci = 0usize;
        let mut prev_end: u32 = 0;
        let mut gap_start: Option<u32> = None;
        'outer: for i in 0..text_chars.len() {
            let offset = if i < para.char_offsets.len() {
                para.char_offsets[i]
            } else {
                prev_end
            };
            while prev_end + 8 <= offset && ci < para.controls.len() {
                if ci == control_idx {
                    gap_start = Some(prev_end);
                    break 'outer;
                }
                ci += 1;
                prev_end += 8;
            }
            let char_size: u32 = if text_chars[i] == '\t' {
                8
            } else if text_chars[i].len_utf16() == 2 {
                2
            } else {
                1
            };
            prev_end = offset + char_size;
        }
        if gap_start.is_none() {
            while ci < para.controls.len() {
                if ci == control_idx {
                    gap_start = Some(prev_end);
                    break;
                }
                ci += 1;
                prev_end += 8;
            }
        }

        if let Some(gs) = gap_start {
            let threshold = gs + 8;
            for offset in para.char_offsets.iter_mut() {
                if *offset >= threshold {
                    *offset -= 8;
                }
            }
        }

        para.controls.remove(control_idx);
        if control_idx < para.ctrl_data_records.len() {
            para.ctrl_data_records.remove(control_idx);
        }
        if para.char_count >= 8 {
            para.char_count -= 8;
        }

        Self::reflow_paragraph_line_segs_after_control_delete(para, &self.styles, self.dpi);
        section.raw_stream = None;
        self.recompose_section(section_idx);
        self.paginate_if_needed();

        self.event_log.push(DocumentEvent::PictureDeleted {
            section: section_idx,
            para: parent_para_idx,
            ctrl: control_idx,
        });
        Ok("{\"ok\":true}".to_string())
    }

    // ─── 각주 삽입/삭제 API ──────────────────────────────
    /// 본문 문단에 수식을 삽입한다 (표 셀/글상자 내부는 미지원).
    /// 커서 위치에 수식 컨트롤을 추가한다.
    /// 반환: JSON `{"ok":true, "paraIdx":N, "controlIdx":N}`
    pub fn insert_equation_native(
        &mut self,
        section_idx: usize,
        para_idx: usize,
        char_offset: usize,
        script: &str,
        font_size: u32,
        color: u32,
    ) -> Result<String, HwpError> {
        use crate::model::control::Equation;
        use crate::model::shape::CommonObjAttr;
        use crate::parser::tags::CTRL_EQUATION;

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

        let equation = Equation {
            common: CommonObjAttr {
                ctrl_id: CTRL_EQUATION,
                treat_as_char: true,
                width: 0,
                height: 0,
                ..Default::default()
            },
            script: script.to_string(),
            font_size,
            color,
            font_name: "HYhwpEQ".to_string(),
            ..Default::default()
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
            .insert(insert_idx, Control::Equation(Box::new(equation)));
        paragraph.ctrl_data_records.insert(insert_idx, None);

        if !paragraph.char_offsets.is_empty() {
            let text_len = paragraph.text.chars().count();
            let safe_offset = char_offset.min(text_len);
            for co in paragraph.char_offsets[safe_offset..].iter_mut() {
                *co += 8;
            }
        }
        paragraph.char_count += 8;
        paragraph.control_mask |= 1u32 << 11;
        paragraph.has_para_text = true;

        // 본문 문단 리플로우
        {
            use crate::renderer::composer::reflow_line_segs;
            use crate::renderer::hwpunit_to_px;
            let page_def = &self.document.sections[section_idx].section_def.page_def;
            let text_width =
                page_def.width as i32 - page_def.margin_left as i32 - page_def.margin_right as i32;
            let available_width = hwpunit_to_px(text_width, self.dpi);
            let para_style = self.styles.para_styles.get(
                self.document.sections[section_idx].paragraphs[para_idx].para_shape_id as usize,
            );
            let margin_left = para_style.map(|s| s.margin_left).unwrap_or(0.0);
            let margin_right = para_style.map(|s| s.margin_right).unwrap_or(0.0);
            let final_width = (available_width - margin_left - margin_right).max(0.0);
            let body_para = &mut self.document.sections[section_idx].paragraphs[para_idx];
            reflow_line_segs(body_para, final_width, &self.styles, self.dpi);
        }

        self.recompose_section(section_idx);
        self.paginate_if_needed();
        self.invalidate_page_tree_cache();

        self.event_log.push(DocumentEvent::PictureInserted {
            section: section_idx,
            para: para_idx,
        });
        Ok(format!(
            "{{\"ok\":true,\"paraIdx\":{},\"controlIdx\":{}}}",
            para_idx, insert_idx
        ))
    }
}

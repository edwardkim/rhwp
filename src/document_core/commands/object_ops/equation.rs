//! 수식 native 명령 (object_ops 분할, #1904).

use crate::document_core::DocumentCore;
use crate::error::HwpError;
use crate::model::control::Control;
use crate::model::event::DocumentEvent;

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
        _dpi: f64,
        props_json: &str,
    ) {
        use crate::document_core::helpers::{json_i32, json_str, json_u32};

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

        // [#5890] 파생(자동 크기)은 봉지가 크기를 지정하지 않은 축에만 적용한다.
        // 종전에는 무조건 덧써서 getter 가 낸 봉지를 그대로 먹여도 크기가 바뀌었다
        // (get∘set ≠ 항등) — 속성 봉지만으로 되돌릴 수 없는 조작이 되고,
        // apply_common_obj_attr_from_json 이 방금 반영한 width/height 도 조용히 무시됐다.
        // UI 다이얼로그는 변경된 키만 담은 부분 봉지를 보내므로(width/height 미포함)
        // 스크립트·글자크기 편집의 자동 크기 재계산은 종전대로 동작한다.
        let explicit_width = json_u32(props_json, "width").is_some();
        let explicit_height = json_u32(props_json, "height").is_some();
        if !explicit_width || !explicit_height {
            let (width, height) =
                crate::renderer::equation::intrinsic_size_hwp(&eq.script, eq.font_size);
            if !explicit_width {
                eq.common.width = width;
            }
            if !explicit_height {
                eq.common.height = height;
            }
        }

        // [#5890] raw 패스스루 무효화는 파괴가 아니라 판정으로 한다.
        // serialize_equation_control 은 raw_ctrl_data 가 비어있지 않으면 원본 CTRL_HEADER
        // 바이트를 방출하지만, #4495 봉인이 서 있으면 `common` 변경만으로 봉인이 어긋나
        // 저장기가 IR 합성으로 내려간다 — 원본 바이트를 지우지 않아도 편집(크기/위치/
        // treat_as_char)은 .hwp 저장에 반영된다. 지우지 않으면 `common` 을 되돌렸을 때
        // 봉인이 다시 맞아 원본 바이트가 그대로 살아난다(속성 봉지만으로 되돌리는 참 역연산).
        // 봉인이 없는 raw(파서를 거치지 않은 합성 IR·어댑터 산출)는 판정 근거가 없어
        // 종전대로 비운다(table_ops 셀 편집 가드, adapt_equation 의 hwpx→hwp 변환과 동형).
        // EQEDIT 자식 레코드(script/font)는 어느 쪽이든 IR 로 재생성되므로 무관.
        if eq.raw_ctrl_seal.is_none() {
            eq.raw_ctrl_data.clear();
        }
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
        use crate::model::shape::{CommonObjAttr, HorzRelTo, TextWrap, VertRelTo};
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

        let (width, height) = crate::renderer::equation::intrinsic_size_hwp(script, font_size);
        let equation_order = self.document.sections[section_idx]
            .paragraphs
            .iter()
            .flat_map(|paragraph| paragraph.controls.iter())
            .filter(|control| matches!(control, Control::Equation(_)))
            .count() as u32;
        let equation_instance_order = self
            .document
            .sections
            .iter()
            .flat_map(|section| section.paragraphs.iter())
            .flat_map(|paragraph| paragraph.controls.iter())
            .filter(|control| matches!(control, Control::Equation(_)))
            .count();
        // 한컴 계열 0x44 접두는 유지하되, 접두와 겹치지 않는 하위 26비트에 문서 전체
        // 수식 순서를 배정한다. 구역 번호를 OR하면 구역 0과 64가 같은 ID가 된다.
        let equation_instance_sequence = u32::try_from(equation_instance_order)
            .ok()
            .and_then(|order| order.checked_add(1))
            .filter(|&order| order <= 0x03ff_ffff)
            .ok_or_else(|| {
                HwpError::RenderError("수식 instance ID를 더 이상 배정할 수 없습니다.".to_string())
            })?;
        let instance_id = 0x4400_0000 | equation_instance_sequence;
        let equation = Equation {
            common: CommonObjAttr {
                ctrl_id: CTRL_EQUATION,
                // 한컴 저장본의 인라인 수식 계약: 문단 기준 위치, 절대 크기,
                // 글자처럼 취급, 글과 함께 이동, 위아래 배치.
                attr: 0x0C2A_2311,
                treat_as_char: true,
                width,
                height,
                z_order: equation_order as i32,
                margin: crate::model::Padding {
                    left: 56,
                    right: 56,
                    top: 0,
                    bottom: 0,
                },
                instance_id,
                flow_with_text: true,
                vert_rel_to: VertRelTo::Para,
                horz_rel_to: HorzRelTo::Para,
                text_wrap: TextWrap::TopAndBottom,
                hwp5_gen_shape_attr_bit26: true,
                description: "수식입니다.".to_string(),
                ..Default::default()
            },
            script: script.to_string(),
            font_size,
            color,
            baseline: 85,
            version_info: "Equation Version 60".to_string(),
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

        // [#3214] controls 기준 인덱스를 ctrl_data_records 에 그대로 쓰기 전에 정렬한다.
        paragraph.align_ctrl_data_records();
        paragraph
            .controls
            .insert(insert_idx, Control::Equation(Box::new(equation)));
        paragraph.ctrl_data_records.insert(insert_idx, None);

        paragraph.shift_for_inline_control_insert(char_offset);
        paragraph.char_count += 8;
        paragraph.control_mask |= 1u32 << 11;
        paragraph.has_para_text = true;

        // 본문 문단 리플로우
        // 본문 문단의 상자는 쪽 폭이 아니라 **열** 폭에서 나온다. 직접 계산하면
        // ColumnDef(다단)·margin_gutter·가로 용지·양면 짝수쪽 여백 교환·손상
        // PageDef 의 A4 폴백을 모두 놓친다 — 그리고 그 결과가 디스크로 나간다.
        // 대화형 편집의 관문과 같은 한 곳을 쓴다.
        self.reflow_paragraph(section_idx, para_idx);

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

#[cfg(test)]
mod tests {
    use crate::document_core::DocumentCore;
    use crate::model::control::Equation;

    /// 봉인이 없는 raw(파서를 거치지 않은 합성 IR·어댑터 산출)는 저장기가 원본
    /// CTRL_HEADER 를 무조건 방출하므로, 판정 근거가 없어 종전대로 비워야 한다.
    /// 비우지 않으면 크기/위치 편집이 저장에서 원복된다.
    #[test]
    fn apply_equation_properties_clears_unsealed_raw_ctrl_data() {
        let mut eq = Equation {
            script: "1 over 2".to_string(),
            font_size: 1000,
            raw_ctrl_data: vec![0xAB; 16],
            ..Default::default()
        };
        assert!(eq.raw_ctrl_seal.is_none(), "합성 IR 은 봉인이 없다");
        DocumentCore::apply_equation_properties(&mut eq, 96.0, r#"{"width":5000,"height":4000}"#);
        assert!(
            eq.raw_ctrl_data.is_empty(),
            "봉인 없는 raw_ctrl_data 는 비워져야 편집이 .hwp 저장에 반영된다"
        );
    }
}

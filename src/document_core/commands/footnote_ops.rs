//! 각주 내용 편집 관련 native 메서드

use crate::document_core::DocumentCore;
use crate::error::HwpError;
use crate::model::control::Control;
use crate::model::event::DocumentEvent;
use crate::model::paragraph::Paragraph;
use crate::renderer::composer::reflow_line_segs;

impl DocumentCore {
    /// 각주 컨트롤 내부 문단의 가변 참조를 얻는다.
    fn get_footnote_paragraph_mut(
        &mut self,
        section_idx: usize,
        para_idx: usize,
        control_idx: usize,
        fn_para_idx: usize,
    ) -> Result<&mut Paragraph, HwpError> {
        let section = self.document.sections.get_mut(section_idx).ok_or_else(|| {
            HwpError::RenderError(format!("구역 인덱스 {} 범위 초과", section_idx))
        })?;
        let para = section
            .paragraphs
            .get_mut(para_idx)
            .ok_or_else(|| HwpError::RenderError(format!("문단 인덱스 {} 범위 초과", para_idx)))?;
        let ctrl = para.controls.get_mut(control_idx).ok_or_else(|| {
            HwpError::RenderError(format!("컨트롤 인덱스 {} 범위 초과", control_idx))
        })?;
        match ctrl {
            Control::Footnote(f) => {
                let len = f.paragraphs.len();
                if fn_para_idx >= len {
                    return Err(HwpError::RenderError(format!(
                        "각주 문단 인덱스 {} 범위 초과 (총 {}개)",
                        fn_para_idx, len
                    )));
                }
                Ok(&mut f.paragraphs[fn_para_idx])
            }
            _ => Err(HwpError::RenderError(format!(
                "컨트롤 {}은 각주가 아닙니다",
                control_idx
            ))),
        }
    }

    /// 각주 컨트롤 내부 문단의 불변 참조를 얻는다.
    fn get_footnote_paragraph_ref(
        &self,
        section_idx: usize,
        para_idx: usize,
        control_idx: usize,
        fn_para_idx: usize,
    ) -> Option<&Paragraph> {
        let section = self.document.sections.get(section_idx)?;
        let para = section.paragraphs.get(para_idx)?;
        let ctrl = para.controls.get(control_idx)?;
        match ctrl {
            Control::Footnote(f) => f.paragraphs.get(fn_para_idx),
            _ => None,
        }
    }

    /// 각주 문단 리플로우
    pub(crate) fn reflow_footnote_paragraph(
        &mut self,
        section_idx: usize,
        para_idx: usize,
        control_idx: usize,
        fn_para_idx: usize,
    ) {
        use crate::renderer::hwpunit_to_px;

        // 각주 영역 폭 = 페이지 텍스트 영역 폭
        let available_width = {
            let section = &self.document.sections[section_idx];
            let page_def = &section.section_def.page_def;
            let text_width =
                page_def.width as i32 - page_def.margin_left as i32 - page_def.margin_right as i32;
            hwpunit_to_px(text_width, self.dpi)
        };

        // 문단 여백 적용
        let para_shape_id = match self.get_footnote_paragraph_ref(
            section_idx,
            para_idx,
            control_idx,
            fn_para_idx,
        ) {
            Some(p) => p.para_shape_id,
            None => return,
        };
        let para_style = self.styles.para_styles.get(para_shape_id as usize);
        let margin_left = para_style.map(|s| s.margin_left).unwrap_or(0.0);
        let margin_right = para_style.map(|s| s.margin_right).unwrap_or(0.0);
        let final_width = (available_width - margin_left - margin_right).max(0.0);

        // 가변 참조로 리플로우 실행
        let section = &mut self.document.sections[section_idx];
        let ctrl = &mut section.paragraphs[para_idx].controls[control_idx];
        if let Control::Footnote(f) = ctrl {
            if let Some(para) = f.paragraphs.get_mut(fn_para_idx) {
                reflow_line_segs(para, final_width, &self.styles, self.dpi);
            }
        }
    }

    /// 각주 문단 정보를 반환한다.
    /// JSON: `{"ok":true,"paraCount":N,"textLen":N,"text":"..."}`
    pub fn get_footnote_info_native(
        &self,
        section_idx: usize,
        para_idx: usize,
        control_idx: usize,
    ) -> Result<String, HwpError> {
        let section = self.document.sections.get(section_idx).ok_or_else(|| {
            HwpError::RenderError(format!("구역 인덱스 {} 범위 초과", section_idx))
        })?;
        let para = section
            .paragraphs
            .get(para_idx)
            .ok_or_else(|| HwpError::RenderError(format!("문단 인덱스 {} 범위 초과", para_idx)))?;
        let ctrl = para.controls.get(control_idx).ok_or_else(|| {
            HwpError::RenderError(format!("컨트롤 인덱스 {} 범위 초과", control_idx))
        })?;
        match ctrl {
            Control::Footnote(f) => {
                let para_count = f.paragraphs.len();
                let texts: Vec<String> = f
                    .paragraphs
                    .iter()
                    .map(|p| p.text.replace('\\', "\\\\").replace('"', "\\\""))
                    .collect();
                let total_len: usize = f.paragraphs.iter().map(|p| p.text.chars().count()).sum();
                Ok(format!(
                    "{{\"ok\":true,\"paraCount\":{},\"totalTextLen\":{},\"number\":{},\"texts\":[{}]}}",
                    para_count,
                    total_len,
                    f.number,
                    texts.iter().map(|t| format!("\"{}\"", t)).collect::<Vec<_>>().join(","),
                ))
            }
            _ => Err(HwpError::RenderError(format!(
                "컨트롤 {}은 각주가 아닙니다",
                control_idx
            ))),
        }
    }

    /// 각주 내 텍스트 삽입
    pub fn insert_text_in_footnote_native(
        &mut self,
        section_idx: usize,
        para_idx: usize,
        control_idx: usize,
        fn_para_idx: usize,
        char_offset: usize,
        text: &str,
    ) -> Result<String, HwpError> {
        let new_chars_count = text.chars().count();
        let fn_para =
            self.get_footnote_paragraph_mut(section_idx, para_idx, control_idx, fn_para_idx)?;
        fn_para.insert_text_at(char_offset, text);

        self.reflow_footnote_paragraph(section_idx, para_idx, control_idx, fn_para_idx);

        self.document.sections[section_idx].raw_stream = None;
        self.mark_section_dirty(section_idx);
        self.paginate_if_needed();

        let new_offset = char_offset + new_chars_count;
        self.event_log.push(DocumentEvent::TextInserted {
            section: section_idx,
            para: para_idx,
            offset: char_offset,
            len: new_chars_count,
        });
        Ok(super::super::helpers::json_ok_with(&format!(
            "\"charOffset\":{}",
            new_offset
        )))
    }

    /// 각주 내 텍스트 삭제
    pub fn delete_text_in_footnote_native(
        &mut self,
        section_idx: usize,
        para_idx: usize,
        control_idx: usize,
        fn_para_idx: usize,
        char_offset: usize,
        count: usize,
    ) -> Result<String, HwpError> {
        let fn_para =
            self.get_footnote_paragraph_mut(section_idx, para_idx, control_idx, fn_para_idx)?;
        fn_para.delete_text_at(char_offset, count);

        self.reflow_footnote_paragraph(section_idx, para_idx, control_idx, fn_para_idx);

        self.document.sections[section_idx].raw_stream = None;
        self.mark_section_dirty(section_idx);
        self.paginate_if_needed();

        self.event_log.push(DocumentEvent::TextDeleted {
            section: section_idx,
            para: para_idx,
            offset: char_offset,
            count,
        });
        Ok(super::super::helpers::json_ok_with(&format!(
            "\"charOffset\":{}",
            char_offset
        )))
    }

    /// 각주 내 문단 분할 (Enter 키)
    pub fn split_paragraph_in_footnote_native(
        &mut self,
        section_idx: usize,
        para_idx: usize,
        control_idx: usize,
        fn_para_idx: usize,
        char_offset: usize,
    ) -> Result<String, HwpError> {
        // 문단 분할
        let new_para = {
            let section = self.document.sections.get_mut(section_idx).ok_or_else(|| {
                HwpError::RenderError(format!("구역 인덱스 {} 범위 초과", section_idx))
            })?;
            let para = section.paragraphs.get_mut(para_idx).ok_or_else(|| {
                HwpError::RenderError(format!("문단 인덱스 {} 범위 초과", para_idx))
            })?;
            let ctrl = para.controls.get_mut(control_idx).ok_or_else(|| {
                HwpError::RenderError(format!("컨트롤 인덱스 {} 범위 초과", control_idx))
            })?;
            match ctrl {
                Control::Footnote(f) => {
                    if fn_para_idx >= f.paragraphs.len() {
                        return Err(HwpError::RenderError(format!(
                            "각주 문단 인덱스 {} 범위 초과",
                            fn_para_idx
                        )));
                    }
                    f.paragraphs[fn_para_idx].split_at(char_offset)
                }
                _ => {
                    return Err(HwpError::RenderError(
                        "컨트롤이 각주가 아닙니다".to_string(),
                    ))
                }
            }
        };

        // 새 문단 삽입
        let new_para_idx = fn_para_idx + 1;
        {
            let ctrl =
                &mut self.document.sections[section_idx].paragraphs[para_idx].controls[control_idx];
            if let Control::Footnote(f) = ctrl {
                f.paragraphs.insert(new_para_idx, new_para);
            }
        }

        // 리플로우
        self.reflow_footnote_paragraph(section_idx, para_idx, control_idx, fn_para_idx);
        self.reflow_footnote_paragraph(section_idx, para_idx, control_idx, new_para_idx);

        self.document.sections[section_idx].raw_stream = None;
        self.mark_section_dirty(section_idx);
        self.paginate_if_needed();

        self.event_log.push(DocumentEvent::ParagraphSplit {
            section: section_idx,
            para: para_idx,
            offset: char_offset,
        });
        Ok(super::super::helpers::json_ok_with(&format!(
            "\"fnParaIndex\":{},\"charOffset\":0",
            new_para_idx
        )))
    }

    /// 각주 내 문단 병합 (Backspace at start)
    pub fn merge_paragraph_in_footnote_native(
        &mut self,
        section_idx: usize,
        para_idx: usize,
        control_idx: usize,
        fn_para_idx: usize,
    ) -> Result<String, HwpError> {
        if fn_para_idx == 0 {
            return Err(HwpError::RenderError(
                "첫 번째 문단은 이전 문단과 병합할 수 없습니다".to_string(),
            ));
        }

        let merge_offset;
        {
            let section = self.document.sections.get_mut(section_idx).ok_or_else(|| {
                HwpError::RenderError(format!("구역 인덱스 {} 범위 초과", section_idx))
            })?;
            let para = section.paragraphs.get_mut(para_idx).ok_or_else(|| {
                HwpError::RenderError(format!("문단 인덱스 {} 범위 초과", para_idx))
            })?;
            let ctrl = para.controls.get_mut(control_idx).ok_or_else(|| {
                HwpError::RenderError(format!("컨트롤 인덱스 {} 범위 초과", control_idx))
            })?;
            match ctrl {
                Control::Footnote(f) => {
                    if fn_para_idx >= f.paragraphs.len() {
                        return Err(HwpError::RenderError(format!(
                            "각주 문단 인덱스 {} 범위 초과",
                            fn_para_idx
                        )));
                    }
                    merge_offset = f.paragraphs[fn_para_idx - 1].text.chars().count();
                    let removed = f.paragraphs.remove(fn_para_idx);
                    f.paragraphs[fn_para_idx - 1].merge_from(&removed);
                }
                _ => {
                    return Err(HwpError::RenderError(
                        "컨트롤이 각주가 아닙니다".to_string(),
                    ))
                }
            }
        }

        let prev_idx = fn_para_idx - 1;
        self.reflow_footnote_paragraph(section_idx, para_idx, control_idx, prev_idx);

        self.document.sections[section_idx].raw_stream = None;
        self.mark_section_dirty(section_idx);
        self.paginate_if_needed();

        self.event_log.push(DocumentEvent::ParagraphMerged {
            section: section_idx,
            para: para_idx,
        });
        Ok(super::super::helpers::json_ok_with(&format!(
            "\"fnParaIndex\":{},\"charOffset\":{}",
            prev_idx, merge_offset
        )))
    }
}

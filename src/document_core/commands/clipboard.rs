//! 내부 클립보드 + HTML 내보내기 관련 native 메서드

use crate::model::control::Control;
use crate::model::paragraph::Paragraph;
use crate::document_core::{DocumentCore, ClipboardData};
use crate::error::HwpError;
use crate::model::event::DocumentEvent;
use super::super::helpers::{
    color_ref_to_css, clipboard_escape_html, clipboard_color_to_css,
    border_line_type_to_u8_val, detect_clipboard_image_mime,
    utf16_pos_to_char_idx, get_textbox_from_shape,
};

impl DocumentCore {
    pub fn has_internal_clipboard_native(&self) -> bool {
        self.clipboard.is_some()
    }

    /// 내부 클립보드의 플레인 텍스트를 반환한다.
    pub fn get_clipboard_text_native(&self) -> String {
        self.clipboard.as_ref()
            .map(|c| c.plain_text.clone())
            .unwrap_or_default()
    }

    /// 내부 클립보드를 초기화한다.
    pub fn clear_clipboard_native(&mut self) {
        self.clipboard = None;
    }

    /// 선택 영역을 내부 클립보드에 복사한다.
    ///
    /// 같은 구역 내 start ~ end 범위의 문단을 클립보드에 저장.
    /// 반환값: JSON `{"ok":true,"text":"<plain_text>"}`
    pub fn copy_selection_native(
        &mut self,
        section_idx: usize,
        start_para_idx: usize,
        start_char_offset: usize,
        end_para_idx: usize,
        end_char_offset: usize,
    ) -> Result<String, HwpError> {
        // 인덱스 범위 검증
        if section_idx >= self.document.sections.len() {
            return Err(HwpError::RenderError(format!(
                "구역 인덱스 {} 범위 초과", section_idx
            )));
        }
        let section = &self.document.sections[section_idx];
        if start_para_idx >= section.paragraphs.len() || end_para_idx >= section.paragraphs.len() {
            return Err(HwpError::RenderError(format!(
                "문단 인덱스 범위 초과 (start={}, end={}, total={})",
                start_para_idx, end_para_idx, section.paragraphs.len()
            )));
        }
        if start_para_idx > end_para_idx {
            return Err(HwpError::RenderError("시작 위치가 끝 위치보다 뒤에 있음".to_string()));
        }

        let mut clip_paragraphs = Vec::new();

        if start_para_idx == end_para_idx {
            // 단일 문단 내 선택
            let mut para = section.paragraphs[start_para_idx].clone();
            let text_len = para.text.chars().count();

            // 오른쪽 잘라내기 (end_offset 이후 제거)
            if end_char_offset < text_len {
                let _ = para.split_at(end_char_offset);
            }
            // 왼쪽 잘라내기 (start_offset 이전 제거)
            if start_char_offset > 0 {
                para = para.split_at(start_char_offset);
            }

            clip_paragraphs.push(para);
        } else {
            // 다중 문단 선택
            // 첫 번째 문단: start_offset부터 끝까지
            let mut first = section.paragraphs[start_para_idx].clone();
            if start_char_offset > 0 {
                first = first.split_at(start_char_offset);
            }
            clip_paragraphs.push(first);

            // 중간 문단: 전체 복사
            for i in (start_para_idx + 1)..end_para_idx {
                clip_paragraphs.push(section.paragraphs[i].clone());
            }

            // 마지막 문단: 처음부터 end_offset까지
            let mut last = section.paragraphs[end_para_idx].clone();
            let last_text_len = last.text.chars().count();
            if end_char_offset < last_text_len {
                let _ = last.split_at(end_char_offset);
            }
            clip_paragraphs.push(last);
        }

        // 구조적 컨트롤(SectionDef, ColumnDef 등) 제거 — 텍스트 복사에 불필요
        for para in &mut clip_paragraphs {
            para.controls.retain(|ctrl| !matches!(ctrl,
                Control::SectionDef(_) | Control::ColumnDef(_)
            ));
        }

        // 플레인 텍스트 추출
        let plain_text: String = clip_paragraphs.iter()
            .map(|p| p.text.as_str())
            .collect::<Vec<_>>()
            .join("\n");

        let escaped = super::super::helpers::json_escape(&plain_text);

        self.clipboard = Some(ClipboardData {
            paragraphs: clip_paragraphs,
            plain_text: plain_text.clone(),
        });

        Ok(super::super::helpers::json_ok_with(&format!("\"text\":\"{}\"", escaped)))
    }

    /// 표 셀 내부 선택 영역을 내부 클립보드에 복사한다.
    pub fn copy_selection_in_cell_native(
        &mut self,
        section_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
        cell_idx: usize,
        start_cell_para_idx: usize,
        start_char_offset: usize,
        end_cell_para_idx: usize,
        end_char_offset: usize,
    ) -> Result<String, HwpError> {
        // 셀 문단 리스트 접근
        let cell_paragraphs = {
            let section = self.document.sections.get(section_idx)
                .ok_or_else(|| HwpError::RenderError(format!("구역 {} 범위 초과", section_idx)))?;
            let para = section.paragraphs.get(parent_para_idx)
                .ok_or_else(|| HwpError::RenderError(format!("문단 {} 범위 초과", parent_para_idx)))?;
            let table = match para.controls.get(control_idx) {
                Some(Control::Table(t)) => t,
                _ => return Err(HwpError::RenderError("표가 아님".to_string())),
            };
            let cell = table.cells.get(cell_idx)
                .ok_or_else(|| HwpError::RenderError(format!("셀 {} 범위 초과", cell_idx)))?;
            &cell.paragraphs
        };

        if start_cell_para_idx >= cell_paragraphs.len() || end_cell_para_idx >= cell_paragraphs.len() {
            return Err(HwpError::RenderError("셀 문단 인덱스 범위 초과".to_string()));
        }

        let mut clip_paragraphs = Vec::new();

        if start_cell_para_idx == end_cell_para_idx {
            let mut para = cell_paragraphs[start_cell_para_idx].clone();
            let text_len = para.text.chars().count();
            if end_char_offset < text_len {
                let _ = para.split_at(end_char_offset);
            }
            if start_char_offset > 0 {
                para = para.split_at(start_char_offset);
            }
            clip_paragraphs.push(para);
        } else {
            let mut first = cell_paragraphs[start_cell_para_idx].clone();
            if start_char_offset > 0 {
                first = first.split_at(start_char_offset);
            }
            clip_paragraphs.push(first);

            for i in (start_cell_para_idx + 1)..end_cell_para_idx {
                clip_paragraphs.push(cell_paragraphs[i].clone());
            }

            let mut last = cell_paragraphs[end_cell_para_idx].clone();
            let last_text_len = last.text.chars().count();
            if end_char_offset < last_text_len {
                let _ = last.split_at(end_char_offset);
            }
            clip_paragraphs.push(last);
        }

        let plain_text: String = clip_paragraphs.iter()
            .map(|p| p.text.as_str())
            .collect::<Vec<_>>()
            .join("\n");

        let escaped = super::super::helpers::json_escape(&plain_text);

        self.clipboard = Some(ClipboardData {
            paragraphs: clip_paragraphs,
            plain_text: plain_text.clone(),
        });

        Ok(super::super::helpers::json_ok_with(&format!("\"text\":\"{}\"", escaped)))
    }

    /// 컨트롤 객체(표, 이미지, 도형)를 내부 클립보드에 복사한다.
    pub fn copy_control_native(
        &mut self,
        section_idx: usize,
        para_idx: usize,
        control_idx: usize,
    ) -> Result<String, HwpError> {
        let section = self.document.sections.get(section_idx)
            .ok_or_else(|| HwpError::RenderError(format!("구역 {} 범위 초과", section_idx)))?;
        let para = section.paragraphs.get(para_idx)
            .ok_or_else(|| HwpError::RenderError(format!("문단 {} 범위 초과", para_idx)))?;
        let control = para.controls.get(control_idx)
            .ok_or_else(|| HwpError::RenderError(format!("컨트롤 {} 범위 초과", control_idx)))?;

        // 컨트롤을 포함하는 단일 문단 생성
        // text는 비워둠 (serialize_para_text가 controls에서 확장 제어문자를 생성)
        // control_mask: 1 << ctrl_char_code (Table/Shape=0x000B→bit11=0x800 등)
        let ctrl_char_code = match control {
            Control::Table(_) | Control::Shape(_) | Control::Picture(_) => 0x000Bu16,
            Control::SectionDef(_) | Control::ColumnDef(_) => 0x0002u16,
            Control::Footnote(_) | Control::Endnote(_) => 0x0011u16,
            Control::Header(_) | Control::Footer(_) => 0x0010u16,
            Control::AutoNumber(_) | Control::NewNumber(_) => 0x0012u16,
            _ => 0x000Bu16,
        };
        // 컨트롤 치수에 맞는 line_segs 생성 (insert_picture_native 패턴)
        let ctrl_line_seg = {
            let ctrl_height = match control {
                Control::Picture(pic) => pic.common.height as i32,
                Control::Shape(shape) => shape.common().height as i32,
                _ => 0,
            };
            if ctrl_height > 0 {
                crate::model::paragraph::LineSeg {
                    text_start: 0,
                    line_height: ctrl_height,
                    text_height: ctrl_height,
                    baseline_distance: (ctrl_height * 850) / 1000,
                    line_spacing: 600,
                    tag: 0x00060000,
                    ..Default::default()
                }
            } else {
                crate::model::paragraph::LineSeg {
                    text_start: 0,
                    line_height: 400,
                    text_height: 400,
                    baseline_distance: 320,
                    tag: 0x00060000,
                    ..Default::default()
                }
            }
        };

        let clip_para = Paragraph {
            text: String::new(),
            char_count: 9, // 확장 제어문자(8 code units) + 문단끝(1)
            control_mask: 1u32 << ctrl_char_code,
            char_offsets: vec![],
            char_shapes: vec![crate::model::paragraph::CharShapeRef {
                start_pos: 0,
                char_shape_id: para.char_shape_id_at(0).unwrap_or(0),
            }],
            line_segs: vec![ctrl_line_seg],
            para_shape_id: para.para_shape_id,
            style_id: para.style_id,
            controls: vec![control.clone()],
            ctrl_data_records: vec![
                para.ctrl_data_records.get(control_idx).cloned().flatten(),
            ],
            has_para_text: true,
            ..Default::default()
        };

        let plain_text = match control {
            Control::Table(_) => "[표]".to_string(),
            Control::Picture(_) => "[그림]".to_string(),
            Control::Shape(_) => "[도형]".to_string(),
            _ => "[컨트롤]".to_string(),
        };

        self.clipboard = Some(ClipboardData {
            paragraphs: vec![clip_para],
            plain_text: plain_text.clone(),
        });

        Ok(super::super::helpers::json_ok_with(&format!("\"text\":\"{}\"", plain_text)))
    }

    /// 내부 클립보드의 내용을 캐럿 위치에 붙여넣는다 (본문 문단).
    pub fn paste_internal_native(
        &mut self,
        section_idx: usize,
        para_idx: usize,
        char_offset: usize,
    ) -> Result<String, HwpError> {
        let clip_paras = match &self.clipboard {
            Some(c) if !c.paragraphs.is_empty() => c.paragraphs.clone(),
            _ => return Ok("{\"ok\":false,\"error\":\"clipboard empty\"}".to_string()),
        };

        // 인덱스 검증
        if section_idx >= self.document.sections.len() {
            return Err(HwpError::RenderError(format!("구역 {} 범위 초과", section_idx)));
        }
        if para_idx >= self.document.sections[section_idx].paragraphs.len() {
            return Err(HwpError::RenderError(format!("문단 {} 범위 초과", para_idx)));
        }

        self.document.sections[section_idx].raw_stream = None;

        let clip_count = clip_paras.len();

        if clip_count == 1 && clip_paras[0].controls.is_empty() {
            // 단일 문단 텍스트 붙여넣기 (컨트롤 없음)
            let clip_text = clip_paras[0].text.clone();
            let clip_char_shapes = clip_paras[0].char_shapes.clone();
            let clip_char_offsets = clip_paras[0].char_offsets.clone();
            let new_chars = clip_text.chars().count();

            // 텍스트 삽입
            self.document.sections[section_idx].paragraphs[para_idx]
                .insert_text_at(char_offset, &clip_text);

            // 클립보드의 글자 모양 적용
            self.apply_clipboard_char_shapes(
                section_idx, para_idx, char_offset,
                &clip_char_shapes, &clip_char_offsets, new_chars,
            );

            self.reflow_paragraph(section_idx, para_idx);
            self.recompose_paragraph(section_idx, para_idx);
            self.paginate_if_needed();

            let new_offset = char_offset + new_chars;
            self.event_log.push(DocumentEvent::ContentPasted { section: section_idx, para: para_idx });
            return Ok(super::super::helpers::json_ok_with(&format!(
                "\"paraIdx\":{},\"charOffset\":{}", para_idx, new_offset
            )));
        }

        // 다중 문단 또는 컨트롤 포함 붙여넣기
        // 1. 현재 문단을 캐럿 위치에서 분할
        let right_half = self.document.sections[section_idx].paragraphs[para_idx]
            .split_at(char_offset);

        // 2. 왼쪽 절반에 첫 번째 클립보드 문단 병합
        self.document.sections[section_idx].paragraphs[para_idx]
            .merge_from(&clip_paras[0]);

        // 3. 나머지 클립보드 문단 삽입
        let mut insert_idx = para_idx + 1;
        for i in 1..clip_count {
            self.document.sections[section_idx].paragraphs
                .insert(insert_idx, clip_paras[i].clone());
            insert_idx += 1;
        }

        // 4. 마지막 삽입된 문단에 오른쪽 절반 병합
        let last_para_idx = insert_idx - 1;
        let merge_point = self.document.sections[section_idx].paragraphs[last_para_idx]
            .merge_from(&right_half);

        // 5. 영향받는 모든 문단 리플로우
        for i in para_idx..=last_para_idx {
            self.reflow_paragraph(section_idx, i);
        }

        // 6. 선택적 재구성: 삽입된 문단 composed 추가 + 영향 문단 재구성
        self.recompose_paragraph(section_idx, para_idx);
        for i in para_idx + 1..=last_para_idx {
            self.insert_composed_paragraph(section_idx, i);
        }
        self.paginate_if_needed();

        self.event_log.push(DocumentEvent::ContentPasted { section: section_idx, para: para_idx });
        Ok(super::super::helpers::json_ok_with(&format!(
            "\"paraIdx\":{},\"charOffset\":{}", last_para_idx, merge_point
        )))
    }

    /// 내부 클립보드의 내용을 표 셀 내부에 붙여넣는다.
    pub fn paste_internal_in_cell_native(
        &mut self,
        section_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
        cell_idx: usize,
        cell_para_idx: usize,
        char_offset: usize,
    ) -> Result<String, HwpError> {
        let clip_paras = match &self.clipboard {
            Some(c) if !c.paragraphs.is_empty() => c.paragraphs.clone(),
            _ => return Ok("{\"ok\":false,\"error\":\"clipboard empty\"}".to_string()),
        };

        // 셀 접근 검증
        let cell_para_count = {
            let section = self.document.sections.get(section_idx)
                .ok_or_else(|| HwpError::RenderError(format!("구역 {} 범위 초과", section_idx)))?;
            let para = section.paragraphs.get(parent_para_idx)
                .ok_or_else(|| HwpError::RenderError(format!("문단 {} 범위 초과", parent_para_idx)))?;
            match para.controls.get(control_idx) {
                Some(Control::Table(t)) => {
                    let cell = t.cells.get(cell_idx)
                        .ok_or_else(|| HwpError::RenderError(format!("셀 {} 범위 초과", cell_idx)))?;
                    cell.paragraphs.len()
                }
                Some(Control::Shape(s)) => {
                    let tb = super::super::helpers::get_textbox_from_shape(s)
                        .ok_or_else(|| HwpError::RenderError("글상자 없음".to_string()))?;
                    tb.paragraphs.len()
                }
                Some(Control::Picture(p)) => {
                    let cap = p.caption.as_ref()
                        .ok_or_else(|| HwpError::RenderError("캡션 없음".to_string()))?;
                    cap.paragraphs.len()
                }
                _ => return Err(HwpError::RenderError("표/글상자/캡션이 아님".to_string())),
            }
        };
        if cell_para_idx >= cell_para_count {
            return Err(HwpError::RenderError(format!("셀 문단 {} 범위 초과", cell_para_idx)));
        }

        self.document.sections[section_idx].raw_stream = None;

        let clip_count = clip_paras.len();

        // 셀 문단에 대한 가변 참조 얻기
        let cell_paras = {
            let section = &mut self.document.sections[section_idx];
            let para = &mut section.paragraphs[parent_para_idx];
            match &mut para.controls[control_idx] {
                Control::Table(t) => &mut t.cells[cell_idx].paragraphs,
                Control::Shape(s) => {
                    &mut super::super::helpers::get_textbox_from_shape_mut(s).unwrap().paragraphs
                }
                Control::Picture(p) => &mut p.caption.as_mut().unwrap().paragraphs,
                _ => unreachable!(),
            }
        };

        if clip_count == 1 && clip_paras[0].controls.is_empty() {
            // 단일 문단 텍스트 붙여넣기
            let clip_text = clip_paras[0].text.clone();
            let new_chars = clip_text.chars().count();

            cell_paras[cell_para_idx].insert_text_at(char_offset, &clip_text);

            // 클립보드 글자 모양 적용
            let clip_char_shapes = clip_paras[0].char_shapes.clone();
            let clip_char_offsets = clip_paras[0].char_offsets.clone();
            Self::apply_clipboard_char_shapes_to_para(
                &mut cell_paras[cell_para_idx], char_offset,
                &clip_char_shapes, &clip_char_offsets, new_chars,
            );

            // 셀 리플로우
            let _ = cell_paras;
            self.reflow_cell_paragraph(section_idx, parent_para_idx, control_idx, cell_idx, cell_para_idx);
            // 부모 컨트롤 dirty 마킹 + 재페이지네이션
            match self.document.sections[section_idx]
                .paragraphs[parent_para_idx].controls.get_mut(control_idx) {
                Some(Control::Table(t)) => { t.dirty = true; }
                _ => {}
            }
            self.mark_section_dirty(section_idx);
            self.paginate_if_needed();

            let new_offset = char_offset + new_chars;
            self.event_log.push(DocumentEvent::ContentPasted { section: section_idx, para: parent_para_idx });
            return Ok(super::super::helpers::json_ok_with(&format!(
                "\"cellParaIdx\":{},\"charOffset\":{}", cell_para_idx, new_offset
            )));
        }

        // 다중 문단 붙여넣기
        let right_half = cell_paras[cell_para_idx].split_at(char_offset);
        cell_paras[cell_para_idx].merge_from(&clip_paras[0]);

        let mut insert_idx = cell_para_idx + 1;
        for i in 1..clip_count {
            cell_paras.insert(insert_idx, clip_paras[i].clone());
            insert_idx += 1;
        }

        let last_para_idx = insert_idx - 1;
        let merge_point = cell_paras[last_para_idx].merge_from(&right_half);

        // 셀 리플로우 (모든 영향받는 문단)
        let _ = cell_paras;
        for i in cell_para_idx..=last_para_idx {
            self.reflow_cell_paragraph(section_idx, parent_para_idx, control_idx, cell_idx, i);
        }
        // 부모 컨트롤 dirty 마킹 + 재페이지네이션
        match self.document.sections[section_idx]
            .paragraphs[parent_para_idx].controls.get_mut(control_idx) {
            Some(Control::Table(t)) => { t.dirty = true; }
            _ => {}
        }
        self.mark_section_dirty(section_idx);
        self.paginate_if_needed();

        self.event_log.push(DocumentEvent::ContentPasted { section: section_idx, para: parent_para_idx });
        Ok(super::super::helpers::json_ok_with(&format!(
            "\"cellParaIdx\":{},\"charOffset\":{}", last_para_idx, merge_point
        )))
    }

    /// 클립보드의 글자 모양(CharShape)을 삽입된 텍스트 범위에 적용한다.
    pub(crate) fn apply_clipboard_char_shapes(
        &mut self,
        section_idx: usize,
        para_idx: usize,
        insert_offset: usize,
        clip_char_shapes: &[crate::model::paragraph::CharShapeRef],
        clip_char_offsets: &[u32],
        inserted_chars: usize,
    ) {
        Self::apply_clipboard_char_shapes_to_para(
            &mut self.document.sections[section_idx].paragraphs[para_idx],
            insert_offset, clip_char_shapes, clip_char_offsets, inserted_chars,
        );
    }

    /// 클립보드의 글자 모양을 특정 문단에 적용한다 (정적 메서드).
    pub(crate) fn apply_clipboard_char_shapes_to_para(
        para: &mut Paragraph,
        insert_offset: usize,
        clip_char_shapes: &[crate::model::paragraph::CharShapeRef],
        clip_char_offsets: &[u32],
        inserted_chars: usize,
    ) {
        if clip_char_shapes.is_empty() {
            return;
        }

        for i in 0..clip_char_shapes.len() {
            let cs = &clip_char_shapes[i];

            // UTF-16 위치를 char 인덱스로 변환
            let start_char_idx = clip_char_offsets.iter()
                .position(|&off| off >= cs.start_pos)
                .unwrap_or(0);

            let end_char_idx = if i + 1 < clip_char_shapes.len() {
                clip_char_offsets.iter()
                    .position(|&off| off >= clip_char_shapes[i + 1].start_pos)
                    .unwrap_or(inserted_chars)
            } else {
                inserted_chars
            };

            if start_char_idx < end_char_idx && end_char_idx <= inserted_chars {
                para.apply_char_shape_range(
                    insert_offset + start_char_idx,
                    insert_offset + end_char_idx,
                    cs.char_shape_id,
                );
            }
        }
    }

    /// 내부 클립보드에 붙여넣기 가능한 개체 컨트롤(표/그림/도형)이 포함되어 있는지 확인한다.
    /// SectionDef, ColumnDef 등 구조적 컨트롤은 개체가 아니므로 제외한다.
    pub fn clipboard_has_control_native(&self) -> bool {
        self.clipboard.as_ref()
            .map(|c| c.paragraphs.first().map(|p| {
                p.controls.iter().any(|ctrl| matches!(ctrl,
                    Control::Table(_) | Control::Picture(_) | Control::Shape(_)
                ))
            }).unwrap_or(false))
            .unwrap_or(false)
    }

    /// 내부 클립보드의 컨트롤 객체를 캐럿 위치에 붙여넣는다 (본문).
    ///
    /// 클립보드에 컨트롤이 없으면 `{"ok":false}` 반환.
    /// 반환값: JSON `{"ok":true,"paraIdx":<idx>,"controlIdx":0}`
    pub fn paste_control_native(
        &mut self,
        section_idx: usize,
        para_idx: usize,
        char_offset: usize,
    ) -> Result<String, HwpError> {
        // 클립보드에서 컨트롤 문단 확인
        let clip_para = match &self.clipboard {
            Some(c) => {
                match c.paragraphs.first() {
                    Some(p) if !p.controls.is_empty() => p.clone(),
                    _ => return Ok("{\"ok\":false,\"error\":\"no control in clipboard\"}".to_string()),
                }
            }
            None => return Ok("{\"ok\":false,\"error\":\"clipboard empty\"}".to_string()),
        };

        // 인덱스 검증
        if section_idx >= self.document.sections.len() {
            return Err(HwpError::RenderError(format!("구역 {} 범위 초과", section_idx)));
        }
        if para_idx >= self.document.sections[section_idx].paragraphs.len() {
            return Err(HwpError::RenderError(format!("문단 {} 범위 초과", para_idx)));
        }

        self.document.sections[section_idx].raw_stream = None;

        // 커서 위치 문단의 속성 상속 (빈 문단 생성용)
        let current_para = &self.document.sections[section_idx].paragraphs[para_idx];
        let default_char_shape_id: u32 = current_para.char_shapes.first()
            .map(|cs| cs.char_shape_id).unwrap_or(0);
        let default_para_shape_id: u16 = current_para.para_shape_id;

        // 편집 영역 폭
        let pd = &self.document.sections[section_idx].section_def.page_def;
        let content_width = (pd.width as i32 - pd.margin_left as i32 - pd.margin_right as i32).max(7200) as u32;

        // 삽입 위치 결정 (create_shape_control_native 패턴)
        let para = &self.document.sections[section_idx].paragraphs[para_idx];
        let is_empty_para = para.text.is_empty() && para.controls.is_empty();

        let insert_para_idx;
        if is_empty_para && char_offset == 0 {
            self.document.sections[section_idx].paragraphs[para_idx] = clip_para;
            insert_para_idx = para_idx;
        } else if char_offset == 0 && para.controls.is_empty() {
            self.document.sections[section_idx].paragraphs.insert(para_idx, clip_para);
            insert_para_idx = para_idx;
        } else {
            if char_offset > 0 && !para.text.is_empty() {
                let new_para = self.document.sections[section_idx].paragraphs[para_idx]
                    .split_at(char_offset);
                self.document.sections[section_idx].paragraphs.insert(para_idx + 1, new_para);
                self.document.sections[section_idx].paragraphs.insert(para_idx + 1, clip_para);
                insert_para_idx = para_idx + 1;
            } else {
                self.document.sections[section_idx].paragraphs.insert(para_idx + 1, clip_para);
                insert_para_idx = para_idx + 1;
            }
        }

        // 삽입된 문단의 line_segs 보정: 컨트롤 치수 반영
        // copy_control_native()에서 line_segs가 기본값(line_height:400, segment_width:0)으로
        // 하드코딩되므로, 실제 컨트롤 크기에 맞게 재설정한다.
        // (insert_picture_native 패턴: line_height=pic.height, segment_width=content_width)
        {
            let inserted = &mut self.document.sections[section_idx].paragraphs[insert_para_idx];
            let ctrl_height = inserted.controls.first().map(|ctrl| {
                match ctrl {
                    Control::Picture(pic) => pic.common.height as i32,
                    Control::Shape(shape) => shape.common().height as i32,
                    _ => 0,
                }
            }).unwrap_or(0);
            if let Some(ls) = inserted.line_segs.first_mut() {
                ls.segment_width = content_width as i32;
                if ctrl_height > 0 {
                    ls.line_height = ctrl_height;
                    ls.text_height = ctrl_height;
                    ls.baseline_distance = (ctrl_height * 850) / 1000;
                    ls.line_spacing = 600;
                }
            }
        }

        // 컨트롤 아래에 빈 문단 추가 (HWP 표준)
        let mut empty_raw = vec![0u8; 10];
        empty_raw[0..2].copy_from_slice(&1u16.to_le_bytes());
        empty_raw[4..6].copy_from_slice(&1u16.to_le_bytes());
        let empty_para = Paragraph {
            text: String::new(),
            char_count: 1,
            char_count_msb: false,
            control_mask: 0,
            para_shape_id: default_para_shape_id,
            style_id: 0,
            char_shapes: vec![crate::model::paragraph::CharShapeRef {
                start_pos: 0,
                char_shape_id: default_char_shape_id,
            }],
            line_segs: vec![crate::model::paragraph::LineSeg {
                text_start: 0,
                line_height: 1000,
                text_height: 1000,
                baseline_distance: 850,
                line_spacing: 600,
                segment_width: content_width as i32,
                tag: 0x00060000,
                ..Default::default()
            }],
            has_para_text: false,
            raw_header_extra: empty_raw,
            ..Default::default()
        };
        self.document.sections[section_idx].paragraphs.insert(insert_para_idx + 1, empty_para);

        // 리플로우 + 페이지네이션
        self.recompose_section(section_idx);
        self.paginate_if_needed();

        self.event_log.push(DocumentEvent::ContentPasted { section: section_idx, para: insert_para_idx });
        Ok(super::super::helpers::json_ok_with(&format!(
            "\"paraIdx\":{},\"controlIdx\":0", insert_para_idx
        )))
    }

    // === 클립보드 HTML 생성 ===

    /// 선택 영역을 HTML 문자열로 변환한다 (본문).
    pub fn export_selection_html_native(
        &self,
        section_idx: usize,
        start_para_idx: usize,
        start_char_offset: usize,
        end_para_idx: usize,
        end_char_offset: usize,
    ) -> Result<String, HwpError> {
        let section = self.document.sections.get(section_idx)
            .ok_or_else(|| HwpError::RenderError(format!("구역 {} 범위 초과", section_idx)))?;

        if start_para_idx >= section.paragraphs.len() || end_para_idx >= section.paragraphs.len() {
            return Err(HwpError::RenderError("문단 범위 초과".to_string()));
        }

        let mut html = String::from("<html><body>\n<!--StartFragment-->\n");

        for pi in start_para_idx..=end_para_idx {
            let para = &section.paragraphs[pi];
            let start = if pi == start_para_idx { Some(start_char_offset) } else { None };
            let end = if pi == end_para_idx { Some(end_char_offset) } else { None };
            html.push_str(&self.paragraph_to_html(para, start, end));
        }

        html.push_str("<!--EndFragment-->\n</body></html>");
        Ok(html)
    }

    /// 선택 영역을 HTML 문자열로 변환한다 (셀 내부).
    pub fn export_selection_in_cell_html_native(
        &self,
        section_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
        cell_idx: usize,
        start_cell_para_idx: usize,
        start_char_offset: usize,
        end_cell_para_idx: usize,
        end_char_offset: usize,
    ) -> Result<String, HwpError> {
        let section = self.document.sections.get(section_idx)
            .ok_or_else(|| HwpError::RenderError(format!("구역 {} 범위 초과", section_idx)))?;
        let para = section.paragraphs.get(parent_para_idx)
            .ok_or_else(|| HwpError::RenderError(format!("문단 {} 범위 초과", parent_para_idx)))?;
        let table = match para.controls.get(control_idx) {
            Some(Control::Table(t)) => t,
            _ => return Err(HwpError::RenderError("표가 아님".to_string())),
        };
        let cell = table.cells.get(cell_idx)
            .ok_or_else(|| HwpError::RenderError(format!("셀 {} 범위 초과", cell_idx)))?;

        let mut html = String::from("<html><body>\n<!--StartFragment-->\n");

        for pi in start_cell_para_idx..=end_cell_para_idx {
            if pi >= cell.paragraphs.len() { break; }
            let cpara = &cell.paragraphs[pi];
            let start = if pi == start_cell_para_idx { Some(start_char_offset) } else { None };
            let end = if pi == end_cell_para_idx { Some(end_char_offset) } else { None };
            html.push_str(&self.paragraph_to_html(cpara, start, end));
        }

        html.push_str("<!--EndFragment-->\n</body></html>");
        Ok(html)
    }

    /// 컨트롤 객체를 HTML 문자열로 변환한다.
    pub fn export_control_html_native(
        &self,
        section_idx: usize,
        para_idx: usize,
        control_idx: usize,
    ) -> Result<String, HwpError> {
        let section = self.document.sections.get(section_idx)
            .ok_or_else(|| HwpError::RenderError(format!("구역 {} 범위 초과", section_idx)))?;
        let para = section.paragraphs.get(para_idx)
            .ok_or_else(|| HwpError::RenderError(format!("문단 {} 범위 초과", para_idx)))?;
        let control = para.controls.get(control_idx)
            .ok_or_else(|| HwpError::RenderError(format!("컨트롤 {} 범위 초과", control_idx)))?;

        let mut html = String::from("<html><body>\n<!--StartFragment-->\n");
        html.push_str(&self.control_to_html(control));
        html.push_str("<!--EndFragment-->\n</body></html>");
        Ok(html)
    }

    /// 단일 문단을 HTML로 변환한다 (선택적 범위 지정).
    pub(crate) fn paragraph_to_html(
        &self,
        para: &Paragraph,
        start_offset: Option<usize>,
        end_offset: Option<usize>,
    ) -> String {
        let chars: Vec<char> = para.text.chars().collect();
        let start_idx = start_offset.unwrap_or(0).min(chars.len());
        let end_idx = end_offset.unwrap_or(chars.len()).min(chars.len());
        if start_idx >= end_idx { return String::new(); }

        // 문단 스타일 CSS
        let para_css = self.para_style_to_css(para.para_shape_id);
        let mut html = format!("<p style=\"margin:0;{}\">\n", para_css);

        // CharShapeRef 경계에서 스타일이 바뀌는 지점을 찾아 span 분할
        let style_ranges = self.get_char_style_ranges(para, start_idx, end_idx);

        for (range_start, range_end, char_shape_id) in &style_ranges {
            let segment: String = chars[*range_start..*range_end].iter()
                .filter(|c| !c.is_control() || **c == '\t')
                .collect();

            if segment.is_empty() { continue; }

            let css = self.char_style_to_css(*char_shape_id);
            html.push_str(&format!(
                "<span style=\"{}\">{}</span>",
                css, clipboard_escape_html(&segment)
            ));
        }

        html.push_str("</p>\n");
        html
    }

    /// 문단 내 char 인덱스 범위에서 CharShapeRef 경계를 기준으로 (start, end, char_shape_id) 목록을 반환한다.
    pub(crate) fn get_char_style_ranges(
        &self,
        para: &Paragraph,
        start_idx: usize,
        end_idx: usize,
    ) -> Vec<(usize, usize, u32)> {
        if para.char_shapes.is_empty() {
            return vec![(start_idx, end_idx, 0)];
        }

        // CharShapeRef의 start_pos (UTF-16) → char index 변환
        let mut boundaries: Vec<(usize, u32)> = Vec::new();
        for cs in &para.char_shapes {
            let char_idx = utf16_pos_to_char_idx(&para.char_offsets, cs.start_pos);
            boundaries.push((char_idx, cs.char_shape_id));
        }

        let mut ranges = Vec::new();
        for i in 0..boundaries.len() {
            let (bound_start, shape_id) = boundaries[i];
            let bound_end = if i + 1 < boundaries.len() {
                boundaries[i + 1].0
            } else {
                end_idx
            };

            // 범위와 교차하는 부분만 포함
            let rs = bound_start.max(start_idx);
            let re = bound_end.min(end_idx);
            if rs < re {
                ranges.push((rs, re, shape_id));
            }
        }

        // 시작점 이전에 스타일이 없으면 첫 CharShapeRef의 스타일 사용
        if ranges.is_empty() && !boundaries.is_empty() {
            let last_before = boundaries.iter().rev()
                .find(|(idx, _)| *idx <= start_idx)
                .map(|(_, id)| *id)
                .unwrap_or(boundaries[0].1);
            ranges.push((start_idx, end_idx, last_before));
        }

        ranges
    }

    /// CharShape ID → CSS 인라인 스타일 문자열 변환.
    pub(crate) fn char_style_to_css(&self, char_shape_id: u32) -> String {
        let cs = match self.styles.char_styles.get(char_shape_id as usize) {
            Some(s) => s,
            None => return String::new(),
        };

        let mut css = String::new();

        // font-family (한국어 + 영어 폰트 목록)
        let mut fonts: Vec<&str> = Vec::new();
        if !cs.font_family.is_empty() {
            fonts.push(&cs.font_family);
        }
        if cs.font_families.len() > 1 && !cs.font_families[1].is_empty()
            && cs.font_families[1] != cs.font_family
        {
            fonts.push(&cs.font_families[1]);
        }
        if !fonts.is_empty() {
            let font_list: Vec<String> = fonts.iter()
                .map(|f| format!("'{}'", clipboard_escape_html(f)))
                .collect();
            css.push_str(&format!("font-family:{};", font_list.join(",")));
        }

        // font-size (px → pt 변환: pt = px * 72 / 96)
        if cs.font_size > 0.0 {
            let pt = cs.font_size * 72.0 / self.dpi;
            css.push_str(&format!("font-size:{:.1}pt;", pt));
        }

        // font-weight / font-style
        if cs.bold { css.push_str("font-weight:bold;"); }
        if cs.italic { css.push_str("font-style:italic;"); }

        // color
        let color = clipboard_color_to_css(cs.text_color);
        css.push_str(&format!("color:{};", color));

        // text-decoration
        let has_underline = !matches!(cs.underline, crate::model::style::UnderlineType::None);
        if has_underline && cs.strikethrough {
            css.push_str("text-decoration:underline line-through;");
        } else if has_underline {
            css.push_str("text-decoration:underline;");
        } else if cs.strikethrough {
            css.push_str("text-decoration:line-through;");
        }

        // letter-spacing (0이 아닌 경우만)
        if cs.letter_spacing.abs() > 0.1 {
            css.push_str(&format!("letter-spacing:{:.1}px;", cs.letter_spacing));
        }

        css
    }

    /// ParaShape ID → CSS 인라인 스타일 문자열 변환.
    pub(crate) fn para_style_to_css(&self, para_shape_id: u16) -> String {
        let ps = match self.styles.para_styles.get(para_shape_id as usize) {
            Some(s) => s,
            None => return String::new(),
        };

        let mut css = String::new();

        // text-align
        let align = match ps.alignment {
            crate::model::style::Alignment::Left => "left",
            crate::model::style::Alignment::Right => "right",
            crate::model::style::Alignment::Center => "center",
            crate::model::style::Alignment::Justify => "justify",
            crate::model::style::Alignment::Distribute => "justify",
            crate::model::style::Alignment::Split => "justify",
        };
        css.push_str(&format!("text-align:{};", align));

        // margin-left / margin-right (px)
        if ps.margin_left > 0.1 {
            css.push_str(&format!("margin-left:{:.1}px;", ps.margin_left));
        }
        if ps.margin_right > 0.1 {
            css.push_str(&format!("margin-right:{:.1}px;", ps.margin_right));
        }

        // text-indent
        if ps.indent.abs() > 0.1 {
            css.push_str(&format!("text-indent:{:.1}px;", ps.indent));
        }

        // line-height
        match ps.line_spacing_type {
            crate::model::style::LineSpacingType::Percent => {
                css.push_str(&format!("line-height:{:.0}%;", ps.line_spacing));
            }
            crate::model::style::LineSpacingType::Fixed => {
                css.push_str(&format!("line-height:{:.1}px;", ps.line_spacing));
            }
            _ => {}
        }

        css
    }

    /// Control 객체를 HTML로 변환한다.
    pub(crate) fn control_to_html(&self, control: &Control) -> String {
        match control {
            Control::Table(table) => self.table_to_html(table),
            Control::Picture(pic) => self.picture_to_html(pic),
            _ => String::new(),
        }
    }

    /// Table 컨트롤을 HTML <table>로 변환한다.
    pub(crate) fn table_to_html(&self, table: &crate::model::table::Table) -> String {
        use crate::renderer::style_resolver::ResolvedBorderStyle;

        let mut html = String::from(
            "<table style=\"border-collapse:collapse;\" cellpadding=\"0\" cellspacing=\"0\">\n"
        );

        // 행별로 그룹화
        let max_row = table.cells.iter().map(|c| c.row).max().unwrap_or(0);
        for row in 0..=max_row {
            html.push_str("<tr>\n");
            let mut row_cells: Vec<&crate::model::table::Cell> = table.cells.iter()
                .filter(|c| c.row == row)
                .collect();
            row_cells.sort_by_key(|c| c.col);

            for cell in &row_cells {
                // 병합된 셀은 첫 번째 셀만 출력 (rowspan/colspan 은 merge 된 셀 정보)
                let mut td_style = String::new();

                // 셀 배경/테두리 (BorderFill)
                if cell.border_fill_id > 0 {
                    if let Some(bs) = self.styles.border_styles.get(cell.border_fill_id as usize) {
                        self.apply_border_fill_css(&mut td_style, bs);
                    }
                }

                // 셀 패딩
                td_style.push_str("padding:1px 5px;");

                // vertical-align
                td_style.push_str("vertical-align:top;");

                let mut td_attrs = format!("style=\"{}\"", td_style);
                if cell.col_span > 1 {
                    td_attrs.push_str(&format!(" colspan=\"{}\"", cell.col_span));
                }
                if cell.row_span > 1 {
                    td_attrs.push_str(&format!(" rowspan=\"{}\"", cell.row_span));
                }

                html.push_str(&format!("<td {}>\n", td_attrs));

                // 셀 내부 문단들
                for cpara in &cell.paragraphs {
                    html.push_str(&self.paragraph_to_html(cpara, None, None));
                }

                html.push_str("</td>\n");
            }
            html.push_str("</tr>\n");
        }

        html.push_str("</table>\n");
        html
    }

    /// BorderFill 스타일을 CSS로 변환하여 추가한다.
    pub(crate) fn apply_border_fill_css(
        &self,
        css: &mut String,
        bs: &crate::renderer::style_resolver::ResolvedBorderStyle,
    ) {
        // 배경색
        if let Some(fill_color) = bs.fill_color {
            if fill_color != 0xFFFFFF && fill_color != 0 {
                css.push_str(&format!("background-color:{};", clipboard_color_to_css(fill_color)));
            }
        }

        // 테두리 (좌, 우, 상, 하)
        let sides = ["left", "right", "top", "bottom"];
        for (i, side) in sides.iter().enumerate() {
            let bl = &bs.borders[i];
            if bl.width > 0 {
                let color = clipboard_color_to_css(bl.color);
                let px = (bl.width as f64).max(1.0);
                css.push_str(&format!(
                    "border-{}:{:.1}px solid {};",
                    side, px, color
                ));
            }
        }
    }

    /// Picture 컨트롤을 HTML <img>로 변환한다.
    pub(crate) fn picture_to_html(&self, pic: &crate::model::image::Picture) -> String {
        use base64::Engine;

        let bin_data_id = pic.image_attr.bin_data_id;
        if bin_data_id == 0 { return String::new(); }

        // 이미지 데이터 찾기 (bin_data_id는 1-indexed 순번)
        let image_data = if bin_data_id > 0 {
            self.document.bin_data_content.get((bin_data_id - 1) as usize)
        } else {
            None
        };

        if let Some(bdc) = image_data {
            let base64_data = base64::engine::general_purpose::STANDARD.encode(&bdc.data);
            let mime_type = detect_clipboard_image_mime(&bdc.data);

            // 크기 계산 (HWPUNIT → px)
            let w = crate::renderer::hwpunit_to_px(pic.common.width as i32, self.dpi);
            let h = crate::renderer::hwpunit_to_px(pic.common.height as i32, self.dpi);

            format!(
                "<img src=\"data:{};base64,{}\" width=\"{:.0}\" height=\"{:.0}\" />\n",
                mime_type, base64_data, w, h
            )
        } else {
            String::new()
        }
    }

    /// 컨트롤의 이미지 바이너리 데이터를 반환한다.
    /// Picture 컨트롤만 지원하며, 다른 타입은 에러를 반환한다.
    pub fn get_control_image_data_native(
        &self,
        section_idx: usize,
        para_idx: usize,
        control_idx: usize,
    ) -> Result<Vec<u8>, HwpError> {
        let section = self.document.sections.get(section_idx)
            .ok_or_else(|| HwpError::RenderError(format!("구역 {} 범위 초과", section_idx)))?;
        let para = section.paragraphs.get(para_idx)
            .ok_or_else(|| HwpError::RenderError(format!("문단 {} 범위 초과", para_idx)))?;
        let control = para.controls.get(control_idx)
            .ok_or_else(|| HwpError::RenderError(format!("컨트롤 {} 범위 초과", control_idx)))?;

        let pic = match control {
            Control::Picture(p) => p,
            _ => return Err(HwpError::RenderError("Picture 컨트롤이 아닙니다".to_string())),
        };

        let bin_data_id = pic.image_attr.bin_data_id;
        if bin_data_id == 0 {
            return Err(HwpError::RenderError("이미지 데이터 없음 (bin_data_id=0)".to_string()));
        }

        let bdc = self.document.bin_data_content.get((bin_data_id - 1) as usize)
            .ok_or_else(|| HwpError::RenderError(format!("바이너리 데이터 {} 범위 초과", bin_data_id)))?;

        Ok(bdc.data.clone())
    }

    /// 컨트롤의 이미지 MIME 타입을 반환한다.
    pub fn get_control_image_mime_native(
        &self,
        section_idx: usize,
        para_idx: usize,
        control_idx: usize,
    ) -> Result<String, HwpError> {
        let section = self.document.sections.get(section_idx)
            .ok_or_else(|| HwpError::RenderError(format!("구역 {} 범위 초과", section_idx)))?;
        let para = section.paragraphs.get(para_idx)
            .ok_or_else(|| HwpError::RenderError(format!("문단 {} 범위 초과", para_idx)))?;
        let control = para.controls.get(control_idx)
            .ok_or_else(|| HwpError::RenderError(format!("컨트롤 {} 범위 초과", control_idx)))?;

        let pic = match control {
            Control::Picture(p) => p,
            _ => return Err(HwpError::RenderError("Picture 컨트롤이 아닙니다".to_string())),
        };

        let bin_data_id = pic.image_attr.bin_data_id;
        if bin_data_id == 0 {
            return Err(HwpError::RenderError("이미지 데이터 없음 (bin_data_id=0)".to_string()));
        }

        let bdc = self.document.bin_data_content.get((bin_data_id - 1) as usize)
            .ok_or_else(|| HwpError::RenderError(format!("바이너리 데이터 {} 범위 초과", bin_data_id)))?;

        Ok(detect_clipboard_image_mime(&bdc.data).to_string())
    }

    /// BinData ID(1-based)로 이미지 바이너리 데이터를 반환한다.
    pub fn get_bin_data_image_data_native(&self, bin_data_id: u16) -> Result<Vec<u8>, HwpError> {
        if bin_data_id == 0 {
            return Err(HwpError::RenderError("이미지 데이터 없음 (bin_data_id=0)".to_string()));
        }
        let bdc = self
            .document
            .bin_data_content
            .get((bin_data_id - 1) as usize)
            .ok_or_else(|| {
                HwpError::RenderError(format!("바이너리 데이터 {} 범위 초과", bin_data_id))
            })?;
        Ok(bdc.data.clone())
    }

    /// BinData ID(1-based)로 이미지 MIME 타입을 반환한다.
    pub fn get_bin_data_image_mime_native(&self, bin_data_id: u16) -> Result<String, HwpError> {
        if bin_data_id == 0 {
            return Err(HwpError::RenderError("이미지 데이터 없음 (bin_data_id=0)".to_string()));
        }
        let bdc = self
            .document
            .bin_data_content
            .get((bin_data_id - 1) as usize)
            .ok_or_else(|| {
                HwpError::RenderError(format!("바이너리 데이터 {} 범위 초과", bin_data_id))
            })?;
        Ok(detect_clipboard_image_mime(&bdc.data).to_string())
    }

    // === 클립보드 HTML 붙여넣기 ===

}

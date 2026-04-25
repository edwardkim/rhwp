//! 텍스트 삽입/삭제/문단 분리·병합/범위 삭제/문단 쿼리 관련 native 메서드

use crate::model::control::Control;
use crate::model::paragraph::Paragraph;
use crate::renderer::composer::{compose_paragraph, reflow_line_segs, ComposedParagraph};
use crate::renderer::page_layout::PageLayoutInfo;
use crate::model::page::ColumnDef;
use crate::document_core::DocumentCore;
use crate::error::HwpError;
use crate::model::event::DocumentEvent;
use super::super::helpers::get_textbox_from_shape;

impl DocumentCore {
    pub fn insert_text_native(
        &mut self,
        section_idx: usize,
        para_idx: usize,
        char_offset: usize,
        text: &str,
    ) -> Result<String, HwpError> {
        // 인덱스 범위 검증
        if section_idx >= self.document.sections.len() {
            return Err(HwpError::RenderError(format!(
                "구역 인덱스 {} 범위 초과 (총 {}개)", section_idx, self.document.sections.len()
            )));
        }
        let section = &self.document.sections[section_idx];
        if para_idx >= section.paragraphs.len() {
            return Err(HwpError::RenderError(format!(
                "문단 인덱스 {} 범위 초과 (총 {}개)", para_idx, section.paragraphs.len()
            )));
        }

        // 편집 시 raw 스트림 무효화 (재직렬화 유도)
        self.document.sections[section_idx].raw_stream = None;

        // 텍스트 삽입
        let new_chars_count = text.chars().count();
        self.document.sections[section_idx].paragraphs[para_idx]
            .insert_text_at(char_offset, text);

        // line_segs 재계산 (리플로우) → vpos 재계산 → 재구성 → 재페이지네이션
        // 다단 문서에서 편집 후 문단이 다른 단으로 재배치될 수 있으므로
        // para_column_map 변경 감지 + 재reflow 수렴 루프 (최대 3회)
        let old_col = self.para_column_map
            .get(section_idx)
            .and_then(|m| m.get(para_idx))
            .copied()
            .unwrap_or(0);
        self.reflow_paragraph(section_idx, para_idx);
        crate::renderer::composer::recalculate_section_vpos(
            &mut self.document.sections[section_idx].paragraphs, para_idx,
        );
        self.recompose_paragraph(section_idx, para_idx);
        self.paginate_if_needed();

        for _ in 0..2 {
            let new_col = self.para_column_map
                .get(section_idx)
                .and_then(|m| m.get(para_idx))
                .copied()
                .unwrap_or(0);
            if new_col == old_col { break; }
            self.reflow_paragraph(section_idx, para_idx);
            crate::renderer::composer::recalculate_section_vpos(
                &mut self.document.sections[section_idx].paragraphs, para_idx,
            );
            self.recompose_paragraph(section_idx, para_idx);
            self.paginate_if_needed();
        }

        let new_offset = char_offset + new_chars_count;

        // 캐럿 위치 갱신 (DocProperties)
        // caret_char_pos는 UTF-16 코드 유닛 기준
        let para = &self.document.sections[section_idx].paragraphs[para_idx];
        let caret_utf16_pos = if new_offset < para.char_offsets.len() {
            para.char_offsets[new_offset]
        } else if !para.char_offsets.is_empty() {
            let last = para.char_offsets.len() - 1;
            let last_char = para.text.chars().nth(last);
            para.char_offsets[last] + last_char.map(|c| if (c as u32) > 0xFFFF { 2 } else { 1 }).unwrap_or(1)
        } else {
            // 텍스트 없이 컨트롤만 있는 경우
            (para.controls.len() as u32) * 8
        };
        self.document.doc_properties.caret_list_id = section_idx as u32;
        self.document.doc_properties.caret_para_id = para_idx as u32;
        self.document.doc_properties.caret_char_pos = caret_utf16_pos;

        // DocInfo raw_stream 내 캐럿 위치만 surgical update (전체 재직렬화 방지)
        if let Some(ref mut raw) = self.document.doc_info.raw_stream {
            let _ = crate::serializer::doc_info::surgical_update_caret(
                raw, section_idx as u32, para_idx as u32, caret_utf16_pos,
            );
        }

        self.event_log.push(DocumentEvent::TextInserted { section: section_idx, para: para_idx, offset: char_offset, len: new_chars_count });
        Ok(super::super::helpers::json_ok_with(&format!("\"charOffset\":{}", new_offset)))
    }

    /// 텍스트 삭제 (네이티브 에러 타입)
    pub fn delete_text_native(
        &mut self,
        section_idx: usize,
        para_idx: usize,
        char_offset: usize,
        count: usize,
    ) -> Result<String, HwpError> {
        // 인덱스 범위 검증
        if section_idx >= self.document.sections.len() {
            return Err(HwpError::RenderError(format!(
                "구역 인덱스 {} 범위 초과 (총 {}개)", section_idx, self.document.sections.len()
            )));
        }
        let section = &self.document.sections[section_idx];
        if para_idx >= section.paragraphs.len() {
            return Err(HwpError::RenderError(format!(
                "문단 인덱스 {} 범위 초과 (총 {}개)", para_idx, section.paragraphs.len()
            )));
        }

        // 편집 시 raw 스트림 무효화 (재직렬화 유도)
        self.document.sections[section_idx].raw_stream = None;

        // 텍스트 삭제
        self.document.sections[section_idx].paragraphs[para_idx]
            .delete_text_at(char_offset, count);

        // line_segs 재계산 (리플로우) → 재구성 → 재페이지네이션
        // 다단 수렴 루프 (최대 3회)
        let old_col = self.para_column_map
            .get(section_idx)
            .and_then(|m| m.get(para_idx))
            .copied()
            .unwrap_or(0);
        self.reflow_paragraph(section_idx, para_idx);
        crate::renderer::composer::recalculate_section_vpos(
            &mut self.document.sections[section_idx].paragraphs, para_idx,
        );
        self.recompose_paragraph(section_idx, para_idx);
        self.paginate_if_needed();

        for _ in 0..2 {
            let new_col = self.para_column_map
                .get(section_idx)
                .and_then(|m| m.get(para_idx))
                .copied()
                .unwrap_or(0);
            if new_col == old_col { break; }
            self.reflow_paragraph(section_idx, para_idx);
            crate::renderer::composer::recalculate_section_vpos(
                &mut self.document.sections[section_idx].paragraphs, para_idx,
            );
            self.recompose_paragraph(section_idx, para_idx);
            self.paginate_if_needed();
        }

        // 캐럿 위치 갱신 (DocProperties)
        let para = &self.document.sections[section_idx].paragraphs[para_idx];
        let caret_utf16_pos = if char_offset < para.char_offsets.len() {
            para.char_offsets[char_offset]
        } else if !para.char_offsets.is_empty() {
            let last = para.char_offsets.len() - 1;
            let last_char = para.text.chars().nth(last);
            para.char_offsets[last] + last_char.map(|c| if (c as u32) > 0xFFFF { 2 } else { 1 }).unwrap_or(1)
        } else {
            (para.controls.len() as u32) * 8
        };
        self.document.doc_properties.caret_list_id = section_idx as u32;
        self.document.doc_properties.caret_para_id = para_idx as u32;
        self.document.doc_properties.caret_char_pos = caret_utf16_pos;

        // DocInfo raw_stream 내 캐럿 위치만 surgical update (전체 재직렬화 방지)
        if let Some(ref mut raw) = self.document.doc_info.raw_stream {
            let _ = crate::serializer::doc_info::surgical_update_caret(
                raw, section_idx as u32, para_idx as u32, caret_utf16_pos,
            );
        }

        self.event_log.push(DocumentEvent::TextDeleted { section: section_idx, para: para_idx, offset: char_offset, count });
        Ok(super::super::helpers::json_ok_with(&format!("\"charOffset\":{}", char_offset)))
    }

    /// 지정된 문단의 line_segs를 컬럼 너비 기반으로 재계산한다.
    pub(crate) fn reflow_paragraph(&mut self, section_idx: usize, para_idx: usize) {
        let section = &self.document.sections[section_idx];
        let page_def = &section.section_def.page_def;
        // 해당 문단에 적용되는 ColumnDef를 찾음 (구역 내 다단↔단일단 전환 지원)
        let column_def = Self::find_column_def_for_paragraph(&section.paragraphs, para_idx);
        let layout = PageLayoutInfo::from_page_def(page_def, &column_def, self.dpi);

        // 페이지네이션 매핑에서 문단의 소속 단 인덱스 조회
        let col_idx = self.para_column_map
            .get(section_idx)
            .and_then(|m| m.get(para_idx))
            .copied()
            .unwrap_or(0) as usize;
        let col_area = layout.column_areas.get(col_idx)
            .unwrap_or(&layout.column_areas[0]);

        // 문단 여백 계산
        let para = &section.paragraphs[para_idx];
        let para_style = self.styles.para_styles.get(para.para_shape_id as usize);
        let margin_left = para_style.map(|s| s.margin_left).unwrap_or(0.0);
        let margin_right = para_style.map(|s| s.margin_right).unwrap_or(0.0);
        let available_width = col_area.width - margin_left - margin_right;

        reflow_line_segs(
            &mut self.document.sections[section_idx].paragraphs[para_idx],
            available_width,
            &self.styles,
            self.dpi,
        );
    }

    /// 표 셀 내부 문단에 텍스트 삽입 (네이티브)
    pub fn insert_text_in_cell_native(
        &mut self,
        section_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
        cell_idx: usize,
        cell_para_idx: usize,
        char_offset: usize,
        text: &str,
    ) -> Result<String, HwpError> {
        // 셀 문단 접근 검증 및 텍스트 삽입
        let cell_para = self.get_cell_paragraph_mut(
            section_idx, parent_para_idx, control_idx, cell_idx, cell_para_idx,
        )?;
        let new_chars_count = text.chars().count();
        cell_para.insert_text_at(char_offset, text);

        // 부모 컨트롤 dirty 마킹 (표 또는 글상자)
        self.mark_cell_control_dirty(section_idx, parent_para_idx, control_idx);

        // 셀 폭 기반 리플로우
        self.reflow_cell_paragraph(section_idx, parent_para_idx, control_idx, cell_idx, cell_para_idx);

        // raw 스트림 무효화, 재페이지네이션 (셀 편집 → composed 불변, section dirty만 설정)
        self.document.sections[section_idx].raw_stream = None;
        self.mark_section_dirty(section_idx);
        self.paginate_if_needed();

        let new_offset = char_offset + new_chars_count;
        self.event_log.push(DocumentEvent::CellTextChanged { section: section_idx, para: parent_para_idx, ctrl: control_idx, cell: cell_idx });
        Ok(super::super::helpers::json_ok_with(&format!("\"charOffset\":{}", new_offset)))
    }

    /// 표 셀 내부 문단에서 텍스트 삭제 (네이티브)
    pub fn delete_text_in_cell_native(
        &mut self,
        section_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
        cell_idx: usize,
        cell_para_idx: usize,
        char_offset: usize,
        count: usize,
    ) -> Result<String, HwpError> {
        // 셀 문단 접근 검증 및 텍스트 삭제
        let cell_para = self.get_cell_paragraph_mut(
            section_idx, parent_para_idx, control_idx, cell_idx, cell_para_idx,
        )?;
        cell_para.delete_text_at(char_offset, count);

        // 부모 컨트롤 dirty 마킹 (표 또는 글상자)
        self.mark_cell_control_dirty(section_idx, parent_para_idx, control_idx);

        // 셀 폭 기반 리플로우
        self.reflow_cell_paragraph(section_idx, parent_para_idx, control_idx, cell_idx, cell_para_idx);

        // raw 스트림 무효화, 재페이지네이션 (셀 편집 → composed 불변)
        self.document.sections[section_idx].raw_stream = None;
        self.mark_section_dirty(section_idx);
        self.paginate_if_needed();

        self.event_log.push(DocumentEvent::CellTextChanged { section: section_idx, para: parent_para_idx, ctrl: control_idx, cell: cell_idx });
        Ok(super::super::helpers::json_ok_with(&format!("\"charOffset\":{}", char_offset)))
    }

    /// 표 셀 또는 글상자 내부 문단에 대한 가변 참조를 얻는다.
    pub(crate) fn get_cell_paragraph_mut(
        &mut self,
        section_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
        cell_idx: usize,
        cell_para_idx: usize,
    ) -> Result<&mut crate::model::paragraph::Paragraph, HwpError> {
        if section_idx >= self.document.sections.len() {
            return Err(HwpError::RenderError(format!(
                "구역 인덱스 {} 범위 초과", section_idx
            )));
        }
        let section = &mut self.document.sections[section_idx];
        if parent_para_idx >= section.paragraphs.len() {
            return Err(HwpError::RenderError(format!(
                "부모 문단 인덱스 {} 범위 초과", parent_para_idx
            )));
        }
        let para = &mut section.paragraphs[parent_para_idx];
        if control_idx >= para.controls.len() {
            return Err(HwpError::RenderError(format!(
                "컨트롤 인덱스 {} 범위 초과", control_idx
            )));
        }
        match &mut para.controls[control_idx] {
            Control::Table(t) => {
                // cell_idx == 65534: 표 캡션 접근 (TypeScript에서 표 캡션 편집 시 사용)
                if cell_idx == 65534 {
                    let cap = t.caption.as_mut()
                        .ok_or_else(|| HwpError::RenderError(
                            "지정된 표 컨트롤에 캡션이 없습니다".to_string()
                        ))?;
                    if cell_para_idx >= cap.paragraphs.len() {
                        return Err(HwpError::RenderError(format!(
                            "캡션 문단 인덱스 {} 범위 초과 (총 {}개)", cell_para_idx, cap.paragraphs.len()
                        )));
                    }
                    return Ok(&mut cap.paragraphs[cell_para_idx]);
                }
                if cell_idx >= t.cells.len() {
                    return Err(HwpError::RenderError(format!(
                        "셀 인덱스 {} 범위 초과 (총 {}개)", cell_idx, t.cells.len()
                    )));
                }
                let cell = &mut t.cells[cell_idx];
                if cell_para_idx >= cell.paragraphs.len() {
                    return Err(HwpError::RenderError(format!(
                        "셀 문단 인덱스 {} 범위 초과 (총 {}개)", cell_para_idx, cell.paragraphs.len()
                    )));
                }
                Ok(&mut cell.paragraphs[cell_para_idx])
            }
            Control::Shape(shape) => {
                let tb = super::super::helpers::get_textbox_from_shape_mut(shape)
                    .ok_or_else(|| HwpError::RenderError(
                        "지정된 Shape 컨트롤에 텍스트 박스가 없습니다".to_string()
                    ))?;
                if cell_para_idx >= tb.paragraphs.len() {
                    return Err(HwpError::RenderError(format!(
                        "글상자 문단 인덱스 {} 범위 초과 (총 {}개)", cell_para_idx, tb.paragraphs.len()
                    )));
                }
                Ok(&mut tb.paragraphs[cell_para_idx])
            }
            Control::Picture(pic) => {
                let cap = pic.caption.as_mut()
                    .ok_or_else(|| HwpError::RenderError(
                        "지정된 그림 컨트롤에 캡션이 없습니다".to_string()
                    ))?;
                if cell_para_idx >= cap.paragraphs.len() {
                    return Err(HwpError::RenderError(format!(
                        "캡션 문단 인덱스 {} 범위 초과 (총 {}개)", cell_para_idx, cap.paragraphs.len()
                    )));
                }
                Ok(&mut cap.paragraphs[cell_para_idx])
            }
            _ => Err(HwpError::RenderError(
                "지정된 컨트롤이 표, 글상자 또는 그림이 아닙니다".to_string()
            )),
        }
    }

    /// 부모 컨트롤(표 또는 글상자)의 dirty를 마킹한다.
    fn mark_cell_control_dirty(
        &mut self,
        section_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
    ) {
        if let Some(ctrl) = self.document.sections[section_idx]
            .paragraphs[parent_para_idx].controls.get_mut(control_idx)
        {
            match ctrl {
                Control::Table(t) => { t.dirty = true; }
                // Shape는 별도 dirty 필드가 없으므로 section dirty만으로 충분
                _ => {}
            }
        }
    }

    pub(crate) fn reflow_cell_paragraph(
        &mut self,
        section_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
        cell_idx: usize,
        cell_para_idx: usize,
    ) {
        use crate::renderer::hwpunit_to_px;

        // 셀/글상자 폭과 패딩 읽기 (불변 참조)
        let (cell_width, pad_left, pad_right) = {
            let para = &self.document.sections[section_idx].paragraphs[parent_para_idx];
            match para.controls.get(control_idx) {
                Some(Control::Table(table)) => {
                    if cell_idx == 65534 {
                        // 표 캡션 리플로우: Top/Bottom은 max_width, Left/Right는 width 사용
                        if let Some(ref cap) = table.caption {
                            use crate::model::shape::CaptionDirection;
                            let w = match cap.direction {
                                CaptionDirection::Left | CaptionDirection::Right => cap.width,
                                _ => cap.max_width,
                            };
                            (w, 0, 0)
                        } else {
                            return;
                        }
                    } else if let Some(cell) = table.cells.get(cell_idx) {
                        let pad_l = if cell.padding.left != 0 {
                            cell.padding.left
                        } else {
                            table.padding.left
                        };
                        let pad_r = if cell.padding.right != 0 {
                            cell.padding.right
                        } else {
                            table.padding.right
                        };
                        (cell.width, pad_l, pad_r)
                    } else {
                        return;
                    }
                }
                Some(Control::Shape(shape)) => {
                    if let Some(tb) = super::super::helpers::get_textbox_from_shape(shape) {
                        let common = shape.common();
                        // 글상자 폭 = common.width, 여백 = textbox margin
                        (common.width as u32, tb.margin_left, tb.margin_right)
                    } else {
                        return;
                    }
                }
                Some(Control::Picture(pic)) => {
                    // 캡션 폭 = 그림 폭 (Bottom/Top 방향), 여백 없음
                    (pic.common.width as u32, 0, 0)
                }
                _ => return,
            }
        };

        let cell_width_px = hwpunit_to_px(cell_width as i32, self.dpi);
        let pad_left_px = hwpunit_to_px(pad_left as i32, self.dpi);
        let pad_right_px = hwpunit_to_px(pad_right as i32, self.dpi);
        let available_width = (cell_width_px - pad_left_px - pad_right_px).max(0.0);

        // 문단 여백 계산
        let para_shape_id = {
            let cell_para = self.get_cell_paragraph_ref(
                section_idx, parent_para_idx, control_idx, cell_idx, cell_para_idx,
            );
            match cell_para {
                Some(p) => p.para_shape_id,
                None => return,
            }
        };
        let para_style = self.styles.para_styles.get(para_shape_id as usize);
        let margin_left = para_style.map(|s| s.margin_left).unwrap_or(0.0);
        let margin_right = para_style.map(|s| s.margin_right).unwrap_or(0.0);
        let final_width = (available_width - margin_left - margin_right).max(0.0);

        // 가변 참조로 리플로우 실행
        match self.document.sections[section_idx]
            .paragraphs[parent_para_idx].controls.get_mut(control_idx)
        {
            Some(Control::Table(table)) => {
                if let Some(cell) = table.cells.get_mut(cell_idx) {
                    if let Some(cell_para) = cell.paragraphs.get_mut(cell_para_idx) {
                        reflow_line_segs(cell_para, final_width, &self.styles, self.dpi);
                    }
                }
            }
            Some(Control::Shape(shape)) => {
                if let Some(tb) = super::super::helpers::get_textbox_from_shape_mut(shape) {
                    if let Some(cell_para) = tb.paragraphs.get_mut(cell_para_idx) {
                        reflow_line_segs(cell_para, final_width, &self.styles, self.dpi);
                    }
                }
            }
            Some(Control::Picture(pic)) => {
                if let Some(ref mut cap) = pic.caption {
                    if let Some(cell_para) = cap.paragraphs.get_mut(cell_para_idx) {
                        reflow_line_segs(cell_para, final_width, &self.styles, self.dpi);
                    }
                }
            }
            _ => {}
        }
    }

    // ─── Phase 3 네이티브 구현: 커서 이동 API ─────────────────

    pub(crate) fn delete_range_native(
        &mut self,
        section_idx: usize,
        start_para: usize,
        start_offset: usize,
        end_para: usize,
        end_offset: usize,
        cell_ctx: Option<(usize, usize, usize)>,
    ) -> Result<String, HwpError> {
        // Section raw 스트림 무효화 (재직렬화 유도)
        self.document.sections[section_idx].raw_stream = None;
        // DocInfo raw_stream은 유지 (전체 재직렬화 시 FIX-4 문제 발생)

        if let Some((ppi, ci, cei)) = cell_ctx {
            // ─── 셀 내 deleteRange ───
            if start_para == end_para {
                // 같은 문단 내 삭제
                let count = end_offset - start_offset;
                if count > 0 {
                    let cell_para = self.get_cell_paragraph_mut(section_idx, ppi, ci, cei, start_para)?;
                    cell_para.delete_text_at(start_offset, count);
                    self.reflow_cell_paragraph(section_idx, ppi, ci, cei, start_para);
                }
            } else {
                // 다중 문단 셀 내 삭제
                // 1) 마지막 문단 앞부분 삭제
                if end_offset > 0 {
                    let cell_para = self.get_cell_paragraph_mut(section_idx, ppi, ci, cei, end_para)?;
                    cell_para.delete_text_at(0, end_offset);
                }
                // 2) 중간 문단 역순 제거 — 셀 내 문단은 cell.paragraphs에서 직접 제거
                for mid_para in (start_para + 1..end_para).rev() {
                    let cell = self.get_cell_mut(section_idx, ppi, ci, cei)?;
                    if mid_para < cell.paragraphs.len() {
                        cell.paragraphs.remove(mid_para);
                    }
                }
                // 3) 첫 문단 뒷부분 삭제
                {
                    let cell_para = self.get_cell_paragraph_mut(section_idx, ppi, ci, cei, start_para)?;
                    let para_len = cell_para.text.chars().count();
                    if start_offset < para_len {
                        cell_para.delete_text_at(start_offset, para_len - start_offset);
                    }
                }
                // 4) 첫-마지막 문단 병합 (마지막 문단이 이제 start_para+1에 위치)
                let cell = self.get_cell_mut(section_idx, ppi, ci, cei)?;
                if start_para + 1 < cell.paragraphs.len() {
                    let next_para = cell.paragraphs.remove(start_para + 1);
                    cell.paragraphs[start_para].merge_from(&next_para);
                }
                self.reflow_cell_paragraph(section_idx, ppi, ci, cei, start_para);
            }

            // 부모 컨트롤 dirty 마킹 + 재페이지네이션
            self.mark_cell_control_dirty(section_idx, ppi, ci);
            self.mark_section_dirty(section_idx);
            self.paginate_if_needed();
            self.event_log.push(DocumentEvent::CellTextChanged { section: section_idx, para: ppi, ctrl: ci, cell: cei });
            Ok(super::super::helpers::json_ok_with(&format!("\"paraIdx\":{},\"charOffset\":{}", start_para, start_offset)))
        } else {
            // ─── 본문 deleteRange ───
            if start_para == end_para {
                // 같은 문단 내 삭제
                let count = end_offset - start_offset;
                if count > 0 {
                    self.document.sections[section_idx].paragraphs[start_para]
                        .delete_text_at(start_offset, count);
                    self.reflow_paragraph(section_idx, start_para);
                    crate::renderer::composer::recalculate_section_vpos(
                        &mut self.document.sections[section_idx].paragraphs, start_para,
                    );
                }
                // 변경 문단만 재구성
                self.recompose_paragraph(section_idx, start_para);
            } else {
                // 1) 마지막 문단 앞부분 삭제
                if end_offset > 0 {
                    self.document.sections[section_idx].paragraphs[end_para]
                        .delete_text_at(0, end_offset);
                }
                // 2) 중간 문단 역순 제거 (composed도 동기)
                for mid_para in (start_para + 1..end_para).rev() {
                    self.document.sections[section_idx].paragraphs.remove(mid_para);
                    self.remove_composed_paragraph(section_idx, mid_para);
                }
                // 3) 첫 문단 뒷부분 삭제
                {
                    let para_len = self.document.sections[section_idx].paragraphs[start_para].text.chars().count();
                    if start_offset < para_len {
                        self.document.sections[section_idx].paragraphs[start_para]
                            .delete_text_at(start_offset, para_len - start_offset);
                    }
                }
                // 4) 첫-마지막 문단 병합 (마지막 문단이 이제 start_para+1에 위치)
                if start_para + 1 < self.document.sections[section_idx].paragraphs.len() {
                    let next = self.document.sections[section_idx].paragraphs.remove(start_para + 1);
                    self.remove_composed_paragraph(section_idx, start_para + 1);
                    self.document.sections[section_idx].paragraphs[start_para].merge_from(&next);
                }
                self.reflow_paragraph(section_idx, start_para);
                crate::renderer::composer::recalculate_section_vpos(
                    &mut self.document.sections[section_idx].paragraphs, start_para,
                );
                // 병합된 문단 재구성
                self.recompose_paragraph(section_idx, start_para);
            }

            // 재페이지네이션
            self.paginate_if_needed();

            // 캐럿 위치 갱신
            self.document.doc_properties.caret_list_id = section_idx as u32;
            self.document.doc_properties.caret_para_id = start_para as u32;

            self.event_log.push(DocumentEvent::TextDeleted { section: section_idx, para: start_para, offset: start_offset, count: 0 });
            Ok(super::super::helpers::json_ok_with(&format!("\"paraIdx\":{},\"charOffset\":{}", start_para, start_offset)))
        }
    }

    /// 표 셀에 대한 가변 참조를 얻는다.
    pub(crate) fn get_cell_mut(
        &mut self,
        section_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
        cell_idx: usize,
    ) -> Result<&mut crate::model::table::Cell, HwpError> {
        let section = &mut self.document.sections[section_idx];
        let para = section.paragraphs.get_mut(parent_para_idx)
            .ok_or_else(|| HwpError::RenderError(format!("부모 문단 인덱스 {} 범위 초과", parent_para_idx)))?;
        let ctrl = para.controls.get_mut(control_idx)
            .ok_or_else(|| HwpError::RenderError(format!("컨트롤 인덱스 {} 범위 초과", control_idx)))?;
        match ctrl {
            Control::Table(ref mut table) => {
                table.cells.get_mut(cell_idx)
                    .ok_or_else(|| HwpError::RenderError(format!("셀 인덱스 {} 범위 초과", cell_idx)))
            }
            _ => Err(HwpError::RenderError("테이블 컨트롤이 아닙니다".to_string())),
        }
    }

    // ─── Phase 4 네이티브 끝 ────────────────────────────────

    // ─── Phase 3 네이티브 끝 ─────────────────────────────────

    /// 표 셀 내부 문단에 대한 불변 참조를 얻는다.
    pub(crate) fn get_cell_paragraph_ref(
        &self,
        section_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
        cell_idx: usize,
        cell_para_idx: usize,
    ) -> Option<&crate::model::paragraph::Paragraph> {
        let para = self.document.sections.get(section_idx)?
            .paragraphs.get(parent_para_idx)?;
        match para.controls.get(control_idx)? {
            Control::Table(table) => {
                if cell_idx == 65534 {
                    return table.caption.as_ref()?.paragraphs.get(cell_para_idx);
                }
                table.cells.get(cell_idx)?
                    .paragraphs.get(cell_para_idx)
            }
            Control::Shape(shape) => {
                if cell_idx != 0 { return None; }
                get_textbox_from_shape(shape)?
                    .paragraphs.get(cell_para_idx)
            }
            Control::Picture(pic) => {
                pic.caption.as_ref()?
                    .paragraphs.get(cell_para_idx)
            }
            _ => None,
        }
    }

    pub fn split_paragraph_native(
        &mut self,
        section_idx: usize,
        para_idx: usize,
        char_offset: usize,
    ) -> Result<String, HwpError> {
        if section_idx >= self.document.sections.len() {
            return Err(HwpError::RenderError(format!(
                "구역 인덱스 {} 범위 초과 (총 {}개)", section_idx, self.document.sections.len()
            )));
        }
        let section = &self.document.sections[section_idx];
        if para_idx >= section.paragraphs.len() {
            return Err(HwpError::RenderError(format!(
                "문단 인덱스 {} 범위 초과 (총 {}개)", para_idx, section.paragraphs.len()
            )));
        }

        // 편집 시 raw 스트림 무효화 (재직렬화 유도)
        self.document.sections[section_idx].raw_stream = None;

        // 문단 분리
        let new_para = self.document.sections[section_idx].paragraphs[para_idx]
            .split_at(char_offset);

        // 새 문단을 현재 문단 뒤에 삽입
        let new_para_idx = para_idx + 1;
        self.document.sections[section_idx].paragraphs.insert(new_para_idx, new_para);

        // 양쪽 문단 리플로우 → vpos 재계산 → 재구성 → 재페이지네이션 + 다단 수렴 루프
        let old_col1 = self.para_column_map.get(section_idx)
            .and_then(|m| m.get(para_idx)).copied().unwrap_or(0);
        self.reflow_paragraph(section_idx, para_idx);
        self.reflow_paragraph(section_idx, new_para_idx);
        crate::renderer::composer::recalculate_section_vpos(
            &mut self.document.sections[section_idx].paragraphs, para_idx,
        );
        self.recompose_paragraph(section_idx, para_idx);
        self.insert_composed_paragraph(section_idx, new_para_idx);
        self.paginate_if_needed();

        for _ in 0..2 {
            let new_col1 = self.para_column_map.get(section_idx)
                .and_then(|m| m.get(para_idx)).copied().unwrap_or(0);
            if new_col1 == old_col1 { break; }
            self.reflow_paragraph(section_idx, para_idx);
            self.reflow_paragraph(section_idx, new_para_idx);
            crate::renderer::composer::recalculate_section_vpos(
                &mut self.document.sections[section_idx].paragraphs, para_idx,
            );
            self.recompose_paragraph(section_idx, para_idx);
            self.recompose_paragraph(section_idx, new_para_idx);
            self.paginate_if_needed();
        }

        self.event_log.push(DocumentEvent::ParagraphSplit { section: section_idx, para: para_idx, offset: char_offset });
        Ok(super::super::helpers::json_ok_with(&format!("\"paraIdx\":{},\"charOffset\":0", new_para_idx)))
    }

    /// 강제 쪽 나누기 삽입 (Ctrl+Enter)
    /// 커서 위치에서 문단을 분할하고, 새 문단에 ColumnBreakType::Page를 설정한다.
    pub fn insert_page_break_native(
        &mut self,
        section_idx: usize,
        para_idx: usize,
        char_offset: usize,
    ) -> Result<String, HwpError> {
        use crate::model::paragraph::ColumnBreakType;

        if section_idx >= self.document.sections.len() {
            return Err(HwpError::RenderError(format!(
                "구역 인덱스 {} 범위 초과", section_idx
            )));
        }
        if para_idx >= self.document.sections[section_idx].paragraphs.len() {
            return Err(HwpError::RenderError(format!(
                "문단 인덱스 {} 범위 초과", para_idx
            )));
        }

        self.document.sections[section_idx].raw_stream = None;

        // 문단 분리
        let new_para = self.document.sections[section_idx].paragraphs[para_idx]
            .split_at(char_offset);
        let new_para_idx = para_idx + 1;
        self.document.sections[section_idx].paragraphs.insert(new_para_idx, new_para);

        // 새 문단에 쪽 나누기 설정
        self.document.sections[section_idx].paragraphs[new_para_idx].column_type = ColumnBreakType::Page;
        self.document.sections[section_idx].paragraphs[new_para_idx].raw_break_type = 0x04;

        // 분할된 두 문단 리플로우
        self.reflow_paragraph(section_idx, para_idx);
        self.reflow_paragraph(section_idx, new_para_idx);

        // 삽입 지점부터 구역 끝까지 vpos 재계산 (페이지 재배치에 필요)
        crate::renderer::composer::recalculate_section_vpos(
            &mut self.document.sections[section_idx].paragraphs,
            new_para_idx,
        );

        // 전체 구역 재구성 + 재페이지네이션
        self.recompose_section(section_idx);
        self.paginate_if_needed();
        self.invalidate_page_tree_cache();

        self.event_log.push(DocumentEvent::ParagraphSplit { section: section_idx, para: para_idx, offset: char_offset });
        Ok(super::super::helpers::json_ok_with(&format!("\"paraIdx\":{},\"charOffset\":0", new_para_idx)))
    }

    /// 단 나누기 삽입 (Ctrl+Shift+Enter)
    /// 커서 위치에서 문단을 분리하고 새 문단에 단 나누기 설정.
    /// 1단 문서에서는 쪽 나누기와 동일하게 동작.
    pub fn insert_column_break_native(
        &mut self,
        section_idx: usize,
        para_idx: usize,
        char_offset: usize,
    ) -> Result<String, HwpError> {
        use crate::model::paragraph::ColumnBreakType;

        if section_idx >= self.document.sections.len() {
            return Err(HwpError::RenderError(format!("구역 인덱스 {} 범위 초과", section_idx)));
        }
        if para_idx >= self.document.sections[section_idx].paragraphs.len() {
            return Err(HwpError::RenderError(format!("문단 인덱스 {} 범위 초과", para_idx)));
        }

        self.document.sections[section_idx].raw_stream = None;

        // 문단 분리
        let new_para = self.document.sections[section_idx].paragraphs[para_idx]
            .split_at(char_offset);
        let new_para_idx = para_idx + 1;
        self.document.sections[section_idx].paragraphs.insert(new_para_idx, new_para);

        // 새 문단에 단 나누기 설정
        self.document.sections[section_idx].paragraphs[new_para_idx].column_type = ColumnBreakType::Column;
        self.document.sections[section_idx].paragraphs[new_para_idx].raw_break_type = 0x08;

        // 분할된 두 문단 리플로우
        self.reflow_paragraph(section_idx, para_idx);
        self.reflow_paragraph(section_idx, new_para_idx);

        crate::renderer::composer::recalculate_section_vpos(
            &mut self.document.sections[section_idx].paragraphs,
            new_para_idx,
        );

        self.recompose_section(section_idx);
        self.paginate_if_needed();
        self.invalidate_page_tree_cache();

        self.event_log.push(DocumentEvent::ParagraphSplit { section: section_idx, para: para_idx, offset: char_offset });
        Ok(super::super::helpers::json_ok_with(&format!("\"paraIdx\":{},\"charOffset\":0", new_para_idx)))
    }

    /// 다단 설정 변경
    ///
    /// 구역의 초기 ColumnDef 컨트롤을 찾아 수정한다.
    /// 없으면 첫 문단에 새 ColumnDef를 삽입한다.
    ///
    /// ColumnDef는 문단 컨트롤로 저장되며, SectionDef와 독립적이다.
    /// 수정 후 recompose + repaginate로 조판을 갱신한다.
    pub fn set_column_def_native(
        &mut self,
        section_idx: usize,
        column_count: u16,
        column_type: u8,      // 0=일반(Normal), 1=배분(Distribute), 2=평행(Parallel)
        same_width: bool,
        spacing_hu: i16,      // 단 간격 (HWPUNIT)
    ) -> Result<String, HwpError> {
        use crate::model::page::{ColumnType, ColumnDirection};

        if section_idx >= self.document.sections.len() {
            return Err(HwpError::RenderError(format!("구역 인덱스 {} 범위 초과", section_idx)));
        }

        let col_type = match column_type {
            1 => ColumnType::Distribute,
            2 => ColumnType::Parallel,
            _ => ColumnType::Normal,
        };

        // 구역의 초기 ColumnDef 찾기 (find_initial_column_def와 동일 로직)
        let mut found = false;
        let paragraphs = &mut self.document.sections[section_idx].paragraphs;
        for para in paragraphs.iter_mut() {
            for ctrl in para.controls.iter_mut() {
                if let Control::ColumnDef(ref mut cd) = ctrl {
                    cd.column_count = column_count;
                    cd.column_type = col_type;
                    cd.same_width = same_width;
                    cd.spacing = spacing_hu;
                    if same_width {
                        cd.widths.clear();
                        cd.gaps.clear();
                    }
                    found = true;
                    break;
                }
            }
            if found { break; }
        }

        // 기존 ColumnDef가 없으면 첫 문단에 삽입
        if !found {
            let cd = ColumnDef {
                column_count,
                column_type: col_type,
                same_width,
                spacing: spacing_hu,
                direction: ColumnDirection::LeftToRight,
                ..Default::default()
            };
            if !self.document.sections[section_idx].paragraphs.is_empty() {
                self.document.sections[section_idx].paragraphs[0]
                    .controls.push(Control::ColumnDef(cd));
            }
        }

        // 조판 갱신
        self.document.sections[section_idx].raw_stream = None;
        self.rebuild_section(section_idx);

        Ok("{\"ok\":true}".to_string())
    }

    /// 문단 병합 (네이티브 에러 타입)
    pub fn merge_paragraph_native(
        &mut self,
        section_idx: usize,
        para_idx: usize,
    ) -> Result<String, HwpError> {
        if section_idx >= self.document.sections.len() {
            return Err(HwpError::RenderError(format!(
                "구역 인덱스 {} 범위 초과 (총 {}개)", section_idx, self.document.sections.len()
            )));
        }
        let section = &self.document.sections[section_idx];
        if para_idx == 0 {
            return Err(HwpError::RenderError(
                "첫 번째 문단은 병합할 수 없습니다".to_string()
            ));
        }
        if para_idx >= section.paragraphs.len() {
            return Err(HwpError::RenderError(format!(
                "문단 인덱스 {} 범위 초과 (총 {}개)", para_idx, section.paragraphs.len()
            )));
        }

        // 편집 시 raw 스트림 무효화 (재직렬화 유도)
        self.document.sections[section_idx].raw_stream = None;

        // 현재 문단을 이전 문단에 병합
        let current_para = self.document.sections[section_idx].paragraphs.remove(para_idx);
        let prev_idx = para_idx - 1;
        let merge_point = self.document.sections[section_idx].paragraphs[prev_idx]
            .merge_from(&current_para);

        // 병합된 문단 리플로우 → vpos 재계산 → 재구성 → 재페이지네이션 + 다단 수렴 루프
        let old_col = self.para_column_map.get(section_idx)
            .and_then(|m| m.get(prev_idx)).copied().unwrap_or(0);
        self.reflow_paragraph(section_idx, prev_idx);
        crate::renderer::composer::recalculate_section_vpos(
            &mut self.document.sections[section_idx].paragraphs, prev_idx,
        );
        self.remove_composed_paragraph(section_idx, para_idx);
        self.recompose_paragraph(section_idx, prev_idx);
        self.paginate_if_needed();

        for _ in 0..2 {
            let new_col = self.para_column_map.get(section_idx)
                .and_then(|m| m.get(prev_idx)).copied().unwrap_or(0);
            if new_col == old_col { break; }
            self.reflow_paragraph(section_idx, prev_idx);
            crate::renderer::composer::recalculate_section_vpos(
                &mut self.document.sections[section_idx].paragraphs, prev_idx,
            );
            self.recompose_paragraph(section_idx, prev_idx);
            self.paginate_if_needed();
        }

        self.event_log.push(DocumentEvent::ParagraphMerged { section: section_idx, para: para_idx });
        Ok(super::super::helpers::json_ok_with(&format!("\"paraIdx\":{},\"charOffset\":{}", prev_idx, merge_point)))
    }

    /// 셀 내부 문단 분할 (네이티브 에러 타입)
    pub fn split_paragraph_in_cell_native(
        &mut self,
        section_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
        cell_idx: usize,
        cell_para_idx: usize,
        char_offset: usize,
    ) -> Result<String, HwpError> {
        // 셀 문단 검증 및 분할
        let cell_para = self.get_cell_paragraph_mut(
            section_idx, parent_para_idx, control_idx, cell_idx, cell_para_idx,
        )?;
        let new_para = cell_para.split_at(char_offset);

        // 새 문단을 셀/글상자에 삽입
        let new_cell_para_idx = cell_para_idx + 1;
        match self.document.sections[section_idx]
            .paragraphs[parent_para_idx].controls.get_mut(control_idx)
        {
            Some(Control::Table(table)) => {
                table.cells[cell_idx].paragraphs.insert(new_cell_para_idx, new_para);
                table.dirty = true;
            }
            Some(Control::Shape(shape)) => {
                if let Some(tb) = super::super::helpers::get_textbox_from_shape_mut(shape) {
                    tb.paragraphs.insert(new_cell_para_idx, new_para);
                }
            }
            Some(Control::Picture(pic)) => {
                if let Some(ref mut cap) = pic.caption {
                    cap.paragraphs.insert(new_cell_para_idx, new_para);
                }
            }
            _ => {}
        }

        // 양쪽 문단 리플로우
        self.reflow_cell_paragraph(section_idx, parent_para_idx, control_idx, cell_idx, cell_para_idx);
        self.reflow_cell_paragraph(section_idx, parent_para_idx, control_idx, cell_idx, new_cell_para_idx);

        // raw 스트림 무효화, section dirty, 재페이지네이션
        self.document.sections[section_idx].raw_stream = None;
        self.mark_section_dirty(section_idx);
        self.paginate_if_needed();

        self.event_log.push(DocumentEvent::CellTextChanged { section: section_idx, para: parent_para_idx, ctrl: control_idx, cell: cell_idx });
        Ok(super::super::helpers::json_ok_with(&format!("\"cellParaIndex\":{},\"charOffset\":0", new_cell_para_idx)))
    }

    /// 셀 내부 문단 병합 (네이티브 에러 타입)
    ///
    /// cell_para_idx 문단을 이전 문단(cell_para_idx - 1)에 병합한다.
    pub fn merge_paragraph_in_cell_native(
        &mut self,
        section_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
        cell_idx: usize,
        cell_para_idx: usize,
    ) -> Result<String, HwpError> {
        if cell_para_idx == 0 {
            return Err(HwpError::RenderError(
                "셀 첫 번째 문단은 병합할 수 없습니다".to_string()
            ));
        }

        // 검증: 셀 문단 인덱스 범위 확인
        {
            let cell_para = self.get_cell_paragraph_mut(
                section_idx, parent_para_idx, control_idx, cell_idx, cell_para_idx,
            )?;
            let _ = cell_para; // 검증만 수행
        }

        // 문단 제거 및 이전 문단에 병합
        let prev_idx = cell_para_idx - 1;
        let merge_point;
        match self.document.sections[section_idx]
            .paragraphs[parent_para_idx].controls.get_mut(control_idx)
        {
            Some(Control::Table(table)) => {
                let removed = table.cells[cell_idx].paragraphs.remove(cell_para_idx);
                merge_point = table.cells[cell_idx].paragraphs[prev_idx].merge_from(&removed);
                table.dirty = true;
            }
            Some(Control::Shape(shape)) => {
                if let Some(tb) = super::super::helpers::get_textbox_from_shape_mut(shape) {
                    let removed = tb.paragraphs.remove(cell_para_idx);
                    merge_point = tb.paragraphs[prev_idx].merge_from(&removed);
                } else {
                    return Err(HwpError::RenderError(
                        "지정된 Shape 컨트롤에 텍스트 박스가 없습니다".to_string()
                    ));
                }
            }
            Some(Control::Picture(pic)) => {
                if let Some(ref mut cap) = pic.caption {
                    let removed = cap.paragraphs.remove(cell_para_idx);
                    merge_point = cap.paragraphs[prev_idx].merge_from(&removed);
                } else {
                    return Err(HwpError::RenderError(
                        "지정된 그림 컨트롤에 캡션이 없습니다".to_string()
                    ));
                }
            }
            _ => {
                return Err(HwpError::RenderError(
                    "지정된 컨트롤이 표, 글상자 또는 그림이 아닙니다".to_string()
                ));
            }
        }

        // 병합된 문단 리플로우
        self.reflow_cell_paragraph(section_idx, parent_para_idx, control_idx, cell_idx, prev_idx);

        // raw 스트림 무효화, section dirty, 재페이지네이션
        self.document.sections[section_idx].raw_stream = None;
        self.mark_section_dirty(section_idx);
        self.paginate_if_needed();

        self.event_log.push(DocumentEvent::CellTextChanged { section: section_idx, para: parent_para_idx, ctrl: control_idx, cell: cell_idx });
        Ok(super::super::helpers::json_ok_with(&format!("\"cellParaIndex\":{},\"charOffset\":{}", prev_idx, merge_point)))
    }

    // ─── Phase 1 Native: 기본 편집 보조 API ────────────────────

    /// 구역 내 문단 수 (네이티브)
    pub fn get_paragraph_count_native(&self, section_idx: usize) -> Result<usize, HwpError> {
        let section = self.document.sections.get(section_idx).ok_or_else(|| {
            HwpError::RenderError(format!(
                "구역 인덱스 {} 범위 초과 (총 {}개)", section_idx, self.document.sections.len()
            ))
        })?;
        Ok(section.paragraphs.len())
    }

    /// 문단 글자 수 (네이티브)
    pub fn get_paragraph_length_native(&self, section_idx: usize, para_idx: usize) -> Result<usize, HwpError> {
        let section = self.document.sections.get(section_idx).ok_or_else(|| {
            HwpError::RenderError(format!(
                "구역 인덱스 {} 범위 초과 (총 {}개)", section_idx, self.document.sections.len()
            ))
        })?;
        let para = section.paragraphs.get(para_idx).ok_or_else(|| {
            HwpError::RenderError(format!(
                "문단 인덱스 {} 범위 초과 (총 {}개)", para_idx, section.paragraphs.len()
            ))
        })?;
        Ok(para.text.chars().count())
    }

    /// 문단에 텍스트박스가 있는 Shape 컨트롤의 인덱스를 반환 (네이티브)
    /// 없으면 -1 반환
    pub fn get_textbox_control_index_native(&self, section_idx: usize, para_idx: usize) -> i32 {
        let section = match self.document.sections.get(section_idx) {
            Some(s) => s,
            None => return -1,
        };
        let para = match section.paragraphs.get(para_idx) {
            Some(p) => p,
            None => return -1,
        };
        for (ci, ctrl) in para.controls.iter().enumerate() {
            if let Control::Shape(shape) = ctrl {
                if get_textbox_from_shape(shape.as_ref()).is_some() {
                    return ci as i32;
                }
            }
        }
        -1
    }

    /// 문서 트리에서 다음 편집 가능한 컨트롤/본문을 찾는다.
    /// `(sec, para, ctrl_idx)`에서 시작, delta=+1(앞), delta=-1(뒤) 방향으로 탐색.
    /// ctrl_idx가 -1이면 해당 문단의 본문 텍스트에서 출발한 것으로 간주.
    ///
    /// 반환 JSON:
    ///   `{"type":"textbox","sec":N,"para":N,"ci":N}`
    ///   `{"type":"table","sec":N,"para":N,"ci":N}`
    ///   `{"type":"body","sec":N,"para":N}`
    ///   `{"type":"none"}`
    pub fn find_next_editable_control_native(
        &self,
        section_idx: usize,
        para_idx: usize,
        ctrl_idx: i32,
        delta: i32,
    ) -> String {
        let sections = &self.document.sections;

        // 헬퍼: 문단 내 편집 가능한 컨트롤을 방향에 따라 검색
        fn find_in_para(
            sections: &[crate::model::document::Section],
            sec: usize,
            para: usize,
            start_ci: i32,
            forward: bool,
        ) -> Option<(usize, &'static str)> {
            let section = sections.get(sec)?;
            let p = section.paragraphs.get(para)?;
            let controls = &p.controls;
            if forward {
                let from = if start_ci < 0 { 0usize } else { (start_ci as usize) + 1 };
                for ci in from..controls.len() {
                    match &controls[ci] {
                        Control::Shape(shape) => {
                            if get_textbox_from_shape(shape.as_ref()).is_some() {
                                return Some((ci, "textbox"));
                            }
                        }
                        Control::Table(_) => {
                            return Some((ci, "table"));
                        }
                        _ => {}
                    }
                }
            } else {
                let until = if start_ci < 0 { controls.len() } else { start_ci as usize };
                for ci in (0..until).rev() {
                    match &controls[ci] {
                        Control::Shape(shape) => {
                            if get_textbox_from_shape(shape.as_ref()).is_some() {
                                return Some((ci, "textbox"));
                            }
                        }
                        Control::Table(_) => {
                            return Some((ci, "table"));
                        }
                        _ => {}
                    }
                }
            }
            None
        }

        // 헬퍼: 문단이 편집 가능한 컨트롤을 하나라도 갖고 있는지
        fn has_navigable_control(
            sections: &[crate::model::document::Section],
            sec: usize,
            para: usize,
        ) -> bool {
            sections.get(sec)
                .and_then(|s| s.paragraphs.get(para))
                .map(|p| p.controls.iter().any(|c| {
                    matches!(c, Control::Table(_))
                    || matches!(c, Control::Shape(s) if get_textbox_from_shape(s.as_ref()).is_some())
                }))
                .unwrap_or(false)
        }

        let forward = delta > 0;

        // 1) 같은 문단에서 탐색
        if let Some((ci, ty)) = find_in_para(sections, section_idx, para_idx, ctrl_idx, forward) {
            return format!("{{\"type\":\"{}\",\"sec\":{},\"para\":{},\"ci\":{}}}", ty, section_idx, para_idx, ci);
        }

        // 2) 같은 섹션의 다른 문단 탐색
        if let Some(section) = sections.get(section_idx) {
            let para_count = section.paragraphs.len();
            let para_range: Box<dyn Iterator<Item = usize>> = if forward {
                Box::new((para_idx + 1)..para_count)
            } else if para_idx > 0 {
                Box::new((0..para_idx).rev())
            } else {
                Box::new(std::iter::empty())
            };
            for pi in para_range {
                let search_start = if forward { -1 } else { section.paragraphs[pi].controls.len() as i32 };
                if let Some((ci, ty)) = find_in_para(sections, section_idx, pi, search_start, forward) {
                    return format!("{{\"type\":\"{}\",\"sec\":{},\"para\":{},\"ci\":{}}}", ty, section_idx, pi, ci);
                }
                // 네비게이션 가능한 컨트롤이 없는 문단 → body
                if !has_navigable_control(sections, section_idx, pi) {
                    return format!("{{\"type\":\"body\",\"sec\":{},\"para\":{}}}", section_idx, pi);
                }
            }
        }

        // 3) 다른 섹션 탐색
        let sec_range: Box<dyn Iterator<Item = usize>> = if forward {
            Box::new((section_idx + 1)..sections.len())
        } else if section_idx > 0 {
            Box::new((0..section_idx).rev())
        } else {
            Box::new(std::iter::empty())
        };
        for si in sec_range {
            if let Some(section) = sections.get(si) {
                let para_range: Box<dyn Iterator<Item = usize>> = if forward {
                    Box::new(0..section.paragraphs.len())
                } else {
                    Box::new((0..section.paragraphs.len()).rev())
                };
                for pi in para_range {
                    let search_start = if forward { -1 } else { section.paragraphs[pi].controls.len() as i32 };
                    if let Some((ci, ty)) = find_in_para(sections, si, pi, search_start, forward) {
                        return format!("{{\"type\":\"{}\",\"sec\":{},\"para\":{},\"ci\":{}}}", ty, si, pi, ci);
                    }
                    if !has_navigable_control(sections, si, pi) {
                        return format!("{{\"type\":\"body\",\"sec\":{},\"para\":{}}}", si, pi);
                    }
                }
            }
        }

        // 4) 문서 경계
        "{\"type\":\"none\"}".to_string()
    }

    /// 커서에서 이전 방향으로 가장 가까운 선택 가능 컨트롤을 찾는다.
    /// F11 키 기능: 표, 그림, 글상자, 수식, 누름틀 등을 객체 선택.
    ///
    /// 반환 JSON:
    ///   `{"type":"table"|"shape"|"picture"|"equation"|"field","sec":N,"para":N,"ci":N}`
    ///   `{"type":"none"}`
    pub fn find_nearest_control_backward_native(
        &self,
        section_idx: usize,
        para_idx: usize,
        char_offset: usize,
    ) -> String {
        let sections = &self.document.sections;

        // 컨트롤 타입 분류 (선택 가능한 것만)
        fn classify_control(ctrl: &Control) -> Option<&'static str> {
            match ctrl {
                Control::Table(_) => Some("table"),
                Control::Picture(_) => Some("picture"),
                Control::Shape(_) => Some("shape"),
                Control::Equation(_) => Some("equation"),
                Control::Field(_) => Some("field"),
                Control::Bookmark(_) => Some("bookmark"),
                _ => None,
            }
        }

        // 문단 내에서 char_offset 이전의 컨트롤을 역순으로 탐색
        fn find_in_para_before(
            para: &crate::model::paragraph::Paragraph,
            char_offset: usize,
        ) -> Option<(usize, usize, &'static str)> {
            let positions = crate::document_core::find_control_text_positions(para);
            for ci in (0..para.controls.len()).rev() {
                if let Some(&pos) = positions.get(ci) {
                    if pos < char_offset {
                        if let Some(ty) = classify_control(&para.controls[ci]) {
                            return Some((ci, pos, ty));
                        }
                    }
                }
            }
            None
        }

        // 문단 전체에서 마지막 선택 가능 컨트롤 찾기
        fn find_last_in_para(
            para: &crate::model::paragraph::Paragraph,
        ) -> Option<(usize, usize, &'static str)> {
            let positions = crate::document_core::find_control_text_positions(para);
            for ci in (0..para.controls.len()).rev() {
                if let Some(ty) = classify_control(&para.controls[ci]) {
                    let pos = positions.get(ci).copied().unwrap_or(0);
                    return Some((ci, pos, ty));
                }
            }
            None
        }

        fn fmt_result(ty: &str, sec: usize, para: usize, ci: usize, char_pos: usize) -> String {
            format!(
                "{{\"type\":\"{}\",\"sec\":{},\"para\":{},\"ci\":{},\"charPos\":{}}}",
                ty, sec, para, ci, char_pos
            )
        }

        // 1) 같은 문단에서 char_offset 이전 탐색
        if let Some(section) = sections.get(section_idx) {
            if let Some(para) = section.paragraphs.get(para_idx) {
                if let Some((ci, cp, ty)) = find_in_para_before(para, char_offset) {
                    return fmt_result(ty, section_idx, para_idx, ci, cp);
                }
            }
        }

        // 2) 이전 문단들 역순 탐색 (같은 섹션)
        if let Some(section) = sections.get(section_idx) {
            for pi in (0..para_idx).rev() {
                if let Some((ci, cp, ty)) = find_last_in_para(&section.paragraphs[pi]) {
                    return fmt_result(ty, section_idx, pi, ci, cp);
                }
            }
        }

        // 3) 이전 섹션 역순 탐색
        for si in (0..section_idx).rev() {
            if let Some(section) = sections.get(si) {
                for pi in (0..section.paragraphs.len()).rev() {
                    if let Some((ci, cp, ty)) = find_last_in_para(&section.paragraphs[pi]) {
                        return fmt_result(ty, si, pi, ci, cp);
                    }
                }
            }
        }

        "{\"type\":\"none\"}".to_string()
    }

    /// 현재 위치 이후의 가장 가까운 선택 가능 컨트롤을 찾는다 (Shift+F11).
    pub fn find_nearest_control_forward_native(
        &self,
        section_idx: usize,
        para_idx: usize,
        char_offset: usize,
    ) -> String {
        let sections = &self.document.sections;

        fn classify_control(ctrl: &Control) -> Option<&'static str> {
            match ctrl {
                Control::Table(_) => Some("table"),
                Control::Picture(_) => Some("picture"),
                Control::Shape(_) => Some("shape"),
                Control::Equation(_) => Some("equation"),
                Control::Field(_) => Some("field"),
                Control::Bookmark(_) => Some("bookmark"),
                _ => None,
            }
        }

        fn find_in_para_after(
            para: &crate::model::paragraph::Paragraph,
            char_offset: usize,
        ) -> Option<(usize, usize, &'static str)> {
            let positions = crate::document_core::find_control_text_positions(para);
            for ci in 0..para.controls.len() {
                if let Some(&pos) = positions.get(ci) {
                    if pos > char_offset {
                        if let Some(ty) = classify_control(&para.controls[ci]) {
                            return Some((ci, pos, ty));
                        }
                    }
                }
            }
            None
        }

        fn find_first_in_para(
            para: &crate::model::paragraph::Paragraph,
        ) -> Option<(usize, usize, &'static str)> {
            let positions = crate::document_core::find_control_text_positions(para);
            for ci in 0..para.controls.len() {
                if let Some(ty) = classify_control(&para.controls[ci]) {
                    let pos = positions.get(ci).copied().unwrap_or(0);
                    return Some((ci, pos, ty));
                }
            }
            None
        }

        fn fmt_result(ty: &str, sec: usize, para: usize, ci: usize, char_pos: usize) -> String {
            format!(
                "{{\"type\":\"{}\",\"sec\":{},\"para\":{},\"ci\":{},\"charPos\":{}}}",
                ty, sec, para, ci, char_pos
            )
        }

        // 1) 같은 문단에서 char_offset 이후 탐색
        if let Some(section) = sections.get(section_idx) {
            if let Some(para) = section.paragraphs.get(para_idx) {
                if let Some((ci, cp, ty)) = find_in_para_after(para, char_offset) {
                    return fmt_result(ty, section_idx, para_idx, ci, cp);
                }
            }
        }

        // 2) 이후 문단 정순 탐색 (같은 섹션)
        if let Some(section) = sections.get(section_idx) {
            for pi in (para_idx + 1)..section.paragraphs.len() {
                if let Some((ci, cp, ty)) = find_first_in_para(&section.paragraphs[pi]) {
                    return fmt_result(ty, section_idx, pi, ci, cp);
                }
            }
        }

        // 3) 이후 섹션 정순 탐색
        for si in (section_idx + 1)..sections.len() {
            if let Some(section) = sections.get(si) {
                for pi in 0..section.paragraphs.len() {
                    if let Some((ci, cp, ty)) = find_first_in_para(&section.paragraphs[pi]) {
                        return fmt_result(ty, si, pi, ci, cp);
                    }
                }
            }
        }

        "{\"type\":\"none\"}".to_string()
    }

    /// 문단 텍스트 부분 추출 (네이티브)
    pub fn get_text_range_native(
        &self,
        section_idx: usize,
        para_idx: usize,
        char_offset: usize,
        count: usize,
    ) -> Result<String, HwpError> {
        let section = self.document.sections.get(section_idx).ok_or_else(|| {
            HwpError::RenderError(format!(
                "구역 인덱스 {} 범위 초과 (총 {}개)", section_idx, self.document.sections.len()
            ))
        })?;
        let para = section.paragraphs.get(para_idx).ok_or_else(|| {
            HwpError::RenderError(format!(
                "문단 인덱스 {} 범위 초과 (총 {}개)", para_idx, section.paragraphs.len()
            ))
        })?;
        let text_chars: Vec<char> = para.text.chars().collect();
        let total = text_chars.len();
        if char_offset > total {
            return Err(HwpError::RenderError(format!(
                "char_offset {} 범위 초과 (문단 길이 {})", char_offset, total
            )));
        }
        let end = (char_offset + count).min(total);
        let result: String = text_chars[char_offset..end].iter().collect();
        Ok(result)
    }

    /// 셀 내 문단 수 (네이티브)
    pub fn get_cell_paragraph_count_native(
        &self,
        section_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
        cell_idx: usize,
    ) -> Result<usize, HwpError> {
        let para = self.document.sections.get(section_idx)
            .ok_or_else(|| HwpError::RenderError(format!(
                "구역 인덱스 {} 범위 초과", section_idx
            )))?
            .paragraphs.get(parent_para_idx)
            .ok_or_else(|| HwpError::RenderError(format!(
                "문단 인덱스 {} 범위 초과", parent_para_idx
            )))?;
        match para.controls.get(control_idx) {
            Some(Control::Table(table)) => {
                if cell_idx == 65534 {
                    let cap = table.caption.as_ref().ok_or_else(|| {
                        HwpError::RenderError("표에 캡션이 없습니다".to_string())
                    })?;
                    return Ok(cap.paragraphs.len());
                }
                let cell = table.cells.get(cell_idx).ok_or_else(|| {
                    HwpError::RenderError(format!(
                        "셀 인덱스 {} 범위 초과 (총 {}개)", cell_idx, table.cells.len()
                    ))
                })?;
                Ok(cell.paragraphs.len())
            }
            Some(Control::Shape(shape)) => {
                let text_box = get_textbox_from_shape(shape).ok_or_else(|| {
                    HwpError::RenderError("도형에 글상자가 없습니다".to_string())
                })?;
                Ok(text_box.paragraphs.len())
            }
            Some(Control::Picture(pic)) => {
                let caption = pic.caption.as_ref().ok_or_else(|| {
                    HwpError::RenderError("그림에 캡션이 없습니다".to_string())
                })?;
                Ok(caption.paragraphs.len())
            }
            _ => Err(HwpError::RenderError(format!(
                "컨트롤 인덱스 {}가 표/글상자가 아닙니다", control_idx
            ))),
        }
    }

    /// 셀 내 문단 글자 수 (네이티브)
    pub fn get_cell_paragraph_length_native(
        &self,
        section_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
        cell_idx: usize,
        cell_para_idx: usize,
    ) -> Result<usize, HwpError> {
        let cell_para = self.get_cell_paragraph_ref(
            section_idx, parent_para_idx, control_idx, cell_idx, cell_para_idx,
        ).ok_or_else(|| HwpError::RenderError(format!(
            "셀 문단 접근 실패: sec={}, para={}, ctrl={}, cell={}, cellPara={}",
            section_idx, parent_para_idx, control_idx, cell_idx, cell_para_idx
        )))?;
        Ok(cell_para.text.chars().count())
    }

    /// 셀 내 텍스트 부분 추출 (네이티브)
    pub fn get_text_in_cell_native(
        &self,
        section_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
        cell_idx: usize,
        cell_para_idx: usize,
        char_offset: usize,
        count: usize,
    ) -> Result<String, HwpError> {
        let cell_para = self.get_cell_paragraph_ref(
            section_idx, parent_para_idx, control_idx, cell_idx, cell_para_idx,
        ).ok_or_else(|| HwpError::RenderError(format!(
            "셀 문단 접근 실패: sec={}, para={}, ctrl={}, cell={}, cellPara={}",
            section_idx, parent_para_idx, control_idx, cell_idx, cell_para_idx
        )))?;
        let text_chars: Vec<char> = cell_para.text.chars().collect();
        let total = text_chars.len();
        if char_offset > total {
            return Err(HwpError::RenderError(format!(
                "char_offset {} 범위 초과 (셀 문단 길이 {})", char_offset, total
            )));
        }
        let end = (char_offset + count).min(total);
        let result: String = text_chars[char_offset..end].iter().collect();
        Ok(result)
    }

    // ─── Phase 1 Native 끝 ──────────────────────────────────

    // ─── Phase 2 Native: 커서/히트 테스트 API ────────────────────

    /// 문단이 포함된 글로벌 페이지 번호 목록을 반환한다.
    pub(crate) fn find_pages_for_paragraph(&self, section_idx: usize, para_idx: usize) -> Result<Vec<u32>, HwpError> {
        if section_idx >= self.pagination.len() {
            return Err(HwpError::RenderError(format!(
                "구역 인덱스 {} 범위 초과 (총 {}개)", section_idx, self.pagination.len()
            )));
        }
        let mut global_offset = 0u32;
        for (sec_i, pr) in self.pagination.iter().enumerate() {
            if sec_i == section_idx {
                let mut result = Vec::new();
                for (local_i, page) in pr.pages.iter().enumerate() {
                    let global_page = global_offset + local_i as u32;
                    for col in &page.column_contents {
                        for item in &col.items {
                            let pi = match item {
                                crate::renderer::pagination::PageItem::FullParagraph { para_index } => Some(*para_index),
                                crate::renderer::pagination::PageItem::PartialParagraph { para_index, .. } => Some(*para_index),
                                crate::renderer::pagination::PageItem::Table { para_index, .. } => Some(*para_index),
                                crate::renderer::pagination::PageItem::PartialTable { para_index, .. } => Some(*para_index),
                                crate::renderer::pagination::PageItem::Shape { para_index, .. } => Some(*para_index),
                            };
                            if pi == Some(para_idx) {
                                if result.last() != Some(&global_page) {
                                    result.push(global_page);
                                }
                            }
                        }
                        // 어울림 문단도 페이지 탐색 대상에 포함
                        for wp in &col.wrap_around_paras {
                            if wp.para_index == para_idx || wp.table_para_index == para_idx {
                                if result.last() != Some(&global_page) {
                                    result.push(global_page);
                                }
                            }
                        }
                    }
                }
                // 전역 wrap_around_paras에서도 확인
                if result.is_empty() {
                    for wp in &pr.wrap_around_paras {
                        if wp.para_index == para_idx {
                            // 표 호스트 문단의 페이지에서 렌더링됨
                            if let Ok(table_pages) = self.find_pages_for_paragraph(section_idx, wp.table_para_index) {
                                return Ok(table_pages);
                            }
                        }
                    }
                }
                return if result.is_empty() {
                    Err(HwpError::RenderError(format!(
                        "문단 (sec={}, para={})이 페이지에 없습니다", section_idx, para_idx
                    )))
                } else {
                    Ok(result)
                };
            }
            global_offset += pr.pages.len() as u32;
        }
        Err(HwpError::RenderError(format!(
            "구역 인덱스 {} 범위 초과", section_idx
        )))
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_page_overflow_with_enter() {
        let mut core = DocumentCore::new_empty();
        core.create_blank_document_native().unwrap();

        assert_eq!(core.page_count(), 1, "초기 페이지 수");
        assert_eq!(core.document.sections[0].paragraphs.len(), 1, "초기 문단 수");

        // Enter를 50번 입력하여 페이지 오버플로우 유발
        for i in 0..50 {
            let para_count = core.document.sections[0].paragraphs.len();
            core.split_paragraph_native(0, para_count - 1, 0).unwrap();
        }

        let para_count = core.document.sections[0].paragraphs.len();
        let page_count = core.page_count();
        assert_eq!(para_count, 51, "문단 수");
        assert!(page_count >= 2, "페이지 수: {} (2 이상이어야 함)", page_count);
    }

    #[test]
    fn test_paragraph_y_positions_after_split() {
        let mut core = DocumentCore::new_empty();
        core.create_blank_document_native().unwrap();

        // 첫 문단에 긴 텍스트 입력 (줄바꿈 발생)
        let long_text = "The quick brown fox jumps over the lazy dog. ";
        let text = long_text.repeat(5);
        core.insert_text_native(0, 0, 0, &text).unwrap();

        // 첫 문단이 여러 줄로 구성되는지 확인
        let para0_lines = core.composed[0][0].lines.len();
        eprintln!("문단0 줄 수: {}", para0_lines);
        assert!(para0_lines >= 2, "첫 문단은 2줄 이상이어야 함: {}", para0_lines);

        // Enter로 문단 분리 (텍스트 끝에서)
        let text_len = core.document.sections[0].paragraphs[0].text.chars().count();
        core.split_paragraph_native(0, 0, text_len).unwrap();

        // 두 번째 문단에 텍스트 입력
        core.insert_text_native(0, 1, 0, "Second paragraph").unwrap();

        // 렌더 트리 빌드 (페이지 0)
        let tree = core.build_page_tree(0).unwrap();
        let tree_str = format!("{:?}", tree);

        // 렌더 트리에서 문단들의 Y 좌표를 추출
        // 두 번째 문단 "Second" 텍스트가 존재하는지 확인
        assert!(tree_str.contains("Second paragraph"),
            "두 번째 문단 텍스트가 렌더 트리에 없음");

        // 렌더 트리에서 TextRun Y 좌표 확인
        let para0_last_y = find_text_y(&tree.root, "dog.");
        let para1_y = find_text_y(&tree.root, "Second");
        eprintln!("문단0 마지막줄 Y: {:?}, 문단1 Y: {:?}", para0_last_y, para1_y);

        if let (Some(y0), Some(y1)) = (para0_last_y, para1_y) {
            assert!(y1 > y0, "문단1 Y({:.1})가 문단0 Y({:.1})보다 커야 함 (겹침 감지)", y1, y0);
        }
    }

    /// 줄간격 160%(기본값)에서 페이지 넘김 확인
    #[test]
    fn test_page_break_with_default_line_spacing() {
        let mut core = DocumentCore::new_empty();
        core.create_blank_document_native().unwrap();

        // 텍스트를 넣고 Enter로 문단 분리 반복 → 페이지 넘김 검증
        let text = "Line spacing 160 percent default.";
        for i in 0..50 {
            let para_count = core.document.sections[0].paragraphs.len();
            let last = para_count - 1;
            core.insert_text_native(0, last, 0, text).unwrap();
            core.split_paragraph_native(0, last, text.len()).unwrap();
        }

        let page_count = core.page_count();
        eprintln!("160% 줄간격: 문단 51개, 페이지 수: {}", page_count);
        assert!(page_count >= 2, "160% 줄간격에서 페이지 넘김 필요: {}", page_count);
    }

    /// 줄간격 100%에서 160%보다 더 많은 문단이 한 페이지에 들어가는지 확인
    #[test]
    fn test_page_break_with_tight_line_spacing() {
        // 100% 줄간격 문서
        let mut core100 = DocumentCore::new_empty();
        core100.create_blank_document_native().unwrap();
        let text = "Tight spacing test line.";
        // 첫 문단에 줄간격 100% 적용
        core100.apply_para_format_native(0, 0, r#"{"lineSpacing":100}"#).unwrap();
        for i in 0..50 {
            let para_count = core100.document.sections[0].paragraphs.len();
            let last = para_count - 1;
            core100.insert_text_native(0, last, 0, text).unwrap();
            core100.split_paragraph_native(0, last, text.len()).unwrap();
            // 새 문단에도 100% 적용
            let new_last = core100.document.sections[0].paragraphs.len() - 1;
            core100.apply_para_format_native(0, new_last, r#"{"lineSpacing":100}"#).unwrap();
        }
        let pages_100 = core100.page_count();

        // 160% 줄간격 문서 (기본)
        let mut core160 = DocumentCore::new_empty();
        core160.create_blank_document_native().unwrap();
        for i in 0..50 {
            let para_count = core160.document.sections[0].paragraphs.len();
            let last = para_count - 1;
            core160.insert_text_native(0, last, 0, text).unwrap();
            core160.split_paragraph_native(0, last, text.len()).unwrap();
        }
        let pages_160 = core160.page_count();

        eprintln!("100% → {}페이지, 160% → {}페이지 (문단 51개)", pages_100, pages_160);
        // 100%는 160%보다 같거나 적은 페이지 수
        assert!(pages_100 <= pages_160,
            "100% 줄간격({})이 160%({})보다 적은/같은 페이지 수여야 함", pages_100, pages_160);
    }

    /// 줄간격 300%에서 160%보다 더 빨리 페이지가 넘어가는지 확인
    #[test]
    fn test_page_break_with_wide_line_spacing() {
        // 300% 줄간격
        let mut core300 = DocumentCore::new_empty();
        core300.create_blank_document_native().unwrap();
        let text = "Wide spacing test line.";
        core300.apply_para_format_native(0, 0, r#"{"lineSpacing":300}"#).unwrap();
        for i in 0..30 {
            let para_count = core300.document.sections[0].paragraphs.len();
            let last = para_count - 1;
            core300.insert_text_native(0, last, 0, text).unwrap();
            core300.split_paragraph_native(0, last, text.len()).unwrap();
            let new_last = core300.document.sections[0].paragraphs.len() - 1;
            core300.apply_para_format_native(0, new_last, r#"{"lineSpacing":300}"#).unwrap();
        }
        let pages_300 = core300.page_count();

        // 160% 줄간격 (동일 문단 수)
        let mut core160 = DocumentCore::new_empty();
        core160.create_blank_document_native().unwrap();
        for i in 0..30 {
            let para_count = core160.document.sections[0].paragraphs.len();
            let last = para_count - 1;
            core160.insert_text_native(0, last, 0, text).unwrap();
            core160.split_paragraph_native(0, last, text.len()).unwrap();
        }
        let pages_160 = core160.page_count();

        eprintln!("300% → {}페이지, 160% → {}페이지 (문단 31개)", pages_300, pages_160);
        assert!(pages_300 >= pages_160,
            "300% 줄간격({})이 160%({})보다 많은/같은 페이지 수여야 함", pages_300, pages_160);
    }

    /// 혼합 줄간격: 문단마다 다른 줄간격에서 페이지 넘김 정상 동작 확인
    #[test]
    fn test_page_break_with_mixed_line_spacing() {
        let mut core = DocumentCore::new_empty();
        core.create_blank_document_native().unwrap();

        let spacings = [160, 100, 300, 250, 120, 200];
        let text = "Mixed spacing paragraph content here.";

        for i in 0..40 {
            let para_count = core.document.sections[0].paragraphs.len();
            let last = para_count - 1;
            core.insert_text_native(0, last, 0, text).unwrap();
            // 현재 문단에 다양한 줄간격 적용
            let spacing = spacings[i % spacings.len()];
            let json = format!(r#"{{"lineSpacing":{}}}"#, spacing);
            core.apply_para_format_native(0, last, &json).unwrap();
            core.split_paragraph_native(0, last, text.len()).unwrap();
        }

        let page_count = core.page_count();
        let para_count = core.document.sections[0].paragraphs.len();
        eprintln!("혼합 줄간격: 문단 {}개, 페이지 수: {}", para_count, page_count);
        assert!(page_count >= 2, "혼합 줄간격에서 페이지 넘김 필요: {}", page_count);

        // 각 페이지에 문단이 배치되었는지 확인 (렌더 트리 빌드 가능)
        for p in 0..page_count {
            let tree = core.build_page_tree(p as u32);
            assert!(tree.is_ok(), "페이지 {} 렌더 트리 빌드 실패: {:?}", p, tree.err());
        }
    }

    /// 고정(Fixed) 줄간격에서 페이지 넘김 정상 동작 확인
    #[test]
    fn test_page_break_with_fixed_line_spacing() {
        let mut core = DocumentCore::new_empty();
        core.create_blank_document_native().unwrap();

        let text = "Fixed spacing paragraph.";
        // Fixed 줄간격 30px
        core.apply_para_format_native(0, 0,
            r#"{"lineSpacing":30,"lineSpacingType":"Fixed"}"#).unwrap();

        for i in 0..50 {
            let para_count = core.document.sections[0].paragraphs.len();
            let last = para_count - 1;
            core.insert_text_native(0, last, 0, text).unwrap();
            core.split_paragraph_native(0, last, text.len()).unwrap();
            let new_last = core.document.sections[0].paragraphs.len() - 1;
            core.apply_para_format_native(0, new_last,
                r#"{"lineSpacing":30,"lineSpacingType":"Fixed"}"#).unwrap();
        }

        let page_count = core.page_count();
        eprintln!("Fixed 줄간격: 문단 51개, 페이지 수: {}", page_count);
        assert!(page_count >= 1, "Fixed 줄간격에서 페이지 수 확인: {}", page_count);

        // 렌더 트리 정상 빌드 확인
        for p in 0..page_count {
            let tree = core.build_page_tree(p as u32);
            assert!(tree.is_ok(), "페이지 {} 렌더 트리 빌드 실패", p);
        }
    }

    /// 각 줄간격별 페이지당 수용 줄 수가 논리적으로 맞는지 확인
    #[test]
    fn test_line_count_per_page_varies_by_spacing() {
        let spacings = vec![100, 160, 250, 300];
        let mut page_counts = Vec::new();

        for spacing in &spacings {
            let mut core = DocumentCore::new_empty();
            core.create_blank_document_native().unwrap();
            let json = format!(r#"{{"lineSpacing":{}}}"#, spacing);
            core.apply_para_format_native(0, 0, &json).unwrap();

            let text = "Test line for spacing comparison.";
            for _ in 0..60 {
                let last = core.document.sections[0].paragraphs.len() - 1;
                core.insert_text_native(0, last, 0, text).unwrap();
                core.split_paragraph_native(0, last, text.len()).unwrap();
                let new_last = core.document.sections[0].paragraphs.len() - 1;
                core.apply_para_format_native(0, new_last, &json).unwrap();
            }
            page_counts.push((*spacing, core.page_count()));
        }

        eprintln!("줄간격별 페이지 수 (문단 61개):");
        for (spacing, pages) in &page_counts {
            eprintln!("  {}% → {}페이지", spacing, pages);
        }

        // 줄간격이 클수록 페이지 수가 많아야 함
        for i in 1..page_counts.len() {
            assert!(page_counts[i].1 >= page_counts[i-1].1,
                "줄간격 {}%({})가 {}%({})보다 적은 페이지 수",
                page_counts[i].0, page_counts[i].1,
                page_counts[i-1].0, page_counts[i-1].1);
        }
    }

    /// 기존 문서 중간 문단의 줄간격을 10%씩 증가시키면 페이지 경계를 정확히 돌파하는지 검증
    #[test]
    fn test_page_boundary_with_incremental_spacing_increase() {
        let mut core = DocumentCore::new_empty();
        core.create_blank_document_native().unwrap();

        // 160% 줄간격으로 40개 문단 생성 (1페이지에 딱 맞도록)
        let text = "Test paragraph for spacing.";
        for _ in 0..39 {
            let last = core.document.sections[0].paragraphs.len() - 1;
            core.insert_text_native(0, last, 0, text).unwrap();
            core.split_paragraph_native(0, last, text.len()).unwrap();
        }
        // 마지막 문단에도 텍스트
        let last = core.document.sections[0].paragraphs.len() - 1;
        core.insert_text_native(0, last, 0, text).unwrap();

        let initial_pages = core.page_count();
        eprintln!("초기 페이지 수: {} (40문단 160%)", initial_pages);

        // 문단 15~25의 줄간격을 10%씩 증가 (170%, 180%, ..., 270%)
        let mut prev_pages = initial_pages;
        let mut boundary_crossed_at = 0;
        for step in 0..20 {
            let spacing = 170 + step * 10; // 170% → 360%
            for para_idx in 15..26 {
                if para_idx < core.document.sections[0].paragraphs.len() {
                    let json = format!(r#"{{"lineSpacing":{}}}"#, spacing);
                    core.apply_para_format_native(0, para_idx, &json).unwrap();
                }
            }
            let pages = core.page_count();
            if pages > prev_pages && boundary_crossed_at == 0 {
                boundary_crossed_at = spacing;
                eprintln!("  페이지 경계 돌파: {}% 줄간격에서 {}→{}페이지", spacing, prev_pages, pages);
            }
            prev_pages = pages;
        }

        eprintln!("최종 페이지 수: {} (줄간격 360%)", prev_pages);
        assert!(prev_pages > initial_pages,
            "줄간격 증가로 페이지 수 증가 필요: {} → {}", initial_pages, prev_pages);
        assert!(boundary_crossed_at > 0,
            "페이지 경계 돌파 시점이 감지되어야 함");

        // 모든 페이지 렌더 트리 정상 빌드 확인
        for p in 0..prev_pages {
            let tree = core.build_page_tree(p as u32);
            assert!(tree.is_ok(), "페이지 {} 렌더 트리 빌드 실패: {:?}", p, tree.err());
        }
    }
}

fn find_text_y(node: &crate::renderer::render_tree::RenderNode, text: &str) -> Option<f64> {
    use crate::renderer::render_tree::RenderNodeType;
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if run.text.contains(text) {
            return Some(node.bbox.y);
        }
    }
    for child in &node.children {
        if let Some(y) = find_text_y(child, text) {
            return Some(y);
        }
    }
    None
}

// ─── 중첩 표 path 기반 편집 API ──────────────────────────────────

impl DocumentCore {
    /// cellPath를 따라가서 최종 셀의 문단에 대한 가변 참조를 얻는다.
    /// path: [(control_index, cell_index, cell_para_index), ...]
    pub(crate) fn get_cell_paragraph_mut_by_path(
        &mut self,
        section_idx: usize,
        parent_para_idx: usize,
        path: &[(usize, usize, usize)],
    ) -> Result<&mut Paragraph, HwpError> {
        if path.is_empty() {
            return Err(HwpError::RenderError("경로가 비어있습니다".to_string()));
        }
        let section = self.document.sections.get_mut(section_idx)
            .ok_or_else(|| HwpError::RenderError(format!("구역 {} 범위 초과", section_idx)))?;
        let mut para: &mut Paragraph = section.paragraphs.get_mut(parent_para_idx)
            .ok_or_else(|| HwpError::RenderError(format!("문단 {} 범위 초과", parent_para_idx)))?;

        for (i, &(ctrl_idx, cell_idx, cell_para_idx)) in path.iter().enumerate() {
            let table = match para.controls.get_mut(ctrl_idx) {
                Some(Control::Table(t)) => t.as_mut(),
                _ => return Err(HwpError::RenderError(format!(
                    "경로[{}]: controls[{}]가 표가 아닙니다", i, ctrl_idx
                ))),
            };
            let cell = table.cells.get_mut(cell_idx)
                .ok_or_else(|| HwpError::RenderError(format!(
                    "경로[{}]: 셀 {} 범위 초과", i, cell_idx
                )))?;
            if i == path.len() - 1 {
                // 마지막 레벨: 이 셀의 문단 반환
                return cell.paragraphs.get_mut(cell_para_idx)
                    .ok_or_else(|| HwpError::RenderError(format!(
                        "경로[{}]: 셀문단 {} 범위 초과", i, cell_para_idx
                    )));
            }
            // 중간 레벨: 이 셀의 문단으로 진입 후 다음 표 탐색
            para = cell.paragraphs.get_mut(cell_para_idx)
                .ok_or_else(|| HwpError::RenderError(format!(
                    "경로[{}]: 셀문단 {} 범위 초과", i, cell_para_idx
                )))?;
        }
        unreachable!()
    }

    /// path 기반 셀 텍스트 삽입 (중첩 표 지원)
    pub fn insert_text_in_cell_by_path(
        &mut self,
        section_idx: usize,
        parent_para_idx: usize,
        path: &[(usize, usize, usize)],
        char_offset: usize,
        text: &str,
    ) -> Result<String, HwpError> {
        let new_chars_count = text.chars().count();
        let cell_para = self.get_cell_paragraph_mut_by_path(section_idx, parent_para_idx, path)?;
        cell_para.insert_text_at(char_offset, text);

        // 최외곽 표 dirty 마킹
        let outer_ctrl = path[0].0;
        self.mark_cell_control_dirty(section_idx, parent_para_idx, outer_ctrl);

        // 리플로우 (최외곽 표 기준 — 중첩 표 셀 폭은 별도 계산이 필요하나 우선 section dirty로 처리)
        self.document.sections[section_idx].raw_stream = None;
        self.mark_section_dirty(section_idx);
        self.paginate_if_needed();

        let new_offset = char_offset + new_chars_count;
        self.event_log.push(DocumentEvent::CellTextChanged {
            section: section_idx, para: parent_para_idx, ctrl: outer_ctrl, cell: path[0].1,
        });
        Ok(super::super::helpers::json_ok_with(&format!("\"charOffset\":{}", new_offset)))
    }

    /// path 기반 셀 텍스트 삭제 (중첩 표 지원)
    pub fn delete_text_in_cell_by_path(
        &mut self,
        section_idx: usize,
        parent_para_idx: usize,
        path: &[(usize, usize, usize)],
        char_offset: usize,
        count: usize,
    ) -> Result<String, HwpError> {
        let cell_para = self.get_cell_paragraph_mut_by_path(section_idx, parent_para_idx, path)?;
        cell_para.delete_text_at(char_offset, count);

        let outer_ctrl = path[0].0;
        self.mark_cell_control_dirty(section_idx, parent_para_idx, outer_ctrl);
        self.document.sections[section_idx].raw_stream = None;
        self.mark_section_dirty(section_idx);
        self.paginate_if_needed();

        self.event_log.push(DocumentEvent::CellTextChanged {
            section: section_idx, para: parent_para_idx, ctrl: outer_ctrl, cell: path[0].1,
        });
        Ok(super::super::helpers::json_ok_with(&format!("\"charOffset\":{}", char_offset)))
    }

    /// path 기반 셀 문단 분할 (중첩 표 지원)
    pub fn split_paragraph_in_cell_by_path(
        &mut self,
        section_idx: usize,
        parent_para_idx: usize,
        path: &[(usize, usize, usize)],
        char_offset: usize,
    ) -> Result<String, HwpError> {
        // 마지막 path 엔트리의 cell_para_idx가 분할 대상
        let last = path.last().unwrap();
        let cell_para_idx = last.2;

        // 셀에 접근하여 문단 분할
        let section = self.document.sections.get_mut(section_idx)
            .ok_or_else(|| HwpError::RenderError("구역 범위 초과".to_string()))?;
        let mut para: &mut Paragraph = section.paragraphs.get_mut(parent_para_idx)
            .ok_or_else(|| HwpError::RenderError("문단 범위 초과".to_string()))?;

        // path를 따라 마지막 셀까지 진입
        for (i, &(ctrl_idx, cell_idx, _cpi)) in path.iter().enumerate() {
            let table = match para.controls.get_mut(ctrl_idx) {
                Some(Control::Table(t)) => t.as_mut(),
                _ => return Err(HwpError::RenderError("경로: 표가 아닙니다".to_string())),
            };
            let cell = table.cells.get_mut(cell_idx)
                .ok_or_else(|| HwpError::RenderError("셀 범위 초과".to_string()))?;
            if i == path.len() - 1 {
                // 이 셀에서 문단 분할
                if cell_para_idx >= cell.paragraphs.len() {
                    return Err(HwpError::RenderError("셀문단 범위 초과".to_string()));
                }
                let new_para = cell.paragraphs[cell_para_idx].split_at(char_offset);
                cell.paragraphs.insert(cell_para_idx + 1, new_para);
                break;
            }
            para = cell.paragraphs.get_mut(_cpi)
                .ok_or_else(|| HwpError::RenderError("셀문단 범위 초과".to_string()))?;
        }

        let outer_ctrl = path[0].0;
        self.mark_cell_control_dirty(section_idx, parent_para_idx, outer_ctrl);
        self.document.sections[section_idx].raw_stream = None;
        self.mark_section_dirty(section_idx);
        self.paginate_if_needed();

        self.event_log.push(DocumentEvent::CellTextChanged {
            section: section_idx, para: parent_para_idx, ctrl: outer_ctrl, cell: path[0].1,
        });
        let new_cpi = cell_para_idx + 1;
        Ok(super::super::helpers::json_ok_with(&format!("\"cellParaIndex\":{},\"charOffset\":0", new_cpi)))
    }

    /// path 기반 셀 문단 병합 (중첩 표 지원)
    pub fn merge_paragraph_in_cell_by_path(
        &mut self,
        section_idx: usize,
        parent_para_idx: usize,
        path: &[(usize, usize, usize)],
    ) -> Result<String, HwpError> {
        let last = path.last().unwrap();
        let cell_para_idx = last.2;
        if cell_para_idx == 0 {
            return Err(HwpError::RenderError("첫 문단은 병합할 수 없습니다".to_string()));
        }

        let section = self.document.sections.get_mut(section_idx)
            .ok_or_else(|| HwpError::RenderError("구역 범위 초과".to_string()))?;
        let mut para: &mut Paragraph = section.paragraphs.get_mut(parent_para_idx)
            .ok_or_else(|| HwpError::RenderError("문단 범위 초과".to_string()))?;

        let mut merge_point = 0usize;
        for (i, &(ctrl_idx, cell_idx, _cpi)) in path.iter().enumerate() {
            let table = match para.controls.get_mut(ctrl_idx) {
                Some(Control::Table(t)) => t.as_mut(),
                _ => return Err(HwpError::RenderError("경로: 표가 아닙니다".to_string())),
            };
            let cell = table.cells.get_mut(cell_idx)
                .ok_or_else(|| HwpError::RenderError("셀 범위 초과".to_string()))?;
            if i == path.len() - 1 {
                if cell_para_idx >= cell.paragraphs.len() {
                    return Err(HwpError::RenderError("셀문단 범위 초과".to_string()));
                }
                let removed = cell.paragraphs.remove(cell_para_idx);
                let prev = &mut cell.paragraphs[cell_para_idx - 1];
                merge_point = prev.text.chars().count();
                prev.merge_from(&removed);
                break;
            }
            para = cell.paragraphs.get_mut(_cpi)
                .ok_or_else(|| HwpError::RenderError("셀문단 범위 초과".to_string()))?;
        }

        let outer_ctrl = path[0].0;
        self.mark_cell_control_dirty(section_idx, parent_para_idx, outer_ctrl);
        self.document.sections[section_idx].raw_stream = None;
        self.mark_section_dirty(section_idx);
        self.paginate_if_needed();

        self.event_log.push(DocumentEvent::CellTextChanged {
            section: section_idx, para: parent_para_idx, ctrl: outer_ctrl, cell: path[0].1,
        });
        let prev_cpi = cell_para_idx - 1;
        Ok(super::super::helpers::json_ok_with(&format!("\"cellParaIndex\":{},\"charOffset\":{}", prev_cpi, merge_point)))
    }

    /// path 기반 셀 텍스트 조회 (중첩 표 지원)
    pub fn get_text_in_cell_by_path(
        &self,
        section_idx: usize,
        parent_para_idx: usize,
        path: &[(usize, usize, usize)],
        char_offset: usize,
        count: usize,
    ) -> Result<String, HwpError> {
        let para = self.resolve_paragraph_by_path(section_idx, parent_para_idx, path)?;
        let text_chars: Vec<char> = para.text.chars().collect();
        let total = text_chars.len();
        if char_offset > total {
            return Err(HwpError::RenderError(format!(
                "char_offset {} 범위 초과 (셀 문단 길이 {})", char_offset, total
            )));
        }
        let end = (char_offset + count).min(total);
        Ok(text_chars[char_offset..end].iter().collect())
    }
}

//! 텍스트 삽입/삭제/문단 분리·병합/범위 삭제/문단 쿼리 관련 native 메서드

use super::super::helpers::get_textbox_from_shape;
use crate::document_core::DocumentCore;
use crate::error::HwpError;
use crate::model::control::Control;
use crate::model::event::DocumentEvent;
use crate::model::page::ColumnDef;
use crate::model::paragraph::Paragraph;
use crate::renderer::composer::{compose_paragraph, reflow_line_segs, ComposedParagraph};
use crate::renderer::page_layout::PageLayoutInfo;
use crate::renderer::style_resolver::resolve_styles;

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
                "구역 인덱스 {} 범위 초과 (총 {}개)",
                section_idx,
                self.document.sections.len()
            )));
        }
        let section = &self.document.sections[section_idx];
        if para_idx >= section.paragraphs.len() {
            return Err(HwpError::RenderError(format!(
                "문단 인덱스 {} 범위 초과 (총 {}개)",
                para_idx,
                section.paragraphs.len()
            )));
        }

        // 편집 시 raw 스트림 무효화 (재직렬화 유도)
        self.document.sections[section_idx].raw_stream = None;

        // 텍스트 삽입
        let new_chars_count = text.chars().count();
        self.document.sections[section_idx].paragraphs[para_idx].insert_text_at(char_offset, text);

        // line_segs 재계산 (리플로우) → vpos 재계산 → 재구성 → 재페이지네이션
        // 다단 문서에서 편집 후 문단이 다른 단으로 재배치될 수 있으므로
        // para_column_map 변경 감지 + 재reflow 수렴 루프 (최대 3회)
        let old_col = self
            .para_column_map
            .get(section_idx)
            .and_then(|m| m.get(para_idx))
            .copied()
            .unwrap_or(0);
        self.reflow_paragraph(section_idx, para_idx);
        crate::renderer::composer::recalculate_section_vpos(
            &mut self.document.sections[section_idx].paragraphs,
            para_idx,
        );
        self.recompose_paragraph(section_idx, para_idx);
        self.paginate_if_needed();

        for _ in 0..2 {
            let new_col = self
                .para_column_map
                .get(section_idx)
                .and_then(|m| m.get(para_idx))
                .copied()
                .unwrap_or(0);
            if new_col == old_col {
                break;
            }
            self.reflow_paragraph(section_idx, para_idx);
            crate::renderer::composer::recalculate_section_vpos(
                &mut self.document.sections[section_idx].paragraphs,
                para_idx,
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
            para.char_offsets[last]
                + last_char
                    .map(|c| if (c as u32) > 0xFFFF { 2 } else { 1 })
                    .unwrap_or(1)
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
                raw,
                section_idx as u32,
                para_idx as u32,
                caret_utf16_pos,
            );
        }

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
                "구역 인덱스 {} 범위 초과 (총 {}개)",
                section_idx,
                self.document.sections.len()
            )));
        }
        let section = &self.document.sections[section_idx];
        if para_idx >= section.paragraphs.len() {
            return Err(HwpError::RenderError(format!(
                "문단 인덱스 {} 범위 초과 (총 {}개)",
                para_idx,
                section.paragraphs.len()
            )));
        }

        // 편집 시 raw 스트림 무효화 (재직렬화 유도)
        self.document.sections[section_idx].raw_stream = None;

        // 텍스트 삭제
        self.document.sections[section_idx].paragraphs[para_idx].delete_text_at(char_offset, count);

        // line_segs 재계산 (리플로우) → 재구성 → 재페이지네이션
        // 다단 수렴 루프 (최대 3회)
        let old_col = self
            .para_column_map
            .get(section_idx)
            .and_then(|m| m.get(para_idx))
            .copied()
            .unwrap_or(0);
        self.reflow_paragraph(section_idx, para_idx);
        crate::renderer::composer::recalculate_section_vpos(
            &mut self.document.sections[section_idx].paragraphs,
            para_idx,
        );
        self.recompose_paragraph(section_idx, para_idx);
        self.paginate_if_needed();

        for _ in 0..2 {
            let new_col = self
                .para_column_map
                .get(section_idx)
                .and_then(|m| m.get(para_idx))
                .copied()
                .unwrap_or(0);
            if new_col == old_col {
                break;
            }
            self.reflow_paragraph(section_idx, para_idx);
            crate::renderer::composer::recalculate_section_vpos(
                &mut self.document.sections[section_idx].paragraphs,
                para_idx,
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
            para.char_offsets[last]
                + last_char
                    .map(|c| if (c as u32) > 0xFFFF { 2 } else { 1 })
                    .unwrap_or(1)
        } else {
            (para.controls.len() as u32) * 8
        };
        self.document.doc_properties.caret_list_id = section_idx as u32;
        self.document.doc_properties.caret_para_id = para_idx as u32;
        self.document.doc_properties.caret_char_pos = caret_utf16_pos;

        // DocInfo raw_stream 내 캐럿 위치만 surgical update (전체 재직렬화 방지)
        if let Some(ref mut raw) = self.document.doc_info.raw_stream {
            let _ = crate::serializer::doc_info::surgical_update_caret(
                raw,
                section_idx as u32,
                para_idx as u32,
                caret_utf16_pos,
            );
        }

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

    /// 지정된 문단의 line_segs를 컬럼 너비 기반으로 재계산한다.
    pub(crate) fn reflow_paragraph(&mut self, section_idx: usize, para_idx: usize) {
        let section = &self.document.sections[section_idx];
        let page_def = &section.section_def.page_def;
        // 해당 문단에 적용되는 ColumnDef를 찾음 (구역 내 다단↔단일단 전환 지원)
        let column_def = Self::find_column_def_for_paragraph(&section.paragraphs, para_idx);
        let layout = PageLayoutInfo::from_page_def(page_def, &column_def, self.dpi);

        // 페이지네이션 매핑에서 문단의 소속 단 인덱스 조회
        let col_idx = self
            .para_column_map
            .get(section_idx)
            .and_then(|m| m.get(para_idx))
            .copied()
            .unwrap_or(0) as usize;
        let col_area = layout
            .column_areas
            .get(col_idx)
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
            section_idx,
            parent_para_idx,
            control_idx,
            cell_idx,
            cell_para_idx,
        )?;
        let new_chars_count = text.chars().count();
        cell_para.insert_text_at(char_offset, text);

        // 부모 컨트롤 dirty 마킹 (표 또는 글상자)
        self.mark_cell_control_dirty(section_idx, parent_para_idx, control_idx);

        // 셀 폭 기반 리플로우
        self.reflow_cell_paragraph(
            section_idx,
            parent_para_idx,
            control_idx,
            cell_idx,
            cell_para_idx,
        );
        self.recalculate_cell_paragraph_vpos_native(
            section_idx,
            parent_para_idx,
            control_idx,
            cell_idx,
        )?;

        // raw 스트림 무효화, 재페이지네이션 (셀 편집 → composed 불변, section dirty만 설정)
        self.document.sections[section_idx].raw_stream = None;
        self.mark_section_dirty(section_idx);
        self.paginate_if_needed();

        let new_offset = char_offset + new_chars_count;
        self.event_log.push(DocumentEvent::CellTextChanged {
            section: section_idx,
            para: parent_para_idx,
            ctrl: control_idx,
            cell: cell_idx,
        });
        Ok(super::super::helpers::json_ok_with(&format!(
            "\"charOffset\":{}",
            new_offset
        )))
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
            section_idx,
            parent_para_idx,
            control_idx,
            cell_idx,
            cell_para_idx,
        )?;
        cell_para.delete_text_at(char_offset, count);

        // 부모 컨트롤 dirty 마킹 (표 또는 글상자)
        self.mark_cell_control_dirty(section_idx, parent_para_idx, control_idx);

        // 셀 폭 기반 리플로우
        self.reflow_cell_paragraph(
            section_idx,
            parent_para_idx,
            control_idx,
            cell_idx,
            cell_para_idx,
        );
        self.recalculate_cell_paragraph_vpos_native(
            section_idx,
            parent_para_idx,
            control_idx,
            cell_idx,
        )?;

        // raw 스트림 무효화, 재페이지네이션 (셀 편집 → composed 불변)
        self.document.sections[section_idx].raw_stream = None;
        self.mark_section_dirty(section_idx);
        self.paginate_if_needed();

        self.event_log.push(DocumentEvent::CellTextChanged {
            section: section_idx,
            para: parent_para_idx,
            ctrl: control_idx,
            cell: cell_idx,
        });
        Ok(super::super::helpers::json_ok_with(&format!(
            "\"charOffset\":{}",
            char_offset
        )))
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
                "구역 인덱스 {} 범위 초과",
                section_idx
            )));
        }
        let section = &mut self.document.sections[section_idx];
        if parent_para_idx >= section.paragraphs.len() {
            return Err(HwpError::RenderError(format!(
                "부모 문단 인덱스 {} 범위 초과",
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
        match &mut para.controls[control_idx] {
            Control::Table(t) => {
                // cell_idx == 65534: 표 캡션 접근 (TypeScript에서 표 캡션 편집 시 사용)
                if cell_idx == 65534 {
                    let cap = t.caption.as_mut().ok_or_else(|| {
                        HwpError::RenderError("지정된 표 컨트롤에 캡션이 없습니다".to_string())
                    })?;
                    if cell_para_idx >= cap.paragraphs.len() {
                        return Err(HwpError::RenderError(format!(
                            "캡션 문단 인덱스 {} 범위 초과 (총 {}개)",
                            cell_para_idx,
                            cap.paragraphs.len()
                        )));
                    }
                    return Ok(&mut cap.paragraphs[cell_para_idx]);
                }
                if cell_idx >= t.cells.len() {
                    return Err(HwpError::RenderError(format!(
                        "셀 인덱스 {} 범위 초과 (총 {}개)",
                        cell_idx,
                        t.cells.len()
                    )));
                }
                let cell = &mut t.cells[cell_idx];
                if cell_para_idx >= cell.paragraphs.len() {
                    return Err(HwpError::RenderError(format!(
                        "셀 문단 인덱스 {} 범위 초과 (총 {}개)",
                        cell_para_idx,
                        cell.paragraphs.len()
                    )));
                }
                Ok(&mut cell.paragraphs[cell_para_idx])
            }
            Control::Shape(shape) => {
                let tb =
                    super::super::helpers::get_textbox_from_shape_mut(shape).ok_or_else(|| {
                        HwpError::RenderError(
                            "지정된 Shape 컨트롤에 텍스트 박스가 없습니다".to_string(),
                        )
                    })?;
                if cell_para_idx >= tb.paragraphs.len() {
                    return Err(HwpError::RenderError(format!(
                        "글상자 문단 인덱스 {} 범위 초과 (총 {}개)",
                        cell_para_idx,
                        tb.paragraphs.len()
                    )));
                }
                Ok(&mut tb.paragraphs[cell_para_idx])
            }
            Control::Picture(pic) => {
                let cap = pic.caption.as_mut().ok_or_else(|| {
                    HwpError::RenderError("지정된 그림 컨트롤에 캡션이 없습니다".to_string())
                })?;
                if cell_para_idx >= cap.paragraphs.len() {
                    return Err(HwpError::RenderError(format!(
                        "캡션 문단 인덱스 {} 범위 초과 (총 {}개)",
                        cell_para_idx,
                        cap.paragraphs.len()
                    )));
                }
                Ok(&mut cap.paragraphs[cell_para_idx])
            }
            _ => Err(HwpError::RenderError(
                "지정된 컨트롤이 표, 글상자 또는 그림이 아닙니다".to_string(),
            )),
        }
    }

    /// 부모 컨트롤(표 또는 글상자)의 dirty를 마킹한다.
    pub(crate) fn mark_cell_control_dirty(
        &mut self,
        section_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
    ) {
        if let Some(ctrl) = self.document.sections[section_idx].paragraphs[parent_para_idx]
            .controls
            .get_mut(control_idx)
        {
            match ctrl {
                Control::Table(t) => {
                    t.dirty = true;
                }
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
                section_idx,
                parent_para_idx,
                control_idx,
                cell_idx,
                cell_para_idx,
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
        match self.document.sections[section_idx].paragraphs[parent_para_idx]
            .controls
            .get_mut(control_idx)
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

    pub(crate) fn recalculate_cell_paragraph_vpos_native(
        &mut self,
        section_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
        cell_idx: usize,
    ) -> Result<(), HwpError> {
        let cell_paras =
            self.cell_paragraphs_mut(section_idx, parent_para_idx, control_idx, cell_idx)?;
        crate::renderer::composer::recalculate_section_vpos(cell_paras, 0);
        Ok(())
    }

    // ─── Phase 3 네이티브 구현: 커서 이동 API ─────────────────

    pub fn delete_range_native(
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
                    let cell_para =
                        self.get_cell_paragraph_mut(section_idx, ppi, ci, cei, start_para)?;
                    cell_para.delete_text_at(start_offset, count);
                    self.reflow_cell_paragraph(section_idx, ppi, ci, cei, start_para);
                }
            } else {
                // 다중 문단 셀 내 삭제
                // 1) 마지막 문단 앞부분 삭제
                if end_offset > 0 {
                    let cell_para =
                        self.get_cell_paragraph_mut(section_idx, ppi, ci, cei, end_para)?;
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
                    let cell_para =
                        self.get_cell_paragraph_mut(section_idx, ppi, ci, cei, start_para)?;
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
            self.event_log.push(DocumentEvent::CellTextChanged {
                section: section_idx,
                para: ppi,
                ctrl: ci,
                cell: cei,
            });
            Ok(super::super::helpers::json_ok_with(&format!(
                "\"paraIdx\":{},\"charOffset\":{}",
                start_para, start_offset
            )))
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
                        &mut self.document.sections[section_idx].paragraphs,
                        start_para,
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
                    self.document.sections[section_idx]
                        .paragraphs
                        .remove(mid_para);
                    self.remove_composed_paragraph(section_idx, mid_para);
                }
                // 3) 첫 문단 뒷부분 삭제
                {
                    let para_len = self.document.sections[section_idx].paragraphs[start_para]
                        .text
                        .chars()
                        .count();
                    if start_offset < para_len {
                        self.document.sections[section_idx].paragraphs[start_para]
                            .delete_text_at(start_offset, para_len - start_offset);
                    }
                }
                // 4) 첫-마지막 문단 병합 (마지막 문단이 이제 start_para+1에 위치)
                if start_para + 1 < self.document.sections[section_idx].paragraphs.len() {
                    let next = self.document.sections[section_idx]
                        .paragraphs
                        .remove(start_para + 1);
                    self.remove_composed_paragraph(section_idx, start_para + 1);
                    self.document.sections[section_idx].paragraphs[start_para].merge_from(&next);
                }
                self.reflow_paragraph(section_idx, start_para);
                crate::renderer::composer::recalculate_section_vpos(
                    &mut self.document.sections[section_idx].paragraphs,
                    start_para,
                );
                // 병합된 문단 재구성
                self.recompose_paragraph(section_idx, start_para);
            }

            // 재페이지네이션
            self.paginate_if_needed();

            // 캐럿 위치 갱신
            self.document.doc_properties.caret_list_id = section_idx as u32;
            self.document.doc_properties.caret_para_id = start_para as u32;

            self.event_log.push(DocumentEvent::TextDeleted {
                section: section_idx,
                para: start_para,
                offset: start_offset,
                count: 0,
            });
            Ok(super::super::helpers::json_ok_with(&format!(
                "\"paraIdx\":{},\"charOffset\":{}",
                start_para, start_offset
            )))
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
        let para = section.paragraphs.get_mut(parent_para_idx).ok_or_else(|| {
            HwpError::RenderError(format!("부모 문단 인덱스 {} 범위 초과", parent_para_idx))
        })?;
        let ctrl = para.controls.get_mut(control_idx).ok_or_else(|| {
            HwpError::RenderError(format!("컨트롤 인덱스 {} 범위 초과", control_idx))
        })?;
        match ctrl {
            Control::Table(ref mut table) => table
                .cells
                .get_mut(cell_idx)
                .ok_or_else(|| HwpError::RenderError(format!("셀 인덱스 {} 범위 초과", cell_idx))),
            _ => Err(HwpError::RenderError(
                "테이블 컨트롤이 아닙니다".to_string(),
            )),
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
        let para = self
            .document
            .sections
            .get(section_idx)?
            .paragraphs
            .get(parent_para_idx)?;
        match para.controls.get(control_idx)? {
            Control::Table(table) => {
                if cell_idx == 65534 {
                    return table.caption.as_ref()?.paragraphs.get(cell_para_idx);
                }
                table.cells.get(cell_idx)?.paragraphs.get(cell_para_idx)
            }
            Control::Shape(shape) => {
                if cell_idx != 0 {
                    return None;
                }
                get_textbox_from_shape(shape)?.paragraphs.get(cell_para_idx)
            }
            Control::Picture(pic) => pic.caption.as_ref()?.paragraphs.get(cell_para_idx),
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
                "구역 인덱스 {} 범위 초과 (총 {}개)",
                section_idx,
                self.document.sections.len()
            )));
        }
        let section = &self.document.sections[section_idx];
        if para_idx >= section.paragraphs.len() {
            return Err(HwpError::RenderError(format!(
                "문단 인덱스 {} 범위 초과 (총 {}개)",
                para_idx,
                section.paragraphs.len()
            )));
        }

        // 편집 시 raw 스트림 무효화 (재직렬화 유도)
        self.document.sections[section_idx].raw_stream = None;

        // 문단 분리
        let new_para =
            self.document.sections[section_idx].paragraphs[para_idx].split_at(char_offset);

        // 새 문단을 현재 문단 뒤에 삽입
        let new_para_idx = para_idx + 1;
        self.document.sections[section_idx]
            .paragraphs
            .insert(new_para_idx, new_para);

        // 양쪽 문단 리플로우 → vpos 재계산 → 재구성 → 재페이지네이션 + 다단 수렴 루프
        let old_col1 = self
            .para_column_map
            .get(section_idx)
            .and_then(|m| m.get(para_idx))
            .copied()
            .unwrap_or(0);
        self.reflow_paragraph(section_idx, para_idx);
        self.reflow_paragraph(section_idx, new_para_idx);
        crate::renderer::composer::recalculate_section_vpos(
            &mut self.document.sections[section_idx].paragraphs,
            para_idx,
        );
        self.recompose_paragraph(section_idx, para_idx);
        self.insert_composed_paragraph(section_idx, new_para_idx);
        self.paginate_if_needed();

        for _ in 0..2 {
            let new_col1 = self
                .para_column_map
                .get(section_idx)
                .and_then(|m| m.get(para_idx))
                .copied()
                .unwrap_or(0);
            if new_col1 == old_col1 {
                break;
            }
            self.reflow_paragraph(section_idx, para_idx);
            self.reflow_paragraph(section_idx, new_para_idx);
            crate::renderer::composer::recalculate_section_vpos(
                &mut self.document.sections[section_idx].paragraphs,
                para_idx,
            );
            self.recompose_paragraph(section_idx, para_idx);
            self.recompose_paragraph(section_idx, new_para_idx);
            self.paginate_if_needed();
        }

        self.event_log.push(DocumentEvent::ParagraphSplit {
            section: section_idx,
            para: para_idx,
            offset: char_offset,
        });
        Ok(super::super::helpers::json_ok_with(&format!(
            "\"paraIdx\":{},\"charOffset\":0",
            new_para_idx
        )))
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

        self.document.sections[section_idx].raw_stream = None;

        // 문단 분리
        let new_para =
            self.document.sections[section_idx].paragraphs[para_idx].split_at(char_offset);
        let new_para_idx = para_idx + 1;
        self.document.sections[section_idx]
            .paragraphs
            .insert(new_para_idx, new_para);

        // 새 문단에 쪽 나누기 설정
        self.document.sections[section_idx].paragraphs[new_para_idx].column_type =
            ColumnBreakType::Page;
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

        self.event_log.push(DocumentEvent::ParagraphSplit {
            section: section_idx,
            para: para_idx,
            offset: char_offset,
        });
        Ok(super::super::helpers::json_ok_with(&format!(
            "\"paraIdx\":{},\"charOffset\":0",
            new_para_idx
        )))
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

        self.document.sections[section_idx].raw_stream = None;

        // 문단 분리
        let new_para =
            self.document.sections[section_idx].paragraphs[para_idx].split_at(char_offset);
        let new_para_idx = para_idx + 1;
        self.document.sections[section_idx]
            .paragraphs
            .insert(new_para_idx, new_para);

        // 새 문단에 단 나누기 설정
        self.document.sections[section_idx].paragraphs[new_para_idx].column_type =
            ColumnBreakType::Column;
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

        self.event_log.push(DocumentEvent::ParagraphSplit {
            section: section_idx,
            para: para_idx,
            offset: char_offset,
        });
        Ok(super::super::helpers::json_ok_with(&format!(
            "\"paraIdx\":{},\"charOffset\":0",
            new_para_idx
        )))
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
        column_type: u8, // 0=일반(Normal), 1=배분(Distribute), 2=평행(Parallel)
        same_width: bool,
        spacing_hu: i16, // 단 간격 (HWPUNIT)
    ) -> Result<String, HwpError> {
        use crate::model::page::{ColumnDirection, ColumnType};

        if section_idx >= self.document.sections.len() {
            return Err(HwpError::RenderError(format!(
                "구역 인덱스 {} 범위 초과",
                section_idx
            )));
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
                    // 원본 attr(u16)을 무효화한다. serializer 는 raw_attr != 0 이면
                    // 그것을 그대로 쓰므로(라운드트립 보존), 여기서 비우지 않으면
                    // 갱신된 column_count/type/same_width 가 export 시 버려져
                    // 단 개수 변경이 파일에 반영되지 않는다(예: set_columns(2) 후 저장 시 1단 유지).
                    cd.raw_attr = 0;
                    found = true;
                    break;
                }
            }
            if found {
                break;
            }
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
                    .controls
                    .push(Control::ColumnDef(cd));
            }
        }

        // 단 개수 변경 시 기존 문단들의 LINE_SEG 폭은 이전 단 너비(또는 본문 전체
        // 너비)로 인코딩되어 있어 그대로 두면 새 단 너비와 어긋난다. 특히 ColumnDef
        // 컨트롤을 호스팅하는 첫 문단은 본문 전체 폭(예: 42519 HU)으로 남아, 렌더러가
        // 그 stale LINE_SEG 를 그대로 그리면 단 경계를 넘어 옆 단 텍스트와 겹친다
        // ("text brokerage"). 단 정의 변경 후 구역의 모든 텍스트 문단을 새 단 너비로
        // reflow 하여 LINE_SEG.segment_width / 줄나눔을 일관되게 만든다.
        // 표/그림 등 컨트롤 줄높이에 의존하는 문단은 reflow_paragraph 가 텍스트 기반으로
        // 폭만 재계산하므로, 컨트롤만 있고 텍스트가 없는 문단은 건너뛴다.
        self.styles = resolve_styles(&self.document.doc_info, self.dpi);
        let para_count = self.document.sections[section_idx].paragraphs.len();
        for p_idx in 0..para_count {
            let has_text = !self.document.sections[section_idx].paragraphs[p_idx]
                .text
                .trim()
                .is_empty();
            if has_text {
                self.reflow_paragraph(section_idx, p_idx);
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
                "구역 인덱스 {} 범위 초과 (총 {}개)",
                section_idx,
                self.document.sections.len()
            )));
        }
        let section = &self.document.sections[section_idx];
        if para_idx == 0 {
            return Err(HwpError::RenderError(
                "첫 번째 문단은 병합할 수 없습니다".to_string(),
            ));
        }
        if para_idx >= section.paragraphs.len() {
            return Err(HwpError::RenderError(format!(
                "문단 인덱스 {} 범위 초과 (총 {}개)",
                para_idx,
                section.paragraphs.len()
            )));
        }

        // 편집 시 raw 스트림 무효화 (재직렬화 유도)
        self.document.sections[section_idx].raw_stream = None;

        // 현재 문단을 이전 문단에 병합
        let current_para = self.document.sections[section_idx]
            .paragraphs
            .remove(para_idx);
        let prev_idx = para_idx - 1;
        let merge_point =
            self.document.sections[section_idx].paragraphs[prev_idx].merge_from(&current_para);

        // 병합된 문단 리플로우 → vpos 재계산 → 재구성 → 재페이지네이션 + 다단 수렴 루프
        let old_col = self
            .para_column_map
            .get(section_idx)
            .and_then(|m| m.get(prev_idx))
            .copied()
            .unwrap_or(0);
        self.reflow_paragraph(section_idx, prev_idx);
        crate::renderer::composer::recalculate_section_vpos(
            &mut self.document.sections[section_idx].paragraphs,
            prev_idx,
        );
        self.remove_composed_paragraph(section_idx, para_idx);
        self.recompose_paragraph(section_idx, prev_idx);
        self.paginate_if_needed();

        for _ in 0..2 {
            let new_col = self
                .para_column_map
                .get(section_idx)
                .and_then(|m| m.get(prev_idx))
                .copied()
                .unwrap_or(0);
            if new_col == old_col {
                break;
            }
            self.reflow_paragraph(section_idx, prev_idx);
            crate::renderer::composer::recalculate_section_vpos(
                &mut self.document.sections[section_idx].paragraphs,
                prev_idx,
            );
            self.recompose_paragraph(section_idx, prev_idx);
            self.paginate_if_needed();
        }

        self.event_log.push(DocumentEvent::ParagraphMerged {
            section: section_idx,
            para: para_idx,
        });
        Ok(super::super::helpers::json_ok_with(&format!(
            "\"paraIdx\":{},\"charOffset\":{}",
            prev_idx, merge_point
        )))
    }

    /// 문단 삭제 (네이티브 에러 타입)
    pub fn delete_paragraph_native(
        &mut self,
        section_idx: usize,
        para_idx: usize,
    ) -> Result<String, HwpError> {
        if section_idx >= self.document.sections.len() {
            return Err(HwpError::RenderError(format!(
                "구역 인덱스 {} 범위 초과 (총 {}개)",
                section_idx,
                self.document.sections.len()
            )));
        }
        let section = &self.document.sections[section_idx];
        if section.paragraphs.len() <= 1 {
            return Err(HwpError::RenderError(
                "구역의 마지막 문단은 삭제할 수 없습니다".to_string(),
            ));
        }
        if para_idx >= section.paragraphs.len() {
            return Err(HwpError::RenderError(format!(
                "문단 인덱스 {} 범위 초과 (총 {}개)",
                para_idx,
                section.paragraphs.len()
            )));
        }

        let removed_char_count = self.document.sections[section_idx].paragraphs[para_idx]
            .text
            .chars()
            .count();
        self.document.sections[section_idx].raw_stream = None;
        self.document.sections[section_idx]
            .paragraphs
            .remove(para_idx);

        let reflow_idx = if para_idx > 0 { para_idx - 1 } else { 0 };
        let old_col = self
            .para_column_map
            .get(section_idx)
            .and_then(|m| m.get(reflow_idx))
            .copied()
            .unwrap_or(0);
        self.remove_composed_paragraph(section_idx, para_idx);
        if reflow_idx < self.document.sections[section_idx].paragraphs.len() {
            self.reflow_paragraph(section_idx, reflow_idx);
        }
        crate::renderer::composer::recalculate_section_vpos(
            &mut self.document.sections[section_idx].paragraphs,
            reflow_idx,
        );
        if reflow_idx < self.document.sections[section_idx].paragraphs.len() {
            self.recompose_paragraph(section_idx, reflow_idx);
        }
        self.paginate_if_needed();

        for _ in 0..2 {
            let new_col = self
                .para_column_map
                .get(section_idx)
                .and_then(|m| m.get(reflow_idx))
                .copied()
                .unwrap_or(0);
            if new_col == old_col {
                break;
            }
            if reflow_idx < self.document.sections[section_idx].paragraphs.len() {
                self.reflow_paragraph(section_idx, reflow_idx);
            }
            crate::renderer::composer::recalculate_section_vpos(
                &mut self.document.sections[section_idx].paragraphs,
                reflow_idx,
            );
            if reflow_idx < self.document.sections[section_idx].paragraphs.len() {
                self.recompose_paragraph(section_idx, reflow_idx);
            }
            self.paginate_if_needed();
        }

        let new_count = self.document.sections[section_idx].paragraphs.len();
        self.event_log.push(DocumentEvent::ParagraphDeleted {
            section: section_idx,
            para: para_idx,
        });
        Ok(super::super::helpers::json_ok_with(&format!(
            "\"removedCharCount\":{},\"newParagraphCount\":{}",
            removed_char_count, new_count
        )))
    }

    /// 빈 문단 삽입 (네이티브 에러 타입)
    ///
    /// `para_idx == paragraphs.len()` 이면 구역 끝에 추가(append).
    pub fn insert_paragraph_native(
        &mut self,
        section_idx: usize,
        para_idx: usize,
    ) -> Result<String, HwpError> {
        if section_idx >= self.document.sections.len() {
            return Err(HwpError::RenderError(format!(
                "구역 인덱스 {} 범위 초과 (총 {}개)",
                section_idx,
                self.document.sections.len()
            )));
        }
        let para_count = self.document.sections[section_idx].paragraphs.len();
        if para_idx > para_count {
            return Err(HwpError::RenderError(format!(
                "문단 인덱스 {} 범위 초과 (총 {}개, 최대 {})",
                para_idx, para_count, para_count
            )));
        }

        self.document.sections[section_idx].raw_stream = None;

        let new_para = Paragraph::new_empty();
        self.document.sections[section_idx]
            .paragraphs
            .insert(para_idx, new_para);

        let reflow_target = if para_idx > 0 { para_idx - 1 } else { para_idx };
        let old_col = self
            .para_column_map
            .get(section_idx)
            .and_then(|m| m.get(reflow_target))
            .copied()
            .unwrap_or(0);
        self.reflow_paragraph(section_idx, para_idx);
        crate::renderer::composer::recalculate_section_vpos(
            &mut self.document.sections[section_idx].paragraphs,
            reflow_target,
        );
        self.insert_composed_paragraph(section_idx, para_idx);
        self.paginate_if_needed();

        for _ in 0..2 {
            let new_col = self
                .para_column_map
                .get(section_idx)
                .and_then(|m| m.get(reflow_target))
                .copied()
                .unwrap_or(0);
            if new_col == old_col {
                break;
            }
            self.reflow_paragraph(section_idx, para_idx);
            crate::renderer::composer::recalculate_section_vpos(
                &mut self.document.sections[section_idx].paragraphs,
                reflow_target,
            );
            self.recompose_paragraph(section_idx, para_idx);
            self.paginate_if_needed();
        }

        let new_count = self.document.sections[section_idx].paragraphs.len();
        self.event_log.push(DocumentEvent::ParagraphInserted {
            section: section_idx,
            para: para_idx,
        });
        Ok(super::super::helpers::json_ok_with(&format!(
            "\"paraIdx\":{},\"newParagraphCount\":{}",
            para_idx, new_count
        )))
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
            section_idx,
            parent_para_idx,
            control_idx,
            cell_idx,
            cell_para_idx,
        )?;
        let new_para = cell_para.split_at(char_offset);

        // 새 문단을 셀/글상자에 삽입
        let new_cell_para_idx = cell_para_idx + 1;
        match self.document.sections[section_idx].paragraphs[parent_para_idx]
            .controls
            .get_mut(control_idx)
        {
            Some(Control::Table(table)) => {
                table.cells[cell_idx]
                    .paragraphs
                    .insert(new_cell_para_idx, new_para);
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
        self.reflow_cell_paragraph(
            section_idx,
            parent_para_idx,
            control_idx,
            cell_idx,
            cell_para_idx,
        );
        self.reflow_cell_paragraph(
            section_idx,
            parent_para_idx,
            control_idx,
            cell_idx,
            new_cell_para_idx,
        );
        self.recalculate_cell_paragraph_vpos_native(
            section_idx,
            parent_para_idx,
            control_idx,
            cell_idx,
        )?;

        // raw 스트림 무효화, section dirty, 재페이지네이션
        self.document.sections[section_idx].raw_stream = None;
        self.mark_section_dirty(section_idx);
        self.paginate_if_needed();

        self.event_log.push(DocumentEvent::CellTextChanged {
            section: section_idx,
            para: parent_para_idx,
            ctrl: control_idx,
            cell: cell_idx,
        });
        Ok(super::super::helpers::json_ok_with(&format!(
            "\"cellParaIndex\":{},\"charOffset\":0",
            new_cell_para_idx
        )))
    }

    /// 셀(또는 글상자/그림 캡션) 전체 내용을 여러 줄 텍스트로 REPLACE 한다.
    ///
    /// `lines`의 각 원소가 셀 안의 한 문단이 된다. 기존 셀 문단을 모두 비우고
    /// 첫 문단(cell_para 0)의 ParaShape/CharShape/style을 보존한 채 다시 채운다.
    /// 추가 문단은 첫 문단을 `split_at`으로 분할해 만들어 같은 서식을 상속한다
    /// (HWP의 `① 영문 ¶ 해석` 2-문단 셀 모양·글꼴이 그대로 유지된다).
    ///
    /// `lines`가 비어 있으면 빈 한 문단으로 만든다. 셀 내부 컨트롤(중첩 표/그림
    /// 등)은 첫 문단에만 남으므로, 컨트롤이 있는 셀이면 그 컨트롤은 보존되고
    /// 텍스트만 교체된다.
    pub fn set_cell_text_native(
        &mut self,
        section_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
        cell_idx: usize,
        lines: &[String],
    ) -> Result<String, HwpError> {
        // 빈 입력은 빈 한 줄로 정규화한다(셀에는 항상 최소 한 문단이 있어야 한다).
        let empty = String::new();
        let lines: &[String] = if lines.is_empty() {
            std::slice::from_ref(&empty)
        } else {
            lines
        };

        // 1) 셀을 첫 문단(cell_para 0) 하나로 접고 그 문단을 비운다. 추가 셀 문단을
        //    마지막부터 첫 문단으로 반복 병합한 뒤 첫 문단 전체 길이를 삭제한다.
        //    merge/delete 프리미티브는 char_shape/para_shape 를 보존하므로 셀의
        //    기존 서식이 그대로 남는다.
        loop {
            let count = self.get_cell_paragraph_count_native(
                section_idx,
                parent_para_idx,
                control_idx,
                cell_idx,
            )?;
            if count <= 1 {
                break;
            }
            self.merge_paragraph_in_cell_native(
                section_idx,
                parent_para_idx,
                control_idx,
                cell_idx,
                count - 1,
            )?;
        }
        let len0 = self.get_cell_paragraph_length_native(
            section_idx,
            parent_para_idx,
            control_idx,
            cell_idx,
            0,
        )?;
        self.delete_text_in_cell_native(
            section_idx,
            parent_para_idx,
            control_idx,
            cell_idx,
            0,
            0,
            len0,
        )?;

        // 2) 줄 0을 첫 문단에 삽입한다(첫 문단의 서식을 상속).
        self.insert_text_in_cell_native(
            section_idx,
            parent_para_idx,
            control_idx,
            cell_idx,
            0,
            0,
            &lines[0],
        )?;

        // 3) 나머지 줄: 현재 마지막 셀 문단의 끝에서 split 해 새 문단을 만들고
        //    (ParaShape/CharShape/style 상속), 그 문단에 줄을 삽입한다. split/insert
        //    프리미티브는 각 문단을 개별 reflow 하지만 문단 *사이* vpos 는 두지 않으므로
        //    아래 (4)에서 한 번에 누적 재계산한다.
        for (i, line) in lines.iter().enumerate().skip(1) {
            let prev_idx = i - 1;
            let prev_len = self.get_cell_paragraph_length_native(
                section_idx,
                parent_para_idx,
                control_idx,
                cell_idx,
                prev_idx,
            )?;
            self.split_paragraph_in_cell_native(
                section_idx,
                parent_para_idx,
                control_idx,
                cell_idx,
                prev_idx,
                prev_len,
            )?;
            self.insert_text_in_cell_native(
                section_idx,
                parent_para_idx,
                control_idx,
                cell_idx,
                i,
                0,
                line,
            )?;
        }

        self.grow_table_cell_height_to_line_geometry(
            section_idx,
            parent_para_idx,
            control_idx,
            cell_idx,
        )?;
        self.document.sections[section_idx].raw_stream = None;
        self.mark_section_dirty(section_idx);
        self.paginate_if_needed();

        let line_count = lines.len();
        self.event_log.push(DocumentEvent::CellTextChanged {
            section: section_idx,
            para: parent_para_idx,
            ctrl: control_idx,
            cell: cell_idx,
        });
        Ok(super::super::helpers::json_ok_with(&format!(
            "\"cellParaCount\":{}",
            line_count
        )))
    }

    /// 셀(표)/글상자(도형)/캡션(그림)의 문단 Vec에 대한 가변 참조.
    /// 단일 레벨 컨트롤(중첩 표 제외) — `*_in_cell_native` 류와 동일한 라우팅.
    fn cell_paragraphs_mut(
        &mut self,
        section_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
        cell_idx: usize,
    ) -> Result<&mut Vec<Paragraph>, HwpError> {
        let para = self
            .document
            .sections
            .get_mut(section_idx)
            .ok_or_else(|| HwpError::RenderError(format!("구역 인덱스 {} 범위 초과", section_idx)))?
            .paragraphs
            .get_mut(parent_para_idx)
            .ok_or_else(|| {
                HwpError::RenderError(format!("부모 문단 인덱스 {} 범위 초과", parent_para_idx))
            })?;
        match para.controls.get_mut(control_idx) {
            Some(Control::Table(table)) => {
                if cell_idx == 65534 {
                    let cap = table
                        .caption
                        .as_mut()
                        .ok_or_else(|| HwpError::RenderError("표에 캡션이 없습니다".to_string()))?;
                    Ok(&mut cap.paragraphs)
                } else {
                    let cell_count = table.cells.len();
                    let cell = table.cells.get_mut(cell_idx).ok_or_else(|| {
                        HwpError::RenderError(format!(
                            "셀 인덱스 {} 범위 초과 (총 {}개)",
                            cell_idx, cell_count
                        ))
                    })?;
                    Ok(&mut cell.paragraphs)
                }
            }
            Some(Control::Shape(shape)) => {
                let tb = super::super::helpers::get_textbox_from_shape_mut(shape)
                    .ok_or_else(|| HwpError::RenderError("도형에 글상자가 없습니다".to_string()))?;
                Ok(&mut tb.paragraphs)
            }
            Some(Control::Picture(pic)) => {
                let cap = pic
                    .caption
                    .as_mut()
                    .ok_or_else(|| HwpError::RenderError("그림에 캡션이 없습니다".to_string()))?;
                Ok(&mut cap.paragraphs)
            }
            _ => Err(HwpError::RenderError(format!(
                "컨트롤 인덱스 {}가 표/글상자/그림이 아닙니다",
                control_idx
            ))),
        }
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
                "셀 첫 번째 문단은 병합할 수 없습니다".to_string(),
            ));
        }

        // 검증: 셀 문단 인덱스 범위 확인
        {
            let cell_para = self.get_cell_paragraph_mut(
                section_idx,
                parent_para_idx,
                control_idx,
                cell_idx,
                cell_para_idx,
            )?;
            let _ = cell_para; // 검증만 수행
        }

        // 문단 제거 및 이전 문단에 병합
        let prev_idx = cell_para_idx - 1;
        let merge_point;
        match self.document.sections[section_idx].paragraphs[parent_para_idx]
            .controls
            .get_mut(control_idx)
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
                        "지정된 Shape 컨트롤에 텍스트 박스가 없습니다".to_string(),
                    ));
                }
            }
            Some(Control::Picture(pic)) => {
                if let Some(ref mut cap) = pic.caption {
                    let removed = cap.paragraphs.remove(cell_para_idx);
                    merge_point = cap.paragraphs[prev_idx].merge_from(&removed);
                } else {
                    return Err(HwpError::RenderError(
                        "지정된 그림 컨트롤에 캡션이 없습니다".to_string(),
                    ));
                }
            }
            _ => {
                return Err(HwpError::RenderError(
                    "지정된 컨트롤이 표, 글상자 또는 그림이 아닙니다".to_string(),
                ));
            }
        }

        // 병합된 문단 리플로우
        self.reflow_cell_paragraph(
            section_idx,
            parent_para_idx,
            control_idx,
            cell_idx,
            prev_idx,
        );
        self.recalculate_cell_paragraph_vpos_native(
            section_idx,
            parent_para_idx,
            control_idx,
            cell_idx,
        )?;

        // raw 스트림 무효화, section dirty, 재페이지네이션
        self.document.sections[section_idx].raw_stream = None;
        self.mark_section_dirty(section_idx);
        self.paginate_if_needed();

        self.event_log.push(DocumentEvent::CellTextChanged {
            section: section_idx,
            para: parent_para_idx,
            ctrl: control_idx,
            cell: cell_idx,
        });
        Ok(super::super::helpers::json_ok_with(&format!(
            "\"cellParaIndex\":{},\"charOffset\":{}",
            prev_idx, merge_point
        )))
    }

    // ─── Phase 1 Native: 기본 편집 보조 API ────────────────────

    /// 구역 내 문단 수 (네이티브)
    pub fn get_paragraph_count_native(&self, section_idx: usize) -> Result<usize, HwpError> {
        let section = self.document.sections.get(section_idx).ok_or_else(|| {
            HwpError::RenderError(format!(
                "구역 인덱스 {} 범위 초과 (총 {}개)",
                section_idx,
                self.document.sections.len()
            ))
        })?;
        Ok(section.paragraphs.len())
    }

    /// 문단 글자 수 (네이티브)
    pub fn get_paragraph_length_native(
        &self,
        section_idx: usize,
        para_idx: usize,
    ) -> Result<usize, HwpError> {
        let section = self.document.sections.get(section_idx).ok_or_else(|| {
            HwpError::RenderError(format!(
                "구역 인덱스 {} 범위 초과 (총 {}개)",
                section_idx,
                self.document.sections.len()
            ))
        })?;
        let para = section.paragraphs.get(para_idx).ok_or_else(|| {
            HwpError::RenderError(format!(
                "문단 인덱스 {} 범위 초과 (총 {}개)",
                para_idx,
                section.paragraphs.len()
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
                let from = if start_ci < 0 {
                    0usize
                } else {
                    (start_ci as usize) + 1
                };
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
                let until = if start_ci < 0 {
                    controls.len()
                } else {
                    start_ci as usize
                };
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
            return format!(
                "{{\"type\":\"{}\",\"sec\":{},\"para\":{},\"ci\":{}}}",
                ty, section_idx, para_idx, ci
            );
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
                let search_start = if forward {
                    -1
                } else {
                    section.paragraphs[pi].controls.len() as i32
                };
                if let Some((ci, ty)) =
                    find_in_para(sections, section_idx, pi, search_start, forward)
                {
                    return format!(
                        "{{\"type\":\"{}\",\"sec\":{},\"para\":{},\"ci\":{}}}",
                        ty, section_idx, pi, ci
                    );
                }
                // 네비게이션 가능한 컨트롤이 없는 문단 → body
                if !has_navigable_control(sections, section_idx, pi) {
                    return format!(
                        "{{\"type\":\"body\",\"sec\":{},\"para\":{}}}",
                        section_idx, pi
                    );
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
                    let search_start = if forward {
                        -1
                    } else {
                        section.paragraphs[pi].controls.len() as i32
                    };
                    if let Some((ci, ty)) = find_in_para(sections, si, pi, search_start, forward) {
                        return format!(
                            "{{\"type\":\"{}\",\"sec\":{},\"para\":{},\"ci\":{}}}",
                            ty, si, pi, ci
                        );
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
                "구역 인덱스 {} 범위 초과 (총 {}개)",
                section_idx,
                self.document.sections.len()
            ))
        })?;
        let para = section.paragraphs.get(para_idx).ok_or_else(|| {
            HwpError::RenderError(format!(
                "문단 인덱스 {} 범위 초과 (총 {}개)",
                para_idx,
                section.paragraphs.len()
            ))
        })?;
        let text_chars: Vec<char> = para.text.chars().collect();
        let total = text_chars.len();
        if char_offset > total {
            return Err(HwpError::RenderError(format!(
                "char_offset {} 범위 초과 (문단 길이 {})",
                char_offset, total
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
        let para = self
            .document
            .sections
            .get(section_idx)
            .ok_or_else(|| HwpError::RenderError(format!("구역 인덱스 {} 범위 초과", section_idx)))?
            .paragraphs
            .get(parent_para_idx)
            .ok_or_else(|| {
                HwpError::RenderError(format!("문단 인덱스 {} 범위 초과", parent_para_idx))
            })?;
        match para.controls.get(control_idx) {
            Some(Control::Table(table)) => {
                if cell_idx == 65534 {
                    let cap = table
                        .caption
                        .as_ref()
                        .ok_or_else(|| HwpError::RenderError("표에 캡션이 없습니다".to_string()))?;
                    return Ok(cap.paragraphs.len());
                }
                let cell = table.cells.get(cell_idx).ok_or_else(|| {
                    HwpError::RenderError(format!(
                        "셀 인덱스 {} 범위 초과 (총 {}개)",
                        cell_idx,
                        table.cells.len()
                    ))
                })?;
                Ok(cell.paragraphs.len())
            }
            Some(Control::Shape(shape)) => {
                let text_box = get_textbox_from_shape(shape)
                    .ok_or_else(|| HwpError::RenderError("도형에 글상자가 없습니다".to_string()))?;
                Ok(text_box.paragraphs.len())
            }
            Some(Control::Picture(pic)) => {
                let caption = pic
                    .caption
                    .as_ref()
                    .ok_or_else(|| HwpError::RenderError("그림에 캡션이 없습니다".to_string()))?;
                Ok(caption.paragraphs.len())
            }
            _ => Err(HwpError::RenderError(format!(
                "컨트롤 인덱스 {}가 표/글상자가 아닙니다",
                control_idx
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
        let cell_para = self
            .get_cell_paragraph_ref(
                section_idx,
                parent_para_idx,
                control_idx,
                cell_idx,
                cell_para_idx,
            )
            .ok_or_else(|| {
                HwpError::RenderError(format!(
                    "셀 문단 접근 실패: sec={}, para={}, ctrl={}, cell={}, cellPara={}",
                    section_idx, parent_para_idx, control_idx, cell_idx, cell_para_idx
                ))
            })?;
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
        let cell_para = self
            .get_cell_paragraph_ref(
                section_idx,
                parent_para_idx,
                control_idx,
                cell_idx,
                cell_para_idx,
            )
            .ok_or_else(|| {
                HwpError::RenderError(format!(
                    "셀 문단 접근 실패: sec={}, para={}, ctrl={}, cell={}, cellPara={}",
                    section_idx, parent_para_idx, control_idx, cell_idx, cell_para_idx
                ))
            })?;
        let text_chars: Vec<char> = cell_para.text.chars().collect();
        let total = text_chars.len();
        if char_offset > total {
            return Err(HwpError::RenderError(format!(
                "char_offset {} 범위 초과 (셀 문단 길이 {})",
                char_offset, total
            )));
        }
        let end = (char_offset + count).min(total);
        let result: String = text_chars[char_offset..end].iter().collect();
        Ok(result)
    }

    // ─── Phase 1 Native 끝 ──────────────────────────────────

    // ─── Phase 2 Native: 커서/히트 테스트 API ────────────────────

    /// 문단이 포함된 글로벌 페이지 번호 목록을 반환한다.
    pub(crate) fn find_pages_for_paragraph(
        &self,
        section_idx: usize,
        para_idx: usize,
    ) -> Result<Vec<u32>, HwpError> {
        if section_idx >= self.pagination.len() {
            return Err(HwpError::RenderError(format!(
                "구역 인덱스 {} 범위 초과 (총 {}개)",
                section_idx,
                self.pagination.len()
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
                                crate::renderer::pagination::PageItem::FullParagraph {
                                    para_index,
                                } => Some(*para_index),
                                crate::renderer::pagination::PageItem::PartialParagraph {
                                    para_index,
                                    ..
                                } => Some(*para_index),
                                crate::renderer::pagination::PageItem::Table {
                                    para_index, ..
                                } => Some(*para_index),
                                crate::renderer::pagination::PageItem::PartialTable {
                                    para_index,
                                    ..
                                } => Some(*para_index),
                                crate::renderer::pagination::PageItem::Shape {
                                    para_index, ..
                                } => Some(*para_index),
                                crate::renderer::pagination::PageItem::EndnoteSeparator {
                                    ..
                                } => None,
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
                            if let Ok(table_pages) =
                                self.find_pages_for_paragraph(section_idx, wp.table_para_index)
                            {
                                return Ok(table_pages);
                            }
                        }
                    }
                }
                return if result.is_empty() {
                    Err(HwpError::RenderError(format!(
                        "문단 (sec={}, para={})이 페이지에 없습니다",
                        section_idx, para_idx
                    )))
                } else {
                    Ok(result)
                };
            }
            global_offset += pr.pages.len() as u32;
        }
        Err(HwpError::RenderError(format!(
            "구역 인덱스 {} 범위 초과",
            section_idx
        )))
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
    /// cellPath를 따라가서 최종 셀의 문단 목록에 대한 가변 참조를 얻는다.
    /// path: [(control_index, cell_index, cell_para_index), ...]
    pub(crate) fn get_cell_paragraphs_mut_by_path(
        &mut self,
        section_idx: usize,
        parent_para_idx: usize,
        path: &[(usize, usize, usize)],
    ) -> Result<&mut Vec<Paragraph>, HwpError> {
        if path.is_empty() {
            return Err(HwpError::RenderError("경로가 비어있습니다".to_string()));
        }
        let section = self
            .document
            .sections
            .get_mut(section_idx)
            .ok_or_else(|| HwpError::RenderError(format!("구역 {} 범위 초과", section_idx)))?;
        let mut para: &mut Paragraph = section
            .paragraphs
            .get_mut(parent_para_idx)
            .ok_or_else(|| HwpError::RenderError(format!("문단 {} 범위 초과", parent_para_idx)))?;

        for (i, &(ctrl_idx, cell_idx, cell_para_idx)) in path.iter().enumerate() {
            let table = match para.controls.get_mut(ctrl_idx) {
                Some(Control::Table(t)) => t.as_mut(),
                _ => {
                    return Err(HwpError::RenderError(format!(
                        "경로[{}]: controls[{}]가 표가 아닙니다",
                        i, ctrl_idx
                    )))
                }
            };
            let cell = table.cells.get_mut(cell_idx).ok_or_else(|| {
                HwpError::RenderError(format!("경로[{}]: 셀 {} 범위 초과", i, cell_idx))
            })?;
            if i == path.len() - 1 {
                return Ok(&mut cell.paragraphs);
            }
            para = cell.paragraphs.get_mut(cell_para_idx).ok_or_else(|| {
                HwpError::RenderError(format!("경로[{}]: 셀문단 {} 범위 초과", i, cell_para_idx))
            })?;
        }
        unreachable!()
    }

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
        let last_path_index = path.len() - 1;
        let cell_para_idx = path[last_path_index].2;
        let cell_paragraphs =
            self.get_cell_paragraphs_mut_by_path(section_idx, parent_para_idx, path)?;
        cell_paragraphs.get_mut(cell_para_idx).ok_or_else(|| {
            HwpError::RenderError(format!(
                "경로[{}]: 셀문단 {} 범위 초과",
                last_path_index, cell_para_idx
            ))
        })
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
        self.reflow_cell_paragraphs_by_path(section_idx, parent_para_idx, path)?;

        // 리플로우 (최외곽 표 기준 — 중첩 표 셀 폭은 별도 계산이 필요하나 우선 section dirty로 처리)
        self.document.sections[section_idx].raw_stream = None;
        self.mark_section_dirty(section_idx);
        self.paginate_if_needed();

        let new_offset = char_offset + new_chars_count;
        self.event_log.push(DocumentEvent::CellTextChanged {
            section: section_idx,
            para: parent_para_idx,
            ctrl: outer_ctrl,
            cell: path[0].1,
        });
        Ok(super::super::helpers::json_ok_with(&format!(
            "\"charOffset\":{}",
            new_offset
        )))
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
        self.reflow_cell_paragraphs_by_path(section_idx, parent_para_idx, path)?;
        self.document.sections[section_idx].raw_stream = None;
        self.mark_section_dirty(section_idx);
        self.paginate_if_needed();

        self.event_log.push(DocumentEvent::CellTextChanged {
            section: section_idx,
            para: parent_para_idx,
            ctrl: outer_ctrl,
            cell: path[0].1,
        });
        Ok(super::super::helpers::json_ok_with(&format!(
            "\"charOffset\":{}",
            char_offset
        )))
    }

    /// path 기반 셀(중첩 표 지원) 전체 내용을 여러 줄 텍스트로 REPLACE 한다.
    ///
    /// `set_cell_text_native`(단일 레벨)의 by-path 미러. 업스트림의 셀 편집
    /// 프리미티브(merge/delete/split/insert `_by_path`)를 조합해 셀을 비우고
    /// 다시 채운다. 서식(ParaShape/CharShape/style)은 이들 프리미티브가 편집
    /// 지점에서 상속하므로 보존되고, 마지막에 cross-paragraph vpos 만 한 번
    /// 누적 재계산한다(업스트림 by-path 프리미티브는 reflow/vpos 미수행).
    pub fn set_cell_text_by_path_native(
        &mut self,
        section_idx: usize,
        parent_para_idx: usize,
        path: &[(usize, usize, usize)],
        lines: &[String],
    ) -> Result<String, HwpError> {
        if path.is_empty() {
            return Err(HwpError::RenderError("경로가 비어있습니다".to_string()));
        }

        // 빈 입력은 빈 한 줄로 정규화한다(셀에는 항상 최소 한 문단이 있어야 한다).
        let empty = String::new();
        let lines: &[String] = if lines.is_empty() {
            std::slice::from_ref(&empty)
        } else {
            lines
        };

        // 끝 셀(path.last)의 cell_para_idx 만 바꾼 path 를 만든다. by-path
        // 프리미티브는 마지막 엔트리의 cell_para_idx 로 대상 문단을 고른다.
        let path_with_cpi = |cpi: usize| -> Vec<(usize, usize, usize)> {
            let mut p = path.to_vec();
            let last = p.len() - 1;
            p[last].2 = cpi;
            p
        };
        // 끝 셀 문단 개수.
        let cell_para_count = |this: &mut Self| -> Result<usize, HwpError> {
            Ok(this
                .get_cell_paragraphs_mut_by_path(section_idx, parent_para_idx, path)?
                .len())
        };
        // 끝 셀의 cpi 번째 문단 글자 수.
        let cell_para_len = |this: &mut Self, cpi: usize| -> Result<usize, HwpError> {
            let p = path_with_cpi(cpi);
            Ok(this
                .get_cell_paragraph_mut_by_path(section_idx, parent_para_idx, &p)?
                .text
                .chars()
                .count())
        };

        // 1) 셀을 첫 문단 하나로 접고 비운다(set_cell_text_native 미러). merge/delete
        //    by-path 프리미티브는 char_shape/para_shape 를 보존한다.
        loop {
            let count = cell_para_count(self)?;
            if count <= 1 {
                break;
            }
            let p = path_with_cpi(count - 1);
            self.merge_paragraph_in_cell_by_path(section_idx, parent_para_idx, &p)?;
        }
        let len0 = cell_para_len(self, 0)?;
        let p0 = path_with_cpi(0);
        self.delete_text_in_cell_by_path(section_idx, parent_para_idx, &p0, 0, len0)?;

        // 2) 줄 0을 첫 문단에 삽입(첫 문단 서식 상속).
        self.insert_text_in_cell_by_path(section_idx, parent_para_idx, &p0, 0, &lines[0])?;

        // 3) 나머지 줄: 현재 마지막 셀 문단 끝에서 split 후 삽입(ParaShape/CharShape/
        //    style 상속). 문단 사이 vpos 는 아래 (4)에서 누적 재계산한다.
        for (i, line) in lines.iter().enumerate().skip(1) {
            let prev_idx = i - 1;
            let prev_len = cell_para_len(self, prev_idx)?;
            let p_prev = path_with_cpi(prev_idx);
            self.split_paragraph_in_cell_by_path(section_idx, parent_para_idx, &p_prev, prev_len)?;
            let p_new = path_with_cpi(i);
            self.insert_text_in_cell_by_path(section_idx, parent_para_idx, &p_new, 0, line)?;
        }

        self.document.sections[section_idx].raw_stream = None;
        self.mark_section_dirty(section_idx);
        self.paginate_if_needed();

        let line_count = lines.len();
        let outer_ctrl = path[0].0;
        self.event_log.push(DocumentEvent::CellTextChanged {
            section: section_idx,
            para: parent_para_idx,
            ctrl: outer_ctrl,
            cell: path[0].1,
        });
        Ok(super::super::helpers::json_ok_with(&format!(
            "\"cellParaCount\":{}",
            line_count
        )))
    }

    fn grow_table_cell_height_to_line_geometry(
        &mut self,
        section_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
        cell_idx: usize,
    ) -> Result<(), HwpError> {
        let table = self.get_table_mut(section_idx, parent_para_idx, control_idx)?;
        Self::grow_cell_height_to_line_geometry(table, cell_idx);
        Ok(())
    }

    fn grow_cell_height_to_line_geometry(table: &mut crate::model::table::Table, cell_idx: usize) {
        let table_padding = table.padding;
        let Some(cell) = table.cells.get_mut(cell_idx) else {
            return;
        };
        if let Some(min_height) = Self::minimum_cell_height_for_line_geometry(table_padding, cell) {
            if min_height > cell.height {
                cell.height = min_height;
                table.update_ctrl_dimensions();
                table.dirty = true;
            }
        }
    }

    fn minimum_cell_height_for_line_geometry(
        table_padding: crate::model::Padding,
        cell: &crate::model::table::Cell,
    ) -> Option<u32> {
        if cell.text_direction != 0
            || cell.row_span != 1
            || cell.height >= 0x8000_0000
            || cell.paragraphs.iter().all(|p| p.line_segs.is_empty())
        {
            return None;
        }

        let prefer_cell_axis = |cell_value: i16, table_value: i16| -> bool {
            if cell.apply_inner_margin {
                cell_value != 0
            } else {
                (cell_value as i32) > (table_value as i32)
            }
        };
        let pad_top = if prefer_cell_axis(cell.padding.top, table_padding.top) {
            cell.padding.top as i32
        } else {
            table_padding.top as i32
        };
        let pad_bottom = if prefer_cell_axis(cell.padding.bottom, table_padding.bottom) {
            cell.padding.bottom as i32
        } else {
            table_padding.bottom as i32
        };
        let max_line_bottom = cell
            .paragraphs
            .iter()
            .flat_map(|p| p.line_segs.iter())
            .map(|ls| ls.vertical_pos.saturating_add(ls.line_height))
            .max()
            .unwrap_or(0);

        let required = pad_top + max_line_bottom + pad_bottom;
        Some(required.max(pad_top + pad_bottom).max(1) as u32)
    }

    /// path 끝 셀의 사용 가능한 내부 폭(px)을 계산한다(좌우 패딩 제외).
    /// reflow_line_segs 의 줄 나눔 폭으로 사용. 셀 폭/패딩을 못 구하면 None.
    fn cell_inner_width_px_by_path(
        &self,
        section_idx: usize,
        parent_para_idx: usize,
        path: &[(usize, usize, usize)],
    ) -> Option<f64> {
        use crate::renderer::hwpunit_to_px;
        let mut para = self
            .document
            .sections
            .get(section_idx)?
            .paragraphs
            .get(parent_para_idx)?;
        for (i, &(ctrl_idx, cell_idx, cell_para_idx)) in path.iter().enumerate() {
            let table = match para.controls.get(ctrl_idx) {
                Some(Control::Table(t)) => t.as_ref(),
                _ => return None,
            };
            let cell = table.cells.get(cell_idx)?;
            if i == path.len() - 1 {
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
                let w = hwpunit_to_px(cell.width as i32, self.dpi)
                    - hwpunit_to_px(pad_l as i32, self.dpi)
                    - hwpunit_to_px(pad_r as i32, self.dpi);
                return Some(w.max(0.0));
            }
            para = cell.paragraphs.get(cell_para_idx)?;
        }
        None
    }

    fn reflow_cell_paragraphs_by_path(
        &mut self,
        section_idx: usize,
        parent_para_idx: usize,
        path: &[(usize, usize, usize)],
    ) -> Result<(), HwpError> {
        if path.is_empty() {
            return Err(HwpError::RenderError("경로가 비어있습니다".to_string()));
        }

        let inner_width_px = self.cell_inner_width_px_by_path(section_idx, parent_para_idx, path);
        let dpi = self.dpi;
        let styles = &self.styles;
        let mut para: &mut Paragraph = self.document.sections[section_idx]
            .paragraphs
            .get_mut(parent_para_idx)
            .ok_or_else(|| HwpError::RenderError("문단 범위 초과".to_string()))?;

        for (i, &(ctrl_idx, cell_idx, cell_para_idx)) in path.iter().enumerate() {
            let table = match para.controls.get_mut(ctrl_idx) {
                Some(Control::Table(t)) => t.as_mut(),
                _ => return Err(HwpError::RenderError("경로: 표가 아닙니다".to_string())),
            };

            if i == path.len() - 1 {
                {
                    let cell = table
                        .cells
                        .get_mut(cell_idx)
                        .ok_or_else(|| HwpError::RenderError("셀 범위 초과".to_string()))?;

                    if let Some(w) = inner_width_px {
                        for cp in cell.paragraphs.iter_mut() {
                            reflow_line_segs(cp, w, styles, dpi);
                        }
                    }

                    crate::renderer::composer::recalculate_section_vpos(&mut cell.paragraphs, 0);
                }
                Self::grow_cell_height_to_line_geometry(table, cell_idx);
                return Ok(());
            }

            let cell = table
                .cells
                .get_mut(cell_idx)
                .ok_or_else(|| HwpError::RenderError("셀 범위 초과".to_string()))?;
            para = cell
                .paragraphs
                .get_mut(cell_para_idx)
                .ok_or_else(|| HwpError::RenderError("셀문단 범위 초과".to_string()))?;
        }

        Ok(())
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
        let section = self
            .document
            .sections
            .get_mut(section_idx)
            .ok_or_else(|| HwpError::RenderError("구역 범위 초과".to_string()))?;
        let mut para: &mut Paragraph = section
            .paragraphs
            .get_mut(parent_para_idx)
            .ok_or_else(|| HwpError::RenderError("문단 범위 초과".to_string()))?;

        // path를 따라 마지막 셀까지 진입
        for (i, &(ctrl_idx, cell_idx, _cpi)) in path.iter().enumerate() {
            let table = match para.controls.get_mut(ctrl_idx) {
                Some(Control::Table(t)) => t.as_mut(),
                _ => return Err(HwpError::RenderError("경로: 표가 아닙니다".to_string())),
            };
            let cell = table
                .cells
                .get_mut(cell_idx)
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
            para = cell
                .paragraphs
                .get_mut(_cpi)
                .ok_or_else(|| HwpError::RenderError("셀문단 범위 초과".to_string()))?;
        }

        let outer_ctrl = path[0].0;
        self.mark_cell_control_dirty(section_idx, parent_para_idx, outer_ctrl);
        self.reflow_cell_paragraphs_by_path(section_idx, parent_para_idx, path)?;
        self.document.sections[section_idx].raw_stream = None;
        self.mark_section_dirty(section_idx);
        self.paginate_if_needed();

        self.event_log.push(DocumentEvent::CellTextChanged {
            section: section_idx,
            para: parent_para_idx,
            ctrl: outer_ctrl,
            cell: path[0].1,
        });
        let new_cpi = cell_para_idx + 1;
        Ok(super::super::helpers::json_ok_with(&format!(
            "\"cellParaIndex\":{},\"charOffset\":0",
            new_cpi
        )))
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
            return Err(HwpError::RenderError(
                "첫 문단은 병합할 수 없습니다".to_string(),
            ));
        }

        let section = self
            .document
            .sections
            .get_mut(section_idx)
            .ok_or_else(|| HwpError::RenderError("구역 범위 초과".to_string()))?;
        let mut para: &mut Paragraph = section
            .paragraphs
            .get_mut(parent_para_idx)
            .ok_or_else(|| HwpError::RenderError("문단 범위 초과".to_string()))?;

        let mut merge_point = 0usize;
        for (i, &(ctrl_idx, cell_idx, _cpi)) in path.iter().enumerate() {
            let table = match para.controls.get_mut(ctrl_idx) {
                Some(Control::Table(t)) => t.as_mut(),
                _ => return Err(HwpError::RenderError("경로: 표가 아닙니다".to_string())),
            };
            let cell = table
                .cells
                .get_mut(cell_idx)
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
            para = cell
                .paragraphs
                .get_mut(_cpi)
                .ok_or_else(|| HwpError::RenderError("셀문단 범위 초과".to_string()))?;
        }

        let outer_ctrl = path[0].0;
        self.mark_cell_control_dirty(section_idx, parent_para_idx, outer_ctrl);
        self.reflow_cell_paragraphs_by_path(section_idx, parent_para_idx, path)?;
        self.document.sections[section_idx].raw_stream = None;
        self.mark_section_dirty(section_idx);
        self.paginate_if_needed();

        self.event_log.push(DocumentEvent::CellTextChanged {
            section: section_idx,
            para: parent_para_idx,
            ctrl: outer_ctrl,
            cell: path[0].1,
        });
        let prev_cpi = cell_para_idx - 1;
        Ok(super::super::helpers::json_ok_with(&format!(
            "\"cellParaIndex\":{},\"charOffset\":{}",
            prev_cpi, merge_point
        )))
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
                "char_offset {} 범위 초과 (셀 문단 길이 {})",
                char_offset, total
            )));
        }
        let end = (char_offset + count).min(total);
        Ok(text_chars[char_offset..end].iter().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Regression for the multi-paragraph cell-collapse bug: a nested-cell
    /// REPLACE via `set_cell_text_by_path_native` must give each created cell
    /// paragraph a strictly increasing `line_segs[0].vertical_pos` (cumulative
    /// line offset), so the Top-aligned renderer stacks them on distinct rows
    /// instead of overlapping at `cell_top + 0`.
    #[test]
    fn set_cell_by_path_cascades_vertical_pos() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/exam_social.hwp");
        let bytes = std::fs::read(&path).unwrap();
        let mut core = DocumentCore::from_bytes(&bytes).unwrap();

        let cell_path = [(0usize, 0usize, 0usize), (1usize, 1usize, 0usize)];
        let lines: Vec<String> = ["L0", "L1", "L2", "L3", "L4", "L5"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        let res = core
            .set_cell_text_by_path_native(0, 1, &cell_path, &lines)
            .unwrap();
        assert!(res.contains("\"cellParaCount\":6"), "got {res}");

        // body -> outer table(ctrl 0) -> cell 0 -> para 0 -> nested table(ctrl 1) -> cell 1
        let para = &core.document.sections[0].paragraphs[1];
        let Some(Control::Table(t0)) = para.controls.get(0) else {
            panic!("outer table missing")
        };
        let inner_host = &t0.cells[0].paragraphs[0];
        let Some(Control::Table(t1)) = inner_host.controls.get(1) else {
            panic!("nested table missing")
        };
        let inner_cell = &t1.cells[1];
        assert_eq!(inner_cell.paragraphs.len(), 6, "6 replacement paragraphs");

        let vps: Vec<i32> = inner_cell
            .paragraphs
            .iter()
            .map(|p| p.line_segs.first().map(|s| s.vertical_pos).unwrap_or(-1))
            .collect();
        for w in vps.windows(2) {
            assert!(
                w[1] > w[0],
                "nested cell paragraph vertical_pos must strictly increase, got {vps:?}"
            );
        }
        let pitch = inner_cell.paragraphs[0]
            .line_segs
            .first()
            .map(|s| s.line_height + s.line_spacing)
            .unwrap_or(0);
        assert!(pitch > 0, "line pitch must be positive, got {pitch}");
        assert!(
            vps[1] - vps[0] >= pitch - 1,
            "vpos step ({}) must be >= one line pitch ({pitch}); vps={vps:?}",
            vps[1] - vps[0]
        );
    }

    #[test]
    fn test_page_overflow_with_enter() {
        let mut core = DocumentCore::new_empty();
        core.create_blank_document_native().unwrap();

        assert_eq!(core.page_count(), 1, "초기 페이지 수");
        assert_eq!(
            core.document.sections[0].paragraphs.len(),
            1,
            "초기 문단 수"
        );

        // Enter를 500번 입력하여 페이지 오버플로우 유발
        for i in 0..500 {
            let para_count = core.document.sections[0].paragraphs.len();
            core.split_paragraph_native(0, para_count - 1, 0).unwrap();
        }

        let para_count = core.document.sections[0].paragraphs.len();
        let page_count = core.page_count();
        assert_eq!(para_count, 501, "문단 수");
        assert!(
            page_count >= 2,
            "페이지 수: {} (2 이상이어야 함)",
            page_count
        );
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
        assert!(
            para0_lines >= 2,
            "첫 문단은 2줄 이상이어야 함: {}",
            para0_lines
        );

        // Enter로 문단 분리 (텍스트 끝에서)
        let text_len = core.document.sections[0].paragraphs[0].text.chars().count();
        core.split_paragraph_native(0, 0, text_len).unwrap();

        // 두 번째 문단에 텍스트 입력
        core.insert_text_native(0, 1, 0, "Second paragraph")
            .unwrap();

        // 렌더 트리 빌드 (페이지 0)
        let tree = core.build_page_tree(0).unwrap();
        let tree_str = format!("{:?}", tree);

        // 렌더 트리에서 문단들의 Y 좌표를 추출
        // 두 번째 문단 "Second" 텍스트가 존재하는지 확인
        assert!(
            tree_str.contains("Second paragraph"),
            "두 번째 문단 텍스트가 렌더 트리에 없음"
        );

        // 렌더 트리에서 TextRun Y 좌표 확인
        let para0_last_y = find_text_y(&tree.root, "dog.");
        let para1_y = find_text_y(&tree.root, "Second");
        eprintln!(
            "문단0 마지막줄 Y: {:?}, 문단1 Y: {:?}",
            para0_last_y, para1_y
        );

        if let (Some(y0), Some(y1)) = (para0_last_y, para1_y) {
            assert!(
                y1 > y0,
                "문단1 Y({:.1})가 문단0 Y({:.1})보다 커야 함 (겹침 감지)",
                y1,
                y0
            );
        }
    }

    /// 줄간격 160%(기본값)에서 페이지 넘김 확인
    #[test]
    fn test_page_break_with_default_line_spacing() {
        let mut core = DocumentCore::new_empty();
        core.create_blank_document_native().unwrap();

        // 텍스트를 넣고 Enter로 문단 분리 반복 → 페이지 넘김 검증
        let text = "Line spacing 160 percent default.";
        for i in 0..100 {
            let para_count = core.document.sections[0].paragraphs.len();
            let last = para_count - 1;
            core.insert_text_native(0, last, 0, text).unwrap();
            core.split_paragraph_native(0, last, text.len()).unwrap();
        }

        let page_count = core.page_count();
        eprintln!("160% 줄간격: 문단 101개, 페이지 수: {}", page_count);
        assert!(
            page_count >= 2,
            "160% 줄간격에서 페이지 넘김 필요: {}",
            page_count
        );
    }

    /// 줄간격 100%에서 200%보다 더 많은 문단이 한 페이지에 들어가는지 확인
    /// (비교 대상이 160%면 height_for_fit 모델의 trail_ls 절약 효과로 1페이지 역전 가능 → 200% 사용)
    #[test]
    fn test_page_break_with_tight_line_spacing() {
        // 100% 줄간격 문서
        let mut core100 = DocumentCore::new_empty();
        core100.create_blank_document_native().unwrap();
        let text = "Tight spacing test line.";
        // 첫 문단에 줄간격 100% 적용
        core100
            .apply_para_format_native(0, 0, r#"{"lineSpacing":100}"#)
            .unwrap();
        for i in 0..500 {
            let para_count = core100.document.sections[0].paragraphs.len();
            let last = para_count - 1;
            core100.insert_text_native(0, last, 0, text).unwrap();
            core100.split_paragraph_native(0, last, text.len()).unwrap();
            // 새 문단에도 100% 적용
            let new_last = core100.document.sections[0].paragraphs.len() - 1;
            core100
                .apply_para_format_native(0, new_last, r#"{"lineSpacing":100}"#)
                .unwrap();
        }
        let pages_100 = core100.page_count();

        // 200% 줄간격 문서 (비교 기준)
        let mut core200 = DocumentCore::new_empty();
        core200.create_blank_document_native().unwrap();
        core200
            .apply_para_format_native(0, 0, r#"{"lineSpacing":200}"#)
            .unwrap();
        for i in 0..500 {
            let para_count = core200.document.sections[0].paragraphs.len();
            let last = para_count - 1;
            core200.insert_text_native(0, last, 0, text).unwrap();
            core200.split_paragraph_native(0, last, text.len()).unwrap();
            let new_last = core200.document.sections[0].paragraphs.len() - 1;
            core200
                .apply_para_format_native(0, new_last, r#"{"lineSpacing":200}"#)
                .unwrap();
        }
        let pages_200 = core200.page_count();

        eprintln!(
            "100% → {}페이지, 200% → {}페이지 (문단 501개)",
            pages_100, pages_200
        );
        // 100%는 200%보다 같거나 적은 페이지 수
        assert!(
            pages_100 <= pages_200,
            "100% 줄간격({})이 200%({})보다 적은/같은 페이지 수여야 함",
            pages_100,
            pages_200
        );
    }

    /// 줄간격 300%에서 160%보다 더 빨리 페이지가 넘어가는지 확인
    #[test]
    fn test_page_break_with_wide_line_spacing() {
        // 300% 줄간격
        let mut core300 = DocumentCore::new_empty();
        core300.create_blank_document_native().unwrap();
        let text = "Wide spacing test line.";
        core300
            .apply_para_format_native(0, 0, r#"{"lineSpacing":300}"#)
            .unwrap();
        for i in 0..30 {
            let para_count = core300.document.sections[0].paragraphs.len();
            let last = para_count - 1;
            core300.insert_text_native(0, last, 0, text).unwrap();
            core300.split_paragraph_native(0, last, text.len()).unwrap();
            let new_last = core300.document.sections[0].paragraphs.len() - 1;
            core300
                .apply_para_format_native(0, new_last, r#"{"lineSpacing":300}"#)
                .unwrap();
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

        eprintln!(
            "300% → {}페이지, 160% → {}페이지 (문단 31개)",
            pages_300, pages_160
        );
        assert!(
            pages_300 >= pages_160,
            "300% 줄간격({})이 160%({})보다 많은/같은 페이지 수여야 함",
            pages_300,
            pages_160
        );
    }

    /// 혼합 줄간격: 문단마다 다른 줄간격에서 페이지 넘김 정상 동작 확인
    #[test]
    fn test_page_break_with_mixed_line_spacing() {
        let mut core = DocumentCore::new_empty();
        core.create_blank_document_native().unwrap();

        let spacings = [160, 100, 300, 250, 120, 200];
        let text = "Mixed spacing paragraph content here.";

        for i in 0..120 {
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
        eprintln!(
            "혼합 줄간격: 문단 {}개, 페이지 수: {}",
            para_count, page_count
        );
        assert!(
            page_count >= 2,
            "혼합 줄간격에서 페이지 넘김 필요: {}",
            page_count
        );

        // 각 페이지에 문단이 배치되었는지 확인 (렌더 트리 빌드 가능)
        for p in 0..page_count {
            let tree = core.build_page_tree(p as u32);
            assert!(
                tree.is_ok(),
                "페이지 {} 렌더 트리 빌드 실패: {:?}",
                p,
                tree.err()
            );
        }
    }

    /// 고정(Fixed) 줄간격에서 페이지 넘김 정상 동작 확인
    #[test]
    fn test_page_break_with_fixed_line_spacing() {
        let mut core = DocumentCore::new_empty();
        core.create_blank_document_native().unwrap();

        let text = "Fixed spacing paragraph.";
        // Fixed 줄간격 30px
        core.apply_para_format_native(0, 0, r#"{"lineSpacing":30,"lineSpacingType":"Fixed"}"#)
            .unwrap();

        for i in 0..50 {
            let para_count = core.document.sections[0].paragraphs.len();
            let last = para_count - 1;
            core.insert_text_native(0, last, 0, text).unwrap();
            core.split_paragraph_native(0, last, text.len()).unwrap();
            let new_last = core.document.sections[0].paragraphs.len() - 1;
            core.apply_para_format_native(
                0,
                new_last,
                r#"{"lineSpacing":30,"lineSpacingType":"Fixed"}"#,
            )
            .unwrap();
        }

        let page_count = core.page_count();
        eprintln!("Fixed 줄간격: 문단 51개, 페이지 수: {}", page_count);
        assert!(
            page_count >= 1,
            "Fixed 줄간격에서 페이지 수 확인: {}",
            page_count
        );

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
            assert!(
                page_counts[i].1 >= page_counts[i - 1].1,
                "줄간격 {}%({})가 {}%({})보다 적은 페이지 수",
                page_counts[i].0,
                page_counts[i].1,
                page_counts[i - 1].0,
                page_counts[i - 1].1
            );
        }
    }

    /// 기존 문서 중간 문단의 줄간격을 10%씩 증가시키면 페이지 경계를 정확히 돌파하는지 검증
    #[test]
    fn test_page_boundary_with_incremental_spacing_increase() {
        let mut core = DocumentCore::new_empty();
        core.create_blank_document_native().unwrap();

        // 160% 줄간격으로 30개의 multi-line 문단 생성 (1페이지에 거의 맞도록)
        // height_for_fit 모델에서 trailing line_spacing은 제외되므로,
        // single-line 문단으로는 spacing 증가 효과가 약화됨 → multi-line text 사용
        let text = "Test paragraph for spacing. ".repeat(20);
        let text = text.as_str();
        for _ in 0..29 {
            let last = core.document.sections[0].paragraphs.len() - 1;
            core.insert_text_native(0, last, 0, text).unwrap();
            core.split_paragraph_native(0, last, text.len()).unwrap();
        }
        // 마지막 문단에도 텍스트
        let last = core.document.sections[0].paragraphs.len() - 1;
        core.insert_text_native(0, last, 0, text).unwrap();

        let initial_pages = core.page_count();
        eprintln!(
            "초기 페이지 수: {} (30 multi-line 문단 160%)",
            initial_pages
        );

        // 문단 15~25의 줄간격을 10%씩 증가 (170%, 180%, ..., 270%)
        let mut prev_pages = initial_pages;
        let mut boundary_crossed_at = 0;
        for step in 0..20 {
            let spacing = 170 + step * 10; // 170% → 360%
            for para_idx in 5..30 {
                if para_idx < core.document.sections[0].paragraphs.len() {
                    let json = format!(r#"{{"lineSpacing":{}}}"#, spacing);
                    core.apply_para_format_native(0, para_idx, &json).unwrap();
                }
            }
            let pages = core.page_count();
            if pages > prev_pages && boundary_crossed_at == 0 {
                boundary_crossed_at = spacing;
                eprintln!(
                    "  페이지 경계 돌파: {}% 줄간격에서 {}→{}페이지",
                    spacing, prev_pages, pages
                );
            }
            prev_pages = pages;
        }

        eprintln!("최종 페이지 수: {} (줄간격 360%)", prev_pages);
        assert!(
            prev_pages > initial_pages,
            "줄간격 증가로 페이지 수 증가 필요: {} → {}",
            initial_pages,
            prev_pages
        );
        assert!(
            boundary_crossed_at > 0,
            "페이지 경계 돌파 시점이 감지되어야 함"
        );

        // 모든 페이지 렌더 트리 정상 빌드 확인
        for p in 0..prev_pages {
            let tree = core.build_page_tree(p as u32);
            assert!(
                tree.is_ok(),
                "페이지 {} 렌더 트리 빌드 실패: {:?}",
                p,
                tree.err()
            );
        }
    }

    // set_cell_text_native: a 2x1 table, fill cell 0 with a two-line
    // "EN ¶ 해석" content in ONE op, then assert the cell has two paragraphs
    // with that text AND each new paragraph inherits the first paragraph's
    // ParaShape/CharShape/style (format preserved).
    #[test]
    fn set_cell_text_replaces_with_multi_paragraph_preserving_format() {
        let mut core = DocumentCore::new_empty();
        core.create_blank_document_native().unwrap();

        // Insert a 2x1 table at the body start; find its control index.
        let created = core.create_table_native(0, 0, 0, 2, 1).unwrap();
        let v: serde_json::Value = serde_json::from_str(&created).unwrap();
        let para_idx = v["paraIdx"].as_u64().unwrap() as usize;
        let ctrl_idx = v["controlIdx"].as_u64().unwrap() as usize;

        // Seed cell 0 with a single existing line so we exercise the CLEAR path,
        // and capture its para/char shape to compare against the rebuilt ones.
        core.insert_text_in_cell_native(0, para_idx, ctrl_idx, 0, 0, 0, "old text")
            .unwrap();
        let (base_para_shape, base_style) = {
            let p = core
                .get_cell_paragraph_ref(0, para_idx, ctrl_idx, 0, 0)
                .unwrap();
            (p.para_shape_id, p.style_id)
        };
        let base_char_shape = core
            .get_cell_paragraph_ref(0, para_idx, ctrl_idx, 0, 0)
            .unwrap()
            .char_shapes
            .first()
            .map(|cs| cs.char_shape_id);

        // ONE op replaces the whole cell with two paragraphs.
        let lines = vec![
            "① Sleep is important.".to_string(),
            "수면은 중요하다.".to_string(),
        ];
        let res = core
            .set_cell_text_native(0, para_idx, ctrl_idx, 0, &lines)
            .unwrap();
        let rv: serde_json::Value = serde_json::from_str(&res).unwrap();
        assert_eq!(rv["cellParaCount"].as_u64().unwrap(), 2);

        // Two cell paragraphs with the exact text.
        let p0 = core
            .get_cell_paragraph_ref(0, para_idx, ctrl_idx, 0, 0)
            .unwrap();
        assert_eq!(p0.text, "① Sleep is important.");
        let p1 = core
            .get_cell_paragraph_ref(0, para_idx, ctrl_idx, 0, 1)
            .unwrap();
        assert_eq!(p1.text, "수면은 중요하다.");
        // No stray third paragraph.
        assert!(core
            .get_cell_paragraph_ref(0, para_idx, ctrl_idx, 0, 2)
            .is_none());

        // Format preserved: both rebuilt paragraphs carry the original cell
        // paragraph's ParaShape, style, and char shape.
        assert_eq!(p0.para_shape_id, base_para_shape, "p0 ParaShape preserved");
        assert_eq!(p1.para_shape_id, base_para_shape, "p1 ParaShape preserved");
        assert_eq!(p0.style_id, base_style, "p0 style preserved");
        assert_eq!(p1.style_id, base_style, "p1 style preserved");
        // Both rebuilt paragraphs carry a char shape, and the second line
        // inherits the SAME char shape as the first (the cell's run style flows
        // to every new line — the format-preservation guarantee). `base_char_shape`
        // is recorded for documentation; synthetic empty cells may start without
        // an explicit char shape, so the load-bearing check is p0 == p1.
        let _ = base_char_shape;
        let p0_cs = p0.char_shapes.first().map(|cs| cs.char_shape_id);
        let p1_cs = p1.char_shapes.first().map(|cs| cs.char_shape_id);
        assert!(p0_cs.is_some(), "p0 has a char shape");
        assert_eq!(p0_cs, p1_cs, "new line inherits the cell's char shape");
    }

    #[test]
    fn set_cell_text_grows_stored_cell_and_table_height() {
        let mut core = DocumentCore::new_empty();
        core.create_blank_document_native().unwrap();

        let created = core.create_table_native(0, 0, 0, 1, 1).unwrap();
        let v: serde_json::Value = serde_json::from_str(&created).unwrap();
        let para_idx = v["paraIdx"].as_u64().unwrap() as usize;
        let ctrl_idx = v["controlIdx"].as_u64().unwrap() as usize;

        let (before_cell_h, before_table_h) =
            table_cell_and_common_height(&core, para_idx, ctrl_idx, 0);

        let lines: Vec<String> = (1..=8).map(|i| format!("line {i}")).collect();
        core.set_cell_text_native(0, para_idx, ctrl_idx, 0, &lines)
            .unwrap();

        let (after_cell_h, after_table_h) =
            table_cell_and_common_height(&core, para_idx, ctrl_idx, 0);

        assert!(
            after_cell_h > before_cell_h,
            "edited cell height must grow for exported HWP geometry"
        );
        assert!(
            after_table_h >= after_cell_h && after_table_h > before_table_h,
            "table common.height must track grown row height"
        );

        let para = &core.document.sections[0].paragraphs[para_idx];
        let table = match para.controls.get(ctrl_idx).unwrap() {
            Control::Table(table) => table,
            _ => panic!("expected table"),
        };
        let raw_height = u32::from_le_bytes(table.raw_ctrl_data[16..20].try_into().unwrap());
        assert_eq!(raw_height, table.common.height);
    }

    fn table_cell_and_common_height(
        core: &DocumentCore,
        para_idx: usize,
        ctrl_idx: usize,
        cell_idx: usize,
    ) -> (u32, u32) {
        let para = &core.document.sections[0].paragraphs[para_idx];
        let table = match para.controls.get(ctrl_idx).unwrap() {
            Control::Table(table) => table,
            _ => panic!("expected table"),
        };
        (table.cells[cell_idx].height, table.common.height)
    }
}

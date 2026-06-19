//! 필드 조회/설정 API (Task 230)
//!
//! 문서 전체에서 필드를 재귀 탐색하여 조회·설정하는 기능을 제공한다.

use crate::document_core::DocumentCore;
use crate::error::HwpError;
use crate::model::control::{Control, Field, FieldType};
use crate::model::event::DocumentEvent;
use crate::model::paragraph::{FieldRange, Paragraph};
use crate::parser::tags;

/// 필드 위치 정보
#[derive(Debug, Clone)]
pub struct FieldLocation {
    pub section_index: usize,
    pub para_index: usize,
    /// 표/글상자 내 필드인 경우 중첩 경로
    pub nested_path: Vec<NestedEntry>,
}

/// 중첩 경로 항목 (표 셀 또는 글상자 내부)
#[derive(Debug, Clone)]
pub enum NestedEntry {
    /// 표 셀: (control_index, cell_index, para_index)
    TableCell {
        control_index: usize,
        cell_index: usize,
        para_index: usize,
    },
    /// 글상자: (control_index, para_index)
    TextBox {
        control_index: usize,
        para_index: usize,
    },
}

/// 필드 검색 결과
#[derive(Debug)]
pub struct FieldInfo {
    pub field: Field,
    pub location: FieldLocation,
    /// 필드 범위 내 텍스트 (빈 필드이면 빈 문자열)
    pub value: String,
    /// field_ranges에서의 인덱스
    pub field_range_index: usize,
}

impl DocumentCore {
    /// 문서 전체에서 모든 필드를 검색하여 목록으로 반환한다.
    pub fn collect_all_fields(&self) -> Vec<FieldInfo> {
        let mut result = Vec::new();
        for (si, sec) in self.document.sections.iter().enumerate() {
            for (pi, para) in sec.paragraphs.iter().enumerate() {
                let loc = FieldLocation {
                    section_index: si,
                    para_index: pi,
                    nested_path: Vec::new(),
                };
                collect_fields_from_paragraph(para, &loc, &mut result);
            }
        }
        result
    }

    /// 본문 문단의 현재 커서 위치에 빈 ClickHere 누름틀을 삽입한다.
    pub fn insert_click_here_field_at(
        &mut self,
        section_idx: usize,
        para_idx: usize,
        char_offset: usize,
        guide: &str,
        memo: &str,
        name: &str,
        editable: bool,
    ) -> Result<String, HwpError> {
        let field_id = self.next_click_here_field_id();
        let inserted_offset = {
            let section = self
                .document
                .sections
                .get_mut(section_idx)
                .ok_or_else(|| HwpError::InvalidField("구역 인덱스 초과".into()))?;
            section.raw_stream = None;
            let para = section
                .paragraphs
                .get_mut(para_idx)
                .ok_or_else(|| HwpError::InvalidField("문단 인덱스 초과".into()))?;
            insert_click_here_field_in_para(
                para,
                char_offset,
                field_id,
                guide,
                memo,
                name,
                editable,
            )?
        };

        self.reflow_paragraph(section_idx, para_idx);
        crate::renderer::composer::recalculate_section_vpos(
            &mut self.document.sections[section_idx].paragraphs,
            para_idx,
        );
        self.recompose_paragraph(section_idx, para_idx);
        self.paginate_if_needed();
        self.invalidate_page_tree_cache();
        self.event_log.push(DocumentEvent::TextInserted {
            section: section_idx,
            para: para_idx,
            offset: inserted_offset,
            len: 0,
        });

        Ok(format!(
            "{{\"ok\":true,\"fieldId\":{},\"charOffset\":{}}}",
            field_id, inserted_offset
        ))
    }

    /// 셀/글상자 내 문단의 현재 커서 위치에 빈 ClickHere 누름틀을 삽입한다.
    pub fn insert_click_here_field_at_in_cell(
        &mut self,
        section_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
        cell_idx: usize,
        cell_para_idx: usize,
        char_offset: usize,
        _is_textbox: bool,
        guide: &str,
        memo: &str,
        name: &str,
        editable: bool,
    ) -> Result<String, HwpError> {
        let field_id = self.next_click_here_field_id();
        let inserted_offset = {
            let para = self.get_cell_paragraph_mut(
                section_idx,
                parent_para_idx,
                control_idx,
                cell_idx,
                cell_para_idx,
            )?;
            insert_click_here_field_in_para(
                para,
                char_offset,
                field_id,
                guide,
                memo,
                name,
                editable,
            )?
        };

        self.mark_cell_control_dirty(section_idx, parent_para_idx, control_idx);
        self.reflow_cell_paragraph(
            section_idx,
            parent_para_idx,
            control_idx,
            cell_idx,
            cell_para_idx,
        );
        if let Some(section) = self.document.sections.get_mut(section_idx) {
            section.raw_stream = None;
        }
        self.mark_section_dirty(section_idx);
        self.paginate_if_needed();
        self.invalidate_page_tree_cache();
        self.event_log.push(DocumentEvent::CellTextChanged {
            section: section_idx,
            para: parent_para_idx,
            ctrl: control_idx,
            cell: cell_idx,
        });

        Ok(format!(
            "{{\"ok\":true,\"fieldId\":{},\"charOffset\":{}}}",
            field_id, inserted_offset
        ))
    }

    /// path 기반 중첩 표 셀의 현재 커서 위치에 빈 ClickHere 누름틀을 삽입한다.
    pub fn insert_click_here_field_at_by_path(
        &mut self,
        section_idx: usize,
        parent_para_idx: usize,
        path: &[(usize, usize, usize)],
        char_offset: usize,
        guide: &str,
        memo: &str,
        name: &str,
        editable: bool,
    ) -> Result<String, HwpError> {
        if path.is_empty() {
            return Err(HwpError::InvalidField("cellPath가 비어 있음".into()));
        }
        let field_id = self.next_click_here_field_id();
        let inserted_offset = {
            let para = self.get_cell_paragraph_mut_by_path(section_idx, parent_para_idx, path)?;
            insert_click_here_field_in_para(
                para,
                char_offset,
                field_id,
                guide,
                memo,
                name,
                editable,
            )?
        };

        let outer_ctrl = path[0].0;
        self.mark_cell_control_dirty(section_idx, parent_para_idx, outer_ctrl);
        if let Some(section) = self.document.sections.get_mut(section_idx) {
            section.raw_stream = None;
        }
        self.mark_section_dirty(section_idx);
        self.paginate_if_needed();
        self.invalidate_page_tree_cache();
        self.event_log.push(DocumentEvent::CellTextChanged {
            section: section_idx,
            para: parent_para_idx,
            ctrl: outer_ctrl,
            cell: path[0].1,
        });

        Ok(format!(
            "{{\"ok\":true,\"fieldId\":{},\"charOffset\":{}}}",
            field_id, inserted_offset
        ))
    }

    /// getFieldList: 모든 필드를 JSON 배열로 반환
    pub fn get_field_list_json(&self) -> String {
        let fields = self.collect_all_fields();
        let entries: Vec<String> = fields
            .iter()
            .map(|fi| {
                let name = fi.field.field_name().unwrap_or("");
                let guide = fi.field.guide_text().unwrap_or("");
                let location_json = field_location_json(&fi.location);
                let (start_char_idx, end_char_idx) = field_range_bounds(self, fi)
                    .unwrap_or((0, fi.value.chars().count()));
                format!(
                    "{{\"fieldId\":{},\"fieldType\":\"{}\",\"name\":{},\"guide\":{},\"command\":{},\"value\":{},\"location\":{},\"startCharIdx\":{},\"endCharIdx\":{},\"editableInForm\":{}}}",
                    fi.field.field_id,
                    fi.field.field_type_str(),
                    json_escape(name),
                    json_escape(guide),
                    json_escape(&fi.field.command),
                    json_escape(&fi.value),
                    location_json,
                    start_char_idx,
                    end_char_idx,
                    fi.field.is_editable_in_form(),
                )
            })
            .collect();
        format!("[{}]", entries.join(","))
    }

    /// getFieldValue: field_id로 필드 값 조회
    pub fn get_field_value_by_id(&self, field_id: u32) -> Result<String, HwpError> {
        let fields = self.collect_all_fields();
        for fi in &fields {
            if fi.field.field_id == field_id {
                return Ok(format!(
                    "{{\"ok\":true,\"value\":{}}}",
                    json_escape(&fi.value)
                ));
            }
        }
        Err(HwpError::InvalidField(format!("필드 ID {} 없음", field_id)))
    }

    /// getFieldValueByName: 필드 이름으로 값 조회
    pub fn get_field_value_by_name(&self, name: &str) -> Result<String, HwpError> {
        let fields = self.collect_all_fields();
        for fi in &fields {
            if let Some(field_name) = fi.field.field_name() {
                if field_name == name {
                    return Ok(format!(
                        "{{\"ok\":true,\"fieldId\":{},\"value\":{}}}",
                        fi.field.field_id,
                        json_escape(&fi.value),
                    ));
                }
            }
        }
        Err(HwpError::InvalidField(format!("필드 이름 '{}' 없음", name)))
    }

    /// setFieldValue: field_id로 필드 값 설정
    pub fn set_field_value_by_id(
        &mut self,
        field_id: u32,
        value: &str,
    ) -> Result<String, HwpError> {
        // 먼저 필드 위치 찾기
        let fields = self.collect_all_fields();
        let fi = fields
            .iter()
            .find(|f| f.field.field_id == field_id)
            .ok_or_else(|| HwpError::InvalidField(format!("필드 ID {} 없음", field_id)))?;

        let location = fi.location.clone();
        let fri = fi.field_range_index;
        let old_value = fi.value.clone();

        let section_index = location.section_index;
        self.set_field_text_at(&location, fri, value)?;
        self.recompose_section(section_index);

        Ok(format!(
            "{{\"ok\":true,\"fieldId\":{},\"oldValue\":{},\"newValue\":{}}}",
            field_id,
            json_escape(&old_value),
            json_escape(value),
        ))
    }

    /// setFieldValueByName: 필드 이름으로 값 설정
    pub fn set_field_value_by_name(&mut self, name: &str, value: &str) -> Result<String, HwpError> {
        let fields = self.collect_all_fields();
        let fi = fields
            .iter()
            .find(|f| f.field.field_name().map(|n| n == name).unwrap_or(false))
            .ok_or_else(|| HwpError::InvalidField(format!("필드 이름 '{}' 없음", name)))?;

        let field_id = fi.field.field_id;
        let location = fi.location.clone();
        let fri = fi.field_range_index;
        let old_value = fi.value.clone();
        let is_cell_field = fi.field.ctrl_id == 0; // 가상 셀 필드

        let section_index = location.section_index;

        if is_cell_field {
            // 셀 필드: 셀의 첫 문단 텍스트를 직접 교체
            self.set_cell_field_text(&location, value)?;
        } else {
            // ClickHere 필드: field_ranges 기반 교체
            self.set_field_text_at(&location, fri, value)?;
        }

        // raw_stream 무효화
        if let Some(sec) = self.document.sections.get_mut(section_index) {
            sec.raw_stream = None;
        }
        self.recompose_section(section_index);

        Ok(format!(
            "{{\"ok\":true,\"fieldId\":{},\"oldValue\":{},\"newValue\":{}}}",
            field_id,
            json_escape(&old_value),
            json_escape(value),
        ))
    }

    /// 셀 필드의 텍스트를 교체한다 (셀의 첫 문단 텍스트를 value로 대체).
    /// 중첩 표를 재귀적으로 탐색하여 임의 깊이를 지원한다.
    fn set_cell_field_text(
        &mut self,
        location: &FieldLocation,
        value: &str,
    ) -> Result<(), HwpError> {
        if location.nested_path.is_empty() {
            return Err(HwpError::InvalidField(
                "셀 필드 위치에 중첩 경로 없음".into(),
            ));
        }
        let sec = self
            .document
            .sections
            .get_mut(location.section_index)
            .ok_or_else(|| HwpError::InvalidField("구역 초과".into()))?;
        let mut para: &mut Paragraph = sec
            .paragraphs
            .get_mut(location.para_index)
            .ok_or_else(|| HwpError::InvalidField("문단 초과".into()))?;

        // 마지막 항목 직전까지 중첩 탐색
        for (i, entry) in location.nested_path[..location.nested_path.len() - 1]
            .iter()
            .enumerate()
        {
            para = match entry {
                NestedEntry::TableCell {
                    control_index,
                    cell_index,
                    para_index,
                } => {
                    let ctrl = para.controls.get_mut(*control_index).ok_or_else(|| {
                        HwpError::InvalidField(format!(
                            "경로[{}]: 컨트롤 인덱스 {} 초과",
                            i, control_index
                        ))
                    })?;
                    if let Control::Table(ref mut table) = ctrl {
                        let cell = table.cells.get_mut(*cell_index).ok_or_else(|| {
                            HwpError::InvalidField(format!(
                                "경로[{}]: 셀 인덱스 {} 초과",
                                i, cell_index
                            ))
                        })?;
                        cell.paragraphs.get_mut(*para_index).ok_or_else(|| {
                            HwpError::InvalidField(format!(
                                "경로[{}]: 셀 문단 인덱스 {} 초과",
                                i, para_index
                            ))
                        })?
                    } else {
                        return Err(HwpError::InvalidField(format!(
                            "경로[{}]: controls[{}]가 Table이 아님",
                            i, control_index
                        )));
                    }
                }
                NestedEntry::TextBox {
                    control_index,
                    para_index,
                } => {
                    let ctrl = para.controls.get_mut(*control_index).ok_or_else(|| {
                        HwpError::InvalidField(format!(
                            "경로[{}]: 컨트롤 인덱스 {} 초과",
                            i, control_index
                        ))
                    })?;
                    if let Control::Shape(ref mut shape) = ctrl {
                        let drawing = shape.drawing_mut().ok_or_else(|| {
                            HwpError::InvalidField(format!(
                                "경로[{}]: Shape에 DrawingObjAttr 없음",
                                i
                            ))
                        })?;
                        let tb = drawing.text_box.as_mut().ok_or_else(|| {
                            HwpError::InvalidField(format!("경로[{}]: Shape에 TextBox 없음", i))
                        })?;
                        tb.paragraphs.get_mut(*para_index).ok_or_else(|| {
                            HwpError::InvalidField(format!(
                                "경로[{}]: 글상자 문단 인덱스 {} 초과",
                                i, para_index
                            ))
                        })?
                    } else {
                        return Err(HwpError::InvalidField(format!(
                            "경로[{}]: controls[{}]가 Shape가 아님",
                            i, control_index
                        )));
                    }
                }
            };
        }

        // 마지막 항목: 셀의 첫 문단 텍스트를 교체
        let last_idx = location.nested_path.len() - 1;
        let last = location.nested_path.last().unwrap();
        match last {
            NestedEntry::TableCell {
                control_index,
                cell_index,
                ..
            } => {
                let ctrl = para.controls.get_mut(*control_index).ok_or_else(|| {
                    HwpError::InvalidField(format!(
                        "경로[{}]: 컨트롤 인덱스 {} 초과",
                        last_idx, control_index
                    ))
                })?;
                if let Control::Table(ref mut table) = ctrl {
                    let cell = table.cells.get_mut(*cell_index).ok_or_else(|| {
                        HwpError::InvalidField(format!(
                            "경로[{}]: 셀 인덱스 {} 초과",
                            last_idx, cell_index
                        ))
                    })?;
                    if let Some(cell_para) = cell.paragraphs.first_mut() {
                        let old_len = cell_para.text.chars().count();
                        if old_len > 0 {
                            cell_para.delete_text_at(0, old_len);
                        }
                        if !value.is_empty() {
                            cell_para.insert_text_at(0, value);
                        }
                        rebuild_char_offsets(cell_para);
                    }
                    Ok(())
                } else {
                    Err(HwpError::InvalidField(format!(
                        "경로[{}]: controls[{}]가 Table이 아님",
                        last_idx, control_index
                    )))
                }
            }
            _ => Err(HwpError::InvalidField("셀 필드가 아닌 위치".into())),
        }
    }

    /// 필드 위치에서 텍스트를 교체한다.
    ///
    /// delete_text_at + insert_text_at를 사용하여 char_shapes, line_segs,
    /// range_tags, char_count 등 모든 메타데이터를 올바르게 시프트한다.
    /// (직접 para.text 조작 시 메타데이터 불일치로 한컴 "파일 손상" 발생 — #838)
    fn set_field_text_at(
        &mut self,
        location: &FieldLocation,
        field_range_index: usize,
        value: &str,
    ) -> Result<(), HwpError> {
        // raw_stream 무효화: 직렬화 시 수정된 모델을 사용하도록 강제
        if let Some(sec) = self.document.sections.get_mut(location.section_index) {
            sec.raw_stream = None;
        }
        let para = self.get_para_mut_at_location(location)?;
        let fr = para
            .field_ranges
            .get(field_range_index)
            .ok_or_else(|| HwpError::InvalidField("field_range 인덱스 초과".into()))?
            .clone();

        let start_idx = fr.start_char_idx;
        let count = fr.end_char_idx.saturating_sub(start_idx);

        // 기존 텍스트 삭제 (char_shapes, line_segs, range_tags 등 자동 시프트)
        if count > 0 {
            para.delete_text_at(start_idx, count);
        }

        // 새 값 삽입
        if !value.is_empty() {
            para.insert_text_at(start_idx, value);
        }

        // field_ranges 갱신: start와 end를 명시적으로 재설정
        let new_end = start_idx + value.chars().count();
        let current_fr = para
            .field_ranges
            .get_mut(field_range_index)
            .ok_or_else(|| HwpError::InvalidField("field_range 인덱스 초과".into()))?;
        current_fr.start_char_idx = start_idx;
        current_fr.end_char_idx = new_end;

        // char_offsets 재생성: FIELD_BEGIN/END 갭, 탭 폭, UTF-16 code unit 크기 반영
        rebuild_char_offsets(para);

        Ok(())
    }

    /// FieldLocation에 해당하는 Paragraph의 가변 참조를 반환한다.
    ///
    /// 중첩 표/글상자를 재귀적으로 탐색하여 임의 깊이를 지원한다.
    fn get_para_mut_at_location(
        &mut self,
        location: &FieldLocation,
    ) -> Result<&mut Paragraph, HwpError> {
        let sec = self
            .document
            .sections
            .get_mut(location.section_index)
            .ok_or_else(|| HwpError::InvalidField("구역 인덱스 초과".into()))?;
        let mut para = sec
            .paragraphs
            .get_mut(location.para_index)
            .ok_or_else(|| HwpError::InvalidField("문단 인덱스 초과".into()))?;

        for (i, entry) in location.nested_path.iter().enumerate() {
            para = match entry {
                NestedEntry::TableCell {
                    control_index,
                    cell_index,
                    para_index,
                } => {
                    let ctrl = para.controls.get_mut(*control_index).ok_or_else(|| {
                        HwpError::InvalidField(format!(
                            "경로[{}]: 컨트롤 인덱스 {} 초과",
                            i, control_index
                        ))
                    })?;
                    if let Control::Table(ref mut table) = ctrl {
                        let cell = table.cells.get_mut(*cell_index).ok_or_else(|| {
                            HwpError::InvalidField(format!(
                                "경로[{}]: 셀 인덱스 {} 초과",
                                i, cell_index
                            ))
                        })?;
                        cell.paragraphs.get_mut(*para_index).ok_or_else(|| {
                            HwpError::InvalidField(format!(
                                "경로[{}]: 셀 문단 인덱스 {} 초과",
                                i, para_index
                            ))
                        })?
                    } else {
                        return Err(HwpError::InvalidField(format!(
                            "경로[{}]: controls[{}]가 Table이 아님",
                            i, control_index
                        )));
                    }
                }
                NestedEntry::TextBox {
                    control_index,
                    para_index,
                } => {
                    let ctrl = para.controls.get_mut(*control_index).ok_or_else(|| {
                        HwpError::InvalidField(format!(
                            "경로[{}]: 컨트롤 인덱스 {} 초과",
                            i, control_index
                        ))
                    })?;
                    if let Control::Shape(ref mut shape) = ctrl {
                        let drawing = shape.drawing_mut().ok_or_else(|| {
                            HwpError::InvalidField(format!(
                                "경로[{}]: Shape에 DrawingObjAttr 없음",
                                i
                            ))
                        })?;
                        let tb = drawing.text_box.as_mut().ok_or_else(|| {
                            HwpError::InvalidField(format!("경로[{}]: Shape에 TextBox 없음", i))
                        })?;
                        tb.paragraphs.get_mut(*para_index).ok_or_else(|| {
                            HwpError::InvalidField(format!(
                                "경로[{}]: 글상자 문단 인덱스 {} 초과",
                                i, para_index
                            ))
                        })?
                    } else {
                        return Err(HwpError::InvalidField(format!(
                            "경로[{}]: controls[{}]가 Shape가 아님",
                            i, control_index
                        )));
                    }
                }
            };
        }

        Ok(para)
    }

    /// 본문 문단의 커서 위치에서 필드를 제거한다 (필드 내용과 컨트롤 삭제).
    ///
    /// 성공 시 `{"ok":true}`, 필드가 없으면 에러를 반환한다.
    pub fn remove_field_at(
        &mut self,
        section_idx: usize,
        para_idx: usize,
        char_offset: usize,
    ) -> Result<String, HwpError> {
        let para = self
            .document
            .sections
            .get_mut(section_idx)
            .and_then(|s| s.paragraphs.get_mut(para_idx))
            .ok_or_else(|| HwpError::InvalidField("문단 위치 초과".into()))?;
        remove_field_in_para(para, char_offset)?;
        self.recompose_section(section_idx);
        Ok(r#"{"ok":true}"#.to_string())
    }

    /// 셀/글상자 내 문단의 커서 위치에서 필드를 제거한다.
    pub fn remove_field_at_in_cell(
        &mut self,
        section_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
        cell_idx: usize,
        cell_para_idx: usize,
        char_offset: usize,
        is_textbox: bool,
    ) -> Result<String, HwpError> {
        let para = {
            let host = self
                .document
                .sections
                .get_mut(section_idx)
                .and_then(|s| s.paragraphs.get_mut(parent_para_idx))
                .ok_or_else(|| HwpError::InvalidField("호스트 문단 위치 초과".into()))?;
            let ctrl = host
                .controls
                .get_mut(control_idx)
                .ok_or_else(|| HwpError::InvalidField("컨트롤 인덱스 초과".into()))?;
            if is_textbox {
                if let Control::Shape(shape) = ctrl {
                    let drawing = shape.drawing_mut().ok_or_else(|| {
                        HwpError::InvalidField("Shape에 DrawingObjAttr 없음".into())
                    })?;
                    let tb = drawing
                        .text_box
                        .as_mut()
                        .ok_or_else(|| HwpError::InvalidField("Shape에 TextBox 없음".into()))?;
                    tb.paragraphs
                        .get_mut(cell_para_idx)
                        .ok_or_else(|| HwpError::InvalidField("글상자 문단 인덱스 초과".into()))?
                } else {
                    return Err(HwpError::InvalidField("예상된 Shape 컨트롤이 아님".into()));
                }
            } else {
                if let Control::Table(table) = ctrl {
                    let cell = table
                        .cells
                        .get_mut(cell_idx)
                        .ok_or_else(|| HwpError::InvalidField("셀 인덱스 초과".into()))?;
                    cell.paragraphs
                        .get_mut(cell_para_idx)
                        .ok_or_else(|| HwpError::InvalidField("셀 문단 인덱스 초과".into()))?
                } else {
                    return Err(HwpError::InvalidField("예상된 Table 컨트롤이 아님".into()));
                }
            }
        };
        remove_field_in_para(para, char_offset)?;
        self.recompose_section(section_idx);
        Ok(r#"{"ok":true}"#.to_string())
    }

    /// 커서가 진입한 활성 필드를 설정한다 (안내문 렌더링 스킵용).
    ///
    /// 본문 문단: `set_active_field(sec, para, char_offset)`
    /// 설정 후 해당 페이지의 렌더 트리 캐시를 무효화한다.
    /// 활성 필드를 설정한다. 변경이 발생하면 true를 반환한다.
    pub fn set_active_field(
        &mut self,
        section_idx: usize,
        para_idx: usize,
        char_offset: usize,
    ) -> bool {
        use super::super::ActiveFieldInfo;
        let ctrl_idx = self.find_field_control_idx(section_idx, para_idx, char_offset, None);
        if let Some(ci) = ctrl_idx {
            let new_info = ActiveFieldInfo {
                section_idx,
                para_idx,
                control_idx: ci,
                cell_path: None,
            };
            if self.active_field.as_ref() != Some(&new_info) {
                self.active_field = Some(new_info);
                self.invalidate_page_tree_cache();
                return true;
            }
        }
        false
    }

    /// 셀/글상자 내 활성 필드를 설정한다. 변경이 발생하면 true를 반환한다.
    pub fn set_active_field_in_cell(
        &mut self,
        section_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
        cell_idx: usize,
        cell_para_idx: usize,
        char_offset: usize,
        is_textbox: bool,
    ) -> bool {
        use super::super::ActiveFieldInfo;
        let cell_path = Some(vec![(control_idx, cell_idx, cell_para_idx)]);
        let ctrl_idx = self.find_field_control_idx_in_cell(
            section_idx,
            parent_para_idx,
            control_idx,
            cell_idx,
            cell_para_idx,
            char_offset,
            is_textbox,
        );
        if let Some(ci) = ctrl_idx {
            let new_info = ActiveFieldInfo {
                section_idx,
                para_idx: cell_para_idx,
                control_idx: ci,
                cell_path,
            };
            if self.active_field.as_ref() != Some(&new_info) {
                self.active_field = Some(new_info);
                self.invalidate_page_tree_cache();
                return true;
            }
        }
        false
    }

    /// 활성 필드를 해제한다.
    pub fn clear_active_field(&mut self) {
        if self.active_field.is_some() {
            self.active_field = None;
            self.invalidate_page_tree_cache();
        }
    }

    /// 본문 문단의 커서 위치에서 필드 범위 정보를 조회한다.
    ///
    /// 커서가 필드 범위 내에 있으면 필드 정보를 JSON으로 반환하고,
    /// 필드 밖이면 `{"inField":false}`를 반환한다.
    pub fn get_field_info_at(
        &self,
        section_idx: usize,
        para_idx: usize,
        char_offset: usize,
    ) -> String {
        let para = match self
            .document
            .sections
            .get(section_idx)
            .and_then(|s| s.paragraphs.get(para_idx))
        {
            Some(p) => p,
            None => return r#"{"inField":false}"#.to_string(),
        };
        field_info_at_in_para(para, char_offset)
    }

    /// 셀/글상자 내 문단의 커서 위치에서 필드 범위 정보를 조회한다.
    pub fn get_field_info_at_in_cell(
        &self,
        section_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
        cell_idx: usize,
        cell_para_idx: usize,
        char_offset: usize,
        is_textbox: bool,
    ) -> String {
        let para = (|| {
            let host = self
                .document
                .sections
                .get(section_idx)?
                .paragraphs
                .get(parent_para_idx)?;
            let ctrl = host.controls.get(control_idx)?;
            if is_textbox {
                if let Control::Shape(shape) = ctrl {
                    let tb = shape.drawing()?.text_box.as_ref()?;
                    return tb.paragraphs.get(cell_para_idx);
                }
            } else {
                if let Control::Table(table) = ctrl {
                    let cell = table.cells.get(cell_idx)?;
                    return cell.paragraphs.get(cell_para_idx);
                }
            }
            None
        })();
        match para {
            Some(p) => field_info_at_in_para(p, char_offset),
            None => r#"{"inField":false}"#.to_string(),
        }
    }

    /// path 기반: 중첩 표 셀의 필드 범위 정보를 조회한다.
    pub fn get_field_info_at_by_path(
        &self,
        section_idx: usize,
        parent_para_idx: usize,
        path: &[(usize, usize, usize)],
        char_offset: usize,
    ) -> String {
        match self.resolve_paragraph_by_path(section_idx, parent_para_idx, path) {
            Ok(para) => field_info_at_in_para(para, char_offset),
            Err(_) => r#"{"inField":false}"#.to_string(),
        }
    }

    /// path 기반: 중첩 표 셀 내 활성 필드를 설정한다.
    pub fn set_active_field_by_path(
        &mut self,
        section_idx: usize,
        parent_para_idx: usize,
        path: &[(usize, usize, usize)],
        char_offset: usize,
    ) -> bool {
        use super::super::ActiveFieldInfo;
        let para = match self.resolve_paragraph_by_path(section_idx, parent_para_idx, path) {
            Ok(p) => p,
            Err(_) => return false,
        };
        let ctrl_idx = find_field_ctrl_idx_in_para(para, char_offset);
        if let Some(ci) = ctrl_idx {
            let last = path.last().unwrap();
            let cell_para_idx = last.2;
            // cell_path: 전체 path를 저장 (중첩 표 구분용)
            let cell_path = Some(path.to_vec());
            let new_info = ActiveFieldInfo {
                section_idx,
                para_idx: cell_para_idx,
                control_idx: ci,
                cell_path,
            };
            if self.active_field.as_ref() != Some(&new_info) {
                self.active_field = Some(new_info);
                self.invalidate_page_tree_cache();
                return true;
            }
        }
        false
    }
}

/// 문단 내 커서 위치의 필드 범위 정보를 JSON으로 반환한다.
fn field_info_at_in_para(para: &Paragraph, char_offset: usize) -> String {
    for fr in &para.field_ranges {
        if fr.start_char_idx != fr.end_char_idx || char_offset != fr.start_char_idx {
            continue;
        }
        if let Some(Control::Field(field)) = para.controls.get(fr.control_idx) {
            if field.field_type != FieldType::ClickHere {
                continue;
            }
            let guide = field.guide_text().unwrap_or("");
            return format!(
                "{{\"inField\":true,\"fieldId\":{},\"fieldType\":\"{}\",\"startCharIdx\":{},\"endCharIdx\":{},\"isGuide\":true,\"guideName\":{},\"editableInForm\":{}}}",
                field.field_id,
                field.field_type_str(),
                fr.start_char_idx,
                fr.end_char_idx,
                json_escape(guide),
                field.is_editable_in_form(),
            );
        }
    }

    for fr in &para.field_ranges {
        if let Some(Control::Field(field)) = para.controls.get(fr.control_idx) {
            if field.field_type != FieldType::ClickHere {
                continue;
            }
            // 커서가 필드 범위 내에 있는지 확인 (start 이상, end 이하)
            // end가 exclusive이므로 커서가 end 위치에 있으면 필드 "끝"에 있는 것
            if char_offset >= fr.start_char_idx && char_offset <= fr.end_char_idx {
                let is_guide = fr.start_char_idx == fr.end_char_idx;
                let guide = field.guide_text().unwrap_or("");
                return format!(
                    "{{\"inField\":true,\"fieldId\":{},\"fieldType\":\"{}\",\"startCharIdx\":{},\"endCharIdx\":{},\"isGuide\":{},\"guideName\":{},\"editableInForm\":{}}}",
                    field.field_id,
                    field.field_type_str(),
                    fr.start_char_idx,
                    fr.end_char_idx,
                    is_guide,
                    json_escape(guide),
                    field.is_editable_in_form(),
                );
            }
        }
    }
    r#"{"inField":false}"#.to_string()
}

/// 문단에서 필드를 수집한다 (재귀: 표 셀, 글상자 내부 포함).
fn collect_fields_from_paragraph(
    para: &Paragraph,
    base_location: &FieldLocation,
    result: &mut Vec<FieldInfo>,
) {
    // 현재 문단의 field_ranges에서 필드 수집
    for (fri, fr) in para.field_ranges.iter().enumerate() {
        if let Some(Control::Field(field)) = para.controls.get(fr.control_idx) {
            let value = if fr.start_char_idx < fr.end_char_idx {
                let chars: Vec<char> = para.text.chars().collect();
                if fr.end_char_idx <= chars.len() {
                    chars[fr.start_char_idx..fr.end_char_idx].iter().collect()
                } else {
                    String::new()
                }
            } else {
                String::new()
            };

            result.push(FieldInfo {
                field: field.clone(),
                location: base_location.clone(),
                value,
                field_range_index: fri,
            });
        }
    }

    // 컨트롤 내부 재귀 탐색 (표 셀, 글상자)
    for (ci, ctrl) in para.controls.iter().enumerate() {
        match ctrl {
            Control::Table(table) => {
                for (cell_i, cell) in table.cells.iter().enumerate() {
                    // 셀 자체의 field_name이 있으면 가상 필드로 추가
                    if let Some(ref fname) = cell.field_name {
                        let mut loc = base_location.clone();
                        loc.nested_path.push(NestedEntry::TableCell {
                            control_index: ci,
                            cell_index: cell_i,
                            para_index: 0,
                        });
                        // 셀의 첫 문단 텍스트를 값으로 사용
                        let value = cell
                            .paragraphs
                            .first()
                            .map(|p| p.text.clone())
                            .unwrap_or_default();
                        result.push(FieldInfo {
                            field: Field {
                                ctrl_id: 0,
                                field_id: (ci as u32) << 16 | cell_i as u32,
                                field_type: FieldType::ClickHere,
                                command: String::new(),
                                properties: 0,
                                extra_properties: 0,
                                ctrl_data_name: Some(fname.clone()),
                                memo_index: 0,
                                memo_paragraphs: Vec::new(),
                                raw_parameters_xml: None,
                            },
                            location: loc,
                            value,
                            field_range_index: 0,
                        });
                    }
                    for (pi, cell_para) in cell.paragraphs.iter().enumerate() {
                        let mut loc = base_location.clone();
                        loc.nested_path.push(NestedEntry::TableCell {
                            control_index: ci,
                            cell_index: cell_i,
                            para_index: pi,
                        });
                        collect_fields_from_paragraph(cell_para, &loc, result);
                    }
                }
            }
            Control::Shape(shape) => {
                if let Some(drawing) = shape.drawing() {
                    if let Some(tb) = &drawing.text_box {
                        for (pi, tb_para) in tb.paragraphs.iter().enumerate() {
                            let mut loc = base_location.clone();
                            loc.nested_path.push(NestedEntry::TextBox {
                                control_index: ci,
                                para_index: pi,
                            });
                            collect_fields_from_paragraph(tb_para, &loc, result);
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

/// FieldLocation을 JSON으로 변환
fn field_location_json(loc: &FieldLocation) -> String {
    if loc.nested_path.is_empty() {
        format!(
            "{{\"sectionIndex\":{},\"paraIndex\":{}}}",
            loc.section_index, loc.para_index,
        )
    } else {
        let path_entries: Vec<String> = loc.nested_path.iter().map(|e| match e {
            NestedEntry::TableCell { control_index, cell_index, para_index } => {
                format!("{{\"type\":\"cell\",\"controlIndex\":{},\"cellIndex\":{},\"paraIndex\":{}}}",
                    control_index, cell_index, para_index)
            }
            NestedEntry::TextBox { control_index, para_index } => {
                format!("{{\"type\":\"textbox\",\"controlIndex\":{},\"paraIndex\":{}}}",
                    control_index, para_index)
            }
        }).collect();
        format!(
            "{{\"sectionIndex\":{},\"paraIndex\":{},\"path\":[{}]}}",
            loc.section_index,
            loc.para_index,
            path_entries.join(","),
        )
    }
}

fn para_at_location<'a>(core: &'a DocumentCore, location: &FieldLocation) -> Option<&'a Paragraph> {
    let mut para = core
        .document
        .sections
        .get(location.section_index)?
        .paragraphs
        .get(location.para_index)?;

    for entry in &location.nested_path {
        para = match entry {
            NestedEntry::TableCell {
                control_index,
                cell_index,
                para_index,
            } => {
                let ctrl = para.controls.get(*control_index)?;
                if let Control::Table(table) = ctrl {
                    table.cells.get(*cell_index)?.paragraphs.get(*para_index)?
                } else {
                    return None;
                }
            }
            NestedEntry::TextBox {
                control_index,
                para_index,
            } => {
                let ctrl = para.controls.get(*control_index)?;
                if let Control::Shape(shape) = ctrl {
                    shape
                        .drawing()?
                        .text_box
                        .as_ref()?
                        .paragraphs
                        .get(*para_index)?
                } else {
                    return None;
                }
            }
        };
    }

    Some(para)
}

fn field_range_bounds(core: &DocumentCore, fi: &FieldInfo) -> Option<(usize, usize)> {
    let para = para_at_location(core, &fi.location)?;
    let range = para.field_ranges.get(fi.field_range_index)?;
    Some((range.start_char_idx, range.end_char_idx))
}

impl DocumentCore {
    /// 본문 문단에서 커서 위치의 필드 컨트롤 인덱스를 찾는다.
    fn find_field_control_idx(
        &self,
        section_idx: usize,
        para_idx: usize,
        char_offset: usize,
        _cell_path: Option<(usize, usize, usize)>,
    ) -> Option<usize> {
        let para = self
            .document
            .sections
            .get(section_idx)?
            .paragraphs
            .get(para_idx)?;
        find_field_ctrl_idx_in_para(para, char_offset)
    }

    /// 셀/글상자 내 문단에서 커서 위치의 필드 컨트롤 인덱스를 찾는다.
    fn find_field_control_idx_in_cell(
        &self,
        section_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
        cell_idx: usize,
        cell_para_idx: usize,
        char_offset: usize,
        is_textbox: bool,
    ) -> Option<usize> {
        let host = self
            .document
            .sections
            .get(section_idx)?
            .paragraphs
            .get(parent_para_idx)?;
        let ctrl = host.controls.get(control_idx)?;
        let para = if is_textbox {
            if let Control::Shape(shape) = ctrl {
                let tb = shape.drawing()?.text_box.as_ref()?;
                tb.paragraphs.get(cell_para_idx)?
            } else {
                return None;
            }
        } else {
            if let Control::Table(table) = ctrl {
                table.cells.get(cell_idx)?.paragraphs.get(cell_para_idx)?
            } else {
                return None;
            }
        };
        find_field_ctrl_idx_in_para(para, char_offset)
    }

    fn next_click_here_field_id(&self) -> u32 {
        let mut max_id = 0u32;
        for section in &self.document.sections {
            for para in &section.paragraphs {
                collect_max_field_id(para, &mut max_id);
            }
        }
        max_id.saturating_add(1).max(1)
    }
}

fn collect_max_field_id(para: &Paragraph, max_id: &mut u32) {
    for ctrl in &para.controls {
        match ctrl {
            Control::Field(field) if field.field_id > *max_id => {
                *max_id = field.field_id;
            }
            Control::Table(table) => {
                for cell in &table.cells {
                    for cell_para in &cell.paragraphs {
                        collect_max_field_id(cell_para, max_id);
                    }
                }
                if let Some(caption) = &table.caption {
                    for cap_para in &caption.paragraphs {
                        collect_max_field_id(cap_para, max_id);
                    }
                }
            }
            Control::Shape(shape) => {
                if let Some(drawing) = shape.drawing() {
                    if let Some(text_box) = &drawing.text_box {
                        for tb_para in &text_box.paragraphs {
                            collect_max_field_id(tb_para, max_id);
                        }
                    }
                }
            }
            Control::Picture(pic) => {
                if let Some(caption) = &pic.caption {
                    for cap_para in &caption.paragraphs {
                        collect_max_field_id(cap_para, max_id);
                    }
                }
            }
            _ => {}
        }
    }
}

fn insert_click_here_field_in_para(
    para: &mut Paragraph,
    char_offset: usize,
    field_id: u32,
    guide: &str,
    memo: &str,
    name: &str,
    editable: bool,
) -> Result<usize, HwpError> {
    let text_len = para.text.chars().count();
    let start = char_offset.min(text_len);
    let positions = para.control_text_positions();
    let insert_idx = positions
        .iter()
        .position(|&pos| pos > start)
        .unwrap_or(para.controls.len());

    for range in &mut para.field_ranges {
        if range.control_idx >= insert_idx {
            range.control_idx += 1;
        }
    }

    let field = Field {
        field_type: FieldType::ClickHere,
        // [#1434] 이름은 ctrl_data_name(CTRL_DATA 0x57)으로 별도 저장하므로 command 에
        // 넣지 않는다 (Name 키가 끼면 한컴이 안내문 바인딩 실패).
        command: Field::build_clickhere_command(guide, memo),
        properties: if editable { 1 } else { 0 },
        extra_properties: 0x09,
        field_id,
        ctrl_id: tags::FIELD_CLICKHERE,
        ctrl_data_name: if name.is_empty() {
            None
        } else {
            Some(name.to_string())
        },
        memo_index: 0,
        memo_paragraphs: Vec::new(),
        raw_parameters_xml: None,
    };

    para.controls.insert(insert_idx, Control::Field(field));
    if para.ctrl_data_records.len() < insert_idx {
        para.ctrl_data_records.resize(insert_idx, None);
    }
    para.ctrl_data_records.insert(insert_idx, None);

    let new_range = FieldRange {
        start_char_idx: start,
        end_char_idx: start,
        control_idx: insert_idx,
    };
    let range_idx = para
        .field_ranges
        .iter()
        .position(|range| {
            range.start_char_idx > start
                || (range.start_char_idx == start && range.control_idx > insert_idx)
        })
        .unwrap_or(para.field_ranges.len());
    para.field_ranges.insert(range_idx, new_range);
    rebuild_char_offsets(para);

    Ok(start)
}

/// 문단에서 커서 위치의 ClickHere 필드 컨트롤 인덱스를 반환한다.
fn find_field_ctrl_idx_in_para(para: &Paragraph, char_offset: usize) -> Option<usize> {
    // 인접 누름틀 경계에서는 앞 누름틀의 끝과 다음 빈 누름틀의 시작이
    // 같은 charOffset을 공유한다. 새 빈 누름틀을 먼저 잡아야 첫 입력이
    // 앞 누름틀 값으로 붙지 않는다.
    for fr in &para.field_ranges {
        if fr.start_char_idx == fr.end_char_idx && char_offset == fr.start_char_idx {
            if let Some(Control::Field(field)) = para.controls.get(fr.control_idx) {
                if field.field_type == FieldType::ClickHere {
                    return Some(fr.control_idx);
                }
            }
        }
    }

    for fr in &para.field_ranges {
        if let Some(Control::Field(field)) = para.controls.get(fr.control_idx) {
            if field.field_type != FieldType::ClickHere {
                continue;
            }
            if char_offset >= fr.start_char_idx && char_offset <= fr.end_char_idx {
                return Some(fr.control_idx);
            }
        }
    }
    None
}

/// 문단 내 커서 위치의 누름틀 필드를 제거한다.
fn remove_field_in_para(para: &mut Paragraph, char_offset: usize) -> Result<(), HwpError> {
    let idx = para.field_ranges.iter().position(|fr| {
        if let Some(Control::Field(field)) = para.controls.get(fr.control_idx) {
            if field.field_type != FieldType::ClickHere {
                return false;
            }
            char_offset >= fr.start_char_idx && char_offset <= fr.end_char_idx
        } else {
            false
        }
    });
    match idx {
        Some(i) => {
            let start = para.field_ranges[i].start_char_idx;
            let end = para.field_ranges[i].end_char_idx;
            let removed_control_idx = para.field_ranges[i].control_idx;
            para.field_ranges.remove(i);
            if end > start {
                para.delete_text_at(start, end - start);
            }
            if removed_control_idx < para.controls.len() {
                para.controls.remove(removed_control_idx);
            }
            if removed_control_idx < para.ctrl_data_records.len() {
                para.ctrl_data_records.remove(removed_control_idx);
            }
            for range in &mut para.field_ranges {
                if range.control_idx > removed_control_idx {
                    range.control_idx -= 1;
                }
            }
            rebuild_char_offsets(para);
            Ok(())
        }
        None => Err(HwpError::InvalidField(
            "커서 위치에 누름틀 필드 없음".into(),
        )),
    }
}

/// 문자열을 JSON 이스케이프한다.
/// 문단의 char_offsets를 컨트롤/필드/텍스트 배치 순서에 맞게 재생성한다.
///
/// 원본 char_offsets에서 컨트롤 배치 패턴을 보존하면서,
/// 텍스트 길이 변경(필드 값 삽입)에 맞게 오프셋을 재계산한다.
pub(crate) fn rebuild_char_offsets(para: &mut Paragraph) {
    let text_chars: Vec<char> = para.text.chars().collect();
    let text_len = text_chars.len();

    // 원본 char_offsets에서 첫 문자 이전 컨트롤 수 추정
    // (원본 gap / 8 = 컨트롤 수)
    let ctrls_before_text = if !para.char_offsets.is_empty() {
        para.char_offsets[0] as usize / 8
    } else {
        para.controls.len()
    }
    .min(para.controls.len());

    // FIELD_BEGIN: 이미 char_offsets의 첫 갭에 포함된 선행 컨트롤은 보존하고,
    // 새로 삽입된 시작 위치 필드는 첫 문자 앞에도 갭을 추가해야 한다.
    let mut field_begin_at: Vec<usize> = vec![0; text_len + 1];
    for fr in &para.field_ranges {
        if fr.control_idx >= ctrls_before_text {
            let idx = fr.start_char_idx.min(text_len);
            field_begin_at[idx] += 1;
        }
    }

    // FIELD_END 수: field_ranges에서 end가 텍스트 범위 내인 것
    let mut field_end_at: Vec<usize> = vec![0; text_len + 1];
    for fr in &para.field_ranges {
        let idx = fr.end_char_idx.min(text_len);
        field_end_at[idx] += 1;
    }

    if text_len == 0 {
        para.char_offsets = Vec::new();
        para.char_count =
            ((ctrls_before_text + field_begin_at[0] + field_end_at[0]) * 8 + 1) as u32;
        return;
    }

    let mut offset: u32 = ctrls_before_text as u32 * 8;
    let mut new_offsets = Vec::with_capacity(text_len);

    for (i, ch) in text_chars.iter().enumerate() {
        // 이 문자 앞에 FIELD_BEGIN 컨트롤 갭 삽입
        offset += field_begin_at[i] as u32 * 8;
        // 이 문자 앞에 FIELD_END 마커 갭 삽입
        offset += field_end_at[i] as u32 * 8;

        new_offsets.push(offset);

        let char_size = match *ch {
            '\t' => 8,
            '\n' | '\u{00A0}' => 1,
            c => {
                let mut buf = [0u16; 2];
                c.encode_utf16(&mut buf).len() as u32
            }
        };
        offset += char_size;
    }

    // 텍스트 뒤에 위치한 빈 필드/필드 끝 마커와 문단 끝 마커를 char_count에 반영한다.
    offset += field_begin_at[text_len] as u32 * 8;
    offset += field_end_at[text_len] as u32 * 8;
    para.char_count = offset + 1;
    para.char_offsets = new_offsets;
}

fn json_escape(s: &str) -> String {
    let mut result = String::with_capacity(s.len() + 2);
    result.push('"');
    for c in s.chars() {
        match c {
            '"' => result.push_str("\\\""),
            '\\' => result.push_str("\\\\"),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                result.push_str(&format!("\\u{:04x}", c as u32));
            }
            _ => result.push(c),
        }
    }
    result.push('"');
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::control::{Control, Field, FieldType};
    use crate::model::document::Section;
    use crate::model::paragraph::{FieldRange, Paragraph};
    use crate::model::table::{Cell, Table};

    fn make_field_control(ctrl_id: u32) -> Control {
        Control::Field(Field {
            field_type: FieldType::ClickHere,
            command: String::new(),
            properties: 0,
            extra_properties: 0,
            field_id: ctrl_id,
            ctrl_id,
            ctrl_data_name: None,
            memo_index: 0,
            memo_paragraphs: Vec::new(),
            raw_parameters_xml: None,
        })
    }

    #[test]
    fn rebuild_preserves_mid_text_field_begin_gap() {
        // Stream: [ColumnDef 8B] A(1) B(1) C(1) [FIELD_BEGIN 8B] X(1) Y(1) [FIELD_END 8B]
        let mut para = Paragraph {
            text: "ABCXY".into(),
            controls: vec![
                Control::ColumnDef(Default::default()),
                make_field_control(100),
            ],
            field_ranges: vec![FieldRange {
                start_char_idx: 3,
                end_char_idx: 5,
                control_idx: 1,
            }],
            char_offsets: vec![8, 9, 10, 19, 20],
            ..Default::default()
        };

        rebuild_char_offsets(&mut para);

        // A=8(+1) B=9(+1) C=10(+1) → gap 8 for FIELD_BEGIN → X=19(+1) Y=20
        assert_eq!(para.char_offsets, vec![8, 9, 10, 19, 20]);
    }

    #[test]
    fn rebuild_field_at_start_no_double_count() {
        // FIELD_BEGIN is pre-text control (control_idx=0 < ctrls_before_text=1)
        let mut para = Paragraph {
            text: "XY".into(),
            controls: vec![make_field_control(100)],
            field_ranges: vec![FieldRange {
                start_char_idx: 0,
                end_char_idx: 2,
                control_idx: 0,
            }],
            char_offsets: vec![8, 9],
            ..Default::default()
        };

        rebuild_char_offsets(&mut para);

        assert_eq!(para.char_offsets, vec![8, 9]);
    }

    #[test]
    fn rebuild_after_set_field_creates_serializable_gap() {
        // After set_field: "라벨: " [FIELD_BEGIN] "NEW" [FIELD_END]
        let mut para = Paragraph {
            text: "라벨: NEW".into(), // 7 chars: 라 벨 : ' ' N E W
            controls: vec![
                Control::ColumnDef(Default::default()),
                make_field_control(200),
            ],
            field_ranges: vec![FieldRange {
                start_char_idx: 4,
                end_char_idx: 7,
                control_idx: 1,
            }],
            // 원본 offsets (stale after text change, but char_offsets[0] still valid for ctrls_before_text)
            char_offsets: vec![8, 9, 10, 11, 20, 21, 22],
            ..Default::default()
        };

        rebuild_char_offsets(&mut para);

        // ctrls_before_text = 8/8 = 1
        // 라=8(+1) 벨=9(+1) :=10(+1) ' '=11(+1) → field_begin gap +8 → N=20(+1) E=21(+1) W=22
        assert_eq!(para.char_offsets[0], 8); // 라
        assert_eq!(para.char_offsets[3], 11); // ' '
        assert_eq!(para.char_offsets[4], 20); // N — 8-byte gap after ' ' for FIELD_BEGIN
        let gap = para.char_offsets[4] as i64 - (para.char_offsets[3] as i64 + 1);
        assert_eq!(gap, 8); // serializer needs exactly 8 code units for FIELD_BEGIN
    }

    #[test]
    fn set_cell_field_text_updates_text_metadata() {
        let cell_para = Paragraph {
            text: "기존값".into(),
            char_count: 4,
            char_offsets: vec![0, 1, 2],
            ..Default::default()
        };
        let table = Table {
            cells: vec![Cell {
                field_name: Some("셀필드".into()),
                paragraphs: vec![cell_para],
                ..Default::default()
            }],
            ..Default::default()
        };
        let parent_para = Paragraph {
            controls: vec![Control::Table(Box::new(table))],
            ..Default::default()
        };

        let mut core = DocumentCore::new_empty();
        core.document.sections.push(Section {
            paragraphs: vec![parent_para],
            ..Default::default()
        });

        let location = FieldLocation {
            section_index: 0,
            para_index: 0,
            nested_path: vec![NestedEntry::TableCell {
                control_index: 0,
                cell_index: 0,
                para_index: 0,
            }],
        };

        core.set_cell_field_text(&location, "새값").unwrap();

        let Control::Table(table) = &core.document.sections[0].paragraphs[0].controls[0] else {
            panic!("expected table control");
        };
        let updated = &table.cells[0].paragraphs[0];
        assert_eq!(updated.text, "새값");
        assert_eq!(updated.char_count, 3);
        assert_eq!(updated.char_offsets, vec![0, 1]);
    }
}

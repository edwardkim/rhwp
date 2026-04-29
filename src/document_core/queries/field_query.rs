//! 필드 조회/설정 API (Task 230)
//!
//! 문서 전체에서 필드를 재귀 탐색하여 조회·설정하는 기능을 제공한다.

use crate::document_core::DocumentCore;
use crate::model::control::{Control, Field, FieldType};
use crate::model::paragraph::Paragraph;
use crate::error::HwpError;

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
    TableCell { control_index: usize, cell_index: usize, para_index: usize },
    /// 글상자: (control_index, para_index)
    TextBox { control_index: usize, para_index: usize },
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

    /// getFieldList: 모든 필드를 JSON 배열로 반환
    pub fn get_field_list_json(&self) -> String {
        let fields = self.collect_all_fields();
        let entries: Vec<String> = fields.iter().map(|fi| {
            let name = fi.field.field_name().unwrap_or("");
            let guide = fi.field.guide_text().unwrap_or("");
            let location_json = field_location_json(&fi.location);
            format!(
                "{{\"fieldId\":{},\"fieldType\":\"{}\",\"name\":{},\"guide\":{},\"command\":{},\"value\":{},\"location\":{}}}",
                fi.field.field_id,
                fi.field.field_type_str(),
                json_escape(name),
                json_escape(guide),
                json_escape(&fi.field.command),
                json_escape(&fi.value),
                location_json,
            )
        }).collect();
        format!("[{}]", entries.join(","))
    }

    /// getFieldValue: field_id로 필드 값 조회
    pub fn get_field_value_by_id(&self, field_id: u32) -> Result<String, HwpError> {
        let fields = self.collect_all_fields();
        for fi in &fields {
            if fi.field.field_id == field_id {
                return Ok(format!("{{\"ok\":true,\"value\":{}}}", json_escape(&fi.value)));
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
    pub fn set_field_value_by_id(&mut self, field_id: u32, value: &str) -> Result<String, HwpError> {
        // 먼저 필드 위치 찾기
        let fields = self.collect_all_fields();
        let fi = fields.iter().find(|f| f.field.field_id == field_id)
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
        let fi = fields.iter().find(|f| {
            f.field.field_name().map(|n| n == name).unwrap_or(false)
        }).ok_or_else(|| HwpError::InvalidField(format!("필드 이름 '{}' 없음", name)))?;

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
    fn set_cell_field_text(&mut self, location: &FieldLocation, value: &str) -> Result<(), HwpError> {
        if location.nested_path.is_empty() {
            return Err(HwpError::InvalidField("셀 필드 위치에 중첩 경로 없음".into()));
        }
        let entry = &location.nested_path[0];
        match entry {
            NestedEntry::TableCell { control_index, cell_index, .. } => {
                let sec = self.document.sections.get_mut(location.section_index)
                    .ok_or_else(|| HwpError::InvalidField("구역 초과".into()))?;
                let para = sec.paragraphs.get_mut(location.para_index)
                    .ok_or_else(|| HwpError::InvalidField("문단 초과".into()))?;
                let table = match para.controls.get_mut(*control_index) {
                    Some(Control::Table(t)) => t,
                    _ => return Err(HwpError::InvalidField("컨트롤이 표가 아님".into())),
                };
                let cell = table.cells.get_mut(*cell_index)
                    .ok_or_else(|| HwpError::InvalidField("셀 인덱스 초과".into()))?;
                // 첫 문단의 텍스트를 교체
                if let Some(cell_para) = cell.paragraphs.first_mut() {
                    cell_para.text = value.to_string();
                    // char_offsets 재생성
                    let new_len = value.chars().count();
                    cell_para.char_offsets = (0..new_len).map(|i| i as u32).collect();
                }
                Ok(())
            }
            _ => Err(HwpError::InvalidField("셀 필드가 아닌 위치".into())),
        }
    }

    /// 필드 위치에서 텍스트를 교체한다.
    fn set_field_text_at(&mut self, location: &FieldLocation, field_range_index: usize, value: &str) -> Result<(), HwpError> {
        // raw_stream 무효화: 직렬화 시 수정된 모델을 사용하도록 강제
        if let Some(sec) = self.document.sections.get_mut(location.section_index) {
            sec.raw_stream = None;
        }
        let para = self.get_para_mut_at_location(location)?;
        let fr = para.field_ranges.get(field_range_index)
            .ok_or_else(|| HwpError::InvalidField("field_range 인덱스 초과".into()))?
            .clone();

        // 필드 범위 내 텍스트 교체
        let text_chars: Vec<char> = para.text.chars().collect();
        let before: String = text_chars[..fr.start_char_idx].iter().collect();
        let after: String = text_chars[fr.end_char_idx..].iter().collect();
        para.text = format!("{}{}{}", before, value, after);

        // field_range 업데이트
        let new_end = fr.start_char_idx + value.chars().count();
        let delta = new_end as isize - fr.end_char_idx as isize;

        // 현재 필드 범위 업데이트
        if let Some(current_fr) = para.field_ranges.get_mut(field_range_index) {
            current_fr.end_char_idx = new_end;
        }

        // 이후 필드 범위들의 위치 조정
        for (i, other_fr) in para.field_ranges.iter_mut().enumerate() {
            if i == field_range_index {
                continue;
            }
            if other_fr.start_char_idx >= fr.end_char_idx {
                other_fr.start_char_idx = (other_fr.start_char_idx as isize + delta) as usize;
                other_fr.end_char_idx = (other_fr.end_char_idx as isize + delta) as usize;
            }
        }

        // char_offsets 재생성: 컨트롤 문자(8 code unit)와 일반 문자(1~2 code unit) 반영
        rebuild_char_offsets(para);

        Ok(())
    }

    /// FieldLocation에 해당하는 Paragraph의 가변 참조를 반환한다.
    ///
    /// 중첩 경로는 1단계만 지원 (표 셀 또는 글상자 내 문단).
    fn get_para_mut_at_location(&mut self, location: &FieldLocation) -> Result<&mut Paragraph, HwpError> {
        let sec = self.document.sections.get_mut(location.section_index)
            .ok_or_else(|| HwpError::InvalidField("구역 인덱스 초과".into()))?;
        let host_para = sec.paragraphs.get_mut(location.para_index)
            .ok_or_else(|| HwpError::InvalidField("문단 인덱스 초과".into()))?;

        if location.nested_path.is_empty() {
            return Ok(host_para);
        }

        // 1단계 중첩만 처리
        let entry = &location.nested_path[0];
        match entry {
            NestedEntry::TableCell { control_index, cell_index, para_index } => {
                let ctrl = host_para.controls.get_mut(*control_index)
                    .ok_or_else(|| HwpError::InvalidField("컨트롤 인덱스 초과".into()))?;
                if let Control::Table(ref mut table) = ctrl {
                    let cell = table.cells.get_mut(*cell_index)
                        .ok_or_else(|| HwpError::InvalidField("셀 인덱스 초과".into()))?;
                    cell.paragraphs.get_mut(*para_index)
                        .ok_or_else(|| HwpError::InvalidField("셀 문단 인덱스 초과".into()))
                } else {
                    Err(HwpError::InvalidField("예상된 Table 컨트롤이 아님".into()))
                }
            }
            NestedEntry::TextBox { control_index, para_index } => {
                let ctrl = host_para.controls.get_mut(*control_index)
                    .ok_or_else(|| HwpError::InvalidField("컨트롤 인덱스 초과".into()))?;
                if let Control::Shape(ref mut shape) = ctrl {
                    let drawing = shape.drawing_mut()
                        .ok_or_else(|| HwpError::InvalidField("Shape에 DrawingObjAttr 없음".into()))?;
                    let tb = drawing.text_box.as_mut()
                        .ok_or_else(|| HwpError::InvalidField("Shape에 TextBox 없음".into()))?;
                    tb.paragraphs.get_mut(*para_index)
                        .ok_or_else(|| HwpError::InvalidField("글상자 문단 인덱스 초과".into()))
                } else {
                    Err(HwpError::InvalidField("예상된 Shape 컨트롤이 아님".into()))
                }
            }
        }
    }

    /// 본문 문단의 커서 위치에서 필드를 제거한다 (텍스트 유지, 필드 마커만 삭제).
    ///
    /// 성공 시 `{"ok":true}`, 필드가 없으면 에러를 반환한다.
    pub fn remove_field_at(&mut self, section_idx: usize, para_idx: usize, char_offset: usize) -> Result<String, HwpError> {
        let para = self.document.sections.get_mut(section_idx)
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
            let host = self.document.sections.get_mut(section_idx)
                .and_then(|s| s.paragraphs.get_mut(parent_para_idx))
                .ok_or_else(|| HwpError::InvalidField("호스트 문단 위치 초과".into()))?;
            let ctrl = host.controls.get_mut(control_idx)
                .ok_or_else(|| HwpError::InvalidField("컨트롤 인덱스 초과".into()))?;
            if is_textbox {
                if let Control::Shape(shape) = ctrl {
                    let drawing = shape.drawing_mut()
                        .ok_or_else(|| HwpError::InvalidField("Shape에 DrawingObjAttr 없음".into()))?;
                    let tb = drawing.text_box.as_mut()
                        .ok_or_else(|| HwpError::InvalidField("Shape에 TextBox 없음".into()))?;
                    tb.paragraphs.get_mut(cell_para_idx)
                        .ok_or_else(|| HwpError::InvalidField("글상자 문단 인덱스 초과".into()))?
                } else {
                    return Err(HwpError::InvalidField("예상된 Shape 컨트롤이 아님".into()));
                }
            } else {
                if let Control::Table(table) = ctrl {
                    let cell = table.cells.get_mut(cell_idx)
                        .ok_or_else(|| HwpError::InvalidField("셀 인덱스 초과".into()))?;
                    cell.paragraphs.get_mut(cell_para_idx)
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
    pub fn set_active_field(&mut self, section_idx: usize, para_idx: usize, char_offset: usize) -> bool {
        use super::super::ActiveFieldInfo;
        let ctrl_idx = self.find_field_control_idx(section_idx, para_idx, char_offset, None);
        if let Some(ci) = ctrl_idx {
            let new_info = ActiveFieldInfo {
                section_idx, para_idx, control_idx: ci, cell_path: None,
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
        &mut self, section_idx: usize, parent_para_idx: usize, control_idx: usize,
        cell_idx: usize, cell_para_idx: usize, char_offset: usize, is_textbox: bool,
    ) -> bool {
        use super::super::ActiveFieldInfo;
        let cell_path = Some(vec![(control_idx, cell_idx, cell_para_idx)]);
        let ctrl_idx = self.find_field_control_idx_in_cell(
            section_idx, parent_para_idx, control_idx, cell_idx, cell_para_idx, char_offset, is_textbox,
        );
        if let Some(ci) = ctrl_idx {
            let new_info = ActiveFieldInfo {
                section_idx, para_idx: cell_para_idx, control_idx: ci, cell_path,
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
    pub fn get_field_info_at(&self, section_idx: usize, para_idx: usize, char_offset: usize) -> String {
        let para = match self.document.sections.get(section_idx)
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
            let host = self.document.sections.get(section_idx)?
                .paragraphs.get(parent_para_idx)?;
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
        &self, section_idx: usize, parent_para_idx: usize,
        path: &[(usize, usize, usize)], char_offset: usize,
    ) -> String {
        match self.resolve_paragraph_by_path(section_idx, parent_para_idx, path) {
            Ok(para) => field_info_at_in_para(para, char_offset),
            Err(_) => r#"{"inField":false}"#.to_string(),
        }
    }

    /// path 기반: 중첩 표 셀 내 활성 필드를 설정한다.
    pub fn set_active_field_by_path(
        &mut self, section_idx: usize, parent_para_idx: usize,
        path: &[(usize, usize, usize)], char_offset: usize,
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
                section_idx, para_idx: cell_para_idx, control_idx: ci, cell_path,
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
                    "{{\"inField\":true,\"fieldId\":{},\"fieldType\":\"{}\",\"startCharIdx\":{},\"endCharIdx\":{},\"isGuide\":{},\"guideName\":{}}}",
                    field.field_id,
                    field.field_type_str(),
                    fr.start_char_idx,
                    fr.end_char_idx,
                    is_guide,
                    json_escape(guide),
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
                        let value = cell.paragraphs.first()
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
            loc.section_index, loc.para_index, path_entries.join(","),
        )
    }
}

impl DocumentCore {
    /// 본문 문단에서 커서 위치의 필드 컨트롤 인덱스를 찾는다.
    fn find_field_control_idx(
        &self, section_idx: usize, para_idx: usize, char_offset: usize,
        _cell_path: Option<(usize, usize, usize)>,
    ) -> Option<usize> {
        let para = self.document.sections.get(section_idx)?
            .paragraphs.get(para_idx)?;
        find_field_ctrl_idx_in_para(para, char_offset)
    }

    /// 셀/글상자 내 문단에서 커서 위치의 필드 컨트롤 인덱스를 찾는다.
    fn find_field_control_idx_in_cell(
        &self, section_idx: usize, parent_para_idx: usize, control_idx: usize,
        cell_idx: usize, cell_para_idx: usize, char_offset: usize, is_textbox: bool,
    ) -> Option<usize> {
        let host = self.document.sections.get(section_idx)?
            .paragraphs.get(parent_para_idx)?;
        let ctrl = host.controls.get(control_idx)?;
        let para = if is_textbox {
            if let Control::Shape(shape) = ctrl {
                let tb = shape.drawing()?.text_box.as_ref()?;
                tb.paragraphs.get(cell_para_idx)?
            } else { return None; }
        } else {
            if let Control::Table(table) = ctrl {
                table.cells.get(cell_idx)?.paragraphs.get(cell_para_idx)?
            } else { return None; }
        };
        find_field_ctrl_idx_in_para(para, char_offset)
    }
}

/// 문단에서 커서 위치의 ClickHere 필드 컨트롤 인덱스를 반환한다.
fn find_field_ctrl_idx_in_para(para: &Paragraph, char_offset: usize) -> Option<usize> {
    for fr in &para.field_ranges {
        if let Some(Control::Field(field)) = para.controls.get(fr.control_idx) {
            if field.field_type != FieldType::ClickHere { continue; }
            if char_offset >= fr.start_char_idx && char_offset <= fr.end_char_idx {
                return Some(fr.control_idx);
            }
        }
    }
    None
}

/// 문단 내 커서 위치의 누름틀 필드를 제거한다 (FieldRange만 삭제, 텍스트 유지).
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
            para.field_ranges.remove(i);
            Ok(())
        }
        None => Err(HwpError::InvalidField("커서 위치에 누름틀 필드 없음".into())),
    }
}

/// 문자열을 JSON 이스케이프한다.
/// 문단의 char_offsets를 컨트롤/필드/텍스트 배치 순서에 맞게 재생성한다.
///
/// 원본 char_offsets에서 컨트롤 배치 패턴을 보존하면서,
/// 텍스트 길이 변경(필드 값 삽입)에 맞게 오프셋을 재계산한다.
fn rebuild_char_offsets(para: &mut Paragraph) {
    let text_chars: Vec<char> = para.text.chars().collect();
    let text_len = text_chars.len();

    if text_len == 0 {
        para.char_offsets = Vec::new();
        return;
    }

    // 원본 char_offsets에서 첫 문자 이전 컨트롤 수 추정
    // (원본 gap / 8 = 컨트롤 수)
    let ctrls_before_text = if !para.char_offsets.is_empty() {
        para.char_offsets[0] as usize / 8
    } else {
        para.controls.len()
    };

    // FIELD_BEGIN: control_idx >= ctrls_before_text이고 start > 0인 필드의 시작 위치에 갭 필요
    let mut field_begin_at: Vec<usize> = vec![0; text_len + 1];
    for fr in &para.field_ranges {
        if fr.control_idx >= ctrls_before_text && fr.start_char_idx > 0 {
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
    use crate::model::paragraph::{FieldRange, Paragraph};

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
        assert_eq!(para.char_offsets[0], 8);  // 라
        assert_eq!(para.char_offsets[3], 11); // ' '
        assert_eq!(para.char_offsets[4], 20); // N — 8-byte gap after ' ' for FIELD_BEGIN
        let gap = para.char_offsets[4] as i64 - (para.char_offsets[3] as i64 + 1);
        assert_eq!(gap, 8); // serializer needs exactly 8 code units for FIELD_BEGIN
    }
}

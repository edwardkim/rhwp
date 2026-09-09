//! 본문·표 셀·글상자 단일 문단의 텍스트 하이퍼링크 편집 계약 (#6963).
//! offset은 UTF-16/바이트가 아닌 Unicode scalar 인덱스이며 범위는 [start, end)다.
//! 개체를 감싼 필드·다단락 필드·캡션은 이 API의 대상이 아니다.

use super::DocumentCore;
use crate::error::HwpError;
use crate::model::control::{Control, Field, FieldType, Parameter};
use crate::model::event::DocumentEvent;
use crate::model::hyperlink::{command_uri, web_command};
use crate::model::paragraph::{FieldRange, Paragraph};
use crate::parser::tags;

/// 문단 주소. 빈 cell_path는 본문이며 각 경로 항목은 (컨트롤, 셀, 내부 문단)이다.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HyperlinkTarget {
    pub section: usize,
    pub para: usize,
    pub cell_path: Vec<(usize, usize, usize)>,
}

impl HyperlinkTarget {
    pub fn body(section: usize, para: usize) -> Self {
        Self {
            section,
            para,
            cell_path: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HyperlinkInfo {
    pub field_id: u32,
    pub start: usize,
    pub end: usize,
    pub text: String,
    pub uri: String,
}

impl DocumentCore {
    /// 지정 문단의 링크를 조회한다. 미지원 scheme도 원문 주소 그대로 반환한다.
    pub fn hyperlinks_native(
        &self,
        target: &HyperlinkTarget,
    ) -> Result<Vec<HyperlinkInfo>, HwpError> {
        let p = self.hyperlink_paragraph(target)?;
        Ok(p.field_ranges
            .iter()
            .filter_map(|range| {
                let Control::Field(field) = p.controls.get(range.control_idx)? else {
                    return None;
                };
                (field.field_type == FieldType::Hyperlink).then(|| HyperlinkInfo {
                    field_id: field.field_id,
                    start: range.start_char_idx,
                    end: range.end_char_idx,
                    text: p
                        .text
                        .chars()
                        .skip(range.start_char_idx)
                        .take(range.end_char_idx.saturating_sub(range.start_char_idx))
                        .collect(),
                    uri: command_uri(&field.command),
                })
            })
            .collect())
    }

    /// 선택한 표시 문자열에 웹 링크를 붙인다. 텍스트·글자 서식은 보존한다.
    /// 겹치는 필드와 빈 범위는 오류다. 무선택 삽입은 host가 텍스트 삽입과 함께 한 snapshot으로 묶는다.
    pub fn insert_hyperlink_native(
        &mut self,
        target: &HyperlinkTarget,
        start: usize,
        end: usize,
        uri: &str,
    ) -> Result<u32, HwpError> {
        let command = web_command(uri)?;
        self.validate_hyperlink_paragraph(target)?;
        let mut candidate = self.hyperlink_paragraph(target)?.clone();
        let len = candidate.text.chars().count();
        if start >= end || end > len {
            return Err(invalid(
                "하이퍼링크는 문단 안의 비어 있지 않은 범위가 필요합니다",
            ));
        }
        if candidate.field_ranges.iter().any(|r| {
            start < r.end_char_idx && r.start_char_idx < end
                || (r.start_char_idx == r.end_char_idx
                    && start <= r.start_char_idx
                    && r.start_char_idx < end)
        }) {
            return Err(invalid("기존 필드와 겹치는 하이퍼링크는 지원하지 않습니다"));
        }
        let mut max_id = 0;
        for s in &self.document.sections {
            for p in &s.paragraphs {
                super::queries::field_query::collect_max_field_id(p, &mut max_id);
            }
        }
        let field_id = max_id
            .checked_add(1)
            .ok_or_else(|| invalid("필드 ID 공간이 소진되었습니다"))?;
        // FieldRange가 정확한 위치 정본이다. END 갭을 controls[] 슬롯으로 오인하지 않는다.
        let positions = candidate.control_text_positions();
        let insert_idx = candidate
            .controls
            .iter()
            .enumerate()
            .position(|(idx, _)| {
                candidate
                    .field_ranges
                    .iter()
                    .find(|r| r.control_idx == idx)
                    .map_or(positions[idx], |r| r.start_char_idx)
                    > start
            })
            .unwrap_or(candidate.controls.len());
        let end_raw = raw_boundary(&candidate, end);
        let start_raw = raw_boundary(&candidate, start);
        shift_axis(&mut candidate, end_raw, 8)?;
        shift_axis(&mut candidate, start_raw, 8)?;
        for r in &mut candidate.field_ranges {
            if r.control_idx >= insert_idx {
                r.control_idx += 1;
            }
        }
        candidate.controls.insert(
            insert_idx,
            Control::Field(Field {
                field_type: FieldType::Hyperlink,
                command,
                ctrl_id: tags::FIELD_HYPERLINK,
                field_id,
                ..Default::default()
            }),
        );
        candidate
            .ctrl_data_records
            .resize(candidate.controls.len() - 1, None);
        candidate.ctrl_data_records.insert(insert_idx, None);
        candidate.field_ranges.push(FieldRange {
            start_char_idx: start,
            end_char_idx: end,
            control_idx: insert_idx,
            ..Default::default()
        });
        candidate
            .field_ranges
            .sort_by_key(|r| (r.start_char_idx, r.control_idx));
        self.commit_hyperlink_paragraph(target, candidate, field_id)?;
        Ok(field_id)
    }

    /// 주소만 바꾼다. 동일 주소는 캐시/이벤트를 건드리지 않고 false를 반환한다.
    pub fn update_hyperlink_native(
        &mut self,
        target: &HyperlinkTarget,
        field_id: u32,
        uri: &str,
    ) -> Result<bool, HwpError> {
        let command = web_command(uri)?;
        self.validate_hyperlink_paragraph(target)?;
        let p = self.hyperlink_paragraph(target)?;
        let (_, idx) = find_link(p, field_id)?;
        let Control::Field(field) = &p.controls[idx] else {
            unreachable!()
        };
        // 기존 mail/file/internal 링크를 웹 링크로 암묵적으로 바꾸지 않는다.
        web_command(&command_uri(&field.command))?;
        if command_uri(&field.command) == uri {
            return Ok(false);
        }
        let mut candidate = p.clone();
        let Control::Field(field) = &mut candidate.controls[idx] else {
            unreachable!()
        };
        field.command = command.clone();
        field.raw_parameters_xml = None;
        let mut has_command = false;
        for item in &mut field.parameters.items {
            if let Parameter::String {
                name: Some(name),
                value,
                ..
            } = item
            {
                match name.as_str() {
                    "Command" => {
                        *value = command.clone();
                        has_command = true;
                    }
                    "Path" => *value = uri.to_string(),
                    _ => {}
                }
            }
        }
        if !has_command && !field.parameters.is_empty() {
            field.parameters.items.push(Parameter::String {
                name: Some("Command".into()),
                value: command,
                preserve_space: false,
            });
        }
        // HWP5 command는 CTRL_HEADER에 따로 저장된다. 해석하지 않는 CTRL_DATA는 보존한다.
        self.commit_hyperlink_paragraph(target, candidate, field_id)?;
        Ok(true)
    }

    /// 링크의 시작/끝 마커만 제거한다. 표시 문자열과 글자 서식은 남긴다.
    pub fn remove_hyperlink_native(
        &mut self,
        target: &HyperlinkTarget,
        field_id: u32,
    ) -> Result<(), HwpError> {
        self.validate_hyperlink_paragraph(target)?;
        let mut candidate = self.hyperlink_paragraph(target)?.clone();
        let (range_idx, ctrl_idx) = find_link(&candidate, field_id)?;
        let range = &candidate.field_ranges[range_idx];
        let end_raw = raw_boundary(&candidate, range.end_char_idx);
        let start_raw = if range.start_char_idx == range.end_char_idx {
            end_raw
                .checked_sub(8)
                .ok_or_else(|| invalid("빈 필드 마커 좌표 오류"))?
        } else {
            raw_boundary(&candidate, range.start_char_idx)
        };
        if start_raw < 8 || end_raw.saturating_sub(start_raw) < 8 {
            return Err(invalid("필드 마커 좌표가 올바르지 않습니다"));
        }
        shift_axis(&mut candidate, end_raw, -8)?;
        shift_axis(&mut candidate, start_raw, -8)?;
        candidate.field_ranges.remove(range_idx);
        candidate.controls.remove(ctrl_idx);
        if ctrl_idx < candidate.ctrl_data_records.len() {
            candidate.ctrl_data_records.remove(ctrl_idx);
        }
        for range in &mut candidate.field_ranges {
            if range.control_idx > ctrl_idx {
                range.control_idx -= 1;
            }
        }
        self.commit_hyperlink_paragraph(target, candidate, field_id)?;
        Ok(())
    }

    fn hyperlink_paragraph(&self, target: &HyperlinkTarget) -> Result<&Paragraph, HwpError> {
        let (paragraphs, idx) = self.hyperlink_paragraph_list(target)?;
        paragraphs
            .get(idx)
            .ok_or_else(|| invalid("문단 인덱스 초과"))
    }

    fn hyperlink_paragraph_list(
        &self,
        target: &HyperlinkTarget,
    ) -> Result<(&[Paragraph], usize), HwpError> {
        let mut paragraphs = self
            .document
            .sections
            .get(target.section)
            .ok_or_else(|| invalid("구역 인덱스 초과"))?
            .paragraphs
            .as_slice();
        let mut para_idx = target.para;
        for &(ctrl_idx, cell_idx, child_para) in &target.cell_path {
            let p = paragraphs
                .get(para_idx)
                .ok_or_else(|| invalid("상위 문단 인덱스 초과"))?;
            paragraphs = match p.controls.get(ctrl_idx) {
                Some(Control::Table(table)) => {
                    &table
                        .cells
                        .get(cell_idx)
                        .ok_or_else(|| invalid("셀 인덱스 초과"))?
                        .paragraphs
                }
                Some(Control::Shape(shape)) if cell_idx == 0 => {
                    &super::get_textbox_from_shape(shape)
                        .ok_or_else(|| invalid("글상자가 없는 도형"))?
                        .paragraphs
                }
                _ => return Err(invalid("지원하지 않는 셀/글상자 경로")),
            };
            para_idx = child_para;
        }
        Ok((paragraphs, para_idx))
    }

    fn validate_hyperlink_paragraph(&self, target: &HyperlinkTarget) -> Result<(), HwpError> {
        let p = self.hyperlink_paragraph(target)?;
        if p.field_ranges.iter().any(|r| {
            r.start_char_idx > r.end_char_idx
                || r.end_char_idx > p.text.chars().count()
                || r.control_idx >= p.controls.len()
        }) {
            return Err(invalid("필드 범위가 문단과 일치하지 않습니다"));
        }
        if self.document.header.version.major == 3
            || !p.orphan_field_ends.is_empty()
            || !p.title_marks.is_empty()
            || p.controls.iter().enumerate().any(|(idx, c)| match c {
                Control::SectionDef(_) | Control::ColumnDef(_) => false,
                Control::Field(_) => !p.field_ranges.iter().any(|r| {
                    r.control_idx == idx
                        && r.start_char_idx <= r.end_char_idx
                        && r.inner_slot_count == 0
                }),
                _ => true,
            })
            || p.char_offsets.len() != p.text.chars().count()
        {
            return Err(invalid(
                "개체·다단락 필드·HWP3 문단의 링크 편집은 아직 지원하지 않습니다",
            ));
        }
        // 앞 문단에서 열린 다단락 필드의 중간 문단도 편집 대상에서 제외한다.
        let mut open = std::collections::HashSet::new();
        let (paragraphs, para_idx) = self.hyperlink_paragraph_list(target)?;
        for preceding in &paragraphs[..para_idx] {
            for (idx, c) in preceding.controls.iter().enumerate() {
                if let Control::Field(f) = c {
                    if !preceding.field_ranges.iter().any(|r| r.control_idx == idx) {
                        open.insert(f.field_id);
                    }
                }
            }
            for end in &preceding.orphan_field_ends {
                open.remove(&end.begin_id_ref);
            }
        }
        if !open.is_empty() {
            return Err(invalid(
                "다단락 필드 안의 링크 편집은 아직 지원하지 않습니다",
            ));
        }
        Ok(())
    }

    fn commit_hyperlink_paragraph(
        &mut self,
        target: &HyperlinkTarget,
        candidate: Paragraph,
        field_id: u32,
    ) -> Result<(), HwpError> {
        let section = target.section;
        let para = target.para;
        let axis_changed = candidate.stored_text_partition_dirty;
        if target.cell_path.is_empty() {
            let stored_end = crate::renderer::composer::paragraph_flow_end(
                &self.document.sections[section].paragraphs[para],
            );
            self.document.sections[section].paragraphs[para] = candidate;
            if axis_changed {
                self.reflow_paragraph(section, para);
                let hwp3_layout = self.document.layout_profile().hwp3_layout();
                crate::renderer::composer::recalculate_section_vpos(
                    &mut self.document.sections[section].paragraphs,
                    para,
                    None,
                    stored_end,
                    &self.styles,
                    self.dpi,
                    hwp3_layout,
                );
                self.recompose_paragraph(section, para);
            }
        } else {
            *self.get_cell_paragraph_mut_by_path(section, para, &target.cell_path)? = candidate;
            let path = &target.cell_path;
            if axis_changed {
                let inner_para = path.last().unwrap().2;
                self.reflow_cell_paragraph_by_path(section, para, path, inner_para);
                self.recalculate_cell_paragraph_vpos_by_path(section, para, path, inner_para, None);
            }
            self.mark_cell_control_dirty(section, para, path[0].0);
        }
        self.document.sections[section].raw_stream = None;
        self.mark_section_dirty(section);
        self.paginate_if_needed();
        self.invalidate_page_tree_cache();
        self.event_log.push(DocumentEvent::HyperlinkChanged {
            section,
            para,
            cell_path: target.cell_path.clone(),
            field_id,
        });
        Ok(())
    }
}

fn invalid(message: &str) -> HwpError {
    HwpError::InvalidField(message.into())
}

fn find_link(p: &Paragraph, id: u32) -> Result<(usize, usize), HwpError> {
    p.field_ranges
        .iter()
        .enumerate()
        .find_map(|(idx, r)| match p.controls.get(r.control_idx) {
            Some(Control::Field(f)) if f.field_type == FieldType::Hyperlink && f.field_id == id => {
                Some((idx, r.control_idx))
            }
            _ => None,
        })
        .ok_or_else(|| invalid("하이퍼링크 필드를 찾을 수 없습니다"))
}

fn raw_boundary(p: &Paragraph, offset: usize) -> u32 {
    p.char_offsets
        .get(offset)
        .copied()
        .unwrap_or(p.char_count.saturating_sub(1))
}

/// 8-unit 마커를 넣거나 빼며 모든 raw UTF-16 참조 축을 함께 옮긴다.
/// 첫 글자 서식의 0 anchor는 그대로 둔다. 표시 문자 인덱스는 변하지 않는다.
fn shift_axis(p: &mut Paragraph, boundary: u32, delta: i32) -> Result<(), HwpError> {
    let shift = |v: &mut u32| -> Result<(), HwpError> {
        if *v >= boundary {
            *v = v
                .checked_add_signed(delta)
                .ok_or_else(|| invalid("필드 좌표 범위 초과"))?;
        }
        Ok(())
    };
    for v in &mut p.char_offsets {
        shift(v)?;
    }
    for shape in &mut p.char_shapes {
        if shape.start_pos != 0 {
            shift(&mut shape.start_pos)?;
        }
    }
    for range in &mut p.range_tags {
        shift(&mut range.start)?;
        shift(&mut range.end)?;
    }
    // 기존 저장 줄 경계는 HWPX 별도 축일 수 있으므로 재조판한다.
    p.stored_text_partition_dirty = true;
    p.char_count = p
        .char_count
        .checked_add_signed(delta)
        .ok_or_else(|| invalid("문단 길이 범위 초과"))?;
    Ok(())
}

//! 책갈피 조회/조작 기능
//!
//! 한컴 책갈피는 두 가지다. 보통 책갈피는 글자 사이의 `Control::Bookmark`(8유닛 `bokm`)이고,
//! 블록 책갈피는 고른 범위를 감싼 `%bmk` 필드(`Field{field_type: Bookmark}`)다. 블록이 여러
//! 문단에 걸치면 시작 문단에 필드가, 끝 문단에 고아 끝 표지(`OrphanFieldEnd`)가 든다.
//! 둘 다 본문뿐 아니라 표 셀·글상자·캡션·각주·미주·숨은 설명·머리말·꼬리말·바탕쪽에 놓일 수 있어,
//! `*At` API는 [`BookmarkPlace`] 주소로 그 문단을 가리킨다.

use crate::document_core::helpers::find_control_text_positions;
use crate::document_core::{header_footer_apply_from_u8, master_page_slot, DocumentCore};
use crate::error::HwpError;
use crate::model::control::{Bookmark, Control, Field, FieldType, Parameter, ParameterList};
use crate::model::paragraph::{FieldRange, OrphanFieldEnd, Paragraph};
use crate::parser::tags;

/// 책갈피가 든 문단의 주소.
///
/// `header_footer`가 없으면 `para`는 본문 문단이고 `cell_path`는 그 문단에서 시작하는
/// (컨트롤, 셀, 안쪽 문단) 경로다. `header_footer`가 있으면 `para`는 그 머리말·꼬리말
/// (`apply_to` 0~2)이나 바탕쪽(`apply_to` 16 이상)의 문단이고 경로는 그 문단에서 시작한다.
/// 셀 65534는 표·도형 캡션, 그림 캡션·각주·미주·숨은 설명·머리말·꼬리말은 셀 0이다.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BookmarkPlace {
    pub section: usize,
    pub para: usize,
    #[serde(default)]
    pub cell_path: Vec<(usize, usize, usize)>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub header_footer: Option<BookmarkHeaderFooter>,
}

/// 구역의 머리말·꼬리말 정의(그 종류·적용 범위의 첫 컨트롤)나 바탕쪽.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BookmarkHeaderFooter {
    pub is_header: bool,
    pub apply_to: u8,
}

/// 책갈피 정보
#[derive(Debug, Clone)]
struct BookmarkInfo {
    name: String,
    sec: usize,
    /// 본문 책갈피면 그 문단, 셀·주석·머리말 안이면 그 영역을 가진 본문 문단(바탕쪽은 0)
    para: usize,
    ctrl_idx: usize,
    /// 텍스트 내 위치 (정렬용). 중첩 책갈피는 옛 계약대로 0이다.
    char_pos: usize,
    place: BookmarkPlace,
    /// 경로 항목마다 그 컨트롤이 품은 영역의 종류
    areas: Vec<&'static str>,
    /// 블록 책갈피의 끝: 같은 문단 목록 안의 (문단, 글자 자리). 보통 책갈피면 None이다.
    end: Option<(usize, usize)>,
    /// 그 문단의 글자 자리. 블록이면 시작이다.
    start: usize,
}

impl DocumentCore {
    /// 문서 내 모든 책갈피 목록을 JSON으로 반환한다(문서 순서). 블록 책갈피(`kind: "block"`)도
    /// 함께 주며, `place`는 `*At` API가 받는 주소다.
    pub fn get_bookmarks_native(&self) -> Result<String, HwpError> {
        let bookmarks = self.collect_bookmarks();
        let items: Vec<String> = bookmarks
            .iter()
            .map(|b| {
                let end = b
                    .end
                    .map(|(para, offset)| format!(",\"endPara\":{para},\"end\":{offset}"))
                    .unwrap_or_default();
                format!(
                    "{{\"name\":{},\"sec\":{},\"para\":{},\"ctrlIdx\":{},\"charPos\":{},\"kind\":\"{}\",\"place\":{},\"areas\":{},\"start\":{}{}}}",
                    json_escape(&b.name),
                    b.sec,
                    b.para,
                    b.ctrl_idx,
                    b.char_pos,
                    if b.end.is_some() { "block" } else { "point" },
                    serde_json::to_string(&b.place).unwrap_or_else(|_| "null".into()),
                    serde_json::to_string(&b.areas).unwrap_or_else(|_| "[]".into()),
                    b.start,
                    end
                )
            })
            .collect();
        Ok(format!("[{}]", items.join(",")))
    }

    /// `addBookmarkAt`: 주소 문단의 글자 자리에 보통 책갈피를 넣는다.
    pub fn add_bookmark_at_native(
        &mut self,
        place: &BookmarkPlace,
        offset: usize,
        name: &str,
    ) -> Result<String, HwpError> {
        if let Some(error) = self.bookmark_name_error(name, None) {
            return Ok(error);
        }
        let (list, idx) = self.bookmark_list_mut(place)?;
        let paragraph = list.get_mut(idx).ok_or_else(|| invalid("문단 범위 초과"))?;
        if offset > paragraph.text.chars().count() {
            return Err(invalid("글자 자리가 문단 끝을 넘습니다"));
        }
        let insert_idx = insert_point_bookmark(paragraph, offset, name);
        self.shift_active_field(place, insert_idx, true);
        self.bookmark_edited(place);
        Ok(r#"{"ok":true}"#.to_string())
    }

    /// `addBlockBookmark`: 주소 문단의 `start`부터 같은 문단 목록의 `end_para` 문단 `end`까지를
    /// 한컴 블록 책갈피(`%bmk` 필드)로 감싼다. 글은 그대로 두고, 다른 필드와 엇갈리는 범위는 거절한다.
    pub fn add_block_bookmark_native(
        &mut self,
        place: &BookmarkPlace,
        start: usize,
        end_para: usize,
        end: usize,
        name: &str,
    ) -> Result<String, HwpError> {
        if let Some(error) = self.bookmark_name_error(name, None) {
            return Ok(error);
        }
        let field_id = self.next_click_here_field_id();
        let (list, idx) = self.bookmark_list_mut(place)?;
        let lengths = |p: &Paragraph| p.text.chars().count();
        if idx >= list.len() || end_para >= list.len() {
            return Err(invalid("문단 범위 초과"));
        }
        if end_para < idx || (end_para == idx && end <= start) {
            return Err(invalid(
                "블록 책갈피는 비어 있지 않은 앞에서 뒤로의 범위가 필요합니다",
            ));
        }
        if start > lengths(&list[idx]) || end > lengths(&list[end_para]) {
            return Err(invalid("글자 자리가 문단 끝을 넘습니다"));
        }
        // 글자별 원시 좌표가 없는 문단(HWP3 개체 표지)은 슬롯 자리를 셀 수 없다.
        if list[idx..=end_para]
            .iter()
            .any(|p| p.text.contains('\u{fffc}') || p.char_offsets.len() != lengths(p))
        {
            return Err(invalid("이 문단에는 블록 책갈피를 넣을 수 없습니다"));
        }
        let range = ((idx, start), (end_para, end));
        if field_spans(list)
            .into_iter()
            .any(|span| crosses(span, range))
        {
            return Ok(
                r#"{"ok":false,"error":"다른 블록 책갈피나 필드와 겹치는 범위에는 넣을 수 없습니다."}"#
                    .to_string(),
            );
        }
        let field = Field {
            field_type: FieldType::Bookmark,
            ctrl_id: tags::FIELD_BOOKMARK,
            field_id,
            ctrl_data_name: Some(name.to_string()),
            // 한컴 12.30 저장본처럼 수정됨 표식(dirty)과 매개변수 Prop=2를 둔다.
            properties: 1 << 15,
            parameters: ParameterList {
                name: Some(String::new()),
                items: vec![Parameter::Integer {
                    name: Some("Prop".into()),
                    value: 2,
                }],
            },
            ..Default::default()
        };
        let paragraph = &mut list[idx];
        let positions = find_control_text_positions(paragraph);
        let insert_idx = find_control_insert_index(paragraph, start);
        // 같은 문단에서 닫히면 BEGIN 뒤·END 앞 사이에 든 컨트롤 수를 적는다(파서와 같은 계약).
        let inner_slot_count = positions
            .iter()
            .skip(insert_idx)
            .filter(|&&pos| end_para == idx && pos < end)
            .count();
        paragraph.shift_for_inline_control_insert(insert_idx, start);
        paragraph.char_count += 8;
        paragraph.align_ctrl_data_records();
        paragraph.controls.insert(insert_idx, Control::Field(field));
        // CTRL_DATA는 저장기가 `ctrl_data_name`으로 만든다.
        paragraph.ctrl_data_records.insert(insert_idx, None);
        if end_para == idx {
            paragraph.shift_for_control_slot_insert(end);
            paragraph.char_count += 8;
            paragraph.field_ranges.push(FieldRange {
                start_char_idx: start,
                end_char_idx: end,
                control_idx: insert_idx,
                inner_slot_count,
                ..Default::default()
            });
            paragraph
                .field_ranges
                .sort_by_key(|range| (range.start_char_idx, range.control_idx));
        }
        paragraph.invalidate_layout_inputs();
        if end_para != idx {
            let last = &mut list[end_para];
            last.shift_for_control_slot_insert(end);
            last.char_count += 8;
            let at = last
                .orphan_field_ends
                .iter()
                .position(|other| other.char_idx > end)
                .unwrap_or(last.orphan_field_ends.len());
            last.orphan_field_ends.insert(
                at,
                OrphanFieldEnd {
                    char_idx: end,
                    begin_id_ref: field_id,
                    begin_ctrl_id: tags::FIELD_BOOKMARK,
                    ..Default::default()
                },
            );
            last.invalidate_layout_inputs();
        }
        self.shift_active_field(place, insert_idx, true);
        self.bookmark_edited(place);
        Ok(r#"{"ok":true}"#.to_string())
    }

    /// `renameBookmarkAt`: 주소 문단의 `ctrl_idx` 책갈피(보통·블록)의 이름을 바꾼다.
    pub fn rename_bookmark_at_native(
        &mut self,
        place: &BookmarkPlace,
        ctrl_idx: usize,
        new_name: &str,
    ) -> Result<String, HwpError> {
        if let Some(error) = self.bookmark_name_error(new_name, Some((place, ctrl_idx))) {
            return Ok(error);
        }
        let (list, idx) = self.bookmark_list_mut(place)?;
        let paragraph = list.get_mut(idx).ok_or_else(|| invalid("문단 범위 초과"))?;
        match paragraph.controls.get_mut(ctrl_idx) {
            Some(Control::Bookmark(bookmark)) => {
                bookmark.name = new_name.to_string();
                if ctrl_idx < paragraph.ctrl_data_records.len() {
                    paragraph.ctrl_data_records[ctrl_idx] =
                        Some(build_bookmark_ctrl_data(new_name));
                }
            }
            Some(Control::Field(field)) if field.field_type == FieldType::Bookmark => {
                field.ctrl_data_name = Some(new_name.to_string());
                // 원본 CTRL_DATA가 옛 이름을 싣고 있으면 저장기가 새 이름으로 다시 만들게 한다.
                if ctrl_idx < paragraph.ctrl_data_records.len() {
                    paragraph.ctrl_data_records[ctrl_idx] = None;
                }
            }
            _ => {
                return Ok(r#"{"ok":false,"error":"해당 컨트롤이 책갈피가 아닙니다."}"#.to_string())
            }
        }
        self.bookmark_edited(place);
        Ok(r#"{"ok":true}"#.to_string())
    }

    /// `deleteBookmarkAt`: 주소 문단의 `ctrl_idx` 책갈피를 지운다. 블록은 감싼 글을 남긴다.
    pub fn delete_bookmark_at_native(
        &mut self,
        place: &BookmarkPlace,
        ctrl_idx: usize,
    ) -> Result<String, HwpError> {
        let (list, idx) = self.bookmark_list_mut(place)?;
        let paragraph = list.get(idx).ok_or_else(|| invalid("문단 범위 초과"))?;
        match paragraph.controls.get(ctrl_idx) {
            Some(Control::Bookmark(_)) => remove_point_bookmark(&mut list[idx], ctrl_idx)?,
            Some(Control::Field(field)) if field.field_type == FieldType::Bookmark => {
                let field_id = field.field_id;
                let closed = paragraph
                    .field_ranges
                    .iter()
                    .any(|range| range.control_idx == ctrl_idx);
                let removed = if closed {
                    list[idx].unwrap_field_control(ctrl_idx)
                } else {
                    let end = (field_id != 0)
                        .then(|| orphan_end_of(list, idx, field_id))
                        .flatten()
                        .ok_or_else(|| invalid("블록 책갈피의 끝을 찾을 수 없습니다"))?;
                    let ok = list[idx].remove_unclosed_field_begin(ctrl_idx);
                    if ok {
                        list[end.0].remove_orphan_field_end(end.1);
                        list[end.0].invalidate_layout_inputs();
                    }
                    ok
                };
                if !removed {
                    return Err(invalid("블록 책갈피의 자리를 확인할 수 없습니다"));
                }
            }
            _ => {
                return Ok(r#"{"ok":false,"error":"해당 컨트롤이 책갈피가 아닙니다."}"#.to_string())
            }
        }
        self.shift_active_field(place, ctrl_idx, false);
        self.bookmark_edited(place);
        Ok(r#"{"ok":true}"#.to_string())
    }

    /// 이름이 비었거나 다른 책갈피가 이미 쓰면 응답 JSON을 준다. `except`는 이름을 바꾸는 그 책갈피다.
    fn bookmark_name_error(
        &self,
        name: &str,
        except: Option<(&BookmarkPlace, usize)>,
    ) -> Option<String> {
        if name.trim().is_empty() {
            return Some(r#"{"ok":false,"error":"책갈피 이름을 입력하세요."}"#.to_string());
        }
        self.collect_bookmarks()
            .iter()
            .any(|b| b.name == name && except != Some((&b.place, b.ctrl_idx)))
            .then(|| {
                r#"{"ok":false,"error":"같은 이름의 책갈피가 이미 등록되어 있습니다."}"#.to_string()
            })
    }

    /// 주소가 가리키는 문단 목록과 그 안의 문단 번호.
    fn bookmark_list_mut(
        &mut self,
        place: &BookmarkPlace,
    ) -> Result<(&mut Vec<Paragraph>, usize), HwpError> {
        let section = place.section;
        if section >= self.document.sections.len() {
            return Err(invalid("구역 범위 초과"));
        }
        let Some(hf) = place.header_footer else {
            let Some(&(_, _, inner)) = place.cell_path.last() else {
                return Ok((&mut self.document.sections[section].paragraphs, place.para));
            };
            let list =
                self.get_cell_paragraphs_mut_by_path(section, place.para, &place.cell_path)?;
            return Ok((list, inner));
        };
        if let Some(slot) = master_page_slot(hf.apply_to) {
            if !place.cell_path.is_empty() {
                return Err(invalid("바탕쪽 안 개체의 책갈피는 고칠 수 없습니다"));
            }
            return Ok((
                &mut self.master_page_mut(section, slot)?.paragraphs,
                place.para,
            ));
        }
        let (host, path) = self.header_footer_path(place, hf)?;
        let inner = path.last().map_or(place.para, |entry| entry.2);
        let list = self.get_cell_paragraphs_mut_by_path(section, host, &path)?;
        Ok((list, inner))
    }

    /// 머리말·꼬리말 주소를 그 컨트롤을 가진 본문 문단과 본문에서 시작하는 경로로 바꾼다.
    fn header_footer_path(
        &self,
        place: &BookmarkPlace,
        hf: BookmarkHeaderFooter,
    ) -> Result<(usize, Vec<(usize, usize, usize)>), HwpError> {
        let (host, control) = self
            .find_header_footer_control(
                place.section,
                hf.is_header,
                header_footer_apply_from_u8(hf.apply_to),
            )
            .ok_or_else(|| invalid("머리말/꼬리말 정의를 찾을 수 없음"))?;
        let mut path = vec![(control, 0, place.para)];
        path.extend_from_slice(&place.cell_path);
        Ok((host, path))
    }

    /// 책갈피를 넣거나 빼면 같은 문단 또는 그 문단의 개체 안에 있는 활성 누름틀의 컨트롤 번호를 맞춘다.
    fn shift_active_field(&mut self, place: &BookmarkPlace, ctrl_idx: usize, inserted: bool) {
        // 활성 누름틀 주소는 본문에서 시작한다. 바탕쪽에는 누름틀 편집이 없다.
        let (host, path) = match place.header_footer {
            None => (place.para, place.cell_path.clone()),
            Some(hf) if master_page_slot(hf.apply_to).is_none() => {
                match self.header_footer_path(place, hf) {
                    Ok(address) => address,
                    Err(_) => return,
                }
            }
            Some(_) => return,
        };
        let Some(active) = self.active_field.as_mut() else {
            return;
        };
        if active.section_idx != place.section || active.para_idx != host {
            return;
        }
        let shift = |index: &mut usize| {
            if inserted && *index >= ctrl_idx {
                *index += 1;
            } else if !inserted && *index > ctrl_idx {
                *index -= 1;
            }
        };
        let active_path = active.cell_path.clone().unwrap_or_default();
        if !active_path.starts_with(&path) {
            return;
        }
        if active_path.len() == path.len() {
            shift(&mut active.control_idx);
        } else if let Some(entry) = active
            .cell_path
            .as_mut()
            .and_then(|cells| cells.get_mut(path.len()))
        {
            shift(&mut entry.0);
        }
    }

    /// 책갈피 편집 뒤 저장 원본을 버리고 다시 조판한다.
    fn bookmark_edited(&mut self, place: &BookmarkPlace) {
        let section = place.section;
        match place.header_footer {
            Some(hf) if master_page_slot(hf.apply_to).is_some() => {
                self.sync_section_def_controls_from_section(section);
            }
            Some(hf) => {
                if let Ok((host, path)) = self.header_footer_path(place, hf) {
                    self.mark_cell_control_dirty(section, host, path[0].0);
                }
            }
            None => {
                if let Some(&(control, _, _)) = place.cell_path.first() {
                    self.mark_cell_control_dirty(section, place.para, control);
                }
            }
        }
        // 원본 스트림 무효화 — 비우지 않으면 저장기가 원본 바이트를 그대로 내 편집이 사라진다.
        self.document.sections[section].raw_stream = None;
        self.recompose_section(section);
    }

    /// 책갈피 추가
    ///
    /// 지정 위치에 Bookmark 컨트롤을 삽입한다.
    /// 중복 이름은 거부한다.
    pub fn add_bookmark_native(
        &mut self,
        sec: usize,
        para: usize,
        char_offset: usize,
        name: &str,
    ) -> Result<String, HwpError> {
        if name.trim().is_empty() {
            return Ok(r#"{"ok":false,"error":"책갈피 이름을 입력하세요."}"#.to_string());
        }

        // 중복 검사
        let existing = self.collect_bookmarks();
        if existing.iter().any(|b| b.name == name) {
            return Ok(
                r#"{"ok":false,"error":"같은 이름의 책갈피가 이미 등록되어 있습니다."}"#
                    .to_string(),
            );
        }

        let section = self
            .document
            .sections
            .get_mut(sec)
            .ok_or_else(|| HwpError::RenderError("구역 범위 초과".into()))?;
        let paragraph = section
            .paragraphs
            .get_mut(para)
            .ok_or_else(|| HwpError::RenderError("문단 범위 초과".into()))?;

        let insert_idx = insert_point_bookmark(paragraph, char_offset, name);
        if let Some(active) = self.active_field.as_mut() {
            if active.section_idx == sec
                && active.para_idx == para
                && active.cell_path.is_none()
                && active.control_idx >= insert_idx
            {
                active.control_idx += 1;
            }
        }

        // 원본 스트림 무효화 — serialize_section 은 raw_stream 이 있으면 IR 을 무시하고
        // 원본 바이트를 그대로 반환하므로(serializer/body_text.rs), 비우지 않으면 방금
        // 삽입한 책갈피 컨트롤이 저장 시 통째로 사라진다. recompose_section 은 화면(구성)만
        // 갱신할 뿐 raw_stream 을 건드리지 않는다. 누름틀·양식 쪽과 동일한 불변식이다.
        if let Some(s) = self.document.sections.get_mut(sec) {
            s.raw_stream = None;
        }
        self.recompose_section(sec);

        Ok(r#"{"ok":true}"#.to_string())
    }

    /// 책갈피 삭제
    pub fn delete_bookmark_native(
        &mut self,
        sec: usize,
        para: usize,
        ctrl_idx: usize,
    ) -> Result<String, HwpError> {
        let section = self
            .document
            .sections
            .get_mut(sec)
            .ok_or_else(|| HwpError::RenderError("구역 범위 초과".into()))?;
        let paragraph = section
            .paragraphs
            .get_mut(para)
            .ok_or_else(|| HwpError::RenderError("문단 범위 초과".into()))?;

        if ctrl_idx >= paragraph.controls.len() {
            return Ok(r#"{"ok":false,"error":"컨트롤 인덱스 범위 초과"}"#.to_string());
        }

        // Bookmark인지 확인
        if !matches!(&paragraph.controls[ctrl_idx], Control::Bookmark(_)) {
            return Ok(r#"{"ok":false,"error":"해당 컨트롤이 책갈피가 아닙니다."}"#.to_string());
        }

        if self.active_field.as_ref().is_some_and(|active| {
            active.section_idx == sec
                && active
                    .cell_path
                    .as_ref()
                    .is_some_and(|path| path.first().is_some_and(|&(table, _, _)| table > ctrl_idx))
        }) {
            // 현재 셀 활성 주소에는 본문 부모 문단 번호가 없어 호스트를 판별할 수 없다.
            return Err(HwpError::InvalidField(
                "셀 누름틀 편집을 끝낸 뒤 책갈피를 삭제하세요.".into(),
            ));
        }
        remove_point_bookmark(paragraph, ctrl_idx)?;
        if let Some(active) = self.active_field.as_mut() {
            if active.section_idx == sec
                && active.para_idx == para
                && active.cell_path.is_none()
                && active.control_idx > ctrl_idx
            {
                active.control_idx -= 1;
            }
        }

        // 원본 스트림 무효화 — 비우지 않으면 삭제한 책갈피가 저장 시 원본 바이트로 되살아난다.
        if let Some(s) = self.document.sections.get_mut(sec) {
            s.raw_stream = None;
        }
        self.recompose_section(sec);

        Ok(r#"{"ok":true}"#.to_string())
    }

    /// 책갈피 이름 변경
    pub fn rename_bookmark_native(
        &mut self,
        sec: usize,
        para: usize,
        ctrl_idx: usize,
        new_name: &str,
    ) -> Result<String, HwpError> {
        if new_name.trim().is_empty() {
            return Ok(r#"{"ok":false,"error":"책갈피 이름을 입력하세요."}"#.to_string());
        }

        // 중복 검사 (자기 자신 제외)
        let existing = self.collect_bookmarks();
        if existing.iter().any(|b| {
            b.name == new_name && !(b.sec == sec && b.para == para && b.ctrl_idx == ctrl_idx)
        }) {
            return Ok(
                r#"{"ok":false,"error":"같은 이름의 책갈피가 이미 등록되어 있습니다."}"#
                    .to_string(),
            );
        }

        let section = self
            .document
            .sections
            .get_mut(sec)
            .ok_or_else(|| HwpError::RenderError("구역 범위 초과".into()))?;
        let paragraph = section
            .paragraphs
            .get_mut(para)
            .ok_or_else(|| HwpError::RenderError("문단 범위 초과".into()))?;

        if ctrl_idx >= paragraph.controls.len() {
            return Ok(r#"{"ok":false,"error":"컨트롤 인덱스 범위 초과"}"#.to_string());
        }

        if let Control::Bookmark(ref mut bm) = paragraph.controls[ctrl_idx] {
            bm.name = new_name.to_string();
            // CTRL_DATA도 갱신
            if ctrl_idx < paragraph.ctrl_data_records.len() {
                paragraph.ctrl_data_records[ctrl_idx] = Some(build_bookmark_ctrl_data(new_name));
            }
            // 원본 스트림 무효화 — 비우지 않으면 이름 변경이 저장 시 옛 이름으로 되돌아간다.
            // add/delete 와 달리 이 함수는 recompose_section 도 호출하지 않았다 — 무효화와
            // 함께 추가한다(다른 뮤테이터와 동일하게 편집 후 구성/커서를 갱신).
            if let Some(s) = self.document.sections.get_mut(sec) {
                s.raw_stream = None;
            }
            self.recompose_section(sec);
            Ok(r#"{"ok":true}"#.to_string())
        } else {
            Ok(r#"{"ok":false,"error":"해당 컨트롤이 책갈피가 아닙니다."}"#.to_string())
        }
    }

    /// 하이퍼링크 대상 검사: 점 책갈피 컨트롤과 블록 책갈피 필드(`%bmk`) 이름을 모두 본다.
    pub(crate) fn has_bookmark_named(&self, name: &str) -> bool {
        self.collect_bookmarks().iter().any(|b| b.name == name)
            || self.collect_all_fields().iter().any(|f| {
                f.field.field_type == crate::model::control::FieldType::Bookmark
                    && f.field.ctrl_data_name.as_deref() == Some(name)
            })
    }

    /// 내부: 모든 책갈피 수집(문서 순서). 구역마다 본문(그 안의 표·글상자·캡션·각주·미주·
    /// 숨은 설명·머리말·꼬리말 포함) 뒤에 바탕쪽을 훑는다.
    fn collect_bookmarks(&self) -> Vec<BookmarkInfo> {
        let mut result = vec![];
        for (sec_idx, section) in self.document.sections.iter().enumerate() {
            let mut walk = BookmarkWalk {
                core: self,
                place: BookmarkPlace {
                    section: sec_idx,
                    para: 0,
                    cell_path: Vec::new(),
                    header_footer: None,
                },
                areas: Vec::new(),
                host: None,
                out: &mut result,
            };
            walk.list(&section.paragraphs);
            for (slot, master) in section.section_def.master_pages.iter().enumerate() {
                walk.place.header_footer = Some(BookmarkHeaderFooter {
                    is_header: false,
                    apply_to: crate::document_core::MASTER_PAGE_APPLY_BASE + slot as u8,
                });
                walk.host = Some(0);
                walk.list(&master.paragraphs);
            }
        }
        result
    }
}

/// 컨트롤 밖의 숨은 슬롯이 없는 구간에서만 책갈피 원시 위치를 확정한다.
fn bookmark_raw_start(paragraph: &Paragraph, control_idx: usize) -> Result<u32, HwpError> {
    let invalid = || HwpError::InvalidField("책갈피의 원시 슬롯 위치를 확인할 수 없습니다.".into());
    if !paragraph.title_marks.is_empty()
        || !paragraph.orphan_field_ends.is_empty()
        || paragraph.text.contains('\u{fffc}')
        || paragraph.controls[..control_idx].iter().any(|control| {
            matches!(
                control,
                Control::Field(_)
                    | Control::Hyperlink(_)
                    | Control::AutoNumber(_)
                    | Control::NewNumber(_)
                    | Control::Ruby(_)
                    | Control::CharOverlap(_)
                    | Control::Unknown(_)
            )
        })
        || paragraph.char_offsets.len() != paragraph.text.chars().count()
    {
        return Err(invalid());
    }
    let checked_start = |start: u32| {
        if start
            .checked_add(8)
            .is_none_or(|end| end >= paragraph.char_count)
        {
            Err(invalid())
        } else {
            Ok(start)
        }
    };
    let mut remaining = u32::try_from(control_idx).map_err(|_| invalid())?;
    let mut end = 0u32;
    for (&offset, ch) in paragraph.char_offsets.iter().zip(paragraph.text.chars()) {
        let gap = offset.checked_sub(end).ok_or_else(invalid)?;
        if gap % 8 != 0 {
            return Err(invalid());
        }
        if remaining < gap / 8 {
            return checked_start(end + remaining * 8);
        }
        remaining -= gap / 8;
        end = offset + if ch == '\t' { 8 } else { ch.len_utf16() as u32 };
    }
    let start = end
        .checked_add(remaining.checked_mul(8).ok_or_else(invalid)?)
        .ok_or_else(invalid)?;
    checked_start(start)
}

/// 책갈피 수집의 현재 문단 목록 주소. `place.para`·경로 마지막 항목의 문단 번호는 훑는 동안 바뀐다.
struct BookmarkWalk<'a, 'b> {
    core: &'a DocumentCore,
    place: BookmarkPlace,
    areas: Vec<&'static str>,
    /// 옛 계약의 본문 문단(셀·주석·머리말 안이면 그 영역을 가진 본문 문단). None이면 본문 목록이다.
    host: Option<usize>,
    out: &'b mut Vec<BookmarkInfo>,
}

impl BookmarkWalk<'_, '_> {
    fn list(&mut self, paragraphs: &[Paragraph]) {
        for (para_idx, para) in paragraphs.iter().enumerate() {
            match self.place.cell_path.last_mut() {
                Some(entry) => entry.2 = para_idx,
                None => self.place.para = para_idx,
            }
            let host = self.host.unwrap_or(para_idx);
            let positions = find_control_text_positions(para);
            for (ctrl_idx, ctrl) in para.controls.iter().enumerate() {
                let position = positions.get(ctrl_idx).copied().unwrap_or(0);
                let (name, start, end) = match ctrl {
                    Control::Bookmark(bm) => (bm.name.clone(), position, None),
                    Control::Field(field) if field.field_type == FieldType::Bookmark => {
                        let range = para
                            .field_ranges
                            .iter()
                            .find(|range| range.control_idx == ctrl_idx);
                        let end = match range {
                            Some(range) => Some((para_idx, range.end_char_idx)),
                            None => (field.field_id != 0)
                                .then(|| orphan_end_of(paragraphs, para_idx, field.field_id))
                                .flatten()
                                .map(|(end_para, end_idx)| {
                                    (
                                        end_para,
                                        paragraphs[end_para].orphan_field_ends[end_idx].char_idx,
                                    )
                                }),
                        };
                        // 끝을 찾지 못한 블록은 시작 자리의 보통 책갈피처럼 보인다.
                        let start = range.map_or(position, |range| range.start_char_idx);
                        (field.ctrl_data_name.clone().unwrap_or_default(), start, end)
                    }
                    _ => {
                        self.children(ctrl_idx, ctrl, host);
                        continue;
                    }
                };
                self.out.push(BookmarkInfo {
                    name,
                    sec: self.place.section,
                    para: host,
                    ctrl_idx,
                    char_pos: if self.host.is_some() { 0 } else { start },
                    place: self.place.clone(),
                    areas: self.areas.clone(),
                    end,
                    start,
                });
            }
        }
    }

    /// 컨트롤이 품은 문단 목록을 훑는다. 본문의 머리말·꼬리말 정의는 그 머리말 주소로 가리킨다.
    fn children(&mut self, ctrl_idx: usize, ctrl: &Control, host: usize) {
        let body = self.place.cell_path.is_empty() && self.place.header_footer.is_none();
        if let (true, Control::Header(_) | Control::Footer(_)) = (body, ctrl) {
            let (is_header, apply) = match ctrl {
                Control::Header(h) => (true, h.apply_to),
                Control::Footer(f) => (false, f.apply_to),
                _ => unreachable!(),
            };
            let canonical =
                self.core
                    .find_header_footer_control(self.place.section, is_header, apply)
                    == Some((self.place.para, ctrl_idx));
            if canonical {
                let saved = self.place.para;
                self.place.header_footer = Some(BookmarkHeaderFooter {
                    is_header,
                    apply_to: crate::document_core::header_footer_apply_to_u8(apply),
                });
                let previous_host = self.host.replace(host);
                if let Some(paragraphs) = ctrl.header_footer_paragraphs() {
                    self.list(paragraphs);
                }
                self.host = previous_host;
                self.place.header_footer = None;
                self.place.para = saved;
                return;
            }
        }
        for (cell, area, paragraphs) in child_lists(ctrl) {
            self.place.cell_path.push((ctrl_idx, cell, 0));
            self.areas.push(area);
            let previous_host = self.host.replace(host);
            self.list(paragraphs);
            self.host = previous_host;
            self.areas.pop();
            self.place.cell_path.pop();
        }
    }
}

/// 컨트롤이 품은 문단 목록과 그 경로 셀 번호·영역 종류.
fn child_lists(ctrl: &Control) -> Vec<(usize, &'static str, &[Paragraph])> {
    let mut lists: Vec<(usize, &'static str, &[Paragraph])> = Vec::new();
    match ctrl {
        Control::Table(table) => {
            for (cell, content) in table.cells.iter().enumerate() {
                lists.push((cell, "table", &content.paragraphs));
            }
            if let Some(caption) = &table.caption {
                lists.push((65534, "caption", &caption.paragraphs));
            }
        }
        Control::Shape(shape) => {
            for index in 0..shape.textbox_shape_count() {
                if let Some(text_box) = crate::document_core::helpers::get_textbox_at(shape, index)
                {
                    lists.push((index, "textBox", &text_box.paragraphs));
                }
            }
            if let Some(caption) = DocumentCore::shape_caption_ref(shape) {
                lists.push((65534, "caption", &caption.paragraphs));
            }
        }
        Control::Picture(picture) => {
            if let Some(caption) = &picture.caption {
                lists.push((0, "caption", &caption.paragraphs));
            }
        }
        Control::Footnote(note) => lists.push((0, "footnote", &note.paragraphs)),
        Control::Endnote(note) => lists.push((0, "endnote", &note.paragraphs)),
        Control::Header(header) => lists.push((0, "header", &header.paragraphs)),
        Control::Footer(footer) => lists.push((0, "footer", &footer.paragraphs)),
        Control::HiddenComment(comment) => lists.push((0, "hiddenComment", &comment.paragraphs)),
        _ => {}
    }
    lists
}

/// `start` 문단에서 연 다단락 필드의 고아 끝 표지: (문단, 그 문단의 고아 끝 번호).
fn orphan_end_of(paragraphs: &[Paragraph], start: usize, field_id: u32) -> Option<(usize, usize)> {
    paragraphs
        .iter()
        .enumerate()
        .skip(start + 1)
        .find_map(|(para_idx, para)| {
            para.orphan_field_ends
                .iter()
                .position(|end| end.begin_id_ref == field_id)
                .map(|end_idx| (para_idx, end_idx))
        })
}

/// 문단 목록의 모든 필드 범위 ((시작 문단, 글자), (끝 문단, 글자)). 끝을 모르는 필드는 시작 자리에서 끝난다.
fn field_spans(paragraphs: &[Paragraph]) -> Vec<((usize, usize), (usize, usize))> {
    let mut spans = Vec::new();
    for (para_idx, para) in paragraphs.iter().enumerate() {
        let positions = find_control_text_positions(para);
        for (ctrl_idx, ctrl) in para.controls.iter().enumerate() {
            let Control::Field(field) = ctrl else {
                continue;
            };
            if let Some(range) = para
                .field_ranges
                .iter()
                .find(|range| range.control_idx == ctrl_idx)
            {
                spans.push((
                    (para_idx, range.start_char_idx),
                    (para_idx, range.end_char_idx),
                ));
                continue;
            }
            let start = (para_idx, positions.get(ctrl_idx).copied().unwrap_or(0));
            let end = (field.field_id != 0)
                .then(|| orphan_end_of(paragraphs, para_idx, field.field_id))
                .flatten()
                .map_or(start, |(end_para, end_idx)| {
                    (
                        end_para,
                        paragraphs[end_para].orphan_field_ends[end_idx].char_idx,
                    )
                });
            spans.push((start, end));
        }
    }
    spans
}

/// 필드 하나와 새 범위가 엇갈리는가. 같은 자리에서 다른 필드가 끝나면 그 END가 새 BEGIN보다,
/// 새 END가 같은 자리에서 시작하는 필드의 BEGIN보다 먼저 저장되므로 맞닿음은 엇갈림이 아니다.
/// 새 범위는 다른 필드를 품거나 그 안에 들 수 있지만, 끝이 같은 자리면 END 차례가 모호해 거절한다.
fn crosses(
    (start, end): ((usize, usize), (usize, usize)),
    (new_start, new_end): ((usize, usize), (usize, usize)),
) -> bool {
    let apart = end <= new_start || start >= new_end;
    let outer = start <= new_start && end > new_end;
    let inner = start > new_start && end < new_end;
    !(apart || outer || inner)
}

/// 문단의 글자 자리에 보통 책갈피를 넣고 그 컨트롤 번호를 돌려준다. 같은 자리의 개체·각주 뒤에 든다.
fn insert_point_bookmark(paragraph: &mut Paragraph, char_offset: usize, name: &str) -> usize {
    let insert_idx = find_control_insert_index(paragraph, char_offset);
    paragraph.controls.insert(
        insert_idx,
        Control::Bookmark(Bookmark {
            name: name.to_string(),
        }),
    );
    // CTRL_DATA 레코드 생성 (ParameterSet: 책갈피 이름)
    let ctrl_data = build_bookmark_ctrl_data(name);
    if paragraph.ctrl_data_records.len() >= insert_idx {
        paragraph
            .ctrl_data_records
            .insert(insert_idx, Some(ctrl_data));
    }
    // 책갈피도 스트림에서 8유닛을 차지한다. 글자별 배열에 항목을 끼우지 않고 뒤 글자와
    // 같은 축의 참조를 함께 민다. 삭제와 같은 이유로 저장 줄 경계는 새 조판 결과를 쓴다.
    paragraph.shift_for_inline_control_insert(insert_idx, char_offset);
    paragraph.char_count += 8;
    paragraph.stored_text_partition_dirty = true;
    insert_idx
}

/// 문단의 보통 책갈피 하나를 그 8유닛 자리와 함께 지운다.
fn remove_point_bookmark(paragraph: &mut Paragraph, ctrl_idx: usize) -> Result<(), HwpError> {
    // 글자별 배열에서 컨트롤 번호의 항목을 빼면 글자 하나의 원시 좌표가 사라진다.
    // 변경 전에 8유닛 슬롯 위치를 확인하고 같은 축의 참조를 함께 당긴다.
    let start = bookmark_raw_start(paragraph, ctrl_idx)?;
    let shift = |position: &mut u32| {
        if *position > start {
            *position = position.saturating_sub(8).max(start);
        }
    };
    for position in &mut paragraph.char_offsets {
        shift(position);
    }
    for shape in &mut paragraph.char_shapes {
        shift(&mut shape.start_pos);
    }
    for range in &mut paragraph.range_tags {
        shift(&mut range.start);
        shift(&mut range.end);
    }
    for mark in &mut paragraph.markpen_marks {
        if let Some(position) = &mut mark.utf16_pos {
            shift(position);
        }
    }
    paragraph.char_count -= 8;
    // 저장 줄 경계는 HWPX 별도 축일 수 있으므로 새 조판 결과를 사용한다.
    paragraph.stored_text_partition_dirty = true;
    paragraph.controls.remove(ctrl_idx);
    if ctrl_idx < paragraph.ctrl_data_records.len() {
        paragraph.ctrl_data_records.remove(ctrl_idx);
    }
    for range in &mut paragraph.field_ranges {
        if range.control_idx > ctrl_idx {
            range.control_idx -= 1;
        }
    }
    paragraph.forget_para_text_if_emptied();
    Ok(())
}

/// `*At` WASM API의 주소 JSON을 읽는다.
pub fn parse_bookmark_place(json: &str) -> Result<BookmarkPlace, HwpError> {
    serde_json::from_str(json).map_err(|error| invalid(&format!("책갈피 주소: {error}")))
}

/// F11·Shift+F11이 고르는 컨트롤 종류. 본문 검색(`find_nearest_control_*_native`)과 같은 갈래다.
fn nearest_kind(ctrl: &Control) -> Option<&'static str> {
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

/// 주소가 지나는 문단 목록들. 첫째는 본문(또는 바탕쪽) 목록이고, 다음은 경로 항목마다 그 컨트롤이 연 목록과 영역 종류다.
type PlaceLists<'a> = Vec<(&'a [Paragraph], &'static str)>;

/// 주소를 푼 결과: (바탕쪽 번호, 바탕 문단, 바탕 문단에서 시작하는 경로, 경로가 지나는 문단 목록).
type PlaceWalk<'a> = (
    Option<usize>,
    usize,
    Vec<(usize, usize, usize)>,
    PlaceLists<'a>,
);

impl DocumentCore {
    /// 주소를 본문(또는 바탕쪽)에서 시작하는 (바탕쪽 번호, 바탕 문단, 경로)로 바꾸고 경로가 지나는 문단 목록을 모은다.
    /// 머리말·꼬리말은 그 정의 컨트롤을 가진 본문 문단에서 시작하는 경로가 된다.
    fn place_lists(&self, place: &BookmarkPlace) -> Result<PlaceWalk<'_>, HwpError> {
        let section = self
            .document
            .sections
            .get(place.section)
            .ok_or_else(|| invalid("구역 범위 초과"))?;
        let master = place
            .header_footer
            .and_then(|hf| master_page_slot(hf.apply_to));
        let (base, host, path): (&[Paragraph], usize, Vec<(usize, usize, usize)>) =
            match (place.header_footer, master) {
                (Some(_), Some(slot)) => (
                    &section
                        .section_def
                        .master_pages
                        .get(slot)
                        .ok_or_else(|| invalid("바탕쪽을 찾을 수 없음"))?
                        .paragraphs,
                    place.para,
                    place.cell_path.clone(),
                ),
                (Some(hf), None) => {
                    let (host, path) = self.header_footer_path(place, hf)?;
                    (&section.paragraphs, host, path)
                }
                _ => (&section.paragraphs, place.para, place.cell_path.clone()),
            };
        let mut lists: PlaceLists<'_> = vec![(base, "")];
        for (depth, &(ctrl_idx, cell, _)) in path.iter().enumerate() {
            let para_idx = match depth {
                0 => host,
                _ => path[depth - 1].2,
            };
            let ctrl = lists[depth]
                .0
                .get(para_idx)
                .and_then(|para| para.controls.get(ctrl_idx))
                .ok_or_else(|| invalid("주소의 컨트롤을 찾을 수 없음"))?;
            let (_, area, paragraphs) = child_lists(ctrl)
                .into_iter()
                .find(|(index, _, _)| *index == cell)
                .ok_or_else(|| invalid("주소의 문단 목록을 찾을 수 없음"))?;
            lists.push((paragraphs, area));
        }
        let last = path.last().map_or(host, |entry| entry.2);
        if lists.last().is_none_or(|(list, _)| last >= list.len()) {
            return Err(invalid("문단 범위 초과"));
        }
        Ok((master, host, path, lists))
    }

    /// `findNearestControlAt`: 주소 문단(`getBookmarks`의 `place` 꼴)의 글자 자리에서 앞(`forward`면 뒤)으로
    /// 가장 가까운 표·그림·도형·수식·필드·책갈피를 찾는다.
    ///
    /// 표 셀·글상자·캡션·각주·미주·머리말·꼬리말 안이면 그 문단 목록을 먼저 훑고, 다 훑으면 그 영역을 가진
    /// 컨트롤로 나온다. 한/글 12.30 F11처럼 앞으로 찾을 때는 그 컨트롤이 표·그림·도형·수식이면 그것을 고르고,
    /// 각주·머리말처럼 고를 수 없으면 그 컨트롤 앞에서 잇는다. 본문에서는 본문 검색처럼 셀 안으로 들어가지 않는다.
    ///
    /// `from_control`이 있으면 글자 자리 대신 주소 문단의 그 컨트롤을 경계로 삼아, 같은 글자 자리의 앞(뒤) 형제부터 잇는다.
    ///
    /// 반환: `{"type","sec","para","ci","charPos","place","areas"}`. `para`는 그 영역을 가진 본문 문단,
    /// `place`·`areas`는 `getBookmarks`와 같은 꼴의 그 컨트롤 문단 주소다. 필드면 `fieldId`·`start`·`end`(그
    /// 문단의 글자 범위)를 더한다. 없으면 `{"type":"none"}`.
    pub fn find_nearest_control_at_native(
        &self,
        place: &BookmarkPlace,
        offset: usize,
        from_control: Option<usize>,
        forward: bool,
    ) -> Result<String, HwpError> {
        let (master, host, path, lists) = self.place_lists(place)?;
        let hit = |section: usize,
                   depth: usize,
                   para_idx: usize,
                   ci: usize,
                   kind: &str|
         -> String {
            let list = if section == place.section {
                lists[depth].0
            } else {
                &self.document.sections[section].paragraphs
            };
            let para = &list[para_idx];
            let char_pos = find_control_text_positions(para)
                .get(ci)
                .copied()
                .unwrap_or(0);
            let mut cell_path = path[..depth].to_vec();
            if let Some(last) = cell_path.last_mut() {
                last.2 = para_idx;
            }
            let mut areas: Vec<&str> = lists[1..=depth].iter().map(|(_, area)| *area).collect();
            let body_para = if depth == 0 { para_idx } else { host };
            let (out_para, header_footer) = match place.header_footer {
                // 머리말·꼬리말 안: 주소는 그 머리말 문단에서 시작한다.
                Some(hf) if master.is_none() && depth > 0 => {
                    areas.remove(0);
                    (cell_path.remove(0).2, Some(hf))
                }
                Some(hf) if master.is_some() => (body_para, Some(hf)),
                _ => (body_para, None),
            };
            let out = BookmarkPlace {
                section,
                para: out_para,
                cell_path,
                header_footer,
            };
            let field = para
                .field_ranges
                .iter()
                .find(|range| range.control_idx == ci)
                .and_then(|range| match &para.controls[ci] {
                    Control::Field(field) => Some(format!(
                        ",\"fieldId\":{},\"fieldType\":\"{}\",\"start\":{},\"end\":{}",
                        field.field_id,
                        field.field_type_str(),
                        range.start_char_idx,
                        range.end_char_idx
                    )),
                    _ => None,
                })
                .unwrap_or_default();
            format!(
                "{{\"type\":\"{kind}\",\"sec\":{section},\"para\":{body_para},\"ci\":{ci},\"charPos\":{char_pos},\"place\":{},\"areas\":{}{field}}}",
                serde_json::to_string(&out).unwrap_or_else(|_| "null".into()),
                serde_json::to_string(&areas).unwrap_or_else(|_| "[]".into()),
            )
        };
        // 한 문단에서 조건에 맞는 첫 컨트롤(앞으로 찾으면 마지막부터).
        let pick = |para: &Paragraph, accept: &dyn Fn(usize, usize) -> bool| {
            let positions = find_control_text_positions(para);
            let mut order: Vec<usize> = (0..para.controls.len()).collect();
            if !forward {
                order.reverse();
            }
            order.into_iter().find_map(|ci| {
                let kind = nearest_kind(&para.controls[ci])?;
                accept(ci, positions.get(ci).copied().unwrap_or(0)).then_some((ci, kind))
            })
        };

        let mut depth = path.len();
        // 나온 영역 컨트롤의 번호. 없으면 캐럿 글자 자리를 경계로 쓴다.
        let mut bound: Option<usize> = from_control;
        loop {
            let list = lists[depth].0;
            let start = if depth == 0 { host } else { path[depth - 1].2 };
            let paras: Vec<usize> = if forward {
                (start..list.len()).collect()
            } else {
                (0..=start).rev().collect()
            };
            for para_idx in paras {
                let accept = |ci: usize, pos: usize| {
                    para_idx != start
                        || match (bound, forward) {
                            (Some(b), true) => ci > b,
                            (Some(b), false) => ci < b,
                            (None, true) => pos > offset,
                            (None, false) => pos < offset,
                        }
                };
                if let Some((ci, kind)) = pick(&list[para_idx], &accept) {
                    return Ok(hit(place.section, depth, para_idx, ci, kind));
                }
            }
            if depth == 0 {
                if master.is_some() {
                    return Ok(r#"{"type":"none"}"#.to_string());
                }
                let sections: Vec<usize> = if forward {
                    (place.section + 1..self.document.sections.len()).collect()
                } else {
                    (0..place.section).rev().collect()
                };
                for section in sections {
                    let list = &self.document.sections[section].paragraphs;
                    let mut paras: Vec<usize> = (0..list.len()).collect();
                    if !forward {
                        paras.reverse();
                    }
                    for para_idx in paras {
                        if let Some((ci, kind)) = pick(&list[para_idx], &|_, _| true) {
                            return Ok(hit(section, 0, para_idx, ci, kind));
                        }
                    }
                }
                return Ok(r#"{"type":"none"}"#.to_string());
            }
            // 영역을 다 훑었으면 그 영역을 가진 컨트롤로 나온다.
            let host_ci = path[depth - 1].0;
            depth -= 1;
            let parent = if depth == 0 { host } else { path[depth - 1].2 };
            if !forward {
                if let Some(kind @ ("table" | "picture" | "shape" | "equation")) =
                    nearest_kind(&lists[depth].0[parent].controls[host_ci])
                {
                    return Ok(hit(place.section, depth, parent, host_ci, kind));
                }
            }
            bound = Some(host_ci);
        }
    }

    /// 같은 자리에 여러 누름틀이 있어도 명시적으로 고른 컨트롤의 현재 범위를 조회한다.
    pub fn field_info_for_control_at(
        &self,
        place: &BookmarkPlace,
        control_idx: usize,
        char_offset: usize,
    ) -> String {
        let result = self
            .place_lists(place)
            .ok()
            .and_then(|(_, host, path, lists)| {
                let para = lists
                    .last()?
                    .0
                    .get(path.last().map_or(host, |entry| entry.2))?;
                super::field_query::click_here_info(para, Some(control_idx), char_offset)
            });
        result.unwrap_or_else(|| r#"{"inField":false}"#.into())
    }

    /// `setActiveFieldAt`: 주소 문단의 `control_idx` 누름틀을 활성 필드로 둔다. 같은 글자 자리에 누름틀이
    /// 여럿이면 글자 자리로는 하나를 가릴 수 없다. 고른 누름틀을 활성으로 두면 그 자리 입력이 그 누름틀에 든다.
    /// 누름틀이 아니면 바꾸지 않고 false를 돌려준다.
    pub fn set_active_field_at(
        &mut self,
        place: &BookmarkPlace,
        control_idx: usize,
    ) -> Result<bool, HwpError> {
        let (master, host, path, lists) = self.place_lists(place)?;
        if master.is_some() {
            return Ok(false);
        }
        let para_idx = path.last().map_or(host, |entry| entry.2);
        let is_click_here = matches!(
            lists.last().and_then(|(list, _)| list[para_idx].controls.get(control_idx)),
            Some(Control::Field(field)) if field.field_type == FieldType::ClickHere
        );
        if !is_click_here {
            return Ok(false);
        }
        let info = crate::document_core::ActiveFieldInfo {
            section_idx: place.section,
            para_idx: host,
            control_idx,
            cell_path: (!path.is_empty()).then_some(path),
        };
        if self.active_field.as_ref() == Some(&info) {
            return Ok(false);
        }
        self.active_field = Some(info);
        self.invalidate_page_tree_cache();
        Ok(true)
    }
}

fn invalid(message: &str) -> HwpError {
    HwpError::InvalidField(message.to_string())
}

/// 문단 내 char_offset에 해당하는 컨트롤 삽입 위치를 결정
fn find_control_insert_index(
    para: &crate::model::paragraph::Paragraph,
    char_offset: usize,
) -> usize {
    let positions = find_control_text_positions(para);
    // char_offset보다 큰 위치를 가진 첫 번째 컨트롤의 인덱스
    for (i, &pos) in positions.iter().enumerate() {
        if pos > char_offset {
            return i;
        }
    }
    para.controls.len()
}

/// 책갈피 CTRL_DATA 바이너리 생성 (ParameterSet 형식)
///
/// 구조: ps_id(2) + count(2) + dummy(2) + item_id(2) + item_type(2) + name_len(2) + name(UTF-16LE)
fn build_bookmark_ctrl_data(name: &str) -> Vec<u8> {
    let utf16: Vec<u16> = name.encode_utf16().collect();
    let mut data = Vec::with_capacity(12 + utf16.len() * 2);
    data.extend_from_slice(&0x021Bu16.to_le_bytes()); // ps_id
    data.extend_from_slice(&1i16.to_le_bytes()); // count = 1
    data.extend_from_slice(&0u16.to_le_bytes()); // dummy
    data.extend_from_slice(&0x4000u16.to_le_bytes()); // item_id
    data.extend_from_slice(&1u16.to_le_bytes()); // item_type = String
    data.extend_from_slice(&(utf16.len() as u16).to_le_bytes()); // name_len
    for &ch in &utf16 {
        data.extend_from_slice(&ch.to_le_bytes());
    }
    data
}

/// JSON 문자열 이스케이프
fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                out.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

#[cfg(test)]
mod tests {
    //! 책갈피 추가/삭제/이름변경의 raw_stream 무효화 회귀 테스트.
    //!
    //! serialize_section(serializer/body_text.rs)은 raw_stream 이 Some 이면 IR 을 무시하고
    //! 원본 바이트를 그대로 반환한다. HWP5 파서는 모든 섹션에 raw_stream 을 채우므로
    //! (parser/mod.rs), 세 뮤테이터가 raw_stream 을 비우지 않으면 — 책갈피 추가·삭제·
    //! 이름변경만 하고 저장하는 워크플로에서 — 편집이 저장 시 통째로 유실된다.
    //! recompose_section 은 화면만 갱신할 뿐 raw_stream 을 건드리지 않는다.
    //! 누름틀 set_field_value_*(field_query.rs)·양식 set_form_value_*(form_query.rs)는
    //! 같은 불변식을 이미 지킨다.

    use crate::document_core::DocumentCore;
    use crate::model::control::{Bookmark, Control};
    use crate::model::document::{Document, Section};
    use crate::model::paragraph::Paragraph;
    use crate::serializer::body_text::serialize_section;

    const SENTINEL: u8 = 0xAB;

    fn core_from(doc: Document) -> DocumentCore {
        let mut core = DocumentCore::new_empty();
        core.document = doc;
        core.composed = vec![Vec::new()];
        core.dirty_sections = vec![true];
        core.dirty_paragraphs = vec![None];
        core
    }

    /// 파싱된 문서를 흉내낸다 — 섹션이 원본 스트림 바이트를 물고 있는 상태.
    fn doc_with_raw_stream(paragraphs: Vec<Paragraph>) -> Document {
        let mut doc = Document::default();
        doc.sections.push(Section {
            paragraphs,
            raw_stream: Some(vec![SENTINEL; 64]),
            ..Default::default()
        });
        doc
    }

    fn text_para(text: &str) -> Paragraph {
        let mut p = Paragraph {
            text: text.to_string(),
            char_offsets: (0..text.chars().count() as u32).collect(),
            char_count: text.chars().count() as u32,
            ..Default::default()
        };
        p.has_para_text = true;
        p
    }

    fn para_with_bookmark(name: &str) -> Paragraph {
        let mut p = Paragraph {
            // 책갈피 8유닛 + 문단 끝 1유닛
            char_count: 9,
            ..Default::default()
        };
        p.controls.push(Control::Bookmark(Bookmark {
            name: name.to_string(),
        }));
        p
    }

    #[test]
    fn add_bookmark_invalidates_raw_stream() {
        let mut core = core_from(doc_with_raw_stream(vec![text_para("안녕하세요")]));
        let r = core
            .add_bookmark_native(0, 0, 2, "중간지점")
            .expect("호출 성공");
        assert!(r.contains(r#""ok":true"#), "전제: 책갈피 추가 성공 ({r})");

        assert!(
            core.document.sections[0].raw_stream.is_none(),
            "raw_stream 이 남으면 추가한 책갈피가 저장 시 사라진다"
        );
        let out = serialize_section(&core.document.sections[0]);
        assert_ne!(
            out,
            vec![SENTINEL; 64],
            "직렬화가 원본 바이트를 반환하면 유실"
        );
    }

    #[test]
    fn delete_bookmark_invalidates_raw_stream() {
        let mut core = core_from(doc_with_raw_stream(vec![para_with_bookmark("삭제대상")]));
        let r = core.delete_bookmark_native(0, 0, 0).expect("호출 성공");
        assert!(r.contains(r#""ok":true"#), "전제: 책갈피 삭제 성공 ({r})");

        assert!(
            core.document.sections[0].raw_stream.is_none(),
            "raw_stream 이 남으면 삭제한 책갈피가 저장 시 되살아난다"
        );
    }

    #[test]
    fn rename_bookmark_invalidates_raw_stream() {
        let mut core = core_from(doc_with_raw_stream(vec![para_with_bookmark("옛이름")]));
        let r = core
            .rename_bookmark_native(0, 0, 0, "새이름")
            .expect("호출 성공");
        assert!(r.contains(r#""ok":true"#), "전제: 이름 변경 성공 ({r})");

        // 이름은 IR 에 반영됐고,
        match &core.document.sections[0].paragraphs[0].controls[0] {
            Control::Bookmark(b) => assert_eq!(b.name, "새이름"),
            _ => panic!("책갈피여야 함"),
        }
        // raw_stream 은 무효화돼야 한다 — 남으면 저장 시 옛 이름으로 되돌아간다.
        assert!(
            core.document.sections[0].raw_stream.is_none(),
            "raw_stream 이 남으면 이름 변경이 저장 시 옛 이름으로 되돌아간다"
        );
    }
}

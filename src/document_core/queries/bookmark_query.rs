//! 책갈피 조회/조작 기능

use crate::document_core::helpers::find_control_text_positions;
use crate::document_core::DocumentCore;
use crate::error::HwpError;
use crate::model::control::{Bookmark, Control};

/// 책갈피 정보
#[derive(Debug, Clone)]
struct BookmarkInfo {
    name: String,
    sec: usize,
    para: usize,
    ctrl_idx: usize,
    /// 텍스트 내 위치 (정렬용)
    char_pos: usize,
}

impl DocumentCore {
    /// 문서 내 모든 책갈피 목록을 JSON으로 반환
    pub fn get_bookmarks_native(&self) -> Result<String, HwpError> {
        let bookmarks = self.collect_bookmarks();
        let items: Vec<String> = bookmarks
            .iter()
            .map(|b| {
                format!(
                    "{{\"name\":{},\"sec\":{},\"para\":{},\"ctrlIdx\":{},\"charPos\":{}}}",
                    json_escape(&b.name),
                    b.sec,
                    b.para,
                    b.ctrl_idx,
                    b.char_pos
                )
            })
            .collect();
        Ok(format!("[{}]", items.join(",")))
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

        // char_offset에 해당하는 컨트롤 삽입 위치 결정
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

        // char_offsets에 컨트롤 위치 정보 추가
        if !paragraph.char_offsets.is_empty() {
            let raw_offset = char_offset_to_raw(paragraph, char_offset, insert_idx);
            paragraph.char_offsets.insert(insert_idx, raw_offset);
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

        paragraph.controls.remove(ctrl_idx);
        if ctrl_idx < paragraph.ctrl_data_records.len() {
            paragraph.ctrl_data_records.remove(ctrl_idx);
        }
        if ctrl_idx < paragraph.char_offsets.len() {
            paragraph.char_offsets.remove(ctrl_idx);
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
            Ok(r#"{"ok":true}"#.to_string())
        } else {
            Ok(r#"{"ok":false,"error":"해당 컨트롤이 책갈피가 아닙니다."}"#.to_string())
        }
    }

    /// 내부: 모든 책갈피 수집 (중첩 구조 포함)
    fn collect_bookmarks(&self) -> Vec<BookmarkInfo> {
        let mut result = vec![];
        for (sec_idx, section) in self.document.sections.iter().enumerate() {
            collect_bookmarks_from_paragraphs(&section.paragraphs, sec_idx, None, &mut result);
        }
        result
    }
}

/// 문단 목록에서 책갈피를 재귀적으로 수집 (표 셀, 글상자 등 중첩 구조 포함)
///
/// `host_para`: 중첩 구조의 경우 소속 최상위 문단 인덱스. None이면 최상위 레벨.
fn collect_bookmarks_from_paragraphs(
    paragraphs: &[crate::model::paragraph::Paragraph],
    sec: usize,
    host_para: Option<usize>,
    result: &mut Vec<BookmarkInfo>,
) {
    for (para_idx, para) in paragraphs.iter().enumerate() {
        // 최상위 레벨이면 para_idx 사용, 중첩이면 호스트 문단 인덱스 유지
        let effective_para = host_para.unwrap_or(para_idx);
        let positions = find_control_text_positions(para);
        for (ctrl_idx, ctrl) in para.controls.iter().enumerate() {
            match ctrl {
                Control::Bookmark(bm) => {
                    let char_pos = if host_para.is_some() {
                        // 중첩 구조 내 책갈피: 호스트 문단 시작점으로 이동
                        0
                    } else {
                        positions.get(ctrl_idx).copied().unwrap_or(0)
                    };
                    result.push(BookmarkInfo {
                        name: bm.name.clone(),
                        sec,
                        para: effective_para,
                        ctrl_idx,
                        char_pos,
                    });
                }
                Control::Table(t) => {
                    for cell in &t.cells {
                        collect_bookmarks_from_paragraphs(
                            &cell.paragraphs,
                            sec,
                            Some(effective_para),
                            result,
                        );
                    }
                }
                Control::Header(h) => {
                    collect_bookmarks_from_paragraphs(
                        &h.paragraphs,
                        sec,
                        Some(effective_para),
                        result,
                    );
                }
                Control::Footer(f) => {
                    collect_bookmarks_from_paragraphs(
                        &f.paragraphs,
                        sec,
                        Some(effective_para),
                        result,
                    );
                }
                Control::Footnote(n) => {
                    collect_bookmarks_from_paragraphs(
                        &n.paragraphs,
                        sec,
                        Some(effective_para),
                        result,
                    );
                }
                Control::Endnote(n) => {
                    collect_bookmarks_from_paragraphs(
                        &n.paragraphs,
                        sec,
                        Some(effective_para),
                        result,
                    );
                }
                Control::HiddenComment(hc) => {
                    collect_bookmarks_from_paragraphs(
                        &hc.paragraphs,
                        sec,
                        Some(effective_para),
                        result,
                    );
                }
                _ => {}
            }
        }
    }
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

/// char_offset을 raw char_offset (파서 원본 기준)으로 변환
fn char_offset_to_raw(
    para: &crate::model::paragraph::Paragraph,
    char_offset: usize,
    insert_idx: usize,
) -> u32 {
    // 기존 char_offsets에서 삽입 위치 주변의 raw offset을 참조
    if insert_idx > 0 && insert_idx <= para.char_offsets.len() {
        // 이전 컨트롤의 raw offset + 8 (컨트롤 문자 크기)
        para.char_offsets[insert_idx - 1] + 8
    } else if !para.char_offsets.is_empty() {
        // 첫 위치에 삽입: 기존 첫 번째보다 작은 값
        let first = para.char_offsets[0];
        if first >= 8 {
            first - 8
        } else {
            0
        }
    } else {
        // char_offsets가 비어있으면 char_offset * 2 (UTF-16 추정)
        (char_offset * 2) as u32
    }
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

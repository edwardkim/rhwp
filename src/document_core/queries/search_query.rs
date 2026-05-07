//! 문서 텍스트 검색/치환 기능
//!
//! 본문, 표 셀, 글상자 등 중첩 컨트롤 내부 텍스트를 포함한 전체 검색.

use crate::document_core::DocumentCore;
use crate::document_core::helpers::get_textbox_from_shape;
use crate::error::HwpError;
use crate::model::control::Control;

/// 검색 결과 위치 정보
#[derive(Debug, Clone)]
struct SearchHit {
    sec: usize,
    para: usize,
    char_offset: usize,
    length: usize,
    /// 표 셀 등 중첩 컨텍스트: (parent_para, ctrl_idx, cell_idx, cell_para)
    cell_context: Option<(usize, usize, usize, usize)>,
}

/// 문단 텍스트에서 query를 검색하여 모든 매치 오프셋을 반환
fn find_in_text(text: &str, query: &str, case_sensitive: bool) -> Vec<usize> {
    if query.is_empty() || text.is_empty() {
        return vec![];
    }
    let mut results = vec![];
    if case_sensitive {
        let chars: Vec<char> = text.chars().collect();
        let qchars: Vec<char> = query.chars().collect();
        let qlen = qchars.len();
        if chars.len() < qlen { return results; }
        for i in 0..=chars.len() - qlen {
            if chars[i..i + qlen] == qchars[..] {
                results.push(i);
            }
        }
    } else {
        let text_lower: String = text.chars().flat_map(|c| c.to_lowercase()).collect();
        let query_lower: String = query.chars().flat_map(|c| c.to_lowercase()).collect();
        let chars: Vec<char> = text_lower.chars().collect();
        let qchars: Vec<char> = query_lower.chars().collect();
        let qlen = qchars.len();
        if chars.len() < qlen { return results; }
        for i in 0..=chars.len() - qlen {
            if chars[i..i + qlen] == qchars[..] {
                results.push(i);
            }
        }
    }
    results
}

/// 문서 본문에서 query의 첫 번째 매치만 반환 (표/글상자 내부 제외, early-exit)
fn search_first_body(doc: &DocumentCore, query: &str, case_sensitive: bool) -> Option<SearchHit> {
    let qlen = query.chars().count();
    for (sec_idx, section) in doc.document.sections.iter().enumerate() {
        for (para_idx, para) in section.paragraphs.iter().enumerate() {
            if let Some(&offset) = find_in_text(&para.text, query, case_sensitive).first() {
                return Some(SearchHit {
                    sec: sec_idx,
                    para: para_idx,
                    char_offset: offset,
                    length: qlen,
                    cell_context: None,
                });
            }
        }
    }
    None
}

/// 문서 전체를 순회하며 query와 일치하는 모든 위치를 반환
fn search_all(doc: &DocumentCore, query: &str, case_sensitive: bool) -> Vec<SearchHit> {
    let mut results = vec![];
    let qlen = query.chars().count();

    for (sec_idx, section) in doc.document.sections.iter().enumerate() {
        for (para_idx, para) in section.paragraphs.iter().enumerate() {
            // 본문 문단
            for offset in find_in_text(&para.text, query, case_sensitive) {
                results.push(SearchHit {
                    sec: sec_idx,
                    para: para_idx,
                    char_offset: offset,
                    length: qlen,
                    cell_context: None,
                });
            }

            // 표 셀
            for (ctrl_idx, ctrl) in para.controls.iter().enumerate() {
                match ctrl {
                    Control::Table(table) => {
                        for (cell_idx, cell) in table.cells.iter().enumerate() {
                            for (cell_para_idx, cell_para) in cell.paragraphs.iter().enumerate() {
                                for offset in find_in_text(&cell_para.text, query, case_sensitive) {
                                    results.push(SearchHit {
                                        sec: sec_idx,
                                        para: para_idx,
                                        char_offset: offset,
                                        length: qlen,
                                        cell_context: Some((para_idx, ctrl_idx, cell_idx, cell_para_idx)),
                                    });
                                }
                            }
                        }
                    }
                    Control::Shape(shape) => {
                        if let Some(tb) = get_textbox_from_shape(shape) {
                            for (tb_para_idx, tb_para) in tb.paragraphs.iter().enumerate() {
                                for offset in find_in_text(&tb_para.text, query, case_sensitive) {
                                    results.push(SearchHit {
                                        sec: sec_idx,
                                        para: para_idx,
                                        char_offset: offset,
                                        length: qlen,
                                        cell_context: Some((para_idx, ctrl_idx, 0, tb_para_idx)),
                                    });
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    results
}

impl DocumentCore {
    /// 문서 텍스트 검색
    ///
    /// from_sec/from_para/from_char: 검색 시작 위치
    /// forward: true=정방향, false=역방향
    /// case_sensitive: 대소문자 구분
    /// cell_context_json: 표 셀 내부에서 시작할 경우 JSON
    ///
    /// 반환: JSON `{"found":true,"sec":0,"para":1,"charOffset":5,"length":3,"cellContext":...}`
    pub fn search_text_native(
        &self,
        query: &str,
        from_sec: usize,
        from_para: usize,
        from_char: usize,
        forward: bool,
        case_sensitive: bool,
    ) -> Result<String, HwpError> {
        if query.is_empty() {
            return Ok(r#"{"found":false}"#.to_string());
        }

        let all_hits = search_all(self, query, case_sensitive);
        if all_hits.is_empty() {
            return Ok(r#"{"found":false}"#.to_string());
        }

        // 본문 결과만 필터 (셀/글상자 내부 제외 — 커서 이동 불가)
        let body_hits: Vec<&SearchHit> = all_hits.iter()
            .filter(|h| h.cell_context.is_none())
            .collect();
        if body_hits.is_empty() {
            return Ok(r#"{"found":false}"#.to_string());
        }

        if forward {
            let after = body_hits.iter().find(|h| {
                h.sec > from_sec
                || (h.sec == from_sec && h.para > from_para)
                || (h.sec == from_sec && h.para == from_para && h.char_offset > from_char)
            });
            match after {
                Some(h) => Ok(format_search_hit(h, false)),
                None => Ok(format_search_hit(body_hits[0], true)),
            }
        } else {
            let before = body_hits.iter().rev().find(|h| {
                h.sec < from_sec
                || (h.sec == from_sec && h.para < from_para)
                || (h.sec == from_sec && h.para == from_para && h.char_offset < from_char)
            });
            match before {
                Some(h) => Ok(format_search_hit(h, false)),
                None => Ok(format_search_hit(body_hits[body_hits.len() - 1], true)),
            }
        }
    }

    /// 텍스트 치환 (단일)
    ///
    /// 검색 결과 위치의 텍스트를 new_text로 교체한다.
    pub fn replace_text_native(
        &mut self,
        sec: usize,
        para: usize,
        char_offset: usize,
        length: usize,
        new_text: &str,
    ) -> Result<String, HwpError> {
        // 삭제 후 삽입
        self.delete_text_native(sec, para, char_offset, length)?;
        self.insert_text_native(sec, para, char_offset, new_text)?;
        let new_len = new_text.chars().count();
        Ok(format!(
            "{{\"ok\":true,\"charOffset\":{},\"newLength\":{}}}",
            char_offset, new_len
        ))
    }

    /// 단일 치환 (검색어 기반)
    ///
    /// 문서 본문에서 query의 첫 번째 매치를 new_text로 교체한다.
    /// 표/글상자 내부는 대상에서 제외 (search_text_native와 동일 범위).
    /// 반환: JSON `{"ok":true,"sec":N,"para":N,"charOffset":N,"newLength":N}` 또는 `{"ok":false}`
    pub fn replace_one_native(
        &mut self,
        query: &str,
        new_text: &str,
        case_sensitive: bool,
    ) -> Result<String, HwpError> {
        if query.is_empty() {
            return Ok(r#"{"ok":false}"#.to_string());
        }

        let hit = match search_first_body(self, query, case_sensitive) {
            Some(h) => h,
            None => return Ok(r#"{"ok":false}"#.to_string()),
        };

        let new_len = new_text.chars().count();
        self.delete_text_native(hit.sec, hit.para, hit.char_offset, hit.length)?;
        self.insert_text_native(hit.sec, hit.para, hit.char_offset, new_text)?;

        Ok(format!(
            "{{\"ok\":true,\"sec\":{},\"para\":{},\"charOffset\":{},\"newLength\":{}}}",
            hit.sec, hit.para, hit.char_offset, new_len
        ))
    }

    /// 전체 치환
    ///
    /// 문서 전체에서 query를 new_text로 모두 교체한다.
    /// 반환: JSON `{"ok":true,"count":N}`
    pub fn replace_all_native(
        &mut self,
        query: &str,
        new_text: &str,
        case_sensitive: bool,
    ) -> Result<String, HwpError> {
        if query.is_empty() {
            return Ok(r#"{"ok":true,"count":0}"#.to_string());
        }

        // 모든 매치를 찾되, 역순으로 치환 (오프셋 변동 방지)
        let mut all_hits = search_all(self, query, case_sensitive);
        // 역순 정렬: 뒤에서부터 치환하여 앞쪽 오프셋에 영향 없도록
        all_hits.reverse();

        let count = all_hits.len();

        for hit in &all_hits {
            if let Some((parent_para, ctrl_idx, cell_idx, cell_para_idx)) = hit.cell_context {
                // 표 셀 내부 치환
                let section = self.document.sections.get_mut(hit.sec)
                    .ok_or_else(|| HwpError::RenderError("구역 범위 초과".into()))?;
                let para = section.paragraphs.get_mut(parent_para)
                    .ok_or_else(|| HwpError::RenderError("문단 범위 초과".into()))?;

                let cell_para = match para.controls.get_mut(ctrl_idx) {
                    Some(Control::Table(table)) => {
                        let cell = table.cells.get_mut(cell_idx)
                            .ok_or_else(|| HwpError::RenderError("셀 범위 초과".into()))?;
                        cell.paragraphs.get_mut(cell_para_idx)
                            .ok_or_else(|| HwpError::RenderError("셀 문단 범위 초과".into()))?
                    }
                    Some(Control::Shape(shape)) => {
                        let tb = crate::document_core::helpers::get_textbox_from_shape_mut(shape)
                            .ok_or_else(|| HwpError::RenderError("글상자 없음".into()))?;
                        tb.paragraphs.get_mut(cell_para_idx)
                            .ok_or_else(|| HwpError::RenderError("글상자 문단 범위 초과".into()))?
                    }
                    _ => continue,
                };
                cell_para.delete_text_at(hit.char_offset, hit.length);
                cell_para.insert_text_at(hit.char_offset, new_text);
            } else {
                // 본문 문단 치환 — delete_text_native + insert_text_native는 recompose를 호출하므로
                // 성능을 위해 직접 문단 수준 조작 후 마지막에 일괄 recompose
                let section = self.document.sections.get_mut(hit.sec)
                    .ok_or_else(|| HwpError::RenderError("구역 범위 초과".into()))?;
                let para = section.paragraphs.get_mut(hit.para)
                    .ok_or_else(|| HwpError::RenderError("문단 범위 초과".into()))?;
                para.delete_text_at(hit.char_offset, hit.length);
                para.insert_text_at(hit.char_offset, new_text);
            }
        }

        // 변경된 섹션들 recompose
        if count > 0 {
            let mut affected_sections: Vec<usize> = all_hits.iter().map(|h| h.sec).collect();
            affected_sections.sort();
            affected_sections.dedup();
            for sec_idx in affected_sections {
                self.recompose_section(sec_idx);
            }
        }

        Ok(format!("{{\"ok\":true,\"count\":{}}}", count))
    }

    /// 글로벌 쪽 번호에 해당하는 첫 번째 문단 위치를 반환
    pub fn get_position_of_page_native(
        &self,
        global_page: usize,
    ) -> Result<String, HwpError> {
        let mut page_offset = 0usize;
        for (sec_idx, pr) in self.pagination.iter().enumerate() {
            for page in &pr.pages {
                if page_offset == global_page {
                    // 이 페이지의 첫 번째 PageItem에서 para_index 추출
                    for col in &page.column_contents {
                        for item in &col.items {
                            let pi = match item {
                                crate::renderer::pagination::PageItem::FullParagraph { para_index } => Some(*para_index),
                                crate::renderer::pagination::PageItem::PartialParagraph { para_index, .. } => Some(*para_index),
                                crate::renderer::pagination::PageItem::Table { para_index, .. } => Some(*para_index),
                                crate::renderer::pagination::PageItem::PartialTable { para_index, .. } => Some(*para_index),
                                crate::renderer::pagination::PageItem::Shape { para_index, .. } => Some(*para_index),
                            };
                            if let Some(para_idx) = pi {
                                return Ok(format!(
                                    "{{\"ok\":true,\"sec\":{},\"para\":{},\"charOffset\":0}}",
                                    sec_idx, para_idx
                                ));
                            }
                        }
                    }
                    // 빈 페이지 fallback
                    return Ok(format!("{{\"ok\":true,\"sec\":{},\"para\":0,\"charOffset\":0}}", sec_idx));
                }
                page_offset += 1;
            }
        }
        Err(HwpError::RenderError(format!("쪽 번호 {} 범위 초과", global_page)))
    }

    /// 위치에 해당하는 글로벌 쪽 번호를 반환
    pub fn get_page_of_position_native(
        &self,
        section_idx: usize,
        para_idx: usize,
    ) -> Result<String, HwpError> {
        let pages = self.find_pages_for_paragraph(section_idx, para_idx)?;
        let page = pages.first().copied().unwrap_or(0);
        Ok(format!("{{\"ok\":true,\"page\":{}}}", page))
    }
}

fn format_search_hit(hit: &SearchHit, wrapped: bool) -> String {
    let cell_ctx = match &hit.cell_context {
        Some((pp, ci, cell, cp)) => format!(
            ",\"cellContext\":{{\"parentPara\":{},\"ctrlIdx\":{},\"cellIdx\":{},\"cellPara\":{}}}",
            pp, ci, cell, cp
        ),
        None => String::new(),
    };
    format!(
        "{{\"found\":true,\"wrapped\":{},\"sec\":{},\"para\":{},\"charOffset\":{},\"length\":{}{}}}",
        wrapped, hit.sec, hit.para, hit.char_offset, hit.length, cell_ctx
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_in_text_case_sensitive() {
        assert_eq!(find_in_text("hello world", "world", true), vec![6]);
        assert_eq!(find_in_text("hello world", "World", true), Vec::<usize>::new());
    }

    #[test]
    fn find_in_text_case_insensitive() {
        assert_eq!(find_in_text("Hello World", "hello", false), vec![0]);
        assert_eq!(find_in_text("Hello World", "WORLD", false), vec![6]);
    }

    #[test]
    fn find_in_text_multiple_matches() {
        assert_eq!(find_in_text("abcabc", "abc", true), vec![0, 3]);
    }

    #[test]
    fn find_in_text_empty_inputs() {
        assert_eq!(find_in_text("", "abc", true), Vec::<usize>::new());
        assert_eq!(find_in_text("abc", "", true), Vec::<usize>::new());
    }

    #[test]
    fn find_in_text_korean() {
        assert_eq!(find_in_text("안녕하세요 세계", "세계", true), vec![6]);
        assert_eq!(find_in_text("가나가나", "가나", true), vec![0, 2]);
    }
}

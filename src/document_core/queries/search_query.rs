//! 문서 텍스트 검색/치환 기능
//!
//! 본문, 표 셀, 글상자 등 중첩 컨트롤 내부 텍스트를 포함한 전체 검색.

use crate::document_core::helpers::get_textbox_from_shape;
use crate::document_core::DocumentCore;
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
    /// `cell_context`가 표 셀이 아니라 글상자 문단을 가리키는지 여부.
    /// Find/F3의 새 opt-in은 표 셀 좌표만 이동·치환할 수 있으므로 이 둘을 구분한다.
    is_text_box: bool,
    /// 수식 script 안의 매치이면, 해당 문단 controls 안의 Equation 인덱스.
    /// `cell_context`가 있으면 그 셀/글상자 문단 안의 인덱스이고, 없으면 본문 문단 인덱스다.
    equation_control: Option<usize>,
}

fn replace_char_range(text: &mut String, offset: usize, length: usize, replacement: &str) {
    let mut chars: Vec<char> = text.chars().collect();
    let start = offset.min(chars.len());
    let end = start.saturating_add(length).min(chars.len());
    chars.splice(start..end, replacement.chars());
    *text = chars.into_iter().collect();
}

/// 문단 텍스트에서 query를 검색하여 모든 매치 오프셋을 반환한다.
///
/// [#3283] `grep`(주소를 가진 검색)이 같은 매칭 규칙을 쓰도록 크레이트에 공개한다 —
/// 검색과 치환이 다른 규칙을 쓰면 "찾았는데 못 바꾸는" 어긋남이 생긴다.
pub(crate) fn find_matches(text: &str, query: &str, case_sensitive: bool) -> Vec<usize> {
    find_in_text(text, query, case_sensitive)
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
        if chars.len() < qlen {
            return results;
        }
        for i in 0..=chars.len() - qlen {
            if chars[i..i + qlen] == qchars[..] {
                results.push(i);
            }
        }
    } else {
        // `to_lowercase()` 는 한 원문 문자를 여러 문자로 확장할 수 있다(예: `İ` →
        // `i` + COMBINING DOT ABOVE). lower-case 버퍼의 인덱스를 그대로 반환하면
        // 호출자가 기대하는 원문 문자 오프셋이 밀린다. 각 확장 문자를 원문 문자
        // 인덱스에 연결해 검색은 folded 텍스트에서 하되 주소는 원문 기준으로 돌려준다.
        let chars: Vec<(char, usize)> = text
            .chars()
            .enumerate()
            .flat_map(|(original_offset, c)| {
                c.to_lowercase().map(move |lower| (lower, original_offset))
            })
            .collect();
        let query_lower: String = query.chars().flat_map(|c| c.to_lowercase()).collect();
        let qchars: Vec<char> = query_lower.chars().collect();
        let qlen = qchars.len();
        if chars.len() < qlen {
            return results;
        }
        for i in 0..=chars.len() - qlen {
            if chars[i..i + qlen]
                .iter()
                .map(|(c, _)| *c)
                .eq(qchars.iter().copied())
            {
                results.push(chars[i].1);
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
                    is_text_box: false,
                    equation_control: None,
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
                    is_text_box: false,
                    equation_control: None,
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
                                        cell_context: Some((
                                            para_idx,
                                            ctrl_idx,
                                            cell_idx,
                                            cell_para_idx,
                                        )),
                                        is_text_box: false,
                                        equation_control: None,
                                    });
                                }
                                for (equation_index, control) in
                                    cell_para.controls.iter().enumerate()
                                {
                                    if let Control::Equation(equation) = control {
                                        for offset in
                                            find_in_text(&equation.script, query, case_sensitive)
                                        {
                                            results.push(SearchHit {
                                                sec: sec_idx,
                                                para: para_idx,
                                                char_offset: offset,
                                                length: qlen,
                                                cell_context: Some((
                                                    para_idx,
                                                    ctrl_idx,
                                                    cell_idx,
                                                    cell_para_idx,
                                                )),
                                                is_text_box: false,
                                                equation_control: Some(equation_index),
                                            });
                                        }
                                    }
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
                                        is_text_box: true,
                                        equation_control: None,
                                    });
                                }
                                for (equation_index, control) in tb_para.controls.iter().enumerate()
                                {
                                    if let Control::Equation(equation) = control {
                                        for offset in
                                            find_in_text(&equation.script, query, case_sensitive)
                                        {
                                            results.push(SearchHit {
                                                sec: sec_idx,
                                                para: para_idx,
                                                char_offset: offset,
                                                length: qlen,
                                                cell_context: Some((
                                                    para_idx,
                                                    ctrl_idx,
                                                    0,
                                                    tb_para_idx,
                                                )),
                                                is_text_box: true,
                                                equation_control: Some(equation_index),
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }
                    // 수식 스크립트 — 렌더 트리(EquationNode)가 아니라 IR을 직접 순회하므로
                    // #3419(export-text/markdown 쪽 수식 텍스트화)와는 별개 경로. 셀/글상자와
                    // 별도 equation_control 로 표시해 커서 이동 대상에서는 제외한다.
                    Control::Equation(eq) => {
                        for offset in find_in_text(&eq.script, query, case_sensitive) {
                            results.push(SearchHit {
                                sec: sec_idx,
                                para: para_idx,
                                char_offset: offset,
                                length: qlen,
                                cell_context: None,
                                is_text_box: false,
                                equation_control: Some(ctrl_idx),
                            });
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
        include_cells: bool,
    ) -> Result<String, HwpError> {
        if query.is_empty() {
            return Ok(r#"{"found":false}"#.to_string());
        }

        let all_hits = search_all(self, query, case_sensitive);
        if all_hits.is_empty() {
            return Ok(r#"{"found":false}"#.to_string());
        }

        // [#3865] 표 셀 안에만 있는 단어가 "결과 없음"으로 나오던 원인이 이 필터였다.
        // 종전 주석의 사유는 "커서 이동 불가"였지만, 그 뒤 편집기가 셀 좌표를 다루게 되어
        // (getCursorRectInCell·DocumentPosition 의 cellIndex 계열) 더는 성립하지 않는다.
        // 다만 셀 히트를 받으면 호출자가 셀 좌표로 이동할 수 있어야 하므로, 옵트인으로 연다 —
        // 기본값은 종전 동작 그대로라 기존 호출자는 무회귀다.
        //
        // 셀 히트의 sec·para 는 표가 놓인 **바깥 문단** 좌표이고, 셀 안 위치는 cellContext
        // (parentPara·ctrlIdx·cellIdx·cellPara)로 따로 실린다. 그래서 아래 전/후 판정은
        // 셀 히트에서도 그대로 성립한다(표가 놓인 문단 기준으로 정렬된다).
        let body_hits: Vec<&SearchHit> = if include_cells {
            // Find/F3가 이동·치환할 수 있는 것은 표 셀의 일반 텍스트뿐이다. 글상자와
            // 수식은 `cellContext`만으로는 표와 구분하거나 안전하게 편집할 수 없으므로
            // 기존 제외 범위를 유지한다.
            all_hits
                .iter()
                .filter(|h| !h.is_text_box && h.equation_control.is_none())
                .collect()
        } else {
            all_hits
                .iter()
                .filter(|h| h.cell_context.is_none() && h.equation_control.is_none())
                .collect()
        };
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

    /// 문서 전체 검색 (모든 매치 반환)
    ///
    /// 본문 문단의 모든 매치를 배열로 반환한다. 표/글상자 내부 포함 여부는
    /// include_cells 파라미터로 결정.
    ///
    /// 반환: JSON `[{"sec":0,"para":1,"charOffset":5,"length":3,"cellContext":...}, ...]`
    pub fn search_all_text_native(
        &self,
        query: &str,
        case_sensitive: bool,
        include_cells: bool,
    ) -> Result<String, HwpError> {
        if query.is_empty() {
            return Ok("[]".to_string());
        }

        let all_hits = search_all(self, query, case_sensitive);
        if all_hits.is_empty() {
            return Ok("[]".to_string());
        }

        let hits: Vec<&SearchHit> = if include_cells {
            all_hits.iter().collect()
        } else {
            all_hits
                .iter()
                .filter(|h| h.cell_context.is_none() && h.equation_control.is_none())
                .collect()
        };

        let mut json_parts: Vec<String> = Vec::with_capacity(hits.len());
        for h in &hits {
            let cell_ctx = match &h.cell_context {
                Some((pp, ci, cell, cp)) => format!(
                    ",\"cellContext\":{{\"parentPara\":{},\"ctrlIdx\":{},\"cellIdx\":{},\"cellPara\":{}}}",
                    pp, ci, cell, cp
                ),
                None => String::new(),
            };
            let equation_ctx = h
                .equation_control
                .map(|control| format!(",\"equationControl\":{}", control))
                .unwrap_or_default();
            json_parts.push(format!(
                "{{\"sec\":{},\"para\":{},\"charOffset\":{},\"length\":{}{}{}}}",
                h.sec, h.para, h.char_offset, h.length, cell_ctx, equation_ctx
            ));
        }

        Ok(format!("[{}]", json_parts.join(",")))
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
        self.replace_matches_native(query, new_text, case_sensitive, None)
    }

    /// [#3395] 문서 순서 k번째(0 기준) 매치 **하나만** 치환한다. 실물 서식의 체크박스
    /// (□ 19개 중 k번째만 ☑)처럼 같은 문자가 여럿일 때 전량 치환은 문서를 망가뜨린다.
    /// 몸통은 replace_all_native 와 동일 경로를 재사용한다 — 새 편집 로직 없음.
    pub fn replace_nth_native(
        &mut self,
        query: &str,
        new_text: &str,
        case_sensitive: bool,
        occurrence: usize,
    ) -> Result<String, HwpError> {
        self.replace_matches_native(query, new_text, case_sensitive, Some(occurrence))
    }

    fn replace_matches_native(
        &mut self,
        query: &str,
        new_text: &str,
        case_sensitive: bool,
        occurrence: Option<usize>,
    ) -> Result<String, HwpError> {
        if query.is_empty() {
            return Ok(r#"{"ok":true,"count":0}"#.to_string());
        }

        // 모든 매치를 찾되, 역순으로 치환 (오프셋 변동 방지)
        let mut all_hits = search_all(self, query, case_sensitive);
        if let Some(n) = occurrence {
            // 문서 순서 k번째 하나만 남긴다. 범위를 벗어나면 count 0 (판정은 데이터).
            all_hits = match all_hits.into_iter().nth(n) {
                Some(hit) => vec![hit],
                None => Vec::new(),
            };
        }
        // 역순 정렬: 뒤에서부터 치환하여 앞쪽 오프셋에 영향 없도록
        all_hits.reverse();

        let mut count = 0usize;
        let mut affected_sections: Vec<usize> = Vec::new();

        for hit in &all_hits {
            if let Some((parent_para, ctrl_idx, cell_idx, cell_para_idx)) = hit.cell_context {
                // 표 셀 내부 치환
                let section = self
                    .document
                    .sections
                    .get_mut(hit.sec)
                    .ok_or_else(|| HwpError::RenderError("구역 범위 초과".into()))?;
                let para = section
                    .paragraphs
                    .get_mut(parent_para)
                    .ok_or_else(|| HwpError::RenderError("문단 범위 초과".into()))?;

                let nested_para = match para.controls.get_mut(ctrl_idx) {
                    Some(Control::Table(table)) => {
                        let cell = table
                            .cells
                            .get_mut(cell_idx)
                            .ok_or_else(|| HwpError::RenderError("셀 범위 초과".into()))?;
                        cell.paragraphs
                            .get_mut(cell_para_idx)
                            .ok_or_else(|| HwpError::RenderError("셀 문단 범위 초과".into()))?
                    }
                    Some(Control::Shape(shape)) => {
                        let tb = crate::document_core::helpers::get_textbox_from_shape_mut(shape)
                            .ok_or_else(|| HwpError::RenderError("글상자 없음".into()))?;
                        tb.paragraphs
                            .get_mut(cell_para_idx)
                            .ok_or_else(|| HwpError::RenderError("글상자 문단 범위 초과".into()))?
                    }
                    _ => continue,
                };
                if let Some(equation_index) = hit.equation_control {
                    let equation = match nested_para.controls.get_mut(equation_index) {
                        Some(Control::Equation(equation)) => equation,
                        _ => {
                            return Err(HwpError::RenderError(
                                "수식 검색 결과의 컨트롤 경로가 유효하지 않음".into(),
                            ));
                        }
                    };
                    replace_char_range(&mut equation.script, hit.char_offset, hit.length, new_text);
                } else {
                    nested_para.delete_text_at(hit.char_offset, hit.length);
                    nested_para.insert_text_at(hit.char_offset, new_text);
                }
                count += 1;
                affected_sections.push(hit.sec);
            } else if let Some(equation_index) = hit.equation_control {
                let section = self
                    .document
                    .sections
                    .get_mut(hit.sec)
                    .ok_or_else(|| HwpError::RenderError("구역 범위 초과".into()))?;
                let para = section
                    .paragraphs
                    .get_mut(hit.para)
                    .ok_or_else(|| HwpError::RenderError("문단 범위 초과".into()))?;
                let equation = match para.controls.get_mut(equation_index) {
                    Some(Control::Equation(equation)) => equation,
                    _ => {
                        return Err(HwpError::RenderError(
                            "수식 검색 결과의 컨트롤 경로가 유효하지 않음".into(),
                        ));
                    }
                };
                replace_char_range(&mut equation.script, hit.char_offset, hit.length, new_text);
                count += 1;
                affected_sections.push(hit.sec);
            } else {
                // 본문 문단 치환 — delete_text_native + insert_text_native는 recompose를 호출하므로
                // 성능을 위해 직접 문단 수준 조작 후 마지막에 일괄 recompose
                let section = self
                    .document
                    .sections
                    .get_mut(hit.sec)
                    .ok_or_else(|| HwpError::RenderError("구역 범위 초과".into()))?;
                let para = section
                    .paragraphs
                    .get_mut(hit.para)
                    .ok_or_else(|| HwpError::RenderError("문단 범위 초과".into()))?;
                para.delete_text_at(hit.char_offset, hit.length);
                para.insert_text_at(hit.char_offset, new_text);
                count += 1;
                affected_sections.push(hit.sec);
            }
        }

        // 변경된 섹션들 recompose
        if count > 0 {
            affected_sections.sort();
            affected_sections.dedup();
            for sec_idx in affected_sections {
                // 편집 시 raw 스트림 무효화 (재직렬화 유도) — 캐시가 남으면 export_hwp가
                // 원본 바이트를 그대로 반환해 치환 결과가 저장에서 유실된다 (#1385)
                self.document.sections[sec_idx].raw_stream = None;
                self.recompose_section(sec_idx);
            }
        }

        Ok(format!("{{\"ok\":true,\"count\":{}}}", count))
    }

    /// 글로벌 쪽 번호에 해당하는 첫 번째 문단 위치를 반환
    pub fn get_position_of_page_native(&self, global_page: usize) -> Result<String, HwpError> {
        let mut page_offset = 0usize;
        for (sec_idx, pr) in self.pagination.iter().enumerate() {
            for page in &pr.pages {
                if page_offset == global_page {
                    // 이 페이지의 첫 번째 PageItem에서 para_index 추출
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
                            if let Some(para_idx) = pi {
                                return Ok(format!(
                                    "{{\"ok\":true,\"sec\":{},\"para\":{},\"charOffset\":0}}",
                                    sec_idx, para_idx
                                ));
                            }
                        }
                    }
                    // 빈 페이지 fallback
                    return Ok(format!(
                        "{{\"ok\":true,\"sec\":{},\"para\":0,\"charOffset\":0}}",
                        sec_idx
                    ));
                }
                page_offset += 1;
            }
        }
        Err(HwpError::RenderError(format!(
            "쪽 번호 {} 범위 초과",
            global_page
        )))
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
    let equation_ctx = hit
        .equation_control
        .map(|control| format!(",\"equationControl\":{}", control))
        .unwrap_or_default();
    format!(
        "{{\"found\":true,\"wrapped\":{},\"sec\":{},\"para\":{},\"charOffset\":{},\"length\":{}{}{}}}",
        wrapped, hit.sec, hit.para, hit.char_offset, hit.length, cell_ctx, equation_ctx
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_in_text_case_sensitive() {
        assert_eq!(find_in_text("hello world", "world", true), vec![6]);
        assert_eq!(
            find_in_text("hello world", "World", true),
            Vec::<usize>::new()
        );
    }

    #[test]
    fn find_in_text_case_insensitive() {
        assert_eq!(find_in_text("Hello World", "hello", false), vec![0]);
        assert_eq!(find_in_text("Hello World", "WORLD", false), vec![6]);
    }

    #[test]
    fn ignore_case_returns_original_char_offset_after_unicode_lowercase_expansion() {
        // `İ`가 두 lower-case 문자로 확장돼도 `stan`의 시작은 원문 2번째 문자다.
        assert_eq!(find_matches("Aİstanbul", "stan", false), vec![2]);
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

    /// [#3413] `search` 가 수식(Equation) 컨트롤의 script 텍스트를 검색 대상에서
    /// 빠뜨리던 버그 회귀 테스트. 실제 표본(`samples/exam_math.hwp`)의 수식에는
    /// `lim` 이 포함돼 있는데도 이전 코드는 matchCount=0 을 반환했다.
    #[test]
    fn search_all_text_finds_equation_script() {
        let data = std::fs::read("samples/exam_math.hwp").expect("샘플 파일 읽기 실패");
        let doc = DocumentCore::from_bytes(&data).expect("샘플 파일 파싱 실패");
        let json = doc
            .search_all_text_native("lim", true, true)
            .expect("검색 실패");
        assert!(
            json.contains("\"charOffset\""),
            "수식 스크립트 내 'lim' 매치를 찾지 못함: {json}"
        );
        assert_ne!(json, "[]", "수식 스크립트 내 'lim' 매치가 비어있음");
        assert!(
            json.contains("\"equationControl\""),
            "수식 매치는 일반 셀 텍스트와 구분되는 주소를 제공해야 함: {json}"
        );
    }

    #[test]
    fn replace_all_updates_equation_scripts_and_reports_actual_count() {
        let data = std::fs::read("samples/exam_math.hwp").expect("샘플 파일 읽기 실패");
        let mut doc = DocumentCore::from_bytes(&data).expect("샘플 파일 파싱 실패");
        let before: serde_json::Value = serde_json::from_str(
            &doc.search_all_text_native("lim", true, true)
                .expect("수식 검색 실패"),
        )
        .expect("수식 검색 JSON 파싱 실패");
        let expected = before.as_array().expect("검색 결과 배열").len();
        assert!(expected > 0, "교체 전 lim 수식이 있어야 함");
        assert_eq!(
            doc.grep("lim", true, None).len(),
            expected,
            "CLI dry-run(grep)과 실제 replace-all의 수식 검색 범위가 같아야 함"
        );

        let result: serde_json::Value = serde_json::from_str(
            &doc.replace_all_native("lim", "LIMIT", true)
                .expect("수식 치환 실패"),
        )
        .expect("수식 치환 JSON 파싱 실패");
        assert_eq!(result["count"].as_u64(), Some(expected as u64));
        assert_eq!(
            doc.search_all_text_native("lim", true, true)
                .expect("치환 후 원문 검색 실패"),
            "[]"
        );
        let replaced: serde_json::Value = serde_json::from_str(
            &doc.search_all_text_native("LIMIT", true, true)
                .expect("치환 후 새 텍스트 검색 실패"),
        )
        .expect("치환 후 검색 JSON 파싱 실패");
        assert_eq!(
            replaced.as_array().expect("치환 후 검색 결과 배열").len(),
            expected
        );
    }
}

//! 문서 텍스트 검색/치환 기능
//!
//! 본문, 표 셀, 글상자 등 중첩 컨트롤 내부 텍스트를 포함한 전체 검색.

use crate::document_core::helpers::get_textbox_from_shape;
use crate::document_core::DocumentCore;
use crate::error::HwpError;
use crate::model::control::Control;

/// 병적으로 깊은 중첩(손상/악의적 문서)에서 순회가 스택을 태우지 않게 하는 상한.
/// `grep` / `table_extract::MAX_NEST_DEPTH` / `explain` / `hidden_text` / `chart_extract` 와
/// 같은 값 — 깊이 0..=7 만 방문한다. 검색과 grep 이 서로 다른 깊이를 보면 "grep 은 찾는데
/// 바꾸지는 못하는" 어긋남이 생기므로 반드시 같이 간다.
const MAX_NEST_DEPTH: usize = 8;

/// 표 셀·글상자 안의 매치 좌표.
///
/// `parent_para` 는 바깥 표/글상자가 놓인 **본문 문단**이고, `path` 는 거기서부터 깊이마다
/// `(control_index, cell_index, cell_para_index)` 를 하나씩 쌓은 경로다. 글상자는
/// `cell_index = 0` sentinel 을 쓴다 — `resolve_cell_paragraph_mut` /
/// `reflow_cell_paragraph_by_path` 등 기존 path 코어와 **같은 표현**이라 그대로 넘길 수 있다.
///
/// [#2792] 종전에는 평면 4-튜플이라 깊이 1(바깥 셀)까지밖에 못 가리켰고, 그래서 셀 안의
/// 표는 검색에서 조용히 누락됐다(`replaceAll` 이 `{"ok":true,"count":0}` 을 성공으로 반환).
#[derive(Debug, Clone)]
struct CellHit {
    parent_para: usize,
    path: Vec<(usize, usize, usize)>,
}

impl CellHit {
    /// 중첩 깊이. 바깥 셀이 1.
    fn depth(&self) -> usize {
        self.path.len()
    }

    /// 깊이 1이면 종전 평면 좌표 `(parent_para, ctrl_idx, cell_idx, cell_para)`.
    ///
    /// 종전 결과 JSON(`cellContext`)을 **바이트까지 그대로** 유지하기 위한 것이다. 깊이 2
    /// 이상은 이 표현으로 담을 수 없으므로 `None` 이고, 결과에는 `cellPath` 로 실린다.
    fn flat(&self) -> Option<(usize, usize, usize, usize)> {
        match self.path.as_slice() {
            [(ctrl_idx, cell_idx, cell_para_idx)] => {
                Some((self.parent_para, *ctrl_idx, *cell_idx, *cell_para_idx))
            }
            _ => None,
        }
    }
}

/// 검색 결과 위치 정보
#[derive(Debug, Clone)]
struct SearchHit {
    sec: usize,
    para: usize,
    char_offset: usize,
    length: usize,
    /// 표 셀·글상자 안의 매치면 그 경로. 본문 매치면 `None`.
    cell_context: Option<CellHit>,
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

/// 한 컨테이너 문단(본문/셀/글상자)에서 그 문단 자신의 매치를 모은다.
///
/// 문단 텍스트와 그 문단 안 수식 script 를 같은 규칙으로 훑는다. 컨트롤을 타고 더
/// 내려가는 일은 하지 않는다 — 하강은 `search_nested_controls` 가 맡는다.
#[allow(clippy::too_many_arguments)]
fn push_container_hits(
    container: &crate::model::paragraph::Paragraph,
    sec_idx: usize,
    para_idx: usize,
    cell: Option<&CellHit>,
    is_text_box: bool,
    query: &str,
    case_sensitive: bool,
    results: &mut Vec<SearchHit>,
) {
    let qlen = query.chars().count();
    for offset in find_in_text(&container.text, query, case_sensitive) {
        results.push(SearchHit {
            sec: sec_idx,
            para: para_idx,
            char_offset: offset,
            length: qlen,
            cell_context: cell.cloned(),
            is_text_box,
            equation_control: None,
        });
    }
    // 수식 스크립트 — 렌더 트리(EquationNode)가 아니라 IR을 직접 순회하므로
    // #3419(export-text/markdown 쪽 수식 텍스트화)와는 별개 경로. 셀/글상자와
    // 별도 equation_control 로 표시해 커서 이동 대상에서는 제외한다.
    for (equation_index, control) in container.controls.iter().enumerate() {
        if let Control::Equation(equation) = control {
            for offset in find_in_text(&equation.script, query, case_sensitive) {
                results.push(SearchHit {
                    sec: sec_idx,
                    para: para_idx,
                    char_offset: offset,
                    length: qlen,
                    cell_context: cell.cloned(),
                    is_text_box,
                    equation_control: Some(equation_index),
                });
            }
        }
    }
}

/// `container` 의 컨트롤을 타고 표 셀·글상자 안으로 내려가며 매치를 모은다.
///
/// [#2792] 셀 문단의 컨트롤에 또 표가 있으면 그 안까지 재귀한다. 종전에는 이 하강이
/// 없어서 — 셀 문단 컨트롤 순회가 수식만 봤다 — 중첩 표 텍스트가 검색·치환 양쪽에서
/// 조용히 버려졌다. `prefix` 는 지금까지 내려온 경로이며, 재귀 전후로 push/pop 한다.
///
/// 매치는 **문서 순서**로 쌓인다: 셀 문단 자신의 매치를 먼저 넣고 그 문단이 품은 더
/// 깊은 표로 내려간다. `replace_matches_native` 는 이 순서를 뒤집어 뒤에서부터 치환하므로,
/// 같은 문단 안 여러 매치의 오프셋이 서로를 밀지 않는다.
fn search_nested_controls(
    container: &crate::model::paragraph::Paragraph,
    sec_idx: usize,
    parent_para: usize,
    prefix: &mut Vec<(usize, usize, usize)>,
    query: &str,
    case_sensitive: bool,
    results: &mut Vec<SearchHit>,
) {
    if prefix.len() >= MAX_NEST_DEPTH {
        return;
    }

    for (ctrl_idx, ctrl) in container.controls.iter().enumerate() {
        match ctrl {
            Control::Table(table) => {
                for (cell_idx, cell) in table.cells.iter().enumerate() {
                    for (cell_para_idx, cell_para) in cell.paragraphs.iter().enumerate() {
                        prefix.push((ctrl_idx, cell_idx, cell_para_idx));
                        let hit = CellHit {
                            parent_para,
                            path: prefix.clone(),
                        };
                        push_container_hits(
                            cell_para,
                            sec_idx,
                            parent_para,
                            Some(&hit),
                            false,
                            query,
                            case_sensitive,
                            results,
                        );
                        search_nested_controls(
                            cell_para,
                            sec_idx,
                            parent_para,
                            prefix,
                            query,
                            case_sensitive,
                            results,
                        );
                        prefix.pop();
                    }
                }
            }
            Control::Shape(shape) => {
                if let Some(tb) = get_textbox_from_shape(shape) {
                    for (tb_para_idx, tb_para) in tb.paragraphs.iter().enumerate() {
                        // 글상자는 cell_index = 0 sentinel — path 코어와 같은 규약.
                        prefix.push((ctrl_idx, 0, tb_para_idx));
                        let hit = CellHit {
                            parent_para,
                            path: prefix.clone(),
                        };
                        push_container_hits(
                            tb_para,
                            sec_idx,
                            parent_para,
                            Some(&hit),
                            true,
                            query,
                            case_sensitive,
                            results,
                        );
                        search_nested_controls(
                            tb_para,
                            sec_idx,
                            parent_para,
                            prefix,
                            query,
                            case_sensitive,
                            results,
                        );
                        prefix.pop();
                    }
                }
            }
            _ => {}
        }
    }
}

/// 문서 전체를 순회하며 query와 일치하는 모든 위치를 반환
fn search_all(doc: &DocumentCore, query: &str, case_sensitive: bool) -> Vec<SearchHit> {
    let mut results = vec![];

    for (sec_idx, section) in doc.document.sections.iter().enumerate() {
        for (para_idx, para) in section.paragraphs.iter().enumerate() {
            // 본문 문단 (자신의 텍스트 + 수식)
            push_container_hits(
                para,
                sec_idx,
                para_idx,
                None,
                false,
                query,
                case_sensitive,
                &mut results,
            );

            // 표 셀·글상자 — 중첩 표까지 내려간다
            let mut prefix = Vec::new();
            search_nested_controls(
                para,
                sec_idx,
                para_idx,
                &mut prefix,
                query,
                case_sensitive,
                &mut results,
            );
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
            //
            // [#2792] 중첩 셀(깊이 ≥2)도 여기서는 제외한다. 그 히트는 결과 JSON 에
            // `cellContext` 대신 `cellPath` 로 실리는데, 호출자(studio find-dialog)는
            // `cellContext` 가 없으면 **본문 좌표 분기**로 떨어져 표가 놓인 바깥 문단을
            // 고친다 — #3865 가 경고한 바로 그 손상이다. 이동·단건 치환이 path 를 받게
            // 되면(이슈의 경로 기반 선택 축) 이 조건만 풀면 된다. 전체 치환
            // (`replace_all_native`)은 이 필터를 타지 않으므로 이미 중첩까지 고친다.
            all_hits
                .iter()
                .filter(|h| {
                    !h.is_text_box
                        && h.equation_control.is_none()
                        && h.cell_context.as_ref().is_none_or(|cell| cell.depth() == 1)
                })
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
                Some(cell) => format_cell_context(cell),
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
        let mut affected_body_paragraphs: Vec<(usize, usize)> = Vec::new();
        // (구역, 부모 문단, 경로) — 경로의 마지막 엔트리가 곧 대상 셀 문단이다.
        let mut affected_cell_paragraphs: Vec<(usize, usize, Vec<(usize, usize, usize)>)> =
            Vec::new();
        let mut body_flow_boundaries = std::collections::BTreeMap::new();

        for hit in &all_hits {
            if let Some(cell) = hit.cell_context.as_ref() {
                // 표 셀 내부 치환
                let section = self
                    .document
                    .sections
                    .get_mut(hit.sec)
                    .ok_or_else(|| HwpError::RenderError("구역 범위 초과".into()))?;

                // [#2792] 평면 좌표를 손으로 풀던 자리다. 경로 해석은 이미 있는 path 코어에
                // 맡긴다 — 표·글상자(cell_index = 0 sentinel)를 깊이 제한 없이 같은 규칙으로
                // 내려가므로, 셀 안의 표도 바깥 셀과 똑같이 닿는다.
                let nested_para =
                    Self::resolve_cell_paragraph_mut(section, cell.parent_para, &cell.path)?;
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
                    affected_cell_paragraphs.push((hit.sec, cell.parent_para, cell.path.clone()));
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
                body_flow_boundaries
                    .entry((hit.sec, hit.para))
                    .or_insert_with(|| {
                        crate::renderer::composer::paragraph_flow_end(
                            &self.document.sections[hit.sec].paragraphs[hit.para],
                        )
                    });
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
                affected_body_paragraphs.push((hit.sec, hit.para));
                count += 1;
                affected_sections.push(hit.sec);
            }
        }

        // 변경된 섹션들 recompose
        if count > 0 {
            affected_body_paragraphs.sort_unstable();
            affected_body_paragraphs.dedup();
            for (section_idx, para_idx) in affected_body_paragraphs {
                self.reflow_paragraph(section_idx, para_idx);
            }
            let mut body_flow_starts = std::collections::BTreeMap::new();
            for ((section_idx, para_idx), stored_end) in body_flow_boundaries {
                body_flow_starts
                    .entry(section_idx)
                    .and_modify(|current: &mut (usize, Option<i32>)| {
                        if para_idx < current.0 {
                            *current = (para_idx, stored_end);
                        }
                    })
                    .or_insert((para_idx, stored_end));
            }
            for (section_idx, (start_para, stored_end)) in body_flow_starts {
                let hwp3_layout = self.document.layout_profile().hwp3_layout();
                crate::renderer::composer::recalculate_section_vpos(
                    &mut self.document.sections[section_idx].paragraphs,
                    start_para,
                    None,
                    stored_end,
                    &self.styles,
                    self.dpi,
                    hwp3_layout,
                );
            }
            affected_cell_paragraphs.sort_unstable();
            affected_cell_paragraphs.dedup();
            // [#2792] 평면 좌표 reflow(`reflow_cell_paragraph`)는 깊이 1까지만 닿는다.
            // #2755 가 깊이 ≥2 용으로 만들어 둔 path 판을 그대로 쓴다.
            //
            // 두 by_path 함수는 **마지막 엔트리의 문단 인덱스를 보지 않고** 그 셀의 문단
            // 목록만 잡는다(대상 문단은 뒤 인자로 따로 받는다). 그래서 흐름 재계산을 묶는
            // 키에서는 마지막 문단 인덱스를 0 으로 정규화해 "같은 셀"을 한 덩어리로 만든다.
            let mut cell_flow_starts = std::collections::BTreeMap::new();
            for (section_idx, parent_para, path) in &affected_cell_paragraphs {
                let Some(inner_para) = path.last().map(|entry| entry.2) else {
                    continue;
                };
                self.reflow_cell_paragraph_by_path(*section_idx, *parent_para, path, inner_para);

                let mut container_key = path.clone();
                if let Some(last) = container_key.last_mut() {
                    last.2 = 0;
                }
                cell_flow_starts
                    .entry((*section_idx, *parent_para, container_key))
                    .and_modify(|start: &mut usize| *start = (*start).min(inner_para))
                    .or_insert(inner_para);
            }
            for ((section_idx, parent_para, path), start_para) in cell_flow_starts {
                self.recalculate_cell_paragraph_vpos_by_path(
                    section_idx,
                    parent_para,
                    &path,
                    start_para,
                    None,
                );
            }
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

/// 셀 히트의 JSON 조각.
///
/// 깊이 1은 종전 `cellContext` 를 **바이트까지 그대로** 낸다 — 기존 소비자(studio
/// find-dialog 의 이동·단건 치환)는 무회귀이고, 중첩이 없는 문서의 결과 JSON 은 종전과 같다.
/// 깊이 2 이상은 그 표현에 담을 수 없으므로 `cellPath` 배열로 싣는다. 배열 모양은
/// `DocumentCore::parse_cell_path` 의 입력과 같아서, 소비자가 받은 값을 그대로 by_path
/// API 에 되먹일 수 있다(부모 문단은 결과의 `para` 가 곧 `parentPara` 다).
fn format_cell_context(cell: &CellHit) -> String {
    if let Some((parent_para, ctrl_idx, cell_idx, cell_para_idx)) = cell.flat() {
        return format!(
            ",\"cellContext\":{{\"parentPara\":{},\"ctrlIdx\":{},\"cellIdx\":{},\"cellPara\":{}}}",
            parent_para, ctrl_idx, cell_idx, cell_para_idx
        );
    }
    let entries: Vec<String> = cell
        .path
        .iter()
        .map(|(ctrl_idx, cell_idx, cell_para_idx)| {
            format!(
                "{{\"controlIndex\":{},\"cellIndex\":{},\"cellParaIndex\":{}}}",
                ctrl_idx, cell_idx, cell_para_idx
            )
        })
        .collect();
    format!(",\"cellPath\":[{}]", entries.join(","))
}

fn format_search_hit(hit: &SearchHit, wrapped: bool) -> String {
    let cell_ctx = match &hit.cell_context {
        Some(cell) => format_cell_context(cell),
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

    fn stored_search_paragraph(text: &str, width_hwp: i32) -> crate::model::paragraph::Paragraph {
        crate::model::paragraph::Paragraph {
            text: text.to_string(),
            char_offsets: (0..text.chars().count() as u32).collect(),
            char_count: text.chars().count() as u32 + 1,
            char_shapes: vec![crate::model::paragraph::CharShapeRef {
                start_pos: 0,
                char_shape_id: 0,
            }],
            line_segs: vec![crate::model::paragraph::LineSeg {
                text_start: 0,
                line_height: 1_000,
                text_height: 900,
                baseline_distance: 800,
                segment_width: width_hwp,
                tag: crate::model::paragraph::LineSeg::TAG_SINGLE_SEGMENT_LINE,
                ..Default::default()
            }],
            ..Default::default()
        }
    }

    fn core_with_search_section(paragraph: crate::model::paragraph::Paragraph) -> DocumentCore {
        let mut core = DocumentCore::new_empty();
        core.document.sections = vec![crate::model::document::Section {
            section_def: crate::model::document::SectionDef {
                page_def: crate::model::page::PageDef {
                    width: 10_000,
                    height: 84_188,
                    margin_left: 0,
                    margin_right: 0,
                    margin_top: 0,
                    margin_bottom: 0,
                    margin_header: 0,
                    margin_footer: 0,
                    margin_gutter: 0,
                    ..Default::default()
                },
                ..Default::default()
            },
            paragraphs: vec![paragraph],
            ..Default::default()
        }];
        core.composed = vec![Vec::new()];
        core.dirty_sections = vec![true];
        core.dirty_paragraphs = vec![None];
        core
    }

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
        bulk_replace_materializes_a_current_body_partition();
        bulk_replace_materializes_a_current_nested_cell_partition();
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

    fn bulk_replace_materializes_a_current_body_partition() {
        let replacement = "A".repeat(30);
        let mut core = core_with_search_section(stored_search_paragraph("old", 10_000));
        let mut following = stored_search_paragraph("tail", 10_000);
        following.line_segs[0].vertical_pos = 1_000;
        core.document.sections[0].paragraphs.push(following);

        core.replace_all_native("old", &replacement, true)
            .expect("body replacement succeeds");
        let paragraph = &core.document.sections[0].paragraphs[0];
        assert!(paragraph.line_segs.len() > 1);
        assert!(!paragraph.stored_text_partition_is_dirty());
        let expected_following_vpos = crate::renderer::composer::paragraph_flow_end(paragraph)
            .expect("replacement paragraph has flow end");
        assert_eq!(
            core.document.sections[0].paragraphs[1].line_segs[0].vertical_pos,
            expected_following_vpos
        );

        let bytes = crate::serializer::body_text::serialize_section(&core.document.sections[0]);
        let parsed = crate::parser::body_text::parse_body_text_section(&bytes).unwrap();
        assert_eq!(
            parsed.paragraphs[0].line_segs.len(),
            paragraph.line_segs.len()
        );
        assert_eq!(
            parsed.paragraphs[1].line_segs[0].vertical_pos,
            expected_following_vpos
        );
    }

    fn bulk_replace_materializes_a_current_nested_cell_partition() {
        let replacement = "A".repeat(30);
        let cell_para = stored_search_paragraph("old", 10_000);
        let mut following = stored_search_paragraph("tail", 10_000);
        following.line_segs[0].vertical_pos = 1_000;
        let table = crate::model::table::Table {
            row_count: 1,
            col_count: 1,
            cells: vec![crate::model::table::Cell {
                row: 0,
                col: 0,
                row_span: 1,
                col_span: 1,
                width: 10_000,
                paragraphs: vec![cell_para, following],
                ..Default::default()
            }],
            ..Default::default()
        };
        let mut parent = crate::model::paragraph::Paragraph::default();
        parent.controls.push(Control::Table(Box::new(table)));
        let mut core = core_with_search_section(parent);

        core.replace_all_native("old", &replacement, true)
            .expect("nested replacement succeeds");
        let Control::Table(table) = &core.document.sections[0].paragraphs[0].controls[0] else {
            panic!("expected table");
        };
        let paragraph = &table.cells[0].paragraphs[0];
        assert!(paragraph.line_segs.len() > 1);
        assert!(!paragraph.stored_text_partition_is_dirty());
        let expected_following_vpos = crate::renderer::composer::paragraph_flow_end(paragraph)
            .expect("replacement cell paragraph has flow end");
        assert_eq!(
            table.cells[0].paragraphs[1].line_segs[0].vertical_pos,
            expected_following_vpos
        );

        assert!(
            !crate::serializer::body_text::serialize_section(&core.document.sections[0]).is_empty()
        );
    }
}

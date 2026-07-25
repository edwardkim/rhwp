//! 주소를 가진 문서 검색 — 매치마다 (구역·문단·**페이지**·문자 오프셋)을 돌려준다.
//!
//! 평문 추출 후 외부에서 검색하면 주소가 소멸해 근거 제시가 불가능하다. rhwp 는 조판
//! 엔진을 갖고 있어 "몇 쪽"에 답할 수 있는데, 그 답을 낼 출구가 없었다.
//!
//! 페이지 매핑 비용은 0이다 — `DocumentCore::from_bytes` 가 로드 시 `paginate()` 를
//! 끝내므로 순수 조회다. 다만 매치마다 조회하면 O(N × 페이지 아이템)이므로
//! `(구역,문단) → 페이지` 인덱스를 **한 번만** 만들어 재사용한다.
//!
//! 파서/렌더 무변경의 읽기 전용 질의(추가 기능).

use std::collections::HashMap;

use serde::Serialize;

use crate::document_core::DocumentCore;
use crate::model::control::Control;
use crate::renderer::pagination::PageItem;

/// 검색 매치 하나.
#[derive(Debug, Clone, Serialize)]
pub struct GrepMatch {
    /// 구역 인덱스.
    pub section: usize,
    /// 본문 문단 인덱스 (표 셀·글상자 매치는 그 컨트롤을 담은 본문 문단).
    pub paragraph: usize,
    /// 0부터 시작하는 글로벌 페이지 번호. 조판에 배치되지 않은 문단이면 생략된다.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<u32>,
    /// 문단 텍스트 내 매치 시작 위치 (문자 단위).
    #[serde(rename = "charOffset")]
    pub char_offset: usize,
    /// 매치 길이 (문자 단위).
    pub length: usize,
    /// 매치가 속한 문단의 전체 텍스트.
    pub text: String,
    /// 매치 주변 발췌 (앞뒤 문맥).
    pub context: String,
    /// 표 셀 안의 매치면 셀 좌표. 본문 매치면 생략된다.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cell: Option<CellRef>,
    /// 글상자 안의 매치면 글상자 좌표. 본문·표 셀 매치면 생략된다.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub textbox: Option<TextBoxRef>,
}

/// 표 셀 매치의 좌표.
#[derive(Debug, Clone, Serialize)]
pub struct CellRef {
    /// 표를 담은 본문 문단의 컨트롤 인덱스.
    pub control: usize,
    /// 셀 인덱스.
    pub cell: usize,
    /// 셀 안의 문단 인덱스.
    pub paragraph: usize,
}

/// 글상자 매치의 좌표.
#[derive(Debug, Clone, Serialize)]
pub struct TextBoxRef {
    /// 글상자를 담은 본문 문단의 컨트롤 인덱스.
    pub control: usize,
    /// 글상자 안의 문단 인덱스.
    pub paragraph: usize,
}

/// 매치 주변 발췌를 만든다 (앞뒤 `WINDOW` 문자).
fn make_context(text: &str, offset: usize, length: usize) -> String {
    const WINDOW: usize = 40;
    let chars: Vec<char> = text.chars().collect();
    let start = offset.saturating_sub(WINDOW);
    let end = (offset + length + WINDOW).min(chars.len());
    let mut out = String::new();
    if start > 0 {
        out.push('…');
    }
    out.extend(chars[start..end].iter());
    if end < chars.len() {
        out.push('…');
    }
    out
}

impl DocumentCore {
    /// `(구역, 문단) → 글로벌 페이지` 인덱스를 한 번에 만든다.
    ///
    /// 한 문단이 여러 쪽에 걸치면 **처음 등장한 쪽**을 쓴다 — 인용은 시작 위치가 기준이다.
    fn build_paragraph_page_index(&self) -> HashMap<(usize, usize), u32> {
        let mut index: HashMap<(usize, usize), u32> = HashMap::new();
        let mut global_offset = 0u32;
        for (sec_idx, pr) in self.pagination.iter().enumerate() {
            for (local_i, page) in pr.pages.iter().enumerate() {
                let global_page = global_offset + local_i as u32;
                for col in &page.column_contents {
                    for item in &col.items {
                        let para_index = match item {
                            PageItem::FullParagraph { para_index }
                            | PageItem::PartialParagraph { para_index, .. }
                            | PageItem::Table { para_index, .. }
                            | PageItem::PartialTable { para_index, .. }
                            | PageItem::Shape { para_index, .. } => Some(*para_index),
                            _ => None,
                        };
                        if let Some(p) = para_index {
                            index.entry((sec_idx, p)).or_insert(global_page);
                        }
                    }
                }
            }
            global_offset += pr.pages.len() as u32;
        }
        index
    }

    /// 문서를 검색해 주소가 붙은 매치 목록을 돌려준다.
    ///
    /// 본문·표 셀·글상자를 순회한다(`search_all` 과 같은 범위). `limit` 이 `Some` 이면
    /// 그 개수에서 멈춘다 — 대형 문서에서 컨텍스트를 아끼기 위한 상한이다.
    pub fn grep(&self, query: &str, case_sensitive: bool, limit: Option<usize>) -> Vec<GrepMatch> {
        if query.is_empty() {
            return Vec::new();
        }
        let page_index = self.build_paragraph_page_index();
        let qlen = query.chars().count();
        let mut out: Vec<GrepMatch> = Vec::new();

        for (sec_idx, section) in self.document.sections.iter().enumerate() {
            for (para_idx, para) in section.paragraphs.iter().enumerate() {
                let page = page_index.get(&(sec_idx, para_idx)).copied();

                let make = |text: &str,
                            offset: usize,
                            cell: Option<CellRef>,
                            textbox: Option<TextBoxRef>| GrepMatch {
                    section: sec_idx,
                    paragraph: para_idx,
                    page,
                    char_offset: offset,
                    length: qlen,
                    text: text.to_string(),
                    context: make_context(text, offset, qlen),
                    cell,
                    textbox,
                };

                for offset in super::search_query::find_matches(&para.text, query, case_sensitive) {
                    out.push(make(&para.text, offset, None, None));
                    if limit.is_some_and(|n| out.len() >= n) {
                        return out;
                    }
                }

                for (ctrl_idx, ctrl) in para.controls.iter().enumerate() {
                    match ctrl {
                        Control::Table(table) => {
                            for (cell_idx, cell) in table.cells.iter().enumerate() {
                                for (cp_idx, cp) in cell.paragraphs.iter().enumerate() {
                                    for offset in super::search_query::find_matches(
                                        &cp.text,
                                        query,
                                        case_sensitive,
                                    ) {
                                        out.push(make(
                                            &cp.text,
                                            offset,
                                            Some(CellRef {
                                                control: ctrl_idx,
                                                cell: cell_idx,
                                                paragraph: cp_idx,
                                            }),
                                            None,
                                        ));
                                        if limit.is_some_and(|n| out.len() >= n) {
                                            return out;
                                        }
                                    }
                                }
                            }
                        }
                        Control::Shape(shape) => {
                            if let Some(tb) =
                                crate::document_core::helpers::get_textbox_from_shape(shape)
                            {
                                for (tp_idx, tp) in tb.paragraphs.iter().enumerate() {
                                    for offset in super::search_query::find_matches(
                                        &tp.text,
                                        query,
                                        case_sensitive,
                                    ) {
                                        out.push(make(
                                            &tp.text,
                                            offset,
                                            None,
                                            Some(TextBoxRef {
                                                control: ctrl_idx,
                                                paragraph: tp_idx,
                                            }),
                                        ));
                                        if limit.is_some_and(|n| out.len() >= n) {
                                            return out;
                                        }
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        out
    }
}

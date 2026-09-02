//! 읽기 전용 질의 — 검색과 텍스트 내보내기.
//!
//! 두 질의 모두 **`Result` 를 돌려주지 않는다**. "매치가 없다", "요청한 쪽이
//! 범위 밖이다", "그 쪽의 텍스트를 못 뽑았다"는 전부 판정이지 실패가 아니고,
//! 판정을 오류로 올리면 소비자는 `Err` 안에서 다시 문자열을 갈라 읽어야 한다.
//! 그래서 여기서는 **판정을 데이터로** 싣는다 — 그 데이터로 exit code 를 정할지
//! 경고만 낼지는 표면의 정책이다.
//!
//! 절단 어휘(`truncated`·`omittedCount`·`nextOffset`)는 이 저장소가 이미 CLI 와
//! MCP 양쪽에서 쓰는 것을 그대로 따른다. 새 어휘를 만들면 세 번째 방언이 된다.

use serde::Serialize;

use crate::document_core::queries::grep::GrepMatch;
use crate::service::open::OpenedDocument;

/// 검색 옵션.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchOptions {
    /// 대소문자 구분. 기본 `true`(현행 `search`·`hwp_doc_search` 기본값).
    pub case_sensitive: bool,
    /// 돌려줄 매치 수 상한. `None` 이면 창을 자르지 않는다.
    ///
    /// 총량(`total_match_count`)은 **언제나 전수**로 센다 — 상한은 표시만 줄인다.
    pub limit: Option<usize>,
    /// 몇 번째 매치부터 돌려줄지(0 기준).
    pub offset: usize,
    /// 매치가 속한 문단의 앞뒤 몇 문단을 함께 담을지. `None` 이면 담지 않는다.
    pub context_paragraphs: Option<usize>,
}

impl Default for SearchOptions {
    fn default() -> Self {
        SearchOptions {
            case_sensitive: true,
            limit: None,
            offset: 0,
            context_paragraphs: None,
        }
    }
}

impl SearchOptions {
    /// 기본 옵션(대소문자 구분, 무제한, 처음부터).
    pub fn new() -> Self {
        SearchOptions::default()
    }

    /// 대소문자 구분 여부를 정한다.
    pub fn case_sensitive(mut self, case_sensitive: bool) -> Self {
        self.case_sensitive = case_sensitive;
        self
    }

    /// 표시 상한을 정한다.
    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    /// 시작 오프셋을 정한다.
    pub fn with_offset(mut self, offset: usize) -> Self {
        self.offset = offset;
        self
    }

    /// 앞뒤 문맥 문단 수를 정한다.
    pub fn with_context_paragraphs(mut self, context: usize) -> Self {
        self.context_paragraphs = Some(context);
        self
    }
}

/// 검색 결과. **매치 0건도 정상 결과다.**
///
/// 필드 이름은 현행 `search --json` 봉투와 같다(`schemaVersion`·`source`·`query`
/// 위쪽 껍데기는 표면이 붙인다). 파생값(`match_count`·`truncated`·`omitted_count`)을
/// 저장 필드로 둔 이유는 생성자에서 한 번에 계산해 두면 소비자가 산술로 유도하다
/// 어긋날 일이 없기 때문이다 — 이 저장소가 `omittedCount` 를 굳이 명시하는 것과
/// 같은 이유다.
///
/// `PartialEq` 는 파생하지 않는다 — `GrepMatch`(`document_core::queries::grep`)가
/// `PartialEq` 가 아니고, 그 타입은 이 PR 의 수정 범위 밖이다. 두 결과가 같은지
/// 봐야 하면 직렬화 값을 비교하라(그쪽이 실제 계약이기도 하다).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchOutcome {
    /// 검색어(에코).
    pub query: String,
    /// 적용된 대소문자 구분.
    pub case_sensitive: bool,
    /// 이 응답에 실린 매치 수.
    pub match_count: usize,
    /// 문서 전체의 매치 수(오프셋·상한과 무관한 전수).
    pub total_match_count: usize,
    /// 이 응답이 전체가 아닌가.
    pub truncated: bool,
    /// 이 응답에서 빠진 매치 수.
    pub omitted_count: usize,
    /// 적용된 시작 오프셋.
    pub offset: usize,
    /// 다음 창의 시작 오프셋. 더 없으면 생략된다.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_offset: Option<usize>,
    /// 좌표가 붙은 매치들(구역·문단·쪽·문자 오프셋, 표 셀·글상자·수식 좌표 포함).
    pub matches: Vec<GrepMatch>,
}

impl SearchOutcome {
    /// 매치가 하나도 없는가. **오류가 아니라 질문에 대한 답이다.**
    pub fn is_empty(&self) -> bool {
        self.total_match_count == 0
    }
}

/// 텍스트 내보내기 옵션.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TextExportOptions {
    /// 뽑을 쪽 번호(0 기준). `None` 이면 전체.
    pub pages: Option<Vec<u32>>,
    /// 돌려줄 문자 수 상한. `None` 이면 무제한.
    pub max_chars: Option<usize>,
    /// 이어붙인 텍스트에서 건너뛸 문자 수(0 기준). 이어읽기용.
    pub char_offset: usize,
}

impl TextExportOptions {
    /// 전체 쪽·무제한.
    pub fn new() -> Self {
        TextExportOptions::default()
    }

    /// 뽑을 쪽을 지정한다.
    pub fn with_pages(mut self, pages: Vec<u32>) -> Self {
        self.pages = Some(pages);
        self
    }

    /// 문자 수 상한을 지정한다.
    pub fn with_max_chars(mut self, max_chars: usize) -> Self {
        self.max_chars = Some(max_chars);
        self
    }

    /// 시작 문자 오프셋을 지정한다.
    pub fn with_char_offset(mut self, char_offset: usize) -> Self {
        self.char_offset = char_offset;
        self
    }
}

/// 쪽 하나의 텍스트.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PageText {
    /// 0 기준 쪽 번호.
    pub page: u32,
    /// 쪽 텍스트. 잘렸거나 추출에 실패했으면 그만큼 짧거나 비어 있다.
    pub text: String,
    /// 이 쪽이 잘렸는가. 안 잘렸으면 직렬화에서 빠진다.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub truncated: bool,
    /// 이 쪽에서 생략된 문자 수. 0이면 직렬화에서 빠진다.
    #[serde(skip_serializing_if = "is_zero")]
    pub omitted_count: usize,
    /// 이 쪽의 텍스트 추출이 실패했으면 그 사유.
    ///
    /// 실패한 쪽도 **목록에서 빼지 않는다**. 빼면 `pageCount` 가 줄어 문서가 실제보다
    /// 짧아 보이고, "빈 쪽"과 "못 읽은 쪽"이 같은 모습이 된다.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extract_error: Option<String>,
}

#[allow(clippy::trivially_copy_pass_by_ref)]
fn is_zero(value: &usize) -> bool {
    *value == 0
}

/// 텍스트 내보내기 결과.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TextExport {
    /// 실린 쪽 수(= `pages.len()`).
    pub page_count: usize,
    /// 어디선가 잘렸는가.
    pub truncated: bool,
    /// 전체에서 생략된 문자 수.
    pub omitted_count: usize,
    /// 적용된 시작 문자 오프셋.
    pub char_offset: usize,
    /// 다음 이어읽기 시작 오프셋. 더 없으면 생략된다.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_offset: Option<usize>,
    /// 요청했으나 문서 범위 밖이라 건너뛴 쪽 번호.
    ///
    /// 빈 결과를 조용히 돌려주면 오타(`--page 999`)가 성공처럼 보인다. 그렇다고
    /// 오류로 올리면 "일부만 유효한 요청"을 표현할 수 없다. 그래서 **데이터로**
    /// 싣는다 — 이걸 보고 끊을지 경고만 할지는 표면의 정책이다.
    pub out_of_range: Vec<u32>,
    /// 쪽별 텍스트.
    pub pages: Vec<PageText>,
}

impl TextExport {
    /// 쪽 텍스트를 줄바꿈으로 이어 붙인 하나의 문자열.
    ///
    /// 파일로 떨구거나 해시를 뜨는 소비자용 편의 함수다.
    pub fn concatenated(&self) -> String {
        let mut out = String::new();
        for page in &self.pages {
            if !out.is_empty() {
                out.push('\n');
            }
            out.push_str(&page.text);
        }
        out
    }

    /// 추출에 실패한 쪽이 하나라도 있는가.
    pub fn has_failures(&self) -> bool {
        self.pages.iter().any(|p| p.extract_error.is_some())
    }
}

impl OpenedDocument {
    /// 문서를 검색한다. 매치마다 **좌표**(구역·문단·쪽·문자 오프셋)가 붙는다.
    ///
    /// 총량은 언제나 전수로 세고 창(offset·limit)만 옮긴다 — `total_match_count`
    /// 의 뜻이 페이지네이션에 따라 흔들리면 "몇 건 중 몇 건"이라는 계약이 무너진다.
    ///
    /// 빈 검색어는 매치 0건이다(오류가 아니다). 빈 검색어를 거절할지는 인자 검증의
    /// 문제이고, 그건 표면의 몫이다.
    ///
    /// 엔진은 [`crate::document_core::DocumentCore::grep_with_context`] 다 — CLI 와
    /// MCP 가 이미 쓰는 그 엔진이며, WASM 만 다른 엔진(`search_all_text_native`)을
    /// 쓰고 있었다. 이 축은 좌표가 붙는 쪽으로 통일한다.
    pub fn search(&self, query: &str, opts: &SearchOptions) -> SearchOutcome {
        let all =
            self.core
                .grep_with_context(query, opts.case_sensitive, None, opts.context_paragraphs);
        let total = all.len();
        let skipped = all.into_iter().skip(opts.offset);
        let matches: Vec<GrepMatch> = match opts.limit {
            Some(limit) => skipped.take(limit).collect(),
            None => skipped.collect(),
        };
        let consumed = opts.offset.saturating_add(matches.len());
        SearchOutcome {
            query: query.to_string(),
            case_sensitive: opts.case_sensitive,
            match_count: matches.len(),
            total_match_count: total,
            truncated: matches.len() < total,
            omitted_count: total.saturating_sub(matches.len()),
            offset: opts.offset,
            next_offset: if consumed < total {
                Some(consumed)
            } else {
                None
            },
            matches,
        }
    }

    /// 쪽 주소가 붙은 텍스트를 뽑는다.
    ///
    /// 순서는 **쪽 선택 → 추출 → 오프셋 창 → 문자 상한**이다. 상한은 추출 시간이
    /// 아니라 **소비자 컨텍스트**를 지키는 장치이므로 전수 추출 뒤 표시만 자른다
    /// (그래야 총량과 생략량을 정직하게 보고할 수 있다).
    ///
    /// 잘린 쪽도, 오프셋에 다 먹힌 쪽도, 추출에 실패한 쪽도 목록에서 빠지지 않는다.
    pub fn export_text(&self, opts: &TextExportOptions) -> TextExport {
        let page_count = self.page_count();
        let (selected, out_of_range): (Vec<u32>, Vec<u32>) = match &opts.pages {
            Some(requested) => requested.iter().copied().partition(|p| *p < page_count),
            None => ((0..page_count).collect(), Vec::new()),
        };

        // 1) 추출 — 실패한 쪽은 빈 텍스트 + 사유로 남긴다.
        let mut extracted: Vec<(u32, String, Option<String>)> = Vec::with_capacity(selected.len());
        for page in selected {
            match self.core.extract_page_text_native(page) {
                Ok(text) => extracted.push((page, text, None)),
                Err(error) => extracted.push((page, String::new(), Some(error.to_string()))),
            }
        }
        let total_chars: usize = extracted.iter().map(|(_, t, _)| t.chars().count()).sum();

        // 2) 오프셋 창 — 쪽 본문 텍스트를 논리적으로 이은 좌표에서 char_offset 만큼
        //    건너뛴다. TextExport::concatenated()가 표시 편의를 위해 삽입하는 줄바꿈은
        //    문서 본문 글자가 아니므로 이 좌표·총량·다음 오프셋에 세지 않는다.
        let mut skip = opts.char_offset;
        let windowed: Vec<(u32, String, Option<String>)> = extracted
            .into_iter()
            .map(|(page, text, failure)| {
                if skip == 0 {
                    return (page, text, failure);
                }
                let len = text.chars().count();
                if skip >= len {
                    skip -= len;
                    (page, String::new(), failure)
                } else {
                    let tail = text.chars().skip(skip).collect();
                    skip = 0;
                    (page, tail, failure)
                }
            })
            .collect();

        // 3) 문자 상한 — 예산이 떨어져도 쪽 항목은 남긴다.
        let mut budget = opts.max_chars;
        let mut omitted_total = 0usize;
        let mut shown_chars = 0usize;
        let mut pages = Vec::with_capacity(windowed.len());
        for (page, text, extract_error) in windowed {
            let total = text.chars().count();
            let keep = match budget {
                Some(remaining) => remaining.min(total),
                None => total,
            };
            if let Some(remaining) = budget.as_mut() {
                *remaining -= keep;
            }
            let omitted = total - keep;
            omitted_total += omitted;
            shown_chars += keep;
            let kept: String = if omitted == 0 {
                text
            } else {
                text.chars().take(keep).collect()
            };
            pages.push(PageText {
                page,
                text: kept,
                truncated: omitted > 0,
                omitted_count: omitted,
                extract_error,
            });
        }

        let consumed = opts.char_offset.saturating_add(shown_chars);
        TextExport {
            page_count: pages.len(),
            truncated: omitted_total > 0,
            omitted_count: omitted_total,
            char_offset: opts.char_offset,
            next_offset: if consumed < total_chars {
                Some(consumed)
            } else {
                None
            },
            out_of_range,
            pages,
        }
    }
}

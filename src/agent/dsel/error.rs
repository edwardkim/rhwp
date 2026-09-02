//! DSEL 진단 — 위치·기대·수복 힌트를 값으로 낸다.
//!
//! ## 왜 문자열이 아니라 구조체인가
//!
//! 선택자를 쓰는 쪽은 대부분 사람이 아니라 모델이다. `"parse error"` 한 줄은
//! 모델에게 "다시 추측하라"는 뜻이고, 추측 왕복 한 번이 실패 한 번이다. CLI 가
//! 이미 `수복: {"nextCall":…}` 규약으로 **다음 호출을 지목**하는 것과 같은 이유로,
//! 선택자 오류도 세 가지를 값으로 실어야 한다.
//!
//! - `offset` — 입력의 **문자(char)** 기준 위치. 바이트가 아니다. 한글 선택자
//!   (`para[style="제목 1"]`)에서 바이트 오프셋을 주면 캐럿이 글자 중간을 가리켜
//!   화면에서 어긋난다.
//! - `expected` — 그 자리에서 받을 수 있었던 것의 **닫힌 목록**. "무엇이 틀렸다"
//!   보다 "무엇이었어야 했다"가 재시도를 한 번에 끝낸다.
//! - `hint` — 흔한 오용의 교정 문장. 추측이 아니라 실제로 관측된 오용만 적는다.
//!
//! ## 캐럿을 여기서 그리는 이유
//!
//! [`SelectorError::render`] 가 캐럿 줄까지 만든다. 호출부(CLI·MCP·바인딩)마다
//! 캐럿을 다시 그리면 오프셋 해석이 갈라지고, 갈라진 순간 어느 쪽이 맞는지
//! 판정할 근거가 사라진다. 그리는 곳은 하나여야 한다.

use std::fmt;

/// 선택자 처리 중 발생한 오류.
///
/// 렉싱·파싱·평가가 같은 타입을 쓴다. 세 단계를 나눠 봐야 소비자는 결국
/// "선택자가 안 먹었다" 하나로 다루고, 나눈 만큼 `From` 변환만 늘어난다.
/// 단계 구분이 필요하면 [`SelectorErrorKind`] 로 본다.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectorError {
    /// 오류 갈래 — 기계 분기용.
    pub kind: SelectorErrorKind,
    /// 사람이 읽는 한 줄. 마침표로 끝내지 않는다(호출부가 문장을 이어 붙인다).
    pub message: String,
    /// 입력에서의 문자 오프셋. 입력 끝이면 `input.chars().count()`.
    pub offset: usize,
    /// 그 자리에서 허용됐던 것들. 비어 있을 수 있다(평가 단계 오류 등).
    pub expected: Vec<String>,
    /// 교정 힌트. 관측된 오용에 대해서만 채운다.
    pub hint: Option<String>,
}

/// 오류 갈래.
///
/// `exitCode` 매핑은 호출부의 몫이다 — 커널은 프로세스 종료 코드를 모른다.
/// 다만 갈래는 "사용법 오류(2)"와 "실행 실패(1)"를 가를 수 있을 만큼은 나눠 둔다.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectorErrorKind {
    /// 토큰으로 쪼갤 수 없는 문자열 — 닫히지 않은 따옴표, 알 수 없는 기호.
    Lex,
    /// 토큰은 맞지만 문법이 아님 — 예상 밖 토큰, 조기 종료.
    Parse,
    /// 문법은 맞지만 뜻이 없음 — 없는 축, 그 축에 없는 속성, 인자 개수 불일치.
    Resolve,
    /// 평가 중 한계 초과 — 중첩 깊이, 결과 폭발 방지 상한.
    Limit,
}

impl SelectorErrorKind {
    /// 갈래의 안정 문자열 이름 — 봉투 `error.kind` 로 그대로 나간다.
    pub const fn as_str(self) -> &'static str {
        match self {
            SelectorErrorKind::Lex => "lex",
            SelectorErrorKind::Parse => "parse",
            SelectorErrorKind::Resolve => "resolve",
            SelectorErrorKind::Limit => "limit",
        }
    }
}

impl SelectorError {
    /// 기본 생성자 — 기대 목록과 힌트는 빌더로 덧댄다.
    pub fn new(kind: SelectorErrorKind, offset: usize, message: impl Into<String>) -> Self {
        SelectorError {
            kind,
            message: message.into(),
            offset,
            expected: Vec::new(),
            hint: None,
        }
    }

    /// 렉싱 단계 오류.
    pub fn lex(offset: usize, message: impl Into<String>) -> Self {
        Self::new(SelectorErrorKind::Lex, offset, message)
    }

    /// 파싱 단계 오류.
    pub fn parse(offset: usize, message: impl Into<String>) -> Self {
        Self::new(SelectorErrorKind::Parse, offset, message)
    }

    /// 의미 해석 단계 오류.
    pub fn resolve(offset: usize, message: impl Into<String>) -> Self {
        Self::new(SelectorErrorKind::Resolve, offset, message)
    }

    /// 한계 초과.
    pub fn limit(offset: usize, message: impl Into<String>) -> Self {
        Self::new(SelectorErrorKind::Limit, offset, message)
    }

    /// 기대 목록을 단다. 여러 번 부르면 누적된다.
    ///
    /// 정렬·중복 제거를 여기서 하는 이유: 파서는 대안을 만나는 **순서대로** 기대를
    /// 쌓는데, 그 순서는 문법 규칙을 어떻게 적었느냐에 달린 구현 세부다. 소비자가
    /// 그 순서에 의존하면 문법을 리팩터링할 때마다 계약이 깨진다.
    pub fn expecting<I, S>(mut self, items: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.expected.extend(items.into_iter().map(Into::into));
        self.expected.sort();
        self.expected.dedup();
        self
    }

    /// 교정 힌트를 단다. 이미 있으면 덮어쓴다(더 구체적인 쪽이 나중에 온다).
    pub fn hinting(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }

    /// 캐럿 줄까지 포함한 사람용 렌더.
    ///
    /// 탭을 공백으로 바꾸지 않고 **그대로 흘린다** — 캐럿 줄에도 같은 탭을 넣으므로
    /// 터미널 탭 폭이 몇이든 캐럿은 맞는 칸에 선다. 공백으로 치환하면 탭 폭 8인
    /// 터미널에서 어긋난다.
    pub fn render(&self, input: &str) -> String {
        let mut out = String::new();
        out.push_str(&self.message);
        if !self.expected.is_empty() {
            out.push_str(" — 기대: ");
            out.push_str(&self.expected.join(" | "));
        }
        out.push('\n');
        out.push_str(input);
        out.push('\n');
        for (i, ch) in input.chars().enumerate() {
            if i >= self.offset {
                break;
            }
            out.push(if ch == '\t' { '\t' } else { ' ' });
        }
        out.push('^');
        if let Some(hint) = &self.hint {
            out.push('\n');
            out.push_str("힌트: ");
            out.push_str(hint);
        }
        out
    }

    /// 봉투에 실을 JSON 표현.
    ///
    /// `offset` 을 항상 싣는 이유: 값이 0 이어도 "맨 앞에서 틀렸다"는 정보다.
    /// 0 을 생략하면 소비자는 "위치 정보 없음"과 구별할 수 없다.
    pub fn to_json(&self) -> serde_json::Value {
        let mut obj = serde_json::Map::new();
        obj.insert("kind".into(), serde_json::json!(self.kind.as_str()));
        obj.insert("message".into(), serde_json::json!(self.message));
        obj.insert("offset".into(), serde_json::json!(self.offset));
        if !self.expected.is_empty() {
            obj.insert("expected".into(), serde_json::json!(self.expected));
        }
        if let Some(hint) = &self.hint {
            obj.insert("hint".into(), serde_json::json!(hint));
        }
        serde_json::Value::Object(obj)
    }
}

impl fmt::Display for SelectorError {
    /// 입력 없이 찍히는 자리(로그·`?` 전파)를 위한 한 줄.
    ///
    /// 캐럿은 입력이 있어야 그릴 수 있으므로 여기서는 오프셋을 숫자로 적는다.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "선택자 {}: {}", self.kind.as_str(), self.message)?;
        if !self.expected.is_empty() {
            write!(f, " (기대: {})", self.expected.join(" | "))?;
        }
        write!(f, " [문자 {}]", self.offset)
    }
}

impl std::error::Error for SelectorError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expected_is_sorted_and_deduped() {
        let err = SelectorError::parse(3, "예상 밖 토큰")
            .expecting(["]", "[", "]"])
            .expecting([":"]);
        assert_eq!(err.expected, vec![":".to_string(), "[".into(), "]".into()]);
    }

    #[test]
    fn caret_lands_on_char_boundary_not_byte() {
        // "한글" 뒤 세 번째 문자에서 오류 — 바이트로 세면 6, 문자로 세면 2.
        let err = SelectorError::parse(2, "여기");
        let rendered = err.render("한글x");
        let caret_line = rendered.lines().last().unwrap();
        // 캐럿 앞 공백이 두 칸이어야 세 번째 글자를 가리킨다.
        assert_eq!(caret_line, "  ^");
    }

    #[test]
    fn tabs_are_preserved_in_caret_line() {
        let err = SelectorError::parse(2, "여기");
        let rendered = err.render("\t\tx");
        assert!(rendered.lines().last().unwrap().starts_with("\t\t"));
    }

    #[test]
    fn json_keeps_zero_offset() {
        let json = SelectorError::lex(0, "닫히지 않은 따옴표").to_json();
        assert_eq!(json["offset"], serde_json::json!(0));
        assert_eq!(json["kind"], serde_json::json!("lex"));
        // 비어 있는 기대·힌트는 싣지 않는다.
        assert!(json.get("expected").is_none());
        assert!(json.get("hint").is_none());
    }

    #[test]
    fn display_has_no_caret_but_carries_offset() {
        let err = SelectorError::resolve(7, "없는 축").expecting(["para", "table"]);
        let text = err.to_string();
        assert!(text.contains("[문자 7]"));
        assert!(text.contains("para | table"));
        assert!(!text.contains('^'));
    }
}

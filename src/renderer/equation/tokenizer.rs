//! 한컴 수식 스크립트 토크나이저
//!
//! 수식 스크립트 문자열을 토큰으로 분리한다.

/// 토큰 타입
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenType {
    Command,     // 명령어 (OVER, SQRT, alpha 등)
    Number,      // 숫자 (123, 3.14)
    Symbol,      // 기호 (+, -, =, <, > 등)
    Text,        // 일반 텍스트 (한글 등)
    LBrace,      // {
    RBrace,      // }
    LParen,      // (
    RParen,      // )
    LBracket,    // [
    RBracket,    // ]
    Subscript,   // _
    Superscript, // ^
    Whitespace,  // 공백 특수문자 (~, `, #, &)
    Quoted,      // 따옴표로 묶인 문자열 ("...")
    Eof,         // 끝
}

/// 토큰
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub ty: TokenType,
    pub value: String,
    pub pos: usize,
    /// 이 토큰 앞에 일반 공백이 있었는지. 무브레이스 첨자 operand 의 경계 판정에 쓰인다
    /// (HWP 수식에서 일반 공백은 시각적으로는 무의미하나 operand 구분자 역할을 한다, #1304).
    pub space_before: bool,
}

impl Token {
    pub fn new(ty: TokenType, value: impl Into<String>, pos: usize) -> Self {
        Self {
            ty,
            value: value.into(),
            pos,
            space_before: false,
        }
    }

    pub fn eof(pos: usize) -> Self {
        Self {
            ty: TokenType::Eof,
            value: String::new(),
            pos,
            space_before: false,
        }
    }
}

/// 토크나이저
pub struct Tokenizer {
    chars: Vec<char>,
    pos: usize,
    /// 직전 `next_token` 호출이 토큰 앞에서 일반 공백을 건너뛰었는지 (#1304).
    last_had_space: bool,
}

impl Tokenizer {
    pub fn new(script: &str) -> Self {
        Self {
            chars: script.chars().collect(),
            pos: 0,
            last_had_space: false,
        }
    }

    fn peek(&self, offset: usize) -> Option<char> {
        self.chars.get(self.pos + offset).copied()
    }

    fn current(&self) -> Option<char> {
        self.peek(0)
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.current();
        if ch.is_some() {
            self.pos += 1;
        }
        ch
    }

    fn skip_spaces(&mut self) {
        // 일반 공백/탭 + 개행. HWP 수식 스크립트는 `#`/`&` 으로 명시적 행/탭 구분을 하므로
        // 실제 개행 문자는 의미 없는 포맷팅으로 간주하여 건너뛴다 (#505).
        while matches!(
            self.current(),
            Some(' ') | Some('\t') | Some('\n') | Some('\r')
        ) {
            self.pos += 1;
        }
    }

    /// 위치 `self.pos`부터 키워드가 prefix로 매치되는지 확인
    fn matches_at(&self, kw: &str) -> bool {
        let kw_chars: Vec<char> = kw.chars().collect();
        if self.pos + kw_chars.len() > self.chars.len() {
            return false;
        }
        for (i, &c) in kw_chars.iter().enumerate() {
            if self.chars[self.pos + i] != c {
                return false;
            }
        }
        true
    }

    /// 위치 `self.pos`부터 ASCII 키워드가 대소문자 무시 prefix로 매치되는지 확인
    fn matches_at_ascii_ci(&self, kw: &str) -> bool {
        let kw_chars: Vec<char> = kw.chars().collect();
        if self.pos + kw_chars.len() > self.chars.len() {
            return false;
        }
        for (i, &c) in kw_chars.iter().enumerate() {
            if !self.chars[self.pos + i].eq_ignore_ascii_case(&c) {
                return false;
            }
        }
        true
    }

    /// 명령어/식별자 읽기 (영문자+숫자)
    ///
    /// hwpeq 문법: 폰트 스타일 키워드(`bold`/`it`/`rm`)는 식별자에 공백 없이
    /// 붙어 쓰일 수 있고(예: `rmK`, `itl`, `boldX`), 키워드 길이만큼만 소비된 뒤
    /// 나머지는 별개 토큰이 된다. 키워드 직후가 식별자 종료(공백/기호/EOF)인
    /// 경우에는 분리하지 않는다.
    ///
    /// [Task #576] times/sim 연산자 키워드도 변수와 인접 시 분리.
    /// HWP 수식 script 에서 "{a timesm}" → "a × m", "rm X simZ" → "X ~ Z"
    /// 의미. 광범위 sweep (158 fixture / 563 unique scripts) 결과 결함 발현
    /// 키워드는 times/sim 만 (대소문자 4 개). alpha/sqrt 등은 항상 공백
    /// 구분되어 prefix-split 불필요 — 그리스 문자 prefix 충돌 회귀 위험 0.
    ///
    /// [Task #1122] HWP 수식 script 에서 분모 숫자가 OVER/ATOP 에 붙는 경우
    /// (`11 over20`, `3 over5`)가 있어 over/atop 뒤가 숫자인 경우에만 분리한다.
    fn read_command(&mut self) -> Token {
        let start = self.pos;

        // OVER/ATOP 분리: 뒤가 숫자(#1122, 임의 길이) 또는 [#1204-E] 짧은(≤2) 글자
        // 피연산자(`overa^2`,`overdx`)면 분리. 단 `overlap`/`overline` 등 긴 word·keyword 는
        // 유지 (trailing alnum ≥3 또는 keyword 는 분리하지 않음).
        for kw in ["over", "atop"] {
            if self.matches_at_ascii_ci(kw) {
                let after = self.peek(kw.len());
                let split = match after {
                    Some(c) if c.is_ascii_digit() => true,
                    Some(c) if c.is_ascii_alphabetic() => {
                        // kw 뒤 연속 alnum 길이
                        let mut j = kw.len();
                        while matches!(self.peek(j), Some(d) if d.is_ascii_alphanumeric()) {
                            j += 1;
                        }
                        j - kw.len() <= 2
                    }
                    _ => false,
                };
                if split {
                    let value: String = self.chars[start..start + kw.len()].iter().collect();
                    self.pos += kw.len();
                    return Token::new(TokenType::Command, value, start);
                }
            }
        }

        for kw in [
            "bold", "it", "rm", "times", "sim", "TIMES", "SIM", "RM", "IT", "BOLD",
        ] {
            if self.matches_at(kw) {
                let after = self.peek(kw.len());
                if matches!(after, Some(c) if c.is_ascii_alphanumeric()) {
                    self.pos += kw.len();
                    return Token::new(TokenType::Command, kw, start);
                }
            }
        }

        // [#1204-A] root/sqrt + 관계연산자(GEQ/LEQ/GE/LE) 가 숫자에 붙은 경우 분리.
        // (`root3`→√3, `GEQ5`→≥5, `GE0`→≥0.) over/atop 와 동일하게 숫자 한정 —
        // letter 변수와의 충돌(Task #576 주석) 회피. GEQ/LEQ 를 GE/LE 보다 먼저 검사
        // (긴 매칭 우선; digit-guard 로도 안전하나 명시적 순서 유지).
        for kw in ["root", "sqrt", "ROOT", "SQRT", "GEQ", "LEQ", "GE", "LE"] {
            if self.matches_at(kw) {
                let after = self.peek(kw.len());
                if matches!(after, Some(c) if c.is_ascii_digit()) {
                    self.pos += kw.len();
                    return Token::new(TokenType::Command, kw, start);
                }
            }
        }

        // [#1204-C] prime 이 글자/숫자에 붙은 경우 분리 (`primeF`→′ F).
        for kw in ["prime", "PRIME"] {
            if self.matches_at(kw) {
                let after = self.peek(kw.len());
                if matches!(after, Some(c) if c.is_ascii_alphanumeric()) {
                    self.pos += kw.len();
                    return Token::new(TokenType::Command, kw, start);
                }
            }
        }

        let mut value = String::new();
        while let Some(ch) = self.current() {
            if ch.is_ascii_alphanumeric() {
                value.push(ch);
                self.pos += 1;
            } else {
                break;
            }
        }

        // [#1204-E] glued keyword prefix 분리: hwpeq 는 키워드를 피연산자에 공백 없이
        // 붙여 쓸 수 있다 (`tanx`→tan x, `barMH`→bar MH, `LEQb`→≤ b, `trianglePQR`→△ PQR).
        // run 전체가 키워드가 아니면, 앞쪽에 붙은 알려진 키워드의 최장 prefix 를 분리한다.
        // 나머지는 다음 호출에서 재토큰화되어 chain (`rmbarFF`→rm bar FF) 도 처리된다.
        if let Some(k) = longest_keyword_prefix(&value) {
            self.pos = start + k;
            return Token::new(TokenType::Command, value[..k].to_string(), start);
        }

        Token::new(TokenType::Command, value, start)
    }

    /// 숫자 읽기 (정수, 소수)
    fn read_number(&mut self) -> Token {
        let start = self.pos;
        let mut value = String::new();
        while let Some(ch) = self.current() {
            if ch.is_ascii_digit() {
                value.push(ch);
                self.pos += 1;
            } else {
                break;
            }
        }
        // 소수점
        if self.current() == Some('.') {
            if let Some(next) = self.peek(1) {
                if next.is_ascii_digit() {
                    value.push('.');
                    self.pos += 1;
                    while let Some(ch) = self.current() {
                        if ch.is_ascii_digit() {
                            value.push(ch);
                            self.pos += 1;
                        } else {
                            break;
                        }
                    }
                }
            }
        }
        Token::new(TokenType::Number, value, start)
    }

    /// 따옴표로 묶인 문자열 읽기 ("...")
    fn read_quoted(&mut self) -> Token {
        let start = self.pos;
        self.pos += 1; // 여는 따옴표 건너뛰기
        let mut value = String::new();
        while let Some(ch) = self.current() {
            if ch == '"' {
                self.pos += 1; // 닫는 따옴표 건너뛰기
                break;
            }
            value.push(ch);
            self.pos += 1;
        }
        Token::new(TokenType::Quoted, value, start)
    }

    /// LaTeX 명령어 읽기 (\frac, \sqrt 등)
    fn read_latex_command(&mut self) -> Token {
        let start = self.pos;
        self.pos += 1; // 백슬래시 건너뛰기
        let mut value = String::new();
        while let Some(ch) = self.current() {
            if ch.is_ascii_alphabetic() {
                value.push(ch);
                self.pos += 1;
            } else {
                break;
            }
        }
        Token::new(TokenType::Command, value, start)
    }

    /// 다중 문자 기호 읽기 (<=, >=, !=, ==, <<, >>, <<<, >>>)
    fn try_read_multi_char_symbol(&mut self) -> Option<Token> {
        let start = self.pos;
        let ch = self.current()?;
        let next = self.peek(1).unwrap_or('\0');
        let third = self.peek(2).unwrap_or('\0');

        // 3문자 기호
        if (ch == '<' || ch == '>') && next == ch && third == ch {
            self.pos += 3;
            let s: String = [ch, ch, ch].iter().collect();
            return Some(Token::new(TokenType::Symbol, s, start));
        }

        // -> (화살표 축약)
        if ch == '-' && next == '>' {
            self.pos += 2;
            return Some(Token::new(TokenType::Symbol, "->", start));
        }

        // 2문자 기호
        let two: String = [ch, next].iter().collect();
        if matches!(two.as_str(), "<=" | ">=" | "!=" | "==" | "<<" | ">>") {
            self.pos += 2;
            return Some(Token::new(TokenType::Symbol, two, start));
        }

        None
    }

    /// 다음 토큰 반환
    pub fn next_token(&mut self) -> Token {
        // 일반 공백 건너뛰기 — 건너뛴 공백 유무를 기록(#1304: operand 경계 판정용)
        let before = self.pos;
        self.skip_spaces();
        self.last_had_space = self.pos > before;

        if self.pos >= self.chars.len() {
            return Token::eof(self.pos);
        }

        let start = self.pos;
        let ch = self.chars[self.pos];

        // 특수 공백 문자
        if matches!(ch, '~' | '`' | '#' | '&') {
            self.pos += 1;
            return Token::new(TokenType::Whitespace, ch.to_string(), start);
        }

        // 괄호
        match ch {
            '{' => {
                self.pos += 1;
                return Token::new(TokenType::LBrace, "{", start);
            }
            '}' => {
                self.pos += 1;
                return Token::new(TokenType::RBrace, "}", start);
            }
            '(' => {
                self.pos += 1;
                return Token::new(TokenType::LParen, "(", start);
            }
            ')' => {
                self.pos += 1;
                return Token::new(TokenType::RParen, ")", start);
            }
            '[' => {
                self.pos += 1;
                return Token::new(TokenType::LBracket, "[", start);
            }
            ']' => {
                self.pos += 1;
                return Token::new(TokenType::RBracket, "]", start);
            }
            _ => {}
        }

        // 첨자
        if ch == '_' {
            self.pos += 1;
            return Token::new(TokenType::Subscript, "_", start);
        }
        if ch == '^' {
            self.pos += 1;
            return Token::new(TokenType::Superscript, "^", start);
        }

        // 따옴표 문자열
        if ch == '"' {
            return self.read_quoted();
        }

        // LaTeX \\(줄바꿈) — 두 개의 백슬래시 연속
        if ch == '\\' && self.peek(1) == Some('\\') {
            self.pos += 2;
            return Token::new(TokenType::Whitespace, "#", start);
        }

        // LaTeX spacing: \, \: \; \! → thin/medium/thick/negative space
        if ch == '\\' {
            if let Some(nc) = self.peek(1) {
                let space_cmd = match nc {
                    ',' => Some("THINSPACE"),
                    ':' => Some("MEDSPACE"),
                    ';' => Some("THICKSPACE"),
                    '!' => Some("NEGSPACE"),
                    _ => None,
                };
                if let Some(cmd) = space_cmd {
                    self.pos += 2;
                    return Token::new(TokenType::Command, cmd, start);
                }
            }
        }

        // LaTeX escaped braces and special chars: \{ \} \| \#
        if ch == '\\' {
            if let Some(nc) = self.peek(1) {
                let brace_tok = match nc {
                    '{' => Some(Token::new(TokenType::LBrace, "{", start)),
                    '}' => Some(Token::new(TokenType::RBrace, "}", start)),
                    '|' => Some(Token::new(TokenType::Symbol, "|", start)),
                    '#' => Some(Token::new(TokenType::Whitespace, "#", start)),
                    _ => None,
                };
                if let Some(tok) = brace_tok {
                    self.pos += 2;
                    return tok;
                }
            }
        }

        // LaTeX 명령어: \frac, \sqrt, \pm 등
        if ch == '\\' && self.peek(1).map_or(false, |c| c.is_ascii_alphabetic()) {
            return self.read_latex_command();
        }

        // 다중 문자 기호
        if let Some(tok) = self.try_read_multi_char_symbol() {
            return tok;
        }

        // 단일 기호
        if matches!(
            ch,
            '+' | '-' | '*' | '/' | '=' | '<' | '>' | '!' | '|' | ':' | ',' | '.' | '\''
        ) {
            self.pos += 1;
            return Token::new(TokenType::Symbol, ch.to_string(), start);
        }

        // 숫자
        if ch.is_ascii_digit() {
            return self.read_number();
        }

        // 명령어/식별자 (영문자)
        if ch.is_ascii_alphabetic() {
            return self.read_command();
        }

        // HWP 수식 PUA(사용자 정의 영역) 특수기호 매핑 (#1343)
        // 한컴 수식 편집기가 PUA로 저장한 기호(예: 조건부 막대 U+E04D)를 표준 기호로
        // 변환한다. 매핑이 없으면 아래 Text 분기로 폴백하여 글리프 미존재 시 두부(▦)가 된다.
        if let Some(sym) = super::symbols::lookup_equation_pua(ch) {
            self.pos += 1;
            return Token::new(TokenType::Symbol, sym, start);
        }

        // 기타 문자 (한글 등) — 연속 비-ASCII 문자를 하나의 Text 토큰으로
        if !ch.is_ascii() {
            let mut value = String::new();
            while let Some(c) = self.current() {
                if c.is_ascii() || c == ' ' {
                    break;
                }
                value.push(c);
                self.pos += 1;
            }
            return Token::new(TokenType::Text, value, start);
        }

        // 알 수 없는 문자
        self.pos += 1;
        Token::new(TokenType::Text, ch.to_string(), start)
    }

    /// 전체 토큰 리스트 반환
    pub fn tokenize(mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        loop {
            let mut token = self.next_token();
            token.space_before = self.last_had_space;
            let is_eof = token.ty == TokenType::Eof;
            tokens.push(token);
            if is_eof {
                break;
            }
        }
        tokens
    }
}

/// [#1204-E] `s` 가 알려진 수식 키워드(함수/기호/장식/구조/글꼴)인지 (대소문자 무시).
/// run 전체가 keyword 면 분리하지 않기 위한 광범위 판정.
fn is_eq_keyword(s: &str) -> bool {
    use super::symbols::{
        is_function, is_structure_command, lookup_symbol, DECORATIONS, FONT_STYLES,
    };
    let lower = s.to_ascii_lowercase();
    is_function(s)
        || is_function(lower.as_str())
        || lookup_symbol(s).is_some()
        || DECORATIONS.contains_key(s)
        || DECORATIONS.contains_key(lower.as_str())
        || FONT_STYLES.contains_key(s)
        || FONT_STYLES.contains_key(lower.as_str())
        || is_structure_command(&s.to_ascii_uppercase())
}

/// [#1204-E] glued 분리에 **안전한** 키워드 allowlist (소문자, 대소문자 무시 비교).
/// hwpeq 에서 피연산자에 공백 없이 붙는 게 흔하고, 변수/그리스/`over`·`root` 등
/// 모호 prefix 와 충돌하지 않는 명령만 포함한다.
/// (제외: greek(alphabet), over/atop(overlap, #1122), root/sqrt(rootn), arg/max(argmax),
///  ge/le 2자(LEFT 등 충돌) — 이들은 분리하지 않는다.)
const GLUE_SAFE: &[&str] = &[
    // 삼각/쌍곡 함수 (longest-first 는 is_eq_keyword whole-check 가 보장)
    "sinh",
    "cosh",
    "tanh",
    "coth",
    "sech",
    "csch",
    "sin",
    "cos",
    "tan",
    "sec",
    "csc",
    "cot",
    // 장식
    "overrightarrow",
    "overleftarrow",
    "widehat",
    "widetilde",
    "overline",
    "underline",
    "bar",
    "vec",
    "hat",
    "tilde",
    "dot",
    "ddot",
    "acute",
    "grave",
    "check",
    "breve",
    // 관계연산자(3자) + 도형 + 집합연산
    "leq",
    "geq",
    "neq",
    "triangle",
    "angle",
    "cap",
    "cup",
    // [#1204] 생략기호(dots) — `cdotscdots`(⋯⋯) 처럼 공백 없이 연접하면 leak.
    // 5자 명시 키워드만(모호 4자 `dots` 는 일반 변수 충돌 우려로 제외).
    "cdots",
    "ldots",
    "vdots",
    "ddots",
];

fn is_glue_safe(s: &str) -> bool {
    let lower = s.to_ascii_lowercase();
    GLUE_SAFE.contains(&lower.as_str())
}

/// [#1204-E] glued identifier 의 앞쪽에 붙은 allowlist 키워드의 최장 prefix 길이.
/// run 전체가 (광범위) 키워드면 분리 불필요(None). 최소 keyword 2자 + remainder 1자.
/// `value` 는 ASCII alnum run 이므로 byte index == char index.
fn longest_keyword_prefix(value: &str) -> Option<usize> {
    if value.len() < 3 || is_eq_keyword(value) {
        return None;
    }
    // 최장 우선 (len-1 .. 2), allowlist 만 분리
    (2..value.len()).rev().find(|&k| is_glue_safe(&value[..k]))
}

/// 수식 스크립트를 토큰 리스트로 변환
pub fn tokenize(script: &str) -> Vec<Token> {
    Tokenizer::new(script).tokenize()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn values(tokens: &[Token]) -> Vec<&str> {
        tokens
            .iter()
            .filter(|t| t.ty != TokenType::Eof)
            .map(|t| t.value.as_str())
            .collect()
    }

    fn types(tokens: &[Token]) -> Vec<TokenType> {
        tokens
            .iter()
            .filter(|t| t.ty != TokenType::Eof)
            .map(|t| t.ty)
            .collect()
    }

    #[test]
    fn test_simple_fraction() {
        let tokens = tokenize("1 over 2");
        assert_eq!(values(&tokens), vec!["1", "over", "2"]);
        assert_eq!(
            types(&tokens),
            vec![TokenType::Number, TokenType::Command, TokenType::Number]
        );
    }

    #[test]
    fn test_task1122_over_atop_followed_by_number_splits() {
        let tokens = tokenize("11 over20");
        assert_eq!(values(&tokens), vec!["11", "over", "20"]);
        assert_eq!(
            types(&tokens),
            vec![TokenType::Number, TokenType::Command, TokenType::Number]
        );

        let tokens = tokenize("7 OVER10");
        assert_eq!(values(&tokens), vec!["7", "OVER", "10"]);
        assert_eq!(
            types(&tokens),
            vec![TokenType::Number, TokenType::Command, TokenType::Number]
        );

        let tokens = tokenize("a atop2");
        assert_eq!(values(&tokens), vec!["a", "atop", "2"]);
    }

    #[test]
    fn test_task1122_over_prefix_non_numeric_identifiers_stay_intact() {
        let tokens = tokenize("overlap overline overset");
        assert_eq!(values(&tokens), vec!["overlap", "overline", "overset"]);

        let tokens = tokenize(r"\overline{AB}");
        assert_eq!(values(&tokens), vec!["overline", "{", "AB", "}"]);
    }

    #[test]
    fn test_superscript() {
        let tokens = tokenize("E=mc^2");
        assert_eq!(values(&tokens), vec!["E", "=", "mc", "^", "2"]);
    }

    #[test]
    fn test_subscript_superscript() {
        let tokens = tokenize("sum_{i=0}^n");
        assert_eq!(
            values(&tokens),
            vec!["sum", "_", "{", "i", "=", "0", "}", "^", "n"]
        );
    }

    #[test]
    fn test_whitespace_chars() {
        let tokens = tokenize("a~b`c#d&e");
        let ws_tokens: Vec<_> = tokens
            .iter()
            .filter(|t| t.ty == TokenType::Whitespace)
            .map(|t| t.value.as_str())
            .collect();
        assert_eq!(ws_tokens, vec!["~", "`", "#", "&"]);
    }

    #[test]
    fn test_korean_text() {
        let tokens = tokenize("평점=입찰가격");
        assert_eq!(
            types(&tokens),
            vec![TokenType::Text, TokenType::Symbol, TokenType::Text]
        );
        assert_eq!(values(&tokens), vec!["평점", "=", "입찰가격"]);
    }

    #[test]
    fn test_quoted_string() {
        let tokens = tokenize("\"1234567890\" over 5");
        assert_eq!(
            types(&tokens),
            vec![TokenType::Quoted, TokenType::Command, TokenType::Number]
        );
        assert_eq!(values(&tokens), vec!["1234567890", "over", "5"]);
    }

    #[test]
    fn test_multi_char_symbols() {
        let tokens = tokenize("a <= b >= c != d == e");
        let syms: Vec<_> = tokens
            .iter()
            .filter(|t| t.ty == TokenType::Symbol)
            .map(|t| t.value.as_str())
            .collect();
        assert_eq!(syms, vec!["<=", ">=", "!=", "=="]);
    }

    #[test]
    fn test_pua_conditional_bar() {
        // #1343: 한컴 수식 PUA 조건부 막대 U+E04D → 단일 `|` 기호와 동일 토큰
        let tokens = tokenize("rm P LEFT ( it A \u{E04D} B RIGHT )");
        let bar: Vec<_> = tokens
            .iter()
            .filter(|t| t.ty == TokenType::Symbol)
            .map(|t| t.value.as_str())
            .collect();
        assert_eq!(bar, vec!["|"]);
        // PUA 원형 코드포인트가 토큰에 남지 않아야 한다(두부 방지)
        assert!(tokens.iter().all(|t| !t.value.contains('\u{E04D}')));
    }

    #[test]
    fn test_arrow() {
        let tokens = tokenize("x->0");
        assert_eq!(values(&tokens), vec!["x", "->", "0"]);
    }

    #[test]
    fn test_left_right() {
        let tokens = tokenize("LEFT ( a over b RIGHT )");
        assert_eq!(
            values(&tokens),
            vec!["LEFT", "(", "a", "over", "b", "RIGHT", ")"]
        );
    }

    #[test]
    fn test_matrix() {
        let tokens = tokenize("matrix{a & b # c & d}");
        assert_eq!(
            values(&tokens),
            vec!["matrix", "{", "a", "&", "b", "#", "c", "&", "d", "}"]
        );
    }

    #[test]
    fn test_decimal_number() {
        let tokens = tokenize("3.14");
        assert_eq!(types(&tokens), vec![TokenType::Number]);
        assert_eq!(values(&tokens), vec!["3.14"]);
    }

    #[test]
    fn test_sample_eq01() {
        // 실제 eq-01.hwp 수식 스크립트의 일부
        let tokens = tokenize("TIMES  LEFT ( {최저입찰가격} over {해당입찰가격} RIGHT )");
        let cmds: Vec<_> = tokens
            .iter()
            .filter(|t| t.ty == TokenType::Command)
            .map(|t| t.value.as_str())
            .collect();
        assert!(cmds.contains(&"TIMES"));
        assert!(cmds.contains(&"LEFT"));
        assert!(cmds.contains(&"over"));
        assert!(cmds.contains(&"RIGHT"));
    }

    // Task #488: 폰트 스타일 키워드(rm/it/bold) prefix 분리

    #[test]
    fn test_font_style_prefix_rm_uppercase() {
        let tokens = tokenize("rmK ^{+}");
        assert_eq!(values(&tokens), vec!["rm", "K", "^", "{", "+", "}"]);
    }

    #[test]
    fn test_font_style_prefix_rm_compound() {
        let tokens = tokenize("rmCa ^{2+}");
        assert_eq!(values(&tokens), vec!["rm", "Ca", "^", "{", "2", "+", "}"]);
    }

    #[test]
    fn test_font_style_prefix_rm_lowercase() {
        let tokens = tokenize("1`rmmol");
        assert_eq!(values(&tokens), vec!["1", "`", "rm", "mol"]);
    }

    #[test]
    fn test_font_style_prefix_it_compound() {
        let tokens = tokenize("LEFT ( itaq RIGHT )");
        assert_eq!(values(&tokens), vec!["LEFT", "(", "it", "aq", "RIGHT", ")"]);
    }

    #[test]
    fn test_font_style_prefix_it_single_letter() {
        let tokens = tokenize("LEFT ( itl RIGHT )");
        assert_eq!(values(&tokens), vec!["LEFT", "(", "it", "l", "RIGHT", ")"]);
    }

    #[test]
    fn test_font_style_prefix_bold() {
        let tokens = tokenize("boldX");
        assert_eq!(values(&tokens), vec!["bold", "X"]);
    }

    #[test]
    fn test_font_style_keyword_alone_unchanged() {
        // 키워드 직후가 공백/기호/EOF: 분리하지 않고 그대로 키워드
        let tokens = tokenize("rm K");
        assert_eq!(values(&tokens), vec!["rm", "K"]);
        let tokens = tokenize("it{x}");
        assert_eq!(values(&tokens), vec!["it", "{", "x", "}"]);
        let tokens = tokenize("rm");
        assert_eq!(values(&tokens), vec!["rm"]);
    }

    // [#1204-A] root/sqrt 가 숫자에 붙은 경우 분리 (`root3`→√3)
    #[test]
    fn test_root_sqrt_prefix_split_on_digit() {
        assert_eq!(values(&tokenize("root3 y")), vec!["root", "3", "y"]);
        assert_eq!(values(&tokenize("sqrt5")), vec!["sqrt", "5"]);
        assert_eq!(
            values(&tokenize("2 over {root3 a}")),
            vec!["2", "over", "{", "root", "3", "a", "}"]
        );
        // 관계연산자도 숫자에 붙으면 분리 (`GEQ5`→≥5, `GE0`→≥0)
        assert_eq!(values(&tokenize("GEQ5")), vec!["GEQ", "5"]);
        assert_eq!(values(&tokenize("GE0")), vec!["GE", "0"]);
        assert_eq!(values(&tokenize("LEQ3")), vec!["LEQ", "3"]);
        // 숫자가 아니면 분리하지 않음 (letter 변수 충돌 회피)
        assert_eq!(values(&tokenize("rootn")), vec!["rootn"]);
        // 중괄호/공백 형태는 영향 없음
        assert_eq!(
            values(&tokenize("root {4} of {x}")),
            vec!["root", "{", "4", "}", "of", "{", "x", "}"]
        );
    }

    // [#1204-C] prime 이 alnum 에 붙은 경우 분리 (`primeF`→′ F)
    #[test]
    fn test_prime_prefix_split() {
        assert_eq!(values(&tokenize("F primeF")), vec!["F", "prime", "F"]);
        // 공백 형태는 기존대로
        assert_eq!(values(&tokenize("f prime")), vec!["f", "prime"]);
    }

    // [#1204-E] 함수/장식/관계연산자/도형 키워드가 글자에 붙은 경우 allowlist 분리.
    #[test]
    fn test_glued_keyword_letter_split() {
        assert_eq!(values(&tokenize("tanx")), vec!["tan", "x"]);
        assert_eq!(values(&tokenize("sinx")), vec!["sin", "x"]);
        assert_eq!(values(&tokenize("barMH")), vec!["bar", "MH"]);
        assert_eq!(values(&tokenize("LEQb")), vec!["LEQ", "b"]);
        assert_eq!(values(&tokenize("trianglePQR")), vec!["triangle", "PQR"]);
        // chain: rm + bar + FF
        assert_eq!(values(&tokenize("rmbarFF")), vec!["rm", "bar", "FF"]);
        // longest-match: cosh 는 cos+h 로 쪼개지지 않음
        assert_eq!(values(&tokenize("coshx")), vec!["cosh", "x"]);
    }

    // [#1204-E] 회귀 가드: greek/root/arg 등 모호 prefix 는 분리 금지.
    #[test]
    fn test_glued_keyword_no_oversplit() {
        assert_eq!(values(&tokenize("alphabet")), vec!["alphabet"]); // greek 제외
        assert_eq!(values(&tokenize("argmax")), vec!["argmax"]); // arg/max 제외
        assert_eq!(values(&tokenize("rootn")), vec!["rootn"]); // root letter 제외
    }

    // [#1204-E] cap/cup (집합연산) 가 글자에 붙은 경우 분리 (`capB`→∩ B)
    #[test]
    fn test_cap_cup_glued_split() {
        assert_eq!(values(&tokenize("capB")), vec!["cap", "B"]);
        assert_eq!(values(&tokenize("cupB")), vec!["cup", "B"]);
    }

    // [#1204] 생략기호 연접 분리 (`cdotscdots`→⋯⋯). 단일/공백형은 기존대로.
    #[test]
    fn test_dots_glued_split() {
        assert_eq!(values(&tokenize("cdotscdots")), vec!["cdots", "cdots"]);
        assert_eq!(values(&tokenize("cdots")), vec!["cdots"]);
        assert_eq!(values(&tokenize("a cdots b")), vec!["a", "cdots", "b"]);
        assert_eq!(values(&tokenize("ldotsldots")), vec!["ldots", "ldots"]);
    }

    // [#1204-E] over/atop 가 짧은(≤2) 글자 피연산자에 붙으면 분리(분수), 긴 word 는 유지.
    #[test]
    fn test_over_glued_short_letter_operand() {
        assert_eq!(values(&tokenize("overa")), vec!["over", "a"]);
        assert_eq!(values(&tokenize("overdx")), vec!["over", "dx"]);
        assert_eq!(values(&tokenize("x overy")), vec!["x", "over", "y"]);
        // 가드: 긴 word(≥3) 및 keyword 는 유지 (#1122)
        assert_eq!(values(&tokenize("overlap")), vec!["overlap"]);
        assert_eq!(values(&tokenize("overline")), vec!["overline"]);
        assert_eq!(values(&tokenize("overset")), vec!["overset"]);
    }

    #[test]
    fn test_existing_commands_unchanged() {
        // 기존 명령은 회귀 없음
        let tokens = tokenize("OVER MATRIX SQRT alpha beta");
        assert_eq!(
            values(&tokens),
            vec!["OVER", "MATRIX", "SQRT", "alpha", "beta"]
        );
    }

    #[test]
    fn test_latex_command_prefix() {
        let tokens = tokenize(r"\frac{1}{2}");
        assert_eq!(
            types(&tokens),
            vec![
                TokenType::Command,
                TokenType::LBrace,
                TokenType::Number,
                TokenType::RBrace,
                TokenType::LBrace,
                TokenType::Number,
                TokenType::RBrace,
            ]
        );
        assert_eq!(values(&tokens), vec!["frac", "{", "1", "}", "{", "2", "}"]);
    }

    // Task #576: times/sim 연산자 키워드 prefix 분리

    #[test]
    fn test_task576_times_lowercase_prefix_split() {
        let tokens = tokenize("a timesm");
        assert_eq!(values(&tokens), vec!["a", "times", "m"]);
    }

    #[test]
    fn test_task576_sim_lowercase_prefix_split() {
        let tokens = tokenize("rm X simZ");
        assert_eq!(values(&tokens), vec!["rm", "X", "sim", "Z"]);
    }

    #[test]
    fn test_task576_times_uppercase_prefix_split() {
        let tokens = tokenize("1TIMES10");
        assert_eq!(values(&tokens), vec!["1", "TIMES", "10"]);
    }

    #[test]
    fn test_task576_sim_uppercase_prefix_split() {
        let tokens = tokenize("rmA SIMC");
        assert_eq!(values(&tokens), vec!["rm", "A", "SIM", "C"]);
    }

    #[test]
    fn test_task576_alpha_no_split() {
        // 회귀 차단: alpha 는 그 자체 keyword 이므로 분리되면 안 됨
        let tokens = tokenize("alpha");
        assert_eq!(values(&tokens), vec!["alpha"]);
        // alphabet 은 일반 식별자 — 분리되지 않아야 함
        let tokens = tokenize("alphabet");
        assert_eq!(values(&tokens), vec!["alphabet"]);
    }

    #[test]
    fn test_task576_times_followed_by_space() {
        // 키워드 다음 공백/기호 오면 분리 불필요 (기존 동작 보존)
        let tokens = tokenize("a times b");
        assert_eq!(values(&tokens), vec!["a", "times", "b"]);
        let tokens = tokenize("rmA SIM C");
        assert_eq!(values(&tokens), vec!["rm", "A", "SIM", "C"]);
    }
}

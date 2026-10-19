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
    /// 저장 문서의 백슬래시 문자 전환으로 선택된 정체 글리프다.
    /// 인자 소유·토큰 위치·공백 경계와 독립적인 표시 속성이다.
    pub roman_literal: bool,
}

impl Token {
    pub fn new(ty: TokenType, value: impl Into<String>, pos: usize) -> Self {
        Self {
            ty,
            value: value.into(),
            pos,
            space_before: false,
            roman_literal: false,
        }
    }

    pub fn eof(pos: usize) -> Self {
        Self {
            ty: TokenType::Eof,
            value: String::new(),
            pos,
            space_before: false,
            roman_literal: false,
        }
    }
}

/// 토크나이저
pub struct Tokenizer {
    chars: Vec<char>,
    pos: usize,
    /// 직전 `next_token` 호출이 토큰 앞에서 일반 공백을 건너뛰었는지 (#1304).
    last_had_space: bool,
    /// 저장 문서의 한컴 명령 해석과 공개 LaTeX 호환 해석을 구분한다.
    document_commands: bool,
}

impl Tokenizer {
    pub fn new(script: &str) -> Self {
        Self {
            chars: script.chars().collect(),
            pos: 0,
            last_had_space: false,
            document_commands: false,
        }
    }

    pub(crate) fn for_document(script: &str) -> Self {
        Self {
            document_commands: true,
            ..Self::new(script)
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
    /// 키워드는 times/sim이었다 (대소문자 4개). 그리스 기호도 한컴 명령이며
    /// 붙은 피연산자와 분리한다. 문자 그대로의 문자열은 따옴표로 구분한다.
    ///
    /// [Task #1122] HWP 수식 script 에서 분모 숫자가 OVER/ATOP 에 붙는 경우
    /// (`11 over20`, `3 over5`)가 있어 over/atop 뒤가 숫자인 경우에만 분리한다.
    fn read_command(&mut self) -> Token {
        let start = self.pos;

        // 한컴 Print는 OVERSET을 OVER + SET으로 읽지만 OVERLINE은
        // 하나의 장식 명령으로 읽는다. 긴 명령을 먼저 읽고 남은 항을
        // 다음 토큰에 맡긴다. 혼합 대소문자를 새로운 명령으로 합치지 않는다.
        if self.document_commands {
            for name in ["OVERLINE", "UNDERLINE", "OVER", "UNDER", "ATOP"] {
                if self.matches_at(name) || self.matches_at(&name.to_ascii_lowercase()) {
                    self.pos += name.len();
                    let value = if name == "UNDER" {
                        "UNDERLINE".to_owned()
                    } else {
                        self.chars[start..self.pos].iter().collect()
                    };
                    return Token::new(TokenType::Command, value, start);
                }
            }
        }

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

        // 글꼴 전환은 붙은 식별자에서도 parser와 같은 대소문자 규칙을 쓴다.
        // 토큰의 원래 철자는 보존하고 나머지 피연산자만 다음 토큰으로 남긴다.
        for kw in [
            "bold", "it", "rm", "times", "sim", "TIMES", "SIM", "RM", "IT", "BOLD",
        ] {
            let matches = self.matches_at(kw)
                || (matches!(kw, "bold" | "it" | "rm") && self.matches_at_ascii_ci(kw));
            if matches {
                let after = self.peek(kw.len());
                if matches!(after, Some(c) if c.is_ascii_alphanumeric()) {
                    self.pos += kw.len();
                    let value: String = self.chars[start..self.pos].iter().collect();
                    return Token::new(TokenType::Command, value, start);
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

        // 곱셈 기호에 숫자가 붙어도 같은 연산자·피연산자 경계다.
        // cdots나 cdotValue 같은 다른 명령·식별자는 숫자 조건에 해당하지 않는다.
        if self.matches_at_ascii_ci("cdot") && self.peek(4).is_some_and(|ch| ch.is_ascii_digit()) {
            self.pos += 4;
            let value = self.chars[start..self.pos].iter().collect::<String>();
            return Token::new(TokenType::Command, value, start);
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

        // 알려진 그리스 기호와 RIGHT가 닫는 구분자 앞에서 붙은 경우다.
        // sigmaRIGHT )를 문자로 그리지 않으며 뒤의 구분자 경계를 보존한다.
        if let Some(prefix_len) = value
            .len()
            .checked_sub(5)
            .filter(|&len| len > 0 && value[len..].eq_ignore_ascii_case("right"))
        {
            let next = self.chars[self.pos..]
                .iter()
                .copied()
                .find(|ch| !ch.is_ascii_whitespace());
            let greek_symbol = is_greek_command(&value[..prefix_len]);
            if matches!(next, Some(')' | ']' | '}' | '|')) && greek_symbol {
                self.pos = start + prefix_len;
                return Token::new(TokenType::Command, value[..prefix_len].to_string(), start);
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
        if self.document_commands && !value.is_empty() {
            // 저장 문서의 백슬래시는 뒤 명령 이름을 문자로 남긴다.
            // 공개 tokenizer의 LaTeX 확장과 pm/mp 별칭은 별도로 유지한다.
            return Token::new(TokenType::Text, value, start);
        }
        // LaTeX 확장과 한컴의 일반 pm/mp 식별자를 구분한다.
        match value.as_str() {
            "pm" => Token::new(TokenType::Command, "PLUSMINUS", start),
            "mp" => Token::new(TokenType::Command, "MINUSPLUS", start),
            // 한컴 NOT의 한 글자 소비와 LaTeX의 그룹 인자를 구별한다.
            "not" => Token::new(TokenType::Command, r"\not", start),
            _ => Token::new(TokenType::Command, value, start),
        }
    }

    /// 다중 문자 기호 읽기 (<=, >=, !=, ==, <<, >>, <<<, >>>)
    fn try_read_multi_char_symbol(&mut self) -> Option<Token> {
        let start = self.pos;
        let ch = self.current()?;
        let next = self.peek(1).unwrap_or('\0');
        let third = self.peek(2).unwrap_or('\0');

        // 붙은 ASCII 부호만 축약한다. 공백·Unicode minus·인용은 보존한다.
        if let Some(symbol) = match (ch, next) {
            ('+', '-') => Some("±"),
            ('-', '+') => Some("∓"),
            _ => None,
        } {
            self.pos += 2;
            return Some(Token::new(TokenType::Symbol, symbol, start));
        }

        // 3문자 기호
        if (ch == '<' || ch == '>') && next == ch && third == ch {
            self.pos += 3;
            let s: String = [ch, ch, ch].iter().collect();
            return Some(Token::new(TokenType::Symbol, s, start));
        }

        // 붙은 ASCII 화살표 축약. `<`-`의 명시적 공백은 이 축약이 아니다.
        if matches!((ch, next), ('-', '>') | ('<', '-')) {
            self.pos += 2;
            return Some(Token::new(
                TokenType::Symbol,
                [ch, next].iter().collect::<String>(),
                start,
            ));
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

        // Unicode 산술·관계·논리 기호는 비-ASCII 피연산자와 붙어도 독립 기호다.
        if !ch.is_ascii()
            && (super::symbols::is_relation_symbol(&ch.to_string())
                || super::symbols::is_binary_symbol(&ch.to_string())
                || super::symbols::is_logical_symbol(&ch.to_string()))
        {
            self.pos += 1;
            return Token::new(TokenType::Symbol, ch.to_string(), start);
        }

        // 기타 문자 (한글 등) — 연속 비-ASCII 문자를 하나의 Text 토큰으로
        if !ch.is_ascii() {
            let mut value = String::new();
            while let Some(c) = self.current() {
                if c.is_ascii()
                    || c == ' '
                    || super::symbols::is_relation_symbol(&c.to_string())
                    || super::symbols::is_binary_symbol(&c.to_string())
                    || super::symbols::is_logical_symbol(&c.to_string())
                {
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
/// hwpeq에서 피연산자에 붙는 명령이다. 그리스 기호 명령은 별도 기호표에서
/// 확인하며, 문자 그대로의 식별자는 따옴표로 보호한다.
/// (제외: over/atop(overlap, #1122), root/sqrt(rootn), arg/max(argmax),
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
    "min",
    // 장식
    "overrightarrow",
    "overleftarrow",
    "widehat",
    "widetilde",
    "overline",
    "underline",
    "bar",
    "vec",
    // [#7593] dyad(양쪽 화살표) — `rm dyadAB` 처럼 붙여 쓰는 표기가 흔하다.
    "dyad",
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

/// 기호표가 실제 그리스 문자 하나로 해석하는 명령만 접두사로 분리한다.
/// 대소문자별 그리스 이름은 기호표의 기존 계약을 그대로 따른다.
fn is_greek_command(command: &str) -> bool {
    super::symbols::lookup_symbol(command).is_some_and(|symbol| {
        let mut chars = symbol.chars();
        matches!((chars.next(), chars.next()), (Some(ch), None) if ('\u{0370}'..='\u{03ff}').contains(&ch))
    })
}

fn is_glue_safe(s: &str) -> bool {
    let lower = s.to_ascii_lowercase();
    GLUE_SAFE.contains(&lower.as_str()) || is_greek_command(s)
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
    // [#7105] 레거시 hwpeq5 OLE 방언(`\CMD 인자 \TAB`)은 먼저 현행 문법으로 옮긴다.
    // 게이트는 `\TAB` 존재이므로 현행 스크립트는 원문 그대로 지나간다.
    match super::legacy_hwpeq::normalize(script) {
        Some(normalized) => Tokenizer::new(&normalized).tokenize(),
        None => Tokenizer::new(script).tokenize(),
    }
}

#[cfg(test)]
#[path = "tokenizer/tests.rs"]
mod tests;

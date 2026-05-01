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
}

impl Token {
    pub fn new(ty: TokenType, value: impl Into<String>, pos: usize) -> Self {
        Self { ty, value: value.into(), pos }
    }

    pub fn eof(pos: usize) -> Self {
        Self { ty: TokenType::Eof, value: String::new(), pos }
    }
}

/// 토크나이저
pub struct Tokenizer {
    chars: Vec<char>,
    pos: usize,
}

impl Tokenizer {
    pub fn new(script: &str) -> Self {
        Self {
            chars: script.chars().collect(),
            pos: 0,
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
        while matches!(self.current(), Some(' ') | Some('\t') | Some('\n') | Some('\r')) {
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

    /// 명령어/식별자 읽기 (영문자+숫자)
    ///
    /// hwpeq 문법: 폰트 스타일 키워드(`bold`/`it`/`rm`)는 식별자에 공백 없이
    /// 붙어 쓰일 수 있고(예: `rmK`, `itl`, `boldX`), 키워드 길이만큼만 소비된 뒤
    /// 나머지는 별개 토큰이 된다. 키워드 직후가 식별자 종료(공백/기호/EOF)인
    /// 경우에는 분리하지 않는다.
    fn read_command(&mut self) -> Token {
        let start = self.pos;

        for kw in ["bold", "it", "rm"] {
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
        // 일반 공백 건너뛰기
        self.skip_spaces();

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
            '{' => { self.pos += 1; return Token::new(TokenType::LBrace, "{", start); }
            '}' => { self.pos += 1; return Token::new(TokenType::RBrace, "}", start); }
            '(' => { self.pos += 1; return Token::new(TokenType::LParen, "(", start); }
            ')' => { self.pos += 1; return Token::new(TokenType::RParen, ")", start); }
            '[' => { self.pos += 1; return Token::new(TokenType::LBracket, "[", start); }
            ']' => { self.pos += 1; return Token::new(TokenType::RBracket, "]", start); }
            _ => {}
        }

        // 첨자
        if ch == '_' { self.pos += 1; return Token::new(TokenType::Subscript, "_", start); }
        if ch == '^' { self.pos += 1; return Token::new(TokenType::Superscript, "^", start); }

        // 따옴표 문자열
        if ch == '"' {
            return self.read_quoted();
        }

        // 다중 문자 기호
        if let Some(tok) = self.try_read_multi_char_symbol() {
            return tok;
        }

        // 단일 기호
        if matches!(ch, '+' | '-' | '*' | '/' | '=' | '<' | '>' | '!' | '|' | ':' | ',' | '.' | '\'') {
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
            let token = self.next_token();
            let is_eof = token.ty == TokenType::Eof;
            tokens.push(token);
            if is_eof {
                break;
            }
        }
        tokens
    }
}

/// 수식 스크립트를 토큰 리스트로 변환
pub fn tokenize(script: &str) -> Vec<Token> {
    Tokenizer::new(script).tokenize()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn values(tokens: &[Token]) -> Vec<&str> {
        tokens.iter()
            .filter(|t| t.ty != TokenType::Eof)
            .map(|t| t.value.as_str())
            .collect()
    }

    fn types(tokens: &[Token]) -> Vec<TokenType> {
        tokens.iter()
            .filter(|t| t.ty != TokenType::Eof)
            .map(|t| t.ty)
            .collect()
    }

    #[test]
    fn test_simple_fraction() {
        let tokens = tokenize("1 over 2");
        assert_eq!(values(&tokens), vec!["1", "over", "2"]);
        assert_eq!(types(&tokens), vec![
            TokenType::Number, TokenType::Command, TokenType::Number
        ]);
    }

    #[test]
    fn test_superscript() {
        let tokens = tokenize("E=mc^2");
        assert_eq!(values(&tokens), vec!["E", "=", "mc", "^", "2"]);
    }

    #[test]
    fn test_subscript_superscript() {
        let tokens = tokenize("sum_{i=0}^n");
        assert_eq!(values(&tokens), vec!["sum", "_", "{", "i", "=", "0", "}", "^", "n"]);
    }

    #[test]
    fn test_whitespace_chars() {
        let tokens = tokenize("a~b`c#d&e");
        let ws_tokens: Vec<_> = tokens.iter()
            .filter(|t| t.ty == TokenType::Whitespace)
            .map(|t| t.value.as_str())
            .collect();
        assert_eq!(ws_tokens, vec!["~", "`", "#", "&"]);
    }

    #[test]
    fn test_korean_text() {
        let tokens = tokenize("평점=입찰가격");
        assert_eq!(types(&tokens), vec![
            TokenType::Text, TokenType::Symbol, TokenType::Text
        ]);
        assert_eq!(values(&tokens), vec!["평점", "=", "입찰가격"]);
    }

    #[test]
    fn test_quoted_string() {
        let tokens = tokenize("\"1234567890\" over 5");
        assert_eq!(types(&tokens), vec![
            TokenType::Quoted, TokenType::Command, TokenType::Number
        ]);
        assert_eq!(values(&tokens), vec!["1234567890", "over", "5"]);
    }

    #[test]
    fn test_multi_char_symbols() {
        let tokens = tokenize("a <= b >= c != d == e");
        let syms: Vec<_> = tokens.iter()
            .filter(|t| t.ty == TokenType::Symbol)
            .map(|t| t.value.as_str())
            .collect();
        assert_eq!(syms, vec!["<=", ">=", "!=", "=="]);
    }

    #[test]
    fn test_arrow() {
        let tokens = tokenize("x->0");
        assert_eq!(values(&tokens), vec!["x", "->", "0"]);
    }

    #[test]
    fn test_left_right() {
        let tokens = tokenize("LEFT ( a over b RIGHT )");
        assert_eq!(values(&tokens), vec!["LEFT", "(", "a", "over", "b", "RIGHT", ")"]);
    }

    #[test]
    fn test_matrix() {
        let tokens = tokenize("matrix{a & b # c & d}");
        assert_eq!(values(&tokens), vec![
            "matrix", "{", "a", "&", "b", "#", "c", "&", "d", "}"
        ]);
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
        let cmds: Vec<_> = tokens.iter()
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

    #[test]
    fn test_existing_commands_unchanged() {
        // 기존 명령은 회귀 없음
        let tokens = tokenize("OVER MATRIX SQRT alpha beta");
        assert_eq!(values(&tokens), vec!["OVER", "MATRIX", "SQRT", "alpha", "beta"]);
    }
}

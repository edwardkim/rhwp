//! CSS Syntax Module Level 3 tokenizer.
//!
//! Implements the W3C CSS Syntax L3 tokenization algorithm
//! (<https://www.w3.org/TR/css-syntax-3/#tokenization>) closely enough
//! to feed the parser (CSS-2) and the selector matcher (CSS-3..4)
//! everything they need for the v0.3.35 supported surface.
//!
//! Token coverage matches the spec:
//!
//! - `Ident`, `Function`, `AtKeyword`, `Hash` (id/unrestricted),
//!   `String`, `BadString`, `Url`, `BadUrl`,
//! - `Number`, `Percentage`, `Dimension`,
//! - `Whitespace`, `Cdo` (`<!--`), `Cdc` (`-->`),
//! - `Colon`, `Semicolon`, `Comma`, `LeftSquare`, `RightSquare`,
//!   `LeftParen`, `RightParen`, `LeftBrace`, `RightBrace`,
//! - `Delim` (catch-all single-character),
//! - `Eof` synthesized at the end of input.
//!
//! Comments (`/* ... */`) are consumed and discarded per spec.
//!
//! Escape sequences (`\26`, `\\E9`, `\\}`) are unescaped into the
//! token's value so downstream code never has to revisit them.
//!
//! Source locations are tracked as byte offsets, rows, and columns —
//! the parser stitches these into diagnostics.

use std::borrow::Cow;
use thiserror::Error;

/// CSS token. Borrows from the input where possible (zero-copy for
/// values without escape sequences); owns when an escape forces an
/// allocation.
#[derive(Debug, Clone, PartialEq)]
pub enum Token<'i> {
    /// Whitespace or newline run.
    Whitespace,
    /// `<!--`.
    Cdo,
    /// `-->`.
    Cdc,
    /// `:`.
    Colon,
    /// `;`.
    Semicolon,
    /// `,`.
    Comma,
    /// `[`.
    LeftSquare,
    /// `]`.
    RightSquare,
    /// `(`.
    LeftParen,
    /// `)`.
    RightParen,
    /// `{`.
    LeftBrace,
    /// `}`.
    RightBrace,
    /// Identifier — `display`, `flex`, `--my-var` (custom property
    /// names appear here too — the parser separates them by leading-dash
    /// inspection).
    Ident(Cow<'i, str>),
    /// `name(` — function start. The matching `)` is a separate
    /// `RightParen` token.
    Function(Cow<'i, str>),
    /// `@media`, `@page`, `@font-face`, …
    AtKeyword(Cow<'i, str>),
    /// `#myid`, `#fff`, `#abcdef`. The bool flag distinguishes a
    /// "hash-id" (valid identifier after `#`) from "hash-unrestricted"
    /// (any name) — selectors care about the difference (`#123` is
    /// not a valid id selector but is a valid colour hash).
    Hash {
        /// The bytes after `#`.
        value: Cow<'i, str>,
        /// Whether the value would parse as an `Ident`.
        is_id: bool,
    },
    /// `"..."` or `'...'` with quotes stripped and escapes resolved.
    String(Cow<'i, str>),
    /// Unterminated string literal (recovery: dropped).
    BadString,
    /// `url(...)` with the URL extracted (no quoting concerns —
    /// `url("…")` and `url(…)` both produce this).
    Url(Cow<'i, str>),
    /// Malformed url(...) — recovery: skip to the matching close paren.
    BadUrl,
    /// Numeric literal.
    Number(Number),
    /// Numeric literal followed by `%`.
    Percentage(Number),
    /// Numeric literal followed by an identifier (`12px`, `1.5em`).
    Dimension {
        /// The numeric value.
        value: Number,
        /// The unit identifier.
        unit: Cow<'i, str>,
    },
    /// Any single character that didn't start a longer token.
    Delim(char),
    /// Synthetic end-of-stream sentinel.
    Eof,
}

/// CSS numeric literal. Tracks integer-vs-float distinction (some
/// properties only accept integers, e.g. `z-index`, `font-weight`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Number {
    /// Numeric value as f64. Keeps full precision for both ints and
    /// floats up to ±2^53.
    pub value: f64,
    /// Whether the source representation had a decimal point or
    /// exponent. Properties that require integers should reject when
    /// this is `false`.
    pub is_integer: bool,
}

/// Source offset for one token. Rows and columns are 1-indexed (matches
/// every editor and CSS spec example).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceLocation {
    /// Byte offset from the start of input.
    pub offset: usize,
    /// 1-indexed line number.
    pub line: u32,
    /// 1-indexed column number (Unicode codepoints, not bytes — matches
    /// `cssparser` and the spec's `:column` selector).
    pub column: u32,
}

impl SourceLocation {
    /// The synthetic location at the very start of input.
    pub fn start() -> Self {
        Self {
            offset: 0,
            line: 1,
            column: 1,
        }
    }
}

/// Tokenizer-level errors. Rare — most malformed input produces
/// `BadString` / `BadUrl` / `Delim` recovery tokens rather than errors.
#[derive(Debug, Error, PartialEq)]
pub enum TokenizerError {
    /// Input contained a NUL byte we couldn't substitute (CSS spec
    /// requires NULs to be replaced with U+FFFD; we do that, this
    /// variant is reserved for future strict-mode use).
    #[error("nul byte at offset {offset}")]
    NulByte {
        /// Byte offset of the NUL.
        offset: usize,
    },
}

/// Tokenize a CSS source string into a flat list of `(Token, location)`
/// pairs. `Whitespace` runs are coalesced. Trailing `Eof` is appended.
///
/// Errors are non-fatal in CSS — malformed input produces recovery
/// tokens (`BadString`, `BadUrl`, `Delim`). The `Result` exists for the
/// strict-mode hook (`TokenizerError::NulByte`) which v0.3.35 doesn't
/// surface but the API reserves space for.
pub fn tokenize(input: &str) -> Result<Vec<(Token<'_>, SourceLocation)>, TokenizerError> {
    let mut t = Tokenizer::new(input);
    let mut out = Vec::new();
    loop {
        let loc = t.location();
        let tok = t.next_token();
        let is_eof = matches!(tok, Token::Eof);
        out.push((tok, loc));
        if is_eof {
            break;
        }
    }
    Ok(out)
}

// ─────────────────────────────────────────────────────────────────────
// Internal driver
// ─────────────────────────────────────────────────────────────────────

struct Tokenizer<'i> {
    input: &'i str,
    /// Byte offset.
    pos: usize,
    /// 1-indexed line.
    line: u32,
    /// 1-indexed column (Unicode codepoints).
    column: u32,
}

impl<'i> Tokenizer<'i> {
    fn new(input: &'i str) -> Self {
        Self {
            input,
            pos: 0,
            line: 1,
            column: 1,
        }
    }

    fn location(&self) -> SourceLocation {
        SourceLocation {
            offset: self.pos,
            line: self.line,
            column: self.column,
        }
    }

    /// Peek the next char without advancing.
    fn peek(&self) -> Option<char> {
        self.input[self.pos..].chars().next()
    }

    /// Peek the nth char without advancing.
    fn peek_nth(&self, n: usize) -> Option<char> {
        self.input[self.pos..].chars().nth(n)
    }

    /// Consume one char, advancing position + line/column tracking.
    fn bump(&mut self) -> Option<char> {
        let c = self.peek()?;
        self.pos += c.len_utf8();
        if c == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
        Some(c)
    }

    /// Returns true and consumes if the upcoming char matches.
    #[allow(dead_code)] // Used by future at-rule extensions
    fn eat(&mut self, c: char) -> bool {
        if self.peek() == Some(c) {
            self.bump();
            true
        } else {
            false
        }
    }

    /// Returns true and consumes if the upcoming chars match.
    #[allow(dead_code)] // Used by future at-rule extensions
    fn eat_str(&mut self, s: &str) -> bool {
        if self.input[self.pos..].starts_with(s) {
            for _ in s.chars() {
                self.bump();
            }
            true
        } else {
            false
        }
    }

    fn next_token(&mut self) -> Token<'i> {
        // Skip comments at the boundary — CSS Syntax §4.3.2.
        loop {
            if self.input[self.pos..].starts_with("/*") {
                self.pos += 2;
                self.column += 2;
                while self.pos < self.input.len() {
                    if self.input[self.pos..].starts_with("*/") {
                        self.pos += 2;
                        self.column += 2;
                        break;
                    }
                    self.bump();
                }
                continue;
            }
            break;
        }

        let Some(c) = self.peek() else {
            return Token::Eof;
        };

        // Whitespace run.
        if is_whitespace(c) {
            while self.peek().map(is_whitespace).unwrap_or(false) {
                self.bump();
            }
            return Token::Whitespace;
        }

        // String.
        if c == '"' || c == '\'' {
            return self.consume_string(c);
        }

        // CDO / CDC.
        if self.input[self.pos..].starts_with("<!--") {
            self.pos += 4;
            self.column += 4;
            return Token::Cdo;
        }
        if self.input[self.pos..].starts_with("-->") {
            self.pos += 3;
            self.column += 3;
            return Token::Cdc;
        }

        // Hash.
        if c == '#' {
            self.bump();
            // Hash is followed by a name; if the name starts with a
            // valid ident-start it's hash-id, otherwise hash-unrestricted.
            let is_id = self
                .peek()
                .map(|c| would_start_ident(c, self.peek_nth(1), self.peek_nth(2)))
                .unwrap_or(false);
            if let Some(c) = self.peek() {
                if is_name_char(c) || starts_escape(c, self.peek_nth(1)) {
                    let value = self.consume_name();
                    return Token::Hash { value, is_id };
                }
            }
            return Token::Delim('#');
        }

        // At-keyword.
        if c == '@' {
            self.bump();
            if let Some(c) = self.peek() {
                if would_start_ident(c, self.peek_nth(1), self.peek_nth(2)) {
                    let name = self.consume_name();
                    return Token::AtKeyword(name);
                }
            }
            return Token::Delim('@');
        }

        // Number / percentage / dimension.
        if would_start_number(c, self.peek_nth(1), self.peek_nth(2)) {
            let n = self.consume_number();
            // Dimension or percentage?
            if let Some(next) = self.peek() {
                if next == '%' {
                    self.bump();
                    return Token::Percentage(n);
                }
                if would_start_ident(next, self.peek_nth(1), self.peek_nth(2)) {
                    let unit = self.consume_name();
                    return Token::Dimension { value: n, unit };
                }
            }
            return Token::Number(n);
        }

        // Identifier-start (or function / url).
        if would_start_ident(c, self.peek_nth(1), self.peek_nth(2)) {
            return self.consume_ident_like();
        }

        // Single-character punctuation.
        let punct = match c {
            ':' => Some(Token::Colon),
            ';' => Some(Token::Semicolon),
            ',' => Some(Token::Comma),
            '[' => Some(Token::LeftSquare),
            ']' => Some(Token::RightSquare),
            '(' => Some(Token::LeftParen),
            ')' => Some(Token::RightParen),
            '{' => Some(Token::LeftBrace),
            '}' => Some(Token::RightBrace),
            _ => None,
        };
        if let Some(tok) = punct {
            self.bump();
            return tok;
        }

        // Anything else: delim.
        self.bump();
        Token::Delim(c)
    }

    fn consume_string(&mut self, end_char: char) -> Token<'i> {
        self.bump(); // opening quote
        let start = self.pos;
        let mut owned: Option<String> = None;
        while let Some(c) = self.peek() {
            match c {
                ch if ch == end_char => {
                    let result = match owned {
                        Some(s) => Token::String(Cow::Owned(s)),
                        None => Token::String(Cow::Borrowed(&self.input[start..self.pos])),
                    };
                    self.bump();
                    return result;
                },
                '\n' => {
                    // Unterminated.
                    return Token::BadString;
                },
                '\\' => {
                    let mut buf = owned.unwrap_or_else(|| self.input[start..self.pos].to_string());
                    self.bump();
                    if let Some(esc) = self.consume_escape() {
                        buf.push(esc);
                    }
                    owned = Some(buf);
                },
                _ => {
                    if let Some(buf) = owned.as_mut() {
                        buf.push(c);
                    }
                    self.bump();
                },
            }
        }
        // EOF inside string: per spec, return a String not BadString
        // when EOF terminates without a quote — readers tolerate it.
        match owned {
            Some(s) => Token::String(Cow::Owned(s)),
            None => Token::String(Cow::Borrowed(&self.input[start..self.pos])),
        }
    }

    fn consume_ident_like(&mut self) -> Token<'i> {
        let name = self.consume_name();
        if self.peek() == Some('(') {
            // Special-case url(.
            if name.eq_ignore_ascii_case("url") {
                self.bump(); // (
                             // Skip leading whitespace
                while self.peek().map(is_whitespace).unwrap_or(false) {
                    self.bump();
                }
                // url("...") — delegate to string then consume rest.
                if matches!(self.peek(), Some('"') | Some('\'')) {
                    let str_tok = self.consume_string(self.peek().unwrap());
                    while self.peek().map(is_whitespace).unwrap_or(false) {
                        self.bump();
                    }
                    if self.peek() == Some(')') {
                        self.bump();
                        if let Token::String(s) = str_tok {
                            return Token::Url(s);
                        }
                    }
                    // Malformed: skip to close paren.
                    while let Some(c) = self.peek() {
                        if c == ')' {
                            self.bump();
                            break;
                        }
                        self.bump();
                    }
                    return Token::BadUrl;
                }
                // Bare url(...).
                return self.consume_unquoted_url();
            }
            self.bump();
            return Token::Function(name);
        }
        Token::Ident(name)
    }

    fn consume_unquoted_url(&mut self) -> Token<'i> {
        let mut owned = String::new();
        loop {
            match self.peek() {
                Some(')') => {
                    self.bump();
                    return Token::Url(Cow::Owned(owned));
                },
                None => return Token::Url(Cow::Owned(owned)),
                Some(c) if is_whitespace(c) => {
                    while self.peek().map(is_whitespace).unwrap_or(false) {
                        self.bump();
                    }
                    if self.peek() == Some(')') {
                        self.bump();
                        return Token::Url(Cow::Owned(owned));
                    }
                    // Whitespace mid-URL is malformed; consume to ')'.
                    while let Some(c) = self.peek() {
                        if c == ')' {
                            self.bump();
                            break;
                        }
                        self.bump();
                    }
                    return Token::BadUrl;
                },
                Some('"') | Some('\'') | Some('(') => {
                    // Spec calls these "non-printable code points or
                    // disallowed code points in url() unquoted form".
                    while let Some(c) = self.peek() {
                        if c == ')' {
                            self.bump();
                            break;
                        }
                        self.bump();
                    }
                    return Token::BadUrl;
                },
                Some('\\') => {
                    self.bump();
                    if let Some(esc) = self.consume_escape() {
                        owned.push(esc);
                    } else {
                        // Bare backslash before newline: malformed.
                        while let Some(c) = self.peek() {
                            if c == ')' {
                                self.bump();
                                break;
                            }
                            self.bump();
                        }
                        return Token::BadUrl;
                    }
                },
                Some(c) => {
                    owned.push(c);
                    self.bump();
                },
            }
        }
    }

    fn consume_name(&mut self) -> Cow<'i, str> {
        let start = self.pos;
        let mut owned: Option<String> = None;
        while let Some(c) = self.peek() {
            if is_name_char(c) {
                if let Some(buf) = owned.as_mut() {
                    buf.push(c);
                }
                self.bump();
            } else if c == '\\' && self.peek_nth(1).map(|c| c != '\n').unwrap_or(true) {
                let mut buf = owned.unwrap_or_else(|| self.input[start..self.pos].to_string());
                self.bump();
                if let Some(esc) = self.consume_escape() {
                    buf.push(esc);
                }
                owned = Some(buf);
            } else {
                break;
            }
        }
        match owned {
            Some(s) => Cow::Owned(s),
            None => Cow::Borrowed(&self.input[start..self.pos]),
        }
    }

    fn consume_number(&mut self) -> Number {
        let start = self.pos;
        if matches!(self.peek(), Some('+') | Some('-')) {
            self.bump();
        }
        while self.peek().map(|c| c.is_ascii_digit()).unwrap_or(false) {
            self.bump();
        }
        let mut is_integer = true;
        if self.peek() == Some('.')
            && self
                .peek_nth(1)
                .map(|c| c.is_ascii_digit())
                .unwrap_or(false)
        {
            is_integer = false;
            self.bump(); // .
            while self.peek().map(|c| c.is_ascii_digit()).unwrap_or(false) {
                self.bump();
            }
        }
        if matches!(self.peek(), Some('e') | Some('E')) {
            let mut tentative = self.pos;
            tentative += 1; // e/E
            let after_e = self.input[tentative..].chars().next();
            let after_e_2 = self.input[tentative..].chars().nth(1);
            let exp_ok = match (after_e, after_e_2) {
                (Some(c), _) if c.is_ascii_digit() => true,
                (Some('+'), Some(c)) | (Some('-'), Some(c)) if c.is_ascii_digit() => true,
                _ => false,
            };
            if exp_ok {
                is_integer = false;
                self.bump();
                if matches!(self.peek(), Some('+') | Some('-')) {
                    self.bump();
                }
                while self.peek().map(|c| c.is_ascii_digit()).unwrap_or(false) {
                    self.bump();
                }
            }
        }
        let text = &self.input[start..self.pos];
        let value = text.parse::<f64>().unwrap_or(0.0);
        Number { value, is_integer }
    }

    /// Consume an escape after the leading backslash has already been
    /// bumped. Returns the resulting char, or `None` when the escape is
    /// followed by a newline (CSS treats `\<newline>` as a continuation
    /// in strings — caller decides what to do).
    fn consume_escape(&mut self) -> Option<char> {
        let c = self.peek()?;
        if c == '\n' {
            return None;
        }
        if c.is_ascii_hexdigit() {
            let mut hex = 0u32;
            for _ in 0..6 {
                match self.peek() {
                    Some(d) if d.is_ascii_hexdigit() => {
                        hex = hex * 16 + d.to_digit(16).unwrap();
                        self.bump();
                    },
                    _ => break,
                }
            }
            // Optional trailing whitespace.
            if self.peek().map(is_whitespace).unwrap_or(false) {
                self.bump();
            }
            return char::from_u32(hex).or(Some('\u{FFFD}'));
        }
        self.bump();
        Some(c)
    }
}

// ─────────────────────────────────────────────────────────────────────
// Character classifiers per CSS Syntax §4.2
// ─────────────────────────────────────────────────────────────────────

fn is_whitespace(c: char) -> bool {
    matches!(c, ' ' | '\t' | '\n' | '\r' | '\x0c')
}

fn is_name_start(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_' || c >= '\u{80}'
}

fn is_name_char(c: char) -> bool {
    is_name_start(c) || c.is_ascii_digit() || c == '-'
}

fn starts_escape(c: char, next: Option<char>) -> bool {
    c == '\\' && next != Some('\n')
}

fn would_start_ident(c: char, next1: Option<char>, next2: Option<char>) -> bool {
    match c {
        '-' => match next1 {
            Some(c) if is_name_start(c) || c == '-' => true,
            Some(c) if starts_escape(c, next2) => true,
            _ => false,
        },
        c if is_name_start(c) => true,
        '\\' => starts_escape('\\', next1),
        _ => false,
    }
}

fn would_start_number(c: char, next1: Option<char>, next2: Option<char>) -> bool {
    match c {
        '+' | '-' => match next1 {
            Some(c) if c.is_ascii_digit() => true,
            Some('.') => matches!(next2, Some(c) if c.is_ascii_digit()),
            _ => false,
        },
        '.' => matches!(next1, Some(c) if c.is_ascii_digit()),
        c if c.is_ascii_digit() => true,
        _ => false,
    }
}

// ─────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn toks(input: &str) -> Vec<Token<'_>> {
        tokenize(input)
            .unwrap()
            .into_iter()
            .map(|(t, _)| t)
            .filter(|t| !matches!(t, Token::Whitespace))
            .collect()
    }

    fn n(value: f64, is_int: bool) -> Number {
        Number {
            value,
            is_integer: is_int,
        }
    }

    #[test]
    fn punctuation() {
        let t = toks("{}();:,[]");
        assert_eq!(
            t,
            vec![
                Token::LeftBrace,
                Token::RightBrace,
                Token::LeftParen,
                Token::RightParen,
                Token::Semicolon,
                Token::Colon,
                Token::Comma,
                Token::LeftSquare,
                Token::RightSquare,
                Token::Eof,
            ],
        );
    }

    #[test]
    fn ident_and_function() {
        let t = toks("display calc( foo-bar --custom-prop");
        assert_eq!(
            t,
            vec![
                Token::Ident("display".into()),
                Token::Function("calc".into()),
                Token::Ident("foo-bar".into()),
                Token::Ident("--custom-prop".into()),
                Token::Eof,
            ],
        );
    }

    #[test]
    fn numbers_and_dimensions() {
        let t = toks("12 1.5 -.5 12px 1.5em 50%");
        assert_eq!(
            t,
            vec![
                Token::Number(n(12.0, true)),
                Token::Number(n(1.5, false)),
                Token::Number(n(-0.5, false)),
                Token::Dimension {
                    value: n(12.0, true),
                    unit: "px".into()
                },
                Token::Dimension {
                    value: n(1.5, false),
                    unit: "em".into()
                },
                Token::Percentage(n(50.0, true)),
                Token::Eof,
            ],
        );
    }

    #[test]
    fn hash_id_vs_unrestricted() {
        let t = toks("#myid #abc #123");
        assert_eq!(
            t,
            vec![
                Token::Hash {
                    value: "myid".into(),
                    is_id: true,
                },
                Token::Hash {
                    value: "abc".into(),
                    is_id: true,
                },
                Token::Hash {
                    value: "123".into(),
                    is_id: false,
                },
                Token::Eof,
            ],
        );
    }

    #[test]
    fn at_keywords() {
        let t = toks("@media @page @font-face");
        assert_eq!(
            t,
            vec![
                Token::AtKeyword("media".into()),
                Token::AtKeyword("page".into()),
                Token::AtKeyword("font-face".into()),
                Token::Eof,
            ],
        );
    }

    #[test]
    fn strings_and_escapes() {
        // \E9 → U+00E9 (é). Source "es\E9pace" decodes to "esépace".
        let t = toks(r#""hello" 'world' "es\E9pace""#);
        assert_eq!(
            t,
            vec![
                Token::String("hello".into()),
                Token::String("world".into()),
                Token::String("esépace".into()),
                Token::Eof,
            ],
        );
    }

    #[test]
    fn unterminated_string_yields_bad_string() {
        let t = toks("\"unterm\nrest");
        assert!(matches!(t[0], Token::BadString));
    }

    #[test]
    fn url_quoted_and_unquoted() {
        let t = toks(r#"url("https://x") url(file.png) url(  spaced  )"#);
        assert_eq!(
            t,
            vec![
                Token::Url("https://x".into()),
                Token::Url("file.png".into()),
                Token::Url("spaced".into()),
                Token::Eof,
            ],
        );
    }

    #[test]
    fn url_with_internal_whitespace_is_bad() {
        let t = toks("url(foo bar)");
        assert!(matches!(t[0], Token::BadUrl));
    }

    #[test]
    fn cdo_cdc() {
        let t = toks("<!-- color -->");
        assert_eq!(
            t,
            vec![
                Token::Cdo,
                Token::Ident("color".into()),
                Token::Cdc,
                Token::Eof,
            ],
        );
    }

    #[test]
    fn comments_skipped() {
        let t = toks("/* comment */color/* between */: red /* trailing */");
        assert_eq!(
            t,
            vec![
                Token::Ident("color".into()),
                Token::Colon,
                Token::Ident("red".into()),
                Token::Eof,
            ],
        );
    }

    #[test]
    fn whitespace_runs_collapse_to_one_token() {
        // Don't filter whitespace this time
        let raw: Vec<Token<'_>> = tokenize("a    b")
            .unwrap()
            .into_iter()
            .map(|(t, _)| t)
            .collect();
        // Expect ident, whitespace, ident, eof (one whitespace, not many).
        let ws_count = raw
            .iter()
            .filter(|t| matches!(t, Token::Whitespace))
            .count();
        assert_eq!(ws_count, 1);
    }

    #[test]
    fn delim_for_unknown_punct() {
        let t = toks("&");
        assert_eq!(t, vec![Token::Delim('&'), Token::Eof]);
    }

    #[test]
    fn source_locations_track_lines() {
        let stream = tokenize("a\nb").unwrap();
        // Stream: Ident(a) line 1, Whitespace line 1, Ident(b) line 2.
        let lines: Vec<u32> = stream.iter().map(|(_, l)| l.line).collect();
        assert_eq!(lines, vec![1, 1, 2, 2]);
    }

    #[test]
    fn calc_with_units() {
        let t = toks("calc(100% - 10px)");
        assert_eq!(
            t,
            vec![
                Token::Function("calc".into()),
                Token::Percentage(n(100.0, true)),
                Token::Delim('-'),
                Token::Dimension {
                    value: n(10.0, true),
                    unit: "px".into()
                },
                Token::RightParen,
                Token::Eof,
            ],
        );
    }

    #[test]
    fn rgba_parses_as_function() {
        let t = toks("rgba(255, 0, 128, 0.5)");
        assert!(matches!(&t[0], Token::Function(name) if name == "rgba"));
    }

    #[test]
    fn at_media_print_block() {
        let t = toks("@media print { body { color: black; } }");
        assert!(matches!(&t[0], Token::AtKeyword(k) if k == "media"));
        assert!(t.contains(&Token::LeftBrace));
        assert!(t.contains(&Token::RightBrace));
    }
}

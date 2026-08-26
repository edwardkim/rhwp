//! DSEL 렉서 — 최장 일치, 문자 오프셋, 이스케이프 해제.
//!
//! ## 문자 단위로 도는 이유
//!
//! 선택자에는 한글이 그대로 들어온다 (`para[style="개요 1"]`, `field[name="수급자성명"]`).
//! 바이트 인덱스로 돌면 오프셋이 글자 경계와 어긋나 캐럿이 글자 중간을 가리키고,
//! 슬라이싱은 `byte index is not a char boundary` 로 패닉한다. 그래서 입력을 한 번
//! `Vec<char>` 로 펼치고 전 구간을 문자 인덱스로 다룬다. 선택자는 길어야 수백 자라
//! 이 사본의 비용은 무시할 수 있고, 얻는 것은 "패닉 가능 경로가 없다"는 성질이다.
//!
//! ## 최장 일치를 손으로 적는 이유
//!
//! 기호가 열 몇 개뿐이고 접두 충돌이 `>`·`>=`, `*`·`*=`, `~`·`~=` 세 쌍뿐이다.
//! 표를 만들어 일반화하면 표를 읽어야 규칙을 알 수 있게 되는데, 규칙이 세 줄이면
//! 세 줄로 적는 편이 감사 가능하다.

use super::error::SelectorError;
use super::token::{Tok, Token};

/// 렉싱 결과 — 토큰 열과 입력 문자 길이(EOF 오프셋 계산용).
#[derive(Debug, Clone)]
pub struct Lexed {
    pub tokens: Vec<Token>,
    /// 입력의 문자 개수. 파서가 "입력 끝"의 오프셋으로 쓴다.
    pub char_len: usize,
}

/// 선택자 문자열을 토큰 열로 쪼갠다.
///
/// 공백은 [`Tok::Ws`] 로 **남는다**(합쳐서 한 토큰). 버리지 않는 이유는
/// `token` 모듈 문서에 적었다.
pub fn lex(input: &str) -> Result<Lexed, SelectorError> {
    let chars: Vec<char> = input.chars().collect();
    let mut tokens = Vec::new();
    let mut i = 0usize;

    while i < chars.len() {
        let start = i;
        let c = chars[i];

        // 공백 덩어리 — 여러 칸이어도 결합자 하나다.
        if c.is_whitespace() {
            while i < chars.len() && chars[i].is_whitespace() {
                i += 1;
            }
            tokens.push(Token::new(Tok::Ws, start));
            continue;
        }

        // 따옴표 문자열.
        if c == '"' || c == '\'' {
            let (value, next) = lex_string(&chars, i, c)?;
            tokens.push(Token::new(Tok::Str(value), start));
            i = next;
            continue;
        }

        // 숫자. 부호는 여기서 먹는다 — `:nth(-1)` 을 파서가 단항 마이너스로
        // 처리하게 하면 `[level>-1]` 같은 자리에서 `>-` 를 연산자로 오인할 여지가
        // 생긴다. 부호 있는 정수를 하나의 토큰으로 확정하는 편이 문법이 단순하다.
        if c.is_ascii_digit()
            || (c == '-' && matches!(chars.get(i + 1), Some(d) if d.is_ascii_digit()))
        {
            let (value, next) = lex_int(&chars, i)?;
            tokens.push(Token::new(Tok::Int(value), start));
            i = next;
            continue;
        }

        // 식별자 — 유니코드 문자로 시작, 이어서 문자·숫자·`_`·`-`.
        //
        // `-` 를 이어붙이는 것이 `a-1` 을 `a`,`-1` 로 읽지 않게 만든다. 뺄셈이
        // 없는 언어라 이 선택에 모호함이 없다.
        if is_ident_start(c) {
            let mut end = i + 1;
            while end < chars.len() && is_ident_continue(chars[end]) {
                end += 1;
            }
            let text: String = chars[i..end].iter().collect();
            tokens.push(Token::new(Tok::Ident(text), start));
            i = end;
            continue;
        }

        // 기호 — 두 글자 먼저, 그 다음 한 글자.
        let two = if i + 1 < chars.len() {
            Some((c, chars[i + 1]))
        } else {
            None
        };
        let (tok, width) = match two {
            Some(('>', '=')) => (Tok::Ge, 2),
            Some(('<', '=')) => (Tok::Le, 2),
            Some(('!', '=')) => (Tok::Ne, 2),
            Some(('^', '=')) => (Tok::Prefix, 2),
            Some(('$', '=')) => (Tok::Suffix, 2),
            Some(('*', '=')) => (Tok::Substr, 2),
            Some(('~', '=')) => (Tok::Glob, 2),
            Some(('.', '.')) => (Tok::DotDot, 2),
            _ => match c {
                '*' => (Tok::Star, 1),
                '[' => (Tok::LBracket, 1),
                ']' => (Tok::RBracket, 1),
                '(' => (Tok::LParen, 1),
                ')' => (Tok::RParen, 1),
                ':' => (Tok::Colon, 1),
                ',' => (Tok::Comma, 1),
                '>' => (Tok::Gt, 1),
                '<' => (Tok::Lt, 1),
                '=' => (Tok::Eq, 1),
                '+' => (Tok::Plus, 1),
                '~' => (Tok::Tilde, 1),
                '!' => {
                    return Err(SelectorError::lex(start, "`!` 뒤에는 `=` 만 올 수 있다")
                        .expecting(["!="])
                        .hinting("부정은 `:not(...)` 으로 쓴다"));
                }
                '.' => {
                    return Err(SelectorError::lex(start, "`.` 하나는 뜻이 없다")
                        .expecting([".."])
                        .hinting("범위는 `:nth(1..3)`, 자손은 공백으로 쓴다"));
                }
                '#' => {
                    return Err(SelectorError::lex(start, "`#` 문법은 없다")
                        .hinting("이름 지목은 `[name=\"…\"]` 으로 쓴다"));
                }
                other => {
                    return Err(SelectorError::lex(
                        start,
                        format!("선택자에 쓸 수 없는 문자 `{other}`"),
                    ));
                }
            },
        };
        tokens.push(Token::new(tok, start));
        i += width;
    }

    Ok(Lexed {
        tokens,
        char_len: chars.len(),
    })
}

/// 식별자 첫 글자로 쓸 수 있나.
///
/// `is_alphabetic` 이라 한글·한자도 통과한다. 축 이름은 ASCII 로만 정의돼 있지만
/// 렉서가 미리 막지는 않는다 — 막으면 오류가 "쓸 수 없는 문자"로 나와서 정작
/// "그런 축은 없다"는 진짜 원인을 가린다. 미지의 축은 해석 단계에서 후보와 함께
/// 거절하는 편이 훨씬 쓸모 있다.
fn is_ident_start(c: char) -> bool {
    c.is_alphabetic() || c == '_'
}

/// 식별자 이어짐.
fn is_ident_continue(c: char) -> bool {
    c.is_alphanumeric() || c == '_' || c == '-'
}

/// 정수 하나를 읽는다.
///
/// 오버플로를 `i64` 범위에서 **명시적으로** 거절한다. `parse::<i64>()` 의 오류를
/// 그대로 흘리면 메시지가 영어 `number too large to fit in target type` 로 나가
/// 봉투의 한국어 진단 규약과 어긋난다.
fn lex_int(chars: &[char], start: usize) -> Result<(i64, usize), SelectorError> {
    let mut i = start;
    if chars[i] == '-' {
        i += 1;
    }
    let digits_from = i;
    while i < chars.len() && chars[i].is_ascii_digit() {
        i += 1;
    }
    debug_assert!(i > digits_from, "호출부가 첫 숫자를 확인하고 부른다");
    let text: String = chars[start..i].iter().collect();
    let value = text
        .parse::<i64>()
        .map_err(|_| SelectorError::lex(start, format!("정수 범위를 벗어난 값 `{text}`")))?;
    Ok((value, i))
}

/// 따옴표 문자열 하나를 읽고 이스케이프를 푼다.
///
/// 지원 이스케이프는 `\\ \" \' \n \t \r \0` 와 `\uXXXX` 뿐이다. 목록을 좁게 두는
/// 이유는 선택자가 감사 대상 문자열이기 때문이다 — 8진·16진 바이트 이스케이프를
/// 열어 두면 같은 선택자를 여러 방식으로 적을 수 있고, 그러면 "이 선택자와 저
/// 선택자가 같은가"를 문자열 비교로 판정할 수 없다.
fn lex_string(chars: &[char], start: usize, quote: char) -> Result<(String, usize), SelectorError> {
    let mut out = String::new();
    let mut i = start + 1;

    while i < chars.len() {
        let c = chars[i];
        if c == quote {
            return Ok((out, i + 1));
        }
        if c != '\\' {
            out.push(c);
            i += 1;
            continue;
        }

        // 이스케이프.
        let esc = chars.get(i + 1).copied().ok_or_else(|| {
            SelectorError::lex(i, "역슬래시로 끝나는 문자열")
                .hinting("역슬래시 자체는 `\\\\` 로 적는다")
        })?;
        match esc {
            '\\' => out.push('\\'),
            '"' => out.push('"'),
            '\'' => out.push('\''),
            'n' => out.push('\n'),
            't' => out.push('\t'),
            'r' => out.push('\r'),
            '0' => out.push('\0'),
            'u' => {
                let (ch, next) = lex_unicode_escape(chars, i)?;
                out.push(ch);
                i = next;
                continue;
            }
            other => {
                return Err(
                    SelectorError::lex(i, format!("알 수 없는 이스케이프 `\\{other}`"))
                        .expecting(["\\\\", "\\\"", "\\'", "\\n", "\\t", "\\r", "\\0", "\\uXXXX"]),
                );
            }
        }
        i += 2;
    }

    Err(
        SelectorError::lex(start, format!("닫히지 않은 문자열 (`{quote}` 로 시작)"))
            .hinting("문자열 안의 따옴표는 역슬래시로 감싼다"),
    )
}

/// `\uXXXX` 를 읽는다. 서로게이트 쌍은 `😀` 형태로 이어 붙인다.
///
/// 서로게이트를 받는 이유: JSON 문자열에서 그대로 옮겨 온 선택자가 BMP 밖 문자를
/// 그 형태로 싣는다. 앞쪽 서로게이트만 오면 유효한 `char` 가 아니므로 여기서
/// 거절해야 하고, 거절하지 않으면 `char::from_u32` 가 `None` 을 주는 자리에서
/// 원인을 알 수 없는 오류가 난다.
fn lex_unicode_escape(chars: &[char], at: usize) -> Result<(char, usize), SelectorError> {
    let first = read_hex4(chars, at)?;
    let mut next = at + 6; // `\uXXXX`

    if (0xD800..0xDC00).contains(&first) {
        // 앞쪽 서로게이트 — 뒤쪽이 반드시 따라와야 한다.
        let has_pair = chars.get(next) == Some(&'\\') && chars.get(next + 1) == Some(&'u');
        if !has_pair {
            return Err(SelectorError::lex(at, "짝 없는 상위 서로게이트")
                .hinting("BMP 밖 문자는 `\\uD83D\\uDE00` 처럼 두 개를 이어 적는다"));
        }
        let second = read_hex4(chars, next)?;
        if !(0xDC00..0xE000).contains(&second) {
            return Err(SelectorError::lex(next, "하위 서로게이트가 아니다"));
        }
        let combined = 0x1_0000 + ((first - 0xD800) << 10) + (second - 0xDC00);
        next += 6;
        let ch = char::from_u32(combined)
            .ok_or_else(|| SelectorError::lex(at, "유효하지 않은 코드포인트"))?;
        return Ok((ch, next));
    }

    if (0xDC00..0xE000).contains(&first) {
        return Err(SelectorError::lex(at, "짝 없는 하위 서로게이트"));
    }

    let ch =
        char::from_u32(first).ok_or_else(|| SelectorError::lex(at, "유효하지 않은 코드포인트"))?;
    Ok((ch, next))
}

/// `at` 위치의 `\u` 다음 네 자리 16진수를 읽는다.
fn read_hex4(chars: &[char], at: usize) -> Result<u32, SelectorError> {
    let mut value = 0u32;
    for k in 0..4 {
        let c = chars.get(at + 2 + k).copied().ok_or_else(|| {
            SelectorError::lex(at, "`\\u` 뒤 16진수 네 자리가 모자라다").expecting(["\\uXXXX"])
        })?;
        let digit = c.to_digit(16).ok_or_else(|| {
            SelectorError::lex(at + 2 + k, format!("16진수가 아닌 문자 `{c}`"))
                .expecting(["0-9", "a-f", "A-F"])
        })?;
        value = value * 16 + digit;
    }
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(input: &str) -> Vec<Tok> {
        lex(input)
            .unwrap()
            .tokens
            .into_iter()
            .map(|t| t.tok)
            .collect()
    }

    #[test]
    fn whitespace_survives_as_a_single_token() {
        // 자손 결합자가 공백이므로 공백이 사라지면 문법이 무너진다.
        assert_eq!(
            kinds("a   b"),
            vec![Tok::Ident("a".into()), Tok::Ws, Tok::Ident("b".into())]
        );
    }

    #[test]
    fn longest_match_wins_on_operator_prefixes() {
        assert_eq!(kinds(">="), vec![Tok::Ge]);
        assert_eq!(kinds(">"), vec![Tok::Gt]);
        assert_eq!(kinds("*="), vec![Tok::Substr]);
        assert_eq!(kinds("*"), vec![Tok::Star]);
        assert_eq!(kinds("~="), vec![Tok::Glob]);
        assert_eq!(kinds("~"), vec![Tok::Tilde]);
        assert_eq!(kinds(".."), vec![Tok::DotDot]);
    }

    #[test]
    fn korean_identifiers_and_strings_keep_char_offsets() {
        let out = lex("표[제목=\"합계\"]").unwrap();
        // 첫 토큰은 0, `[` 는 문자 1 — 바이트로 세면 3 이 되어 캐럿이 어긋난다.
        assert_eq!(out.tokens[0].offset, 0);
        assert_eq!(out.tokens[1].offset, 1);
        assert_eq!(out.tokens[1].tok, Tok::LBracket);
    }

    #[test]
    fn negative_int_is_one_token() {
        assert_eq!(kinds("-1"), vec![Tok::Int(-1)]);
        // 비교 연산자 뒤의 음수도 갈라지지 않는다.
        assert_eq!(kinds(">-1"), vec![Tok::Gt, Tok::Int(-1)]);
    }

    #[test]
    fn hyphen_binds_into_identifiers() {
        assert_eq!(kinds("page-break"), vec![Tok::Ident("page-break".into())]);
    }

    #[test]
    fn string_escapes_are_unescaped_once() {
        assert_eq!(kinds(r#""a\"b""#), vec![Tok::Str("a\"b".into())]);
        assert_eq!(kinds(r#""tab\there""#), vec![Tok::Str("tab\there".into())]);
        assert_eq!(kinds(r#""A""#), vec![Tok::Str("A".into())]);
    }

    #[test]
    fn surrogate_pairs_combine() {
        assert_eq!(kinds(r#""😀""#), vec![Tok::Str("😀".into())]);
    }

    #[test]
    fn lone_surrogate_is_rejected_with_a_hint() {
        let err = lex(r#""\uD83D""#).unwrap_err();
        assert!(err.message.contains("서로게이트"));
        assert!(err.hint.is_some());
    }

    #[test]
    fn unterminated_string_points_at_the_opening_quote() {
        let err = lex("para[style=\"열림").unwrap_err();
        // 시작 따옴표 위치를 가리켜야 어디부터 닫아야 할지 알 수 있다.
        assert_eq!(err.offset, 11);
    }

    #[test]
    fn unknown_escape_lists_the_allowed_set() {
        let err = lex(r#""\q""#).unwrap_err();
        assert!(err.expected.iter().any(|e| e == "\\uXXXX"));
    }

    #[test]
    fn hash_and_dot_redirect_to_real_syntax() {
        let hash = lex("#T1").unwrap_err();
        assert!(hash.hint.unwrap().contains("[name="));
        let dot = lex("a.b").unwrap_err();
        assert!(dot.hint.unwrap().contains(":nth"));
    }

    #[test]
    fn bang_alone_points_at_not() {
        let err = lex("a!b").unwrap_err();
        assert!(err.hint.unwrap().contains(":not"));
    }

    #[test]
    fn int_overflow_is_a_korean_diagnostic() {
        let err = lex("99999999999999999999").unwrap_err();
        assert!(err.message.contains("정수 범위"));
    }

    #[test]
    fn char_len_is_reported_for_eof_offsets() {
        // 한글 3자 = 바이트 9. 파서가 EOF 오프셋으로 쓸 값은 3 이어야 한다.
        assert_eq!(lex("문단표").unwrap().char_len, 3);
    }
}

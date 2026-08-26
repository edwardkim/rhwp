//! DSEL 토큰 정의.
//!
//! ## 왜 공백이 토큰인가
//!
//! DSEL 은 CSS 처럼 **공백 자체가 결합자**다 (`table cell` = 표 아래 어딘가의 셀,
//! `table > cell` = 표의 직계 셀). 렉서가 공백을 버리면 이 둘을 구분할 방법이
//! 사라진다. 그래서 공백은 [`Tok::Ws`] 로 남기고, 의미가 없는 자리(괄호 안,
//! 결합자 주변)에서 파서가 **명시적으로** 흘려보낸다. "렉서가 공백을 지운다"는
//! 흔한 기본값이 여기서는 문법 파괴다.
//!
//! ## 비교 연산자를 렉서가 아는 이유
//!
//! `>` 는 대괄호 밖에서는 직계 결합자, 안에서는 비교 연산자다. 렉싱을 문맥에
//! 의존시키면(대괄호 깊이를 렉서가 세면) 렉서와 파서가 같은 상태를 두 벌 갖게
//! 되고, 두 벌은 언젠가 어긋난다. 그래서 렉서는 **최장 일치로 기호만 확정**하고
//! (`>=` 는 한 토큰, `>` 는 한 토큰), 그게 결합자인지 비교자인지는 파서가 정한다.

use std::fmt;

/// 토큰 한 개 — 종류와 입력에서의 문자 오프셋.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    /// 토큰 종류와 실린 값.
    pub tok: Tok,
    /// 토큰이 시작한 **문자** 오프셋(바이트 아님 — `error` 모듈과 같은 규약).
    pub offset: usize,
}

impl Token {
    pub fn new(tok: Tok, offset: usize) -> Self {
        Token { tok, offset }
    }
}

/// 토큰 종류.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Tok {
    /// 식별자 — 축 이름·속성 이름·의사 이름·`last` 같은 문맥 키워드.
    ///
    /// 키워드를 따로 토큰화하지 않는 이유: `last` 는 `:nth(last)` 에서는 키워드지만
    /// `para[style="last"]` 에서는 그냥 값이고, `[name=last]` 에서는 따옴표 없는
    /// 값이다. 렉서가 키워드로 승격시키면 파서는 매번 도로 식별자로 강등해야 한다.
    Ident(String),
    /// 따옴표 문자열 — 이스케이프가 이미 풀린 값.
    Str(String),
    /// 정수. 음수 허용(`:nth(-1)` 은 뒤에서 첫째).
    Int(i64),
    /// `*` — 전체 축 와일드카드.
    Star,
    /// `[`
    LBracket,
    /// `]`
    RBracket,
    /// `(`
    LParen,
    /// `)`
    RParen,
    /// `:`
    Colon,
    /// `,`
    Comma,
    /// `..` — 범위.
    DotDot,
    /// `>` — 직계 결합자 또는 초과 비교.
    Gt,
    /// `<` — 미만 비교(결합자 아님).
    Lt,
    /// `>=`
    Ge,
    /// `<=`
    Le,
    /// `=`
    Eq,
    /// `!=`
    Ne,
    /// `^=` — 접두 일치.
    Prefix,
    /// `$=` — 접미 일치.
    Suffix,
    /// `*=` — 부분 일치.
    Substr,
    /// `~=` — 글롭 일치.
    Glob,
    /// `+` — 다음 형제 결합자.
    Plus,
    /// `~` — 이후 형제 결합자.
    Tilde,
    /// 공백 한 덩어리 — 자손 결합자 후보.
    Ws,
}

impl Tok {
    /// 오류 메시지에 쓸 표시 이름.
    ///
    /// 값을 실은 토큰은 값까지 보여 준다 — `기대: ]` 옆에 `실제: para` 가 붙어야
    /// 무엇을 지웠는지 알 수 있다.
    pub fn describe(&self) -> String {
        match self {
            Tok::Ident(s) => format!("식별자 `{s}`"),
            Tok::Str(s) => format!("문자열 \"{s}\""),
            Tok::Int(n) => format!("정수 {n}"),
            other => format!("`{}`", other.symbol()),
        }
    }

    /// 기호 토큰의 원문. 값 토큰은 종류 이름을 돌려준다.
    pub fn symbol(&self) -> &'static str {
        match self {
            Tok::Ident(_) => "식별자",
            Tok::Str(_) => "문자열",
            Tok::Int(_) => "정수",
            Tok::Star => "*",
            Tok::LBracket => "[",
            Tok::RBracket => "]",
            Tok::LParen => "(",
            Tok::RParen => ")",
            Tok::Colon => ":",
            Tok::Comma => ",",
            Tok::DotDot => "..",
            Tok::Gt => ">",
            Tok::Lt => "<",
            Tok::Ge => ">=",
            Tok::Le => "<=",
            Tok::Eq => "=",
            Tok::Ne => "!=",
            Tok::Prefix => "^=",
            Tok::Suffix => "$=",
            Tok::Substr => "*=",
            Tok::Glob => "~=",
            Tok::Plus => "+",
            Tok::Tilde => "~",
            Tok::Ws => "공백",
        }
    }

    /// 이 토큰이 속성 비교 연산자인가.
    ///
    /// `Star`(`*`)와 `Tilde`(`~`)가 여기 없는 것이 중요하다 — 둘은 각각 와일드카드·
    /// 형제 결합자이고, 비교자로 쓰이는 형태는 `*=`·`~=` 라는 **다른 토큰**이다.
    /// 최장 일치 렉싱이 그 구분을 이미 끝내 두었으므로 여기서 되짚을 필요가 없다.
    pub fn is_compare_op(&self) -> bool {
        matches!(
            self,
            Tok::Eq
                | Tok::Ne
                | Tok::Gt
                | Tok::Lt
                | Tok::Ge
                | Tok::Le
                | Tok::Prefix
                | Tok::Suffix
                | Tok::Substr
                | Tok::Glob
        )
    }
}

impl fmt::Display for Tok {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.describe())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compare_ops_exclude_bare_star_and_tilde() {
        assert!(Tok::Substr.is_compare_op());
        assert!(Tok::Glob.is_compare_op());
        assert!(!Tok::Star.is_compare_op());
        assert!(!Tok::Tilde.is_compare_op());
    }

    #[test]
    fn describe_shows_the_value_not_just_the_kind() {
        assert_eq!(Tok::Ident("para".into()).describe(), "식별자 `para`");
        assert_eq!(Tok::Int(-1).describe(), "정수 -1");
        assert_eq!(Tok::Ge.describe(), "`>=`");
    }
}

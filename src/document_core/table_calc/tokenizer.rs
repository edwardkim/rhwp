//! 계산식 토크나이저: 문자열 → 토큰 스트림

/// 와일드카드 행을 나타내는 내부 센티널 값. 실제 행 번호(1부터 시작)와
/// 절대 충돌하지 않도록 0이 아닌 u32::MAX를 사용한다.
pub const WILDCARD_ROW: u32 = u32::MAX;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    /// 숫자 리터럴
    Number(f64),
    /// 셀 참조 (column_name, row_num) — 예: ("A", 1), ("AA", 3), ("?", 3)
    CellRef(String, u32),
    /// 함수 이름 (대문자)
    Function(String),
    /// 방향 지정자
    Direction(DirectionKind),
    /// 연산자
    Plus,
    Minus,
    Star,
    Slash,
    /// 구분자
    LParen,
    RParen,
    Comma,
    Colon,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DirectionKind {
    Left,
    Right,
    Above,
    Below,
}

/// 계산식 문자열을 토큰 스트림으로 변환한다.
/// 선행 '=' 또는 '@'는 제거한다.
pub fn tokenize(input: &str) -> Vec<Token> {
    let s = input.trim();
    let s = if s.starts_with('=') || s.starts_with('@') {
        &s[1..]
    } else {
        s
    };

    let chars: Vec<char> = s.chars().collect();
    let len = chars.len();
    let mut tokens = Vec::new();
    let mut i = 0;

    while i < len {
        let ch = chars[i];

        // 공백 건너뛰기
        if ch.is_whitespace() {
            i += 1;
            continue;
        }

        // 숫자 (정수 또는 소수)
        if ch.is_ascii_digit() || (ch == '.' && i + 1 < len && chars[i + 1].is_ascii_digit()) {
            let start = i;
            while i < len && (chars[i].is_ascii_digit() || chars[i] == '.') {
                i += 1;
            }
            let num_str: String = chars[start..i].iter().collect();
            if let Ok(n) = num_str.parse::<f64>() {
                tokens.push(Token::Number(n));
            }
            continue;
        }

        // 알파벳 또는 '?': 셀 참조, 함수, 방향 지정자
        if ch.is_ascii_alphabetic() || ch == '?' {
            let start = i;
            while i < len
                && (chars[i].is_ascii_alphanumeric() || chars[i] == '?' || chars[i] == '_')
            {
                i += 1;
            }
            let word: String = chars[start..i].iter().collect();
            let upper = word.to_uppercase();

            // 방향 지정자
            match upper.as_str() {
                "LEFT" => {
                    tokens.push(Token::Direction(DirectionKind::Left));
                    continue;
                }
                "RIGHT" => {
                    tokens.push(Token::Direction(DirectionKind::Right));
                    continue;
                }
                "ABOVE" => {
                    tokens.push(Token::Direction(DirectionKind::Above));
                    continue;
                }
                "BELOW" => {
                    tokens.push(Token::Direction(DirectionKind::Below));
                    continue;
                }
                _ => {}
            }

            // 셀 참조: 한 글자 이상 열(A-Z, AA...) 또는 와일드카드 `?` + 행 숫자/`?`.
            // 뒤에 `(`가 오면 LOG10(...) 같은 함수 이름이므로 셀 참조로 오인하지 않는다.
            if i >= len || chars[i] != '(' {
                let col_len = if upper.starts_with('?') {
                    1
                } else {
                    upper
                        .chars()
                        .take_while(|c| c.is_ascii_alphabetic())
                        .count()
                };
                if col_len > 0 && col_len < upper.len() {
                    let col: String = upper.chars().take(col_len).collect();
                    let rest: String = upper.chars().skip(col_len).collect();
                    let valid_col = col == "?" || col.chars().all(|c| c.is_ascii_alphabetic());
                    if valid_col && (rest.chars().all(|c| c.is_ascii_digit()) || rest == "?") {
                        // 행은 1부터 시작한다 (mydocs/plans/archives/task_370.md).
                        // 명시적 0행은 와일드카드와 혼동하지 않고 함수 이름 경로로 넘긴다.
                        let row = if rest == "?" {
                            Some(WILDCARD_ROW)
                        } else {
                            match rest.parse::<u32>() {
                                Ok(0) => None,
                                Ok(n) => Some(n),
                                Err(_) => None,
                            }
                        };
                        if let Some(row) = row {
                            tokens.push(Token::CellRef(col, row));
                            continue;
                        }
                    }
                }
            }

            // 단일 문자 셀 참조 + 뒤에 숫자가 오는 패턴 확인
            // 이미 단어에 포함됨 (예: A1)

            // 함수 이름 (다음 문자가 '(' 인지 확인)
            tokens.push(Token::Function(upper));
            continue;
        }

        // 연산자 및 구분자
        match ch {
            '+' => tokens.push(Token::Plus),
            '-' => tokens.push(Token::Minus),
            '*' => tokens.push(Token::Star),
            '/' => tokens.push(Token::Slash),
            '(' => tokens.push(Token::LParen),
            ')' => tokens.push(Token::RParen),
            ',' => tokens.push(Token::Comma),
            ':' => tokens.push(Token::Colon),
            _ => {} // 알 수 없는 문자 무시
        }
        i += 1;
    }

    tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_number() {
        let tokens = tokenize("=123");
        assert_eq!(tokens, vec![Token::Number(123.0)]);
    }

    #[test]
    fn test_cell_ref() {
        let tokens = tokenize("=A1+B3");
        assert_eq!(
            tokens,
            vec![
                Token::CellRef("A".into(), 1),
                Token::Plus,
                Token::CellRef("B".into(), 3),
            ]
        );
    }

    #[test]
    fn test_function_call() {
        let tokens = tokenize("=SUM(A1:B5)");
        assert_eq!(
            tokens,
            vec![
                Token::Function("SUM".into()),
                Token::LParen,
                Token::CellRef("A".into(), 1),
                Token::Colon,
                Token::CellRef("B".into(), 5),
                Token::RParen,
            ]
        );
    }

    #[test]
    fn test_direction() {
        let tokens = tokenize("=sum(left)");
        assert_eq!(
            tokens,
            vec![
                Token::Function("SUM".into()),
                Token::LParen,
                Token::Direction(DirectionKind::Left),
                Token::RParen,
            ]
        );
    }

    #[test]
    fn test_complex_formula() {
        let tokens = tokenize("=a1+(b3-3)*2+sum(a1:b5,avg(c3,e5-3))");
        assert!(tokens.len() > 10);
        assert_eq!(tokens[0], Token::CellRef("A".into(), 1));
        assert_eq!(tokens[1], Token::Plus);
    }

    #[test]
    fn test_wildcard() {
        let tokens = tokenize("=SUM(?1:?3)");
        assert_eq!(
            tokens,
            vec![
                Token::Function("SUM".into()),
                Token::LParen,
                Token::CellRef("?".into(), 1),
                Token::Colon,
                Token::CellRef("?".into(), 3),
                Token::RParen,
            ]
        );
    }

    #[test]
    fn test_at_prefix() {
        let tokens = tokenize("@SUM(A1:A5)");
        assert_eq!(tokens[0], Token::Function("SUM".into()));
    }
}

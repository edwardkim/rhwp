//! 계산식 토크나이저: 문자열 → 토큰 스트림

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    /// 숫자 리터럴
    Number(f64),
    /// 셀 참조 (col_char, row_num) — 예: ('A', 1), ('?', 3)
    CellRef(char, u32),
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

            // 셀 참조: 1~2글자 열(A-Z/a-z 또는 ?) + 숫자(행) 또는 ?
            // 예: A1, b3, ?1, A?, ??
            if upper.len() >= 2 {
                let first = upper.chars().next().unwrap();
                let rest: String = upper[first.len_utf8()..].to_string();
                if (first.is_ascii_alphabetic() || first == '?')
                    && (rest.chars().all(|c| c.is_ascii_digit()) || rest == "?")
                {
                    let row = if rest == "?" {
                        0 // 와일드카드 행 (0으로 표시)
                    } else {
                        rest.parse::<u32>().unwrap_or(0)
                    };
                    let col_char = if first == '?' { '?' } else { first };
                    tokens.push(Token::CellRef(col_char, row));
                    continue;
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
            vec![Token::CellRef('A', 1), Token::Plus, Token::CellRef('B', 3),]
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
                Token::CellRef('A', 1),
                Token::Colon,
                Token::CellRef('B', 5),
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
        assert_eq!(tokens[0], Token::CellRef('A', 1));
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
                Token::CellRef('?', 1),
                Token::Colon,
                Token::CellRef('?', 3),
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

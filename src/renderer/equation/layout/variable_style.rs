//! 수학 변수의 글리프 선택. 원본 AST는 보존하고 표시 문자열만 투영한다.
//!
//! MathML의 수학 알파벳 매핑을 따른다. 수학 face의 이탤릭 글리프에
//! 일반 글꼴의 합성 기울임을 다시 적용하지 않도록 출력 경로가 같은 판정을 쓴다.

pub(super) fn display_text(text: &str, italic: bool, bold: bool) -> String {
    if !italic
        || text.is_empty()
        || !text
            .chars()
            .all(|ch| ch.is_ascii_alphabetic() || is_greek_variable(ch))
    {
        return text.to_owned();
    }
    text.chars()
        .map(|ch| {
            // Unicode 수학 알파벳의 그리스 변수도 실제 이탤릭 글리프를 선택한다.
            // 정체(rm)는 위에서 반환하며 논리 전진폭은 바꾸지 않는다.
            if is_greek_variable(ch) {
                let (start, first) = match (bold, ch.is_uppercase()) {
                    (false, true) => (0x1D6E2, '\u{0391}'),
                    (false, false) => (0x1D6FC, '\u{03B1}'),
                    (true, true) => (0x1D71C, '\u{0391}'),
                    (true, false) => (0x1D736, '\u{03B1}'),
                };
                return char::from_u32(start + u32::from(ch) - u32::from(first))
                    .expect("그리스 수학 알파벳 매핑은 유효한 Unicode scalar다");
            }
            let code = match (bold, ch.is_ascii_uppercase()) {
                (false, true) => 0x1D434 + u32::from(ch) - u32::from('A'),
                (false, false) if ch == 'h' => 0x210E,
                (false, false) => 0x1D44E + u32::from(ch) - u32::from('a'),
                (true, true) => 0x1D468 + u32::from(ch) - u32::from('A'),
                (true, false) => 0x1D482 + u32::from(ch) - u32::from('a'),
            };
            char::from_u32(code).expect("수학 알파벳 매핑은 유효한 Unicode scalar다")
        })
        .collect()
}

pub(super) fn is_greek_variable(ch: char) -> bool {
    matches!(ch, '\u{0391}'..='\u{03A1}' | '\u{03A3}'..='\u{03A9}' | '\u{03B1}'..='\u{03C9}')
}

pub(crate) fn is_styled_variable(text: &str) -> bool {
    !text.is_empty()
        && text
            .chars()
            .all(|ch| matches!(ch, '\u{1D400}'..='\u{1D7FF}' | '\u{210E}'))
}

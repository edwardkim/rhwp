//! 여는 그룹이 없는 닫는 중괄호 뒤의 수식 내용을 보존한다.

use super::tokenizer::{Token, TokenType};

/// 한컴은 그룹을 열지 않은 `}`를 무시하고 뒤의 수식을 계속 표시한다.
/// 정상 그룹과 LEFT/RIGHT의 표시용 괄호는 그 소유를 유지한다.
/// 문자열 안의 중괄호는 Quoted 토큰이므로 복구 대상이 아니다.
pub(super) fn retain_owned_closing_braces(tokens: &mut Vec<Token>) {
    let mut group_depth = 0usize;
    let mut next_is_delimiter = false;
    tokens.retain(|token| {
        if next_is_delimiter {
            next_is_delimiter = false;
            return true;
        }
        if token.ty == TokenType::Command
            && (token.value.eq_ignore_ascii_case("LEFT")
                || token.value.eq_ignore_ascii_case("RIGHT"))
        {
            next_is_delimiter = true;
        } else if token.ty == TokenType::LBrace {
            group_depth += 1;
        } else if token.ty == TokenType::RBrace {
            if group_depth == 0 {
                return false;
            }
            group_depth -= 1;
        }
        true
    });
}

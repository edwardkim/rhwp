//! 한컴 NOT는 그룹 전체가 아니라 다음 표시 글자 하나를 부정한다.

use super::{EqNode, EqParser, TokenType};
use crate::renderer::equation::symbols::{
    is_function, is_structure_command, lookup_symbol, DecoKind, DECORATIONS, FONT_STYLES,
};

impl EqParser {
    pub(super) fn parse_hancom_not(&mut self) -> EqNode {
        let ty = self.current_type();
        let value = self.current_value().to_owned();
        let plain_command = ty == TokenType::Command
            && lookup_symbol(&value).is_none()
            && !is_function(&value)
            && !is_structure_command(&value)
            && !DECORATIONS.contains_key(value.to_ascii_lowercase().as_str())
            && !FONT_STYLES.contains_key(value.to_ascii_lowercase().as_str());
        let body = if ty == TokenType::LBrace {
            // 독립 Print는 여는 중괄호를 표시하고 그룹 내용은 뒤에 보존한다.
            // 그룹 토큰을 남겨 다음 요소가 닫는 중괄호와 첨자 소유를 처리한다.
            EqNode::Symbol("{".to_owned())
        } else if matches!(ty, TokenType::Text | TokenType::Number) || plain_command {
            match value.chars().next() {
                Some(first) if first.len_utf8() < value.len() => {
                    // 나머지는 다음 요소로 남긴다. 뒤 첨자를 앞 글자에 붙이지 않는다.
                    let token = &mut self.tokens[self.pos];
                    token.value = value[first.len_utf8()..].to_owned();
                    token.pos += 1;
                    token.space_before = false;
                    if ty == TokenType::Number {
                        EqNode::Number(first.to_string())
                    } else {
                        EqNode::Text(first.to_string())
                    }
                }
                _ => self.parse_single_atom_or_group(),
            }
        } else {
            self.parse_single_atom_or_group()
        };
        EqNode::Decoration {
            kind: DecoKind::StrikeThrough,
            body: Box::new(body),
        }
    }
}

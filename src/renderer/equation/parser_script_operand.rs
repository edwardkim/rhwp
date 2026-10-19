//! 한컴의 붙은 첨자 식과 부모의 위·아래 슬롯/중첩 첨자 소속을 구분한다.
//! 독립 정상 저장/Print 64식: e11874fc-a4ca-4b5e-b00b-dbb95a56d487,
//! 33b85391-0ae4-472a-9c78-e5f81ac7f686, 82959c79-aafe-4846-9e75-6bc955c598a6.
use super::{EqNode, EqParser, TokenType, MAX_EQ_DEPTH};

impl EqParser {
    pub(super) fn parse_script_operand(&mut self, has_sup: bool) -> EqNode {
        self.parse_script_span(TokenType::Superscript, has_sup)
    }

    pub(super) fn parse_superscript_operand(&mut self, has_sub: bool) -> EqNode {
        self.parse_script_span(TokenType::Subscript, has_sub)
    }

    fn parse_script_span(&mut self, opposite: TokenType, occupied: bool) -> EqNode {
        // 명시적으로 닫힌 인자를 뒤 본문과 결합하지 않는다.
        if self.current_type() == TokenType::LBrace {
            return self.parse_group();
        }
        // 붙은 반복 첨자도 구조 명령과 같은 깊이 상한을 지킨다.
        self.depth += 1;
        if self.depth > MAX_EQ_DEPTH {
            self.depth -= 1;
            return EqNode::Empty;
        }
        let mut items = Vec::new();
        while !self.at_end() {
            let ty = self.current_type();
            if matches!(
                ty,
                TokenType::RBrace | TokenType::RParen | TokenType::RBracket | TokenType::Whitespace
            ) || (!items.is_empty() && self.tokens[self.pos].space_before)
                // 문서의 무브레이스 첨자 뒤 새 그룹은 다음 본체다.
                // `_3{rmC}_1`의 C를 앞 첨자 크기로 축소하지 않는다.
                // 처음부터 braced인 인자는 위 parse_group 경로가 끝까지 소유한다.
                || (self.document_terms && !items.is_empty() && ty == TokenType::LBrace)
                || (ty == TokenType::Command && Self::cmd_eq(self.current_value(), "RIGHT"))
                || (ty == opposite && !occupied)
            {
                break;
            }
            if matches!(ty, TokenType::Subscript | TokenType::Superscript) {
                let Some(base) = items.pop() else {
                    break;
                };
                items.push(self.try_parse_scripts(base));
                continue;
            }
            let node = if ty == TokenType::LParen {
                self.parse_paren_group()
            } else if ty == TokenType::Symbol && matches!(self.current_value(), "->" | "<-") {
                let text = if self.current_value() == "->" {
                    "→"
                } else {
                    "←"
                };
                self.pos += 1;
                EqNode::MathSymbol(text.to_owned())
            } else if ty == TokenType::Command {
                let command = self.current_value().to_owned();
                self.pos += 1;
                self.parse_command_atom(&command)
            } else {
                self.parse_single_atom_or_group()
            };
            items.push(node);
        }
        self.depth -= 1;
        EqNode::Row(items).simplify()
    }
}

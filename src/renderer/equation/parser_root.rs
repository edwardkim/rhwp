//! 근호 인자의 지수/피연산자 소속. 기존 묶인 구문과 단일 원자 OF를 함께 처리한다.

use super::{EqNode, EqParser, TokenType};

impl EqParser {
    /// 제곱근 파싱: SQRT x, SQRT(n) of x
    pub(super) fn parse_sqrt(&mut self, latex_index: bool) -> EqNode {
        // LaTeX \sqrt[n]{x} 패턴
        if latex_index && self.current_type() == TokenType::LBracket {
            self.pos += 1; // [
            let previous_limit = self.term_limit;
            let close = self.tokens[self.pos..]
                .iter()
                .position(|token| token.ty == TokenType::RBracket)
                .map_or(self.tokens.len(), |offset| self.pos + offset);
            self.term_limit = self.term_limit.min(close);
            let mut index_nodes = Vec::new();
            while !self.at_end() && self.current_type() != TokenType::RBracket {
                index_nodes.push(self.parse_element());
            }
            self.term_limit = previous_limit;
            self.expect(TokenType::RBracket);

            let body = self.parse_single_or_group();
            return EqNode::Sqrt {
                index: Some(Box::new(EqNode::Row(index_nodes).simplify())),
                body: Box::new(body),
            };
        }

        // 소괄호는 OF가 뒤따를 때 지수이며, 그 외에는 근호 본체다.
        // 한컴 Print의 (3) 지수에는 괄호도 표시되므로 구문에 보존한다.
        if self.current_type() == TokenType::LParen {
            let close = self.paren_match[self.pos];
            let has_of = self.tokens.get(close + 1).is_some_and(|token| {
                token.ty == TokenType::Command && token.value.eq_ignore_ascii_case("of")
            });
            let group = self.parse_paren_group();
            if has_of {
                self.pos += 1; // OF
                return EqNode::Sqrt {
                    index: Some(Box::new(group)),
                    body: Box::new(self.parse_single_or_group()),
                };
            }
            return EqNode::Sqrt {
                index: None,
                body: Box::new(group),
            };
        }

        // SQRT {n} of {x} 패턴 확인 — 중괄호 + of
        if self.current_type() == TokenType::LBrace {
            // 먼저 {n} 뒤에 'of'가 있는지 미리 확인
            let saved_pos = self.pos;
            let brace_end = self.find_matching_brace(self.pos + 1);
            let after_brace = brace_end + 1;
            let has_of = after_brace < self.tokens.len()
                && self.tokens[after_brace].ty == TokenType::Command
                && self.tokens[after_brace].value.eq_ignore_ascii_case("of");

            if has_of {
                // {n} 파싱
                let index = self.parse_group();
                // 'of' 건너뛰기
                if self.current_type() == TokenType::Command
                    && self.current_value().eq_ignore_ascii_case("of")
                {
                    self.pos += 1;
                }
                let body = self.parse_single_or_group();
                return EqNode::Sqrt {
                    index: Some(Box::new(index)),
                    body: Box::new(body),
                };
            }
            // of가 없으면 되돌리고 일반 제곱근으로 처리
            self.pos = saved_pos;
        }

        // OF가 뒤따르는 단일 원자도 근 지수다. OF 없는 일반 근호는 소비하지 않는다.
        if matches!(
            self.current_type(),
            TokenType::Number | TokenType::Command | TokenType::Text | TokenType::Quoted
        ) && self.tokens.get(self.pos + 1).is_some_and(|token| {
            token.ty == TokenType::Command && token.value.eq_ignore_ascii_case("of")
        }) {
            let index = self.parse_single_or_group();
            self.pos += 1; // OF
            let body = self.parse_single_or_group();
            return EqNode::Sqrt {
                index: Some(Box::new(index)),
                body: Box::new(body),
            };
        }

        // 일반 제곱근
        let body = self.parse_single_or_group();
        EqNode::Sqrt {
            index: None,
            body: Box::new(body),
        }
    }
}

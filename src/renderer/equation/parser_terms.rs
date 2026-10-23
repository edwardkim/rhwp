//! 한컴 항의 경계를 구조 파서의 모든 요소 수집 경로에 공급한다.

use super::{EqNode, EqParser, TokenType};

impl EqParser {
    /// 원소(항) 하나를 읽는다. 완성한 원소의 시작 위치와 직전 글꼴 상태를
    /// `last_element` 에 남긴다(중첩 호출이 덮어쓴 값은 바깥 원소가 끝날 때 바깥 값으로
    /// 되돌아온다) — [#7873] 분수의 분자 재읽기.
    pub(super) fn parse_element(&mut self) -> EqNode {
        let start = self.pos;
        let state_before = self.font_state;
        let node = self.parse_element_term();
        self.last_element = Some((start, state_before));
        node
    }

    fn parse_element_term(&mut self) -> EqNode {
        if self.document_terms && self.pos >= self.term_limit {
            return EqNode::Empty;
        }
        let first_type = self.current_type();
        let first = self.parse_element_atom();
        if !self.document_terms || matches!(first_type, TokenType::Whitespace | TokenType::Eof) {
            return first;
        }
        if is_font_switch(&first) {
            // 전환 marker는 뒤 항에도 적용된다. 그 바로 뒤 첨자는 같은 인자
            // 경로로 소비하되 marker를 보존해 정체/이탤릭 범위를 유지한다.
            // 기저 없는 새 항으로 다시 읽으면 유효한 it_{n}rmP it_{r}가 중단된다.
            // marker를 여기서 Row로 단순화하면 첨자가 없는 전환이 사라지거나
            // 첨자만 인자 범위로 묶인다. 현재 그룹의 Row가 전환을 소비하게 한다.
            return self.try_parse_scripts(first);
        }
        if first_type == TokenType::LBrace {
            return self.finish_named_scripts(first);
        }
        let mut items = vec![first];
        while self.has_term_continuation() {
            let start = self.pos;
            items.push(self.parse_element_atom());
            if self.pos == start {
                break;
            }
        }
        self.finish_named_scripts(EqNode::Row(items).simplify())
    }

    fn has_term_continuation(&self) -> bool {
        if self.pos >= self.term_limit || self.at_end() {
            return false;
        }
        let token = &self.tokens[self.pos];
        if token.space_before
            || matches!(
                token.ty,
                TokenType::LBrace | TokenType::RBrace
            )
            // 독립 Print의 a~b over c는 a와 b가 같은 분자다. 붙은 normal
            // 공간만 항 안에 남기며 얇은 공간·행/칸 구분은 경계로 유지한다.
            || (token.ty == TokenType::Whitespace && token.value != "~")
        {
            return false;
        }
        // 구조 연산자·닫힘과 글꼴 전환은 피연산자 안으로 소비하지 않는다.
        token.ty != TokenType::Command
            || ![
                "OVER", "ATOP", "CHOOSE", "LSUB", "LSUP", "SUB", "SUP", "RIGHT", "END", "OF", "RM",
                "IT",
            ]
            .iter()
            .any(|name| token.value.eq_ignore_ascii_case(name))
    }
}

fn is_font_switch(node: &EqNode) -> bool {
    matches!(node, EqNode::FontStyle { body, .. } if matches!(body.as_ref(), EqNode::Empty))
}

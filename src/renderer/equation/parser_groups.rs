//! 기존 입력 소유와 공통 파서 상태를 보존한다.

use super::*;

impl EqParser {
    /// 중괄호 그룹 파싱: {...} (깊이 가드).
    /// 그룹 내의 OVER는 parse_expression의 중위 연산자 처리로 자동 처리된다.
    pub(super) fn parse_group(&mut self) -> EqNode {
        if self.current_type() != TokenType::LBrace {
            // 여는 중괄호가 아니면 그룹이 아님 — 위임 (깊이 변화 없음).
            return self.parse_element();
        }
        self.depth += 1;
        if self.depth > MAX_EQ_DEPTH {
            // 상한 초과: 그룹 전체를 건너뛰어(truncate) 진행을 보장한다.
            self.depth -= 1;
            self.skip_braced_group();
            return EqNode::Empty;
        }
        let node = self.parse_group_inner();
        self.depth -= 1;
        node
    }

    pub(super) fn parse_group_inner(&mut self) -> EqNode {
        self.expect(TokenType::LBrace); // 여는 '{' 소비 (호출부에서 존재 보장)

        let mut children = Vec::new();
        let mut over_run = 0u32;
        while !self.at_end() {
            if self.current_type() == TokenType::RBrace {
                break;
            }
            // OVER/ATOP 중위 연산자: 그룹 내에서도 동일하게 처리
            if self.try_consume_infix_over_atop(&mut children, &mut over_run) {
                continue;
            }
            children.push(self.parse_element());
        }

        // 닫는 괄호 건너뛰기
        self.expect(TokenType::RBrace);

        EqNode::Row(children).simplify()
    }

    /// 현재 `{` 부터 매칭되는 `}` 까지 통째로 소비한다(깊이 상한 초과 시 truncate).
    pub(super) fn skip_braced_group(&mut self) {
        // 전제: 현재 토큰이 LBrace.
        self.pos += 1; // 여는 '{' 소비
        let close = self.find_matching_brace(self.pos);
        self.pos = close.min(self.tokens.len());
        self.expect(TokenType::RBrace); // 매칭 '}' 가 있으면 소비
    }

    /// 현재 `(` 부터 매칭되는 `)` 까지 통째로 소비한다(깊이 상한 초과 시 truncate).
    pub(super) fn skip_paren_group(&mut self) {
        // 전제: 현재 토큰이 LParen.
        let mut depth = 0i32;
        while self.pos < self.tokens.len() {
            match self.tokens[self.pos].ty {
                TokenType::LParen => depth += 1,
                TokenType::RParen => {
                    depth -= 1;
                    self.pos += 1;
                    if depth <= 0 {
                        return;
                    }
                    continue;
                }
                TokenType::Eof => return,
                _ => {}
            }
            self.pos += 1;
        }
    }

    /// 매칭되는 닫는 괄호 위치 찾기
    pub(super) fn find_matching_brace(&self, start: usize) -> usize {
        let mut depth = 1i32;
        let mut pos = start;
        while pos < self.tokens.len() {
            match self.tokens[pos].ty {
                TokenType::LBrace => depth += 1,
                TokenType::RBrace => {
                    depth -= 1;
                    if depth == 0 {
                        return pos;
                    }
                }
                _ => {}
            }
            pos += 1;
        }
        self.tokens.len()
    }

    /// 현재 LParen 의 매칭 RParen 다음 첨자(`^`/`_`)인지 (#1305).
    /// RParen 뒤의 Thin 공백(`) 하나는 첨자 결합 전에 보존되는 공백이므로 건너뛴다.
    /// 참이면 `(...)` 를 Paren 그룹으로 묶어 첨자를 결합해야 한다.
    /// 현재 토큰이 LParen 이라는 전제. 사전계산된 `paren_match` 로 O(1) 조회한다
    /// (원래는 매 호출 O(n) 전방 스캔 → 괄호 많은 입력에서 O(n^2) 행 유발).
    pub(super) fn paren_then_script(&self) -> bool {
        let close = self
            .paren_match
            .get(self.pos)
            .copied()
            .unwrap_or(self.tokens.len());
        let next = close + 1;
        let script = if self
            .tokens
            .get(next)
            .is_some_and(|token| token.ty == TokenType::Whitespace && token.value == "`")
        {
            next + 1
        } else {
            next
        };
        matches!(
            self.tokens.get(script).map(|t| t.ty),
            Some(TokenType::Subscript) | Some(TokenType::Superscript)
        ) || (self.document_terms
            && self.tokens.get(script).is_some_and(|token| {
                token.ty == TokenType::Command
                    && ["LSUB", "LSUP", "SUB", "SUP"]
                        .iter()
                        .any(|name| Self::cmd_eq(&token.value, name))
            }))
    }

    /// `(...)` 를 자동크기 괄호 그룹으로 파싱 (#1305). 현재 LParen 전제 (깊이 가드).
    pub(super) fn parse_paren_group(&mut self) -> EqNode {
        self.depth += 1;
        if self.depth > MAX_EQ_DEPTH {
            // 상한 초과: 괄호 그룹 전체를 건너뛰어(truncate) 진행을 보장한다.
            self.depth -= 1;
            self.skip_paren_group();
            return EqNode::Empty;
        }
        let node = self.parse_paren_group_inner();
        self.depth -= 1;
        node
    }

    pub(super) fn parse_paren_group_inner(&mut self) -> EqNode {
        // 안쪽 소괄호의 닫힘을 바깥 묶음의 끝으로 오인하지 않는다.
        let close = self.paren_match[self.pos];
        let previous_limit = self.term_limit;
        self.term_limit = self.term_limit.min(close);
        self.pos += 1; // '(' 소비
        let mut items = Vec::new();
        let mut over_run = 0u32;
        while !self.at_end() && self.pos < close {
            if self.try_consume_infix_over_atop(&mut items, &mut over_run) {
                continue;
            }
            items.push(self.parse_element());
        }
        self.term_limit = previous_limit;
        self.expect(TokenType::RParen); // ')' 소비
        EqNode::Paren {
            left: "(".to_string(),
            right: ")".to_string(),
            body: Box::new(EqNode::Row(items).simplify()),
        }
    }
}

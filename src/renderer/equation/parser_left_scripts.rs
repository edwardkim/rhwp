//! 한컴의 명령형 첨자를 현재 항·함수 인자의 소속 범위에서 결합한다.

use super::{EqNode, EqParser, TokenType, MAX_EQ_DEPTH};

fn marker(value: &str) -> Option<(bool, bool)> {
    if value.eq_ignore_ascii_case("LSUB") {
        Some((true, false))
    } else if value.eq_ignore_ascii_case("LSUP") {
        Some((true, true))
    } else if value.eq_ignore_ascii_case("SUB") {
        Some((false, false))
    } else if value.eq_ignore_ascii_case("SUP") {
        Some((false, true))
    } else {
        None
    }
}

impl EqParser {
    pub(super) fn finish_named_scripts(&mut self, mut base: EqNode) -> EqNode {
        if !self.document_terms
            || self.pos >= self.term_limit
            || self.current_type() != TokenType::Command
            || marker(self.current_value()).is_none()
        {
            return base;
        }
        let (mut left_sub, mut left_sup) = (None, None);
        if let EqNode::LeftScripts {
            base: inner,
            sub,
            sup,
        } = base
        {
            base = *inner;
            left_sub = sub;
            left_sup = sup;
        }
        let (mut right_sub, mut right_sup) = (None, None);
        base = match base {
            EqNode::Subscript { base, sub } => {
                right_sub = Some(sub);
                *base
            }
            EqNode::Superscript { base, sup } => {
                right_sup = Some(sup);
                *base
            }
            EqNode::SubSup { base, sub, sup } => {
                right_sub = Some(sub);
                right_sup = Some(sup);
                *base
            }
            body => body,
        };
        for _ in 0..MAX_EQ_DEPTH {
            if self.at_end() || self.pos >= self.term_limit {
                break;
            }
            let kind = match self.current_type() {
                TokenType::Command => marker(self.current_value()),
                TokenType::Subscript => Some((false, false)),
                TokenType::Superscript => Some((false, true)),
                _ => None,
            };
            let Some((left, upper)) = kind else { break };
            let slot = match (left, upper) {
                (true, false) => &mut left_sub,
                (true, true) => &mut left_sup,
                (false, false) => &mut right_sub,
                (false, true) => &mut right_sup,
            };
            if slot.is_some() {
                break;
            }
            self.pos += 1;
            let operand = self.parse_named_script_operand();
            if !matches!(operand, EqNode::Empty) {
                *slot = Some(Box::new(operand));
            }
        }
        let right = match (right_sub, right_sup) {
            (Some(sub), Some(sup)) => EqNode::SubSup {
                base: Box::new(base),
                sub,
                sup,
            },
            (Some(sub), None) => EqNode::Subscript {
                base: Box::new(base),
                sub,
            },
            (None, Some(sup)) => EqNode::Superscript {
                base: Box::new(base),
                sup,
            },
            (None, None) => base,
        };
        if left_sub.is_some() || left_sup.is_some() {
            EqNode::LeftScripts {
                base: Box::new(right),
                sub: left_sub,
                sup: left_sup,
            }
        } else {
            right
        }
    }

    /// 명시 그룹의 내부 첨자는 그룹이 소유한다. 묶이지 않은 다음 marker는
    /// 현재 인자에 삼키지 않고 부모의 다음 슬롯으로 돌려준다.
    pub(super) fn parse_named_script_operand(&mut self) -> EqNode {
        if self.at_end() || self.pos >= self.term_limit {
            return EqNode::Empty;
        }
        // 인자 누락은 닫힘·행·칸 경계를 소비하지 않는다.
        if matches!(
            self.current_type(),
            TokenType::RBrace | TokenType::RParen | TokenType::RBracket
        ) || (self.current_type() == TokenType::Whitespace
            && matches!(self.current_value(), "#" | "&"))
            || (self.current_type() == TokenType::Command
                && ["OVER", "ATOP", "CHOOSE", "RIGHT", "END", "OF"]
                    .iter()
                    .any(|name| Self::cmd_eq(self.current_value(), name)))
        {
            return EqNode::Empty;
        }
        if self.current_type() == TokenType::LBrace {
            return self.parse_group();
        }
        let previous_limit = self.term_limit;
        let mut nesting = 0_u32;
        for index in self.pos..previous_limit {
            let token = &self.tokens[index];
            if nesting == 0
                && (matches!(token.ty, TokenType::Subscript | TokenType::Superscript)
                    || (token.ty == TokenType::Command && marker(&token.value).is_some()))
            {
                self.term_limit = index;
                break;
            }
            match token.ty {
                TokenType::LBrace | TokenType::LParen | TokenType::LBracket => nesting += 1,
                TokenType::RBrace | TokenType::RParen | TokenType::RBracket if nesting > 0 => {
                    nesting -= 1
                }
                TokenType::RBrace | TokenType::RParen | TokenType::RBracket => break,
                _ => {}
            }
        }
        let result = self.parse_element();
        self.term_limit = previous_limit;
        result
    }

    /// 큰 연산자의 이름 뒤 marker는 그 템플릿의 인자다. 바깥 그룹의
    /// 왼쪽 첨자와 구별하고 원본 스크립트 자체는 문서 모델에서 보존한다.
    pub(super) fn consume_operator_script(
        &mut self,
        sub: &mut Option<Box<EqNode>>,
        sup: &mut Option<Box<EqNode>>,
    ) -> bool {
        if !self.document_terms || self.current_type() != TokenType::Command {
            return false;
        }
        let Some((left, upper)) = marker(self.current_value()) else {
            return false;
        };
        self.pos += 1;
        let operand = self.parse_named_script_operand();
        if !left {
            let slot = if upper { sup } else { sub };
            if slot.is_none() && !matches!(operand, EqNode::Empty) {
                *slot = Some(Box::new(operand));
            }
        }
        true
    }
    /// 적분과 합계 기호가 소유하는 첨자 인자를 같은 경로에서 읽는다.
    pub(super) fn parse_operator_scripts(&mut self) -> (Option<Box<EqNode>>, Option<Box<EqNode>>) {
        let mut sub = None;
        let mut sup = None;

        // 첨자 파싱
        loop {
            if self.at_end() || (self.document_terms && self.pos >= self.term_limit) {
                break;
            }
            if self.consume_operator_script(&mut sub, &mut sup) {
                continue;
            }
            if self.current_type() == TokenType::Subscript && sub.is_none() {
                self.pos += 1;
                sub = Some(Box::new(self.parse_script_operand(sup.is_some())));
            } else if self.current_type() == TokenType::Superscript && sup.is_none() {
                self.pos += 1;
                sup = Some(Box::new(self.parse_superscript_operand(sub.is_some())));
            } else {
                break;
            }
        }

        (sub, sup)
    }
}

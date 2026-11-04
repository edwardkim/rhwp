//! 부착 첨자의 인자 경계와 문서 오류 이후 입력 중단.

use super::*;

impl EqParser {
    /// 첨자(subscript/superscript) 파싱 시도
    /// 한컴 수식에서 함수/기호 뒤에 Thin 공백(`)이 오고 첨자가 따라오는 패턴이 일반적이므로,
    /// Thin 공백 뒤에 첨자가 있으면 공백을 건너뛰고 첨자를 파싱한다.
    pub(super) fn try_parse_scripts(&mut self, base: EqNode) -> EqNode {
        let mut result = base;
        // 구형 명시적 그룹 기저는 첫 첨자 안으로 뒤 첨자를 이어받는다.
        // 일반 a_b^c의 병렬 슬롯과 {a}_b^c의 중첩 소속을 구분한다.
        // 독립 세대·순서·중괄호 Print: f4eb7352-d61b-4cdc-956b-46934265d01c.
        let grouped_legacy_base = self.legacy_display && matches!(result, EqNode::Group(_));
        let mut has_sub = false;
        let mut has_sup = false;
        let mut sub = None;
        let mut sup = None;

        loop {
            if self.at_end() || (self.document_terms && self.pos >= self.term_limit) {
                break;
            }
            // Thin 공백(`) 뒤에 첨자가 오면 첨자 결합은 유지하되(과거 exam_math
            // `log`_{2}` 첨자 파싱 실패 수정), 공백의 시각 폭은 base 뒤에 보존한다.
            // 한컴은 `a_{2}` 와 `` a`_{2} `` 를 다르게 렌더한다 — 공백 토큰을
            // 삭제만 하면 첨자가 붙는 위치(가로)와 AST 정보가 함께 사라진다(#5534).
            // Subscript/Superscript 레이아웃은 base box 오른쪽 끝에 첨자를 두므로
            // base 를 Row[base, Space(Thin)] 로 감싸면 첨자가 1/4 공백만큼
            // 오른쪽에 놓인다.
            if self.current_type() == TokenType::Whitespace && self.current_value() == "`" {
                let next_pos = self.pos + 1;
                if next_pos < self.tokens.len() {
                    let next_ty = self.tokens[next_pos].ty;
                    if next_ty == TokenType::Subscript || next_ty == TokenType::Superscript {
                        self.pos += 1; // Thin 공백 소비 (폭은 아래에서 보존)
                        result = EqNode::Row(vec![result, EqNode::Space(SpaceKind::Thin)]);
                    }
                }
            }
            let ty = self.current_type();
            if ty == TokenType::Subscript && !has_sub {
                self.pos += 1;
                if self.stop_at_missing_script_operand() {
                    break;
                }
                sub = Some(self.parse_script_operand(has_sup || grouped_legacy_base));
                has_sub = true;
            } else if ty == TokenType::Superscript && !has_sup {
                self.pos += 1;
                if self.stop_at_missing_script_operand() {
                    break;
                }
                sup = Some(self.parse_superscript_operand(has_sub));
                has_sup = true;
            } else {
                break;
            }
        }

        if has_sub && has_sup {
            EqNode::SubSup {
                base: Box::new(result),
                sub: Box::new(sub.unwrap_or(EqNode::Empty)),
                sup: Box::new(sup.unwrap_or(EqNode::Empty)),
            }
        } else if has_sub {
            EqNode::Subscript {
                base: Box::new(result),
                sub: Box::new(sub.unwrap_or(EqNode::Empty)),
            }
        } else if has_sup {
            EqNode::Superscript {
                base: Box::new(result),
                sup: Box::new(sup.unwrap_or(EqNode::Empty)),
            }
        } else {
            result
        }
    }

    /// 명시적 빈 그룹은 유효한 첨자 인자이며 누락과 구분한다.
    fn stop_at_missing_script_operand(&mut self) -> bool {
        if self.document_terms
            && (self.at_end()
                || self.pos >= self.term_limit
                || matches!(
                    self.current_type(),
                    TokenType::RBrace
                        | TokenType::RParen
                        | TokenType::RBracket
                        | TokenType::Whitespace
                        | TokenType::Subscript
                        | TokenType::Superscript
                ))
        {
            self.stopped_document_input = true;
            true
        } else {
            false
        }
    }
}

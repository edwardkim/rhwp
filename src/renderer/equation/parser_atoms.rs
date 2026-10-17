//! 기존 입력 소유와 공통 파서 상태를 보존한다.

use super::*;

impl EqParser {
    /// 단일 요소 파싱
    pub(super) fn parse_element_atom(&mut self) -> EqNode {
        if self.at_end() {
            return EqNode::Empty;
        }

        let ty = self.current_type();
        let val = self.current_value().to_string();
        let roman_literal = self.document_terms
            && self
                .tokens
                .get(self.pos)
                .is_some_and(|token| token.roman_literal);

        match ty {
            TokenType::Command => {
                self.pos += 1;
                self.parse_command(&val)
            }
            TokenType::Number => {
                self.pos += 1;
                self.try_parse_scripts(EqNode::Number(val))
            }
            TokenType::Symbol => {
                self.pos += 1;
                let attach_scripts = self.document_terms
                    || super::super::symbols::is_logical_symbol(&val)
                    || (!val.is_ascii() && super::super::symbols::is_binary_symbol(&val));
                // 한컴의 붙은 ASCII 화살표 축약을 의미 기호로 보존한다.
                let node = if matches!(val.as_str(), "->" | "<-") {
                    EqNode::MathSymbol(if val == "->" { "→" } else { "←" }.to_string())
                } else if !val.is_ascii() && super::super::symbols::is_binary_symbol(&val) {
                    // 새 기호 토큰도 기존 Text/명령 경로의 첨자 결합을 유지한다.
                    EqNode::MathSymbol(val)
                } else if !val.is_ascii()
                    && (super::super::symbols::is_relation_symbol(&val)
                        || super::super::symbols::is_logical_symbol(&val))
                {
                    EqNode::MathSymbol(val)
                } else {
                    EqNode::Symbol(val)
                };
                if attach_scripts {
                    // 표시 기호도 뒤 첨자의 기저다. 첨자를 남겨 두면 다음 항에서
                    // orphan 오류로 중단되어 정상 저장본의 뒤 수식 전체를 잃는다.
                    // 처음부터 기저가 없는 첨자의 중단 계약은 아래에서 유지한다.
                    self.try_parse_scripts(node)
                } else {
                    node
                }
            }
            TokenType::Text => {
                self.pos += 1;
                let node = if super::super::symbols::is_relation_symbol(&val) {
                    EqNode::MathSymbol(val)
                } else {
                    literal_font(self.styled_text(val), roman_literal)
                };
                self.try_parse_scripts(node)
            }
            TokenType::Quoted => {
                self.pos += 1;
                self.try_parse_scripts(literal_font(EqNode::Quoted(val), roman_literal))
            }
            TokenType::Whitespace => {
                self.pos += 1;
                match val.as_str() {
                    // 한컴 Print의 명시적 공간은 첨자의 기저도 소유한다.
                    // 공간 없는 orphan 첨자는 아래의 입력 중단 계약을 유지한다.
                    "~" => self.try_parse_scripts(EqNode::Space(SpaceKind::Normal)),
                    "`" => self.try_parse_scripts(EqNode::Space(SpaceKind::Thin)),
                    "#" => EqNode::Newline,
                    "&" => EqNode::Space(SpaceKind::Tab),
                    _ => EqNode::Empty,
                }
            }
            TokenType::LBrace => {
                let group = self.parse_group();
                self.try_parse_scripts(group)
            }
            TokenType::LParen => {
                // #1305: `(...)` 뒤에 첨자가 오면 Paren 그룹으로 묶어 첨자를 결합한다
                // (그렇지 않으면 `)` 뒤 `^2` 가 base 없는 orphan Superscript 가 됨).
                // 첨자가 없으면 기존대로 느슨한 Symbol 흐름 유지 → 일반 괄호 렌더 무변경.
                if self.paren_then_script() {
                    let group = self.parse_paren_group();
                    self.try_parse_scripts(group)
                } else {
                    self.pos += 1;
                    let node = EqNode::Symbol(val);
                    if self.document_terms {
                        self.try_parse_scripts(node)
                    } else {
                        node
                    }
                }
            }
            TokenType::RParen | TokenType::LBracket | TokenType::RBracket => {
                self.pos += 1;
                let node = EqNode::Symbol(val);
                if self.document_terms {
                    self.try_parse_scripts(node)
                } else {
                    node
                }
            }
            TokenType::Subscript | TokenType::Superscript => {
                if self.document_terms {
                    // 한컴 Print의 기저 없는 첨자는 이미 읽은 항을 남기고
                    // 이 위치 이후 입력을 중단한다. 공개 호환 파서는 별도다.
                    self.stopped_document_input = true;
                    return EqNode::Empty;
                }
                // 베이스 없는 첨자 (예: _{y} x)
                self.try_parse_scripts(EqNode::Empty)
            }
            _ => {
                self.pos += 1;
                EqNode::Empty
            }
        }
    }

    /// 단일 토큰 또는 그룹 파싱 (첨자/인자용)
    pub(super) fn parse_single_or_group(&mut self) -> EqNode {
        if self.at_end() || self.current_type() == TokenType::RBrace || self.pos >= self.term_limit
        {
            return EqNode::Empty;
        }
        if self.document_terms {
            self.parse_element()
        } else {
            self.parse_single_atom_or_group()
        }
    }

    pub(super) fn parse_single_atom_or_group(&mut self) -> EqNode {
        if self.at_end() {
            return EqNode::Empty;
        }

        // RBrace는 그룹 종료 마커 — 소비하지 않고 빈 노드 반환
        if self.current_type() == TokenType::RBrace {
            return EqNode::Empty;
        }

        if self.current_type() == TokenType::LBrace {
            return self.parse_group();
        }

        // 단일 토큰
        let ty = self.current_type();
        let val = self.current_value().to_string();
        let roman_literal = self.document_terms
            && self
                .tokens
                .get(self.pos)
                .is_some_and(|token| token.roman_literal);
        self.pos += 1;

        match ty {
            TokenType::Command => {
                if let Some(symbol) = lookup_symbol(&val) {
                    EqNode::MathSymbol(symbol.to_string())
                } else if is_function(&val) {
                    EqNode::Function(lookup_function(&val).unwrap_or(&val).to_string())
                } else {
                    // [#1204-B] decoration/구조 명령(bar, sqrt 등)도 단일 인자/body 로
                    // 올 수 있다 (`rm bar {...}`). parse_command 로 위임 — 미지 명령은
                    // parse_command 의 fall-through 가 Text 로 처리하므로 안전.
                    self.parse_command(&val)
                }
            }
            TokenType::Number => EqNode::Number(val),
            TokenType::Text => literal_font(self.styled_text(val), roman_literal),
            TokenType::Quoted => literal_font(EqNode::Quoted(val), roman_literal),
            TokenType::Symbol => EqNode::Symbol(val),
            TokenType::Whitespace => whitespace_node(&val),
            _ => self.styled_text(val),
        }
    }
}

/// 글리프 선택은 공통 AST 글꼴 노드로 전달한다. 토큰 경계나 인자 소유는 바꾸지 않는다.
pub(super) fn literal_font(node: EqNode, roman_literal: bool) -> EqNode {
    if roman_literal {
        EqNode::FontStyle {
            style: FontStyleKind::Roman,
            body: Box::new(node),
        }
    } else {
        node
    }
}

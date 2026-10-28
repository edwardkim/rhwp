//! 기존 파서 상태를 공유하며 입력 소유 범위 안에서 처리한다.

use super::*;

impl EqParser {
    /// 명령어 처리 (깊이 가드).
    ///
    /// 구조 명령(SQRT/MATRIX/LEFT/…)은 모두 여기서 처리되고, 그 인자 파싱이 다시
    /// `parse_command`/`parse_group`/`parse_paren_group` 로 되돌아오므로 이 셋을
    /// 공유 카운터로 감싸면 모든 중첩 경로가 [`MAX_EQ_DEPTH`] 로 제한된다. 상한을
    /// 넘으면 더 내려가지 않고 `Empty` 를 돌려준다(명령 토큰은 호출부에서 이미
    /// 소비했으므로 진행이 보장된다).
    pub(super) fn parse_command(&mut self, cmd: &str) -> EqNode {
        self.parse_command_with_tail(cmd, true)
    }

    // 첨자 span에서는 뒤 marker의 소속을 부모 슬롯이 먼저 판정한다.
    pub(super) fn parse_command_atom(&mut self, cmd: &str) -> EqNode {
        self.parse_command_with_tail(cmd, false)
    }

    pub(super) fn parse_command_with_tail(&mut self, cmd: &str, attach_scripts: bool) -> EqNode {
        self.depth += 1;
        if self.depth > MAX_EQ_DEPTH {
            self.depth -= 1;
            return EqNode::Empty;
        }
        let node = self.parse_command_inner(cmd, attach_scripts);
        self.depth -= 1;
        node
    }

    pub(super) fn parse_command_inner(&mut self, cmd: &str, attach_scripts: bool) -> EqNode {
        // 문서 tokenizer가 표시한 명시적 LaTeX 근호만 각괄호 지수를 쓴다.
        let explicit_root = cmd
            .strip_prefix('\\')
            .filter(|name| name.eq_ignore_ascii_case("sqrt") || name.eq_ignore_ascii_case("root"));
        let cmd = explicit_root.unwrap_or(cmd);
        let cmd_upper = cmd.to_ascii_uppercase();
        let cu = cmd_upper.as_str();
        match classify_command(cmd) {
            EqCommandClass::InfixDiscard => EqNode::Empty,
            EqCommandClass::LatexFraction => self.parse_latex_fraction(),
            EqCommandClass::RomanText => {
                // 제한: 토크나이저가 일반 공백을 건너뛰므로 \text{a b} 내부 공백은 보존되지 않음.
                // 공백이 필요하면 hwpeq 관례대로 ~ 사용 (\text{if~}).
                let body = self.parse_single_or_group();
                EqNode::FontStyle {
                    style: FontStyleKind::Roman,
                    body: Box::new(body),
                }
            }
            EqCommandClass::Phantom => {
                self.parse_single_or_group();
                EqNode::Text(" ".to_string())
            }
            EqCommandClass::LatexSpacing => {
                let space = lookup_symbol(cu).unwrap_or(" ");
                EqNode::Text(space.to_string())
            }
            EqCommandClass::Overset => {
                let over = self.parse_single_or_group();
                let base = self.parse_single_or_group();
                EqNode::Superscript {
                    base: Box::new(base),
                    sup: Box::new(over),
                }
            }
            EqCommandClass::Underset => {
                let under = self.parse_single_or_group();
                let base = self.parse_single_or_group();
                EqNode::Subscript {
                    base: Box::new(base),
                    sub: Box::new(under),
                }
            }
            EqCommandClass::BeginEnv => self.parse_latex_environment(),
            EqCommandClass::EndEnv => {
                self.skip_brace_arg();
                EqNode::Empty
            }
            EqCommandClass::Sqrt => {
                self.parse_sqrt(explicit_root.is_some() || !self.document_terms)
            }
            EqCommandClass::IntegralNolimits => {
                // nolimits: 큰 기호 + 일반 첨자 (BigOp이 아닌 MathSymbol로 처리)
                let symbol = lookup_symbol(cu)
                    .or_else(|| lookup_symbol(cmd))
                    .unwrap_or("∫")
                    .to_string();
                let node = EqNode::MathSymbol(symbol);
                if self.document_terms {
                    let (sub, sup) = self.parse_operator_scripts();
                    match (sub, sup) {
                        (Some(sub), Some(sup)) => EqNode::SubSup {
                            base: Box::new(node),
                            sub,
                            sup,
                        },
                        (Some(sub), None) => EqNode::Subscript {
                            base: Box::new(node),
                            sub,
                        },
                        (None, Some(sup)) => EqNode::Superscript {
                            base: Box::new(node),
                            sup,
                        },
                        (None, None) => node,
                    }
                } else {
                    self.finish_command_atom(node, attach_scripts)
                }
            }
            EqCommandClass::SmallOperator => {
                let symbol = lookup_symbol(cu).unwrap_or("?").to_string();
                self.finish_command_atom(EqNode::MathSymbol(symbol), attach_scripts)
            }
            EqCommandClass::BigOperator => {
                let symbol = if is_big_operator(cu) {
                    lookup_symbol(cu).unwrap_or("?").to_string()
                } else {
                    lookup_symbol(cmd).unwrap_or("?").to_string()
                };
                self.parse_big_op(symbol)
            }
            EqCommandClass::Limit => self.parse_limit(cmd == "Lim"),
            EqCommandClass::Matrix => self.parse_matrix(matrix_style(cu)),
            EqCommandClass::Column if self.document_terms => self.parse_column(pile_align(cu)),
            EqCommandClass::Column => self.parse_command_fallback(cmd, attach_scripts),
            EqCommandClass::Cases => self.parse_cases(),
            EqCommandClass::EqAlign => self.parse_eqalign(),
            EqCommandClass::Pile => self.parse_pile(pile_align(cu)),
            EqCommandClass::LeftDelim => {
                // ★ KeepGong fix: 구분기호 그룹(left|...right| 등) 뒤 첨자(^/_)를 그룹 전체에 부착.
                //   기존엔 try_parse_scripts 를 안 거쳐 |x|^3 의 ^3 가 base 없는 고아 첨자가 됐다.
                let node = self.parse_left_right();
                self.finish_command_atom(node, attach_scripts)
            }
            EqCommandClass::RightDiscard => EqNode::Empty,
            EqCommandClass::Rel => {
                let is_buildrel = cu == "BUILDREL";
                let arrow_node = self.parse_element_atom();
                let arrow = match &arrow_node {
                    EqNode::MathSymbol(s) => s.clone(),
                    EqNode::Symbol(s) => s.clone(),
                    EqNode::Text(s) => s.clone(),
                    _ => "→".to_string(),
                };
                let over = self.parse_single_or_group();
                let under = if !is_buildrel {
                    Some(Box::new(self.parse_single_or_group()))
                } else {
                    None
                };
                EqNode::Rel {
                    arrow,
                    over: Box::new(over),
                    under,
                }
            }
            EqCommandClass::LongDiv => {
                let divisor = self.parse_single_or_group();
                let quotient = self.parse_single_or_group();
                let body = self.parse_single_or_group();
                EqNode::LongDiv {
                    divisor: Box::new(divisor),
                    quotient: Box::new(quotient),
                    body: Box::new(body),
                }
            }
            EqCommandClass::Ladder => self.parse_matrix(if cu == "SLADDER" {
                MatrixStyle::ShortLadder
            } else {
                MatrixStyle::Ladder
            }),
            EqCommandClass::Benzene => EqNode::MathSymbol("⌬".to_string()),
            EqCommandClass::Bigg => self.parse_element(),
            EqCommandClass::Choose => {
                // n CHOOSE r → 이전 요소와 다음 요소를 조합으로
                let body = self.parse_single_or_group();
                EqNode::Binomial {
                    top: Box::new(EqNode::Empty),
                    bottom: Box::new(body),
                }
            }
            EqCommandClass::Binom => {
                let top = self.parse_single_or_group();
                let bottom = self.parse_single_or_group();
                EqNode::Binomial {
                    top: Box::new(top),
                    bottom: Box::new(bottom),
                }
            }
            EqCommandClass::Color => self.parse_color(),
            EqCommandClass::LeftScript => {
                if self.document_terms {
                    self.invalid_document_script = true;
                    return EqNode::Empty;
                }
                let script = self.parse_single_or_group();
                let body = self.parse_single_or_group();
                if cu == "LSUB" {
                    EqNode::Subscript {
                        base: Box::new(body),
                        sub: Box::new(script),
                    }
                } else {
                    EqNode::Superscript {
                        base: Box::new(body),
                        sup: Box::new(script),
                    }
                }
            }
            EqCommandClass::Sup | EqCommandClass::Sub => {
                if self.document_terms {
                    self.invalid_document_script = true;
                    return EqNode::Empty;
                }
                let body = self.parse_single_or_group();
                self.finish_command_atom(body, attach_scripts)
            }
            EqCommandClass::Fallback => self.parse_command_fallback(cmd, attach_scripts),
        }
    }

    /// 장식·글꼴·기호·함수·미지 명령. 분류표에 없는 이름만 여기로 온다.
    pub(super) fn parse_command_fallback(&mut self, cmd: &str, attach_scripts: bool) -> EqNode {
        let roman_literal = self.document_terms
            && self
                .pos
                .checked_sub(1)
                .and_then(|index| self.tokens.get(index))
                .is_some_and(|token| token.roman_literal);
        // [#1204] hwpeq 명령은 대소문자 무시 — DECORATIONS/FONT_STYLES 는 소문자 키이므로
        // 1차 lookup 실패 시 소문자 fallback (`RM`/`BAR` 등 대문자 변형이 leak 되지 않도록).
        let cmd_lower = cmd.to_ascii_lowercase();

        if cmd_lower == "not" {
            return self.parse_hancom_not();
        }
        if cmd == r"\not" {
            return EqNode::Decoration {
                kind: symbols::DecoKind::StrikeThrough,
                body: Box::new(self.parse_single_or_group()),
            };
        }

        if let Some(&deco) = DECORATIONS
            .get(cmd)
            .or_else(|| DECORATIONS.get(cmd_lower.as_str()))
        {
            let body = self.parse_single_or_group();
            let body = self.finish_command_atom(body, attach_scripts);
            return EqNode::Decoration {
                kind: deco,
                body: Box::new(body),
            };
        }

        if let Some(&style) = FONT_STYLES
            .get(cmd)
            .or_else(|| FONT_STYLES.get(cmd_lower.as_str()))
        {
            // 한컴의 rm/it은 단일 인자가 아니라 글꼴 전환이다. 명시적 공백을 인자로
            // 소비하지 않는다. [#7873] 전환은 그룹·분수·첨자 경계를 넘어 수식 끝까지
            // 지속되므로(`font_state` 주석) 이후 글자 식별자가 `styled_literal` 로 감싸진다.
            // 현재 그룹 Row 의 전환 marker 는 그대로 남긴다.
            if matches!(cmd_lower.as_str(), "rm" | "it") {
                self.font_state = Some(style);
                return EqNode::FontStyle {
                    style,
                    body: Box::new(EqNode::Empty),
                };
            }
            // [#7873] 인자형 글꼴 명령과 다음 글자 사이의 공백 명령(`` ` ``·`~`)은 간격이지
            // body 가 아니다 — 공백을 그대로 두고 그 뒤 원소에 글꼴을 적용한다.
            let mut leading = Vec::new();
            while !self.document_terms
                && self.current_type() == TokenType::Whitespace
                && matches!(self.current_value(), "`" | "~")
            {
                leading.push(super::atoms::whitespace_node(self.current_value()));
                self.pos += 1;
            }
            // 다음 토큰이 구조 명령어(LEFT, RIGHT 등)이면 body 없이 반환
            // bold LEFT(...) 에서 bold 가 LEFT 를 body 로 먹지 않도록
            let body = if self.current_type() == TokenType::Command
                && is_structure_command(&self.current_value().to_ascii_uppercase())
            {
                EqNode::Empty
            } else {
                self.parse_single_or_group()
            };
            let styled = EqNode::FontStyle {
                style,
                body: Box::new(body),
            };
            if leading.is_empty() {
                return styled;
            }
            leading.push(styled);
            return EqNode::Row(leading);
        }

        // Unicode 기호 매핑 — 함수보다 우선 (hwpeq inf=∞ vs LaTeX \inf=infimum 충돌 방지)
        if let Some(symbol) = lookup_symbol(cmd) {
            let node = EqNode::MathSymbol(symbol.to_string());
            return self.finish_command_atom(node, attach_scripts);
        }

        if is_function(cmd) {
            let func_name = lookup_function(cmd).unwrap_or(cmd).to_string();
            // 함수명 뒤 Thin 공백은 종전대로 소비하되, 바로 뒤가 첨자면 남겨서
            // try_parse_scripts 가 폭을 보존하며 결합하게 한다(#5534).
            if attach_scripts
                && self.current_type() == TokenType::Whitespace
                && self.current_value() == "`"
            {
                let after_space_is_script = self.tokens.get(self.pos + 1).is_some_and(|t| {
                    t.ty == TokenType::Subscript || t.ty == TokenType::Superscript
                });
                if !after_space_is_script {
                    self.pos += 1;
                }
            }
            let node = EqNode::Function(func_name);
            return self.finish_command_atom(node, attach_scripts);
        }

        let node = self.styled_literal(EqNode::Text(cmd.to_string()), roman_literal);
        self.finish_command_atom(node, attach_scripts)
    }
}

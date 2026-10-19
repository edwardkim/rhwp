//! 한컴 수식 스크립트 재귀 하강 파서
//!
//! 토큰 리스트를 AST(EqNode)로 변환한다.

use super::ast::*;
use super::dispatch::{classify_command, matrix_style, pile_align, EqCommandClass};
use super::symbols::{
    self, is_big_operator, is_function, is_structure_command, lookup_function, lookup_symbol,
    FontStyleKind, DECORATIONS, FONT_STYLES,
};
use super::tokenizer::{tokenize, Token, TokenType};

#[path = "parser_commands.rs"]
mod commands;

#[path = "parser_fractions.rs"]
mod fractions;

#[path = "parser_structures.rs"]
mod structures;

#[path = "parser_groups.rs"]
mod groups;

#[path = "parser_atoms.rs"]
mod atoms;

#[path = "parser_root.rs"]
mod root;

#[path = "parser_negation.rs"]
mod negation;

#[path = "parser_alignment.rs"]
mod alignment;

#[path = "parser_attached_scripts.rs"]
mod attached_scripts;

#[path = "parser_script_operand.rs"]
mod script_operand;

#[path = "parser_terms.rs"]
mod terms;

#[path = "parser_left_scripts.rs"]
mod left_scripts;

/// 수식 중첩 깊이 상한.
///
/// 적대적으로 깊게 중첩된 입력(`{{{…}}}`, `sqrt sqrt …`, `LEFT ( LEFT ( …`,
/// 중첩 `matrix`/`cases`/`pile`/`eqalign`, 긴 `OVER`/`ATOP` 연쇄)은 재귀 하강
/// 파서를 무한 재귀로 몰거나, 반복으로 쌓인 깊은 `EqNode` 트리의 재귀적
/// `Drop`/layout/svg 에서 스택을 오버플로한다. 형제 파서 가드
/// (`MAX_HWPX_SECTION_DEPTH = 64`, `MAX_HWP5_SHAPE_DEPTH`, `MAX_DRAWING_OBJECT_DEPTH`)와
/// 같은 취지로 상한을 둔다. 실측 디버그 스택 오버플로 임계(중첩 sqrt ≈ 144)보다
/// 충분히 낮고, 실제 수식의 중첩 깊이(수 단계)는 넉넉히 웃돈다. 초과분은 조용히
/// 잘라내(truncate) 유효 수식 출력은 바뀌지 않는다.
pub(super) const MAX_EQ_DEPTH: u32 = 64;

/// 수식 파서
pub struct EqParser {
    tokens: Vec<Token>,
    pos: usize,
    /// 현재 재귀 깊이. 중첩 게이트웨이(`parse_command`/`parse_group`/
    /// `parse_paren_group`)에서 증감하여 무한 재귀·깊은 트리를 막는다.
    depth: u32,
    /// LParen 인덱스 → 매칭 RParen 인덱스 사전계산 결과. `paren_then_script` 의
    /// 매 호출당 O(n) 전방 스캔을 O(1) 조회로 바꿔, 괄호가 많은/깊은 입력에서의
    /// O(n^2) 행(hang)을 없앤다. LParen 이 아니거나 짝이 없으면 `tokens.len()`.
    paren_match: Vec<usize>,
    document_terms: bool,
    term_limit: usize,
    /// 독립 Print의 명령형 첨자 오류는 중첩 그룹/행을 포함한 수식 전체에 적용된다.
    invalid_document_script: bool,
    /// 잘못된 구조 인자 뒤 입력만 중단한다. 이미 읽은 앞 항은 유지한다.
    stopped_document_input: bool,
    /// [#7873] `rm`/`it` 가 켠 라틴 글꼴 상태. 한/글 2020·2024 Print 실측: 두 명령은 다음
    /// 원소 하나가 아니라 그 지점부터 수식 끝까지의 상태 전환이며 `{}`·분수·첨자·
    /// `LEFT…RIGHT`·`sqrt` 경계를 넘어 지속된다(`{rm a} b`·`{rm a} over {b} c`·
    /// `a rm b^{c} d`·`rm a LEFT ( b RIGHT ) c` 가 모두 로만체, `rm a it b c` 는 `b c`
    /// 이탤릭). 글자 식별자가 만들어질 때 이 상태로 감싼다(`styled_literal`). `bold` 는
    /// 다음 원소 하나에만 적용되고 그리스 문자는 영향받지 않는다.
    font_state: Option<FontStyleKind>,
    /// [#7873] `parse_element` 가 마지막으로 완성한 원소의 (시작 토큰 위치, 그 직전의
    /// `font_state`). 분수 결합이 분모를 먼저 평가하는 한/글 순서를 재현할 때 분자를
    /// 다시 읽기 위해 쓴다. 중위 결합이 만든 분수가 마지막 자식이면 `None`.
    last_element: Option<(usize, Option<FontStyleKind>)>,
}

impl EqParser {
    pub fn new(mut tokens: Vec<Token>) -> Self {
        super::brace_recovery::retain_owned_closing_braces(&mut tokens);
        let paren_match = Self::compute_paren_match(&tokens);
        let term_limit = tokens.len();
        Self {
            tokens,
            pos: 0,
            depth: 0,
            paren_match,
            document_terms: false,
            term_limit,
            invalid_document_script: false,
            stopped_document_input: false,
            font_state: None,
            last_element: None,
        }
    }

    /// [#7873] 글자 식별자 노드에 글꼴을 적용한다. 문서 백슬래시 문자 전환
    /// (`roman_literal`)이 우선하고, 그 밖에는 현재 `rm`/`it` 상태로 감싼다.
    fn styled_literal(&self, node: EqNode, roman_literal: bool) -> EqNode {
        if roman_literal {
            return atoms::literal_font(node, true);
        }
        match self.font_state {
            Some(style) => EqNode::FontStyle {
                style,
                body: Box::new(node),
            },
            None => node,
        }
    }

    pub(crate) fn for_document(tokens: Vec<Token>) -> Self {
        Self {
            document_terms: true,
            ..Self::new(tokens)
        }
    }

    /// LParen→RParen 짝을 스택으로 한 번에(O(n)) 계산한다. `paren_then_script` 의
    /// 원래 선형 깊이 스캔과 동일하게 LParen/RParen 만 세고 그 외 토큰(중괄호 등)은
    /// 무시한다. 짝 없는 LParen 은 `tokens.len()` 으로 남는다.
    fn compute_paren_match(tokens: &[Token]) -> Vec<usize> {
        let n = tokens.len();
        let mut m = vec![n; n];
        let mut stack: Vec<usize> = Vec::new();
        for (i, t) in tokens.iter().enumerate() {
            match t.ty {
                TokenType::LParen => stack.push(i),
                TokenType::RParen => {
                    if let Some(open) = stack.pop() {
                        m[open] = i;
                    }
                }
                _ => {}
            }
        }
        m
    }

    fn current(&self) -> Option<&Token> {
        if self.stopped_document_input {
            return None;
        }
        self.tokens.get(self.pos).filter(|t| t.ty != TokenType::Eof)
    }

    fn current_type(&self) -> TokenType {
        if self.stopped_document_input {
            return TokenType::Eof;
        }
        self.tokens
            .get(self.pos)
            .map(|t| t.ty)
            .unwrap_or(TokenType::Eof)
    }

    fn current_value(&self) -> &str {
        self.tokens
            .get(self.pos)
            .map(|t| t.value.as_str())
            .unwrap_or("")
    }

    fn at_end(&self) -> bool {
        self.stopped_document_input
            || self.pos >= self.tokens.len()
            || self.current_type() == TokenType::Eof
    }

    fn advance(&mut self) -> Option<&Token> {
        if self.at_end() {
            return None;
        }
        let tok = &self.tokens[self.pos];
        self.pos += 1;
        Some(tok)
    }

    fn expect(&mut self, ty: TokenType) -> bool {
        if self.current_type() == ty {
            self.pos += 1;
            true
        } else {
            false
        }
    }

    /// 명령어 대소문자 무시 비교
    fn cmd_eq(val: &str, target: &str) -> bool {
        val.eq_ignore_ascii_case(target)
    }

    /// 최상위 레벨에서 OVER가 있는지 확인 (괄호/LEFT-RIGHT 내부 제외)
    fn has_toplevel_over(tokens: &[Token]) -> bool {
        let mut brace_depth = 0i32;
        let mut lr_depth = 0i32;
        for t in tokens {
            match t.ty {
                TokenType::LBrace => brace_depth += 1,
                TokenType::RBrace => brace_depth -= 1,
                TokenType::Command => {
                    if Self::cmd_eq(&t.value, "LEFT") {
                        lr_depth += 1;
                    } else if Self::cmd_eq(&t.value, "RIGHT") {
                        lr_depth -= 1;
                    } else if Self::cmd_eq(&t.value, "OVER") && brace_depth == 0 && lr_depth == 0 {
                        return true;
                    }
                }
                _ => {}
            }
        }
        false
    }

    /// 수식 전체 파싱 (엔트리 포인트)
    pub fn parse(&mut self) -> EqNode {
        let result = self.parse_expression();
        if self.document_terms && self.invalid_document_script {
            EqNode::Empty
        } else {
            result
        }
    }

    /// OVER/ATOP/CHOOSE 중위 연산자 처리. 현재 토큰이 OVER/ATOP 이면 children 의 마지막 요소를
    /// pop 하여 분수/atop 으로 결합한다. 처리했으면 true, 아니면 false.
    /// CASES/PILE/EQALIGN 등 row-collecting 파서가 분수를 인식하지 못하는 결함(#505)을
    /// 방지하기 위해 모든 token-collecting 루프에서 호출한다.
    ///
    /// `over_run` 은 현재 루프의 "연속 OVER/ATOP 결합 횟수"다. `1 over 1 over …`
    /// 같은 긴 연쇄는 반복으로 좌편향 깊은 트리를 쌓아 [`MAX_EQ_DEPTH`] 재귀 가드를
    /// 우회하므로(그 뒤 재귀적 Drop/layout/svg 에서 오버플로), 연쇄 길이를
    /// [`MAX_EQ_DEPTH`] 로 제한한다. 상한 초과 시 결합을 멈춰 false 를 돌려주면 남은
    /// OVER 는 parse_element 가 Empty 로 소비하므로 진행이 보장된다.
    fn try_consume_infix_over_atop(
        &mut self,
        children: &mut Vec<EqNode>,
        over_run: &mut u32,
    ) -> bool {
        if self.current_type() != TokenType::Command {
            *over_run = 0; // 다음은 일반 요소 — 연쇄 종료, 카운터 리셋.
            return false;
        }
        let val = self.current_value();
        let is_over = Self::cmd_eq(val, "OVER");
        let is_atop = Self::cmd_eq(val, "ATOP");
        let is_choose = Self::cmd_eq(val, "CHOOSE");
        if !is_over && !is_atop && !is_choose {
            *over_run = 0; // OVER/ATOP 이 아닌 명령 — 연쇄 종료, 카운터 리셋.
            return false;
        }
        if *over_run >= MAX_EQ_DEPTH {
            // 상한 도달: 결합 중단(리셋하지 않음). 남은 OVER 는 parse_element 가
            // Empty 로 소비하고, 이어지는 일반 요소가 카운터를 리셋한다.
            return false;
        }
        *over_run += 1;
        self.pos += 1;
        let mut top = children.pop().unwrap_or(EqNode::Empty);
        // [#7873] 한/글 Print 실측(`a over {rm b} c` → a·b·c 로만체, `{rm a} over {b} c`
        // → b 만 이탤릭): 분수는 **분모를 먼저** 평가하고 그 뒤 분자, 그 뒤 후속 원소
        // 순으로 `rm`/`it` 상태를 잇는다. 분모는 분자 직전 상태에서 읽고, 분자가 상태를
        // 바꾸지 않았는데 분모가 바꿨으면 분자를 분모 뒤 상태로 다시 읽는다. 분자가
        // 중위 결합의 산물(연쇄 분수)이면 다시 읽지 않는다.
        let numer = self.last_element.take();
        let state_before_numer = numer.map_or(self.font_state, |(_, state)| state);
        let state_after_numer = self.font_state;
        self.font_state = state_before_numer;
        let bottom = self.parse_element();
        let state_after_denom = self.font_state;
        let numer_changed = state_after_numer != state_before_numer;
        if let Some((start, _)) = numer {
            if !numer_changed && state_after_denom != state_before_numer {
                let after_denom = self.pos;
                self.pos = start;
                let reread = self.parse_element();
                // 같은 토큰을 같은 규칙으로 읽었으므로 OVER/ATOP/CHOOSE 앞에서 멈춰야 한다.
                let stopped_at_infix = self.current_type() == TokenType::Command
                    && ["OVER", "ATOP", "CHOOSE"]
                        .iter()
                        .any(|name| Self::cmd_eq(self.current_value(), name));
                if stopped_at_infix {
                    top = reread;
                }
                self.pos = after_denom;
            }
        }
        self.font_state = if numer_changed {
            state_after_numer
        } else {
            state_after_denom
        };
        self.last_element = None;
        children.push(if is_choose {
            EqNode::Binomial {
                top: Box::new(top),
                bottom: Box::new(bottom),
            }
        } else if is_atop {
            EqNode::Atop {
                top: Box::new(top),
                bottom: Box::new(bottom),
            }
        } else {
            EqNode::Fraction {
                numer: Box::new(top),
                denom: Box::new(bottom),
            }
        });
        true
    }

    /// 표현식 파싱 (중단 조건 없이 끝까지)
    /// OVER/ATOP을 중위 연산자로 처리: 바로 앞/뒤 요소를 위아래로 배치
    fn parse_expression(&mut self) -> EqNode {
        let mut children = Vec::new();
        let mut over_run = 0u32;
        while !self.at_end() {
            // 그룹 종료 또는 RIGHT 만나면 중단
            if self.current_type() == TokenType::RBrace {
                break;
            }
            if self.current_type() == TokenType::Command
                && Self::cmd_eq(self.current_value(), "RIGHT")
            {
                break;
            }
            // OVER/ATOP 중위 연산자: 직전/직후 요소를 위아래로 결합
            if self.try_consume_infix_over_atop(&mut children, &mut over_run) {
                continue;
            }
            children.push(self.parse_element());
        }
        EqNode::Row(children).simplify()
    }

    fn finish_command_atom(&mut self, node: EqNode, attach_scripts: bool) -> EqNode {
        if attach_scripts {
            self.try_parse_scripts(node)
        } else {
            node
        }
    }

    /// 큰 연산자 파싱 (적분, 합 등) — 첨자 포함
    fn parse_big_op(&mut self, symbol: String) -> EqNode {
        let (sub, sup) = self.parse_operator_scripts();
        EqNode::BigOp { symbol, sub, sup }
    }

    /// 극한 파싱
    fn parse_limit(&mut self, is_upper: bool) -> EqNode {
        let mut sub = None;

        if self.current_type() == TokenType::Subscript {
            self.pos += 1;
            sub = Some(Box::new(self.parse_script_operand(false)));
        }

        EqNode::Limit { is_upper, sub }
    }

    /// LEFT-RIGHT 괄호 파싱
    /// 내부의 OVER는 parse_expression의 중위 연산자 처리로 자동 처리된다.
    fn parse_left_right(&mut self) -> EqNode {
        // LEFT 다음 괄호 문자 읽기
        let left = self.read_bracket_char();

        // RIGHT까지의 내용을 parse_expression으로 파싱
        // parse_expression은 RIGHT를 만나면 자동 중단하고, OVER도 중위 연산자로 처리
        let body = self.parse_expression();

        // RIGHT 건너뛰기
        if self.current_type() == TokenType::Command && Self::cmd_eq(self.current_value(), "RIGHT")
        {
            self.pos += 1;
        }

        // RIGHT 다음 괄호 문자 읽기
        let right = self.read_bracket_char();

        EqNode::Paren {
            left,
            right,
            body: Box::new(body),
        }
    }

    /// 괄호 문자 읽기 (LEFT/RIGHT 뒤)
    fn read_bracket_char(&mut self) -> String {
        if self.at_end() {
            return String::new();
        }

        let ty = self.current_type();
        let val = self.current_value().to_string();

        match ty {
            TokenType::LParen
            | TokenType::RParen
            | TokenType::LBracket
            | TokenType::RBracket
            | TokenType::LBrace
            | TokenType::RBrace => {
                self.pos += 1;
                val
            }
            TokenType::Symbol if val == "|" || val == "." => {
                self.pos += 1;
                if val == "." {
                    String::new() // . = 괄호 생략
                } else {
                    val
                }
            }
            TokenType::Command => {
                // LBRACE, RBRACE 등 명령어
                if let Some(sym) = lookup_symbol(&val) {
                    self.pos += 1;
                    sym.to_string()
                } else {
                    String::new()
                }
            }
            _ => String::new(),
        }
    }

    /// RIGHT 위치 찾기 (LEFT-RIGHT 쌍 고려)
    fn find_right_pos(&self) -> usize {
        let mut depth = 1i32;
        let mut pos = self.pos;
        while pos < self.tokens.len() {
            let t = &self.tokens[pos];
            if t.ty == TokenType::Command {
                if Self::cmd_eq(&t.value, "LEFT") {
                    depth += 1;
                } else if Self::cmd_eq(&t.value, "RIGHT") {
                    depth -= 1;
                    if depth == 0 {
                        return pos;
                    }
                }
            }
            pos += 1;
        }
        self.tokens.len()
    }

    /// 범위 내 분수 파싱
    /// OVER 앞/뒤에 중괄호 그룹이 있으면 해당 그룹만 분자/분모로 사용하고
    /// 나머지는 분수 바깥 요소로 처리한다.
    fn parse_fraction_in_range(&mut self, end: usize) -> EqNode {
        // OVER 앞의 모든 요소를 파싱
        let mut before_nodes = Vec::new();
        while self.pos < end && !self.at_end() {
            if self.current_type() == TokenType::Command
                && Self::cmd_eq(self.current_value(), "OVER")
            {
                self.pos += 1;
                break;
            }
            before_nodes.push(self.parse_element());
        }

        // 분자: OVER 바로 앞의 마지막 요소 (또는 그룹)
        // 나머지 앞 요소들은 분수 앞에 배치
        let (pre_nodes, numer) = if before_nodes.len() > 1 {
            let numer = before_nodes.pop().unwrap();
            (before_nodes, numer)
        } else {
            (Vec::new(), EqNode::Row(before_nodes).simplify())
        };

        // 분모: OVER 바로 뒤의 첫 번째 요소 (또는 그룹)
        let denom = if self.pos < end && !self.at_end() {
            self.parse_element()
        } else {
            EqNode::Empty
        };

        // 분수 뒤 나머지 요소
        let mut after_nodes = Vec::new();
        while self.pos < end && !self.at_end() {
            if self.current_type() == TokenType::Command
                && Self::cmd_eq(self.current_value(), "RIGHT")
            {
                break;
            }
            after_nodes.push(self.parse_element());
        }

        let fraction = EqNode::Fraction {
            numer: Box::new(numer),
            denom: Box::new(denom),
        };

        // 앞/뒤 요소와 분수를 Row로 조립
        if pre_nodes.is_empty() && after_nodes.is_empty() {
            fraction
        } else {
            let mut all = pre_nodes;
            all.push(fraction);
            all.extend(after_nodes);
            EqNode::Row(all).simplify()
        }
    }

    /// COLOR의 색 인자와 본체를 각각 소비한다. 비 RGB 색 인자는 한컴처럼 무시한다.
    fn parse_color(&mut self) -> EqNode {
        if !self.expect(TokenType::LBrace) {
            // 독립 Print는 이름/미지원 색 인자를 표시하지 않고 본체를 검정으로 그린다.
            let _color_argument = self.parse_element();
            return self.parse_element();
        }

        // R, G, B 값 읽기
        let mut rgb = [0u8; 3];
        for i in 0..3 {
            if self.current_type() == TokenType::Number {
                rgb[i] = self.current_value().parse().unwrap_or(0);
                self.pos += 1;
            }
            // 콤마 건너뛰기
            if self.current_type() == TokenType::Symbol && self.current_value() == "," {
                self.pos += 1;
            }
        }
        self.expect(TokenType::RBrace);

        let body = self.parse_single_or_group();

        EqNode::Color {
            r: rgb[0],
            g: rgb[1],
            b: rgb[2],
            body: Box::new(body),
        }
    }
}

/// 수식 스크립트를 AST로 파싱
pub fn parse(script: &str) -> EqNode {
    let tokens = tokenize(script);
    let mut parser = EqParser::new(tokens);
    parser.parse()
}

#[cfg(test)]
mod tests {
    use super::symbols::{DecoKind, FontStyleKind};
    use super::*;

    /// [PR #1226] LEFT-RIGHT 구분기호 그룹 뒤 첨자(^/_)가 그룹 전체에 결합돼야 한다.
    /// 기존엔 LEFT 분기가 try_parse_scripts 를 안 거쳐 `|x|^3` 의 ^3 가 base 없는
    /// orphan Superscript{base:Empty} 가 됐다(3 이 superscript 높이로 안 올라감).
    #[test]
    fn left_right_group_binds_trailing_script() {
        // |x|^3 → Superscript{ base: Paren, sup: 3 } (orphan 아님)
        let ast = parse("left | x right | ^3");
        let s = format!("{:?}", ast);
        assert!(
            s.contains("Superscript") && s.contains("Paren") && !s.contains("base: Empty"),
            "|x|^3 의 ^3 가 Paren 그룹에 결합돼야 함(orphan 금지): {s}"
        );

        // |x|_3 → Subscript{ base: Paren, sub: 3 }
        let sub = format!("{:?}", parse("left | x right | _3"));
        assert!(
            sub.contains("Subscript") && sub.contains("Paren") && !sub.contains("base: Empty"),
            "|x|_3 의 _3 가 Paren 그룹에 결합돼야 함: {sub}"
        );

        // 회귀 가드: x^2 는 영향 없음
        let x2 = format!("{:?}", parse("x^2"));
        assert!(x2.contains("Superscript"), "x^2 정상: {x2}");
    }

    #[test]
    fn test_simple_fraction() {
        let ast = parse("1 over 2");
        match &ast {
            EqNode::Fraction { numer, denom } => {
                assert!(matches!(numer.as_ref(), EqNode::Number(n) if n == "1"));
                assert!(matches!(denom.as_ref(), EqNode::Number(n) if n == "2"));
            }
            _ => panic!("Expected Fraction, got {:?}", ast),
        }
    }

    #[test]
    fn test_task1122_over_followed_by_number_parses_as_fraction() {
        for (script, expected_numer, expected_denom) in [
            ("11 over20", "11", "20"),
            ("3 over5", "3", "5"),
            ("7 OVER10", "7", "10"),
            ("{8} over {13}", "8", "13"),
        ] {
            let ast = parse(script);
            match &ast {
                EqNode::Fraction { numer, denom } => {
                    assert!(
                        matches!(numer.as_ref(), EqNode::Number(n) if n == expected_numer),
                        "unexpected numerator for {script:?}: {ast:?}"
                    );
                    assert!(
                        matches!(denom.as_ref(), EqNode::Number(n) if n == expected_denom),
                        "unexpected denominator for {script:?}: {ast:?}"
                    );
                }
                _ => panic!("Expected Fraction for {script:?}, got {:?}", ast),
            }
        }
    }

    #[test]
    fn test_task1122_over_prefix_non_numeric_identifiers_parse_as_text() {
        let ast = parse("overlap");
        assert!(matches!(ast, EqNode::Text(ref t) if t == "overlap"));

        let ast = parse(r"\overline{AB}");
        assert!(
            matches!(
                ast,
                EqNode::Decoration {
                    kind: DecoKind::Overline,
                    ..
                }
            ),
            r"Expected Decoration(Overline), got {:?}",
            ast
        );
    }

    #[test]
    fn test_atop() {
        let ast = parse("a atop b");
        match &ast {
            EqNode::Atop { top, bottom } => {
                assert!(matches!(top.as_ref(), EqNode::Text(t) if t == "a"));
                assert!(matches!(bottom.as_ref(), EqNode::Text(t) if t == "b"));
            }
            _ => panic!("Expected Atop, got {:?}", ast),
        }
    }

    #[test]
    fn test_superscript() {
        let ast = parse("E=mc^2");
        // E = mc^2 → Row([Text("E"), Symbol("="), Superscript(Text("mc"), Number("2"))])
        match &ast {
            EqNode::Row(children) => {
                assert!(children.len() >= 3);
                assert!(matches!(&children[2], EqNode::Superscript { .. }));
            }
            _ => panic!("Expected Row, got {:?}", ast),
        }
    }

    #[test]
    fn test_sqrt() {
        let ast = parse("SQRT x");
        match &ast {
            EqNode::Sqrt { index, body } => {
                assert!(index.is_none());
                assert!(matches!(body.as_ref(), EqNode::Text(t) if t == "x"));
            }
            _ => panic!("Expected Sqrt, got {:?}", ast),
        }
    }

    #[test]
    fn test_sqrt_with_index() {
        let ast = parse("SQRT(3) of x");
        match &ast {
            EqNode::Sqrt { index, body } => {
                assert!(index.is_some());
                assert!(matches!(body.as_ref(), EqNode::Text(t) if t == "x"));
            }
            _ => panic!("Expected Sqrt with index, got {:?}", ast),
        }
    }

    #[test]
    fn test_greek() {
        let ast = parse("alpha + beta");
        match &ast {
            EqNode::Row(children) => {
                assert!(matches!(&children[0], EqNode::MathSymbol(s) if s == "α"));
                assert!(matches!(&children[1], EqNode::Symbol(s) if s == "+"));
                assert!(matches!(&children[2], EqNode::MathSymbol(s) if s == "β"));
            }
            _ => panic!("Expected Row, got {:?}", ast),
        }
    }

    #[test]
    fn test_integral() {
        // 적분은 nolimits: MathSymbol + SubSup (일반 첨자)
        let ast = parse("INT_0^{inf}");
        match &ast {
            EqNode::SubSup { base, sub, sup } => {
                assert!(matches!(base.as_ref(), EqNode::MathSymbol(s) if s == "∫"));
            }
            _ => panic!("Expected SubSup, got {:?}", ast),
        }
    }

    #[test]
    fn test_sum() {
        let ast = parse("SUM_{i=0}^n");
        match &ast {
            EqNode::BigOp { symbol, sub, sup } => {
                assert_eq!(symbol, "∑");
                assert!(sub.is_some());
                assert!(sup.is_some());
            }
            _ => panic!("Expected BigOp, got {:?}", ast),
        }
    }

    #[test]
    fn test_limit() {
        let ast = parse("lim_{x->0}");
        match &ast {
            EqNode::Limit { is_upper, sub } => {
                assert!(!is_upper);
                assert!(sub.is_some());
            }
            _ => panic!("Expected Limit, got {:?}", ast),
        }
    }

    #[test]
    fn test_matrix() {
        let ast = parse("matrix{a & b # c & d}");
        match &ast {
            EqNode::Matrix { rows, style } => {
                assert_eq!(*style, MatrixStyle::Plain);
                assert_eq!(rows.len(), 2);
                assert_eq!(rows[0].len(), 2);
                assert_eq!(rows[1].len(), 2);
            }
            _ => panic!("Expected Matrix, got {:?}", ast),
        }
    }

    #[test]
    fn test_left_right() {
        let ast = parse("LEFT ( a over b RIGHT )");
        match &ast {
            EqNode::Paren { left, right, body } => {
                assert_eq!(left, "(");
                assert_eq!(right, ")");
                assert!(matches!(body.as_ref(), EqNode::Fraction { .. }));
            }
            _ => panic!("Expected Paren, got {:?}", ast),
        }
    }

    #[test]
    fn test_decoration() {
        let ast = parse("hat x");
        match &ast {
            EqNode::Decoration { kind, body } => {
                assert_eq!(*kind, DecoKind::Hat);
                assert!(matches!(body.as_ref(), EqNode::Text(t) if t == "x"));
            }
            _ => panic!("Expected Decoration, got {:?}", ast),
        }
    }

    #[test]
    fn test_font_style() {
        let ast = parse("rm abc");
        match &ast {
            EqNode::FontStyle { style, body } => {
                assert_eq!(*style, FontStyleKind::Roman);
            }
            _ => panic!("Expected FontStyle, got {:?}", ast),
        }
    }

    // [#1204-B] 글꼴 명령(rm) body 로 온 decoration(bar)이 Text 로 leak 되지 않고
    // Decoration 으로 파싱되어야 한다.
    #[test]
    fn test_font_style_body_decoration_not_leaked() {
        let ast = parse("rm bar {F prime F}");
        let s = format!("{:?}", ast);
        assert!(
            s.contains("Decoration"),
            "bar 가 Decoration 으로 파싱돼야 함: {s}"
        );
        assert!(
            !s.contains(r#"Text("bar")"#),
            "bar 가 텍스트로 leak 되면 안 됨: {s}"
        );
    }

    // [#1204] 대문자 글꼴/장식 명령(RM/BAR)도 대소문자 무시로 인식 (leak 방지).
    #[test]
    fn test_uppercase_font_and_deco_commands() {
        let s = format!("{:?}", parse("RM {vec{EC}}"));
        assert!(!s.contains(r#"Text("RM")"#), "RM 이 leak 되면 안 됨: {s}");
        assert!(
            s.contains("FontStyle"),
            "RM 이 FontStyle 로 인식돼야 함: {s}"
        );
        let s2 = format!("{:?}", parse("BAR {AB}"));
        assert!(
            s2.contains("Decoration"),
            "BAR 가 Decoration 으로 인식돼야 함: {s2}"
        );
    }

    // [#1204-A] root3 (분리 후 root + 3) 이 Sqrt 로 파싱되어야 한다.
    #[test]
    fn test_root_glued_digit_parses_as_sqrt() {
        let ast = parse("root3 y");
        let s = format!("{:?}", ast);
        assert!(s.contains("Sqrt"), "root3 이 Sqrt 로 파싱돼야 함: {s}");
        assert!(
            !s.contains(r#"Text("root3")"#),
            "root3 이 텍스트로 leak 되면 안 됨: {s}"
        );
    }

    #[test]
    fn test_cases() {
        let ast = parse("CASES{ 1 & x>0 # -1 & x<0 }");
        match &ast {
            EqNode::Cases { rows } => {
                assert_eq!(rows.len(), 2);
            }
            _ => panic!("Expected Cases, got {:?}", ast),
        }
    }

    #[test]
    fn test_sample_eq01_script() {
        // samples/eq-01.hwp의 첫 번째 수식
        let script =
            "평점=입찰가격평가~배점한도 TIMES  LEFT ( {최저입찰가격} over {해당입찰가격} RIGHT )";
        let ast = parse(script);
        // 파싱 실패 없이 AST 생성되면 성공
        match &ast {
            EqNode::Row(children) => {
                assert!(children.len() > 1);
                // TIMES, LEFT-RIGHT 구조 확인
                let has_paren = children.iter().any(|c| matches!(c, EqNode::Paren { .. }));
                assert!(has_paren, "Should contain Paren node");
            }
            _ => {} // 단일 노드도 허용
        }
    }

    #[test]
    fn test_cos_fraction_with_left_right() {
        // cos`left({pi} over {2}+theta right)=`-{1} over {5}`
        // OVER는 바로 앞/뒤 그룹만 분수로 만든다:
        //   LEFT-RIGHT 안: {pi} over {2} → Fraction{π,2}, +θ는 분수 밖
        //   최상위: {1} over {5} → Fraction{1,5}, cos(...)=-는 분수 밖
        let script = " cos ` left({ pi} over {2}+ theta  right)=`-{1} over {5}`";
        let ast = parse(script);
        eprintln!("AST: {:#?}", ast);
        // 최상위는 Row: [cos, Paren{π/2+θ}, =, -, Fraction{1,5}]
        let ast_str = format!("{:?}", ast);
        assert!(ast_str.contains("cos"), "cos가 있어야 함");
        assert!(ast_str.contains("Paren"), "Paren이 있어야 함");
        // Fraction{1,5}가 독립적으로 존재해야 함
        assert!(
            ast_str.contains("Fraction { numer: Number(\"1\"), denom: Number(\"5\")"),
            "Fraction{{1,5}}가 있어야 함: {}",
            ast_str
        );
    }

    #[test]
    fn test_latex_frac() {
        let ast = parse(r"\frac{1}{2}");
        match &ast {
            EqNode::Fraction { numer, denom } => {
                assert!(matches!(numer.as_ref(), EqNode::Number(n) if n == "1"));
                assert!(matches!(denom.as_ref(), EqNode::Number(n) if n == "2"));
            }
            _ => panic!("Expected Fraction, got {:?}", ast),
        }
    }

    #[test]
    fn test_latex_quadratic_slice() {
        let ast = parse(r"x=\frac{-b \pm \sqrt{b^2}}{2a}");
        let ast_str = format!("{:?}", ast);
        assert!(
            ast_str.contains("Fraction"),
            "Fraction이 있어야 함: {}",
            ast_str
        );
        assert!(
            ast_str.contains("MathSymbol(\"±\")"),
            "± 기호가 있어야 함: {}",
            ast_str
        );
        assert!(ast_str.contains("Sqrt"), "Sqrt가 있어야 함: {}", ast_str);
    }

    #[test]
    fn test_latex_sqrt_with_bracket_index() {
        let ast = parse(r"\sqrt[3]{x}");
        match &ast {
            EqNode::Sqrt { index, body } => {
                assert!(index.is_some(), "sqrt index가 있어야 함");
                assert!(matches!(body.as_ref(), EqNode::Text(t) if t == "x"));
            }
            _ => panic!("Expected indexed Sqrt, got {:?}", ast),
        }
    }
}

#[cfg(test)]
#[test]
fn test_lim_fraction() {
    let script = " lim _{h ``rarrow`` 0} {f left(2+h  right)-f left(2  right)} over {h}`";
    let ast = parse(script);
    eprintln!("LIM AST: {:#?}", ast);
    let ast_str = format!("{:?}", ast);
    // lim_{h→0} 가 있어야 함
    assert!(ast_str.contains("Limit"), "Limit가 있어야 함: {}", ast_str);
    // Fraction이 있어야 함
    assert!(
        ast_str.contains("Fraction"),
        "Fraction이 있어야 함: {}",
        ast_str
    );
}

#[cfg(test)]
#[test]
fn test_bar_rm_it() {
    let script = "bar {{rm{AB}} it }< bar {{rm{AC}} it }`";
    let ast = parse(script);
    eprintln!("BAR AST: {:#?}", ast);
    let ast_str = format!("{:?}", ast);
    assert!(ast_str.contains("Decoration"), "Decoration이 있어야 함");
    // }} 가 텍스트로 나오면 안 됨
    assert!(
        !ast_str.contains(r#"Text("}")"#),
        "brace가 텍스트로 나오면 안 됨"
    );
}

#[cfg(test)]
#[test]
fn test_cases_double_amp() {
    let script = "{cases{eqalign{``x^{3}#}&&eqalign{~LEFT(x LEQ 0 RIGHT)#}#``f LEFT(x RIGHT)&&~LEFT(x>0 RIGHT)}}";
    let ast = parse(script);
    eprintln!("CASES AST: {:#?}", ast);
    let s = format!("{:?}", ast);
    assert!(s.contains("Tab"), "Tab이 있어야 함: {}", s);
}

#[cfg(test)]
#[test]
fn test_rm_p_left() {
    let script = "{rm{P}} it  left(A``|` B` right)";
    let ast = parse(script);
    eprintln!("RM_P AST: {:#?}", ast);
    let s = format!("{:?}", ast);
    assert!(s.contains("Paren"), "Paren이 있어야 함: {}", s);
}

// LaTeX 명령어 호환 확장 테스트 (#143 2차)

#[cfg(test)]
#[path = "parser_latex_compat_tests.rs"]
mod latex_compat_tests;

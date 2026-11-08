//! 기존 파서 상태를 공유하며 입력 소유 범위 안에서 처리한다.

use super::*;

impl EqParser {
    /// 분수 파싱: 최상위 OVER 기준으로 분자/분모 분리
    /// LEFT-RIGHT 내부의 OVER는 무시하고 최상위 레벨의 OVER만 분수 분기점으로 사용한다.
    pub(super) fn parse_fraction(&mut self) -> EqNode {
        // 최상위 OVER 위치를 먼저 찾는다 (brace_depth==0 && lr_depth==0)
        let toplevel_over_pos = {
            let mut brace_depth = 0i32;
            let mut lr_depth = 0i32;
            let mut found = None;
            for i in self.pos..self.tokens.len() {
                let t = &self.tokens[i];
                match t.ty {
                    TokenType::LBrace => brace_depth += 1,
                    TokenType::RBrace => brace_depth -= 1,
                    TokenType::Command => {
                        if Self::cmd_eq(&t.value, "LEFT") {
                            lr_depth += 1;
                        } else if Self::cmd_eq(&t.value, "RIGHT") {
                            lr_depth -= 1;
                        } else if Self::cmd_eq(&t.value, "OVER")
                            && brace_depth == 0
                            && lr_depth == 0
                        {
                            found = Some(i);
                            break;
                        }
                    }
                    _ => {}
                }
            }
            found
        };

        let over_pos = match toplevel_over_pos {
            Some(p) => p,
            None => return self.parse_expression(), // fallback
        };

        // OVER 앞의 모든 요소를 파싱
        let mut before_nodes = Vec::new();
        while self.pos < over_pos && !self.at_end() {
            before_nodes.push(self.parse_element());
        }
        // OVER 건너뛰기
        if self.current_type() == TokenType::Command && Self::cmd_eq(self.current_value(), "OVER") {
            self.pos += 1;
        }

        // 분자: OVER 바로 앞의 마지막 요소 (그룹 또는 단일 요소)
        let (pre_nodes, numer) = if before_nodes.len() > 1 {
            let numer = before_nodes.pop().unwrap();
            (before_nodes, numer)
        } else {
            (Vec::new(), EqNode::Row(before_nodes).simplify())
        };

        // 분모: OVER 바로 뒤의 첫 번째 요소 (그룹 또는 단일 요소)
        let denom = if !self.at_end() {
            self.parse_element()
        } else {
            EqNode::Empty
        };

        // 분수 뒤 나머지 요소
        let mut after_nodes = Vec::new();
        while !self.at_end() {
            if self.current_type() == TokenType::RBrace {
                break;
            }
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

    /// RBrace까지 분수 파싱
    pub(super) fn parse_fraction_until_rbrace(&mut self) -> EqNode {
        let mut numer_nodes = Vec::new();
        while !self.at_end() {
            if self.current_type() == TokenType::RBrace {
                break;
            }
            if self.current_type() == TokenType::Command
                && (Self::cmd_eq(self.current_value(), "OVER"))
            {
                self.pos += 1;
                break;
            }
            numer_nodes.push(self.parse_element());
        }

        let mut denom_nodes = Vec::new();
        while !self.at_end() {
            if self.current_type() == TokenType::RBrace {
                break;
            }
            denom_nodes.push(self.parse_element());
        }

        EqNode::Fraction {
            numer: Box::new(EqNode::Row(numer_nodes).simplify()),
            denom: Box::new(EqNode::Row(denom_nodes).simplify()),
        }
    }

    /// LaTeX 분수 파싱: \frac{numer}{denom}
    pub(super) fn parse_latex_fraction(&mut self) -> EqNode {
        let numer = self.parse_single_or_group();
        let denom = self.parse_single_or_group();
        EqNode::Fraction {
            numer: Box::new(numer),
            denom: Box::new(denom),
        }
    }

    /// \begin{env}...\end{env} 환경 파싱
    pub(super) fn parse_latex_environment(&mut self) -> EqNode {
        let env_name = self.read_brace_arg();
        let env_upper = env_name.to_ascii_uppercase();

        match env_upper.as_str() {
            "PMATRIX" => self.parse_latex_env_matrix(MatrixStyle::Paren, &env_name),
            "BMATRIX" => self.parse_latex_env_matrix(MatrixStyle::Bracket, &env_name),
            "VMATRIX" => self.parse_latex_env_matrix(MatrixStyle::Vert, &env_name),
            "BVMATRIX" => self.parse_latex_env_matrix(MatrixStyle::Vert, &env_name),
            "SMALLMATRIX" => self.parse_latex_env_matrix(MatrixStyle::Plain, &env_name),
            "MATRIX" | "ARRAY" => self.parse_latex_env_matrix(MatrixStyle::Plain, &env_name),
            "CASES" => self.parse_latex_env_cases(&env_name),
            "ALIGNED" | "ALIGN" | "EQNARRAY" | "SPLIT" | "GATHER" | "GATHERED" => {
                self.parse_latex_env_eqalign(&env_name)
            }
            _ => EqNode::Empty,
        }
    }

    /// {env_name} 읽기 — 중괄호 안의 텍스트를 반환
    pub(super) fn read_brace_arg(&mut self) -> String {
        if self.current_type() != TokenType::LBrace {
            return String::new();
        }
        self.pos += 1;
        let mut name = String::new();
        while !self.at_end() && self.current_type() != TokenType::RBrace {
            name.push_str(self.current_value());
            self.pos += 1;
        }
        self.expect(TokenType::RBrace);
        name
    }

    /// \end{env} 의 {env} 인자를 소비하고 건너뛰기
    pub(super) fn skip_brace_arg(&mut self) {
        if self.current_type() == TokenType::LBrace {
            self.pos += 1;
            while !self.at_end() && self.current_type() != TokenType::RBrace {
                self.pos += 1;
            }
            self.expect(TokenType::RBrace);
        }
    }

    /// \end{env} 도달 여부 확인
    pub(super) fn at_latex_env_end(&self, env_name: &str) -> bool {
        if self.current_type() != TokenType::Command {
            return false;
        }
        let val = self.current_value();
        if !val.eq_ignore_ascii_case("end") {
            return false;
        }
        let next = self.pos + 1;
        if next >= self.tokens.len() || self.tokens[next].ty != TokenType::LBrace {
            return false;
        }
        let mut i = next + 1;
        let mut name = String::new();
        while i < self.tokens.len() && self.tokens[i].ty != TokenType::RBrace {
            name.push_str(&self.tokens[i].value);
            i += 1;
        }
        name.eq_ignore_ascii_case(env_name)
    }

    /// \end{env}를 소비 (command + {env_name})
    pub(super) fn consume_latex_env_end(&mut self) {
        if self.current_type() == TokenType::Command
            && self.current_value().eq_ignore_ascii_case("end")
        {
            self.pos += 1;
            self.skip_brace_arg();
        }
    }

    /// LaTeX matrix 환경 파싱: \begin{pmatrix} a & b \\ c & d \end{pmatrix}
    pub(super) fn parse_latex_env_matrix(&mut self, style: MatrixStyle, env_name: &str) -> EqNode {
        let mut rows: Vec<Vec<EqNode>> = vec![vec![]];
        let mut current_cell = Vec::new();
        let mut over_run = 0u32;

        while !self.at_end() && !self.at_latex_env_end(env_name) {
            if self.current_type() == TokenType::Whitespace && self.current_value() == "#" {
                self.pos += 1;
                if let Some(last_row) = rows.last_mut() {
                    last_row.push(EqNode::Row(current_cell).simplify());
                }
                current_cell = Vec::new();
                rows.push(vec![]);
            } else if self.current_type() == TokenType::Whitespace && self.current_value() == "&" {
                if let Some(last_row) = rows.last_mut() {
                    last_row.push(EqNode::Row(current_cell).simplify());
                }
                current_cell = Vec::new();
                self.pos += 1;
            } else if self.try_consume_infix_over_atop(&mut current_cell, &mut over_run) {
                continue;
            } else {
                current_cell.push(self.parse_element());
            }
        }

        if !current_cell.is_empty() || rows.last().map_or(false, |r| !r.is_empty()) {
            if let Some(last_row) = rows.last_mut() {
                last_row.push(EqNode::Row(current_cell).simplify());
            }
        }
        // 빈 마지막 행 제거 (후행 \\ 대응)
        if rows.last().map_or(false, |r| r.is_empty()) {
            rows.pop();
        }

        self.consume_latex_env_end();
        EqNode::Matrix { rows, style }
    }

    /// LaTeX cases 환경 파싱: \begin{cases} expr & cond \\ ... \end{cases}
    pub(super) fn parse_latex_env_cases(&mut self, env_name: &str) -> EqNode {
        let mut case_rows = Vec::new();
        let mut current_row = Vec::new();
        let mut over_run = 0u32;

        while !self.at_end() && !self.at_latex_env_end(env_name) {
            if self.current_type() == TokenType::Whitespace && self.current_value() == "#" {
                self.pos += 1;
                case_rows.push(EqNode::Row(current_row).simplify());
                current_row = Vec::new();
            } else if self.current_type() == TokenType::Whitespace && self.current_value() == "&" {
                current_row.push(EqNode::Space(SpaceKind::Tab));
                self.pos += 1;
            } else if self.try_consume_infix_over_atop(&mut current_row, &mut over_run) {
                continue;
            } else {
                current_row.push(self.parse_element());
            }
        }

        if !current_row.is_empty() {
            case_rows.push(EqNode::Row(current_row).simplify());
        }

        self.consume_latex_env_end();
        EqNode::Cases { rows: case_rows }
    }

    /// LaTeX aligned/align 환경 파싱
    pub(super) fn parse_latex_env_eqalign(&mut self, env_name: &str) -> EqNode {
        let mut alignment_marks = Vec::new();
        let mut eq_rows: Vec<(EqNode, EqNode)> = Vec::new();
        let mut current_left = Vec::new();
        let mut current_right: Option<Vec<EqNode>> = None;
        let mut over_run = 0u32;

        while !self.at_end() && !self.at_latex_env_end(env_name) {
            if self.current_type() == TokenType::Whitespace && self.current_value() == "#" {
                self.pos += 1;
                alignment_marks.push(current_right.is_some());
                let left = EqNode::Row(current_left).simplify();
                let right = current_right
                    .map(|r| EqNode::Row(r).simplify())
                    .unwrap_or(EqNode::Empty);
                eq_rows.push((left, right));
                current_left = Vec::new();
                current_right = None;
            } else if self.current_type() == TokenType::Whitespace && self.current_value() == "&" {
                if current_right.is_none() {
                    current_right = Some(Vec::new());
                }
                self.pos += 1;
            } else {
                let consumed = if let Some(ref mut right) = current_right {
                    self.try_consume_infix_over_atop(right, &mut over_run)
                } else {
                    self.try_consume_infix_over_atop(&mut current_left, &mut over_run)
                };
                if consumed {
                    continue;
                }
                if let Some(ref mut right) = current_right {
                    right.push(self.parse_element());
                } else {
                    current_left.push(self.parse_element());
                }
            }
        }

        if !current_left.is_empty() || current_right.is_some() {
            alignment_marks.push(current_right.is_some());
            let left = EqNode::Row(current_left).simplify();
            let right = current_right
                .map(|r| EqNode::Row(r).simplify())
                .unwrap_or(EqNode::Empty);
            eq_rows.push((left, right));
        }

        self.consume_latex_env_end();
        EqNode::EqAlign {
            rows: eq_rows,
            alignment_marks,
        }
    }
}

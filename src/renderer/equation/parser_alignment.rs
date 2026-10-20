//! 조건식과 정렬 묶음의 명시적 행 구분을 보존한다.

use super::{EqNode, EqParser, TokenType};

impl EqParser {
    /// 조건식 파싱: CASES{...}
    pub(super) fn parse_cases(&mut self) -> EqNode {
        let Some(end) = self.begin_structure_body(false) else {
            return EqNode::Cases {
                rows: vec![EqNode::Empty],
            };
        };
        let mut rows = Vec::new();
        let mut current_row = Vec::new();
        let mut over_run = 0u32;

        while self.pos < end && !self.at_end() {
            if self.current_type() == TokenType::RBrace {
                break;
            }
            if self.current_type() == TokenType::Whitespace && self.current_value() == "#" {
                rows.push(EqNode::Row(current_row).simplify());
                current_row = Vec::new();
                self.pos += 1;
            } else if self.current_type() == TokenType::Whitespace && self.current_value() == "&" {
                // && (연속 &): 큰 탭 공간으로 조건 부분 분리
                let mut amp_count = 0;
                while self.pos < end
                    && self.current_type() == TokenType::Whitespace
                    && self.current_value() == "&"
                {
                    amp_count += 1;
                    self.pos += 1;
                }
                for _ in 0..amp_count {
                    current_row.push(EqNode::Space(super::SpaceKind::Tab));
                }
            } else if self.try_consume_infix_over_atop(&mut current_row, &mut over_run) {
                // OVER/ATOP 중위 처리 (#505)
                continue;
            } else {
                current_row.push(self.parse_element());
            }
        }

        // 명시적 빈 템플릿도 한 논리 행을 예약한다.
        rows.push(EqNode::Row(current_row).simplify());

        self.expect(TokenType::RBrace);

        EqNode::Cases { rows }
    }

    /// EQALIGN 파싱: EQALIGN{row1_left & row1_right # row2_left & row2_right}
    pub(super) fn parse_eqalign(&mut self) -> EqNode {
        let Some(end) = self.begin_structure_body(false) else {
            return EqNode::EqAlign {
                rows: vec![(EqNode::Empty, EqNode::Empty)],
                alignment_marks: vec![false],
            };
        };
        let mut rows: Vec<(EqNode, EqNode)> = Vec::new();
        let mut alignment_marks = Vec::new();
        let mut current_left = Vec::new();
        let mut current_right: Option<Vec<EqNode>> = None;
        let mut over_run = 0u32;

        while self.pos < end && !self.at_end() {
            if self.current_type() == TokenType::RBrace {
                break;
            }
            if self.current_type() == TokenType::Whitespace && self.current_value() == "#" {
                // 새 행: 현재 행 완료
                alignment_marks.push(current_right.is_some());
                let left = EqNode::Row(current_left).simplify();
                let right = current_right
                    .map(|r| EqNode::Row(r).simplify())
                    .unwrap_or(EqNode::Empty);
                rows.push((left, right));
                current_left = Vec::new();
                current_right = None;
                self.pos += 1;
            } else if self.current_type() == TokenType::Whitespace && self.current_value() == "&" {
                // & 구분: 왼쪽→오른쪽 전환
                // 연속 &&: 큰 탭 공간 (조건 부분 분리용)
                let mut amp_count = 0;
                while self.pos < end
                    && self.current_type() == TokenType::Whitespace
                    && self.current_value() == "&"
                {
                    amp_count += 1;
                    self.pos += 1;
                }
                if current_right.is_none() {
                    current_right = Some(Vec::new());
                    // 연속 && 이면 큰 탭 공간 삽입
                    if amp_count >= 2 {
                        if let Some(ref mut right) = current_right {
                            right.push(EqNode::Space(super::SpaceKind::Tab));
                        }
                    }
                } else if let Some(ref mut right) = current_right {
                    // 이미 오른쪽: 추가 & → 탭 공간
                    for _ in 0..amp_count {
                        right.push(EqNode::Space(super::SpaceKind::Tab));
                    }
                }
            } else {
                // OVER/ATOP 중위 처리 (#505) — 활성 측(right 우선) 의 children 에 적용
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

        // 마지막 구분자 뒤의 빈 행과 빈 묶음의 한 행도 보존한다.
        {
            alignment_marks.push(current_right.is_some());
            let left = EqNode::Row(current_left).simplify();
            let right = current_right
                .map(|r| EqNode::Row(r).simplify())
                .unwrap_or(EqNode::Empty);
            rows.push((left, right));
        }

        self.expect(TokenType::RBrace);

        EqNode::EqAlign {
            rows,
            alignment_marks,
        }
    }
}

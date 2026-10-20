//! 계산식 파서: 토큰 스트림 → AST

use super::tokenizer::{tokenize, DirectionKind, Token};

/// 수식 AST 노드
#[derive(Debug, Clone, PartialEq)]
pub enum FormulaNode {
    /// 숫자 리터럴
    Number(f64),
    /// 셀 참조 (col: "A"-"Z", "AA"... 또는 "?", row: 1~ 또는 와일드카드)
    CellRef { col: String, row: u32 },
    /// 범위 참조 (시작 셀 : 끝 셀)
    Range {
        start: Box<FormulaNode>,
        end: Box<FormulaNode>,
    },
    /// 방향 지정자 (left, right, above, below)
    Direction(DirectionKind),
    /// 이항 연산 (+, -, *, /)
    BinOp {
        op: BinOpKind,
        left: Box<FormulaNode>,
        right: Box<FormulaNode>,
    },
    /// 단항 음수 (-x)
    Negate(Box<FormulaNode>),
    /// 함수 호출
    FuncCall {
        name: String,
        args: Vec<FormulaNode>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinOpKind {
    Add,
    Sub,
    Mul,
    Div,
}

/// 계산식 문자열을 파싱하여 AST를 반환한다.
///
/// 문법 밖 글자, 피연산자 없이 매달린 연산자, 짝이 맞지 않는 괄호, 식 뒤에 남은 토큰은
/// 잘못된 계산식이라 `None`을 돌려준다.
pub fn parse_formula(input: &str) -> Option<FormulaNode> {
    let mut parser = Parser {
        tokens: tokenize(input)?,
        pos: 0,
    };
    let node = parser.parse_expr()?;
    // 식 하나를 읽고도 토큰이 남으면(`=SUM(A1:A2)xyz`, `=1+2)`) 잘못된 계산식이다.
    (parser.pos == parser.tokens.len()).then_some(node)
}

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn advance(&mut self) -> Option<Token> {
        if self.pos < self.tokens.len() {
            let tok = self.tokens[self.pos].clone();
            self.pos += 1;
            Some(tok)
        } else {
            None
        }
    }

    fn expect(&mut self, expected: &Token) -> Option<()> {
        if self.peek() == Some(expected) {
            self.advance();
            Some(())
        } else {
            None
        }
    }

    /// expr = term (('+' | '-') term)*
    fn parse_expr(&mut self) -> Option<FormulaNode> {
        let mut left = self.parse_term()?;
        while let Some(tok) = self.peek() {
            match tok {
                Token::Plus => {
                    self.advance();
                    let right = self.parse_term()?;
                    left = FormulaNode::BinOp {
                        op: BinOpKind::Add,
                        left: Box::new(left),
                        right: Box::new(right),
                    };
                }
                Token::Minus => {
                    self.advance();
                    let right = self.parse_term()?;
                    left = FormulaNode::BinOp {
                        op: BinOpKind::Sub,
                        left: Box::new(left),
                        right: Box::new(right),
                    };
                }
                _ => break,
            }
        }
        Some(left)
    }

    /// term = factor (('*' | '/') factor)*
    fn parse_term(&mut self) -> Option<FormulaNode> {
        let mut left = self.parse_factor()?;
        while let Some(tok) = self.peek() {
            match tok {
                Token::Star => {
                    self.advance();
                    let right = self.parse_factor()?;
                    left = FormulaNode::BinOp {
                        op: BinOpKind::Mul,
                        left: Box::new(left),
                        right: Box::new(right),
                    };
                }
                Token::Slash => {
                    self.advance();
                    let right = self.parse_factor()?;
                    left = FormulaNode::BinOp {
                        op: BinOpKind::Div,
                        left: Box::new(left),
                        right: Box::new(right),
                    };
                }
                _ => break,
            }
        }
        Some(left)
    }

    /// factor = NUMBER | cell_ref (':' cell_ref)? | func_call | '(' expr ')' | '-' factor
    fn parse_factor(&mut self) -> Option<FormulaNode> {
        match self.peek().cloned() {
            Some(Token::Number(n)) => {
                self.advance();
                Some(FormulaNode::Number(n))
            }
            Some(Token::CellRef(col, row)) => {
                self.advance();
                let cell = FormulaNode::CellRef { col, row };
                // 범위 참조 확인 (A1:B5)
                if self.peek() == Some(&Token::Colon) {
                    self.advance();
                    if let Some(Token::CellRef(col2, row2)) = self.peek().cloned() {
                        self.advance();
                        Some(FormulaNode::Range {
                            start: Box::new(cell),
                            end: Box::new(FormulaNode::CellRef {
                                col: col2,
                                row: row2,
                            }),
                        })
                    } else {
                        None // ':' 뒤에 셀 참조가 없다
                    }
                } else {
                    Some(cell)
                }
            }
            Some(Token::Function(name)) => {
                self.advance();
                self.expect(&Token::LParen)?;
                let args = self.parse_arg_list()?;
                self.expect(&Token::RParen)?;
                Some(FormulaNode::FuncCall { name, args })
            }
            Some(Token::Direction(dir)) => {
                self.advance();
                Some(FormulaNode::Direction(dir))
            }
            Some(Token::LParen) => {
                self.advance();
                let inner = self.parse_expr()?;
                self.expect(&Token::RParen)?;
                Some(inner)
            }
            Some(Token::Minus) => {
                self.advance();
                let inner = self.parse_factor()?;
                Some(FormulaNode::Negate(Box::new(inner)))
            }
            // 연산자·구분자 자리나 식 끝에는 피연산자가 없다(`=1+`, `=SUM(A1,)`).
            _ => None,
        }
    }

    /// arg_list = arg (',' arg)*
    fn parse_arg_list(&mut self) -> Option<Vec<FormulaNode>> {
        let mut args = Vec::new();
        if self.peek() == Some(&Token::RParen) {
            return Some(args); // 빈 인수
        }
        args.push(self.parse_expr()?);
        while self.peek() == Some(&Token::Comma) {
            self.advance();
            args.push(self.parse_expr()?);
        }
        Some(args)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_add() {
        let ast = parse_formula("=1+2").unwrap();
        assert_eq!(
            ast,
            FormulaNode::BinOp {
                op: BinOpKind::Add,
                left: Box::new(FormulaNode::Number(1.0)),
                right: Box::new(FormulaNode::Number(2.0)),
            }
        );
    }

    #[test]
    fn test_cell_add() {
        let ast = parse_formula("=A1+B3").unwrap();
        match ast {
            FormulaNode::BinOp {
                op: BinOpKind::Add,
                left,
                right,
            } => {
                assert_eq!(
                    *left,
                    FormulaNode::CellRef {
                        col: "A".into(),
                        row: 1
                    }
                );
                assert_eq!(
                    *right,
                    FormulaNode::CellRef {
                        col: "B".into(),
                        row: 3
                    }
                );
            }
            _ => panic!("expected BinOp"),
        }
    }

    #[test]
    fn test_function_sum_range() {
        let ast = parse_formula("=SUM(A1:B5)").unwrap();
        match ast {
            FormulaNode::FuncCall { name, args } => {
                assert_eq!(name, "SUM");
                assert_eq!(args.len(), 1);
                match &args[0] {
                    FormulaNode::Range { start, end } => {
                        assert_eq!(
                            **start,
                            FormulaNode::CellRef {
                                col: "A".into(),
                                row: 1
                            }
                        );
                        assert_eq!(
                            **end,
                            FormulaNode::CellRef {
                                col: "B".into(),
                                row: 5
                            }
                        );
                    }
                    _ => panic!("expected Range"),
                }
            }
            _ => panic!("expected FuncCall"),
        }
    }

    #[test]
    fn test_function_direction() {
        let ast = parse_formula("=sum(left)").unwrap();
        match ast {
            FormulaNode::FuncCall { name, args } => {
                assert_eq!(name, "SUM");
                assert_eq!(args[0], FormulaNode::Direction(DirectionKind::Left));
            }
            _ => panic!("expected FuncCall"),
        }
    }

    #[test]
    fn test_precedence() {
        // 1+2*3 = 1+(2*3)
        let ast = parse_formula("=1+2*3").unwrap();
        match ast {
            FormulaNode::BinOp {
                op: BinOpKind::Add,
                left,
                right,
            } => {
                assert_eq!(*left, FormulaNode::Number(1.0));
                match *right {
                    FormulaNode::BinOp {
                        op: BinOpKind::Mul, ..
                    } => {}
                    _ => panic!("expected Mul"),
                }
            }
            _ => panic!("expected Add"),
        }
    }

    #[test]
    fn test_negate() {
        let ast = parse_formula("=-A1").unwrap();
        match ast {
            FormulaNode::Negate(inner) => {
                assert_eq!(
                    *inner,
                    FormulaNode::CellRef {
                        col: "A".into(),
                        row: 1
                    }
                );
            }
            _ => panic!("expected Negate"),
        }
    }

    #[test]
    fn test_nested_function() {
        let ast = parse_formula("=sum(a1:b5,avg(c3,e5))").unwrap();
        match ast {
            FormulaNode::FuncCall { name, args } => {
                assert_eq!(name, "SUM");
                assert_eq!(args.len(), 2);
                match &args[1] {
                    FormulaNode::FuncCall { name, .. } => assert_eq!(name, "AVG"),
                    _ => panic!("expected nested FuncCall"),
                }
            }
            _ => panic!("expected FuncCall"),
        }
    }
}

//! HWP 표 계산식 엔진
//!
//! 계산식 문자열을 파싱하고 평가하여 숫자 결과를 반환한다.
//!
//! 지원 기능:
//! - 셀 참조: A1, B3
//! - 범위: A1:B5
//! - 와일드카드: ?1:?3, A?:C?
//! - 방향 지정자: left, right, above, below
//! - 사칙연산: +, -, *, /
//! - 시트 함수: SUM, AVG, PRODUCT, MIN, MAX, COUNT 등 22개

mod evaluator;
mod parser;
mod tokenizer;

pub use evaluator::{evaluate_formula, TableContext};
pub use parser::FormulaNode;

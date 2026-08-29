//! [#4135] 표 계산식의 다중 문자 열 참조를 공개 API 경계에서 고정한다.
//!
//! `Z` 다음 열은 `AA`로 이어져야 하며, 숫자가 붙은 함수 이름은 셀 참조로
//! 오인하지 않아야 한다. 파서·토크나이저의 내부 AST 형상 대신 사용자가 실제로
//! 관찰하는 계산 결과를 검증한다.

use rhwp::document_core::table_calc::{evaluate_formula, TableContext};

fn wide_table_context() -> TableContext {
    TableContext {
        row_count: 2,
        col_count: 27,
        current_row: 0,
        current_col: 0,
    }
}

fn column_number(col: usize, _row: usize) -> Option<f64> {
    Some((col + 1) as f64)
}

#[test]
fn multi_letter_column_resolves_after_z_without_stealing_function_names() {
    let ctx = wide_table_context();

    assert_eq!(
        evaluate_formula("=AA1", &ctx, &column_number).unwrap(),
        27.0
    );
    assert_eq!(
        evaluate_formula("=LOG10(100)", &ctx, &column_number).unwrap(),
        2.0,
        "digit-suffixed functions must stay functions"
    );
}

#[test]
fn range_can_cross_z_to_aa() {
    let ctx = wide_table_context();

    assert_eq!(
        evaluate_formula("=SUM(Z1:AA1)", &ctx, &column_number).unwrap(),
        53.0
    );
}

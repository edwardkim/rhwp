//! 계산식 함수의 뜻을 한/글에 맞춘다.
//!
//! 한컴 도움말(`table/calculation/calculation(function).htm`)은 LOG를 상용로그(밑 10),
//! LN을 자연로그로 적고 DEGTORAD·RADTODEG와 자릿수를 받는 ROUND를 둔다. 한/글 12.30(Mac)의
//! 표 > 계산식에서 같은 식을 넣어 읽은 값: LOG(100)=2, LN(100)=4.60517,
//! ROUND(123.456,1)=123.5, ROUND(123.456,2)=123.46, ROUND(123.456,-1)=120,
//! ROUND(-2.5,0)=-3, ROUND(0.5,0)=1, ROUND(2.675,2)=2.68, ROUND(123.456,1.7)=123.5,
//! ROUND(123.456,-1.7)=120, DEGTORAD(180)=3.141593, RADTODEG(3.14159265358979)=180.
//! 자릿수 없는 ROUND(2.5)는 한/글에서 "잘못된 계산식"이다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::table_calc::{evaluate_formula, TableContext};

fn eval(formula: &str) -> f64 {
    let ctx = TableContext {
        row_count: 2,
        col_count: 2,
        current_row: 1,
        current_col: 1,
    };
    // A1=100, 나머지 칸은 비어 있다.
    let cell = |col: usize, row: usize| (col == 0 && row == 0).then_some(100.0);
    evaluate_formula(formula, &ctx, &cell).unwrap_or_else(|e| panic!("{formula}: {e}"))
}

fn assert_close(formula: &str, expected: f64) {
    let value = eval(formula);
    assert!(
        (value - expected).abs() < 1e-9,
        "{formula} = {value}, 기대 {expected}"
    );
}

#[test]
fn log_is_common_logarithm_and_ln_is_natural() {
    assert_close("=LOG(100)", 2.0);
    assert_close("=LOG(A1)", 2.0);
    assert_close("=LN(100)", 100f64.ln());
    assert_close("=LN(A1)", 100f64.ln());
}

#[test]
fn degree_and_radian_conversions() {
    assert_close("=DEGTORAD(180)", std::f64::consts::PI);
    assert_close("=RADIAN(180)", std::f64::consts::PI);
    assert_close("=RADTODEG(3.14159265358979)", 180.0);
}

#[test]
fn round_takes_decimal_digits_like_hancom() {
    assert_eq!(eval("=ROUND(123.456,1)"), 123.5);
    assert_eq!(eval("=ROUND(123.456,2)"), 123.46);
    assert_eq!(eval("=ROUND(123.456,-1)"), 120.0);
    assert_eq!(eval("=ROUND(123.456,0)"), 123.0);
    assert_eq!(eval("=ROUND(-2.5,0)"), -3.0);
    assert_eq!(eval("=ROUND(0.5,0)"), 1.0);
    assert_eq!(eval("=ROUND(2.675,2)"), 2.68);
    // 자릿수의 소수 부분은 버린다.
    assert_eq!(eval("=ROUND(123.456,1.7)"), 123.5);
    assert_eq!(eval("=ROUND(123.456,-1.7)"), 120.0);
    // 자릿수 없는 ROUND는 0자리로 계산하던 기존 형태를 지킨다.
    assert_eq!(eval("=ROUND(2.5)"), 3.0);
}

#[test]
fn wrong_argument_counts_are_rejected() {
    let ctx = TableContext {
        row_count: 1,
        col_count: 1,
        current_row: 0,
        current_col: 0,
    };
    for formula in ["=ROUND(1,2,3)", "=ROUND()", "=LN()", "=RADTODEG()"] {
        assert!(
            evaluate_formula(formula, &ctx, &|_, _| None).is_err(),
            "{formula}"
        );
    }
}

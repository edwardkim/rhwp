//! 검사 유형: 관계형(오류 시 문서 불변)·고정형(독립 함수 의미와 계산 결과); 페이지 절대 px 좌표 핀 없음.
//! 주석 갱신: 2026-11-05T15:49:15+09:00; 직전 파일 변경: 709a7fbf5350ef63601dac255844f6b1dbcfb424 2026-11-05T14:38:03+09:00.
//! 측정일 2026-11-05, 제품 source 6337fe4d10; 96 DPI·Print·Native Skia/fresh WASM Canvas·2px raw 실루엣.
//! table-formula-easy: 전1쪽 Native 최저 100.00% / fresh WASM 최저 100.00%; 누락 없음.
//! table-formula-hancom-result: 전1쪽 Native 최저 100.00% / fresh WASM 최저 100.00%; 누락 없음.
//! table-formula-stale: 전1쪽 Native 최저 100.00% / fresh WASM 최저 100.00%; 누락 없음.
//! table-formula-cycles: 전1쪽 Native 최저 100.00% / fresh WASM 최저 100.00%; 누락 없음.
//! table-formula-easy-edited-hwpx: 전1쪽 Native 최저 100.00% / fresh WASM 최저 100.00%; 누락 없음.
//! table-formula-recalculated-hwp: 전1쪽 Native 최저 100.00% / fresh WASM 최저 100.00%; 누락 없음.
//! table-formula-recalculated-hwpx: 전1쪽 Native 최저 100.00% / fresh WASM 최저 100.00%; 누락 없음.
//! 증거: mydocs/pr/assets/semanticist21-20261105-next10/validation/pre8622/candidate/{native,wasm}/<위 key>.tsv, validation.md 및 같은 영역의 독립 Print 확대 판독.
//! 순수 API/합성 계약 검사와 실물 문서의 시각 측정은 구분한다. 계산한 원본 크기·관계는 유지한다.
//!
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
use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;

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

#[test]
fn non_finite_results_are_rejected_before_writing_a_cell() {
    let mut core =
        DocumentCore::from_bytes(include_bytes!("../fixtures/table_formula/easy.hwpx")).unwrap();
    let (para, control) = core.document().sections[0]
        .paragraphs
        .iter()
        .enumerate()
        .find_map(|(pi, paragraph)| {
            paragraph
                .controls
                .iter()
                .position(|control| matches!(control, Control::Table(_)))
                .map(|ci| (pi, ci))
        })
        .unwrap();
    let before = format!("{:?}", core.document());
    for formula in ["=LN(-1)", "=LN(0)", "=LOG(0)", "=SQRT(-1)", "=EXP(1000)"] {
        for write_result in [false, true] {
            let error = core
                .evaluate_table_formula(0, para, control, 0, 0, formula, write_result)
                .expect_err("비유한 결과는 JSON 생성과 셀 변경 전에 거부한다");
            assert!(error.to_string().contains("유한한 숫자"), "{error}");
            assert_eq!(format!("{:?}", core.document()), before, "{formula}");
        }
    }
}

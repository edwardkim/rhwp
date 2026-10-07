#![cfg(not(target_arch = "wasm32"))]
//! 표 계산식은 끝에 매달린 연산자, 짝이 맞지 않는 괄호, 식 뒤에 남은 토큰과 문법 밖 글자를
//! 오류로 돌려준다. 빠진 피연산자를 0으로 채우거나 남은 토큰을 버리면 `=1+`이 1,
//! `=SUM(A1:A2`와 `=SUM(A1:A2)xyz`가 합계로 계산되어 칸에 적힌다. 올바른 식의 결과는 그대로다.
use rhwp::document_core::table_calc::{evaluate_formula, TableContext};
use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::wasm_api::HwpDocument;

/// 5×5 표의 A5 칸에서 계산한다.
fn ctx() -> TableContext {
    TableContext {
        row_count: 5,
        col_count: 5,
        current_row: 4,
        current_col: 0,
    }
}

/// 칸 값은 행 번호 × 10 + 열 번호다(A1 = 11, B2 = 22, E5 = 55).
fn cell(col: usize, row: usize) -> Option<f64> {
    Some(((row + 1) * 10 + col + 1) as f64)
}

#[test]
fn malformed_formulas_are_errors() {
    let computed: Vec<String> = [
        // 피연산자가 빠진 연산자·구분자
        "=1+",
        "=2*",
        "=-",
        "=+1",
        "=()",
        "=A1:",
        "=SUM(A1:)",
        "=A1:5",
        "=SUM(A1,)",
        "=SUM(,A1)",
        // 짝이 맞지 않거나 빠진 괄호
        "=SUM(A1:A2",
        "=(1+2",
        "=1+2)",
        "=SUM",
        "=SUM A1",
        // 식 뒤에 남은 토큰
        "=SUM(A1:A2)xyz",
        "=1 2",
        "=A1 B1",
        // 문법 밖 글자
        "=1+2#",
        "=A1*10%",
        "=SUM(1.2.3)",
    ]
    .into_iter()
    .filter_map(|formula| {
        let value = evaluate_formula(formula, &ctx(), &cell).ok()?;
        Some(format!("{formula} → {value}"))
    })
    .collect();
    assert!(
        computed.is_empty(),
        "잘못된 계산식을 계산했다: {computed:?}"
    );
}

#[test]
fn well_formed_formulas_keep_their_results() {
    for (formula, expected) in [
        ("=42", 42.0),
        ("=.5+1", 1.5),
        ("=A1+B2*2", 55.0),
        ("=(A1+B2)*2", 66.0),
        ("=((1+2))*3", 9.0),
        ("=1/4", 0.25),
        ("=-A1", -11.0),
        ("=--A1", 11.0),
        ("=1*-2", -2.0),
        ("=1--2", 3.0),
        ("@SUM(A1:A3)", 63.0),
        (" = sum( a1 : a3 ) ", 63.0),
        ("=SUM(A1:A3,AVG(B1,B2))", 80.0),
        ("=a1+(b3-3)*2+sum(a1:b5,avg(c3,e5-3))", 426.5),
        ("=SUM(above)", 104.0),
        ("=SUM(right)", 214.0),
        ("=SUM(?1:?3)", 63.0),
        ("=SUM(A?:C?)", 156.0),
        ("=AVERAGE(A1:A3)", 21.0),
        ("=AVG(A1:A3)", 21.0),
        ("=PRODUCT(B1,C3)", 396.0),
        ("=MIN(A1:C1)", 11.0),
        ("=MAX(A1:C1)", 13.0),
        ("=COUNT(A1:B2)", 4.0),
        ("=ABS(-25)", 25.0),
        ("=SQRT(16)", 4.0),
        ("=EXP(0)", 1.0),
        ("=LOG(1)", 0.0),
        ("=LOG10(100)", 2.0),
        ("=SIN(0)", 0.0),
        ("=COS(0)", 1.0),
        ("=TAN(0)", 0.0),
        ("=ASIN(0)", 0.0),
        ("=ACOS(1)", 0.0),
        ("=ATAN(0)", 0.0),
        ("=RADIAN(180)", std::f64::consts::PI),
        ("=SIGN(-3)", -1.0),
        ("=INT(2.7)", 2.0),
        ("=CEILING(2.1)", 3.0),
        ("=FLOOR(2.9)", 2.0),
        ("=ROUND(2.5)", 3.0),
        ("=TRUNC(-2.7)", -2.0),
        ("=MOD(10,3)", 1.0),
        ("=IF(1,10,20)", 10.0),
        ("=IF(0,10,20)", 20.0),
    ] {
        assert_eq!(
            evaluate_formula(formula, &ctx(), &cell),
            Ok(expected),
            "{formula}"
        );
    }
}

/// 1행 3열 표의 C1 글.
fn c1_text(core: &DocumentCore) -> &str {
    let Some(Control::Table(table)) = core.document().sections[0].paragraphs[0].controls.first()
    else {
        panic!("표를 찾지 못했다");
    };
    &table.cells[2].paragraphs[0].text
}

#[test]
fn table_formula_command_does_not_write_malformed_results() {
    let mut doc = HwpDocument::create_empty();
    doc.create_table_native(0, 0, 0, 1, 3).expect("1×3 표");
    for (cell_idx, text) in [(0, "1"), (1, "2")] {
        doc.insert_text_in_cell_native(0, 0, 0, cell_idx, 0, 0, text)
            .expect("칸 입력");
    }
    let core: &mut DocumentCore = &mut doc;

    // 잘못된 식은 오류이고 C1에 아무것도 적지 않는다.
    for formula in ["=1+", "=SUM(A1:B1", "=SUM(A1:B1)xyz"] {
        let result = core.evaluate_table_formula(0, 0, 0, 0, 2, formula, true);
        assert!(
            result.is_err() && c1_text(core).is_empty(),
            "{formula}: {result:?}, C1 = {:?}",
            c1_text(core)
        );
    }
    core.evaluate_table_formula(0, 0, 0, 0, 2, "=SUM(A1:B1)", true)
        .expect("올바른 합계");
    assert_eq!(c1_text(core), "3");
}

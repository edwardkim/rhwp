//! [#4135] 표 계산식의 열 참조와 병합 표 좌표 처리를 공개 API 경계에서 고정한다.
//!
//! `Z` 다음 열은 `AA`로 이어져야 하며, 숫자가 붙은 함수 이름은 셀 참조로
//! 오인하지 않아야 한다. 병합으로 셀 배열이 줄어도 논리 좌표의 원본·결과 셀을
//! 찾아야 한다. 파서·토크나이저의 내부 AST 형상 대신 사용자가 실제로 관찰하는
//! 계산 결과를 검증한다.

use rhwp::document_core::table_calc::{evaluate_formula, TableContext};
use rhwp::model::control::Control;
use rhwp::wasm_api::HwpDocument;
use serde_json::Value;

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

#[test]
fn merged_header_does_not_shift_formula_sources_or_result_target() {
    let mut doc = HwpDocument::create_empty();
    doc.create_table_native(0, 0, 0, 3, 4)
        .expect("3행 4열 표 생성");
    doc.merge_table_cells_native(0, 0, 0, 0, 0, 0, 1)
        .expect("머리글 A1:B1 병합");

    // 병합 뒤 실제 cells 인덱스는 A2=3, B2=4, C2=5, D2=6, A3=7이다.
    for (cell_idx, text) in [(3, "1"), (4, "2"), (5, "3")] {
        doc.insert_text_in_cell_native(0, 0, 0, cell_idx, 0, 0, text)
            .expect("계산 원본 셀 입력");
    }

    let result: Value = serde_json::from_str(
        &doc.evaluate_table_formula(0, 0, 0, 1, 3, "=SUM(A2:C2)", true)
            .expect("병합 머리글 아래 블록 합계"),
    )
    .expect("블록 합계 JSON");
    assert_eq!(result["result"].as_f64(), Some(6.0));

    let table = match &doc.document().sections[0].paragraphs[0].controls[0] {
        Control::Table(table) => table,
        _ => panic!("표 컨트롤 필요"),
    };
    let text_at = |row: u16, col: u16| {
        table
            .cells
            .iter()
            .find(|cell| cell.row == row && cell.col == col)
            .and_then(|cell| cell.paragraphs.first())
            .map(|paragraph| paragraph.text.as_str())
            .expect("좌표 셀 문단 필요")
    };
    assert_eq!(text_at(1, 3), "6", "결과는 D2에 기록되어야 한다");
    assert_eq!(text_at(2, 0), "", "다음 행 A3를 바꾸면 안 된다");
}

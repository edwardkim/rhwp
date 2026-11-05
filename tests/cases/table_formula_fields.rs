//! 표 계산식은 결과 글이 아니라 한/글 12.30처럼 셀의 FORMULA 필드로 남고, 셀을 고치면 다시 계산한다.
//!
//! 기대값은 한/글 12.30 Mac에서 직접 계산해 저장한 값이다. `table_formula/stale.hwpx`는 한/글
//! 저장본 틀에 계산식 필드를 넣고 결과를 모두 999로 둔 문서이고, 한/글은 이 표의 셀 하나를 고치자
//! 아래 `HANCOM_RESULTS`처럼 다시 계산해 저장했다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::table_calc::{evaluate_formula, format_formula_result, TableContext};
use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use serde_json::Value;

const FIXTURES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/table_formula");

fn open(name: &str) -> DocumentCore {
    let bytes = std::fs::read(format!("{FIXTURES}/{name}")).unwrap();
    DocumentCore::from_bytes(&bytes).unwrap()
}

/// 문서의 첫 표 (문단, 컨트롤).
fn first_table(doc: &DocumentCore) -> (usize, usize) {
    for (pi, para) in doc.document().sections[0].paragraphs.iter().enumerate() {
        for (ci, control) in para.controls.iter().enumerate() {
            if matches!(control, Control::Table(_)) {
                return (pi, ci);
            }
        }
    }
    panic!("표가 없다");
}

/// "B10" 같은 셀 이름의 (셀 번호, 첫 문단 글).
fn cell(doc: &DocumentCore, name: &str) -> (usize, String) {
    let (pi, ci) = first_table(doc);
    let Control::Table(table) = &doc.document().sections[0].paragraphs[pi].controls[ci] else {
        unreachable!()
    };
    let col = (name.as_bytes()[0] - b'A') as u16;
    let row = name[1..].parse::<u16>().unwrap() - 1;
    let idx = table
        .cells
        .iter()
        .position(|c| c.row == row && c.col == col)
        .unwrap();
    (idx, table.cells[idx].paragraphs[0].text.clone())
}

fn formula_commands(doc: &DocumentCore) -> Vec<String> {
    let fields: Vec<Value> = serde_json::from_str(&doc.get_field_list_json()).unwrap();
    fields
        .iter()
        .filter(|f| f["fieldType"] == "formula")
        .map(|f| f["command"].as_str().unwrap().to_string())
        .collect()
}

/// 셀 글을 바꾼다(앱의 셀 타자와 같은 엔진 경로).
fn retype(doc: &mut DocumentCore, name: &str, text: &str) {
    let (pi, ci) = first_table(doc);
    let (idx, old) = cell(doc, name);
    let len = old.chars().count();
    if len > 0 {
        doc.delete_text_in_cell_native(0, pi, ci, idx, 0, 0, len)
            .unwrap();
    }
    doc.insert_text_in_cell_native(0, pi, ci, idx, 0, 0, text)
        .unwrap();
}

fn recalculate(doc: &mut DocumentCore) -> Value {
    let (pi, ci) = first_table(doc);
    serde_json::from_str(
        &doc.recalculate_table_formulas(0, pi, ci, None, true)
            .unwrap(),
    )
    .unwrap()
}

/// 한/글 12.30이 `stale.hwpx`의 F1에 7을 치자 다시 계산해 저장한 결과(`cprobe1-hc.hwpx`).
/// #8484를 함께 통합해 LN과 자릿수 ROUND도 같은 실제 필드 경로로 대조한다.
const HANCOM_RESULTS: [(&str, &str); 37] = [
    ("D1", "20.00"),         // AVG(LEFT): 빈 셀은 세지 않는다
    ("D2", "6"),             // PRODUCT(LEFT): 빈 셀은 곱하지 않는다
    ("C3", "3"),             // SUM(LEFT)
    ("D3", "6"),             // SUM(LEFT): 같은 방향 합계 칸도 더한다
    ("C4", "3"),             // SUM(LEFT)
    ("D4", "6"),             // PRODUCT(LEFT): 같은 방향 합계 칸도 곱한다
    ("C5", "3"),             // =A5+B5
    ("D5", "2.00"),          // AVG(LEFT): 방향 없는 식 칸은 넣는다
    ("C6", "3"),             // SUM(LEFT)
    ("D6", "2.00"),          // =AVG(A6:C6): 범위 평균은 계산식 칸을 넣는다
    ("A7", "잘못된 계산식"), // =1/0
    ("B7", "잘못된 계산식"), // =SQRT(-1)
    ("C7", "잘못된 계산식"), // =LN(0)
    ("D7", "잘못된 계산식"), // =MOD(5,0)
    ("E7", "잘못된 계산식"), // =ASIN(2)
    ("F7", "1"),             // =A7+1: 글 셀은 0
    ("A8", "1,414,214,000"), // =SQRT(2)*1000000000: 함수 결과를 소수 여섯째 자리로 맞춘다
    ("B8", "1.23457"),       // =SUM(0.1234567,0)*10
    ("C8", "1,000,000"),     // =1/3*3000000: 사칙연산은 맞추지 않는다
    ("D8", "333,333"),       // =ROUND(1/3,7)*1000000: 필드 함수 결과는 여섯째 자리
    ("E8", "2,333,333"),     // =AVG(1,2,4)*1000000
    ("F8", "3"),             // =ABS(-0.0000025)*1000000
    ("A9", "10"),            // =B10+0: 뒤 셀 계산식의 새 값을 쓴다
    ("B10", "10"),           // =A10*2
    ("C11", "1.50"),         // AVG(LEFT)
    ("D11", "4.5"),          // SUM(LEFT): 같은 방향 평균 칸도 더한다
    ("C12", "2"),            // PRODUCT(LEFT)
    ("D12", "1.50"),         // AVG(LEFT): 같은 방향 곱 칸은 뺀다
    ("C13", "3"),            // =SUM(A13:B13)
    ("D13", "2.00"),         // AVG(LEFT): 범위 식 칸은 넣는다
    ("C14", "1,000,051.5"),  // SUM(ABOVE)
    ("D14", "333,352.50"),   // AVG(LEFT): 다른 방향 식 칸은 넣는다
    ("C15", "1,002"),        // SUM(LEFT): 쉼표 숫자를 읽는다
    ("A1", "10"),
    ("B1", ""),
    ("A15", "1,000"),
    ("F1", "7"),
];

#[test]
fn recalculates_a_table_the_way_hancom_did() {
    let mut doc = open("stale.hwpx");
    assert_eq!(cell(&doc, "D1").1, "999");
    retype(&mut doc, "F1", "7");
    recalculate(&mut doc);
    for (name, expected) in HANCOM_RESULTS {
        assert_eq!(cell(&doc, name).1, expected, "{name}");
    }
    // 마지막 결과도 명령에 남는다.
    let commands = formula_commands(&doc);
    assert!(commands.contains(&"=SQRT(2)*1000000000??%g,;;1,414,214,000".to_string()));
    assert!(commands.contains(&"=1/0??%g,;;잘못된 계산식".to_string()));
    assert!(commands.contains(&"=LN(0)??%g,;;잘못된 계산식".to_string()));
    assert!(commands.contains(&"=ROUND(1/3,7)*1000000??%g,;;333,333".to_string()));
    // 이미 맞는 표는 다시 계산해도 바뀌지 않는다.
    assert_eq!(recalculate(&mut doc)["changed"], serde_json::json!([]));
}

#[test]
fn hancom_hwp_formula_fields_read_and_stay_stable() {
    let mut doc = open("hancom.hwp");
    let commands = formula_commands(&doc);
    assert_eq!(commands.len(), 33);
    assert!(commands.contains(&"=AVG(LEFT)??%.2f,;;20.00".to_string()));
    retype(&mut doc, "A1", "40");
    recalculate(&mut doc);
    assert_eq!(cell(&doc, "D1").1, "35.00");
    assert_eq!(cell(&doc, "C14").1, "1,000,051.5");
}

#[test]
fn editing_a_source_cell_matches_hancom_easy_formula_results() {
    // 한/글 12.30: tblcalc-easy(A1=10)의 A1을 100으로 고치자 60→150, 20.00→50.00, 14→104, 24.67→54.67.
    let mut doc = open("easy.hwpx");
    assert_eq!(cell(&doc, "D1").1, "60");
    retype(&mut doc, "A1", "100");
    recalculate(&mut doc);
    for (name, expected) in [
        ("D1", "150"),
        ("E1", "50.00"),
        ("D2", "6"),
        ("A4", "104"),
        ("B4", "9.00"),
        ("C4", "90"),
        ("D4", "54.67"),
    ] {
        assert_eq!(cell(&doc, name).1, expected, "{name}");
    }
}

#[test]
fn function_results_round_to_six_decimals_before_the_next_step() {
    // 한/글 12.30 계산식에 직접 넣어 저장한 결과(`tblcalc-probe`·`cprobe1-hc`).
    let (mut doc, pi, ci) = new_table(&[&["", ""]]);
    for (formula, expected) in [
        ("=SQRT(2)*1000000000", "1,414,214,000"),
        ("=SQRT(2000000)*1000", "1,414,213.562"),
        ("=ABS(-1/3)*3", "0.999999"),
        ("=ABS(100/7)*1000000", "14,285,714"),
        ("=SUM(0.1234567,0)*10", "1.23457"),
        ("=AVG(1,2,4)*1000000", "2,333,333"),
        // 사칙연산은 맞추지 않는다.
        ("=1/3*3000000", "1,000,000"),
    ] {
        let set: Value = serde_json::from_str(
            &doc.set_table_cell_formula(0, pi, ci, 0, formula, "%g,")
                .unwrap(),
        )
        .unwrap();
        assert_eq!(set["text"], expected, "{formula}");
    }
    // 결과만 돌려주는 evaluate_formula는 전 정밀도를 지킨다.
    let ctx = TableContext {
        row_count: 1,
        col_count: 1,
        current_row: 0,
        current_col: 0,
    };
    let value = evaluate_formula("=SQRT(2)*1000000000", &ctx, &|_, _| None).unwrap();
    assert!((value - 1_414_213_562.373095).abs() < 1e-3);
}

#[test]
fn result_text_follows_hancom_formats() {
    // 앞 줄들은 한/글 12.30 계산식에 직접 넣어 저장한 값이다.
    for (value, format, expected) in [
        (1.0 / 3.0, "%g,", "0.333333"),
        (1.0 / 128.0, "%g,", "0.007812"),
        (-1.0 / 128.0, "%g,", "-0.007812"),
        (3.0 / 128.0, "%g,", "0.023438"),
        (0.1 + 0.2, "%g,", "0.3"),
        (0.0000004, "%g,", "0"),
        (-0.0000004, "%g,", "-0"),
        (-1234.5, "%g,", "-1,234.5"),
        (123_456_789_000.0, "%g,", "123,456,789,000"),
        (1e15 / 3.0, "%g,", "333,333,333,333,333.3125"),
        (1e24, "%.2f,", "999,999,999,999,999,983,222,784.00"),
        (2.5, "%.0f,", "3"),
        (-2.5, "%.0f,", "-3"),
        (0.125, "%.2f,", "0.13"),
        (20.0, "%.0f", "20"),
        (-0.0, "%g,", "0"),
        (99.5, "%.0f,", "100"),
        (1500.0, "%g", "1500"),
    ] {
        assert_eq!(
            format_formula_result(value, format),
            expected,
            "{value} {format}"
        );
    }
}

fn new_table(rows: &[&[&str]]) -> (DocumentCore, usize, usize) {
    let mut doc = rhwp::wasm_api::HwpDocument::create_empty();
    doc.create_blank_document_native().unwrap();
    let created: Value = serde_json::from_str(
        &doc.create_table_native(0, 0, 0, rows.len() as u16, rows[0].len() as u16)
            .unwrap(),
    )
    .unwrap();
    let pi = created["paraIdx"].as_u64().unwrap() as usize;
    let ci = created["controlIdx"].as_u64().unwrap() as usize;
    for (r, row) in rows.iter().enumerate() {
        for (c, text) in row.iter().enumerate() {
            if !text.is_empty() {
                doc.insert_text_in_cell_native(0, pi, ci, r * row.len() + c, 0, 0, text)
                    .unwrap();
            }
        }
    }
    let mut core = DocumentCore::new_empty();
    core.set_document(doc.document().clone());
    (core, pi, ci)
}

#[test]
fn set_formula_writes_a_hancom_formula_field_that_survives_hwp_and_hwpx() {
    let (mut doc, pi, ci) = new_table(&[&["10", "20", "30", "", ""], &["1", "2", "3", "", ""]]);
    let set = |doc: &mut DocumentCore, cell: usize, formula: &str, format: &str| -> Value {
        serde_json::from_str(
            &doc.set_table_cell_formula(0, pi, ci, cell, formula, format)
                .unwrap(),
        )
        .unwrap()
    };
    // 쉬운 계산식 가로 합계·가로 평균: 평균은 합계 칸을 뺀다(한/글 20.00).
    assert_eq!(set(&mut doc, 3, "=SUM(LEFT)", "%g,")["text"], "60");
    assert_eq!(set(&mut doc, 4, "=AVG(LEFT)", "%.2f,")["text"], "20.00");
    // 한/글은 0으로 나누면 셀에 "잘못된 계산식"을 쓴다.
    let invalid = set(&mut doc, 8, "=1/0", "%g,");
    assert_eq!(invalid["text"], "잘못된 계산식");
    assert!(invalid["result"].is_null());
    // 식을 바꾸면 같은 필드를 고친다.
    assert_eq!(set(&mut doc, 9, "=A2*2", "%g")["text"], "2");
    assert_eq!(set(&mut doc, 9, "=A2*3", "%.1f")["text"], "3.0");
    assert!(doc
        .set_table_cell_formula(0, pi, ci, 9, "=SUM(", "%g")
        .is_err());

    let expected = [
        "=SUM(LEFT)??%g,;;60",
        "=AVG(LEFT)??%.2f,;;20.00",
        "=1/0??%g,;;잘못된 계산식",
        "=A2*3??%.1f;;3.0",
    ];
    assert_eq!(formula_commands(&doc), expected);

    // 원본 셀을 고치면 결과가 따라온다.
    doc.insert_text_in_cell_native(0, pi, ci, 0, 0, 2, "0")
        .unwrap();
    doc.recalculate_table_formulas(0, pi, ci, None, true)
        .unwrap();
    assert_eq!(cell(&doc, "D1").1, "150");
    assert_eq!(cell(&doc, "E1").1, "50.00");

    let hwpx = doc.export_hwpx_native().unwrap();
    let section = {
        let mut zip = zip::ZipArchive::new(std::io::Cursor::new(&hwpx)).unwrap();
        let mut xml = String::new();
        std::io::Read::read_to_string(&mut zip.by_name("Contents/section0.xml").unwrap(), &mut xml)
            .unwrap();
        xml
    };
    assert!(section.contains(
        r#"<hp:integerParam name="Prop">8</hp:integerParam><hp:stringParam name="Command">=SUM(LEFT)??%g,;;150</hp:stringParam><hp:stringParam name="Formula">=SUM(LEFT)</hp:stringParam><hp:stringParam name="ResultFormat">%g,</hp:stringParam><hp:stringParam name="LastResult">150</hp:stringParam>"#
    ));
    assert!(section.contains(r#"type="FORMULA""#) && section.contains(r#"fieldid="627469685""#));

    let after = [
        "=SUM(LEFT)??%g,;;150",
        "=AVG(LEFT)??%.2f,;;50.00",
        "=1/0??%g,;;잘못된 계산식",
        "=A2*3??%.1f;;3.0",
    ];
    for bytes in [doc.export_hwp_native().unwrap(), hwpx] {
        let mut reopened = DocumentCore::from_bytes(&bytes).unwrap();
        assert_eq!(formula_commands(&reopened), after);
        assert_eq!(cell(&reopened, "D1").1, "150");
        retype(&mut reopened, "B1", "2");
        recalculate(&mut reopened);
        assert_eq!(cell(&reopened, "D1").1, "132");
        assert_eq!(cell(&reopened, "E1").1, "44.00");
    }
}

#[test]
fn typing_after_a_result_stays_outside_and_removal_keeps_the_text() {
    // 한/글 12.30: 결과 3 뒤에 9를 치면 9는 필드 밖에 남고(A2를 고치자 129), 계산식 지우기는 결과를
    // 일반 글로 남겼다.
    let (mut doc, pi, ci) = new_table(&[&["1", "2", "", ""]]);
    doc.set_table_cell_formula(0, pi, ci, 2, "=SUM(LEFT)", "%g,")
        .unwrap();
    doc.set_table_cell_formula(0, pi, ci, 3, "=SUM(LEFT)", "%g,")
        .unwrap();
    doc.insert_text_in_cell_native(0, pi, ci, 2, 0, 1, "9")
        .unwrap();
    assert_eq!(cell(&doc, "C1").1, "39");
    doc.recalculate_table_formulas(0, pi, ci, None, true)
        .unwrap();
    // D1은 C1의 계산식 결과 3을 읽는다.
    assert_eq!(cell(&doc, "D1").1, "6");
    doc.insert_text_in_cell_native(0, pi, ci, 0, 0, 1, "0")
        .unwrap();
    doc.recalculate_table_formulas(0, pi, ci, None, true)
        .unwrap();
    assert_eq!(cell(&doc, "C1").1, "129");
    assert_eq!(cell(&doc, "D1").1, "24");

    let removed: Value =
        serde_json::from_str(&doc.remove_table_cell_formulas(0, pi, ci, 3).unwrap()).unwrap();
    assert_eq!(removed["removed"], 1);
    assert_eq!(cell(&doc, "D1").1, "24");
    assert_eq!(formula_commands(&doc), ["=SUM(LEFT)??%g,;;12"]);
    doc.insert_text_in_cell_native(0, pi, ci, 1, 0, 1, "0")
        .unwrap();
    doc.recalculate_table_formulas(0, pi, ci, None, true)
        .unwrap();
    assert_eq!(cell(&doc, "C1").1, "309");
    assert_eq!(cell(&doc, "D1").1, "24");
}

#[test]
fn wildcard_formulas_ending_in_a_question_mark_keep_their_formula() {
    let (mut doc, pi, ci) = new_table(&[&["1", "2", ""]]);
    doc.set_table_cell_formula(0, pi, ci, 2, "=A?+B?", "%g,")
        .unwrap();
    assert_eq!(formula_commands(&doc), ["=A?+B???%g,;;3"]);
    doc.insert_text_in_cell_native(0, pi, ci, 0, 0, 1, "0")
        .unwrap();
    doc.recalculate_table_formulas(0, pi, ci, Some(0), true)
        .unwrap();
    assert_eq!(cell(&doc, "C1").1, "12");
    let hwpx = doc.export_hwpx_native().unwrap();
    let reopened = DocumentCore::from_bytes(&hwpx).unwrap();
    assert_eq!(formula_commands(&reopened), ["=A?+B???%g,;;12"]);
}

#[test]
fn self_and_circular_references_are_refused() {
    let (mut doc, pi, ci) = new_table(&[&["", "", ""]]);
    // 값을 낼 수 없는 식도 자기 셀을 참조하면 받지 않는다.
    assert!(doc
        .set_table_cell_formula(0, pi, ci, 2, "=C1+1/B1", "%g,")
        .is_err());
    doc.set_table_cell_formula(0, pi, ci, 0, "=B1+1", "%g,")
        .unwrap();
    assert!(doc
        .set_table_cell_formula(0, pi, ci, 1, "=A1+1", "%g,")
        .is_err());
    assert!(doc
        .set_table_cell_formula(0, pi, ci, 1, "=SUM(RIGHT)", "%g,")
        .is_ok());
    assert!(doc
        .set_table_cell_formula(0, pi, ci, 2, "=SUM(LEFT)", "%g,")
        .is_err());
    assert_eq!(formula_commands(&doc).len(), 2);
}

#[test]
fn a_hand_edited_result_waits_for_its_own_source_cells() {
    // 한/글 도움말: 손으로 고친 결과는 그 식이 쓰는 셀을 고칠 때까지 다시 계산하지 않는다.
    let (mut doc, pi, ci) = new_table(&[&["1", "2", "", "5"]]);
    doc.set_table_cell_formula(0, pi, ci, 2, "=A1+B1", "%g,")
        .unwrap();
    // 결과 3을 지운다.
    doc.delete_text_in_cell_native(0, pi, ci, 2, 0, 0, 1)
        .unwrap();
    doc.recalculate_table_formulas(0, pi, ci, Some(2), true)
        .unwrap();
    assert_eq!(cell(&doc, "C1").1, "");
    // 식이 쓰지 않는 D1을 고쳐도 그대로다.
    doc.insert_text_in_cell_native(0, pi, ci, 3, 0, 1, "0")
        .unwrap();
    doc.recalculate_table_formulas(0, pi, ci, Some(3), true)
        .unwrap();
    assert_eq!(cell(&doc, "C1").1, "");
    // 식이 쓰는 A1을 고치면 다시 계산한다.
    doc.insert_text_in_cell_native(0, pi, ci, 0, 0, 1, "0")
        .unwrap();
    doc.recalculate_table_formulas(0, pi, ci, Some(0), true)
        .unwrap();
    assert_eq!(cell(&doc, "C1").1, "12");
}

#[test]
fn circular_formulas_keep_their_results_and_self_references_are_invalid() {
    // 한/글 12.30: A1 =B1+1(5)·B1 =A1+1(6)·A2 =A2+1(7)인 표의 D1에 글자를 치자 A1·B1은 그대로,
    // A2는 "잘못된 계산식"이 되었다(`cycles.hwpx`).
    let mut doc = open("cycles.hwpx");
    retype(&mut doc, "D1", "1");
    recalculate(&mut doc);
    assert_eq!(cell(&doc, "A1").1, "5");
    assert_eq!(cell(&doc, "B1").1, "6");
    assert_eq!(cell(&doc, "A2").1, "잘못된 계산식");
}

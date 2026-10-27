#![cfg(not(target_arch = "wasm32"))]
//! [#8262] 표 칸 안에 표를 만든다 — `create_table_in_cell_by_path_native`.
//!
//! 기대값은 구현이 아니라 한/글 2020(11.0.0.8808)이 만든 문서에서 정했다. 이슈 #8262 의
//! 표본 `nested-hwp2020.hwp`(3×3 가운데 빈 칸에 2×2)와 `nested-mid-hwp2020.hwp`
//! (2×2 첫 칸 `가나다라` 의 `나`·`다` 사이에 2×2)를 rhwp 파서로 읽은 값이다.
//!
//! - 칸 안 표는 글자처럼 취급, `common.attr = 0x082a2211`
//!   (세로 기준 문단 · 가로 기준 단 · 자리차지 · 본문 흐름 · 번호 종류 표)
//! - 폭 = 칸 안쪽 폭(칸 폭 − 왼/오른 안 여백) − 바깥 여백 566, 칸 수로 균등
//! - 문단 나눔: 앞쪽이 비면 원래 문단이 표 문단이 되고, 뒤쪽은 비어도 새 문단
use rhwp::model::control::Control;
use rhwp::model::paragraph::Paragraph;
use rhwp::model::shape::{HorzRelTo, TextWrap, VertRelTo};
use rhwp::model::table::Table;
use rhwp::wasm_api::HwpDocument;

const HNC_NESTED_TABLE_ATTR: u32 = 0x082a_2211;

/// 빈 문서 첫 문단에 글자처럼 취급 표를 만든다. 표는 SectionDef·ColumnDef 뒤 컨트롤 2.
fn doc_with_body_table(rows: u16, cols: u16) -> HwpDocument {
    let mut doc = HwpDocument::create_empty();
    doc.create_blank_document().unwrap();
    doc.create_table_ex_native(0, 0, 0, rows, cols, true, None, None)
        .unwrap();
    doc
}

fn body_table(doc: &HwpDocument) -> &Table {
    match &doc.document().sections[0].paragraphs[0].controls[2] {
        Control::Table(t) => t,
        _ => panic!("문단 0 컨트롤 2 는 표여야 한다"),
    }
}

fn only_table(para: &Paragraph) -> &Table {
    assert_eq!(para.controls.len(), 1, "표 문단에는 표만 있다");
    match &para.controls[0] {
        Control::Table(t) => t,
        _ => panic!("표 문단의 컨트롤은 표여야 한다"),
    }
}

/// 표 모양을 한/글 값과 대조한다. `table.attr` 는 형식마다 파서 표현이 달라 따로 본다 —
/// HWPX 파서는 거기에 글자처럼 취급 비트만 싣는다(한/글이 저장한 HWPX 를 읽어도 `0x1`).
fn assert_hnc_nested_shape(inner: &Table, rows: u16, cols: u16, col_w: u32) {
    assert_eq!((inner.row_count, inner.col_count), (rows, cols));
    assert_eq!(inner.common.attr, HNC_NESTED_TABLE_ATTR, "common.attr");
    assert!(inner.common.treat_as_char, "글자처럼 취급");
    assert_eq!(inner.common.text_wrap, TextWrap::TopAndBottom);
    assert_eq!(inner.common.vert_rel_to, VertRelTo::Para);
    assert_eq!(inner.common.horz_rel_to, HorzRelTo::Column);
    let m = &inner.common.margin;
    assert_eq!(
        (m.left, m.right, m.top, m.bottom),
        (283, 283, 283, 283),
        "바깥 여백"
    );
    assert_eq!(inner.common.width, col_w * cols as u32, "표 폭");
    for cell in &inner.cells {
        assert_eq!(cell.width, col_w, "칸 ({},{}) 폭", cell.row, cell.col);
        let p = &cell.padding;
        assert_eq!(
            (p.left, p.right, p.top, p.bottom),
            (510, 510, 141, 141),
            "칸 안 여백"
        );
    }
}

#[test]
fn create_table_in_empty_cell_matches_hancom() {
    // 3×3 → 가운데 칸(4)에 2×2. 바깥 칸 폭 13,984 → 안쪽 12,964 → −566 = 12,398 → 6,199
    let mut doc = doc_with_body_table(3, 3);
    assert_eq!(
        body_table(&doc).cells[4].width,
        13_984,
        "바깥 칸 폭(한/글과 같음)"
    );

    let json = doc
        .create_table_in_cell_by_path_native(0, 0, &[(2, 4, 0)], 0, 2, 2, None, None)
        .unwrap();
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(v["ok"], true);
    assert_eq!(v["cellParaIdx"], 0);
    assert_eq!(v["controlIdx"], 0);
    assert_eq!(
        v["cellPath"],
        serde_json::json!([
            {"controlIndex": 2, "cellIndex": 4, "cellParaIndex": 0},
            {"controlIndex": 0, "cellIndex": 0, "cellParaIndex": 0}
        ]),
        "캐럿이 갈 안쪽 표 첫 칸까지의 경로"
    );

    let cell = &body_table(&doc).cells[4];
    assert_eq!(cell.paragraphs.len(), 2, "표 문단 + 뒤 빈 문단");
    let host = &cell.paragraphs[0];
    assert!(host.text.is_empty());
    assert_eq!(host.char_count, 9, "컨트롤 8 + 문단 끝 1");
    assert_hnc_nested_shape(only_table(host), 2, 2, 6_199);
    assert_eq!(
        only_table(host).attr,
        HNC_NESTED_TABLE_ATTR,
        "attr = common.attr"
    );
    let tail = &cell.paragraphs[1];
    assert!(
        tail.text.is_empty() && tail.controls.is_empty(),
        "뒤는 빈 문단"
    );

    for (i, other) in body_table(&doc).cells.iter().enumerate() {
        if i != 4 {
            assert_eq!(other.paragraphs.len(), 1, "칸 {} 는 그대로", i);
        }
    }
}

/// 2×2 표 첫 칸에 `가나다라` 를 넣은 문서.
fn doc_with_text_in_first_cell() -> HwpDocument {
    let mut doc = doc_with_body_table(2, 2);
    doc.insert_text_in_cell_native(0, 0, 2, 0, 0, 0, "가나다라")
        .unwrap();
    doc
}

fn cell0_texts(doc: &HwpDocument) -> Vec<String> {
    body_table(doc).cells[0]
        .paragraphs
        .iter()
        .map(|p| {
            if p.controls.iter().any(|c| matches!(c, Control::Table(_))) {
                "<표>".to_string()
            } else {
                p.text.clone()
            }
        })
        .collect()
}

#[test]
fn create_table_between_characters_in_cell_matches_hancom() {
    // 바깥 칸 폭 20,977 → 안쪽 19,957 → −566 = 19,391 → 칸당 9,695 (한/글과 같음)
    let mut doc = doc_with_text_in_first_cell();
    let json = doc
        .create_table_in_cell_by_path_native(0, 0, &[(2, 0, 0)], 2, 2, 2, None, None)
        .unwrap();
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(v["cellParaIdx"], 1, "앞쪽 `가나` 가 남아 표는 둘째 문단");

    assert_eq!(cell0_texts(&doc), vec!["가나", "<표>", "다라"]);
    let host = &body_table(&doc).cells[0].paragraphs[1];
    assert_eq!(host.char_count, 9);
    assert_hnc_nested_shape(only_table(host), 2, 2, 9_695);
    assert_eq!(
        only_table(host).attr,
        HNC_NESTED_TABLE_ATTR,
        "attr = common.attr"
    );
}

#[test]
fn create_table_at_cell_text_start_uses_original_paragraph() {
    // 앞쪽이 비면 따로 문단을 두지 않는다 — 원래 문단이 표 문단이 된다.
    let mut doc = doc_with_text_in_first_cell();
    doc.create_table_in_cell_by_path_native(0, 0, &[(2, 0, 0)], 0, 1, 1, None, None)
        .unwrap();
    assert_eq!(cell0_texts(&doc), vec!["<표>", "가나다라"]);
}

#[test]
fn create_table_at_cell_text_end_adds_empty_paragraph_after() {
    // 뒤쪽은 비어도 새 문단으로 내린다.
    let mut doc = doc_with_text_in_first_cell();
    doc.create_table_in_cell_by_path_native(0, 0, &[(2, 0, 0)], 4, 1, 1, None, None)
        .unwrap();
    assert_eq!(cell0_texts(&doc), vec!["가나다라", "<표>", ""]);
}

#[test]
fn create_table_in_nested_cell_by_path() {
    // 깊이 2: 3×3 가운데 칸의 2×2, 그 첫 칸에 다시 2×2.
    // 안쪽 칸 폭 6,199 → 안쪽 5,179 → −566 = 4,613 → 칸당 2,306
    let mut doc = doc_with_body_table(3, 3);
    doc.create_table_in_cell_by_path_native(0, 0, &[(2, 4, 0)], 0, 2, 2, None, None)
        .unwrap();
    let json = doc
        .create_table_in_cell_by_path_native(0, 0, &[(2, 4, 0), (0, 0, 0)], 0, 2, 2, None, None)
        .unwrap();
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(v["cellPath"].as_array().map(|a| a.len()), Some(3));

    let mid = only_table(&body_table(&doc).cells[4].paragraphs[0]);
    let deep_cell = &mid.cells[0];
    assert_eq!(deep_cell.paragraphs.len(), 2);
    assert_hnc_nested_shape(only_table(&deep_cell.paragraphs[0]), 2, 2, 2_306);
}

#[test]
fn create_table_in_cell_rejects_bad_input_without_change() {
    let mut doc = doc_with_text_in_first_cell();
    let before = cell0_texts(&doc);
    let bf_before = doc.document().doc_info.border_fills.len();

    // 칸 번호 범위 밖
    assert!(doc
        .create_table_in_cell_by_path_native(0, 0, &[(2, 9, 0)], 0, 2, 2, None, None)
        .is_err());
    // 칸 문단 번호 범위 밖
    assert!(doc
        .create_table_in_cell_by_path_native(0, 0, &[(2, 0, 5)], 0, 2, 2, None, None)
        .is_err());
    // 글자 수보다 큰 offset
    assert!(doc
        .create_table_in_cell_by_path_native(0, 0, &[(2, 0, 0)], 5, 2, 2, None, None)
        .is_err());
    // 빈 경로 · 0 행
    assert!(doc
        .create_table_in_cell_by_path_native(0, 0, &[], 0, 2, 2, None, None)
        .is_err());
    assert!(doc
        .create_table_in_cell_by_path_native(0, 0, &[(2, 0, 0)], 0, 0, 2, None, None)
        .is_err());

    assert_eq!(
        cell0_texts(&doc),
        before,
        "실패한 호출은 문서를 바꾸지 않는다"
    );
    assert_eq!(doc.document().doc_info.border_fills.len(), bf_before);
}

#[test]
fn create_table_in_cell_survives_hwp_and_hwpx_roundtrip() {
    let mut doc = doc_with_text_in_first_cell();
    doc.create_table_in_cell_by_path_native(0, 0, &[(2, 0, 0)], 2, 2, 2, None, None)
        .unwrap();

    // 다시 읽은 `table.attr` 는 한/글이 저장한 같은 형식의 문서를 읽었을 때와 같아야 한다.
    for (label, bytes, reloaded_attr) in [
        (
            "hwp",
            doc.export_hwp_native().unwrap(),
            HNC_NESTED_TABLE_ATTR,
        ),
        ("hwpx", doc.export_hwpx_native().unwrap(), 0x1),
    ] {
        let reloaded =
            HwpDocument::new(&bytes).unwrap_or_else(|_| panic!("{label} 다시 읽기 실패"));
        assert_eq!(
            cell0_texts(&reloaded),
            vec!["가나", "<표>", "다라"],
            "{label}: 칸 문단 구조"
        );
        let inner = only_table(&body_table(&reloaded).cells[0].paragraphs[1]);
        assert_hnc_nested_shape(inner, 2, 2, 9_695);
        assert_eq!(inner.attr, reloaded_attr, "{label}: 다시 읽은 table.attr");
    }
}

//! 관계형 및 고정형 회귀: [#8262] 표 칸 안 표의 포함·문단 순서와 원본 속성·HWPUNIT 폭.
//! 주석 갱신: 2026-11-01T17:04:07+09:00 (Asia/Seoul). 절대 px 위치를 고정하지 않는다.
//! 직전 파일 변경: 6135bc2135b8366839f02bbada3df5fe33b13963, 2026-11-01T15:58:43+09:00.
//! 원 기여 변경: 5747f049c51108f6baf08eba74dadd4d3eccec94, 2026-10-27T09:03:38+09:00.
//! 시각 측정: 2026-11-01 (정확한 실행 시각은 아래 visual-summary.json), Print 프로필,
//! 96 DPI·색상 경계 보정 전 2px raw 실루엣. 모든 입력은 1쪽이며 Native/fresh WASM 각 100.00%,
//! 전체 7개 문서의 전쪽 최저도 각각 100.00%이고 독립 Print PDF와 쪽수가 일치한다.
//! 기존 입력: mydocs/pr/assets/issue_8262/input/의 rhwp-nested.hwp·.hwpx와 rhwp-nested-mid.hwp.
//! 새 API 생성 입력: mydocs/pr/assets/pr8300-maintainer-20261101/created/의 nested-empty·nested-mid,
//! HWP/HWPX 각 1쪽. 동일 바이트의 Studio 저장본도 확인했으며 기준은 pdf/issue8262/*-2020.pdf의
//! 한/글 2020(11.0.0.9136) Print 출력이다. 입력·PDF 해시는 print-execution.json에 기록했다.
//! 정식 증거: mydocs/pr/assets/pr8300-maintainer-20261101/visual-summary.json,
//! created/{native,wasm}/*/silhouette.tsv, supplied/{native,wasm}/*/silhouette.tsv 및 *-detail.png.
//! 확대 판독: 외곽·내부 선, 셀 안 여백, 가나–표–다라 순서·간격의 보존을 확인했다.
//! 글꼴이 없는 최소 환경에서도 구조·서식 및 상대적 포함 관계를 검사한다.
//!
//! 기대값은 구현이 아니라 한/글 2020(11.0.0.8808)이 만든 문서에서 정했다. 이슈 #8262 의
//! 표본 `nested-hwp2020.hwp`(3×3 가운데 빈 칸에 2×2)와 `nested-mid-hwp2020.hwp`
//! (2×2 첫 칸 `가나다라` 의 `나`·`다` 사이에 2×2)를 rhwp 파서로 읽은 값이다.
//!
//! - 칸 안 표는 글자처럼 취급, `common.attr = 0x082a2211`
//!   (세로 기준 문단 · 가로 기준 단 · 자리차지 · 본문 흐름 · 번호 종류 표)
//! - 폭 = 칸 안쪽 폭(칸 폭 − 왼/오른 안 여백) − 바깥 여백 566, 칸 수로 균등
//! - 문단 나눔: 앞쪽이 비면 원래 문단이 표 문단이 되고, 뒤쪽은 비어도 새 문단
#![cfg(not(target_arch = "wasm32"))]

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

/// 실제 배치도 원본의 셀 안 여백과 표 바깥 여백을 보존한다. 글꼴·절대 좌표에는 의존하지 않는다.
fn assert_nested_placement(doc: &HwpDocument, cell_index: usize, host_index: usize) {
    use serde_json::Value;

    fn find_table(node: &Value) -> Option<&Value> {
        if node["type"] == "Table" {
            return Some(node);
        }
        node["children"].as_array()?.iter().find_map(find_table)
    }

    let outer = body_table(doc);
    let source_cell = &outer.cells[cell_index];
    let inner = only_table(&source_cell.paragraphs[host_index]);
    let tree: Value = serde_json::from_str(&doc.get_page_render_tree(0).unwrap()).unwrap();
    let table = find_table(&tree).expect("본문 표");
    let cell = table["children"]
        .as_array()
        .unwrap()
        .iter()
        .find(|node| {
            node["type"] == "Cell"
                && node["row"] == source_cell.row
                && node["col"] == source_cell.col
        })
        .expect("생성 대상 셀");
    let children = cell["children"].as_array().unwrap();
    let nested = children
        .iter()
        .find(|node| node["type"] == "Table")
        .expect("셀 소유의 중첩 표");
    let box_value = |node: &Value, key: &str| node["bbox"][key].as_f64().unwrap();
    let px = |hu: i16| hu as f64 * 96.0 / 7200.0;
    // 트리 JSON은 소수 첫째 자리로 직렬화되므로 두 상자의 반올림만 허용한다.
    let epsilon = 0.2;
    assert!(
        box_value(nested, "x") + epsilon
            >= box_value(cell, "x") + px(source_cell.padding.left) + px(inner.common.margin.left),
        "왼쪽 셀 안 여백과 표 바깥 여백"
    );
    assert!(
        box_value(nested, "y") + epsilon
            >= box_value(cell, "y") + px(source_cell.padding.top) + px(inner.common.margin.top),
        "위쪽 셀 안 여백과 표 바깥 여백"
    );
    assert!(
        box_value(nested, "x") + box_value(nested, "w")
            <= box_value(cell, "x") + box_value(cell, "w")
                - px(source_cell.padding.right)
                - px(inner.common.margin.right)
                + epsilon,
        "오른쪽 여백까지 부모 셀 내부에 포함"
    );
    let nested_bottom = box_value(nested, "y") + box_value(nested, "h");
    assert!(
        nested_bottom
            <= box_value(cell, "y") + box_value(cell, "h")
                - px(source_cell.padding.bottom)
                - px(inner.common.margin.bottom)
                + epsilon,
        "표 아래쪽 여백까지 부모 셀 내부에 포함"
    );
    let tail = children
        .iter()
        .find(|node| node["type"] == "TextLine" && node["pi"] == host_index + 1)
        .expect("중첩 표 뒤 문단의 실제 줄");
    assert!(
        box_value(tail, "y") + epsilon >= nested_bottom + px(inner.common.margin.bottom),
        "뒤 문단은 표와 아래쪽 여백 뒤에 배치"
    );
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
    assert_nested_placement(&doc, 4, 0);
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
    assert_nested_placement(&doc, 0, 1);
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
        assert_nested_placement(&reloaded, 0, 1);
    }
}

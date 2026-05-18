//! samples/table-vpos-01.hwp 5쪽 인라인 표 셀 클릭 진입 회귀 테스트.
//!
//! 재현 문서: `samples/table-vpos-01.hwp`
//! 대상: 5쪽 (page index = 4) 의 3개 TAC inline 표:
//!   - pi=30 ci=1  1x2  "참고" | "정부혁신 비전 및 추진전략"
//!   - pi=32 ci=0  1x1  "국민이 주도하고 AI가 뒷받침하는 국민주권정부"
//!   - pi=34 ci=0  1x1  (외곽 wrapper, 내부 1x1 title + 11x3 본문 표)
//!
//! 좌표는 `cargo run -- export-svg samples/table-vpos-01.hwp -p 4 --debug-overlay`
//! SVG 의 cell-clip 영역에서 측정 (96 DPI).
//!
//! [Task #990] pi=33(빈 문단 위 treat-as-char 도형) advance 이중 가산 정정으로
//! pi=34 외곽 표 및 내부 11x3 표가 30.84px 위로 이동 — pi=34 inner 11x3 좌표 갱신.
//!
//! 본 테스트는 hit_test_native 반환 검증 + 실제 cell-entry(insert_text_in_cell_by_path)
//! 검증을 모두 수행한다.

use std::path::Path;

use rhwp::wasm_api::HwpDocument;
use serde_json::Value;

fn load_doc() -> HwpDocument {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/table-vpos-01.hwp");
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", path.display(), e));
    HwpDocument::from_bytes(&bytes).expect("parse table-vpos-01.hwp")
}

fn hit_json(doc: &HwpDocument, page: u32, x: f64, y: f64) -> Value {
    let json = doc
        .hit_test_native(page, x, y)
        .unwrap_or_else(|e| panic!("hit_test_native({page}, {x}, {y}): {e}"));
    serde_json::from_str(&json).unwrap_or_else(|e| panic!("parse hit json `{json}`: {e}"))
}

fn path_tuples(hit: &Value) -> Vec<(usize, usize, usize)> {
    hit.get("cellPath")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .map(|entry| {
                    (
                        entry["controlIndex"].as_u64().expect("controlIndex") as usize,
                        entry["cellIndex"].as_u64().expect("cellIndex") as usize,
                        entry["cellParaIndex"].as_u64().expect("cellParaIndex") as usize,
                    )
                })
                .collect()
        })
        .unwrap_or_default()
}

/// hit_test 결과가 (parent_para, control_index) 외곽 표에 안착했는지 + 셀 path 가 비어있지 않은지 검증.
fn assert_table_hit(hit: &Value, parent_para: u64, control: u64) {
    assert_eq!(
        hit["sectionIndex"].as_u64(),
        Some(0),
        "section must be 0, hit={hit}"
    );
    assert_eq!(
        hit["parentParaIndex"].as_u64(),
        Some(parent_para),
        "click must report parentParaIndex={parent_para}, hit={hit}"
    );
    assert_eq!(
        hit["controlIndex"].as_u64(),
        Some(control),
        "click must report controlIndex={control}, hit={hit}"
    );
    assert!(
        hit.get("cellPath").is_some(),
        "click must include cellPath, hit={hit}"
    );
}

/// 중첩 클릭에서 cellPath 마지막 entry 의 cellIndex 가 기대 inner cell_index 와 일치하는지 검증.
fn assert_nested_inner_cell(hit: &Value, expected_inner_cell_index: usize) {
    let path = path_tuples(hit);
    assert!(
        path.len() >= 2,
        "deeply nested click must have cellPath length >= 2, got {:?}, hit={hit}",
        path
    );
    assert_eq!(
        path.last().unwrap().1,
        expected_inner_cell_index,
        "inner cellPath last entry must point to inner cell_index={expected_inner_cell_index}, got {:?}, hit={hit}",
        path
    );
}

// =======================================================================
// pi=30 / pi=32 — 비중첩 표 (정상 동작 기대)
// =======================================================================

#[test]
fn page5_header_cell0_center_enters_cell() {
    let doc = load_doc();
    let hit = hit_json(&doc, 4, 113.7, 113.4);
    assert_table_hit(&hit, 30, 1);
    assert_eq!(hit["cellIndex"].as_u64(), Some(0), "hit={hit}");
}

#[test]
fn page5_header_cell1_center_enters_cell() {
    let doc = load_doc();
    let hit = hit_json(&doc, 4, 433.0, 113.4);
    assert_table_hit(&hit, 30, 1);
    assert_eq!(hit["cellIndex"].as_u64(), Some(1), "hit={hit}");
}

#[test]
fn page5_title_cell_center_enters_cell() {
    let doc = load_doc();
    let hit = hit_json(&doc, 4, 396.8, 164.0);
    assert_table_hit(&hit, 32, 0);
    assert_eq!(hit["cellIndex"].as_u64(), Some(0), "hit={hit}");
}

// =======================================================================
// pi=34 외곽 1x1 안의 inner 1x1 title — 비교 기준 (정상 동작 기대)
// =======================================================================

#[test]
fn page5_big_inner_title_cell_returns_nested_path() {
    let doc = load_doc();
    let hit = hit_json(&doc, 4, 396.8, 260.6);
    assert_table_hit(&hit, 34, 0);
    let path = path_tuples(&hit);
    assert!(
        path.len() >= 2,
        "inner 1x1 title click must have cellPath length >= 2, got {:?}, hit={hit}",
        path
    );
}

// =======================================================================
// pi=34 inner 11x3 — c=0 column 라벨 셀들 (rowspan=2)
// =======================================================================

/// cell[0] r=0,c=0 "1|참여소통" — cell-clip-52 (x=86.2 y=267.16 w=83.4 h=164.9), 중심 (128, 349.16)
#[test]
fn page5_inner_11x3_c0_row0_label_cell() {
    let doc = load_doc();
    let hit = hit_json(&doc, 4, 128.0, 349.16);
    assert_table_hit(&hit, 34, 0);
    assert_nested_inner_cell(&hit, 0);
}

/// cell[7] r=3,c=0 "2|기본사회" — cell-clip-82 (x=86.2 y=444.76 w=83.4 h=164.9), 중심 (128, 527.16)
#[test]
fn page5_inner_11x3_c0_row3_label_cell() {
    let doc = load_doc();
    let hit = hit_json(&doc, 4, 128.0, 527.16);
    assert_table_hit(&hit, 34, 0);
    assert_nested_inner_cell(&hit, 7);
}

/// cell[14] r=6,c=0 "3|공직혁신" — cell-clip-111 (x=86.2 y=622.26 w=83.4 h=159.1), 중심 (128, 701.86)
#[test]
fn page5_inner_11x3_c0_row6_label_cell() {
    let doc = load_doc();
    let hit = hit_json(&doc, 4, 128.0, 701.86);
    assert_table_hit(&hit, 34, 0);
    assert_nested_inner_cell(&hit, 14);
}

/// cell[19] r=9,c=0 "4|공공 AX" — cell-clip-136 (x=86.2 y=818.56 w=83.4 h=157.4), 중심 (128, 897.26)
#[test]
fn page5_inner_11x3_c0_row9_label_cell() {
    let doc = load_doc();
    let hit = hit_json(&doc, 4, 128.0, 897.26);
    assert_table_hit(&hit, 34, 0);
    assert_nested_inner_cell(&hit, 19);
}

// =======================================================================
// pi=34 inner 11x3 — c=2 column 본문 셀들
// =======================================================================

/// cell[2] r=0,c=2 "국민 주도..." — cell-clip-61 (x=177.6 y=298.06 w=529.9 h=45.1), 중심 (442.5, 320.56)
#[test]
fn page5_inner_11x3_c2_row0_content_cell() {
    let doc = load_doc();
    let hit = hit_json(&doc, 4, 442.5, 320.56);
    assert_table_hit(&hit, 34, 0);
    assert_nested_inner_cell(&hit, 2);
}

/// cell[3] r=1,c=2 "대국민 소통..." — cell-clip-65 (x=177.6 y=312.26 w=529.9 h=119.9), 중심 (442.5, 372.16)
#[test]
fn page5_inner_11x3_c2_row1_content_cell() {
    let doc = load_doc();
    let hit = hit_json(&doc, 4, 442.5, 372.16);
    assert_table_hit(&hit, 34, 0);
    assert_nested_inner_cell(&hit, 3);
}

/// cell[9] r=3,c=2 "포용과 균형의 기본사회 구현" — cell-clip-91 (x=177.6 y=475.56 w=529.9 h=45.1), 중심 (442.5, 498.16)
#[test]
fn page5_inner_11x3_c2_row3_content_cell() {
    let doc = load_doc();
    let hit = hit_json(&doc, 4, 442.5, 498.16);
    assert_table_hit(&hit, 34, 0);
    assert_nested_inner_cell(&hit, 9);
}

/// cell[16] r=6,c=2 "성과로 신뢰..." — cell-clip-120 (x=177.6 y=653.16 w=529.9 h=45.1), 중심 (442.5, 675.66)
#[test]
fn page5_inner_11x3_c2_row6_content_cell() {
    let doc = load_doc();
    let hit = hit_json(&doc, 4, 442.5, 675.66);
    assert_table_hit(&hit, 34, 0);
    assert_nested_inner_cell(&hit, 16);
}

// =======================================================================
// 실제 cell-entry 검증: 클릭 결과 path 가 inner 셀에 텍스트를 삽입할 수 있는가
// =======================================================================
// insert_text_in_cell_by_path 는 path 가 길이 1이라도 외곽 cell paragraph 까지만
// 진입하여 정상 반환한다. 따라서 "삽입된 텍스트가 inner 셀의 텍스트와 함께 나타나는지"
// 까지 검증해야 진짜 inner 진입 여부를 확인할 수 있다.

/// inner 11x3 c=2 row=0 셀에 텍스트 삽입 후, 그 셀(예상 path=[(0,0,1),(0,2,0)]) 의
/// 텍스트 첫 글자가 "X" 인지 확인. WASM hit_test_native 가 올바른 path 를 반환한다면
/// 삽입이 inner 셀 내부에 일어나야 함.
#[test]
fn page5_inner_11x3_c2_row0_insert_lands_in_inner_cell() {
    let mut doc = load_doc();
    let hit = hit_json(&doc, 4, 442.5, 320.56);
    let path = path_tuples(&hit);
    let parent_para = hit["parentParaIndex"].as_u64().expect("parentParaIndex") as usize;
    let char_offset = hit["charOffset"].as_u64().expect("charOffset") as usize;
    doc.insert_text_in_cell_by_path(0, parent_para, &path, char_offset, "ZZZTEST")
        .unwrap_or_else(|e| panic!("insert failed: {e:?}, hit={hit}, path={:?}", path));

    // inner 11x3 r=0,c=2 셀의 expected path. 이 경로 안에 "ZZZTEST" 가 보여야 한다.
    // (insert 위치는 hit.charOffset 에 따라 달라지므로 cell 전체 텍스트 substring 검사)
    let expected_inner_path = vec![(0usize, 0usize, 1usize), (0usize, 2usize, 0usize)];
    let inner_text = doc
        .get_text_in_cell_by_path(0, 34, &expected_inner_path, 0, 64)
        .unwrap_or_else(|e| panic!("get_text inner cell failed: {e:?}"));
    assert!(
        inner_text.contains("ZZZTEST"),
        "inserted text must appear in inner 11x3 r=0,c=2 (any position), but inner cell text = {:?}, hit={hit}",
        inner_text
    );
}

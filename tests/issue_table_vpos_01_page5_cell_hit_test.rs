//! samples/table-vpos-01.hwp 5쪽 인라인 표 셀 클릭 진입 회귀 테스트.
//!
//! 재현 문서: `samples/table-vpos-01.hwp`
//! 대상: 5쪽 (page index = 4) 의 3개 TAC inline 표:
//!   - pi=30 ci=1  1x2  "참고" | "정부혁신 비전 및 추진전략"
//!   - pi=32 ci=0  1x1  "국민이 주도하고 AI가 뒷받침하는 국민주권정부"
//!   - pi=34 ci=0  1x1  (외곽 wrapper, 내부 1x1 title + 11x3 본문 표)
//!
//! 좌표는 `cargo run --bin rhwp -- export-svg samples/table-vpos-01.hwp -p 4 --debug-overlay`
//! SVG 의 cell-clip 영역에서 측정 (96 DPI).
//!
//! [Task #990] pi=33(빈 문단 위 treat-as-char 도형) advance 이중 가산 정정으로
//! pi=34 외곽 표 및 내부 11x3 표가 30.84px 위로 이동 — pi=34 inner 11x3 좌표 갱신.
//!
//! [#3386] 중첩 11x3 클릭점은 **하드코딩을 버리고 실행 시점 기하에서 유도**한다.
//! hit_test 는 셀 사각형 전체가 아니라 셀 안 각 줄의 line band(≈20px)에서만 중첩
//! 경로를 반환하므로, 표 행높이가 조금만 움직여도 박아둔 셀 중심 좌표가 줄 사이
//! 빈틈으로 빠진다. 행높이는 설치 폰트에 의존해 환경마다 다르므로(로컬 Windows =
//! 한글 폰트 보유, CI Linux = 폴백) 절대 좌표는 어느 한쪽에서만 맞는다. #3386
//! A/C/D 행경계 교정이 CI 에서만 두 셀을 줄 밖으로 밀어낸 것이 그 사례다.
//! 그래서 대상 셀의 커서 사각형(path→rect)에서 클릭점을 만들고 그 점이 다시 같은
//! 셀 경로로 해석되는지(rect→path) 왕복을 검증한다.
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

/// pi=34 외곽 1x1 안 inner 표의 `cell_index` 셀 첫 글자 커서 사각형에서 클릭점을 만든다.
///
/// 반환값은 (x, y) — 커서 사각형 왼쪽에서 4px 안쪽, 줄 높이의 중앙. 행높이가
/// 환경/수정으로 이동해도 항상 그 셀의 줄 위에 있으므로 절대 좌표 하드코딩 없이
/// "그 셀을 클릭했을 때" 를 재현한다.
fn inner_cell_click_point(doc: &HwpDocument, cell_index: usize) -> (f64, f64) {
    let path_json = format!(
        r#"[{{"controlIndex":0,"cellIndex":0,"cellParaIndex":1}},
            {{"controlIndex":0,"cellIndex":{cell_index},"cellParaIndex":0}}]"#
    );
    let rect_json = doc
        .get_cursor_rect_by_path(0, 34, &path_json, 0)
        .unwrap_or_else(|e| panic!("cursor rect for inner cell {cell_index}: {e:?}"));
    let rect: Value = serde_json::from_str(&rect_json)
        .unwrap_or_else(|e| panic!("parse cursor rect `{rect_json}`: {e}"));
    assert_eq!(
        rect["pageIndex"].as_u64(),
        Some(4),
        "inner cell {cell_index} must live on page index 4, rect={rect_json}"
    );
    let x = rect["x"].as_f64().expect("cursor x") + 4.0;
    let y = rect["y"].as_f64().expect("cursor y")
        + rect["height"].as_f64().expect("cursor height") / 2.0;
    (x, y)
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

/// cell[0] r=0,c=0 "1|참여소통" (rowspan=2 라벨 셀)
#[test]
fn page5_inner_11x3_c0_row0_label_cell() {
    let doc = load_doc();
    let (x, y) = inner_cell_click_point(&doc, 0);
    let hit = hit_json(&doc, 4, x, y);
    assert_table_hit(&hit, 34, 0);
    assert_nested_inner_cell(&hit, 0);
}

/// cell[7] r=3,c=0 "2|기본사회"
#[test]
fn page5_inner_11x3_c0_row3_label_cell() {
    let doc = load_doc();
    let (x, y) = inner_cell_click_point(&doc, 7);
    let hit = hit_json(&doc, 4, x, y);
    assert_table_hit(&hit, 34, 0);
    assert_nested_inner_cell(&hit, 7);
}

/// cell[14] r=6,c=0 "3|공직혁신"
#[test]
fn page5_inner_11x3_c0_row6_label_cell() {
    let doc = load_doc();
    let (x, y) = inner_cell_click_point(&doc, 14);
    let hit = hit_json(&doc, 4, x, y);
    assert_table_hit(&hit, 34, 0);
    assert_nested_inner_cell(&hit, 14);
}

/// cell[19] r=9,c=0 "4|공공 AX"
#[test]
fn page5_inner_11x3_c0_row9_label_cell() {
    let doc = load_doc();
    let (x, y) = inner_cell_click_point(&doc, 19);
    let hit = hit_json(&doc, 4, x, y);
    assert_table_hit(&hit, 34, 0);
    assert_nested_inner_cell(&hit, 19);
}

/// inner 11x3 c=0 row=9 의 10번 글자겹침 마커는 두 개의 PUA 구성 글자로
/// 저장되지만, 편집 커서는 한 글자 단위로 이동해야 한다.
#[test]
fn page5_inner_11x3_char_overlap_marker_advances_one_box() {
    let doc = load_doc();
    let path_json = r#"
        [
          {"controlIndex":0,"cellIndex":0,"cellParaIndex":1},
          {"controlIndex":0,"cellIndex":19,"cellParaIndex":0}
        ]
    "#;
    let before_json = doc
        .get_cursor_rect_by_path(0, 34, path_json, 0)
        .unwrap_or_else(|e| panic!("cursor before marker failed: {e:?}"));
    let after_json = doc
        .get_cursor_rect_by_path(0, 34, path_json, 1)
        .unwrap_or_else(|e| panic!("cursor after marker failed: {e:?}"));
    let before: serde_json::Value =
        serde_json::from_str(&before_json).expect("parse cursor before marker");
    let after: serde_json::Value =
        serde_json::from_str(&after_json).expect("parse cursor after marker");
    let x0 = before["x"].as_f64().expect("before x");
    let x1 = after["x"].as_f64().expect("after x");
    let delta = x1 - x0;
    assert!(
        delta > 16.0 && delta < 30.0,
        "CharOverlap cursor advance must cover one full marker box, got {delta:.2}; before={before_json}, after={after_json}"
    );
}

// =======================================================================
// pi=34 inner 11x3 — c=2 column 본문 셀들
// =======================================================================

/// cell[2] r=0,c=2 "국민 주도 참여‧소통 거버넌스 구현"
#[test]
fn page5_inner_11x3_c2_row0_content_cell() {
    let doc = load_doc();
    let (x, y) = inner_cell_click_point(&doc, 2);
    let hit = hit_json(&doc, 4, x, y);
    assert_table_hit(&hit, 34, 0);
    assert_nested_inner_cell(&hit, 2);
}

/// cell[3] r=1,c=2 "대국민 소통..."
#[test]
fn page5_inner_11x3_c2_row1_content_cell() {
    let doc = load_doc();
    let (x, y) = inner_cell_click_point(&doc, 3);
    let hit = hit_json(&doc, 4, x, y);
    assert_table_hit(&hit, 34, 0);
    assert_nested_inner_cell(&hit, 3);
}

/// cell[9] r=3,c=2 "포용과 균형의 기본사회 구현"
#[test]
fn page5_inner_11x3_c2_row3_content_cell() {
    let doc = load_doc();
    let (x, y) = inner_cell_click_point(&doc, 9);
    let hit = hit_json(&doc, 4, x, y);
    assert_table_hit(&hit, 34, 0);
    assert_nested_inner_cell(&hit, 9);
}

/// cell[16] r=6,c=2 "성과로 신뢰..."
#[test]
fn page5_inner_11x3_c2_row6_content_cell() {
    let doc = load_doc();
    let (x, y) = inner_cell_click_point(&doc, 16);
    let hit = hit_json(&doc, 4, x, y);
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
    let (x, y) = inner_cell_click_point(&doc, 2);
    let hit = hit_json(&doc, 4, x, y);
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

//! Issue #7442: 중첩 표 칸의 빈 영역·괘선 hitTest 가 바깥 칸 경로를 돌려주던 결함.
//!
//! 재현 문서: `samples/basic/issue1994_behindtext_table_20200830.hwp`
//! (1쪽, ppi=5 의 표 칸 10 안에 18×9 중첩 표).
//!
//! 증상: 중첩 칸의 글자 run 위에서만 hitTest 가 깊이 2 cellPath를 돌려주고,
//! 같은 칸의 빈 영역·괘선에서는 바깥 칸의 깊이 1 경로를 돌려줘 Studio 의
//! 칸 크기 조절·칸 블록 선택·괘선 클릭 개체 선택이 모두 동작하지 않았다.

use std::path::Path;

use rhwp::wasm_api::HwpDocument;
use serde_json::Value;

const PAGE: u32 = 0;
const PARENT_PARA: u32 = 5;
/// 중첩 표를 담은 바깥 칸 경로 + 안쪽 표의 첫 칸 (깊이 2).
const NESTED_TABLE_PATH: &str = r#"[{"controlIndex":0,"cellIndex":10,"cellParaIndex":0},{"controlIndex":0,"cellIndex":0,"cellParaIndex":0}]"#;
const OUTER_CELL_PATH: &str = r#"[{"controlIndex":0,"cellIndex":10,"cellParaIndex":0}]"#;

fn load_fixture() -> HwpDocument {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/basic/issue1994_behindtext_table_20200830.hwp");
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", path.display(), e));
    HwpDocument::from_bytes(&bytes).expect("parse issue1994 fixture")
}

fn hit_json(doc: &HwpDocument, x: f64, y: f64) -> Value {
    hit_json_on_page(doc, PAGE, x, y)
}

fn hit_json_on_page(doc: &HwpDocument, page: u32, x: f64, y: f64) -> Value {
    let json = doc
        .hit_test_native(page, x, y)
        .unwrap_or_else(|e| panic!("hit_test_native({page},{x},{y}): {e}"));
    serde_json::from_str(&json).unwrap_or_else(|e| panic!("parse hit json `{json}`: {e}"))
}

fn path_tuples(hit: &Value) -> Vec<(u64, u64, u64)> {
    hit["cellPath"]
        .as_array()
        .map(|a| {
            a.iter()
                .map(|e| {
                    (
                        e["controlIndex"].as_u64().unwrap_or(u64::MAX),
                        e["cellIndex"].as_u64().unwrap_or(u64::MAX),
                        e["cellParaIndex"].as_u64().unwrap_or(u64::MAX),
                    )
                })
                .collect()
        })
        .unwrap_or_default()
}

fn nested_bboxes(doc: &HwpDocument) -> Vec<Value> {
    let json = doc
        .get_table_cell_bboxes_by_path(0, PARENT_PARA, NESTED_TABLE_PATH)
        .expect("getTableCellBboxesByPath nested");
    serde_json::from_str(&json).unwrap_or_else(|e| panic!("parse bboxes `{json}`: {e}"))
}

/// (a) 중첩 칸 빈 영역 — 글자 run이 아닌 칸 내부/괘선 좌표에서도 깊이 2 경로.
#[test]
fn nested_cell_empty_area_returns_innermost_cell_path() {
    let doc = load_fixture();
    // 수정 전 실측: 이 좌표들은 중첩 칸 bbox 내부이지만 깊이 1을 돌려주었다.
    for (x, y) in [(59.0, 406.0), (59.9, 406.0), (61.5, 406.0)] {
        let hit = hit_json(&doc, x, y);
        let path = path_tuples(&hit);
        assert_eq!(
            hit["parentParaIndex"].as_u64(),
            Some(PARENT_PARA as u64),
            "outer table hit expected at ({x},{y}), hit={hit}"
        );
        assert_eq!(
            path.len(),
            2,
            "nested cell empty area must return depth-2 cellPath at ({x},{y}), hit={hit}"
        );
        assert_eq!(path[0], (0, 10, 0), "outer entry at ({x},{y}), hit={hit}");
        assert_eq!(path[1], (0, 0, 0), "inner entry at ({x},{y}), hit={hit}");
    }
}

/// (a-2) 모든 중첩 칸의 내부 그리드 샘플이 깊이 2 경로를 돌려주는지 전수 확인.
#[test]
fn every_nested_cell_interior_returns_inner_path() {
    let doc = load_fixture();
    let bboxes = nested_bboxes(&doc);
    assert!(
        bboxes.len() >= 40,
        "nested 18x9 cell bboxes: {}",
        bboxes.len()
    );
    let mut checked = 0;
    for b in &bboxes {
        let (x, y, w, h) = (
            b["x"].as_f64().unwrap(),
            b["y"].as_f64().unwrap(),
            b["w"].as_f64().unwrap(),
            b["h"].as_f64().unwrap(),
        );
        let page = b["pageIndex"].as_u64().unwrap() as u32;
        for fy in [0.25, 0.5, 0.75] {
            for fx in [0.25, 0.5, 0.75] {
                let hit = hit_json_on_page(&doc, page, x + w * fx, y + h * fy);
                assert_eq!(hit["sectionIndex"].as_u64(), Some(0), "bbox={b}, hit={hit}");
                assert_eq!(
                    hit["parentParaIndex"].as_u64(),
                    Some(PARENT_PARA as u64),
                    "bbox={b}, hit={hit}"
                );
                let path = path_tuples(&hit);
                assert_eq!(path.len(), 2, "bbox={b}, hit={hit}");
                assert_eq!(path[0], (0, 10, 0), "bbox={b}, hit={hit}");
                assert_eq!(path[1].0, 0, "bbox={b}, hit={hit}");
                // 같은 깊이의 텍스트 run은 셀 bbox보다 우선한다. 이 검사는
                // leaf 칸의 기하학적 소유가 아니라 안쪽 표 경로 보존을 검증한다.
                checked += 1;
            }
        }
    }
    assert_eq!(checked, bboxes.len() * 9);
}

/// (b) 중첩 표 내부 괘선 좌표도 깊이 2 (수정 전후 모두 — 회귀 가드).
#[test]
fn nested_internal_border_keeps_inner_path() {
    let doc = load_fixture();
    let hit = hit_json(&doc, 67.0, 414.3);
    let path = path_tuples(&hit);
    assert_eq!(path.len(), 2, "border hit={hit}");
    assert_eq!(path[0], (0, 10, 0), "hit={hit}");
}

/// (c) 대조군: 중첩 표를 담지 않은 바깥 표 칸은 여전히 깊이 1.
#[test]
fn plain_outer_cell_stays_depth_one() {
    let doc = load_fixture();
    // 바깥 표의 평범한 칸(칸 0) 중심에서 hitTest — 칸 10(중첩 소유 칸)과 무관.
    let json = doc
        .get_table_cell_bboxes(0, PARENT_PARA, 0, Some(PAGE))
        .expect("flat cell bboxes");
    let cells: Vec<Value> = serde_json::from_str(&json).expect("parse flat bboxes");
    let cell = cells
        .iter()
        .find(|c| c["cellIdx"].as_u64() == Some(0))
        .expect("outer cell 0");
    let x = cell["x"].as_f64().unwrap() + cell["w"].as_f64().unwrap() / 2.0;
    let y = cell["y"].as_f64().unwrap() + cell["h"].as_f64().unwrap() / 2.0;
    let hit = hit_json(&doc, x, y);
    let path = path_tuples(&hit);
    assert_eq!(
        path.len(),
        1,
        "plain outer cell must stay depth-1, hit={hit}"
    );
    assert_eq!(path[0].1, 0, "hit={hit}");
}

/// (d) 대조군: 현재 가시 글자 경계의 커서→hitTest 왕복이 중첩 경로·offset을 보존한다.
#[test]
fn nested_text_hit_keeps_same_offset() {
    let doc = load_fixture();
    let model_path = [(0, 10, 0), (0, 0, 0)];
    let text = doc
        .get_text_in_cell_by_path(0, PARENT_PARA as usize, &model_path, 0, 2)
        .expect("중첩 첫 칸의 모델 글자");
    assert_eq!(text, "* ", "가시 별표 뒤의 공백이 있는 원본 칸");

    // 뒤 공백의 offset 2는 좁은 칸 밖으로 나가 커서가 칸 끝으로 제한된다.
    // 과거 절대 클릭 좌표를 그 offset으로 고정하지 않고, 가시 별표의 앞뒤
    // 현재 커서 경계에서 편집 경로와 문자 소유를 검증한다.
    for offset in 0..=1 {
        let raw = doc
            .get_cursor_rect_by_path(0, PARENT_PARA, NESTED_TABLE_PATH, offset)
            .expect("중첩 글자 경계의 커서");
        let rect: Value = serde_json::from_str(&raw).expect("커서 JSON");
        let height = rect["height"].as_f64().expect("커서 높이");
        assert!(height > 0.0, "가시 커서 높이: {rect}");
        let hit = hit_json_on_page(
            &doc,
            rect["pageIndex"].as_u64().expect("커서 쪽") as u32,
            rect["x"].as_f64().expect("커서 x"),
            rect["y"].as_f64().expect("커서 y") + height / 2.0,
        );
        assert_eq!(hit["sectionIndex"].as_u64(), Some(0), "hit={hit}");
        assert_eq!(
            hit["parentParaIndex"].as_u64(),
            Some(PARENT_PARA as u64),
            "hit={hit}"
        );
        assert_eq!(
            path_tuples(&hit),
            vec![(0, 10, 0), (0, 0, 0)],
            "글자 커서의 전체 중첩 경로: offset={offset}, hit={hit}"
        );
        assert_eq!(
            hit["charOffset"].as_u64(),
            Some(offset as u64),
            "가시 문자 경계의 offset 왕복: offset={offset}, rect={rect}, hit={hit}"
        );
    }
}

/// (e) getTableCellBboxesByPath 의 cellIdx 가 모델 셀 인덱스와 일치하는지 검증.
/// cellIdx로 경로를 만들어 getCellInfoByPath 를 부르면 같은 row/col 이 나와야 한다.
#[test]
fn path_bbox_cell_idx_matches_model_index() {
    let doc = load_fixture();
    let bboxes = nested_bboxes(&doc);
    assert!(!bboxes.is_empty());
    for b in &bboxes {
        let cell_idx = b["cellIdx"].as_u64().unwrap();
        let path_json = format!(
            r#"[{{"controlIndex":0,"cellIndex":10,"cellParaIndex":0}},{{"controlIndex":0,"cellIndex":{cell_idx},"cellParaIndex":0}}]"#
        );
        let info_json = doc
            .get_cell_info_by_path(0, PARENT_PARA, &path_json)
            .unwrap_or_else(|e| panic!("getCellInfoByPath cellIdx {cell_idx}: {e:?}"));
        let info: Value = serde_json::from_str(&info_json).expect("parse cell info");
        assert_eq!(
            info["row"].as_u64(),
            b["row"].as_u64(),
            "cellIdx {cell_idx} row mismatch: bbox={b} info={info}"
        );
        assert_eq!(
            info["col"].as_u64(),
            b["col"].as_u64(),
            "cellIdx {cell_idx} col mismatch: bbox={b} info={info}"
        );
    }
}

/// 참고: 바깥 칸 경로만 주면 바깥 표 칸 bbox를 돌려주는 기존 계약은 유지.
#[test]
fn outer_path_still_returns_outer_bboxes() {
    let doc = load_fixture();
    let json = doc
        .get_table_cell_bboxes_by_path(0, PARENT_PARA, OUTER_CELL_PATH)
        .expect("outer path bboxes");
    let cells: Vec<Value> = serde_json::from_str(&json).expect("parse");
    assert!(!cells.is_empty(), "outer cell bboxes");
}

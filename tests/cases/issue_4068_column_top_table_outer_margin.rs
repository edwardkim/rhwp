//! 검사 유형: 관계형·고정형.
//! 2026-10-28 측정 raw(96dpi, PDFium):
//! pic-in-table-with-toggle.hwp: 1쪽 Native·fresh WASM 각 100.00000%; 전1쪽 최저 각 100.00000%.
//! 정책연구용역사업 중간진도보고서 HWP: 24쪽 Native 96.57615% / fresh WASM 96.62165%; 전215쪽 최저 59.29844% / 58.93395%.
//! 2912695_civil_petition_form.hwp: 2쪽 Native·fresh WASM 각 100.00000%; 전2쪽 최저 각 100.00000%.
//! 근거: mydocs/working/assets/review_integration_20261026/validation/post-rebase-8328/header-evidence-ted3-matrix.json,
//! mydocs/working/assets/review_integration_20261026/validation/post-rebase-8328/original-floatfix-native-comparison.json,
//! mydocs/working/assets/review_integration_20261026/validation/post-rebase-8328/original-floatfix-wasm-comparison.json.
//! 독립 Print 표 경계: mydocs/working/assets/review_integration_20261026/validation/post-rebase-8328/outer-margin-pdf-boundaries.json.
//!
//! [Issue #4068] 단 맨 위에 선 문단 기준 자리차지 표는 테두리를 바깥 여백
//! (`outMargin.top`) 아래에 그린다.
//!
//! 이슈의 "셀 안 부동 그림이 한/글보다 4~6px 위" 잔여는 그림이 아니라 표 테두리가
//! 바깥 위 여백만큼 위에 있던 결과다. 세 형상을 한/글 정본으로 고정한다.
//!
//! | 형상 | 표본 | 정본 |
//! | --- | --- | --- |
//! | 구역 첫 문단 앵커 | `pic-in-table-with-toggle.hwp` 1쪽 | 표 윗변 = 본문 + 원본 위 여백, 뒤 그림은 표 아래 + 원본 아래 여백 |
//! | 앞 쪽에서 넘어온 표 | `정책연구용역사업 중간진도보고서…` 24쪽 | 이전 단 vOff를 버리고 본문 + 원본 위 여백 |
//! | 같은 쪽 둘째 표 | `2912695_civil_petition_form.hwp` 2쪽 | 첫 표의 원본 위 여백 적용, 둘째 표는 저장 흐름 유지 |
//!
//! 절대 좌표 대신 같은 쪽 노드 사이의 관계(본문 윗변·앞 표 아랫변·뒤 그림)를 잰다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::table::Table;
use serde_json::Value;
use std::path::Path;

fn source_table(core: &DocumentCore, paragraph: usize) -> &Table {
    core.document().sections[0].paragraphs[paragraph]
        .controls
        .iter()
        .find_map(|control| match control {
            Control::Table(table) => Some(table.as_ref()),
            _ => None,
        })
        .expect("원본 문단의 표")
}

fn hu_to_px(hu: f64) -> f64 {
    hu * 96.0 / 7200.0
}

fn core(rel: &str) -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    DocumentCore::from_bytes(
        &std::fs::read(&path).unwrap_or_else(|e| panic!("원본 {rel} 읽기: {e}")),
    )
    .unwrap_or_else(|e| panic!("원본 {rel} 열기: {e:?}"))
}

#[derive(Debug, Clone)]
struct Node {
    kind: String,
    y: f64,
    h: f64,
}

fn page_tree(core: &DocumentCore, page: u32) -> Value {
    let tree = core
        .build_page_render_tree(page)
        .unwrap_or_else(|e| panic!("{}쪽 렌더 트리: {e:?}", page + 1));
    serde_json::from_str(&tree.root.to_json()).expect("렌더 트리 JSON")
}

fn bbox(node: &Value) -> (f64, f64) {
    let b = node.get("bbox").expect("bbox");
    let f = |k: &str| b.get(k).and_then(Value::as_f64).unwrap_or(0.0);
    (f("y"), f("h"))
}

fn find_first<'a>(node: &'a Value, kind: &str) -> Option<&'a Value> {
    if node.get("type").and_then(Value::as_str) == Some(kind) {
        return Some(node);
    }
    node.get("children")
        .and_then(Value::as_array)?
        .iter()
        .find_map(|child| find_first(child, kind))
}

fn column(tree: &Value) -> &Value {
    find_first(tree, "Column").expect("Column 노드")
}

/// 단(Column) 바로 아래 노드들.
fn column_children(tree: &Value) -> Vec<Node> {
    column(tree)
        .get("children")
        .and_then(Value::as_array)
        .expect("Column children")
        .iter()
        .map(|child| {
            let (y, h) = bbox(child);
            Node {
                kind: child
                    .get("type")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string(),
                y,
                h,
            }
        })
        .collect()
}

/// 단 바로 아래 표들을 위에서부터.
fn top_level_tables(tree: &Value) -> Vec<Node> {
    let mut tables: Vec<Node> = column_children(tree)
        .into_iter()
        .filter(|n| n.kind == "Table")
        .collect();
    tables.sort_by(|a, b| a.y.total_cmp(&b.y));
    tables
}

fn body_top(tree: &Value) -> f64 {
    bbox(find_first(tree, "Body").expect("Body 노드")).0
}

#[test]
fn issue_4068_section_first_anchor_table_frame_sits_below_outer_margin() {
    let doc = core("samples/pic-in-table-with-toggle.hwp");
    let tree = page_tree(&doc, 0);
    let top = body_top(&tree);
    let table = top_level_tables(&tree)
        .into_iter()
        .next()
        .expect("1쪽 자리차지 표");
    assert!(
        (table.y - 136.01).abs() <= 2.0,
        "독립 Print 1쪽 표 윗변: {:.2}px",
        table.y
    );
    assert!(
        (table.y + table.h - 652.57).abs() <= 2.0,
        "독립 Print 1쪽 표 아랫변: {:.2}px",
        table.y + table.h
    );
    let om = hu_to_px(f64::from(source_table(&doc, 0).outer_margin_top));

    assert!(
        (table.y - top - om).abs() < 0.3,
        "구역 첫 문단에 앵커된 표 윗변은 본문 윗변 + outMargin.top({om:.2}px): 표 {:.2}, 본문 {top:.2}",
        table.y
    );

    // 흐름은 그대로다: 표 뒤 글자처럼 그림은 표 아랫변 + outMargin.bottom 에 선다
    // (저장 다음 줄 vpos 39354 = 283 + 38788 + 283).
    let next_image = column_children(&tree)
        .into_iter()
        .filter(|n| n.kind == "Image" && n.y >= table.y + table.h - 0.5)
        .min_by(|a, b| a.y.total_cmp(&b.y))
        .expect("표 뒤 글자처럼 그림");
    let bottom_margin = hu_to_px(f64::from(source_table(&doc, 0).outer_margin_bottom));
    let gap = next_image.y - (table.y + table.h);
    assert!(
        (gap - bottom_margin).abs() < 0.5,
        "표 아랫변과 뒤 그림 사이는 원본 outMargin.bottom({bottom_margin:.2}px): {gap:.2}"
    );

    // 칸 안 부동 그림은 표와 함께 움직여 첫 칸 안에 남는다.
    let table_node = column(&tree)
        .get("children")
        .and_then(Value::as_array)
        .unwrap()
        .iter()
        .find(|n| n.get("type").and_then(Value::as_str) == Some("Table"))
        .unwrap();
    let (cell_y, cell_h) = bbox(find_first(table_node, "Cell").expect("첫 칸"));
    let (img_y, img_h) = bbox(find_first(table_node, "Image").expect("칸 안 그림"));
    assert!(
        img_y >= cell_y && img_y + img_h <= cell_y + cell_h,
        "칸 안 그림 ({img_y:.2}+{img_h:.2}) 이 첫 칸 ({cell_y:.2}+{cell_h:.2}) 안에 있어야 한다"
    );
}

#[test]
fn issue_4068_table_carried_to_next_page_drops_vertical_offset_keeps_outer_margin() {
    // `pi=344`: vert=문단(560HU), outMargin 283HU. 앞 쪽 끝에 앵커됐지만(저장 vpos 52230)
    // 자리가 모자라 24쪽 맨 위로 넘어왔다.
    let doc = core(
        "samples/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구).hwp",
    );
    let tree = page_tree(&doc, 23);
    let top = body_top(&tree);
    let table = top_level_tables(&tree)
        .into_iter()
        .next()
        .expect("24쪽 첫 표");
    let source = source_table(&doc, 344);
    let om = hu_to_px(f64::from(source.outer_margin_top));
    let v_off = hu_to_px(f64::from(source.common.vertical_offset));
    assert!(
        (table.y - top - om).abs() < 0.3,
        "넘어온 표는 세로 오프셋({v_off:.2}px)을 버리고 본문 윗변 + outMargin.top({om:.2}px): 표 {:.2}, 본문 {top:.2}",
        table.y
    );
}

#[test]
fn issue_4068_column_top_margin_edit_advances_the_following_table_once() {
    // 2쪽: 단 맨 위 표(`pi=3`, outMargin 141HU) 와 그 아래 표(`pi=4`).
    // 정본(`pdf/2912695_civil_petition_form-2020.pdf`): 첫 표 110.40..877.08, 둘째 표 888.92.
    let mut doc = core("samples/issue6032/2912695_civil_petition_form.hwp");
    let tree = page_tree(&doc, 1);
    let top = body_top(&tree);
    let tables = top_level_tables(&tree);
    assert_eq!(tables.len(), 2, "2쪽 표 두 개: {tables:?}");
    let (first, second) = (&tables[0], &tables[1]);
    assert!(
        (first.y - 110.28).abs() <= 2.0,
        "독립 Print 2쪽 첫 표 윗변: {:.2}px",
        first.y
    );
    assert!(
        (first.y + first.h - 876.16).abs() <= 2.0,
        "독립 Print 2쪽 첫 표 아랫변: {:.2}px",
        first.y + first.h
    );
    assert!(
        (second.y - 887.99).abs() <= 2.0,
        "독립 Print 2쪽 다음 표 윗변: {:.2}px",
        second.y
    );

    let om = hu_to_px(f64::from(source_table(&doc, 3).outer_margin_top));
    assert!(
        (first.y - top - om).abs() < 0.3,
        "단 맨 위 표 윗변은 본문 윗변 + outMargin.top({om:.2}px): 표 {:.2}, 본문 {top:.2}",
        first.y
    );

    assert!(
        second.y >= first.y + first.h,
        "둘째 표는 첫 표와 겹치지 않아야 한다"
    );
    // 원본의 저장 흐름은 그대로 읽지만, 여백을 편집한 뒤에는 후속 흐름도
    // 그 차이를 한 번 소비한다. 독립 한컴 Print 쌍에서 141HU를 제거하면
    // 둘째 표도 같은 여백만큼 올라간다. 그림용 여백만 바뀐다는 옛 전제는 틀렸다.
    let mut source = doc.document().clone();
    for control in &mut source.sections[0].paragraphs[3].controls {
        if let Control::Table(table) = control {
            table.outer_margin_top = 0;
        }
    }
    doc.set_document(source);
    let without_margin = top_level_tables(&page_tree(&doc, 1));
    assert_eq!(without_margin.len(), tables.len(), "편집 뒤 표 소유 보존");
    assert!(
        (without_margin[0].y - top).abs() < 0.3,
        "여백0이면 첫 표는 본문 윗변에 놓인다"
    );
    assert!(
        (without_margin[1].y - second.y + om).abs() < 0.3,
        "후속 표는 제거한 원본 여백({om:.2}px)을 한 번 소비한다: 변화 {:.2}px",
        without_margin[1].y - second.y
    );
}

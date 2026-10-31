#![cfg(not(target_arch = "wasm32"))]
//! 셀 영역(cellzone)의 끝 주소는 그리드 좌표가 아니라 칸 주소다(#6619). 테두리는 끝 칸의
//! span 으로 환산하지만, 배경(단색·그림 채우기)은 `end_col + 1` 로 영역을 잡아서 끝 칸이
//! 병합 칸이면 열 경계에서 멈췄다. 한글은 그 병합 칸의 끝까지 칠한다.
//!
//! 표본은 2행 3열 제목 표의 셀 영역 (1행, 0~1칸)에 회색 채우기가 있고, 끝 칸 (1,1) 이
//! 마지막 두 열을 합친 HWPX 다(한글 2024 저장, 글은 채움 글자). 고치기 전에는 배경이 표
//! 폭 544.3px 가운데 264.6px 에서 멈췄다.
use rhwp::model::control::Control;
use rhwp::wasm_api::HwpDocument;
use serde_json::Value;

const SAMPLE: &str = "samples/cellzone-merged-end-fill.hwpx";

fn bbox(node: &Value) -> (f64, f64, f64, f64) {
    let b = &node["bbox"];
    (
        b["x"].as_f64().unwrap(),
        b["y"].as_f64().unwrap(),
        b["w"].as_f64().unwrap(),
        b["h"].as_f64().unwrap(),
    )
}

/// 문서 순서로 처음 나오는 문단 0 의 표 노드.
fn first_table(node: &Value) -> Option<&Value> {
    if node["type"] == "Table" && node["pi"].as_u64() == Some(0) {
        return Some(node);
    }
    node["children"]
        .as_array()
        .into_iter()
        .flatten()
        .find_map(first_table)
}

#[test]
fn cellzone_fill_reaches_end_of_merged_end_cell() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let doc = HwpDocument::from_bytes(&std::fs::read(path).unwrap()).unwrap();
    assert_eq!(doc.page_count(), 1, "독립 한글 출력과 같은 쪽 수");

    // 표본의 전제: 셀 영역이 끝나는 칸이 병합 칸이다.
    let table = doc.document().sections[0].paragraphs[0]
        .controls
        .iter()
        .find_map(|control| match control {
            Control::Table(table) => Some(table),
            _ => None,
        })
        .expect("첫 문단의 제목 표");
    let zone = table
        .zones
        .iter()
        .find(|zone| zone.border_fill_id > 0)
        .expect("채우기가 있는 셀 영역");
    let end_cell = table
        .cells
        .iter()
        .find(|cell| cell.row == zone.end_row && cell.col == zone.end_col)
        .expect("셀 영역 끝 주소의 칸");
    assert!(end_cell.col_span > 1, "셀 영역 끝 칸이 열을 합쳤다");

    let tree: Value = serde_json::from_str(&doc.get_page_render_tree(0).unwrap()).unwrap();
    let title = first_table(&tree).expect("1쪽의 제목 표 노드");
    let (table_x, _, table_w, _) = bbox(title);
    let children = title["children"].as_array().unwrap();
    let row = children
        .iter()
        .find(|c| c["type"] == "Cell" && c["row"].as_u64() == Some(u64::from(zone.start_row)))
        .map(bbox)
        .expect("셀 영역 행의 칸");
    let fill = children
        .iter()
        .filter(|c| c["type"] == "Rect")
        .map(bbox)
        .find(|(_, y, _, _)| (y - row.1).abs() < 0.5)
        .expect("셀 영역 배경 사각형");

    assert!(
        (fill.0 - table_x).abs() < 0.5,
        "배경 왼쪽 {} 이 표 왼쪽 {table_x} 과 같다",
        fill.0
    );
    assert!(
        (fill.0 + fill.2 - (table_x + table_w)).abs() < 0.5,
        "배경 오른쪽 {:.1} 이 병합 끝 칸의 오른쪽(표 오른쪽 {:.1})까지 간다",
        fill.0 + fill.2,
        table_x + table_w
    );
    assert!(
        (fill.3 - row.3).abs() < 0.5,
        "배경 높이가 셀 영역 행 높이와 같다"
    );
}

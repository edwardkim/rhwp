//! 관계형·고정형: 이어받은 일반 행도 독립 Print의 물리 종료선을 소유한다.
//! 편람 실측 Native 2026-10-28 / fresh WASM 2026-10-29: 96dpi, 한컴 2024 Print PDFium, 2px raw 실루엣.
//! HWP 312쪽 Native 100.00000% / fresh WASM 100.00000%.
//! HWP 전 383쪽 최저 Native 99.96463% / fresh WASM 99.96402%.
//! HWPX 312쪽 Native 100.00000% / fresh WASM 100.00000%.
//! HWPX 전 383쪽 최저 Native 99.96463% / fresh WASM 99.96402%.
//! 근거: mydocs/pr/assets/issue_8153/handbook-validation-20261028/README.md.
//! 312쪽의 표·셀·내용 컷을 보존하고 세 세로선의 끝을 ±2px로 확인한다.
//! 근거: issue_8153_ordinary_row_continuation_frame_20261026.md.
#![cfg(not(target_arch = "wasm32"))]
use rhwp::{model::control::Control, paint::RenderProfile, DocumentCore};
use serde_json::{json, Value};
fn table_item(core: &DocumentCore, page_index: u32) -> Value {
    let pages = core.dump_page_items_json(Some(page_index));
    let pages = pages.as_array().unwrap();
    assert_eq!(pages.len(), 1);
    let page = &pages[0];
    assert_eq!(page["pageIndex"], page_index);
    assert_eq!(page["section"], 11);
    let mut found = Vec::new();
    for column in page["columns"].as_array().unwrap() {
        for item in column["items"].as_array().unwrap() {
            if item["kind"] == "partialTable" && item["paraIndex"] == 3 && item["controlIndex"] == 0
            {
                found.push(item.clone());
            }
        }
    }
    assert_eq!(found.len(), 1, "법령 대조표 조각의 소유와 누락·중복");
    found.remove(0)
}
fn verticals(v: &Value, out: &mut Vec<[f64; 4]>) {
    match v {
        Value::Object(o) => {
            if o.get("type").and_then(Value::as_str) == Some("line") {
                let numbers: [Option<f64>; 4] =
                    ["x1", "y1", "x2", "y2"].map(|k| o.get(k).and_then(Value::as_f64));
                if let [Some(x1), Some(y1), Some(x2), Some(y2)] = numbers {
                    if x1 == x2 && y2 > y1 {
                        out.push([x1, y1, x2, y2]);
                    }
                }
            }
            for value in o.values() {
                verticals(value, out);
            }
        }
        Value::Array(a) => {
            for value in a {
                verticals(value, out);
            }
        }
        _ => {}
    }
}
fn check(extension: &str) {
    let input = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("samples/2025 행정업무운영 편람(최종).{extension}"));
    let mut core = DocumentCore::from_bytes(&std::fs::read(input).unwrap()).unwrap();
    core.set_hangul2024_compat(true);
    assert_eq!(core.page_count(), 383, "독립 Print의 전체 쪽수");
    let previous = table_item(&core, 310);
    let current = table_item(&core, 311);
    let following = table_item(&core, 312);
    assert_eq!(current["startRow"], 5);
    assert_eq!(current["endRow"], 6);
    assert_eq!(current["startCut"], json!([10, 1]));
    assert_eq!(current["endCut"], json!([33, 1]));
    assert_eq!(
        previous["endCut"], current["startCut"],
        "앞 조각에서 소비한 내용 보존"
    );
    assert_eq!(
        current["endCut"], following["startCut"],
        "다음 조각의 내용 소유 연속"
    );
    assert_eq!(following["startRow"], current["startRow"]);
    let Control::Table(table) = &core.document().sections[11].paragraphs[3].controls[0] else {
        panic!("원본 표 소유");
    };
    assert_eq!((table.row_count, table.col_count), (103, 2));
    let mut owners: Vec<_> = table.cells.iter().filter(|c| c.row == 5).collect();
    owners.sort_by_key(|cell| cell.col);
    assert_eq!(owners.len(), 2);
    for (column, cell) in owners.iter().enumerate() {
        assert_eq!(usize::from(cell.col), column);
        assert_eq!(cell.row_span, 1);
        assert_eq!(cell.col_span, 1);
        assert_eq!(cell.height, 90787, "원본 선언의 HU 계약");
    }
    let tree = core
        .build_page_layer_tree_with_profile(311, RenderProfile::Print)
        .unwrap();
    let json: Value = serde_json::from_str(&tree.to_json()).unwrap();
    let mut lines = Vec::new();
    verticals(&json, &mut lines);
    lines.sort_by(|a, b| a[0].total_cmp(&b[0]));
    assert_eq!(lines.len(), 3, "같은 표의 바깥/열 경계 세로선");
    for (line, x) in lines.iter().zip([94.55067, 502.51066, 910.47201]) {
        assert!((line[0] - x).abs() <= 2.0, "독립 Print의 열 경계");
        assert!((line[1] - 94.29867).abs() <= 2.0, "표 머리와 시작 경계");
        assert!(
            (line[3] - 644.264).abs() <= 2.0,
            "{extension}: 312쪽 일반 행의 독립 Print 세로 종료선 644.264±2px, 실제 {}",
            line[3]
        );
    }
}
#[test]
fn hwp_continuing_ordinary_row_keeps_source_cut_and_physical_end() {
    check("hwp");
}
#[test]
fn hwpx_continuing_ordinary_row_keeps_source_cut_and_physical_end() {
    check("hwpx");
}

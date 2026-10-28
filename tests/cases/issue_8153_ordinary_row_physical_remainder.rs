//! 관계형·고정형: 내용 프레임 수와 분리해 일반 행의 물리 잔여·다음 컷을 보존한다.
//! 편람 실측 Native 2026-10-28 / fresh WASM 2026-10-29: 96dpi, 한컴 2024 Print PDFium, 2px raw 실루엣.
//! HWP 317쪽 Native 100.00000% / fresh WASM 100.00000%.
//! HWP 327쪽 Native 100.00000% / fresh WASM 100.00000%.
//! HWP 353쪽 Native 100.00000% / fresh WASM 100.00000%.
//! HWP 전 383쪽 최저 Native 99.96463% / fresh WASM 99.96402%.
//! HWPX 317쪽 Native 100.00000% / fresh WASM 100.00000%.
//! HWPX 327쪽 Native 100.00000% / fresh WASM 100.00000%.
//! HWPX 353쪽 Native 100.00000% / fresh WASM 100.00000%.
//! HWPX 전 383쪽 최저 Native 99.96463% / fresh WASM 99.96402%.
//! 근거: mydocs/pr/assets/issue_8153/handbook-validation-20261028/README.md.
//! 이전 후보의 두 편람 전쪽 Native/fresh 표준 WASM 최저 99.96446/99.96402%와 독립 Print 근거는
//! mydocs/plans/issue_8153_ordinary_row_remainder_20261027.md에 연결한다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::{model::control::Control, paint::RenderProfile, DocumentCore};
use serde_json::{json, Value};

fn table_item(core: &DocumentCore, index: u32) -> Value {
    let pages = core.dump_page_items_json(Some(index));
    let pages = pages.as_array().unwrap();
    assert_eq!(pages.len(), 1);
    let page = &pages[0];
    assert_eq!(page["pageIndex"], index);
    assert_eq!(page["section"], 11);
    let found: Vec<_> = page["columns"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|column| column["items"].as_array().unwrap())
        .filter(|item| {
            item["kind"] == "partialTable" && item["paraIndex"] == 3 && item["controlIndex"] == 0
        })
        .cloned()
        .collect();
    assert_eq!(found.len(), 1, "원본 표 조각의 누락·중복·소유");
    found[0].clone()
}

fn verticals(value: &Value, out: &mut Vec<[f64; 4]>) {
    match value {
        Value::Object(object) => {
            if object.get("type").and_then(Value::as_str) == Some("line") {
                let points =
                    ["x1", "y1", "x2", "y2"].map(|key| object.get(key).and_then(Value::as_f64));
                if let [Some(x1), Some(y1), Some(x2), Some(y2)] = points {
                    if x1 == x2 && y2 > y1 {
                        out.push([x1, y1, x2, y2]);
                    }
                }
            }
            for child in object.values() {
                verticals(child, out);
            }
        }
        Value::Array(array) => {
            for child in array {
                verticals(child, out);
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
    assert_eq!(core.page_count(), 383, "독립 Print 쪽수");
    let original = core.export_hwp_native().unwrap();
    for (page, row, start, end, height) in [
        (317, 12, json!([4, 4]), json!([17, 27]), 157036),
        (327, 22, json!([7, 7]), json!([30, 30]), 68028),
        (353, 71, json!([10, 10]), json!([20, 33]), 102320),
    ] {
        let previous = table_item(&core, page - 2);
        let current = table_item(&core, page - 1);
        let following = table_item(&core, page);
        assert_eq!(current["startRow"], row);
        assert_eq!(current["endRow"], row + 1);
        assert_eq!(current["startCut"], start);
        assert_eq!(current["endCut"], end);
        assert_eq!(previous["endCut"], current["startCut"], "앞 컷 보존");
        assert_eq!(current["endCut"], following["startCut"], "다음 내용 보존");
        assert_eq!(following["startRow"], row, "같은 행의 원본 잔여 소유");
        let Control::Table(table) = &core.document().sections[11].paragraphs[3].controls[0] else {
            panic!("원본 법령 대조표");
        };
        assert_eq!((table.row_count, table.col_count), (103, 2));
        let owners: Vec<_> = table
            .cells
            .iter()
            .filter(|cell| u32::from(cell.row) == row)
            .collect();
        assert_eq!(owners.len(), 2);
        for cell in owners {
            assert_eq!((cell.row_span, cell.col_span), (1, 1));
            assert_eq!(cell.height, height, "원본 HWP/HWPX의 독립 HU 최소");
        }
        let tree = core
            .build_page_layer_tree_with_profile(page - 1, RenderProfile::Print)
            .unwrap();
        let value = serde_json::from_str(&tree.to_json()).unwrap();
        let mut lines = Vec::new();
        verticals(&value, &mut lines);
        lines.sort_by(|a, b| a[0].total_cmp(&b[0]));
        assert_eq!(lines.len(), 3, "표 양 끝과 두 열 사이 경계");
        // 독립 Print의 96dpi 표 외곽이다. 구현의 상자 높이에서 복사하지 않는다.
        for (line, x) in lines.iter().zip([94.550669, 502.510661, 910.472005]) {
            assert!((line[0] - x).abs() <= 2.0);
            assert!((line[1] - 94.298665).abs() <= 2.0);
            assert!(
                (line[3] - 644.263997).abs() <= 2.0,
                "{extension} {page}쪽: 물리 잔여를 넘긴 끝선 644.263997±2px, 실제 {}",
                line[3]
            );
        }
    }
    assert_eq!(core.export_hwp_native().unwrap(), original);
}

#[test]
fn hwp_ordinary_row_physical_remainder_survives_multiple_content_frames() {
    check("hwp");
}

#[test]
fn hwpx_ordinary_row_physical_remainder_survives_multiple_content_frames() {
    check("hwpx");
}

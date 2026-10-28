//! 관계형·고정형: 여러 저장 프레임을 가진 일반 행의 첫 물리 상자와 내용 소유.
//! 편람 실측 Native 2026-10-28 / fresh WASM 2026-10-29: 96dpi, 한컴 2024 Print PDFium, 2px raw 실루엣.
//! HWP 316쪽 Native 100.00000% / fresh WASM 100.00000%.
//! HWP 326쪽 Native 100.00000% / fresh WASM 100.00000%.
//! HWP 344쪽 Native 100.00000% / fresh WASM 100.00000%.
//! HWP 352쪽 Native 100.00000% / fresh WASM 100.00000%.
//! HWP 전 383쪽 최저 Native 99.96463% / fresh WASM 99.96402%.
//! HWPX 316쪽 Native 100.00000% / fresh WASM 100.00000%.
//! HWPX 326쪽 Native 100.00000% / fresh WASM 100.00000%.
//! HWPX 344쪽 Native 100.00000% / fresh WASM 100.00000%.
//! HWPX 352쪽 Native 100.00000% / fresh WASM 100.00000%.
//! HWPX 전 383쪽 최저 Native 99.96463% / fresh WASM 99.96402%.
//! 근거: mydocs/pr/assets/issue_8153/handbook-validation-20261028/README.md.
//! 독립 근거: issue_8153_ordinary_row_opening_frame_20261026.md.
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
fn check(extension: &str, page: u32) {
    let (start_row, end_row, start_cut, end_cut, original_height) = match page {
        316 => (11, 13, json!([4, 4]), json!([4, 4]), 157036),
        326 => (20, 23, json!([9, 9]), json!([7, 7]), 68028),
        344 => (53, 55, json!([14, 1]), json!([6, 1]), 53411),
        352 => (67, 72, json!([]), json!([10, 10]), 102320),
        _ => unreachable!(),
    };
    let input = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("samples/2025 행정업무운영 편람(최종).{extension}"));
    let mut core = DocumentCore::from_bytes(&std::fs::read(input).unwrap()).unwrap();
    core.set_hangul2024_compat(true);
    assert_eq!(core.page_count(), 383, "독립 Print의 전체 쪽수");
    let previous = table_item(&core, page - 2);
    let current = table_item(&core, page - 1);
    let following = table_item(&core, page);
    assert_eq!(current["startRow"], start_row);
    assert_eq!(current["endRow"], end_row);
    assert_eq!(current["startCut"], start_cut);
    assert_eq!(current["endCut"], end_cut);
    assert_eq!(
        previous["endCut"], current["startCut"],
        "앞 조각의 내용 컷 보존"
    );
    assert_eq!(
        current["endCut"], following["startCut"],
        "다음 내용의 누락·중복 방지"
    );
    assert_eq!(following["startRow"], end_row - 1);
    assert!(
        start_row < end_row - 1,
        "앞 행 뒤에서 새 일반 행을 여는 경계"
    );
    let Control::Table(table) = &core.document().sections[11].paragraphs[3].controls[0] else {
        panic!("원본 표 소유");
    };
    assert_eq!((table.row_count, table.col_count), (103, 2));
    let mut owners: Vec<_> = table
        .cells
        .iter()
        .filter(|cell| u32::from(cell.row) == end_row - 1)
        .collect();
    owners.sort_by_key(|cell| cell.col);
    assert_eq!(owners.len(), 2);
    for (column, cell) in owners.iter().enumerate() {
        assert_eq!(usize::from(cell.col), column);
        assert_eq!((cell.row_span, cell.col_span), (1, 1));
        assert_eq!(cell.height, original_height, "원본 HU 선언의 독립 계약");
    }
    let tree = core
        .build_page_layer_tree_with_profile(page - 1, RenderProfile::Print)
        .unwrap();
    let json: Value = serde_json::from_str(&tree.to_json()).unwrap();
    let mut lines = Vec::new();
    verticals(&json, &mut lines);
    lines.sort_by(|a, b| a[0].total_cmp(&b[0]));
    assert_eq!(lines.len(), 3, "표 바깥선과 열 경계의 소유");
    for (line, x) in lines.iter().zip([94.550669, 502.510661, 910.472005]) {
        assert!((line[0] - x).abs() <= 2.0, "독립 Print의 열 경계");
        assert!((line[1] - 94.298665).abs() <= 2.0, "표 머리의 시작");
        assert!(
            (line[3] - 644.263997).abs() <= 2.0,
            "{extension} {page}쪽: 독립 Print의 일반 행 첫 종료선 644.263997±2px, 실제 {}",
            line[3]
        );
    }
}
#[test]
fn hwp_p316_ordinary_row_opening_owns_physical_frame() {
    check("hwp", 316);
}
#[test]
fn hwp_p326_ordinary_row_opening_owns_physical_frame() {
    check("hwp", 326);
}
#[test]
fn hwp_p344_ordinary_row_opening_owns_physical_frame() {
    check("hwp", 344);
}
#[test]
fn hwp_p352_ordinary_row_opening_owns_physical_frame() {
    check("hwp", 352);
}
#[test]
fn hwpx_p316_ordinary_row_opening_owns_physical_frame() {
    check("hwpx", 316);
}
#[test]
fn hwpx_p326_ordinary_row_opening_owns_physical_frame() {
    check("hwpx", 326);
}
#[test]
fn hwpx_p344_ordinary_row_opening_owns_physical_frame() {
    check("hwpx", 344);
}
#[test]
fn hwpx_p352_ordinary_row_opening_owns_physical_frame() {
    check("hwpx", 352);
}

//! #7513 — 표 셀의 개요 문단도 개요 탐색(`getOutlineNavigation`) 목록에 나온다.
//!
//! 화면은 본문 개요와 셀 개요를 `1.`·`2.` 로 그린다. 목록도 두 항목이어야 하고, 셀
//! 항목에는 그 문단으로 이동할 셀 좌표가 붙는다. 본문 항목의 모양은 그대로다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::wasm_api::HwpDocument;
use serde_json::{json, Value};

const OUTLINE_LEVEL_1: &str = r#"{"headType":"Outline","paraLevel":0}"#;

/// 본문 문단 '본문 개요' 뒤에 1×1 표, 그 셀 문단 '셀 개요'. 둘 다 개요 1수준이다.
fn document() -> (HwpDocument, usize) {
    let mut core = HwpDocument::create_empty();
    core.create_blank_document_native().unwrap();
    core.insert_text_native(0, 0, 0, "본문 개요").unwrap();
    core.insert_paragraph_native(0, 1).unwrap();
    let created: Value =
        serde_json::from_str(&core.create_table_native(0, 1, 0, 1, 1).unwrap()).unwrap();
    let table_para = created["paraIdx"].as_u64().unwrap() as usize;
    core.insert_text_in_cell_native(0, table_para, 0, 0, 0, 0, "셀 개요")
        .unwrap();
    core.apply_para_format_native(0, 0, OUTLINE_LEVEL_1)
        .unwrap();
    core.apply_para_format_in_cell_native(0, table_para, 0, 0, 0, OUTLINE_LEVEL_1)
        .unwrap();
    (core, table_para)
}

/// 1쪽 SVG `<text>` 내용을 이어 붙이고 공백을 지운 문자열.
fn page_text(core: &DocumentCore) -> String {
    let svg = core.render_page_svg_native(0).unwrap();
    let mut out = String::new();
    let mut rest = svg.as_str();
    while let Some(start) = rest.find("<text") {
        let Some(open_end) = rest[start..].find('>') else {
            break;
        };
        let after = &rest[start + open_end + 1..];
        let Some(close) = after.find("</text>") else {
            break;
        };
        out.push_str(&after[..close]);
        rest = &after[close..];
    }
    out.chars().filter(|c| !c.is_whitespace()).collect()
}

#[test]
fn table_cell_outline_is_listed_with_its_cell_path() {
    let (core, table_para) = document();

    let rendered = page_text(&core);
    assert!(
        rendered.contains("1.본문개요"),
        "본문 개요 번호: {rendered}"
    );
    assert!(rendered.contains("2.셀개요"), "셀 개요 번호: {rendered}");

    let navigation: Value =
        serde_json::from_str(&core.get_outline_navigation_native().unwrap()).unwrap();
    let outline = navigation["outline"].as_array().unwrap();
    let listed: Vec<_> = outline
        .iter()
        .map(|item| {
            (
                item["number"].as_str().unwrap(),
                item["title"].as_str().unwrap(),
            )
        })
        .collect();
    assert_eq!(listed, [("1.", "본문 개요"), ("2.", "셀 개요")]);

    // 본문 항목은 종전 필드만 갖는다.
    assert_eq!(
        outline[0],
        json!({"level": 1, "number": "1.", "title": "본문 개요", "page": 1, "section": 0, "paragraph": 0})
    );

    // 셀 항목: `paragraph` 는 표를 품은 문단, 셀 좌표는 `hitTest` 와 같은 이름이다.
    let path = json!([{"controlIndex": 0, "cellIndex": 0, "cellParaIndex": 0}]);
    assert_eq!(
        outline[1],
        json!({
            "level": 1, "number": "2.", "title": "셀 개요", "page": 1,
            "section": 0, "paragraph": table_para,
            "parentParaIndex": table_para, "controlIndex": 0, "cellIndex": 0, "cellParaIndex": 0,
            "cellPath": path.clone(),
        })
    );

    // 그 경로로 셀 문단의 캐럿 좌표를 바로 얻는다 — 소비자가 이동에 쓰는 경로다.
    let rect: Value = serde_json::from_str(
        &core
            .get_cursor_rect_by_path(0, table_para as u32, &path.to_string(), 0)
            .unwrap(),
    )
    .unwrap();
    assert_eq!(rect["pageIndex"], 0);
}

//! 실제 요약표 삽입 요청 뒤에도 기존 본문의 소유·내용·정렬을 유지한다.
//! 시각 근거: mydocs/pr/assets/table-insertion-flow/manifest.json
#![cfg(not(target_arch = "wasm32"))]
use rhwp::{
    document_core::DocumentCore,
    model::{control::Control, table::VerticalAlign},
};
use serde_json::{json, Value};
use std::{fs, path::Path};

const MARKER: &str = "산·학·연 공동연구";
fn words(s: &str) -> String {
    s.chars()
        .filter(|c| !c.is_whitespace() && !c.is_control() && *c != '\u{fffc}')
        .collect()
}
fn text(node: &Value) -> String {
    let mut s = node["text"].as_str().unwrap_or("").to_owned();
    if let Some(children) = node["children"].as_array() {
        for child in children {
            s.push_str(&text(child));
        }
    }
    s
}
fn find<'a>(node: &'a Value, kind: &str, out: &mut Vec<&'a Value>) {
    if node["type"] == kind {
        out.push(node);
    }
    if let Some(children) = node["children"].as_array() {
        for child in children {
            find(child, kind, out);
        }
    }
}
fn box_value(node: &Value, key: &str) -> f64 {
    node["bbox"][key].as_f64().unwrap()
}
fn replay(core: &mut DocumentCore, trace: &[Value]) {
    for call in trace {
        let a = call["args"].as_array().unwrap();
        let n = |i: usize| usize::try_from(a[i].as_u64().unwrap()).unwrap();
        match call["name"].as_str().unwrap() {
            "saveSnapshot" => {
                core.save_snapshot_native();
            }
            "splitParagraph" => {
                assert!(a[3].is_null());
                core.split_paragraph_native(n(0), n(1), n(2), None).unwrap();
            }
            "createTable" => {
                core.create_table_native(
                    n(0),
                    n(1),
                    n(2),
                    n(3).try_into().unwrap(),
                    n(4).try_into().unwrap(),
                )
                .unwrap();
            }
            "insertTextInCellDeferredPagination" => {
                core.insert_text_in_cell_native_deferred_pagination(
                    n(0),
                    n(1),
                    n(2),
                    n(3),
                    n(4),
                    n(5),
                    a[6].as_str().unwrap(),
                )
                .unwrap();
            }
            "insertTextInCell" => {
                core.insert_text_in_cell_native(
                    n(0),
                    n(1),
                    n(2),
                    n(3),
                    n(4),
                    n(5),
                    a[6].as_str().unwrap(),
                )
                .unwrap();
            }
            "splitParagraphInCell" => {
                assert!(a[6].is_null());
                core.split_paragraph_in_cell_native(n(0), n(1), n(2), n(3), n(4), n(5), None)
                    .unwrap();
            }
            "flushDeferredPagination" => {
                core.flush_deferred_pagination();
            }
            other => panic!("재현에 없는 API: {other}"),
        }
    }
}
fn relations(core: &DocumentCore, expected: &str) -> Value {
    let count = core.page_count();
    let pages: Vec<Value> = (0..count)
        .map(|page| {
            serde_json::from_str(&core.build_page_render_tree(page).unwrap().root.to_json())
                .unwrap()
        })
        .collect();
    let mut tail = Vec::new();
    let mut penalty = Vec::new();
    for (page, tree) in pages.iter().enumerate() {
        let mut tables = Vec::new();
        find(tree, "Table", &mut tables);
        for table in tables {
            if words(&text(table)).starts_with("감점사항") {
                penalty.push((page, table));
            }
            let mut cells = Vec::new();
            find(table, "Cell", &mut cells);
            for cell in cells {
                if text(cell).contains(MARKER) {
                    tail.push((page, table, cell));
                }
            }
        }
    }
    assert_eq!(tail.len(), 1, "실제 꼬리 셀은 한 번만 존재한다");
    let (page, table, cell) = tail[0];
    let mut lines = Vec::new();
    find(cell, "TextLine", &mut lines);
    lines.retain(|line| !text(line).trim().is_empty());
    assert!(!lines.is_empty());
    let top = lines
        .iter()
        .map(|line| box_value(line, "y"))
        .fold(f64::INFINITY, f64::min);
    let bottom = lines
        .iter()
        .map(|line| box_value(line, "y") + box_value(line, "h"))
        .fold(f64::NEG_INFINITY, f64::max);
    let top_gap = top - box_value(cell, "y");
    let bottom_gap = box_value(cell, "y") + box_value(cell, "h") - bottom;
    let checks = json!({
        "independentPageCount": count == 24,
        "tailOwnedByIndependentPage": page == 20,
        "originalTailContentPreserved": words(&text(cell)) == words(expected),
        "tailInsideCell": top_gap >= 0.0 && bottom_gap >= 0.0,
        "tailCentered": (top_gap - bottom_gap).abs() <= 1.0,
        "penaltyExactlyOnceInDocument": penalty.len() == 1,
        "penaltyFollowsTail": penalty.len() == 1 && penalty[0].0 == page && box_value(penalty[0].1, "y") >= box_value(table, "y") + box_value(table, "h"),
    });
    json!({"pageCount":count,"tailPage":page + 1,"cell":cell["bbox"],"lineCount":lines.len(),"topGap":top_gap,"bottomGap":bottom_gap,"centerDifference":(top_gap-bottom_gap).abs(),"actualTailText":text(cell),"expectedTailText":expected,"pass":checks.as_object().unwrap().values().all(|v| v.as_bool() == Some(true)),"checks":checks})
}
#[test]
fn inserted_summary_keeps_existing_body_flow_live_and_after_reopen() {
    let input = fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/pic-in-table-01.hwp"))
        .unwrap();
    let trace: Vec<Value> =
        serde_json::from_slice(include_bytes!("../fixtures/inserted_summary_calls.json")).unwrap();
    assert_eq!(trace.len(), 53);
    let source = rhwp::parse_document(&input).unwrap();
    let mut expected = Vec::new();
    for section in &source.sections {
        for para in &section.paragraphs {
            for control in &para.controls {
                if let Control::Table(table) = control {
                    for cell in &table.cells {
                        if let Some(first) =
                            cell.paragraphs.iter().position(|p| p.text.contains(MARKER))
                        {
                            assert_eq!(cell.vertical_align, VerticalAlign::Center);
                            assert_eq!(cell.padding.top, cell.padding.bottom);
                            assert_eq!(table.padding.top, table.padding.bottom);
                            expected.push(
                                cell.paragraphs[first..]
                                    .iter()
                                    .map(|p| p.text.as_str())
                                    .collect::<String>(),
                            );
                        }
                    }
                }
            }
        }
    }
    assert_eq!(expected.len(), 1);
    let mut core = DocumentCore::from_bytes(&input).unwrap();
    core.convert_to_editable_native().unwrap();
    core.set_hangul2024_compat(false);
    let _ = core.page_count();
    replay(&mut core, &trace);
    // 저장으로 live 측정 캐시를 교체하기 전에 실제 편집 결과부터 검사한다.
    let live = relations(&core, &expected[0]);
    let saved = core.export_hwpx_native().unwrap();
    let mut reopened = DocumentCore::from_bytes(&saved).unwrap();
    reopened.convert_to_editable_native().unwrap();
    reopened.set_hangul2024_compat(false);
    let reopened_result = relations(&reopened, &expected[0]);
    assert_eq!(
        live["pass"], true,
        "편집 중 원본 본문 소유·내용·정렬: {live}"
    );
    assert_eq!(
        reopened_result["pass"], true,
        "재열기 후 원본 본문 소유·내용·정렬: {reopened_result}"
    );
}

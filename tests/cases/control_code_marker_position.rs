//! 조판 부호를 켜면 글자 사이 컨트롤의 부호(`[새 번호]`·`[숨은 설명]` 따위)는 그 자리에 한 번 그린다.
//!
//! 글자 모양이 같은 글 한 덩어리(run) 가운데의 컨트롤은 run 시작에서만 부호를 찾던 탓에 다음 run 앞이나
//! 줄 끝에 그려졌고, 여러 줄 문단에서는 부호를 문단 전체에서 모아 줄마다 다시 그렸다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::wasm_api::HwpDocument;

/// text 의 at 자리에 새 번호를 넣고 조판 부호를 켠 문서.
fn doc_with_new_number(text: &str, at: usize) -> HwpDocument {
    let mut doc = HwpDocument::create_empty();
    doc.create_blank_document_native().unwrap();
    doc.insert_text_native(0, 0, 0, text).unwrap();
    doc.insert_new_number_native(0, 0, at, 1).unwrap();
    doc.set_show_control_codes(true);
    doc
}

/// 첫 쪽의 (y, x, 글) 차례. 글자나 부호가 든 run 만 본다.
fn drawn_runs(doc: &HwpDocument) -> Vec<(f64, f64, String)> {
    let layout: serde_json::Value =
        serde_json::from_str(&doc.get_page_text_layout_native(0).unwrap()).unwrap();
    let mut runs: Vec<(f64, f64, String)> = layout["runs"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|run| {
            let text = run["text"].as_str()?;
            (text.contains('[') || text.chars().any(|c| c.is_ascii_alphanumeric())).then(|| {
                (
                    run["y"].as_f64().unwrap(),
                    run["x"].as_f64().unwrap(),
                    text.to_string(),
                )
            })
        })
        .collect();
    runs.sort_by(|a, b| a.partial_cmp(b).unwrap());
    runs
}

#[test]
fn control_code_inside_a_run_is_drawn_at_its_place() {
    let doc = doc_with_new_number("abcd", 2);
    let texts: Vec<String> = drawn_runs(&doc).into_iter().map(|run| run.2).collect();
    assert_eq!(texts, ["ab", "[새 번호]", "cd"]);
}

#[test]
fn control_code_is_drawn_once_on_its_own_line() {
    let doc = doc_with_new_number(&"word ".repeat(60), 2);
    let runs = drawn_runs(&doc);
    let lines: std::collections::BTreeSet<i64> =
        runs.iter().map(|run| run.0.round() as i64).collect();
    assert!(lines.len() > 1, "여러 줄 문단이다: {runs:?}");
    let marks: Vec<_> = runs.iter().filter(|run| run.2 == "[새 번호]").collect();
    assert_eq!(marks.len(), 1, "{runs:?}");
    assert_eq!(marks[0].0, runs[0].0, "첫 줄에 그린다");
}

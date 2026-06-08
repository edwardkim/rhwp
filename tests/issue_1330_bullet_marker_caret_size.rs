//! Issue #1330: Enter로 생성한 빈 글머리표 줄에 입력하면 marker/caret 크기가 커짐.
//!
//! `Paragraph::split_at()`은 새 빈 문단의 활성 CharShape를 보존하지만, 렌더러의
//! 빈 runs fallback 이 기본 CharShape(0)를 사용하면 빈 상태 marker/caret 만 작게
//! 보이다가 입력 후 실제 run 스타일로 커진다.

use serde_json::Value;

fn parse_json(json: String, label: &str) -> Value {
    serde_json::from_str(&json).unwrap_or_else(|e| panic!("parse {label}: {e}"))
}

fn text_layout(doc: &rhwp::wasm_api::HwpDocument) -> Value {
    parse_json(
        doc.get_page_text_layout_native(0)
            .unwrap_or_else(|e| panic!("text layout: {e:?}")),
        "text layout",
    )
}

fn cursor_rect(doc: &rhwp::wasm_api::HwpDocument, para_idx: usize, char_offset: usize) -> Value {
    parse_json(
        doc.get_cursor_rect_native(0, para_idx, char_offset)
            .unwrap_or_else(|e| panic!("cursor para={para_idx} offset={char_offset}: {e:?}")),
        "cursor rect",
    )
}

fn run_font_size(run: &Value, label: &str) -> f64 {
    run["fontSize"]
        .as_f64()
        .unwrap_or_else(|| panic!("{label} fontSize missing: {run}"))
}

fn cursor_height(rect: &Value, label: &str) -> f64 {
    rect["height"]
        .as_f64()
        .unwrap_or_else(|| panic!("{label} height missing: {rect}"))
}

fn para_runs(layout: &Value, para_idx: u64) -> Vec<&Value> {
    layout["runs"]
        .as_array()
        .expect("runs")
        .iter()
        .filter(|run| run["paraIdx"].as_u64() == Some(para_idx))
        .collect()
}

fn empty_anchor_run<'a>(runs: &'a [&'a Value], label: &str) -> &'a Value {
    runs.iter()
        .copied()
        .find(|run| run["charStart"].as_u64() == Some(0) && run["text"].as_str() == Some(""))
        .unwrap_or_else(|| panic!("{label} empty anchor run missing: {runs:?}"))
}

fn body_run<'a>(runs: &'a [&'a Value], text: &str, label: &str) -> &'a Value {
    runs.iter()
        .copied()
        .find(|run| run["charStart"].as_u64() == Some(0) && run["text"].as_str() == Some(text))
        .unwrap_or_else(|| panic!("{label} body run missing: {runs:?}"))
}

fn marker_font_size_near_y(layout: &Value, anchor_y: f64, label: &str) -> f64 {
    let run = layout["runs"]
        .as_array()
        .expect("runs")
        .iter()
        .filter(|run| {
            run.get("charStart").is_none() && run["text"].as_str().is_some_and(|s| !s.is_empty())
        })
        .min_by(|a, b| {
            let ay = (a["y"].as_f64().unwrap_or(f64::INFINITY) - anchor_y).abs();
            let by = (b["y"].as_f64().unwrap_or(f64::INFINITY) - anchor_y).abs();
            ay.partial_cmp(&by).unwrap()
        })
        .unwrap_or_else(|| panic!("{label} marker run missing: {}", layout["runs"]));

    let marker_y = run["y"]
        .as_f64()
        .unwrap_or_else(|| panic!("{label} marker y missing: {run}"));
    assert!(
        (marker_y - anchor_y).abs() <= 1.0,
        "{label} marker is not on the target line: marker_y={marker_y:.1}, anchor_y={anchor_y:.1}, run={run}"
    );
    run_font_size(run, label)
}

fn assert_close(actual: f64, expected: f64, label: &str) {
    assert!(
        (actual - expected).abs() <= 0.1,
        "{label}: actual={actual:.3}, expected={expected:.3}"
    );
}

#[test]
fn split_empty_bullet_line_uses_active_char_shape_before_and_after_typing() {
    let mut doc = rhwp::wasm_api::HwpDocument::create_empty();
    doc.create_blank_document_native()
        .unwrap_or_else(|e| panic!("create blank document: {e:?}"));
    let bullet_id = doc.ensure_default_bullet("□");

    doc.insert_text_native(0, 0, 0, "개념")
        .unwrap_or_else(|e| panic!("insert seed text: {e:?}"));
    doc.apply_char_format_native(0, 0, 0, 2, r#"{"fontSize":1800}"#)
        .unwrap_or_else(|e| panic!("apply char format: {e:?}"));
    doc.apply_para_format_native(
        0,
        0,
        &format!(r#"{{"headType":"Bullet","paraLevel":0,"numberingId":{bullet_id}}}"#),
    )
    .unwrap_or_else(|e| panic!("apply bullet format: {e:?}"));

    let split_result = parse_json(
        doc.split_paragraph_native(0, 0, 2)
            .unwrap_or_else(|e| panic!("split paragraph: {e:?}")),
        "split result",
    );
    assert_eq!(split_result["paraIdx"].as_u64(), Some(1));

    let empty_layout = text_layout(&doc);
    let empty_runs = para_runs(&empty_layout, 1);
    let empty_anchor = empty_anchor_run(&empty_runs, "empty");
    let empty_anchor_size = run_font_size(empty_anchor, "empty anchor");
    let empty_anchor_y = empty_anchor["y"].as_f64().expect("empty anchor y");
    let empty_marker_size = marker_font_size_near_y(&empty_layout, empty_anchor_y, "empty marker");
    let empty_caret_height = cursor_height(&cursor_rect(&doc, 1, 0), "empty caret");

    assert!(
        empty_marker_size > 20.0,
        "빈 글머리표 marker가 기본 크기로 떨어지면 안 됨: size={empty_marker_size}"
    );
    assert_close(
        empty_anchor_size,
        empty_marker_size,
        "empty anchor font size",
    );
    assert_close(
        empty_caret_height,
        empty_marker_size,
        "empty caret height follows active char shape",
    );

    doc.insert_text_native(0, 1, 0, "가")
        .unwrap_or_else(|e| panic!("insert typed text: {e:?}"));

    let typed_layout = text_layout(&doc);
    let typed_runs = para_runs(&typed_layout, 1);
    let typed_body = body_run(&typed_runs, "가", "typed");
    let typed_body_size = run_font_size(typed_body, "typed body");
    let typed_body_y = typed_body["y"].as_f64().expect("typed body y");
    let typed_marker_size = marker_font_size_near_y(&typed_layout, typed_body_y, "typed marker");
    let typed_caret_height = cursor_height(&cursor_rect(&doc, 1, 0), "typed caret");

    assert_close(
        typed_marker_size,
        empty_marker_size,
        "marker size must not jump after typing",
    );
    assert_close(
        typed_body_size,
        empty_anchor_size,
        "typed body should inherit split active char shape",
    );
    assert_close(
        typed_caret_height,
        empty_caret_height,
        "caret height must not jump after typing",
    );
}

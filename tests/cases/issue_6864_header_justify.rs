//! #6864: 저장된 머리말의 양쪽 정렬을 나눔 정렬처럼 늘리지 않는다.

use rhwp::model::control::Control;
use rhwp::model::style::Alignment;
use rhwp::parse_document;
use rhwp::wasm_api::HwpDocument;
use serde_json::Value;

fn sample() -> Vec<u8> {
    std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/samples/issue6864/header-justify.hwp"
    ))
    .expect("#6864 공개 원본 HWP")
}

fn header_runs(node: &Value, inside_header: bool, runs: &mut Vec<(String, f64, f64)>) {
    let inside_header = inside_header || node["type"] == "Header";
    if inside_header && node["type"] == "TextRun" {
        if let Some(text) = node["text"].as_str() {
            if !text.is_empty() {
                let x = node["bbox"]["x"].as_f64().expect("머리말 run x");
                let width = node["bbox"]["w"].as_f64().expect("머리말 run width");
                runs.push((text.to_string(), x, x + width));
            }
        }
    }
    match node {
        Value::Object(fields) => {
            for child in fields.values() {
                header_runs(child, inside_header, runs);
            }
        }
        Value::Array(children) => {
            for child in children {
                header_runs(child, inside_header, runs);
            }
        }
        _ => {}
    }
}

fn assert_natural_header(bytes: &[u8]) {
    let model = parse_document(bytes).expect("문서 파싱");
    let header = model.sections[0]
        .paragraphs
        .iter()
        .flat_map(|paragraph| &paragraph.controls)
        .find_map(|control| match control {
            Control::Header(header) => Some(header),
            _ => None,
        })
        .expect("원본 머리말");
    let paragraph = header
        .paragraphs
        .iter()
        .find(|paragraph| paragraph.text.contains("TEST TEXT"))
        .expect("머리말 TEST TEXT");
    assert_eq!(
        model.doc_info.para_shapes[paragraph.para_shape_id as usize].alignment,
        Alignment::Justify,
        "HWP5/HWPX의 양쪽 정렬 의미를 Split으로 바꾸면 안 된다"
    );
    let doc = HwpDocument::from_bytes(bytes).expect("문서 조판");
    let tree: Value = serde_json::from_str(&doc.get_page_render_tree(0).expect("첫 페이지 트리"))
        .expect("트리 JSON");
    let mut runs = Vec::new();
    header_runs(&tree, false, &mut runs);
    assert_eq!(
        runs.iter().map(|run| run.0.as_str()).collect::<String>(),
        "TEST TEXT"
    );
    let left = runs.iter().map(|run| run.1).fold(f64::INFINITY, f64::min);
    let right = runs
        .iter()
        .map(|run| run.2)
        .fold(f64::NEG_INFINITY, f64::max);
    let width = right - left;
    assert!(
        width > 0.0 && width < 200.0,
        "짧은 한 줄이 페이지 폭으로 늘어나면 안 된다: width={width}, runs={runs:?}"
    );
}

#[test]
fn imported_hwp_header_justify_keeps_last_line_at_natural_width() {
    assert_natural_header(&sample());
}

#[test]
fn header_justify_keeps_natural_width_after_hwp_and_hwpx_roundtrip() {
    let mut doc = HwpDocument::from_bytes(&sample()).expect("원본 읽기");
    assert_natural_header(&doc.export_hwp().expect("HWP 저장"));
    assert_natural_header(&doc.export_hwpx().expect("HWPX 저장"));
}

//! Issue #1308: 강제 줄바꿈 뒤 수식과 커서가 원본 줄의 내어쓰기 축을 보존한다.
//! 같은 원본의 한컴 Print 대조를 선행하고 문단·컨트롤 소유와 상대 순서를 검사한다.

use std::fs;
use std::path::Path;

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::wasm_api::HwpDocument;
use serde_json::Value;

fn load_document(rel: &str) -> HwpDocument {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {rel}: {e}"));
    HwpDocument::from_bytes(&bytes).unwrap_or_else(|e| panic!("parse {rel}: {e:?}"))
}

fn cursor_rect(doc: &HwpDocument, para: u32, offset: u32) -> Value {
    let json = doc
        .get_cursor_rect(0, para, offset)
        .unwrap_or_else(|e| panic!("cursor para={para} offset={offset}: {e:?}"));
    serde_json::from_str(&json.to_string()).expect("커서 사각형 JSON")
}

fn owned_equations(
    node: &RenderNode,
    para: usize,
    control: usize,
    out: &mut Vec<rhwp::renderer::render_tree::BoundingBox>,
) {
    if let RenderNodeType::Equation(equation) = &node.node_type {
        if equation.section_index == Some(0)
            && equation.para_index == Some(para)
            && equation.control_index == Some(control)
            && equation.note_ref.is_none()
            && equation.cell_index.is_none()
        {
            out.push(node.bbox);
        }
    }
    for child in &node.children {
        owned_equations(child, para, control, out);
    }
}

fn equation(
    node: &RenderNode,
    para: usize,
    control: usize,
) -> rhwp::renderer::render_tree::BoundingBox {
    let mut found = Vec::new();
    owned_equations(node, para, control, &mut found);
    assert_eq!(
        found.len(),
        1,
        "본문 문단 {para} 컨트롤 {control}은 한 번 표시되어야 함"
    );
    found[0]
}

fn assert_same_axis(a: f64, b: f64, message: &str) {
    let tolerance = f64::EPSILON * a.abs().max(b.abs()).max(1.0) * 16.0;
    assert!((a - b).abs() <= tolerance, "{message}: {a} / {b}");
}

fn assert_forced_break_indent(rel: &str) {
    let doc = load_document(rel);
    let tree = doc.build_page_render_tree(0).expect("첫 쪽 render tree");
    let before = equation(&tree.root, 4, 0);
    let after = equation(&tree.root, 4, 2);
    assert!(
        after.y >= before.y + before.height,
        "강제 개행 뒤 수식은 앞 줄 아래에 있어야 함: {rel}"
    );
    assert_same_axis(
        after.x,
        before.x,
        "탭으로 시작한 앞 줄과 강제 개행 뒤 내어쓰기의 수식 시작 축",
    );
    assert!(equation(&tree.root, 4, 1).x > before.x);
    assert!(equation(&tree.root, 4, 3).x > after.x);
}

fn assert_forced_break_cursor(rel: &str) {
    let doc = load_document(rel);
    let next: Value = serde_json::from_str(&doc.navigate_next_editable_wasm(0, 4, 3, 1, "[]"))
        .expect("커서 이동 JSON");
    assert_eq!(
        next["charOffset"], 4,
        "강제 개행 뒤 첫 수식 앞에 진입해야 함: {rel}"
    );
    let first = cursor_rect(&doc, 4, 1);
    let second = cursor_rect(&doc, 4, 4);
    assert!(second["y"].as_f64().unwrap() > first["y"].as_f64().unwrap());
    assert_same_axis(
        first["x"].as_f64().unwrap(),
        second["x"].as_f64().unwrap(),
        "앞 줄과 다음 줄 수식 앞 커서의 내어쓰기 축",
    );
}

fn assert_boundary_cursor(rel: &str) {
    let doc = load_document(rel);
    let end = doc.get_paragraph_length(0, 0).expect("문단 길이");
    let next: Value = serde_json::from_str(&doc.navigate_next_editable_wasm(0, 0, end, 1, "[]"))
        .expect("문단 경계 커서 이동 JSON");
    assert_eq!(next["para"], 1);
    assert_eq!(next["charOffset"], 0, "수식 앞에 진입해야 함: {rel}");
}

fn comma_run(node: &RenderNode, out: &mut Vec<rhwp::renderer::render_tree::BoundingBox>) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if run.para_index == Some(6) && run.text.contains(',') {
            out.push(node.bbox);
        }
    }
    for child in &node.children {
        comma_run(child, out);
    }
}

fn assert_original_tac_order(rel: &str) {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    let parsed =
        rhwp::parser::parse_document(&fs::read(path).expect("원본 파일")).expect("원본 파싱");
    let composed = rhwp::renderer::composer::compose_paragraph(&parsed.sections[0].paragraphs[6]);
    assert_eq!(
        composed
            .tac_controls
            .iter()
            .map(|(position, _, _)| *position)
            .collect::<Vec<_>>(),
        vec![0, 0, 2, 2, 4]
    );
    let doc = load_document(rel);
    let tree = doc.build_page_render_tree(0).expect("첫 쪽 render tree");
    let first = equation(&tree.root, 6, 0);
    let reference = equation(&tree.root, 1, 0);
    assert_same_axis(
        first.x,
        reference.x,
        "문단 시작 수식은 앞 문단의 같은 본문 시작 축을 사용",
    );
    let mut commas = Vec::new();
    comma_run(&tree.root, &mut commas);
    assert_eq!(commas.len(), 1);
    assert!(
        first.x + first.width <= commas[0].x,
        "첫 수식은 쉼표 앞에 있어야 함"
    );
    let second = equation(&tree.root, 6, 1);
    assert!(second.x >= first.x + first.width && second.x + second.width <= commas[0].x);
    let third = equation(&tree.root, 6, 2);
    assert!(third.x >= commas[0].x + commas[0].width);
    let fourth = equation(&tree.root, 6, 3);
    let last = equation(&tree.root, 6, 4);
    assert!(fourth.x >= third.x + third.width && last.x >= fourth.x + fourth.width);
}

#[test]
fn issue7642_hwp_forced_break_tac_equation_line_uses_hanging_indent() {
    assert_forced_break_indent("samples/eq-002.hwp");
}
#[test]
fn issue7642_hwpx_forced_break_tac_equation_line_uses_hanging_indent() {
    assert_forced_break_indent("samples/hwpx/eq-002.hwpx");
}
#[test]
fn issue7642_hwp_tac_equation_text_and_fixed_tabs_keep_editor_order() {
    assert_original_tac_order("samples/eq-002.hwp");
}
#[test]
fn issue7642_hwpx_tac_equation_text_and_fixed_tabs_keep_editor_order() {
    assert_original_tac_order("samples/hwpx/eq-002.hwpx");
}
#[test]
fn issue7642_hwp_cursor_navigation_enters_forced_break_tac_line() {
    assert_forced_break_cursor("samples/eq-002.hwp");
}
#[test]
fn issue7642_hwpx_cursor_navigation_enters_forced_break_tac_line() {
    assert_forced_break_cursor("samples/hwpx/eq-002.hwpx");
}
#[test]
fn issue7642_hwp_cursor_navigation_enters_next_paragraph_before_leading_tac() {
    assert_boundary_cursor("samples/eq-002.hwp");
}
#[test]
fn issue7642_hwpx_cursor_navigation_enters_next_paragraph_before_leading_tac() {
    assert_boundary_cursor("samples/hwpx/eq-002.hwpx");
}

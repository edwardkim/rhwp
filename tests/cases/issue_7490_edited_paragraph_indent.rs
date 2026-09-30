//! [#7490] 편집한 문단에도 들여쓰기·내어쓰기가 그려져야 한다.
//!
//! 렌더러는 저장 줄의 `TAG_INDENTATION`(bit 20)이 꺼져 있으면 그 줄에 문단
//! `indent` 를 얹지 않는다(#6190). 편집 재조판이 이 비트를 비운 채 줄을 발행하면
//! 문단 모양에 저장된 들여쓰기·내어쓰기가 화면에서 사라진다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::wasm_api::HwpDocument;

const TEXT: &str = "1) 가나다라마바사아자차카타파하 가나다라마바사아자차카타파하 \
가나다라마바사아자차카타파하 가나다라마바사아자차카타파하 가나다라마바사아자차카타파하";

/// 새 문서 첫 문단에 두 줄 넘게 입력하고 문단 모양 `indent` 를 준 뒤의 줄별 시작 x.
fn typed_line_starts(indent: i32) -> Vec<f64> {
    let mut doc = HwpDocument::create_empty();
    doc.create_blank_document_native().expect("blank document");
    doc.insert_text_native(0, 0, 0, TEXT).expect("insert text");
    doc.apply_para_format_native(0, 0, &format!(r#"{{"indent":{indent}}}"#))
        .expect("apply indent");
    let starts = line_starts(&doc, 0);
    assert!(starts.len() >= 2, "두 줄 넘게 입력해야 한다: {starts:?}");
    starts
}

#[test]
fn hanging_indent_moves_following_lines_of_typed_paragraph() {
    let flat = typed_line_starts(0);
    let hanging = typed_line_starts(-3000);
    assert!(
        (hanging[0] - flat[0]).abs() < 0.5,
        "내어쓰기는 첫 줄을 옮기지 않는다: {hanging:?}"
    );
    assert!(
        hanging[1] - flat[1] > 5.0,
        "내어쓰기는 둘째 줄부터 오른쪽으로 민다 — 없음 {flat:?}, 내어쓰기 {hanging:?}"
    );
}

#[test]
fn first_line_indent_moves_first_line_of_typed_paragraph() {
    let flat = typed_line_starts(0);
    let indented = typed_line_starts(3000);
    assert!(
        indented[0] - flat[0] > 5.0,
        "들여쓰기는 첫 줄을 오른쪽으로 민다 — 없음 {flat:?}, 들여쓰기 {indented:?}"
    );
    assert!(
        (indented[1] - flat[1]).abs() < 0.5,
        "들여쓰기는 둘째 줄을 옮기지 않는다: {indented:?}"
    );
}

#[test]
fn editing_keeps_stored_hanging_indent() {
    // biz_plan.hwp 문단 51: indent=-7284, 저장 줄 tag 0x60000 / 0x160000.
    const PARA: usize = 51;
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/biz_plan.hwp");
    let mut doc =
        HwpDocument::from_bytes(&std::fs::read(path).expect("read sample")).expect("open sample");
    let before = line_starts(&doc, PARA);
    assert!(
        before.len() >= 2 && before[1] - before[0] > 5.0,
        "원본 내어쓰기: {before:?}"
    );

    // 문단 끝(67자 뒤)에 한 글자를 넣는다.
    doc.insert_text_native(0, PARA, 67, "가")
        .expect("insert text");
    let after = line_starts(&doc, PARA);
    assert!(
        after.len() == before.len() && after.iter().zip(&before).all(|(a, b)| (a - b).abs() < 0.5),
        "글자를 넣어도 내어쓰기는 그대로다 — 편집 전 {before:?}, 편집 후 {after:?}"
    );
}

/// 구역 0 문단 `para` 의 줄마다 첫 글자 run 의 x.
fn line_starts(doc: &HwpDocument, para: usize) -> Vec<f64> {
    let mut starts = Vec::new();
    for page in 0..doc.page_count() {
        let tree = doc.build_page_render_tree(page).expect("render tree");
        collect_line_starts(&tree.root, para, &mut starts);
    }
    starts
}

fn collect_line_starts(node: &RenderNode, para: usize, out: &mut Vec<f64>) {
    if let RenderNodeType::TextLine(line) = &node.node_type {
        if line.section_index == Some(0) && line.para_index == Some(para) {
            if let Some(x) = first_run_x(node) {
                out.push(x);
            }
        }
        return;
    }
    for child in &node.children {
        collect_line_starts(child, para, out);
    }
}

fn first_run_x(node: &RenderNode) -> Option<f64> {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if run.cell_context.is_none() && !run.text.trim().is_empty() {
            return Some(node.bbox.x);
        }
    }
    node.children.iter().find_map(first_run_x)
}

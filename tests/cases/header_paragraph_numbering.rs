//! 머리말 문단의 번호·개요 머리를 그리고, 본문과 한 계수기로 센다.
//!
//! 시험 문서는 한 쪽이다. 머리말에 번호·글머리표·개요 문단이 있고 본문에도 같은 세 문단이
//! 있다. 번호와 개요는 같은 번호 정의(1, 수준 1)를 쓴다. 한컴 한글 12.30(Mac) 인쇄
//! PDF(`pdf/header_footer_numbering/`)는 머리말에 "1.", "2.", 본문에 "3.", "4."를 찍는다.
//! 글머리표는 한/글이 머리말과 본문 어디에도 그리지 않아 여기서 보지 않는다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "tests/fixtures/header_footer_numbering/header-numbering.hwp";

/// 머리말 안과 밖(본문)의 글자를 공백 없이 모은다.
fn collect_text(node: &RenderNode, in_header: bool, header: &mut String, body: &mut String) {
    let in_header = in_header || matches!(node.node_type, RenderNodeType::Header);
    if let RenderNodeType::TextRun(run) = &node.node_type {
        let text = run.display_or_text().chars().filter(|c| !c.is_whitespace());
        if in_header {
            header.extend(text);
        } else {
            body.extend(text);
        }
    }
    for child in &node.children {
        collect_text(child, in_header, header, body);
    }
}

#[test]
fn header_paragraph_numbering_shares_counter_with_body() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");
    assert_eq!(core.page_count(), 1);

    let tree = core.build_page_render_tree(0).expect("render tree");
    let (mut header, mut body) = (String::new(), String::new());
    collect_text(&tree.root, false, &mut header, &mut body);

    for expected in ["1.머리말번호문단", "2.머리말개요문단"] {
        assert!(
            header.contains(expected),
            "머리말에 {expected:?} 없음: {header}"
        );
    }
    for expected in ["3.본문번호문단", "4.본문개요문단"] {
        assert!(body.contains(expected), "본문에 {expected:?} 없음: {body}");
    }
}

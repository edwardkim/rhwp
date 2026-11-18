//! HWPX `hp:switch` 밖 평문 `<hh:lineSpacing>` 의 고정·여백 줄간격 단위는 패키지 xmlVersion 을 따른다.
//!
//! 한컴 웹한글기안기 인쇄 PDF(`pdf/issue8676/*-webhwp-print.pdf`)에서 같은 평문 값이 아래처럼 놓인다.
//!
//! - xmlVersion 1.4: 고정 2000 → 줄 피치 19.92pt, 여백 500 → 10pt 줄 + 5pt
//! - xmlVersion 1.2: 고정 2000 → 줄 피치 10.08pt, 여백 500 → 10pt 줄 + 2.5pt
//!
//! 1.4 의 평문 값은 평문 여백처럼 물리 HWPUNIT 이고, 1.2 는 HWP5 저장값과 같은 2× IR 이다.
//!
//! 수정 전 rhwp 는 판본과 상관없이 평문 값을 2× IR 로 읽어 1.4 문서의 고정·여백 줄간격을 절반만 띄웠다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::{
    renderer::render_tree::{RenderNode, RenderNodeType},
    DocumentCore,
};

fn line_top(node: &RenderNode, para: usize) -> Option<f64> {
    if let RenderNodeType::TextLine(line) = &node.node_type {
        if line.para_index == Some(para) && line.section_index == Some(0) {
            return Some(node.bbox.y);
        }
    }
    node.children.iter().find_map(|child| line_top(child, para))
}

/// 대상 줄간격 문단 AONE(문단 1) 윗변에서 같은 모양 문단 ATWO(문단 2) 윗변까지(px).
fn target_pitch(name: &str) -> f64 {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/issue8676")
        .join(name);
    let bytes = std::fs::read(&path).expect("줄간격 재현물");
    let core = DocumentCore::from_bytes(&bytes).unwrap();
    assert_eq!(core.page_count(), 1, "{name}: 웹한글기안기와 같은 쪽수");
    let tree = core.build_page_render_tree(0).unwrap();
    let one = line_top(&tree.root, 1).expect("문단 1 줄");
    let two = line_top(&tree.root, 2).expect("문단 2 줄");
    two - one
}

#[test]
fn plain_fixed_line_spacing_is_physical_from_xml_version_1_4() {
    let v14 = target_pitch("plain_fixed_v14.hwpx");
    let v12 = target_pitch("plain_fixed_v12.hwpx");
    // 한컴: 1.4 는 19.92pt, 1.2 는 10.08pt — 같은 평문 값이 1.4 에서 두 배로 띄운다.
    assert!(
        (v14 - 2.0 * v12).abs() < 1.0,
        "평문 고정 2000: 1.4 피치 {v14:.2}px 는 1.2 피치 {v12:.2}px 의 두 배여야 한다"
    );
    // 1.2 는 HWP5 저장값과 같은 IR 이라 10pt 문단의 한 줄(13.33px)과 같다.
    assert!(
        (v12 - 10.0 * 96.0 / 72.0).abs() < 0.5,
        "1.2 평문 고정 2000 은 10pt 피치: {v12:.2}px"
    );
}

#[test]
fn plain_between_lines_spacing_is_physical_from_xml_version_1_4() {
    let v14 = target_pitch("plain_between_v14.hwpx");
    let v12 = target_pitch("plain_between_v12.hwpx");
    let text_line = 10.0 * 96.0 / 72.0;
    // 한컴: 여백 500 은 1.4 에서 5pt, 1.2 에서 2.5pt 를 10pt 줄 아래에 더한다.
    assert!(
        ((v14 - text_line) - 2.0 * (v12 - text_line)).abs() < 1.0,
        "평문 여백 500: 1.4 여백 {:.2}px 는 1.2 여백 {:.2}px 의 두 배여야 한다",
        v14 - text_line,
        v12 - text_line
    );
}

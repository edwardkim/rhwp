//! Issue #776: H1' (column top sb) + H3b (zone 전환 ColumnDef.spacing/2) 정정
//!
//! RFC #774 산출 결과로 식별된 2개 independent paragraph spacing 결함:
//!
//! - H1': 단/페이지 첫 paragraph 의 ParaShape.spacing_before 누락
//!   (`paragraph_layout.rs:744-748` 의 `is_column_top` 가드)
//! - H3b: zone 전환 시 ColumnDef.spacing / 2 vertical 추가 spacing 미적용
//!   (`layout.rs:1240` 영역)
//!
//! 정합 기대 (PDF 한글 2022):
//! - shortcut.hwp 페이지 1 본문 baseline ~137.87 px (body_top=56.7 px 기준 +81 px)
//! - sungeo.hwp pi=0 heading 위치 ~12.6 px (body_top=132.3 px 기준)
//! - treatise sample.hwp pi=0 heading 위치 ~23.7 px (body_top=132.3 px 기준)
//!
//! 본 task 정정 후 모든 가드 통과 (GREEN).

use std::fs;
use std::path::Path;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

fn find_body_y(node: &RenderNode) -> Option<f64> {
    if matches!(node.node_type, RenderNodeType::Body { .. }) {
        return Some(node.bbox.y);
    }
    for child in &node.children {
        if let Some(y) = find_body_y(child) {
            return Some(y);
        }
    }
    None
}

/// Body 노드 하위에서만 pi 일치 TextLine 의 y 검색 (바탕쪽/머리말 제외).
fn find_body_first_textline_y(node: &RenderNode, target_pi: usize, in_body: bool) -> Option<f64> {
    let now_in_body = in_body || matches!(node.node_type, RenderNodeType::Body { .. });
    let in_header_footer = matches!(node.node_type, RenderNodeType::Header | RenderNodeType::Footer);
    if in_header_footer {
        return None;
    }
    if now_in_body {
        if let RenderNodeType::TextLine(tl) = &node.node_type {
            if tl.para_index == Some(target_pi) {
                return Some(node.bbox.y);
            }
        }
    }
    for child in &node.children {
        if let Some(y) = find_body_first_textline_y(child, target_pi, now_in_body) {
            return Some(y);
        }
    }
    None
}

fn build_page0_tree(sample: &str) -> rhwp::renderer::render_tree::PageRenderTree {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let bytes = fs::read(Path::new(repo_root).join(sample))
        .unwrap_or_else(|e| panic!("read {}: {}", sample, e));
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {}: {}", sample, e));
    doc.build_page_render_tree(0).expect("build_page_render_tree(0)")
}

/// H1' guard: shortcut.hwp 페이지 1 의 heading paragraph (pi=0) 가
/// body_top 으로부터 ParaShape.spacing_before 만큼 떨어져야 함 (PDF 26.83 px).
#[test]
fn issue_776_h1prime_shortcut_heading_offset() {
    let tree = build_page0_tree("samples/basic/shortcut.hwp");
    let body_y = find_body_y(&tree.root).expect("Body 노드 누락");
    let pi0_y = find_body_first_textline_y(&tree.root, 0, false)
        .expect("pi=0 (heading) 가 페이지 0 에 없음");
    let offset = pi0_y - body_y;

    eprintln!(
        "[issue_776/H1'] shortcut.hwp pi=0 body_y={:.2} pi0_y={:.2} offset={:+.2} (PDF expected ~26.83)",
        body_y, pi0_y, offset
    );

    // PDF 측정: heading top y=83.53, body_top=56.7, offset=26.83
    // 허용 오차 ±5 px (sub-pixel + 한컴 알고리즘 미세 차이)
    assert!(
        (offset - 26.83).abs() < 5.0,
        "H1' 가드: shortcut.hwp pi=0 heading offset {:.2} (기대 ~26.83, 허용 ±5)",
        offset,
    );
}

/// H3b 가드: shortcut.hwp 페이지 1 의 본문 (pi=2) 가 body_top 으로부터
/// PDF 정합 위치에 있어야 함.
#[test]
fn issue_776_h3b_shortcut_body_offset() {
    let tree = build_page0_tree("samples/basic/shortcut.hwp");
    let body_y = find_body_y(&tree.root).expect("Body 노드 누락");
    let pi2_y = find_body_first_textline_y(&tree.root, 2, false)
        .expect("pi=2 (본문 첫 줄) 가 페이지 0 에 없음");
    let offset = pi2_y - body_y;

    eprintln!(
        "[issue_776/H3b] shortcut.hwp pi=2 body_y={:.2} pi2_y={:.2} offset={:+.2} (PDF expected ~137.87)",
        body_y, pi2_y, offset
    );

    // PDF 측정: 본문 baseline 137.87 px (body_top 기준)
    // H1' (26.45) + H3b 누적 (37.8) = 64.25 px → 73.76 + 64.25 = 138.01 px ≈ PDF 137.87
    assert!(
        (offset - 137.87).abs() < 8.0,
        "H3b 가드: shortcut.hwp pi=2 body offset {:.2} (기대 ~137.87, 허용 ±8)",
        offset,
    );
}

/// H1' 가드: sungeo.hwp pi=0 heading offset (PDF 12.63 px).
#[test]
fn issue_776_h1prime_sungeo_heading_offset() {
    let tree = build_page0_tree("samples/basic/sungeo.hwp");
    let body_y = find_body_y(&tree.root).expect("Body 노드 누락");
    let pi0_y = find_body_first_textline_y(&tree.root, 0, false)
        .expect("pi=0 가 페이지 0 에 없음");
    let offset = pi0_y - body_y;

    eprintln!(
        "[issue_776/H1'] sungeo.hwp pi=0 body_y={:.2} pi0_y={:.2} offset={:+.2} (PDF expected ~12.63)",
        body_y, pi0_y, offset
    );

    assert!(
        (offset - 12.63).abs() < 5.0,
        "H1' 가드: sungeo.hwp pi=0 heading offset {:.2} (기대 ~12.63, 허용 ±5)",
        offset,
    );
}

/// H1' 가드: treatise sample.hwp pi=0 heading offset (PDF 23.69 px).
#[test]
fn issue_776_h1prime_treatise_heading_offset() {
    let tree = build_page0_tree("samples/basic/treatise sample.hwp");
    let body_y = find_body_y(&tree.root).expect("Body 노드 누락");
    let pi0_y = find_body_first_textline_y(&tree.root, 0, false)
        .expect("pi=0 가 페이지 0 에 없음");
    let offset = pi0_y - body_y;

    eprintln!(
        "[issue_776/H1'] treatise sample.hwp pi=0 body_y={:.2} pi0_y={:.2} offset={:+.2} (PDF expected ~23.69)",
        body_y, pi0_y, offset
    );

    assert!(
        (offset - 23.69).abs() < 5.0,
        "H1' 가드: treatise sample.hwp pi=0 heading offset {:.2} (기대 ~23.69, 허용 ±5)",
        offset,
    );
}

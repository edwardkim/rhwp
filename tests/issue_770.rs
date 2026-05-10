//! Issue #770: shortcut.hwp 페이지 2~7 헤더 TAC 1x1 표 후속 spacing 누락
//!
//! 페이지 2 헤더 ('파일') ~ 본문 ('새 문서') 거리가 PDF 권위(한글 2022) 대비
//! 약 40 px 압축. dump-pages 단 0 (헤더 zone) used 가 hwp_used 보다 13-33 px 부족.
//!
//! 정합 동작: 페이지 2 의 pi=37 ('새 문서') 가 본문 영역 상단으로부터 약 47 px
//! 아래 (= 헤더 표 1x1 + 후속 spacing 합계, hwp_used) 에 등장.
//!
//! 권위 자료: `pdf/basic/shortcut-2022.pdf` 페이지 2.

use std::fs;
use std::path::Path;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/basic/shortcut.hwp";
const TARGET_PI: usize = 37;
const TARGET_PAGE: u32 = 1; // 페이지 2 (0-based)
// pi=36 의 vpos_end = ls[1].vpos + lh + ls = 1200 + 2332 + 0 = 3532 HU = 47.1 px (hwp_used)
// 즉 본문 첫 paragraph (pi=37) 는 body 상단으로부터 약 47.1 px 아래 등장해야 정합.
// 0.5 px 허용 오차.
const EXPECTED_BODY_OFFSET_MIN: f64 = 40.0; // 보수적 경계 (현재 21 px → fail)

fn find_first_textline_y(node: &RenderNode, target_pi: usize) -> Option<f64> {
    if let RenderNodeType::TextLine(tl) = &node.node_type {
        if tl.para_index == Some(target_pi) {
            return Some(node.bbox.y);
        }
    }
    for child in &node.children {
        if let Some(y) = find_first_textline_y(child, target_pi) {
            return Some(y);
        }
    }
    None
}

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

#[test]
fn issue_770_page2_body_paragraph_below_header_zone() {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let hwp_path = Path::new(repo_root).join(SAMPLE);
    let bytes = fs::read(&hwp_path).unwrap_or_else(|e| panic!("read {}: {}", SAMPLE, e));
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {}: {}", SAMPLE, e));
    let tree = doc.build_page_render_tree(TARGET_PAGE).expect("build_page_render_tree");

    let body_y = find_body_y(&tree.root).expect("Body 노드 누락");
    let pi37_y = find_first_textline_y(&tree.root, TARGET_PI)
        .unwrap_or_else(|| panic!("pi={} 가 페이지 인덱스 {} 에 없음", TARGET_PI, TARGET_PAGE));

    let offset = pi37_y - body_y;
    eprintln!(
        "[issue_770] page_index={} body_y={:.2} pi={}_y={:.2} offset={:.2} (expected_min={})",
        TARGET_PAGE, body_y, TARGET_PI, pi37_y, offset, EXPECTED_BODY_OFFSET_MIN,
    );

    assert!(
        offset >= EXPECTED_BODY_OFFSET_MIN,
        "페이지 인덱스 {} 의 pi={} (본문 첫 paragraph) 가 body 상단으로부터 {:.2} px 위치. \
         hwp_used 정합 최소값 {} px 미달 — 헤더 zone 압축 결함.",
        TARGET_PAGE, TARGET_PI, offset, EXPECTED_BODY_OFFSET_MIN,
    );
}

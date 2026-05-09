//! Issue #716: hongbo page 1 마지막 줄 LAYOUT_OVERFLOW_DRAW (Task #332 잔존 영역)
//!
//! 결함: `samples/20250130-hongbo.hwp` 의 페이지 0 (1쪽) 마지막 텍스트 줄이
//! 컬럼(=Body 영역) 하단을 +20.1 px 초과하여 cropping 되는 결함.
//!
//! ```
//! LAYOUT_OVERFLOW_DRAW: section=0 pi=15 line=2 y=1048.2 col_bottom=1028.0 overflow=20.1px
//! LAYOUT_OVERFLOW: page=0, col=0, para=15, type=PartialParagraph, y=1059.4, bottom=1028.0, overflow=31.3px
//! ```
//!
//! 본질: 음수 line_spacing(ls<0) 을 layout y_offset advance 가 무시 → drift 누적.
//! TAC 표 호스트 (pi=0 lh=3560 ls=-600, pi=2 lh=3148 ls=-900) 와 빈 문단
//! (pi=1 lh=1500 ls=-600, pi=3 lh=1500 ls=-900) 마다 drift 가 +|ls_px| 누적.
//! VPOS_CORR 의 4중 가드(layout.rs:1402/1488/1484/1536) 가 forward drift 회수
//! 차단 → 페이지 끝까지 누적.
//!
//! 정상 동작: page 0 의 모든 TextLine 의 bbox 하단이 Body 영역 하단 이내.

use std::fs;
use std::path::Path;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/20250130-hongbo.hwp";

/// RenderTree 내 Body 노드의 bbox 를 반환 (페이지의 컬럼 영역 = 본문 사용 가능 영역).
fn find_body_bbox(node: &RenderNode) -> Option<(f64, f64, f64, f64)> {
    if matches!(node.node_type, RenderNodeType::Body { .. }) {
        let b = &node.bbox;
        return Some((b.x, b.y, b.width, b.height));
    }
    for child in &node.children {
        if let Some(found) = find_body_bbox(child) {
            return Some(found);
        }
    }
    None
}

/// RenderTree 의 모든 TextLine 노드 bbox 의 (y, y+h) 수집.
/// 머리말/꼬리말 (Header/Footer 자식) 영역은 제외 — 본문 컬럼 결함만 검증.
fn collect_body_text_line_bboxes(node: &RenderNode, out: &mut Vec<(f64, f64)>, in_body: bool) {
    let now_in_body = in_body || matches!(node.node_type, RenderNodeType::Body { .. });
    let in_header_footer = matches!(node.node_type, RenderNodeType::Header | RenderNodeType::Footer);
    if in_header_footer {
        return; // 머리말/꼬리말 제외
    }
    if now_in_body {
        if let RenderNodeType::TextLine(_) = &node.node_type {
            out.push((node.bbox.y, node.bbox.y + node.bbox.height));
        }
    }
    for child in &node.children {
        collect_body_text_line_bboxes(child, out, now_in_body);
    }
}

#[test]
fn issue_716_page1_last_text_line_within_body() {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let hwp_path = Path::new(repo_root).join(SAMPLE);
    let bytes = fs::read(&hwp_path).unwrap_or_else(|e| panic!("read {}: {}", SAMPLE, e));
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {}: {}", SAMPLE, e));

    let tree = doc.build_page_render_tree(0).expect("build_page_render_tree(0)");

    let (body_x, body_y, body_w, body_h) = find_body_bbox(&tree.root)
        .expect("Body 노드 bbox 누락");
    let body_bottom = body_y + body_h;

    let mut bboxes = Vec::new();
    collect_body_text_line_bboxes(&tree.root, &mut bboxes, false);
    assert!(!bboxes.is_empty(), "page 0 본문 영역 TextLine 0건");

    let max_bottom = bboxes.iter().map(|(_, b)| *b).fold(f64::MIN, f64::max);
    let overflow = max_bottom - body_bottom;

    eprintln!(
        "[issue_716] page 0 body=[x={:.2} y={:.2} w={:.2} h={:.2} bottom={:.2}] \
         text_lines={} max_bottom={:.2} overflow={:+.2}",
        body_x, body_y, body_w, body_h, body_bottom,
        bboxes.len(), max_bottom, overflow,
    );

    // 0.5 px 허용 오차 (sub-pixel rounding).
    assert!(
        max_bottom <= body_bottom + 0.5,
        "page 0 본문 텍스트 줄이 Body 하단 초과: max_bottom={:.2}, body_bottom={:.2}, overflow={:+.2} px",
        max_bottom, body_bottom, overflow,
    );
}

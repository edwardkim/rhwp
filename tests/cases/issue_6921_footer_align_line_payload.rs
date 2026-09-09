//! [Issue #6921] 꼬리말 세로 정렬이 옮긴 문단 테두리 이중선이 **옮기기 전 자리에**
//! 그려져 본문 마지막 글줄을 가로지른다 (148733091 e-브리핑 속기자료).
//!
//! 근인: `LayoutEngine::translate_subtree_y` 가 `node.bbox.y` 만 옮겼다. 백엔드는
//! 노드마다 다른 것을 읽는다 — `Rectangle`·`Ellipse`·`Image`·글자는 `node.bbox` 로
//! 그리지만 `Line` 은 `x1/y1–x2/y2` 를, `Path` 는 `commands` 의 절대 좌표를 경로로
//! 삼는다(`svg.rs::draw_line` · `draw_path_with_gradient`). 그래서 `bbox` 만 옮은
//! 선은 **bbox 와 방출 좌표가 따로 놀았다.**
//!
//! ```text
//!   dy = 23.88px (꼬리말 vertAlign 정렬)
//!     bbox      1007.92 → 1031.80
//!     SVG 방출  1007.92            ← 안 옮겨짐. 본문 바닥 1009.2 안이라 글줄을 가로지른다
//! ```
//!
//! 정본 `pdf/148733091-briefing-stenographic-record-2020.pdf`(engine 2020) 12쪽은
//! 같은 선을 `1028.2 / 1029.9` 에 그린다. 수정 후 rhwp 는 `1032.4 / 1034.5` 로
//! 본문 밖에 놓인다.
//!
//! ⚠ 정본과의 남은 `+4.2px` 는 꼬리말 문단 자체의 세로 위치라 다른 축이다 — 이 시험은
//! **payload 좌표가 bbox 와 같은가**와 **본문 바닥 아래인가**만 잠근다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::wasm_api::HwpDocument;

const SAMPLE: &str = "samples/issue6921/148733091-briefing-stenographic-record.hwp";
/// 꼬리말 구분선이 있는 쪽 (0-based).
const PAGE: u32 = 11;

#[test]
fn issue_6921_footer_line_payload_follows_its_bbox() {
    let root = tree();
    let lines = footer_lines(&root);
    assert!(
        lines.len() >= 2,
        "꼬리말 구분선 이중선을 찾지 못했다 (찾은 선 {}개)",
        lines.len()
    );
    for (bbox_y, line) in &lines {
        // 선은 경로를 중심 정렬로 칠하므로 bbox 는 획의 절반만큼 위로 번진다.
        let payload_from_bbox = bbox_y + line_half_stroke(line);
        assert!(
            (line.y1 - payload_from_bbox).abs() <= 0.5,
            "방출 좌표 y1={:.2} 가 bbox {:.2} 와 어긋난다 (획 {:.2})",
            line.y1,
            bbox_y,
            line.style.width,
        );
        assert!(
            (line.y1 - line.y2).abs() <= 0.01,
            "가로 구분선인데 y1={:.2} y2={:.2} 로 기울었다",
            line.y1,
            line.y2,
        );
    }
}

/// 구분선은 **본문 바닥 아래**에 있어야 한다 — 그 위면 본문 글줄을 가로지른다.
#[test]
fn issue_6921_footer_line_stays_below_body() {
    let root = tree();
    let body_bottom = body_bottom(&root).expect("본문 영역");
    for (_, line) in footer_lines(&root) {
        assert!(
            line.y1 >= body_bottom,
            "구분선이 y={:.2} 로 본문 바닥 {body_bottom:.2} 위에 그려진다",
            line.y1,
        );
    }
}

fn tree() -> RenderNode {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path)
        .unwrap_or_else(|error| panic!("fixture 를 읽을 수 없다 ({}): {error}", path.display()));
    HwpDocument::from_bytes(&bytes)
        .expect("문서 로드")
        .build_page_render_tree(PAGE)
        .unwrap_or_else(|error| panic!("쪽 idx {PAGE} render tree: {error:?}"))
        .root
}

fn line_half_stroke(line: &rhwp::renderer::render_tree::LineNode) -> f64 {
    line.style.width.max(0.0) / 2.0
}

fn body_bottom(node: &RenderNode) -> Option<f64> {
    if matches!(node.node_type, RenderNodeType::Body { .. }) {
        return Some(node.bbox.y + node.bbox.height);
    }
    node.children.iter().find_map(body_bottom)
}

/// 꼬리말 안 가로 구분선의 `(bbox.y, LineNode)`.
fn footer_lines(root: &RenderNode) -> Vec<(f64, &rhwp::renderer::render_tree::LineNode)> {
    let mut out = Vec::new();
    collect(root, false, &mut out);
    out
}

fn collect<'a>(
    node: &'a RenderNode,
    in_footer: bool,
    out: &mut Vec<(f64, &'a rhwp::renderer::render_tree::LineNode)>,
) {
    let in_footer = in_footer || matches!(node.node_type, RenderNodeType::Footer);
    if in_footer {
        if let RenderNodeType::Line(line) = &node.node_type {
            if node.bbox.width > 100.0 {
                out.push((node.bbox.y, line));
            }
        }
    }
    for child in &node.children {
        collect(child, in_footer, out);
    }
}

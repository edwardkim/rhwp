//! [Issue #6269] 본문 좌단에 붙은 세로 테두리선의 획 절반이 body clip 에 잘려 거의
//! 안 보인다 (156739836 2·3쪽).
//!
//! 근인은 clip 이 아니라 **선의 bbox 규약**이었다. 백엔드(SVG·Canvas·Skia)는
//! `line.x1/y1`–`x2/y2` 를 경로로 삼아 획을 **중심 정렬**로 칠하므로 잉크는
//! `[경로-획/2, 경로+획/2]` 다. 그런데 테두리선의 bbox 는 `[경로, 경로+획]` 로 잡혀
//! 있어 잉크보다 반 획 밀려 있었다. body clip 은 자식 bbox 로 넓어지므로, 경계에
//! 붙은 선은 그 밀린 만큼 바깥 절반이 잘렸다.
//!
//! ```text
//! body clip      x = 75.5867            (본문 좌단)
//! 왼쪽 세로선    x = 75.5867  획 1.5    → 잉크 74.84..76.34, 왼쪽 0.75px 가 clip 밖
//! 오른쪽 세로선  x = 703.03   획 1.5    → clip 안쪽이라 온전
//! ```
//!
//! 헤드리스 Chrome 잉크 실측(scale 2): 왼쪽 **102** vs 오른쪽 204 — 같은 굵기인데
//! 정확히 절반이었다.
//!
//! `LineNode::ink_bbox()` 로 bbox 를 잉크 범위로 통일했으므로 잠금은 두 겹이다.
//! (1) 선의 bbox 는 잉크 범위다. (2) body clip 은 자기 자식 선의 bbox 를 자르지 않는다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{BoundingBox, LineNode, RenderNode, RenderNodeType};
use rhwp::renderer::{LineStyle, StrokeDash};

const SAMPLE: &str = "samples/issue6269/156739836_public_sector_jobs_stats.hwpx";
/// 결함이 나타나는 쪽(0-based). 3쪽도 같은 틀을 쓴다.
const PAGE: u32 = 1;

fn stroke(width: f64) -> LineStyle {
    LineStyle {
        width,
        dash: StrokeDash::Solid,
        ..Default::default()
    }
}

/// (1) 규약 자체 — 잉크는 경로에서 획 절반만큼 양쪽으로 번진다.
#[test]
fn issue_6269_line_ink_bbox_straddles_the_path() {
    // 가로선: 경로 y=100, 획 1.5 → 잉크 99.25..100.75. 획 방향(가로)은 butt cap 이라
    // 경로 끝에서 더 번지지 않는다.
    let horizontal = LineNode::new(10.0, 100.0, 60.0, 100.0, stroke(1.5));
    let bb = horizontal.ink_bbox();
    assert!((bb.x - 10.0).abs() <= 1e-9, "가로선 좌단: {bb:?}");
    assert!((bb.width - 50.0).abs() <= 1e-9, "가로선 길이: {bb:?}");
    assert!((bb.y - 99.25).abs() <= 1e-9, "가로선 잉크 위쪽: {bb:?}");
    assert!((bb.height - 1.5).abs() <= 1e-9, "가로선 잉크 두께: {bb:?}");

    // 세로선: 경로 x=75.5867, 획 1.5 → 잉크 74.8367..76.3367 (#6269 그 선).
    let vertical = LineNode::new(75.5867, 200.0, 75.5867, 400.0, stroke(1.5));
    let bb = vertical.ink_bbox();
    assert!((bb.x - 74.8367).abs() <= 1e-9, "세로선 잉크 왼쪽: {bb:?}");
    assert!((bb.width - 1.5).abs() <= 1e-9, "세로선 잉크 두께: {bb:?}");
    assert!((bb.y - 200.0).abs() <= 1e-9, "세로선 상단: {bb:?}");
    assert!((bb.height - 200.0).abs() <= 1e-9, "세로선 길이: {bb:?}");

    // 대각선은 두 축 모두 번진다.
    let diagonal = LineNode::new(0.0, 0.0, 10.0, 10.0, stroke(2.0));
    let bb = diagonal.ink_bbox();
    assert!((bb.x + 1.0).abs() <= 1e-9, "대각선 왼쪽: {bb:?}");
    assert!((bb.y + 1.0).abs() <= 1e-9, "대각선 위쪽: {bb:?}");
    assert!((bb.width - 12.0).abs() <= 1e-9, "대각선 폭: {bb:?}");
    assert!((bb.height - 12.0).abs() <= 1e-9, "대각선 높이: {bb:?}");
}

/// (2) 실문서 불변식 — body clip 은 자기 자식 선의 잉크를 자르지 않는다.
#[test]
fn issue_6269_body_clip_contains_line_ink() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");
    let page = core
        .build_page_render_tree(PAGE)
        .expect("page 2 render tree");

    let (clip, body) = find_body_clip(&page.root).expect("body clip_rect");
    let mut lines = Vec::new();
    collect_lines(body, &mut lines);
    assert!(!lines.is_empty(), "2쪽에 테두리 선이 있어야 한다");

    let clip_left = clip.x;
    let clip_top = clip.y;
    let clip_right = clip.x + clip.width;
    let clip_bottom = clip.y + clip.height;

    // bbox 가 곧 잉크 범위다 — 더 부풀리지 않는다.
    for bb in lines {
        assert!(
            bb.x >= clip_left - 0.01,
            "선의 잉크 왼쪽({:.2})이 body clip({clip_left:.2}) 밖이다 — 획이 잘린다",
            bb.x
        );
        assert!(
            bb.y >= clip_top - 0.01,
            "선의 잉크 위쪽({:.2})이 body clip({clip_top:.2}) 밖이다",
            bb.y
        );
        assert!(
            bb.x + bb.width <= clip_right + 0.01,
            "선의 잉크 오른쪽({:.2})이 body clip({clip_right:.2}) 밖이다",
            bb.x + bb.width
        );
        assert!(
            bb.y + bb.height <= clip_bottom + 0.01,
            "선의 잉크 아래쪽({:.2})이 body clip({clip_bottom:.2}) 밖이다",
            bb.y + bb.height
        );
    }
}

/// `Body` 의 clip_rect 과 그 노드를 찾는다.
fn find_body_clip(node: &RenderNode) -> Option<(BoundingBox, &RenderNode)> {
    if let RenderNodeType::Body {
        clip_rect: Some(cr),
    } = &node.node_type
    {
        return Some((*cr, node));
    }
    node.children.iter().find_map(find_body_clip)
}

fn collect_lines(node: &RenderNode, out: &mut Vec<BoundingBox>) {
    if matches!(node.node_type, RenderNodeType::Line(_)) {
        out.push(node.bbox);
    }
    for child in &node.children {
        collect_lines(child, out);
    }
}

//! [Issue #6269] 본문 좌단에 붙은 세로 테두리선의 획 절반이 body clip 에 잘려 거의
//! 안 보인다 (156739836 2·3쪽).
//!
//! 선의 **잉크는 bbox 밖으로 나간다.** bbox 는 `[경로, 경로+획]` 인데 백엔드는
//! `line.x1/y1` 을 경로로 삼아 획을 **중심 정렬**로 칠하므로 실제 잉크는
//! `[경로-획/2, 경로+획/2]` 다. body clip 은 자식 **bbox** 로만 넓어지므로, 경계에
//! 붙은 선은 획의 바깥 절반이 잘린다.
//!
//! ```text
//! body clip      x = 75.5867            (본문 좌단)
//! 왼쪽 세로선    x = 75.5867  획 1.5    → 잉크 74.84..76.34, 왼쪽 0.75px 가 clip 밖
//! 오른쪽 세로선  x = 703.03   획 1.5    → clip 안쪽이라 온전
//! ```
//!
//! 헤드리스 Chrome 잉크 실측(scale 2): 왼쪽 **102** vs 오른쪽 204 — 같은 굵기인데
//! 정확히 절반이다. 수정 후 왼쪽 **188** 로, 이슈가 실측한 rhwp PDF 값(94×2=188)과
//! 같아진다.
//!
//! 잠금은 픽셀이 아니라 **불변식**을 건다 — body clip 은 자기 자식 선의 잉크를
//! 자르지 않는다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue6269/156739836_public_sector_jobs_stats.hwpx";
/// 결함이 나타나는 쪽(0-based). 3쪽도 같은 틀을 쓴다.
const PAGE: u32 = 1;

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

    for bb in lines {
        // 잉크 범위 = bbox 를 획 절반만큼 부풀린 것 (백엔드가 경로 중심으로 칠한다).
        let half = bb.width.min(bb.height) / 2.0;
        assert!(
            bb.x - half >= clip_left - 0.01,
            "선의 잉크 왼쪽({:.2})이 body clip({clip_left:.2}) 밖이다 — 획이 잘린다",
            bb.x - half
        );
        assert!(
            bb.y - half >= clip_top - 0.01,
            "선의 잉크 위쪽({:.2})이 body clip({clip_top:.2}) 밖이다",
            bb.y - half
        );
        assert!(
            bb.x + bb.width + half <= clip_right + 0.01,
            "선의 잉크 오른쪽({:.2})이 body clip({clip_right:.2}) 밖이다",
            bb.x + bb.width + half
        );
        assert!(
            bb.y + bb.height + half <= clip_bottom + 0.01,
            "선의 잉크 아래쪽({:.2})이 body clip({clip_bottom:.2}) 밖이다",
            bb.y + bb.height + half
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

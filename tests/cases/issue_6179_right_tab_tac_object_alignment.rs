//! [Issue #6179] 오른쪽 탭 뒤 **자리차지(TAC) 그림**이 자기 폭만큼 오른쪽으로 밀려
//! 용지 밖으로 잘린다.
//!
//! 꼬리말 문단이 `[로고] \t [로고]` 로 저장돼 있고 문단 탭 정의는 `auto_tab_right`
//! 다. 오른쪽 탭은 "탭 뒤 블록의 **오른쪽 변**을 우단에 맞춘다"는 뜻인데, 되밀기 폭을
//! 재는 `text_measurement` 는 탭 뒤 **글자**만 본다. run 이 TAC 개체 위치에서 조각으로
//! 쪼개져 측정되므로 탭 뒤 조각에는 남는 글자가 없고 → 되밀기 폭 0 → 그림의 **왼쪽**
//! 변이 우단에 놓인다. 편차는 정확히 그림 폭(147.1px)이다.
//!
//! | | rhwp(수정 전) | rhwp(수정 후) | 한글 2024 |
//! |---|---|---|---|
//! | 그림 왼쪽 x | 717.6 | 570.6 | 570.73 |
//! | 그림 오른쪽 x | 864.7 (용지 793.7 밖) | 717.7 | 717.69 |
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue6179/right_tab_footer_logo.hwpx";
/// 본문 우단(px) — 오른쪽 탭이 겨누는 위치.
const BODY_RIGHT_PX: f64 = 718.1;
/// 용지 폭(px). 그림 오른쪽 변이 이 밖으로 나가면 잘린다.
const PAGE_WIDTH_PX: f64 = 793.7;

#[test]
fn issue_6179_right_tab_aligns_object_right_edge() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");
    let page = core.build_page_render_tree(0).expect("page 1 render tree");

    let mut images = Vec::new();
    collect_footer_images(&page.root, false, &mut images);
    assert_eq!(
        images.len(),
        2,
        "꼬리말에 로고 그림 두 개가 있어야 한다 — 실측 {images:?}"
    );

    // 오른쪽 로고: 오른쪽 변이 우단에 맞아야 한다.
    let (right_x, right_w) = images
        .iter()
        .cloned()
        .fold((f64::NEG_INFINITY, 0.0), |best, cur| {
            if cur.0 > best.0 {
                cur
            } else {
                best
            }
        });
    let right_edge = right_x + right_w;
    assert!(
        right_edge <= PAGE_WIDTH_PX,
        "오른쪽 탭 뒤 그림이 용지({PAGE_WIDTH_PX}) 밖으로 나갔다 — \
         x={right_x:.1} w={right_w:.1} 오른쪽 변={right_edge:.1}"
    );
    assert!(
        (right_edge - BODY_RIGHT_PX).abs() <= 2.0,
        "오른쪽 탭은 그림의 오른쪽 변을 우단({BODY_RIGHT_PX})에 맞춘다 — \
         실측 오른쪽 변 {right_edge:.1} (x={right_x:.1} w={right_w:.1}). \
         왼쪽 변이 우단에 놓이면 그림 폭만큼 밀려 잘린다."
    );
}

/// 꼬리말 영역 안 이미지의 `(x, width)`.
fn collect_footer_images(node: &RenderNode, in_footer: bool, out: &mut Vec<(f64, f64)>) {
    let in_footer = in_footer || matches!(node.node_type, RenderNodeType::Footer);
    if in_footer {
        if let RenderNodeType::Image(_) = &node.node_type {
            out.push((node.bbox.x, node.bbox.width));
        }
    }
    for child in &node.children {
        collect_footer_images(child, in_footer, out);
    }
}

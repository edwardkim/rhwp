//! [Issue #6185] 세로 오프셋이 **자기 높이의 정확한 음수**인 자리차지 로고 글상자를
//! 그 값대로 49.8px 올려 그려, 바로 위 `담당 부서` 표의 둘째 행을 통째로 덮는다
//! (156570535 2쪽).
//!
//! ```
//! 문단 pi=23  [사각형] 크기 90.7mm × 13.2mm (25698×3736 HU)
//!   위치: 세로=문단 오프셋 4294963560  → signed -3736 = -(높이 3736)
//!   배치: 자리차지, 글자처럼=false
//! ```
//!
//! 오프셋이 자기 높이의 정확한 음수인 것은 **배치 의도가 아니라 자기 변위 잔재**다 —
//! 한글은 무시하고 문단 자리에 그린다(글상자 상단 514.7 로 표 하단 499.5 아래).
//! rhwp 는 그대로 적용해 465.5 로 올렸다. #6110 의 `vpos == 그림 높이` 자기-변위
//! 지문과 같은 축이다.
//!
//! 재현물은 원본(5.1MB)의 문단 22..23 창을 잘라낸 IR 슬라이스(64KB)이고, 이미지는
//! 1×1 PNG 로 갈아 끼웠다 — 위치·크기는 컨트롤 속성이 결정하므로 화소는 무관하다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue6185/156570535_logo_box_self_displacement.hwpx";

#[test]
fn issue_6185_self_displacement_offset_does_not_lift_the_logo_box() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");

    let page = core.build_page_render_tree(0).expect("page 1 render tree");
    let table_bottom = table_bottom(&page.root).expect("담당 부서 표");
    let logo_top = images(&page.root)
        .into_iter()
        .map(|(top, _)| top)
        .fold(f64::INFINITY, f64::min);
    assert!(logo_top.is_finite(), "로고 이미지를 찾지 못했다");

    assert!(
        logo_top >= table_bottom,
        "자기 변위 오프셋(-높이)은 배치에 쓰지 않는다 — 표 아래끝 {table_bottom:.1}, \
         로고 위끝 {logo_top:.1} (덮으면 표 둘째 행이 가려진다)"
    );
}

/// 쪽에서 가장 아래로 내려온 표의 아래끝.
fn table_bottom(node: &RenderNode) -> Option<f64> {
    let own =
        matches!(node.node_type, RenderNodeType::Table(_)).then(|| node.bbox.y + node.bbox.height);
    node.children
        .iter()
        .filter_map(table_bottom)
        .chain(own)
        .fold(None, |acc: Option<f64>, bottom| {
            Some(acc.map_or(bottom, |best: f64| best.max(bottom)))
        })
}

/// 페이지의 모든 이미지 노드 (위끝, 아래끝).
fn images(node: &RenderNode) -> Vec<(f64, f64)> {
    let mut out = Vec::new();
    if matches!(node.node_type, RenderNodeType::Image(_)) {
        out.push((node.bbox.y, node.bbox.y + node.bbox.height));
    }
    for child in &node.children {
        out.extend(images(child));
    }
    out
}

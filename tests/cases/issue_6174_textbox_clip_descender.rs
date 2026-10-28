//! [Issue #6174] 글상자 clip 하단이 줄 baseline 보다 위라 로고 옆 `경 찰 청` 받침이
//! 4pt 잘린다 (156661338·156601658 1쪽).
//!
//! `TextBox` 노드의 bbox 를 두 백엔드가 그대로 clip 사각형으로 쓴다
//! (SVG `textbox-clip-*`, paint `ClipKind::TextBox`). 그 값은 상자 − `textMargin` 이라
//! 줄 높이가 안쪽 영역보다 크면 글자가 잘린다.
//!
//! ```xml
//! <hp:rect …>  <hc:pt0 x="0" y="0"/> … <hc:pt2 x="6073" y="1691"/>
//!   <hp:drawText …><hp:subList vertAlign="CENTER">
//!       <hp:t>경<hp:fwSpace/>찰<hp:fwSpace/>청</hp:t>
//!       <hp:lineseg vertsize="1200" …/>            <!-- 줄 16px -->
//!   <hp:textMargin left="283" right="283" top="283" bottom="283"/>   <!-- 3.77px -->
//! ```
//!
//! clip 11.41px(= 18.96 − 2×3.77) 안에 16px 줄이 놓여 baseline(130.69)이 clip
//! 하단(128.51) 밖으로 2.19px 나갔다. 한글은 이 글상자에서 세로로 자르지 않는다 —
//! 잉크가 상자 하단(132.28)보다 아래인 132.81 까지 그려진다.
//!
//! 잠금은 좌표 상수 대신 **불변식**을 건다 — 글상자 clip 은 자기 글줄을 세로로 자르지
//! 않는다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue6174/156661338_police_press_release.hwpx";
const PAGE: u32 = 0;
/// 잘리던 글상자의 글자.
const NEEDLE: &str = "경";

#[test]
fn issue_6174_textbox_clip_contains_its_own_line() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");

    let page = core
        .build_page_render_tree(PAGE)
        .expect("page 1 render tree");
    let mut hits = Vec::new();
    collect_textbox_with_needle(&page.root, &mut hits);
    assert!(
        !hits.is_empty(),
        "`{NEEDLE}` 를 담은 글상자를 1쪽에서 찾지 못했다"
    );

    for (clip_top, clip_bottom, run_top, run_bottom) in hits {
        assert!(
            run_bottom <= clip_bottom + 0.01,
            "글상자 clip 하단({clip_bottom:.2})이 글줄 아래끝({run_bottom:.2})을 자른다"
        );
        assert!(
            run_top >= clip_top - 0.01,
            "글상자 clip 상단({clip_top:.2})이 글줄 위끝({run_top:.2})을 자른다"
        );
    }
}

/// `NEEDLE` 을 담은 `TextBox` 마다 (clip 위끝, clip 아래끝, 글줄 위끝, 글줄 아래끝).
fn collect_textbox_with_needle(node: &RenderNode, out: &mut Vec<(f64, f64, f64, f64)>) {
    if matches!(node.node_type, RenderNodeType::TextBox) {
        if let Some((run_top, run_bottom)) = run_extent(node) {
            out.push((
                node.bbox.y,
                node.bbox.y + node.bbox.height,
                run_top,
                run_bottom,
            ));
        }
    }
    for child in &node.children {
        collect_textbox_with_needle(child, out);
    }
}

/// 하위에서 `NEEDLE` 을 포함한 텍스트 run 들의 (최소 위끝, 최대 아래끝).
fn run_extent(node: &RenderNode) -> Option<(f64, f64)> {
    let mut found: Option<(f64, f64)> = None;
    fold_runs(node, &mut found);
    found
}

fn fold_runs(node: &RenderNode, acc: &mut Option<(f64, f64)>) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if run.text.contains(NEEDLE) {
            let top = node.bbox.y;
            let bottom = node.bbox.y + node.bbox.height;
            *acc = Some(match *acc {
                Some((t, b)) => (t.min(top), b.max(bottom)),
                None => (top, bottom),
            });
        }
    }
    for child in &node.children {
        fold_runs(child, acc);
    }
}

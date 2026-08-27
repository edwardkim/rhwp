//! [Issue #6186] 꼬리말 `vertAlign=BOTTOM` 이 안 걸려 쪽번호가 밴드 위쪽에 붙는다
//! (156755659). 이 문서는 같은 자리에 겹쳐 놓은 글상자로도 `2 - 2` 를 그리는데,
//! 꼬리말만 21.8px 위에 놓여 **두 줄로 갈라져** 보인다.
//!
//! 근인: 파서는 HWPX `<hp:subList vertAlign="BOTTOM">` 을 이미 `list_attr` 비트
//! 21~22 로 싣고 모델까지 온전히 전달하는데, **레이아웃이 그 값을 읽지 않아** 늘
//! 밴드 맨 위에 놓았다.
//!
//! 정렬 기준은 **문서가 선언한 밴드 높이**(`<hp:subList textHeight="2834">`
//! = 37.79px)다. 공유 `layout.footer_area` 는 아래 여백까지 품고 있고(56.7px) 그
//! rect 는 쪽 계산에도 쓰여 건드리면 쪽수가 흔들린다(issue_1733 등 8핀 실측).
//!
//! 한글 2020 실측: 꼬리말 줄 y=1049.86, 같은 자리 글상자 y=1045.78.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue6186/156755659_footer_vertalign_bottom.hwpx";
/// 한글이 놓는 꼬리말 줄 위끝(px).
const EXPECTED_TOP_PX: f64 = 1049.9;

#[test]
fn issue_6186_footer_bottom_alignment_matches_hancom() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");

    let page = core.build_page_render_tree(1).expect("page 2 render tree");
    let footer = footer_node(&page.root).expect("꼬리말 노드");
    let line_top = first_line_top(footer).expect("꼬리말 줄");

    assert!(
        (line_top - EXPECTED_TOP_PX).abs() <= 2.0,
        "꼬리말은 밴드 아래쪽 정렬이어야 한다 — 줄 위끝 {line_top:.1} \
         (한글 {EXPECTED_TOP_PX:.1}, 밴드 {:.1}..{:.1})",
        footer.bbox.y,
        footer.bbox.y + footer.bbox.height
    );
    assert!(
        line_top > footer.bbox.y + 10.0,
        "밴드 맨 위에 붙으면 안 된다 — 밴드 위끝 {:.1}, 줄 위끝 {line_top:.1}",
        footer.bbox.y
    );
}

/// 저장 왕복에서도 세로 정렬이 보존되어야 한다 — 직렬화기가 `vertAlign` 을 늘
/// `"TOP"` 으로 굳혀 저장하면 재렌더가 밴드 맨 위로 돌아간다(visual roundtrip
/// baseline 이 exam_social·k-water-rfp 에서 25~32px 변위로 잡아냈다).
#[test]
fn issue_6186_footer_vertalign_survives_hwpx_roundtrip() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");
    let saved = rhwp::serializer::hwpx::serialize_hwpx(core.document()).expect("serialize");
    let again = DocumentCore::from_bytes(&saved).expect("reopen");

    let page = again.build_page_render_tree(1).expect("page 2 render tree");
    let footer = footer_node(&page.root).expect("꼬리말 노드");
    let line_top = first_line_top(footer).expect("꼬리말 줄");
    assert!(
        (line_top - EXPECTED_TOP_PX).abs() <= 2.0,
        "왕복 뒤에도 아래쪽 정렬이어야 한다 — 줄 위끝 {line_top:.1} (한글 {EXPECTED_TOP_PX:.1})"
    );
}

fn footer_node(node: &RenderNode) -> Option<&RenderNode> {
    if matches!(node.node_type, RenderNodeType::Footer) {
        return Some(node);
    }
    node.children.iter().find_map(footer_node)
}

fn first_line_top(node: &RenderNode) -> Option<f64> {
    let own = matches!(node.node_type, RenderNodeType::TextLine(_)).then_some(node.bbox.y);
    node.children
        .iter()
        .filter_map(first_line_top)
        .chain(own)
        .fold(None, |acc: Option<f64>, top| {
            Some(acc.map_or(top, |best: f64| best.min(top)))
        })
}

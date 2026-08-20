//! [Issue #5721] 글상자 안 자리차지 표 두 개의 순서가 뒤집힌다 — page-기준 vpos
//! 재기저화(origin 빼기)가 box-기준 스트림에 오발동.
//!
//! 2568129 글상자(용지 앵커, vertical_offset≈origin) 내부 문단들의 저장 vpos 는
//! **글상자 좌표**다. 종전 재기저화는 "vpos ≥ origin 인 줄만" origin 을 빼는 줄
//! 단위 휴리스틱이라, origin 을 우연히 넘는 뒤쪽 문단(제목 표, vpos 228.6px)만
//! 재기저화되어 29.4px 로 꺾였다 — 앞의 발신처 표(110.3px) **위**로 올라가고
//! 나머지 내용이 통째로 밀렸다. 한글 2022: 발신처 표가 위, 제목 표가 아래,
//! 간격 = 저장 vpos 차(118.3px).
//!
//! 수정: 스트림 첫 유효 vpos 가 origin 이상일 때만(=전체가 page 좌표일 때만)
//! origin 을 유지한다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue5721/2568129_textbox_float_tables.hwp";

fn walk<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    out.push(node);
    for child in &node.children {
        walk(child, out);
    }
}

#[test]
fn issue_5721_textbox_float_tables_keep_stored_order() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");
    let page = core.build_page_render_tree(0).expect("page 1");
    let mut nodes = Vec::new();
    walk(&page.root, &mut nodes);

    // 발신처 표(107.9px 높이)와 제목 표(36.2px 높이).
    let sender = nodes
        .iter()
        .find(|n| {
            matches!(&n.node_type, RenderNodeType::Table(_)) && (n.bbox.height - 107.9).abs() < 2.0
        })
        .expect("발신처 표");
    let title = nodes
        .iter()
        .find(|n| {
            matches!(&n.node_type, RenderNodeType::Table(_)) && (n.bbox.height - 36.2).abs() < 2.0
        })
        .expect("제목 표");

    assert!(
        sender.bbox.y < title.bbox.y,
        "발신처 표(저장 vpos 110.3px)가 제목 표(228.6px)보다 위여야 한다: 발신 {:.1} vs 제목 {:.1}",
        sender.bbox.y,
        title.bbox.y
    );
    // 두 표의 간격 = 저장 vpos 차 118.3px.
    assert!(
        (title.bbox.y - sender.bbox.y - 118.3).abs() < 2.0,
        "표 간격은 저장 vpos 차(118.3px)여야 한다: {:.1}",
        title.bbox.y - sender.bbox.y
    );
}

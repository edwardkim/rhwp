//! Issue #4690: 저장 `LINE_SEG.column_start` 가 정한 줄 시작에 ParaShape 내어쓰기를
//! 다시 더하면 안 된다.
//!
//! `30098` p3 의 `pi=48` 은 좌우로 갈린 두 조각이다. 오른쪽 조각의 저장값은
//! `cs=45305`(= 679.7px), `sw=2883`(= 38.4px)로 본문 오른쪽 끝(718.1px)에 정확히 닿는다.
//! 여기에 `|indent|`(41.5px)가 더해지면 줄이 본문 밖(721.2px)에서 시작하고 폭이
//! **-1.6px** 가 된다 — 그 줄의 글자는 정상적으로 그려질 수 없다.
//!
//! 쪽수·픽셀 평균·텍스트 추출 어느 지표도 이 이동을 보지 못하므로(가로 축 오라클로만
//! 잡힌다) 좌표를 직접 고정한다.
#![cfg(not(target_arch = "wasm32"))]

use std::fs;
use std::path::Path;

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::wasm_api::HwpDocument;

const SAMPLE: &str = "samples/issue4690/30098_indent_over_stored_cs.hwp";
const PAGE_INDEX: u32 = 2; // 0-based — 저장 사다리 실측이 있는 p3

fn walk<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    out.push(node);
    for child in &node.children {
        walk(child, out);
    }
}

#[test]
fn issue_4690_stored_column_start_line_keeps_body_width() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let document = HwpDocument::from_bytes(&bytes).expect("parse issue4690 sample");
    let tree = document
        .build_page_render_tree(PAGE_INDEX)
        .expect("render p3");

    let mut nodes = Vec::new();
    walk(&tree.root, &mut nodes);

    let wrap_line = nodes
        .iter()
        .copied()
        .find(|node| {
            matches!(
                &node.node_type,
                RenderNodeType::TextLine(line)
                    if line.para_index == Some(48) && line.line_index == Some(1)
            )
        })
        .expect("p3 pi=48 의 우측 조각 (저장 cs=45305)");

    // 저장값 그대로여야 한다 — cs/75 = 679.7px, sw/75 = 38.4px.
    assert!(
        (wrap_line.bbox.x - 679.7).abs() < 1.0,
        "저장 column_start 가 정한 679.7px 에서 시작해야 한다 (실제 {:.1}px). \
         내어쓰기가 이중 적용되면 721.2px 로 밀린다.",
        wrap_line.bbox.x
    );
    assert!(
        wrap_line.bbox.width > 0.0,
        "줄 폭은 양수여야 한다 (실제 {:.1}px) — 음수 폭은 글자를 그릴 수 없다는 뜻이다.",
        wrap_line.bbox.width
    );

    // 본문 오른쪽 경계를 넘지 않아야 한다.
    let body = nodes
        .iter()
        .copied()
        .find(|node| matches!(node.node_type, RenderNodeType::Body { .. }))
        .expect("Body 노드");
    let body_right = body.bbox.x + body.bbox.width;
    assert!(
        wrap_line.bbox.x <= body_right + 0.5,
        "줄 시작 {:.1}px 이 본문 오른쪽 끝 {:.1}px 을 넘었다.",
        wrap_line.bbox.x,
        body_right
    );
}

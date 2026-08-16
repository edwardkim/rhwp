//! Issue #4889: 이어지는 조각의 content origin 보정이 조각의 가시 내용을 통째로
//! 먹으면 안 된다 — 먹으면 중첩 표가 쪽 **위쪽 밖**으로 나가 사라진다.
//!
//! `mixed_nested_split_from_cut` 의 `compensate_first_visible` 은 앞 조각이 물리
//! reservation 으로 이미 전진시킨 첫 unit 을 다시 그리지 않으려고 content origin 을
//! 한 unit 앞으로 민다(42065 p12–p17). 그 전제는 "첫 unit 은 줄 하나 크기" 다.
//! 그런데 블록 중첩 표는 **표 전체가 unit 하나**라, 가시 unit 이 그 표뿐인 조각에서는
//! 표 높이만큼 밀려 조각에 아무것도 안 남는다.
//!
//! `18098267` p2 실측: `offset 36.4 + first_visible 2095.6 = 2132.0` 으로 원점이
//! 내려가, 높이 2091.9px 인 55×3 표가 `-2049.9..42.0` 에 놓여 가시 창(79.3..1084.7)에
//! 하나도 안 걸렸다. 보이는 글자가 한/글 747자 대비 **29자**였다(고친 뒤 735자).
//!
//! 이 축은 어떤 게이트도 못 본다 — 쪽수는 한/글과 같고(3/3, 10k 쪽수 게이트에서도
//! 바뀐 문서 0), 글자는 트리에 남아 있어 텍스트 추출도 통과한다. 그래서 좌표를
//! 직접 고정한다.
#![cfg(not(target_arch = "wasm32"))]

use std::fs;
use std::path::Path;

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::wasm_api::HwpDocument;

const SAMPLE: &str = "samples/issue4889/18098267_nested_fragment_origin.hwp";
const PAGE_INDEX: u32 = 1; // 0-based — 55×3 표 조각이 처음 놓이는 p2

fn walk<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    out.push(node);
    for child in &node.children {
        walk(child, out);
    }
}

#[test]
fn issue_4889_nested_fragment_origin_stays_on_page() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let document = HwpDocument::from_bytes(&bytes).expect("parse issue4889 sample");
    let tree = document
        .build_page_render_tree(PAGE_INDEX)
        .expect("render p2");

    let mut nodes = Vec::new();
    walk(&tree.root, &mut nodes);

    let nested = nodes
        .iter()
        .copied()
        .find(|node| match &node.node_type {
            RenderNodeType::Table(t) => t.row_count == 55 && t.col_count == 3,
            _ => false,
        })
        .expect("p2 에 55x3 중첩 표가 있어야 한다");

    let top = nested.bbox.y;
    let bottom = nested.bbox.y + nested.bbox.height;

    // 회귀 시 top 은 -2049.9, bottom 은 42.0 이었다 — 쪽(0..) 위로 통째로 나가 있었다.
    assert!(
        bottom > 0.0,
        "중첩 표 조각이 쪽 위쪽 밖으로 통째로 나갔다 (top={top:.1} bottom={bottom:.1})"
    );
    assert!(
        top > -0.5,
        "중첩 표 조각의 원점이 쪽 위로 밀렸다 (top={top:.1}, 기대: >= 0)"
    );

    // 이 쪽에서 실제로 글자가 보여야 한다 — 원점만 양수이고 내용이 비면 의미가 없다.
    let visible_runs = nodes
        .iter()
        .filter(|node| matches!(node.node_type, RenderNodeType::TextRun(_)))
        .filter(|node| node.bbox.y + node.bbox.height > 0.0)
        .count();
    assert!(
        visible_runs > 100,
        "p2 에 보이는 TextRun 이 {visible_runs}개뿐이다 — 조각이 사실상 비었다"
    );
}

//! Issue #4090: 빈 host의 우측 Square 표 옆 본문은 좌측 띠에 남아야 한다.
//!
//! HWP 2020 PDF p5는 `pi=44`의 non-TAC Square 표 왼쪽에 다음 `pi=45`의 앞 6줄을
//! 배치하고, 마지막 줄만 표 아래 전폭으로 되돌린다. tail의 페이지 이동만 검사하면
//! 앞 6줄이 render tree에서 소실되어도 놓친다.
#![cfg(not(target_arch = "wasm32"))]

use std::fs;
use std::path::Path;

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType, TableNode};
use rhwp::wasm_api::HwpDocument;

const SAMPLE: &str = "samples/issue4090/156492236_규제샌드박스_min.hwpx";

fn walk<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    out.push(node);
    for child in &node.children {
        walk(child, out);
    }
}

#[test]
fn issue_4090_empty_host_right_square_table_keeps_left_wrap_prefix() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let document = HwpDocument::from_bytes(&bytes).expect("parse issue4090 sample");
    let tree = document.build_page_render_tree(4).expect("render PDF p5");

    let mut nodes = Vec::new();
    walk(&tree.root, &mut nodes);
    let table = nodes
        .iter()
        .copied()
        .find(|node| {
            matches!(
                node.node_type,
                RenderNodeType::Table(TableNode {
                    para_index: Some(44),
                    ..
                })
            )
        })
        .expect("p5의 pi=44 Square 표");

    let prefix_runs: Vec<_> = nodes
        .iter()
        .copied()
        .filter(|node| {
            matches!(
                &node.node_type,
                RenderNodeType::TextRun(run)
                    if run.para_index == Some(45) && !run.text.trim().is_empty()
            ) && node.bbox.y >= table.bbox.y - 0.5
                && node.bbox.y < table.bbox.y + table.bbox.height - 0.5
                && node.bbox.x + node.bbox.width <= table.bbox.x + 0.5
        })
        .collect();
    assert!(
        !prefix_runs.is_empty(),
        "pi=45의 앞 본문이 우측 Square 표 좌측 띠에서 소실됐다: table={:?}",
        table.bbox
    );

    assert!(
        nodes.iter().copied().any(|node| {
            matches!(
                &node.node_type,
                RenderNodeType::TextRun(run)
                    if run.para_index == Some(45) && !run.text.trim().is_empty()
            ) && node.bbox.y >= table.bbox.y + table.bbox.height - 0.5
        }),
        "pi=45의 표 아래 full-width tail도 유지되어야 한다"
    );
}

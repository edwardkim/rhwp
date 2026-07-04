//! Issue #1352: HWPX 표 셀 안 TAC picture + text 세로 정렬 회귀 가드.
//!
//! `samples/hwpx/hy-001.hwpx` 첫 표 첫 셀은 `vertAlign=CENTER` 셀 안에
//! 자리차지 picture와 텍스트 `광부`가 같은 줄로 들어 있다. 한컴 PDF 기준에서는
//! picture와 텍스트가 셀 중앙 높이에 함께 놓이지만, 회귀 상태에서는 picture가
//! 아래로 밀려 셀 clip 밖으로 잘린다.

use std::fs;
use std::path::Path;

use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};
use rhwp::wasm_api::HwpDocument;

const SAMPLE: &str = "samples/hwpx/hy-001.hwpx";

fn walk<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    out.push(node);
    for child in &node.children {
        walk(child, out);
    }
}

fn all_nodes(root: &RenderNode) -> Vec<&RenderNode> {
    let mut nodes = Vec::new();
    walk(root, &mut nodes);
    nodes
}

fn node_text(node: &RenderNode) -> String {
    let mut text = String::new();
    collect_text(node, &mut text);
    text
}

fn collect_text(node: &RenderNode, out: &mut String) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        out.push_str(&run.text);
    }
    for child in &node.children {
        collect_text(child, out);
    }
}

fn find_cell_with_text<'a>(node: &'a RenderNode, text: &str) -> Option<&'a RenderNode> {
    if matches!(node.node_type, RenderNodeType::TableCell(_)) && node_text(node).contains(text) {
        return Some(node);
    }
    node.children
        .iter()
        .find_map(|child| find_cell_with_text(child, text))
}

fn first_text_run_bbox(root: &RenderNode, text: &str) -> Option<BoundingBox> {
    if matches!(&root.node_type, RenderNodeType::TextRun(run) if run.text.contains(text)) {
        return Some(root.bbox);
    }
    root.children
        .iter()
        .find_map(|child| first_text_run_bbox(child, text))
}

#[test]
fn hy001_first_cell_tac_picture_stays_inside_center_aligned_cell() {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let path = Path::new(repo_root).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", SAMPLE, e));
    let doc = HwpDocument::from_bytes(&bytes).unwrap_or_else(|e| panic!("parse {}: {}", SAMPLE, e));
    let tree = doc.build_page_render_tree(0).expect("render hy-001 page 1");

    let cell = find_cell_with_text(&tree.root, "광부").expect("`광부`가 있는 표 셀 누락");
    let text_bbox = first_text_run_bbox(cell, "광부").expect("`광부` TextRun 누락");
    let images: Vec<&RenderNode> = all_nodes(cell)
        .into_iter()
        .filter(|node| matches!(node.node_type, RenderNodeType::Image(_)))
        .collect();
    assert_eq!(
        images.len(),
        1,
        "`광부` 셀에는 TAC picture가 하나만 있어야 함"
    );
    let image = images[0];

    let cell_bottom = cell.bbox.y + cell.bbox.height;
    let image_bottom = image.bbox.y + image.bbox.height;
    let text_image_y_delta = (image.bbox.y - text_bbox.y).abs();

    eprintln!(
        "[issue_1352] cell=[y={:.2}, h={:.2}] text=[y={:.2}, h={:.2}] image=[y={:.2}, h={:.2}]",
        cell.bbox.y,
        cell.bbox.height,
        text_bbox.y,
        text_bbox.height,
        image.bbox.y,
        image.bbox.height,
    );

    assert!(
        image_bottom <= cell_bottom + 1.0,
        "TAC picture가 셀 clip 아래로 잘리면 안 됨: image_bottom={image_bottom:.2}, cell_bottom={cell_bottom:.2}"
    );
    assert!(
        text_image_y_delta <= 4.0,
        "같은 줄의 picture와 `광부` 텍스트 y 위치가 과도하게 벌어지면 안 됨: delta={text_image_y_delta:.2}"
    );
}

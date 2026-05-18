//! Issue #990: 빈 문단 위 treat-as-char 글상자 — `FullParagraph`/`Shape` PageItem
//! advance 이중 가산.
//!
//! 본질: 빈 문단(텍스트 없음) 위에 얹힌 `treat_as_char` 글상자(도형)가 세로로
//! 연속 배치될 때, `FullParagraph` 와 `Shape` 두 PageItem 이 각각 `y_offset` 을
//! 진행시켜 박스 사이 세로 advance 가 LINE_SEG 1회분(`lh + ls`)의 2배가 된다.
//!
//! 픽스처 `samples/issue-990-tac-box.hwpx` 4쪽(global_idx=3)에는 `※ …` 글상자
//! 3개가 빈 문단(`pi=54/55/56`) 위에 연속 배치되어 있다.

use std::fs;
use std::path::Path;

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

/// 글상자(사각형) 노드를 para_index 와 함께 수집한다.
fn collect_box_rects<'a>(node: &'a RenderNode, out: &mut Vec<(usize, f64, f64)>) {
    if let RenderNodeType::Rectangle(rect) = &node.node_type {
        if rect.section_index == Some(0)
            && rect.control_index == Some(0)
            && matches!(rect.para_index, Some(54) | Some(55) | Some(56))
        {
            out.push((rect.para_index.unwrap(), node.bbox.y, node.bbox.height));
        }
    }
    for child in &node.children {
        collect_box_rects(child, out);
    }
}

#[test]
fn issue_990_empty_para_tac_box_advance_not_doubled() {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let hwp_path = Path::new(repo_root).join("samples/issue-990-tac-box.hwpx");
    let bytes =
        fs::read(&hwp_path).unwrap_or_else(|e| panic!("read {}: {}", hwp_path.display(), e));

    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("parse issue-990 fixture");

    // 4쪽 = global_idx 3
    let tree = doc
        .build_page_render_tree(3)
        .expect("render issue-990 fixture page 4");

    let mut boxes = Vec::new();
    collect_box_rects(&tree.root, &mut boxes);
    boxes.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

    assert_eq!(
        boxes.len(),
        3,
        "4쪽 글상자 3개(pi=54/55/56)를 찾지 못함: {boxes:?}"
    );

    let (_, y1, h1) = boxes[0];
    let (_, y2, _) = boxes[1];
    let (_, y3, _) = boxes[2];
    let advance_1_2 = y2 - y1;
    let advance_2_3 = y3 - y2;

    println!(
        "issue #990 box y=[{y1:.2}, {y2:.2}, {y3:.2}] h1={h1:.2} \
         advance=[{advance_1_2:.2}, {advance_2_3:.2}]"
    );

    // 빈 호스트 문단의 LINE_SEG advance(lh+ls = 4983 HU ≈ 66.4px) 1회분이어야 한다.
    // 이중 가산 버그 시 advance ≈ 132.9px (정확히 2배).
    assert!(
        advance_1_2 < 90.0,
        "박스1→박스2 advance 이중 가산: {advance_1_2:.2}px (기대 ≈66px, LINE_SEG 1회분)"
    );
    assert!(
        advance_2_3 < 90.0,
        "박스2→박스3 advance 이중 가산: {advance_2_3:.2}px (기대 ≈66px, LINE_SEG 1회분)"
    );
}

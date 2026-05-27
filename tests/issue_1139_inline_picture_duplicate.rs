//! Task #1139: 문27 inline TAC Picture가 다음 줄까지 미리 렌더되어 중복 출력되던 회귀 방지.

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::wasm_api::HwpDocument;

fn collect_small_bin5_images(node: &RenderNode, out: &mut Vec<(Option<usize>, Option<usize>)>) {
    if let RenderNodeType::Image(img) = &node.node_type {
        if img.bin_data_id == 5 && (node.bbox.width - 23.8).abs() < 0.1 {
            out.push((img.para_index, img.control_index));
        }
    }
    for child in &node.children {
        collect_small_bin5_images(child, out);
    }
}

fn render_tree_contains_text(node: &RenderNode, needle: &str) -> bool {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if run.text.contains(needle) {
            return true;
        }
    }
    node.children
        .iter()
        .any(|child| render_tree_contains_text(child, needle))
}

#[test]
fn issue_1139_small_inline_picture_rendered_once_per_control() {
    let bytes = std::fs::read("samples/3-09월_교육_통합_2022.hwp").expect("sample");
    let doc = HwpDocument::from_bytes(&bytes).expect("parse");
    let tree = doc.build_page_render_tree(4).expect("page 5 render tree");

    let mut images = Vec::new();
    collect_small_bin5_images(&tree.root, &mut images);
    images.sort();

    assert_eq!(
        images,
        vec![(Some(321), Some(10)), (Some(323), Some(4))],
        "문27 작은 inline Picture는 원본 컨트롤 2개만 렌더되어야 함"
    );
}

#[test]
fn issue_1139_exam_2022_page_count_matches_hancom_after_endnotes() {
    let bytes = std::fs::read("samples/3-09월_교육_통합_2022.hwp").expect("sample");
    let doc = HwpDocument::from_bytes(&bytes).expect("parse");

    assert_eq!(doc.page_count(), 23, "한컴오피스 기준 페이지 수");

    let page9 = doc.dump_page_items(Some(8));
    assert!(
        page9.contains("FullParagraph[미주]  pi=523"),
        "9쪽에서 pi=522 뒤 미주가 같은 쪽에 이어져야 함\n{page9}"
    );
}

#[test]
fn issue_1139_page9_endnote_shape_textbox_is_rendered() {
    let bytes = std::fs::read("samples/3-09월_교육_통합_2022.hwp").expect("sample");
    let doc = HwpDocument::from_bytes(&bytes).expect("parse");
    let tree = doc.build_page_render_tree(8).expect("page 9 render tree");

    assert!(
        render_tree_contains_text(&tree.root, "다른 풀이"),
        "9쪽 문7 미주 내부 TAC Shape 그룹의 글상자 텍스트가 렌더되어야 함"
    );
}

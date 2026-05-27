//! Task #1139: 문27 inline TAC Picture가 다음 줄까지 미리 렌더되어 중복 출력되던 회귀 방지.

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::wasm_api::HwpDocument;
use serde_json::Value;

fn hwpunit_to_mm(hu: i32) -> f64 {
    hu as f64 * 25.4 / 7200.0
}

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
fn issue_1139_exam_2022_endnote_shape_matches_hancom_reference() {
    let bytes = std::fs::read("samples/3-09월_교육_통합_2022.hwp").expect("sample");
    let doc = HwpDocument::from_bytes(&bytes).expect("parse");
    let shape = &doc.document().sections[0].section_def.endnote_shape;

    assert_eq!(shape.prefix_char, '문');
    assert_eq!(shape.suffix_char, '\u{ff09}');
    assert!((hwpunit_to_mm(shape.separator_length as i32) - 50.0).abs() < 0.05);
    assert_eq!(shape.separator_margin_top, 0);
    assert!(
        (hwpunit_to_mm(shape.note_spacing as i32) - 2.0).abs() < 0.05,
        "HWP5 binary field maps to Hancom '구분선 아래'"
    );
    assert!(
        (hwpunit_to_mm(shape.raw_unknown as i32) - 7.0).abs() < 0.05,
        "HWP5 raw_unknown preserves Hancom '미주 사이'"
    );
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
    let page10 = doc.dump_page_items(Some(9));
    assert!(
        page9.contains("FullParagraph[미주]  pi=522"),
        "9쪽에는 문7 미주 마지막 문단 pi=522가 남아야 함\n{page9}"
    );
    assert!(
        !page9.contains("FullParagraph[미주]  pi=523"),
        "한컴오피스 기준 문8 미주 pi=523은 9쪽에 들어가면 안 됨\n{page9}"
    );
    assert!(
        page10.contains("FullParagraph[미주]  pi=523"),
        "한컴오피스 기준 문8 미주 pi=523은 10쪽에서 시작해야 함\n{page10}"
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

#[test]
fn issue_1139_page9_endnote_shape_properties_resolve_virtual_para_index() {
    let bytes = std::fs::read("samples/3-09월_교육_통합_2022.hwp").expect("sample");
    let doc = HwpDocument::from_bytes(&bytes).expect("parse");
    let layout = doc
        .get_page_control_layout_native(8)
        .expect("page 9 control layout");
    let parsed: Value = serde_json::from_str(&layout).expect("control layout json");

    let group = parsed["controls"]
        .as_array()
        .expect("controls array")
        .iter()
        .find(|ctrl| {
            ctrl["type"] == "group"
                && ctrl["paraIdx"].as_u64() == Some(518)
                && ctrl["controlIdx"].as_u64() == Some(0)
        })
        .expect("문7 [다른 풀이] group shape");

    assert_eq!(group["secIdx"].as_u64(), Some(0));

    let props = doc
        .get_shape_properties_native(0, 518, 0)
        .expect("미주 가상 문단 Shape 속성 조회");
    let props: Value = serde_json::from_str(&props).expect("shape props json");
    assert!(
        props["width"].as_u64().unwrap_or(0) > 0,
        "미주 내부 Shape 속성이 실제 값으로 조회되어야 함: {props}"
    );
}

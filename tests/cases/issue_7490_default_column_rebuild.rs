//! #7490 기본 단 정의 재저장의 원본 계약과 정상 입력의 배치를 검사한다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::wasm_api::HwpDocument;
use std::path::Path;

#[test]
fn default_column_rebuild_preserves_tac_slice_source_contract() {
    // #5700 축약본의 독립 Print는 12쪽이다. 기존 엔진의 10쪽과 동일한 배치를
    // 정답으로 고정하지 않고, 아래 원본 내용·용지·문단 모양의 저장 계약은 유지한다.
    rebuild_preserving_source_contract(
        "samples/issue5701/1530000-200800002_slice_p139_tac_reset_tail.hwp",
    );
}

#[test]
fn default_column_rebuild_preserves_no_column_source_contracts() {
    // 단 정의를 추가하면 한컴이 활성화하는 용지가 달라진다. 원본 선언 속성의
    // 저장 계약과 서로 다른 활성 용지에서의 배치 동일성을 혼동하지 않는다.
    for input in [
        "samples/issue5802/hf_cross_section_inherit.hwp",
        "samples/issue5871/ws_host_float_double_charge.hwp",
        "samples/issue6133/156483831_poster_title_above_offset_float.hwp",
        "samples/issue6135/156544683_title_row_underfit.hwp",
        "samples/issue6184/156489124_tail_line_before_deferred_table.hwp",
        "samples/issue6196/cell_char_spacing_fit.hwp",
        "samples/issue6204/square_picture_band_host.hwp",
    ] {
        rebuild_preserving_source_contract(input);
    }
}

#[test]
fn explicit_column_rebuild_preserves_content_and_object_geometry() {
    // 이 입력에는 이미 본문 단 정의가 있어 재작성 전후 유효 용지가 같다.
    let samples = ["samples/issue6190/center_align_first_line_indent.hwp"];
    fn painted_boxes(node: &RenderNode, out: &mut Vec<(f64, f64, f64, f64)>) {
        if matches!(
            node.node_type,
            RenderNodeType::Table(_)
                | RenderNodeType::TextRun(_)
                | RenderNodeType::Image(_)
                | RenderNodeType::Line(_)
                | RenderNodeType::Rectangle(_)
                | RenderNodeType::Ellipse(_)
                | RenderNodeType::Path(_)
                | RenderNodeType::Group(_)
                | RenderNodeType::TextBox
                | RenderNodeType::Equation(_)
                | RenderNodeType::FormObject(_)
                | RenderNodeType::Placeholder(_)
                | RenderNodeType::RawSvg(_)
        ) {
            out.push((node.bbox.x, node.bbox.y, node.bbox.width, node.bbox.height));
        }
        for child in &node.children {
            painted_boxes(child, out);
        }
    }
    for input in samples {
        let (bytes, saved) = rebuild_preserving_source_contract(input);
        let before = HwpDocument::from_bytes(&bytes).expect("source layout");
        let after = HwpDocument::from_bytes(&saved).expect("saved layout");
        assert_eq!(before.page_count(), after.page_count(), "{input}: pages");
        for page in 0..before.page_count() {
            let mut old_boxes = Vec::new();
            let mut new_boxes = Vec::new();
            painted_boxes(
                &before
                    .build_page_render_tree(page)
                    .expect("source tree")
                    .root,
                &mut old_boxes,
            );
            painted_boxes(
                &after.build_page_render_tree(page).expect("saved tree").root,
                &mut new_boxes,
            );
            assert_eq!(
                old_boxes.len(),
                new_boxes.len(),
                "{input} page {page}: painted node ownership"
            );
            for (a, b) in old_boxes.iter().zip(&new_boxes) {
                assert!(
                    same_position(a.0, b.0)
                        && same_position(a.1, b.1)
                        && same_position(a.2, b.2)
                        && same_position(a.3, b.3),
                    "{input} page {page}: object geometry {a:?} -> {b:?}"
                );
            }
        }
    }
}

fn rebuild_preserving_source_contract(input: &str) -> (Vec<u8>, Vec<u8>) {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(input);
    let bytes = std::fs::read(path).expect("source");
    let source = rhwp::parser::parse_document(&bytes).expect("source IR");
    let mut rebuilt = source.clone();
    rebuilt.doc_info.raw_stream_dirty = true;
    for section in &mut rebuilt.sections {
        section.raw_stream = None;
    }
    let saved = rhwp::serializer::serialize_document(&rebuilt).expect("rebuilt save");
    let reopened = rhwp::parser::parse_document(&saved).expect("rebuilt IR");
    assert_eq!(source.sections.len(), reopened.sections.len(), "{input}");
    for (before, after) in source.sections.iter().zip(&reopened.sections) {
        assert_eq!(
            format!("{:?}", before.section_def.page_def),
            format!("{:?}", after.section_def.page_def),
            "{input}: paper"
        );
        assert_eq!(
            before.paragraphs.len(),
            after.paragraphs.len(),
            "{input}: paragraphs"
        );
        for (before, after) in before.paragraphs.iter().zip(&after.paragraphs) {
            assert_eq!(before.text, after.text, "{input}: content");
            assert_eq!(
                before.para_shape_id, after.para_shape_id,
                "{input}: paragraph style"
            );
        }
    }
    (bytes, saved)
}

fn same_position(a: f64, b: f64) -> bool {
    (a - b).abs() <= f64::EPSILON * a.abs().max(b.abs()).max(1.0) * 64.0
}

//! [#5543] 자리차지 프레임·저장 기준선과 후속 문단의 소유 계약.
//!
//! 독립 기준은 메인터너가 출력한 pdf/issue5543_carried_anchor_ladder-2020.pdf다.
//! 전체 4쪽이며 ‘기상이변…’은 4쪽에 있다. 저장본의 재저장 지문만으로 주장했던
//! ‘compat2024이면 3쪽’은 실제 출력으로 확인되지 않았다. 사용자 승인에 따라
//! 2020 출력의 계약을 검사하며, 한컴 2024 실제 출력은 미검증으로 남긴다.
//! 2쪽의 표→글상자 간격은 저장 줄간격을 한 번만 소비하고, 글상자 안 두 로고는
//! 원본 LineSeg의 기준선을 공유한다. 절대 화면 좌표나 그림 폭으로 노드를 고르지 않는다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::shape::ShapeObject;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::wasm_api::HwpDocument;

fn sample_bytes() -> Vec<u8> {
    std::fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/issue5543_carried_anchor_ladder.hwpx"),
    )
    .expect("sample")
}

/// 문단이 처음 등장하는 1-기반 쪽 번호.
fn para_page(doc: &HwpDocument, target_pi: u64) -> u64 {
    let v = doc.dump_page_items_json(None);
    let pages = v.as_array().expect("pages array");
    for (pg_idx, page) in pages.iter().enumerate() {
        for col in page["columns"].as_array().expect("columns") {
            for item in col["items"].as_array().expect("items") {
                if item["paraIndex"].as_u64() == Some(target_pi) {
                    return (pg_idx as u64) + 1;
                }
            }
        }
    }
    panic!("pi {target_pi} 미배치");
}

#[test]
fn compatibility_toggle_preserves_reference_page_ownership() {
    let mut doc = HwpDocument::from_bytes(&sample_bytes()).expect("parse");
    for compatibility in [false, true, false] {
        doc.set_hangul2024_compat(compatibility);
        assert_eq!(doc.page_count(), 4, "독립 2020 정본의 전체 쪽수");
        assert_eq!(para_page(&doc, 42), 4, "기상이변 문단의 소유 쪽");
        assert_eq!(para_page(&doc, 43), 4, "뒤 문단도 같은 쪽에 보존");
    }
}

fn collect<'a>(node: &'a RenderNode, nodes: &mut Vec<&'a RenderNode>) {
    nodes.push(node);
    for child in &node.children {
        collect(child, nodes);
    }
}

fn contains_image(node: &RenderNode, id: u16) -> bool {
    matches!(&node.node_type, RenderNodeType::Image(image) if image.bin_data_id == id)
        || node.children.iter().any(|child| contains_image(child, id))
}

#[test]
fn table_textbox_spacing_and_inline_logos_share_saved_metrics() {
    let core = DocumentCore::from_bytes(&sample_bytes()).expect("parse");
    let host = &core.document().sections[0].paragraphs[25];
    let Control::Table(table) = &host.controls[0] else {
        panic!("source first table");
    };
    let Control::Shape(shape) = &host.controls[1] else {
        panic!("source logo shape");
    };
    let ShapeObject::Rectangle(shape) = shape.as_ref() else {
        panic!("source logo textbox");
    };
    let source_box = shape.drawing.text_box.as_ref().expect("source textbox");
    let source_line = &source_box.paragraphs[0].line_segs[0];
    let ids: Vec<_> = source_box.paragraphs[0]
        .controls
        .iter()
        .filter_map(|control| match control {
            Control::Picture(picture) => Some(picture.image_attr.bin_data_id),
            _ => None,
        })
        .collect();
    assert_eq!(ids.len(), 2, "two source logos");
    assert_ne!(ids[0], ids[1]);
    let tree = core.build_page_render_tree(1).expect("page 2");
    let mut nodes = Vec::new();
    collect(&tree.root, &mut nodes);
    let preceding: Vec<_> = nodes
        .iter()
        .copied()
        .filter(|node| {
            matches!(&node.node_type, RenderNodeType::Table(t)
            if t.section_index == Some(0) && t.para_index == Some(25)
                && t.control_index == Some(0) && t.cell_context.is_none())
        })
        .collect();
    assert_eq!(preceding.len(), 1, "source table placed once");
    let frames: Vec<_> = nodes
        .iter()
        .copied()
        .filter(|node| {
            matches!(node.node_type, RenderNodeType::Rectangle(_))
                && ids.iter().all(|id| contains_image(node, *id))
        })
        .collect();
    assert_eq!(frames.len(), 1, "logos belong to one outer shape frame");
    let frame = frames[0];
    let boxes: Vec<_> = nodes
        .iter()
        .copied()
        .filter(|node| {
            matches!(node.node_type, RenderNodeType::TextBox)
                && ids.iter().all(|id| contains_image(node, *id))
        })
        .collect();
    assert_eq!(boxes.len(), 1, "one textbox owner");
    let textbox = boxes[0];
    // 독립 원본의 HU 메트릭. 96dpi의 1HU만 직렬화 오차로 허용한다.
    let hu = 96.0 / 7200.0;
    let source_gap =
        f64::from(table.common.margin.bottom) + f64::from(host.line_segs[0].line_spacing);
    let actual_gap = frame.bbox.y - (preceding[0].bbox.y + preceding[0].bbox.height);
    assert!(
        (actual_gap - source_gap * hu).abs() <= hu,
        "표 뒤 간격은 저장 줄간격과 바깥여백을 한 번만 소비: {actual_gap}"
    );
    assert!(
        (textbox.bbox.y - frame.bbox.y - f64::from(source_box.margin_top) * hu).abs() <= hu,
        "textbox top padding is consumed once"
    );
    let mut logos = Vec::new();
    for id in ids {
        let images: Vec<_> = nodes.iter().copied().filter(|node| {
            matches!(&node.node_type, RenderNodeType::Image(image) if image.bin_data_id == id)
        }).collect();
        assert_eq!(images.len(), 1, "each source logo painted once");
        let image = images[0];
        assert!(image.bbox.y >= textbox.bbox.y - hu);
        assert!((image.bbox.y + image.bbox.height) <= (textbox.bbox.y + textbox.bbox.height) + hu);
        logos.push(image);
    }
    assert!(source_line.line_height > 0);
    let baseline_ratio =
        f64::from(source_line.baseline_distance) / f64::from(source_line.line_height);
    let baseline = |node: &RenderNode| node.bbox.y + node.bbox.height * baseline_ratio;
    assert!(
        (baseline(logos[0]) - baseline(logos[1])).abs() <= hu,
        "different-height inline logos share the original saved baseline"
    );
}

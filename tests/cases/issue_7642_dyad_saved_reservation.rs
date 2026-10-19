//! 공개 원본의 큰 DYAD 예약 프레임과 셀 본문 기준선 관계.
//! 같은 원본의 Native/fresh WASM 전체 1쪽 98.50470%, 최소 대조군 99.17678%.
//! 저장 속성·소유·간격 관계로 검사하며 실물 절대 픽셀 위치를 고정하지 않는다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::renderer::{hwpunit_to_px, DEFAULT_DPI};

fn descendants<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    out.push(node);
    for child in &node.children {
        descendants(child, out);
    }
}

#[test]
fn saved_dyad_reservation_keeps_the_cell_text_below_the_visible_math() {
    let bytes = include_bytes!("../../samples/issue7593/repro-equation-issue7593.hwpx");
    let core = DocumentCore::from_bytes(bytes).unwrap();
    assert_eq!(core.page_count(), 1, "독립 Print의 전체 내용은 한 쪽이다");
    let tree = core.build_page_render_tree(0).unwrap();
    let mut nodes = Vec::new();
    descendants(&tree.root, &mut nodes);
    let line = nodes
        .iter()
        .find(|node| {
            node.children.iter().any(|child| {
                matches!(&child.node_type, RenderNodeType::Equation(eq)
            if eq.script == "dyad{rm AB}" && eq.cell_index.is_some())
            })
        })
        .expect("보기 ㄴ의 수식과 본문이 소속된 실제 줄");
    let (node, eq) = line
        .children
        .iter()
        .find_map(|node| match &node.node_type {
            RenderNodeType::Equation(eq) if eq.script == "dyad{rm AB}" => Some((node, eq)),
            _ => None,
        })
        .unwrap();
    let (text_node, text) = line
        .children
        .iter()
        .find_map(|node| match &node.node_type {
            RenderNodeType::TextRun(text) if text.text.contains("이다") => Some((node, text)),
            _ => None,
        })
        .expect("수식 뒤 본문 보존");
    let context = text.cell_context.as_ref().unwrap();
    let cell = context.path.first().unwrap();
    assert_eq!(eq.para_index, Some(context.parent_para_index));
    assert_eq!(eq.control_index, Some(cell.control_index));
    assert_eq!(eq.cell_index, Some(cell.cell_index));
    assert_eq!(eq.cell_para_index, Some(cell.cell_para_index));
    assert_eq!(text.para_index, Some(cell.cell_para_index));
    let Control::Table(table) = &core.document().sections[0].paragraphs[eq.para_index.unwrap()]
        .controls[eq.control_index.unwrap()]
    else {
        panic!("보기 수식의 표 소유");
    };
    let paragraph = &table.cells[eq.cell_index.unwrap()].paragraphs[eq.cell_para_index.unwrap()];
    let saved = paragraph
        .controls
        .iter()
        .find_map(|control| match control {
            Control::Equation(saved) if saved.script == eq.script => Some(saved),
            _ => None,
        })
        .unwrap();
    let declared_height = hwpunit_to_px(saved.common.height as i32, DEFAULT_DPI);
    assert!(
        declared_height > eq.layout_box.height,
        "독립 원본은 큰 예약 프레임이다"
    );
    assert!(
        (node.bbox.height / declared_height - 1.0).abs() <= f64::EPSILON * 128.0,
        "실제 점유 높이는 저장 예약 영역을 보존해야 한다"
    );
    let text_baseline = text_node.bbox.y + text.baseline;
    let anchor_ratio = (text_baseline - node.bbox.y) / declared_height;
    assert!(
        (anchor_ratio - f64::from(saved.baseline) / 100.0).abs() <= f64::EPSILON * 128.0,
        "본문 줄의 기준선은 독립 원본의 예약 영역 기준선을 소비한다"
    );
    assert!(
        node.bbox.x + node.bbox.width <= text_node.bbox.x,
        "앞뒤 내용은 겹치지 않는다"
    );
    assert!(
        node.bbox.y + eq.layout_box.y + eq.layout_box.baseline < text_baseline,
        "Print의 보기 ㄴ 수식은 뒤 본문 기준선보다 위에 표시된다"
    );
}

//! #7491: 실제 셀 높이가 선언 높이보다 클 때 글줄과 표의 점유 영역을 공유한다.
//! 독립 한컴 Print PDF에서 같은 줄의 본문 기준선은 셀 글자 기준선 아래에 있다.
//! Native/fresh WASM Visual Sweep 100%인 공개 편집 입력을 사용한다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

fn descendants<'a>(node: &'a RenderNode, nodes: &mut Vec<&'a RenderNode>) {
    nodes.push(node);
    for child in &node.children {
        descendants(child, nodes);
    }
}

fn check_band(core: &DocumentCore) {
    assert_eq!(core.page_count(), 1, "작은 표는 머리 문단과 같은 쪽이다");
    let paragraph = &core.document().sections[0].paragraphs[1];
    assert_eq!(
        paragraph.line_segs.len(),
        1,
        "접두 글자와 표는 같은 저장 줄이다"
    );
    assert_eq!(paragraph.logical_control_positions(), vec![1]);
    let tree = core.build_page_render_tree(0).unwrap();
    let mut nodes = Vec::new();
    descendants(&tree.root, &mut nodes);
    let text = |word: &str| {
        let matches: Vec<_> = nodes
            .iter()
            .filter_map(|node| match &node.node_type {
                RenderNodeType::TextRun(run) if run.text == word => Some((*node, run)),
                _ => None,
            })
            .collect();
        assert_eq!(matches.len(), 1, "'{word}'은 한 번만 그린다");
        matches[0]
    };
    let (prefix, prefix_run) = text("앞");
    let (cell, cell_run) = text("칸");
    let tables: Vec<_> = nodes
        .iter()
        .filter_map(|node| match &node.node_type {
            RenderNodeType::Table(table) => Some((*node, table)),
            _ => None,
        })
        .collect();
    assert_eq!(tables.len(), 1);
    let (table, owner) = tables[0];
    assert_eq!(owner.para_index, Some(1));
    assert_eq!(prefix_run.para_index, owner.para_index);
    assert!(prefix_run.cell_context.is_none());
    assert!(cell_run.cell_context.is_some());
    assert!(prefix.bbox.x + prefix.bbox.width <= table.bbox.x);
    let Control::Table(stored_table) = &paragraph.controls[0] else {
        panic!("공개 편집 입력의 표를 보존한다");
    };
    // Print에서 확인한 저장 밴드: 표 위 간격은 바깥여백이며 추가 기준선 여백이 아니다.
    // 절대 좌표 대신 저장 줄의 본문 높이에 대한 여백 비율을 대조한다.
    let stored_body_height = paragraph.line_segs[0].line_height
        - i32::from(stored_table.outer_margin_top)
        - i32::from(stored_table.outer_margin_bottom);
    assert!(stored_body_height > stored_table.common.height as i32);
    let expected_ratio = f64::from(stored_table.outer_margin_top) / f64::from(stored_body_height);
    let actual_ratio = (table.bbox.y - prefix.bbox.y) / table.bbox.height;
    assert!(
        (actual_ratio - expected_ratio).abs() <= f64::EPSILON * 128.0,
        "저장 외곽 밴드 위 여백을 두 번 더하지 않는다: 실제 비율={actual_ratio}, 저장 비율={expected_ratio}"
    );
    let bottom = table.bbox.y + table.bbox.height;
    assert!(cell.bbox.x >= table.bbox.x);
    assert!(cell.bbox.x + cell.bbox.width <= table.bbox.x + table.bbox.width);
    assert!(cell.bbox.y >= table.bbox.y);
    assert!(cell.bbox.y + cell.bbox.height <= bottom);
    let prefix_baseline = prefix.bbox.y + prefix_run.baseline;
    let cell_baseline = cell.bbox.y + cell_run.baseline;
    assert!(
        prefix_baseline > cell_baseline,
        "독립 Print의 기준선 순서를 보존한다: 본문={prefix_baseline}, 셀={cell_baseline}"
    );
    let following = nodes
        .iter()
        .find(|node| matches!(&node.node_type, RenderNodeType::TextLine(line) if line.para_index == Some(2)))
        .expect("표 뒤의 빈 문단도 공간을 점유한다");
    assert!(following.bbox.y >= bottom);
}

#[test]
fn measured_inline_table_band_preserves_prefix_and_cell_baseline_order() {
    let bytes = include_bytes!("../fixtures/issue7491/smallfit-measured.hwp");
    let core = DocumentCore::from_bytes(bytes).unwrap();
    check_band(&core);
    let reopened = DocumentCore::from_bytes(&core.export_hwp_native().unwrap()).unwrap();
    check_band(&reopened);
}

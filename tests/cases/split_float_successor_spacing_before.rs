//! 분할된 자리차지 표의 이어지는 조각 바로 뒤 문단은 앞 간격을 조각 안에서 소비한다.
//!
//! 이어지는 조각은 단 위끝에서 시작하고 뒤 문단의 상단도 그 단 위끝이다. 앞 간격은 문단
//! 상단에서 재므로 첫 글줄은 `max(단 위끝 + 앞 간격, 조각 끝)` = 조각 끝에 놓인다. 한/글
//! 저장 vpos 가 그 자리를 적을 때 그대로 따른다(교육부 보도자료 2쪽: 조각 끝 404.8px 에
//! 첫 줄, rhwp 는 앞 간격 20px 를 더 내려 닫는 표 둘이 3쪽으로 밀렸다).
#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::control::Control;
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};
use rhwp::wasm_api::HwpDocument;

fn collect(
    node: &RenderNode,
    tables: &mut Vec<BoundingBox>,
    lines: &mut Vec<(usize, BoundingBox)>,
) {
    match &node.node_type {
        RenderNodeType::Table(_) => {
            tables.push(node.bbox.clone());
            return;
        }
        RenderNodeType::TextLine(line) => {
            if let Some(pi) = line.para_index {
                lines.push((pi, node.bbox.clone()));
            }
        }
        _ => {}
    }
    for child in &node.children {
        collect(child, tables, lines);
    }
}

#[test]
fn paragraph_after_split_float_continuation_absorbs_spacing_before() {
    let mut doc = HwpDocument::create_empty();
    doc.create_blank_document_native().unwrap();
    doc.create_table_native(0, 0, 0, 60, 2).unwrap();
    let (host_para, table_ctrl) = doc.document().sections[0]
        .paragraphs
        .iter()
        .enumerate()
        .find_map(|(pi, para)| {
            para.controls
                .iter()
                .position(|c| matches!(c, Control::Table(_)))
                .map(|ci| (pi, ci))
        })
        .unwrap();
    doc.set_table_properties_native(
        0,
        host_para,
        table_ctrl,
        r#"{"treatAsChar":false,"textWrap":"TopAndBottom","vertRelTo":"Para","vertAlign":"Top","vertOffset":0}"#,
    )
    .unwrap();
    let next = host_para + 1;
    doc.insert_text_native(0, next, 0, "다음 문단").unwrap();
    doc.apply_para_format_native(0, next, r#"{"spacingBefore":3000}"#)
        .unwrap();
    // 저장본으로 다시 열고, 한/글 저장 사다리처럼 뒤 문단 첫 줄을 표 바깥 상자 바로 뒤에 둔다.
    let mut doc = HwpDocument::from_bytes(&doc.export_hwpx_native().unwrap()).unwrap();
    let mut model = doc.document().clone();
    let paragraphs = &mut model.sections[0].paragraphs;
    let Control::Table(table) = &paragraphs[host_para].controls[table_ctrl] else {
        unreachable!()
    };
    let box_end = paragraphs[host_para].line_segs[0].vertical_pos
        + table.outer_margin_top as i32
        + table.common.height as i32
        + table.outer_margin_bottom as i32;
    paragraphs[next].line_segs[0].vertical_pos = box_end;
    doc.set_document(model);
    assert_eq!(doc.page_count(), 2, "60행 표는 두 쪽에 걸친다");

    let tree = doc.build_page_render_tree(1).unwrap();
    let (mut tables, mut lines) = (Vec::new(), Vec::new());
    collect(&tree.root, &mut tables, &mut lines);
    let [fragment] = tables.as_slice() else {
        panic!("2쪽 표 조각 하나: {tables:?}");
    };
    let fragment_bottom = fragment.y + fragment.height;
    let line = lines
        .iter()
        .find(|(pi, _)| *pi == next)
        .expect("뒤 문단 글줄")
        .1
        .clone();
    let gap = line.y - fragment_bottom;
    assert!(
        (-0.5..5.0).contains(&gap),
        "뒤 문단 첫 줄은 앞 간격 없이 조각 끝에 놓여야 한다: 조각 끝 {fragment_bottom:.1}, 글줄 {line:?}"
    );
}

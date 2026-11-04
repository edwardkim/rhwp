//! 실제 한컴 저장 RowBreak 표의 마지막 조각 뒤에서 문단 앞 간격을 중복 소비하지 않는다.
//!
//! 원 #8055 factory는 None 표와 수동 LineSeg로 분할을 가정했다. 원본은 같은 asset에
//! 보존하고 실제 분할 계약은 한컴 저장 RowBreak와 그 자체의 독립 Print로 확인한다.
//! 전쪽 Native/fresh raw100%와 확대 판독 근거는 float_fragment_frames_20261020에 있다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};

fn collect(
    node: &RenderNode,
    tables: &mut Vec<BoundingBox>,
    lines: &mut Vec<(usize, BoundingBox)>,
) {
    match &node.node_type {
        RenderNodeType::Table(_) => {
            tables.push(node.bbox);
            return;
        }
        RenderNodeType::TextLine(line) => {
            if let Some(pi) = line.para_index {
                lines.push((pi, node.bbox));
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
    let input = include_bytes!(
        "../../mydocs/pr/assets/float_fragment_frames_20261020/inputs/hwp/public-rowbreak-normalized-2020.hwp"
    );
    let doc = DocumentCore::from_bytes(input).unwrap();
    let source = &doc.document().sections[0].paragraphs;
    let (host, table) = source
        .iter()
        .enumerate()
        .find_map(|(pi, para)| {
            para.controls.iter().find_map(|control| match control {
                Control::Table(table) => Some((pi, table)),
                _ => None,
            })
        })
        .unwrap();
    assert_eq!(format!("{:?}", table.page_break), "RowBreak");
    assert_eq!(source[host].line_segs.len(), 1);
    assert_eq!(source[host].line_segs[0].tag & 0x8000_0000, 0);
    let next = source
        .iter()
        .position(|p| p.text.contains("다음 문단"))
        .unwrap();
    assert!(next > host);
    assert_eq!(doc.page_count(), 2, "독립 Print의 50행·10행 두 쪽");

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
    let source_unit = rhwp::renderer::hwpunit_to_px(1, 96.0);
    let outer_bottom = rhwp::renderer::hwpunit_to_px(i32::from(table.outer_margin_bottom), 96.0);
    assert!(
        gap >= -source_unit && gap <= outer_bottom + source_unit,
        "뒤 글줄은 마지막 조각 뒤 바깥여백 안에 놓인다: gap={gap}, margin={outer_bottom}"
    );
}

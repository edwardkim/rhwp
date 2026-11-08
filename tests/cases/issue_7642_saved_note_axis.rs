//! 각주 표식이 있는 저장 본문 뒤 표가 문단의 간격을 보존한다.
//!
//! 같은 저장본의 독립 한컴 2020 Print 전체 11쪽과 Native/fresh WASM을
//! 대조했다. 최저 95.76069% / 95.76066%, 미달 0쪽이다.
//! 증거: mydocs/working/task_m100_7642_stage164.md와
//! mydocs/pr/assets/issue7642-saved-note-axis-20261012/README.md.
//! 절대 좌표 대신 실제 앞 글줄 끝과 원본의 줄·문단 간격을 검사한다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::renderer::{hwpunit_to_px, DEFAULT_DPI};

fn body_nodes<'a>(node: &'a RenderNode, in_body: bool, out: &mut Vec<&'a RenderNode>) {
    let in_body = match node.node_type {
        RenderNodeType::Body { .. } => true,
        RenderNodeType::FootnoteArea => false,
        _ => in_body,
    };
    if in_body {
        out.push(node);
    }
    for child in &node.children {
        body_nodes(child, in_body, out);
    }
}

#[test]
fn saved_note_marker_restart_keeps_the_following_table_after_owned_spacing() {
    let core = DocumentCore::from_bytes(include_bytes!(
        "../../pdf/issue6812-multiple-square-pictures-print/multiple-square-pictures.hwpx"
    ))
    .expect("독립 Print와 같은 저장 입력");
    assert_eq!(core.page_count(), 11, "독립 Print 전체 쪽수와 내용 보존");
    let document = core.document();
    let paragraphs = &document.sections[0].paragraphs;
    let (table_owner, table_control) = paragraphs
        .iter()
        .enumerate()
        .find_map(|(owner, paragraph)| {
            paragraph
                .controls
                .iter()
                .enumerate()
                .find_map(|(control, value)| match value {
                    Control::Table(table)
                        if table.cells.iter().any(|cell| {
                            cell.paragraphs
                                .iter()
                                .any(|p| p.text.contains("돼지의 사회성 검증"))
                        }) =>
                    {
                        Some((owner, control))
                    }
                    _ => None,
                })
        })
        .expect("Print에서 확인한 그림과 설명의 표 소유");
    let previous_owner = table_owner.checked_sub(1).expect("표 앞 본문");
    let previous = &paragraphs[previous_owner];
    assert!(previous
        .controls
        .iter()
        .any(|control| matches!(control, Control::Footnote(_))));
    let last_line = previous.line_segs.last().expect("정상 저장 마지막 줄");
    let previous_style = &document.doc_info.para_shapes[previous.para_shape_id as usize];
    let table_style =
        &document.doc_info.para_shapes[paragraphs[table_owner].para_shape_id as usize];
    let paragraph_gap = previous_style.spacing_after + table_style.spacing_before;
    assert!(last_line.line_spacing > 0 && paragraph_gap > 0);
    // ParaShape 간격은 HWPUNIT의 두 배로 저장된다. 줄 간격은 원래 HWPUNIT이다.
    let owned_gap = hwpunit_to_px(last_line.line_spacing, DEFAULT_DPI)
        + hwpunit_to_px(paragraph_gap, DEFAULT_DPI) / 2.0;

    let tree = core.build_page_render_tree(1).expect("Print의 둘째 쪽");
    let mut nodes = Vec::new();
    body_nodes(&tree.root, false, &mut nodes);
    let tables: Vec<_> = nodes
        .iter()
        .filter(|node| {
            matches!(&node.node_type, RenderNodeType::Table(table)
                if table.para_index == Some(table_owner)
                    && table.control_index == Some(table_control)
                    && table.cell_context.is_none())
        })
        .collect();
    assert_eq!(
        tables.len(),
        1,
        "표 누락·중복으로 간격 검사를 우회하지 않는다"
    );
    let preceding_bottom = nodes
        .iter()
        .filter_map(|node| match &node.node_type {
            RenderNodeType::TextRun(run)
                if run.para_index == Some(previous_owner)
                    && run.cell_context.is_none()
                    && !run.text.trim().is_empty() =>
            {
                Some(node.bbox.y + node.bbox.height)
            }
            _ => None,
        })
        .reduce(f64::max)
        .expect("앞 본문 글줄이 실제로 같은 쪽에 표시되어야 한다");
    assert!(
        tables[0].bbox.y >= preceding_bottom + owned_gap,
        "표는 앞 글줄의 줄간격과 두 문단 사이 간격을 소비한 뒤에 놓여야 한다"
    );
}

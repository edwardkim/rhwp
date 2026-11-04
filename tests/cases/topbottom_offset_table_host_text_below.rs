//! 글줄보다 작은 양수 오프셋의 자리차지 표 뒤 host 글은 표 아래에서 시작한다.
//!
//! 한/글은 오프셋 안에 글줄이 들어가면 글을 표 위에 두고, 아니면 표 아래로 보낸다
//! (#7418 계약). 학교 급식 안내문(`vertOffset=1155` HU, 글줄 1500 HU)의 저장 줄
//! vpos 69438 = 문단 상단 + 오프셋 + 표 높이다. 종전 출력은 글을 문단 상단 흐름에
//! 두어 날짜 줄이 표 마지막 행 위에 겹쳤고, 조판은 그 줄 높이를 표 밴드 안에서
//! 소비해 다음 문단이 한 줄 위로 당겨졌다.
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
fn host_text_starts_below_table_when_offset_is_smaller_than_its_line() {
    let mut doc = HwpDocument::create_empty();
    doc.create_blank_document_native().unwrap();
    doc.create_table_native(0, 0, 0, 3, 2).unwrap();
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
    // 10pt 글줄(1000 HU)보다 작은 오프셋 600 HU.
    doc.set_table_properties_native(
        0,
        host_para,
        table_ctrl,
        r#"{"treatAsChar":false,"textWrap":"TopAndBottom","vertRelTo":"Para","vertAlign":"Top","vertOffset":600}"#,
    )
    .unwrap();
    let table_pos =
        doc.document().sections[0].paragraphs[host_para].control_text_positions()[table_ctrl];
    doc.insert_text_native(0, host_para, table_pos + 1, "2026. 9. 30.")
        .unwrap();
    doc.insert_text_native(0, host_para + 1, 0, "학교장")
        .unwrap();

    let tree = doc.build_page_render_tree(0).unwrap();
    let (mut tables, mut lines) = (Vec::new(), Vec::new());
    collect(&tree.root, &mut tables, &mut lines);
    let [table] = tables.as_slice() else {
        panic!("표 하나: {tables:?}");
    };
    let table_bottom = table.y + table.height;
    let host = lines
        .iter()
        .find(|(pi, _)| *pi == host_para)
        .expect("host 글줄")
        .1
        .clone();
    let next = lines
        .iter()
        .find(|(pi, _)| *pi == host_para + 1)
        .expect("다음 문단 글줄")
        .1
        .clone();
    assert!(
        host.y >= table_bottom - 0.5,
        "host 글줄은 표 아래에서 시작해야 한다: 표 하단 {table_bottom:.1}, 글줄 {host:?}"
    );
    assert!(
        next.y >= host.y + host.height - 0.5,
        "다음 문단은 host 글줄 아래에 와야 한다: host {host:?}, 다음 {next:?}"
    );
}

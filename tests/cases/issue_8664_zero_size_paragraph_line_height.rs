//! 글자 크기 0 으로 저장된 문단은 줄 높이를 만들지 않는다.
//!
//! 한컴 웹한글기안기 인쇄 PDF(`pdf/issue8664/zero_size_para_before_table-webhwp-print.pdf`)에서
//! 저장 줄이 없는 문단 `TOP` 과 글자처럼 취급한 표 사이에 글자 크기 0(장평 0) 글자 137 개만 든
//! 문단(줄간격 비율 160%)이 있어도 표는 그 문단이 없을 때와 같은 자리에 놓인다
//! (`TOP` 글자 윗변에서 표 안 `TABLE` 글자 윗변까지 84.00pt, 0 크기 문단을 뺀 문서도 84.00pt).
//!
//! 수정 전 rhwp 는 글자 크기 0 을 "크기 없음"으로 보고 대체값 12px 로 줄을 세워
//! 표를 한 줄(19.2px)만큼 아래로 밀었다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::{
    renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType},
    DocumentCore,
};

const SAMPLE: &str = "samples/issue8664/zero_size_para_before_table.hwp";

fn text_line_of<'a>(node: &'a RenderNode, para: usize) -> Option<&'a RenderNode> {
    if let RenderNodeType::TextLine(line) = &node.node_type {
        if line.para_index == Some(para) {
            return Some(node);
        }
    }
    node.children
        .iter()
        .find_map(|child| text_line_of(child, para))
}

fn table_bbox(node: &RenderNode) -> Option<BoundingBox> {
    if let RenderNodeType::Table(_) = &node.node_type {
        return Some(node.bbox.clone());
    }
    node.children.iter().find_map(table_bbox)
}

#[test]
fn zero_size_paragraph_adds_no_line_before_inline_table() {
    let bytes = std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE))
        .expect("글자 크기 0 문단 재현물");
    let core = DocumentCore::from_bytes(&bytes).unwrap();
    assert_eq!(core.page_count(), 1, "웹한글기안기 인쇄와 같은 쪽수");
    let source = &core.document().sections[0].paragraphs[1];
    assert!(
        !source.char_shapes.is_empty()
            && source.char_shapes.iter().all(|shape| {
                core.document().doc_info.char_shapes[shape.char_shape_id as usize].base_size == 0
            }),
        "재현물 문단 1 은 글자 크기 0 글자만 가진다"
    );

    let tree = core.build_page_render_tree(0).unwrap();
    let top = text_line_of(&tree.root, 0).expect("문단 0(TOP) 줄");
    let table = table_bbox(&tree.root).expect("글자처럼 취급한 표");

    // TOP 줄(10pt, 160%)의 줄 피치는 16pt = 21.33px 이다. 0 크기 문단이 줄을 만들지 않으면
    // 표 윗변은 TOP 줄 윗변에서 한 줄 피치와 표 바깥 여백만큼 떨어진다. 대체 크기 12px 로
    // 줄을 세우면 19.2px(12px × 160%)가 더해지므로 그 절반을 경계로 둔다.
    let one_line_pitch = 16.0 * 96.0 / 72.0;
    let gap = table.y - top.bbox.y;
    assert!(
        gap < one_line_pitch + 9.6,
        "0 크기 문단이 줄 높이를 만들어 표가 밀렸다: TOP→표 {gap:.2}px, 한 줄 피치 {one_line_pitch:.2}px"
    );

    // 0 크기 문단 뒤 문단(AFTER)도 표 아래에서 한 줄 피치 안에 이어진다.
    let after = text_line_of(&tree.root, 3).expect("문단 3(AFTER) 줄");
    let table_bottom = table.y + table.height;
    assert!(
        after.bbox.y >= table_bottom - 0.5 && after.bbox.y - table_bottom < one_line_pitch,
        "AFTER 줄은 표 바로 아래에 있다: 표 아래 {table_bottom:.2}px, AFTER {:.2}px",
        after.bbox.y
    );
    if let Some(zero) = text_line_of(&tree.root, 1) {
        assert!(
            zero.bbox.y <= table.y + 0.5,
            "0 크기 문단 줄은 표 위에서 시작한다"
        );
    }
}

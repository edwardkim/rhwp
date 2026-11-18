//! 검사 유형: 고정형 — 시행령 70쪽 표 하단과 그림 원점을 독립 Print의 ±2px로 보호한다.
//! 주석 갱신: 2026-11-18 07:17 KST.
//! 보정 전 Native 98.75022%(fbbfb55cfb), fresh WASM 98.75534%(058acd32a1, 같은 높이 코드).
//! 96 DPI·PDFium·2px raw이며, 최신 보정 소스의 전쪽 WASM 결과로 대신하지 않는다.
//! 한컴 2020 Hancom PDF Print 대조군은 그림 호스트의 앞 간격만 200→0 HU로 바꿔도
//! 전 74쪽·표선·그림 원점이 동일하다. 원본의 표 하단은 y=870.728px다.
//! 개인 글꼴을 읽지 않고 공개 원본의 쪽·개체 소속과 최종 상자를 검사한다.

use rhwp::{
    renderer::render_tree::{RenderNode, RenderNodeType},
    DocumentCore,
};

fn table(node: &RenderNode) -> Option<&RenderNode> {
    if matches!(&node.node_type, RenderNodeType::Table(meta)
        if meta.para_index == Some(719) && meta.control_index == Some(0))
    {
        return Some(node);
    }
    node.children.iter().find_map(table)
}
fn images<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    if matches!(node.node_type, RenderNodeType::Image(_)) {
        out.push(node);
    }
    for child in &node.children {
        images(child, out);
    }
}

#[test]
fn empty_picture_anchor_before_space_does_not_lengthen_the_table() {
    let input =
        include_bytes!("../../samples/issue6776/78494-virtual-convergence-industry-decree.hwpx");
    let mut core = DocumentCore::from_bytes(input).unwrap();
    core.set_hangul2024_compat(false);
    assert_eq!(core.page_count(), 74);
    let page = core.build_page_render_tree(69).unwrap();
    let table = table(&page.root).expect("같은 70쪽 원본 표");
    let mut pictures = Vec::new();
    images(table, &mut pictures);
    assert_eq!(pictures.len(), 1, "원래 그림을 이 표에 한 번만 보존한다");
    let picture = pictures[0];
    assert!(
        (picture.bbox.y - 398.284).abs() <= 2.0,
        "독립 그림 원점: {}",
        picture.bbox.y
    );
    assert!(
        (table.bbox.y + table.bbox.height - 870.728).abs() <= 2.0,
        "그림이 흡수한 호스트 앞 간격을 표 아래에 중복 계상하지 않는다: {}",
        table.bbox.y + table.bbox.height
    );
    assert!(
        picture.bbox.y + picture.bbox.height <= table.bbox.y + table.bbox.height,
        "표는 원래 그림을 담고 닫힌다"
    );
}

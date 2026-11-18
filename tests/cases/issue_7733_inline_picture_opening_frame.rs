//! 검사 유형: 관계형 — 첫 Common 상자와 그림 줄의 잉크·안여백을 대조한다.
//! 주석 갱신: 2026-11-18 00:06 KST.
//! 화학 원본 79쪽 보정 전 Native 92.22271%, 이전 fresh WASM 92.77094%다.
//! PDF 절대 좌표를 고정하지 않는다. 원본이 독립적으로 선언한 첫 상자와
//! 그림 줄 뒤의 간격이 다음 조각에 속하는지, 세 그림의 누락·중복이 없는지 검사한다.

use rhwp::model::control::Control;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::DocumentCore;

fn find_table(node: &RenderNode) -> Option<&RenderNode> {
    if matches!(&node.node_type, RenderNodeType::Table(meta)
        if meta.section_index == Some(4) && meta.para_index == Some(129)
            && meta.control_index == Some(0))
    {
        Some(node)
    } else {
        node.children.iter().find_map(find_table)
    }
}

#[test]
fn first_picture_ink_closes_the_original_opening_box_before_the_next_frame() {
    let mut core = DocumentCore::from_bytes(include_bytes!(
        "../../samples/issue6782/1480000-201900042-chemical-labeling-standards.hwp"
    ))
    .unwrap();
    core.set_hangul2024_compat(false);
    let Control::Table(source) = &core.document().sections[4].paragraphs[129].controls[0] else {
        panic!("원본 그림 표");
    };
    let body = source
        .cells
        .iter()
        .find(|cell| cell.row == 1 && cell.col == 0)
        .unwrap();
    let first = &body.paragraphs[0].line_segs[0];
    let next = &body.paragraphs[1].line_segs[0];
    assert_eq!(first.vertical_pos, 0);
    assert_eq!(next.vertical_pos, 0);
    assert!(first.line_spacing > 0);
    let padding = body.effective_padding(&source.padding);
    let prefix = source
        .cells
        .iter()
        .find(|cell| cell.row == 0 && cell.col == 0)
        .unwrap()
        .height;
    let declared = source.common.height;
    assert_eq!(
        declared,
        prefix
            + first.line_height as u32
            + u32::try_from(padding.top).unwrap()
            + u32::try_from(padding.bottom).unwrap(),
        "첫 줄의 잉크와 안여백이 원본 Common 상자를 독립적으로 닫는다"
    );
    let tree = core.build_page_render_tree(78).unwrap();
    let table = find_table(&tree.root).expect("79쪽 원본 표 첫 조각");
    let cell = table
        .children
        .iter()
        .find(|node| {
            matches!(&node.node_type, RenderNodeType::TableCell(meta)
            if meta.row == 1 && meta.col == 0)
        })
        .expect("그림 칸 첫 조각");
    assert_eq!(images(table), 1, "첫 그림이 앞 조각에 한 번만 속한다");
    let continued_tree = core.build_page_render_tree(79).unwrap();
    let continued = find_table(&continued_tree.root).expect("다음 쪽 그림 표 이어받기");
    assert_eq!(images(continued), 2, "나머지 두 그림의 누락·중복 방지");
    let actual = cell.bbox.y + cell.bbox.height - table.bbox.y;
    let expected = f64::from(declared) * 96.0 / 7200.0;
    assert!(
        (actual - expected).abs() < 1e-6,
        "그림 다음 줄의 간격은 첫 상자를 늘리지 않는다: actual={actual}, source={expected}"
    );
}

fn images(node: &RenderNode) -> usize {
    usize::from(matches!(node.node_type, RenderNodeType::Image(_)))
        + node.children.iter().map(images).sum::<usize>()
}

#[test]
fn local_zero_restart_without_a_matching_common_box_keeps_the_line_advance() {
    let mut core = DocumentCore::from_bytes(include_bytes!(
        "../../samples/issue6782/1480000-201900042-chemical-labeling-standards.hwp"
    ))
    .unwrap();
    core.set_hangul2024_compat(false);
    let mut original = core.document().clone();
    let Control::Table(source) = &mut original.sections[4].paragraphs[129].controls[0] else {
        panic!("원본 그림 표");
    };
    let body = source
        .cells
        .iter()
        .find(|cell| cell.row == 1 && cell.col == 0)
        .unwrap();
    let spacing = body.paragraphs[0].line_segs[0].line_spacing;
    let complete_slot =
        f64::from(source.common.height) * 96.0 / 7200.0 + f64::from(spacing) * 96.0 / 7200.0;
    // 한 HU라도 독립 상자와 닫히지 않으면 문단별 원점 0만으로 간격을 버리지 않는다.
    source.common.height += 1;
    core.set_document(original);
    let tree = core.build_page_render_tree(78).unwrap();
    let table = find_table(&tree.root).unwrap();
    let cell = table
        .children
        .iter()
        .find(|node| {
            matches!(&node.node_type, RenderNodeType::TableCell(meta)
            if meta.row == 1 && meta.col == 0)
        })
        .unwrap();
    let actual = cell.bbox.y + cell.bbox.height - table.bbox.y;
    assert!(
        (actual - complete_slot).abs() < 1e-6,
        "증명되지 않은 물리 끝으로 줄간격을 지우지 않는다: {actual} / {complete_slot}"
    );
}

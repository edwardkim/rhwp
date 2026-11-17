//! 검사 유형: 관계형 — 일반 라벨과 중첩 표가 공유하는 첫 조각의 선언 최소·정렬을 검사한다.
//! 주석 갱신: 2026-11-18 03:31 KST.
//! 시행령 36쪽 Native 97.04691%, 최신 fresh WASM 전쪽은 미검증이다.
//! 독립 한컴 2020 Hancom PDF Print의 같은 행은 다음 문단을 다음 쪽에 보존하며 라벨을 정렬한다.
//! PDF 절대 좌표를 고정하지 않는다. 같은 표의 원점을 1px 옮겨 수용 상자와 라벨의 상대 변화를 비교한다.
//! 개인 글꼴 설치에 의존하지 않으며 중첩 표의 내부 컷을 일반 라벨의 정렬 높이로 쓰지 않는다.

use rhwp::{
    model::{control::Control, table::VerticalAlign},
    renderer::render_tree::{RenderNode, RenderNodeType},
    DocumentCore,
};

fn table(node: &RenderNode) -> Option<&RenderNode> {
    if matches!(&node.node_type, RenderNodeType::Table(meta)
        if meta.para_index == Some(331) && meta.control_index == Some(0))
    {
        return Some(node);
    }
    node.children.iter().find_map(table)
}

fn first_label(node: &RenderNode) -> Option<f64> {
    if matches!(&node.node_type, RenderNodeType::TextLine(meta)
        if meta.para_index == Some(0) && meta.line_index == Some(0))
    {
        return Some(node.bbox.y);
    }
    node.children.iter().find_map(first_label)
}

fn frame(offset: u32, align: VerticalAlign) -> (f64, f64, f64) {
    let source = DocumentCore::from_bytes(include_bytes!(
        "../../samples/issue6776/78494-virtual-convergence-industry-decree.hwpx"
    ))
    .unwrap();
    let mut doc = source.document().clone();
    let Control::Table(parent) = &mut doc.sections[0].paragraphs[331].controls[0] else {
        panic!("원본 라벨·중첩 표의 부모 표")
    };
    assert_eq!((parent.row_count, parent.col_count), (5, 3));
    assert!(!parent.common.treat_as_char);
    parent.common.vertical_offset += offset;
    let label = parent
        .cells
        .iter_mut()
        .find(|c| c.row == 2 && c.col == 1)
        .unwrap();
    assert_eq!(label.vertical_align, VerticalAlign::Center);
    assert!(label
        .paragraphs
        .iter()
        .all(|p| p.line_segs.is_empty() && p.controls.is_empty()));
    label.vertical_align = align;
    let mut core = DocumentCore::new_empty();
    core.set_hangul2024_compat(false);
    core.set_document(doc);
    assert_eq!(core.page_count(), 74);
    let tree = core.build_page_render_tree(35).unwrap();
    let parent = table(&tree.root).expect("같은 36쪽의 부모 표");
    let label = parent.children.iter().find(|n|
        matches!(&n.node_type, RenderNodeType::TableCell(meta) if meta.row == 2 && meta.col == 1))
        .expect("같은 36쪽의 라벨 셀");
    (
        parent.bbox.y,
        label.bbox.y + label.bbox.height,
        first_label(label).unwrap(),
    )
}

#[test]
fn mixed_row_first_frame_keeps_its_page_limit_when_the_table_origin_moves() {
    let (before_top, before_bottom, _) = frame(0, VerticalAlign::Center);
    let (after_top, after_bottom, _) = frame(75, VerticalAlign::Center);
    assert!(
        (after_top - before_top - 1.0).abs() <= 96.0 / 7200.0,
        "원본 표의 원점만 1px 전진한다: {before_top} -> {after_top}"
    );
    assert!(
        (after_bottom - before_bottom).abs() <= 96.0 / 7200.0,
        "중첩 표가 있는 행도 같은 물리 하단을 소유한다: {before_bottom} -> {after_bottom}"
    );
}

#[test]
fn center_label_uses_its_selected_prefix_while_top_label_keeps_its_origin() {
    let (_, _, center_before) = frame(0, VerticalAlign::Center);
    let (_, _, center_after) = frame(75, VerticalAlign::Center);
    let (_, _, top_before) = frame(0, VerticalAlign::Top);
    let (_, _, top_after) = frame(75, VerticalAlign::Top);
    let centered = center_after - center_before;
    let top = top_after - top_before;
    assert!(
        (top - 1.0).abs() <= 96.0 / 7200.0,
        "위 정렬은 원점 이동만 따른다: {top}"
    );
    assert!(
        (centered - top * 0.5).abs() <= 96.0 / 7200.0,
        "같은 하단 안에서 가운데 라벨은 선택된 앞 문단만 정렬한다: center={centered}, top={top}"
    );
}

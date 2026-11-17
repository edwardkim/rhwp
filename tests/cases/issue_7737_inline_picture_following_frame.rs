//! 검사 유형: 관계형 — 저장 줄 없는 TAC 그림 뒤 간격의 paint와 표 조각 회계를 비교한다.
//! 주석 갱신: 2026-11-18 03:27 KST.
//! 시행령 19쪽 보정 전 Native 95.47612%, 최신 fresh WASM 전쪽은 미검증이다.
//! 독립 한컴 2020 Hancom PDF Print의 그림·후속 본문 위치는 이미 일치하며 하단선이 짧다.
//! PDF 절대 좌표를 고정하지 않고, 원본 그림 호스트의 줄간격 변경량을 양쪽 소비자에서 비교한다.
//! #6776의 TAC 그림 높이·표 간격 보정을 유지하며 개인 글꼴 설치에 의존하지 않는다.

use rhwp::{
    model::control::Control,
    renderer::render_tree::{RenderNode, RenderNodeType},
    DocumentCore,
};

fn table(node: &RenderNode, para: usize) -> Option<&RenderNode> {
    if matches!(&node.node_type, RenderNodeType::Table(meta)
        if meta.para_index == Some(para) && meta.control_index == Some(0))
    {
        return Some(node);
    }
    node.children.iter().find_map(|child| table(child, para))
}

fn following_line(node: &RenderNode) -> Option<f64> {
    if matches!(&node.node_type, RenderNodeType::TextLine(meta)
        if meta.para_index == Some(14) && meta.line_index == Some(0))
    {
        return Some(node.bbox.y);
    }
    node.children.iter().find_map(following_line)
}

fn frame_and_following_line(percent: i32) -> (f64, f64, f64) {
    let source = DocumentCore::from_bytes(include_bytes!(
        "../../samples/issue6776/78494-virtual-convergence-industry-decree.hwpx"
    ))
    .unwrap();
    let mut doc = source.document().clone();
    let Control::Table(parent) = &mut doc.sections[0].paragraphs[193].controls[0] else {
        panic!("원본 부모 표")
    };
    let owner = parent
        .cells
        .iter_mut()
        .find(|cell| cell.row == 6 && cell.col == 1)
        .unwrap();
    let Control::Table(child) = owner.paragraphs[0]
        .controls
        .iter_mut()
        .find(|control| matches!(control, Control::Table(_)))
        .unwrap()
    else {
        unreachable!()
    };
    let host = &mut child.cells[0].paragraphs[12];
    assert!(host.line_segs.is_empty());
    assert!(host.text.trim().is_empty());
    assert!(host.controls.iter().all(
        |control| matches!(control, Control::Picture(picture) if picture.common.treat_as_char)
    ));
    assert_eq!(host.controls.len(), 1);
    let mut style = doc.doc_info.para_shapes[host.para_shape_id as usize].clone();
    assert_eq!(style.line_spacing, 120);
    let em =
        f64::from(doc.doc_info.char_shapes[host.char_shapes[0].char_shape_id as usize].base_size)
            * 96.0
            / 7200.0;
    style.line_spacing = percent;
    style.line_spacing_v2 = percent as u32;
    style.raw_data = None;
    host.para_shape_id = doc.doc_info.para_shapes.len() as u16;
    doc.doc_info.para_shapes.push(style);
    let mut core = DocumentCore::new_empty();
    core.set_hangul2024_compat(false);
    core.set_document(doc);
    assert_eq!(core.page_count(), 74);
    let tree = core.build_page_render_tree(18).unwrap();
    let parent = table(&tree.root, 193).expect("같은 19쪽의 부모 표 조각");
    let cell = parent.children.iter().find(|node|
        matches!(&node.node_type, RenderNodeType::TableCell(meta) if meta.row == 6 && meta.col == 1))
        .expect("그림이 들어 있는 같은 원본 셀");
    (
        parent.bbox.y + parent.bbox.height,
        following_line(cell).expect("그림 뒤 원문 문단"),
        em,
    )
}

#[test]
fn picture_host_following_spacing_advances_the_owned_frame_and_paint_once() {
    let (before_bottom, before_text, em) = frame_and_following_line(120);
    let (after_bottom, after_text, _) = frame_and_following_line(160);
    let expected = em * 0.4;
    let painted = after_text - before_text;
    let reserved = after_bottom - before_bottom;
    assert!(
        (painted - expected).abs() <= 96.0 / 7200.0,
        "그림 뒤 본문이 호스트의 간격 변화만 한 번 소비한다: {painted} vs {expected}"
    );
    assert!(
        (reserved - painted).abs() <= 96.0 / 7200.0,
        "표 조각도 이미 paint한 그림 뒤 간격을 예약한다: reserved={reserved}, painted={painted}"
    );
}

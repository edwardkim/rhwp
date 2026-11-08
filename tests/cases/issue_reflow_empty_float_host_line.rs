//! 빈 자리차지 호스트의 양수 첫줄 들여쓰기도 줄 공간을 소유한다.
//!
//! 원본 `issue7702/curly_quote_latin_slot.hwpx`의 「경력사항」 표와 「발표 논문」
//! 사이에는 한컴 Print에서 빈 호스트 줄이 있다. 글자 배치의 들여쓰기 취소가 그 줄의
//! 점유를 없애지 않는다. 선언 종단 글꼴의 한 줄 높이를 최소 포함 관계로 검사한다.
//! 절대 좌표·SVG 해시·구현 helper의 반환값을 기대값으로 사용하지 않는다.
//! Native/fresh WASM 전쪽98.50387%, 실제 브라우저 Canvas2D98.25768%와 확대 판독
//! 기록은 `pr_7815_review_impl.md`의 호스트 줄 대조 절에 연결한다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

fn find_owned<'a>(
    node: &'a RenderNode,
    matches_owner: &impl Fn(&RenderNodeType) -> bool,
) -> Option<&'a RenderNode> {
    if matches_owner(&node.node_type) {
        return Some(node);
    }
    node.children
        .iter()
        .find_map(|child| find_owned(child, matches_owner))
}

#[test]
fn next_heading_clears_the_owned_empty_host_line_after_the_float() {
    let core = DocumentCore::from_bytes(include_bytes!(
        "../../samples/issue7702/curly_quote_latin_slot.hwpx"
    ))
    .expect("원본 로드");
    let doc = core.document();
    let paragraphs = &doc.sections[0].paragraphs;
    let heading = paragraphs
        .iter()
        .position(|para| para.text.split_whitespace().collect::<String>() == "발표논문")
        .expect("뒤 제목의 원본 소유");
    let host_index = heading.checked_sub(1).expect("앞 호스트");
    let host = &paragraphs[host_index];
    assert!(host.text.is_empty() && host.line_segs.is_empty());
    assert!(doc.doc_info.para_shapes[host.para_shape_id as usize].indent > 0);
    let (control_index, control) = host
        .controls
        .iter()
        .enumerate()
        .find(
            |(_, control)| matches!(control, Control::Table(table) if !table.common.treat_as_char),
        )
        .expect("독립 자리차지 표");
    let Control::Table(source_table) = control else {
        unreachable!()
    };
    let outer_bottom = f64::from(source_table.outer_margin_bottom) * 96.0 / 7200.0;
    let end_shape = host.char_shapes.last().expect("종단 글자 모양");
    let end_font_height =
        f64::from(doc.doc_info.char_shapes[end_shape.char_shape_id as usize].base_size) * 96.0
            / 7200.0;
    assert!(end_font_height > 0.0);

    let tree = core.build_page_render_tree(0).expect("원본 전체 한쪽");
    let table = find_owned(&tree.root, &|kind| {
        matches!(kind, RenderNodeType::Table(table)
            if table.section_index == Some(0)
                && table.para_index == Some(host_index)
                && table.control_index == Some(control_index)
                && table.cell_context.is_none())
    })
    .expect("호스트 소유 표");
    let title_line = find_owned(&tree.root, &|kind| {
        matches!(kind, RenderNodeType::TextLine(line)
            if line.section_index == Some(0) && line.para_index == Some(heading))
    })
    .expect("뒤 제목 소유 줄");
    let gap = title_line.bbox.y - (table.bbox.y + table.bbox.height);
    assert!(
        gap - outer_bottom >= end_font_height,
        "뒤 제목은 표 아래에 빈 호스트 글꼴 줄을 남긴다: gap={gap}, declared em={end_font_height}"
    );
}

#[test]
fn zero_indent_host_keeps_the_following_empty_paragraph_as_its_own_line() {
    let core = DocumentCore::from_bytes(include_bytes!(
        "../../samples/issue7702/curly_quote_latin_slot.hwpx"
    ))
    .expect("원본 로드");
    let doc = core.document();
    let paragraphs = &doc.sections[0].paragraphs;
    let heading = paragraphs
        .iter()
        .position(|para| para.text.split_whitespace().collect::<String>() == "경력사항")
        .expect("대조군 제목");
    let empty_index = heading.checked_sub(1).expect("독립 빈 문단");
    let host_index = empty_index.checked_sub(1).expect("앞 표 호스트");
    let host = &paragraphs[host_index];
    assert!(host.text.is_empty());
    assert!(paragraphs[empty_index].text.is_empty());
    assert_eq!(
        doc.doc_info.para_shapes[host.para_shape_id as usize].indent,
        0
    );
    let (control_index, control) = host
        .controls
        .iter()
        .enumerate()
        .find(
            |(_, control)| matches!(control, Control::Table(table) if !table.common.treat_as_char),
        )
        .expect("대조군 표");
    let Control::Table(source_table) = control else {
        unreachable!()
    };
    let outer_bottom = f64::from(source_table.outer_margin_bottom) * 96.0 / 7200.0;
    let tree = core.build_page_render_tree(0).expect("원본 렌더 트리");
    let table = find_owned(&tree.root, &|kind| {
        matches!(kind, RenderNodeType::Table(table)
            if table.section_index == Some(0)
                && table.para_index == Some(host_index)
                && table.control_index == Some(control_index)
                && table.cell_context.is_none())
    })
    .expect("대조군 호스트 표");
    let empty_line = find_owned(&tree.root, &|kind| {
        matches!(kind, RenderNodeType::TextLine(line)
            if line.section_index == Some(0) && line.para_index == Some(empty_index))
    })
    .expect("뒤 빈 문단 소유 줄");
    // 빈 호스트가 가짜 줄을 더하면 다음 독립 문단 앞에 한 줄 높이의 빈 밴드가 생긴다.
    // 바깥 여백은 원본에서 가져오며 허용오차는 소유 줄의 em에 비례한다.
    let extra_band = empty_line.bbox.y - table.bbox.y - table.bbox.height - outer_bottom;
    assert!(extra_band.abs() < empty_line.bbox.height / 32.0);
}

//! 독립 한컴 Print에서 확인한 쪽 끝 글줄과 위 캡션 그림의 흐름 관계.
//! #8014는 전체 쪽, #8039는 검사하는 5~6쪽을 Native와 fresh WASM으로 검증했다.
//! #8039 표지는 WASM 90% 미만이므로 전체 쪽수는 고정하지 않는다.
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::DocumentCore;

fn text_of(node: &RenderNode, result: &mut String) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        result.push_str(&run.text);
    }
    for child in &node.children {
        text_of(child, result);
    }
}

fn find_line<'a>(node: &'a RenderNode, needle: &str, found: &mut Vec<&'a RenderNode>) {
    if matches!(&node.node_type, RenderNodeType::TextLine(_)) {
        let mut text = String::new();
        text_of(node, &mut text);
        if text.contains(needle) {
            found.push(node);
        }
    }
    for child in &node.children {
        find_line(child, needle, found);
    }
}

fn find_nodes<'a>(
    node: &'a RenderNode,
    predicate: fn(&RenderNodeType) -> bool,
    found: &mut Vec<&'a RenderNode>,
) {
    if predicate(&node.node_type) {
        found.push(node);
    }
    for child in &node.children {
        find_nodes(child, predicate, found);
    }
}

#[test]
fn printed_table_tail_stays_with_table_and_next_table_starts_on_next_page() {
    let input = include_bytes!("../../mydocs/pr/assets/pr8319_evidence_alignment_20261027/official-inputs/2026년+7월+인구동향.hwp");
    let core = DocumentCore::from_bytes(input).expect("인구동향 HWP");
    assert_eq!(core.page_count(), 41, "독립 Print의 모든 쪽을 보존");
    let page = core.build_page_render_tree(13).unwrap();
    let next = core.build_page_render_tree(14).unwrap();
    let (mut notes, mut body, mut tables, mut next_heading) =
        (Vec::new(), Vec::new(), Vec::new(), Vec::new());
    find_line(&page.root, "주1) (당월 건수", &mut notes);
    find_nodes(
        &page.root,
        |k| matches!(k, RenderNodeType::Body { .. }),
        &mut body,
    );
    find_nodes(
        &page.root,
        |k| matches!(k, RenderNodeType::Table(t) if t.para_index == Some(200) && t.cell_context.is_none()),
        &mut tables,
    );
    find_line(&next.root, "- 출생 -", &mut next_heading);
    assert_eq!(
        (notes.len(), body.len(), tables.len(), next_heading.len()),
        (1, 1, 1, 1)
    );
    assert!(
        tables[0].bbox.y + tables[0].bbox.height <= notes[0].bbox.y,
        "표 다음 주석"
    );
    assert!(
        notes[0].bbox.y + notes[0].bbox.height <= body[0].bbox.y + body[0].bbox.height,
        "주석은 본문 안"
    );
    // 한컴 2020 Print PDF 14쪽, 96dpi: 주석 텍스트 상단 1024.27px.
    assert!(
        (notes[0].bbox.y - 1024.27).abs() <= 2.0,
        "독립 Print의 핵심 주석 위치"
    );
    // 한컴 2020 Print PDF 15쪽, 96dpi: 후속 제목 상단 90.56px.
    assert!(
        (next_heading[0].bbox.y - 90.56).abs() <= 2.0,
        "독립 Print의 후속 제목 위치"
    );
}

#[test]
fn printed_top_caption_picture_and_following_lines_keep_their_page() {
    let input = include_bytes!("../../mydocs/pr/assets/pr8319_evidence_alignment_20261027/official-inputs/pr8039-official.hwpx");
    let core = DocumentCore::from_bytes(input).expect("기계설비 발전 기본계획 HWPX");
    let page = core.build_page_render_tree(4).unwrap();
    let next = core.build_page_render_tree(5).unwrap();
    let (mut first, mut second, mut images, mut heading, mut body) =
        (Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new());
    find_line(&page.root, "건설, 건축, 에너지, 환경", &mut first);
    find_line(&page.root, "따라 건설산업기본법", &mut second);
    find_nodes(
        &page.root,
        |k| matches!(k, RenderNodeType::Image(_)),
        &mut images,
    );
    find_nodes(
        &page.root,
        |k| matches!(k, RenderNodeType::Body { .. }),
        &mut body,
    );
    find_line(&next.root, "기계설비산업 규모 및 관련 동향", &mut heading);
    assert_eq!(
        (
            first.len(),
            second.len(),
            images.len(),
            body.len(),
            heading.len()
        ),
        (1, 1, 1, 1, 1)
    );
    assert!(
        images[0].bbox.y + images[0].bbox.height <= first[0].bbox.y,
        "그림은 후속 본문 위"
    );
    assert!(
        first[0].bbox.y + first[0].bbox.height <= second[0].bbox.y,
        "후속 두 줄은 순서대로 배치"
    );
    assert!(
        second[0].bbox.y + second[0].bbox.height <= body[0].bbox.y + body[0].bbox.height,
        "둘째 줄은 본문 안"
    );
    // 한컴 2020 Print PDF 6쪽, 96dpi: 후속 제목 상단 75.30px.
    assert!(
        (heading[0].bbox.y - 75.30).abs() <= 2.0,
        "독립 Print의 후속 제목 위치"
    );
}

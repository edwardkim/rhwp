//! 관계형·고정형 — #7792 분할 표의 3쪽 소속, 컷의 열린 변과 닫힌 변, 내용 순서와 보존.
//! 주석 갱신 2026-11-28T03:36:12+09:00.
//! Native Skia·fresh WASM Print를 독립 한컴 PDF에 96 DPI·PDFium·2px raw 실루엣으로 대조했다.
//! 측정 코드 f3dae4a6483f62e5854417ef73cac111ec5dde10.
//! table-text-anchor-wrap · 기준 mydocs/pr/assets/issue7445/table-text-anchor-wrap/table_text_anchor_wrap-fresh-2024.pdf · 전체 3쪽.
//! 전쪽 최저 Native 99.74% / fresh WASM 100.00%.
//! 1쪽 Native 99.74% / fresh WASM 100.00%.
//! 2쪽 Native 100.00% / fresh WASM 100.00%.
//! 3쪽 Native 100.00% / fresh WASM 100.00%.
//! 전쪽 TSV와 대표 PNG는 mydocs/pr/assets/issues7789_7812/table-text-anchor-wrap/에 있다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::control::Control;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::DocumentCore;

const SOURCE: &str =
    "mydocs/pr/assets/issue7445/table-text-anchor-wrap/table_text_anchor_wrap.hwpx";

fn visit<'tree>(node: &'tree RenderNode, f: &mut impl FnMut(&'tree RenderNode)) {
    f(node);
    for child in &node.children {
        visit(child, f);
    }
}

fn table_fragment(root: &RenderNode) -> &RenderNode {
    let mut fragments = Vec::new();
    visit(root, &mut |node| {
        if matches!(&node.node_type, RenderNodeType::Table(table)
            if table.section_index == Some(0) && table.para_index == Some(0)
            && table.control_index == Some(2) && table.cell_context.is_none())
        {
            fragments.push(node);
        }
    });
    let [fragment] = fragments.as_slice() else {
        panic!("해당 쪽에서 원본 표의 조각 한 개를 그려야 한다");
    };
    fragment
}

fn text(root: &RenderNode) -> String {
    let mut words = String::new();
    visit(root, &mut |node| {
        if let RenderNodeType::TextRun(run) = &node.node_type {
            words.extend(
                run.display_or_text()
                    .chars()
                    .filter(|ch| !ch.is_whitespace()),
            );
        }
    });
    words
}

#[test]
fn anchored_table_fragment_keeps_its_opening_and_terminal_rows_in_order() {
    let mut core = DocumentCore::from_bytes(&std::fs::read(SOURCE).unwrap()).unwrap();
    core.set_hangul2024_compat(true);
    assert_eq!(core.page_count(), 3, "독립 한컴 Print의 전체 쪽수");
    let source = &core.document().sections[0].paragraphs[0];
    assert!(source.text.contains("약사법 시행령"));
    assert!(matches!(source.controls.get(2), Some(Control::Table(_))));

    let pages: Vec<_> = (0..3)
        .map(|page| core.build_page_render_tree(page).unwrap())
        .collect();
    let fragments: Vec<_> = pages
        .iter()
        .map(|page| table_fragment(&page.root))
        .collect();
    let content: Vec<_> = fragments.iter().map(|fragment| text(fragment)).collect();
    assert!(
        content[0].starts_with("업무정지처분을갈음하여부과하는과징금산정기준"),
        "표 머리와 1절의 시작"
    );
    assert!(content[1].starts_with("가.제조(수입)업무의정지처분을갈음하여"));
    assert!(content[1].ends_with("824만원30억원이상35억원미만1억2천만원이상"));
    let first_continued = "1억3천5백만원미만";
    assert!(!content[1].contains(first_continued));
    assert!(content[2].starts_with(first_continued));
    assert_eq!(
        content
            .iter()
            .map(|part| part.matches(first_continued).count())
            .sum::<usize>(),
        1
    );
    assert!(content[2].ends_with("분기별,월별또는일별매출금액을기준으로산출또는조정한다."));

    let mut title_bottom = None;
    visit(&pages[0].root, &mut |node| {
        if matches!(&node.node_type, RenderNodeType::TextRun(run)
            if run.cell_context.is_none() && run.text.contains("약사법 시행령"))
        {
            title_bottom = Some(node.bbox.y + node.bbox.height);
        }
    });
    assert!(title_bottom.expect("표 바깥의 제목") <= fragments[0].bbox.y);
}

fn full_width_edge(fragment: &RenderNode, y: f64) -> bool {
    let left = fragment.bbox.x;
    let right = left + fragment.bbox.width;
    fragment.children.iter().any(|child| {
        let RenderNodeType::Line(line) = &child.node_type else {
            return false;
        };
        let tolerance = line.style.width.max(0.5);
        (line.y1 - line.y2).abs() < 1e-6
            && (line.y1 - y).abs() <= tolerance
            && (line.x1 - left).abs() <= tolerance
            && (line.x2 - right).abs() <= tolerance
    })
}

#[test]
fn anchored_table_draws_only_the_owned_cut_edges() {
    let mut core = DocumentCore::from_bytes(&std::fs::read(SOURCE).unwrap()).unwrap();
    core.set_hangul2024_compat(true);
    let pages: Vec<_> = (0..3)
        .map(|page| core.build_page_render_tree(page).unwrap())
        .collect();
    let first = table_fragment(&pages[0].root);
    let middle = table_fragment(&pages[1].root);
    let last = table_fragment(&pages[2].root);
    assert!(
        !full_width_edge(first, first.bbox.y + first.bbox.height)
            && !full_width_edge(middle, middle.bbox.y),
        "독립 Print의 비고 셀은 첫 컷의 두 변이 모두 열려 있다"
    );
    assert_eq!(
        (
            full_width_edge(middle, middle.bbox.y + middle.bbox.height),
            full_width_edge(last, last.bbox.y),
        ),
        (true, true),
        "독립 Print의 금액 행은 마지막 이어받기 컷 양쪽에 전폭 경계가 있다"
    );
}

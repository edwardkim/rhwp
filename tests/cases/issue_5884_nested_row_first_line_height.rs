//! #5884: 중첩 표의 종료 호스트 줄과 분할 내용의 소유를 보존한다.
//! 원본 HWPX와 독립 Print 전 2쪽의 Native/fresh WASM 검증 후 관계를 검사한다.
//! Print의 예제 제목은 앞 조각, 계산식과 그림은 다음 조각에 속한다.

use rhwp::document_core::DocumentCore;
use rhwp::model::{control::Control, paragraph::Paragraph, table::Table};
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use std::{fs, path::Path};

const SAMPLE: &str = "samples/issue5884/nested_row_first_line_height.hwpx";
fn load_doc() -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let mut doc = DocumentCore::from_bytes(&fs::read(path).expect("원본 읽기")).expect("열기");
    doc.set_hangul2024_compat(false);
    doc
}
fn flat(text: &str) -> String {
    text.chars().filter(|c| !c.is_whitespace()).collect()
}
fn words(node: &RenderNode) -> String {
    let mut text = String::new();
    if let RenderNodeType::TextRun(run) = &node.node_type {
        text.push_str(&run.text);
    }
    for child in &node.children {
        text.push_str(&words(child));
    }
    flat(&text)
}
fn deepest_table<'a>(node: &'a RenderNode, needle: &str) -> Option<&'a RenderNode> {
    for child in &node.children {
        if let Some(found) = deepest_table(child, needle) {
            return Some(found);
        }
    }
    (matches!(node.node_type, RenderNodeType::Table(_)) && words(node).contains(needle))
        .then_some(node)
}
fn outer_table<'a>(node: &'a RenderNode, needle: &str) -> Option<&'a RenderNode> {
    if matches!(node.node_type, RenderNodeType::Table(_)) && words(node).contains(needle) {
        return Some(node);
    }
    node.children
        .iter()
        .find_map(|child| outer_table(child, needle))
}
fn images<'a>(node: &'a RenderNode, found: &mut Vec<&'a RenderNode>) {
    if matches!(node.node_type, RenderNodeType::Image(_)) {
        found.push(node);
    }
    for child in &node.children {
        images(child, found);
    }
}
fn image_cells<'a>(node: &'a RenderNode, found: &mut Vec<&'a RenderNode>) {
    let mut owned = Vec::new();
    images(node, &mut owned);
    if matches!(node.node_type, RenderNodeType::TableCell(_)) && owned.len() == 1 {
        found.push(node);
        return;
    }
    for child in &node.children {
        image_cells(child, found);
    }
}
fn bottom(node: &RenderNode) -> f64 {
    node.bbox.y + node.bbox.height
}
fn source_table_text(table: &Table) -> String {
    table
        .cells
        .iter()
        .flat_map(|cell| &cell.paragraphs)
        .map(|para| flat(&para.text))
        .collect()
}
fn source_host<'a>(para: &'a Paragraph, needle: &str) -> Option<&'a Paragraph> {
    for control in &para.controls {
        if let Control::Table(table) = control {
            for child in table.cells.iter().flat_map(|cell| &cell.paragraphs) {
                if let Some(found) = source_host(child, needle) {
                    return Some(found);
                }
            }
            if source_table_text(table).contains(needle) {
                return Some(para);
            }
        }
    }
    None
}
fn original_host_line_height(doc: &DocumentCore, needle: &str) -> f64 {
    let host = doc
        .document()
        .sections
        .iter()
        .flat_map(|section| &section.paragraphs)
        .find_map(|para| source_host(para, needle))
        .expect("원본 중첩 표 호스트");
    assert_eq!(host.line_segs.len(), 1, "원본에 저장된 단일 종료 줄");
    assert!(host.line_segs[0].line_height > 0);
    f64::from(host.line_segs[0].line_height) * 96.0 / 7200.0
}
#[test]
fn issue_5884_page_count_matches_hangul_2() {
    let doc = load_doc();
    assert_eq!(doc.page_count(), 2, "독립 Print 전체 쪽수");
    let all = (0..doc.page_count())
        .map(|page| words(&doc.build_page_render_tree(page).expect("쪽 트리").root))
        .collect::<String>();
    for needle in ["예1)", "예2)", "0.066", "0.273"] {
        assert_eq!(all.matches(needle).count(), 1, "{needle}: 내용 누락·중복");
    }
}
#[test]
fn issue_5884_example_fragments_keep_content_owners() {
    let doc = load_doc();
    let pages = (0..doc.page_count())
        .map(|page| doc.build_page_render_tree(page).expect("쪽 트리"))
        .collect::<Vec<_>>();
    let heading = pages
        .iter()
        .position(|page| words(&page.root).contains("예1)"))
        .expect("제목 조각");
    let body = pages
        .iter()
        .position(|page| words(&page.root).contains("0.066"))
        .expect("계산식 조각");
    assert_eq!(
        heading + 1,
        body,
        "Print의 제목과 계산식은 연속한 서로 다른 조각"
    );
    assert!(words(&pages[heading].root).contains("예2)"));
    assert!(!words(&pages[body].root).contains("예1)"));
    assert!(!words(&pages[body].root).contains("예2)"));
    let table = deepest_table(&pages[body].root, "0.066").expect("계산식의 중첩 표");
    assert!(words(table).contains("0.273"));
    let mut drawings = Vec::new();
    images(&pages[body].root, &mut drawings);
    assert_eq!(drawings.len(), 2, "원본 예제 그림 두 개 보존");
    let mut cells = Vec::new();
    image_cells(table, &mut cells);
    assert_eq!(cells.len(), 2);
    for cell in cells {
        assert!(words(cell).contains("직선보간법적용"));
        let mut owned = Vec::new();
        images(cell, &mut owned);
        let image = owned[0];
        assert!(image.bbox.x >= cell.bbox.x && image.bbox.y >= cell.bbox.y);
        assert!(bottom(image) <= bottom(cell));
        assert!(image.bbox.x + image.bbox.width <= cell.bbox.x + cell.bbox.width);
    }
}
#[test]
fn issue_5884_closing_host_line_separates_small_tables() {
    let doc = load_doc();
    let page = doc.build_page_render_tree(0).expect("제목 쪽");
    let formula = deepest_table(&page.root, "신규규격중량").expect("중량 계산 표");
    let examples = deepest_table(&page.root, "예1)").expect("예제 제목 표");
    let gap = examples.bbox.y - bottom(formula);
    let source_line = original_host_line_height(&doc, "신규규격중량");
    assert!(
        gap >= source_line,
        "종료 호스트 줄 공간 보존: gap={gap}, 원본 줄={source_line}"
    );
}
#[test]
fn issue_5884_terminal_host_line_belongs_to_parent_fragment() {
    let doc = load_doc();
    let page = doc
        .build_page_render_tree(doc.page_count() - 1)
        .expect("마지막 조각");
    let child = deepest_table(&page.root, "0.066").expect("계산식 중첩 표");
    let parent = outer_table(&page.root, "0.066").expect("부모 표 조각");
    assert_ne!(child.id, parent.id);
    let gap = bottom(parent) - bottom(child);
    let source_line = original_host_line_height(&doc, "예1)");
    assert!(
        gap >= source_line,
        "마지막 호스트 줄의 부모 공간 보존: gap={gap}, 원본 줄={source_line}"
    );
}
#[test]
fn issue_5884_shadowed_circle_preserves_raw_character_and_owner() {
    fn collect<'a>(node: &'a RenderNode, found: &mut Vec<&'a RenderNode>) {
        if let RenderNodeType::TextRun(run) = &node.node_type {
            if run.text.contains('\u{f06d}') {
                found.push(node);
            }
        }
        for child in &node.children {
            collect(child, found);
        }
    }
    let doc = load_doc();
    let mut found = Vec::new();
    let page = doc.build_page_render_tree(0).expect("원형 기호 쪽");
    collect(&page.root, &mut found);
    assert_eq!(found.len(), 1);
    let RenderNodeType::TextRun(run) = &found[0].node_type else {
        unreachable!()
    };
    assert_eq!(run.text, "\u{f06d}", "원본 문자와 주변 본문 런 분리");
    assert_eq!(
        run.display_text.as_deref(),
        Some("\u{1f53e}"),
        "독립 Print의 그림자 원형 기호"
    );
    assert!(run.cell_context.is_some(), "원본 셀 소유 보존");
}
#[test]
fn issue_5884_no_overflow_cell_lines() {
    let doc = load_doc();
    let _ = doc.take_overflow_cell_lines();
    let mut total = 0u64;
    for page in 0..doc.page_count() {
        doc.render_page_svg_native(page)
            .unwrap_or_else(|e| panic!("render page {}: {e:?}", page + 1));
        total += u64::from(doc.take_overflow_cell_lines());
    }
    assert_eq!(
        total, 0,
        "셀 측정이 내용을 담으면 쪽 밖 소실 줄이 없어야 한다"
    );
}

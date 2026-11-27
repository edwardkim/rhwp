//! 관계형·고정형 — #7812 48쪽에서 중첩 행과 마지막 부속서의 소속·순서·보존.
//! 주석 갱신 2026-11-28T04:50:06+09:00. 직전 파일 커밋 1c5b9928b96c1aba5654d62029a9f9aabd75efba 2026-11-28T03:38:16+09:00.
//! Native Skia·fresh WASM Print를 독립 한컴 PDF에 96 DPI·PDFium·2px raw 실루엣으로 대조했다.
//! 측정 코드 1e773b0314979e9198435eb4a6249e00430c6f58.
//! overfill-hwpx · 기준 pdf/table_giant_cell_overfill-hwpx-2024.pdf · 전체 48쪽.
//! 전쪽 최저 Native 98.25% / fresh WASM 98.25%.
//! 19쪽 Native 100.00% / fresh WASM 100.00%.
//! 29쪽 Native 100.00% / fresh WASM 100.00%.
//! 46쪽 Native 100.00% / fresh WASM 100.00%.
//! 47쪽 Native 100.00% / fresh WASM 100.00%.
//! 48쪽 Native 100.00% / fresh WASM 100.00%.
//! 전쪽 TSV와 대표 PNG는 mydocs/pr/assets/issues7789_7812/overfill-hwpx/에 있다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::diagnostics::layout_anomaly::{scan_page, AnomalyOptions};
use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

fn walk(node: &RenderNode, visit: &mut impl FnMut(&RenderNode)) {
    visit(node);
    for child in &node.children {
        walk(child, visit);
    }
}

fn text(node: &RenderNode) -> String {
    let mut output = String::new();
    walk(node, &mut |node| {
        if let RenderNodeType::TextRun(run) = &node.node_type {
            output.extend(
                run.display_or_text()
                    .chars()
                    .filter(|ch| !ch.is_whitespace()),
            );
        }
    });
    output
}

fn sample() -> DocumentCore {
    let source = std::fs::read("samples/table_giant_cell_overfill.hwpx").expect("원본 열기");
    let mut core = DocumentCore::from_bytes(&source).expect("원본 조판");
    core.set_hangul2024_compat(true);
    core
}

#[test]
fn overfill_attachments_are_preserved_once_across_the_final_fragments() {
    let core = sample();
    assert_eq!(core.page_count(), 48);
    let pages: Vec<_> = (38..48)
        .map(|page| {
            let tree = core.build_page_render_tree(page).expect("끝 조각의 쪽");
            let anomalies = scan_page(page, &tree.root, 48, &AnomalyOptions::default());
            assert!(
                anomalies.off_canvas.is_empty(),
                "용지 밖에 내용이 있다: {}",
                page + 1
            );
            assert!(
                anomalies.text_overlap.is_empty(),
                "끝 조각에서 본문이 겹친다: {}",
                page + 1
            );
            text(&tree.root)
        })
        .collect();
    for (page, unique_text) in [
        (7, "부속서Ⅱ특별히개조된운전장치"),
        (8, "부속서Ⅲ기술도서"),
        (9, "전기안전장치및안전회로"),
    ] {
        assert_eq!(pages[page].matches(unique_text).count(), 1, "{unique_text}");
        assert_eq!(
            pages
                .iter()
                .map(|s| s.matches(unique_text).count())
                .sum::<usize>(),
            1
        );
    }
}

#[test]
fn overfill_middle_fragment_preserves_nested_rows_and_closing_text() {
    let core = sample();
    for (page, first, after) in [
        (18, "전기설비의절연저항", "카가더이상움직이지않아야한다"),
        (28, "가)입석식카", "카의의자는이용자를안전하게"),
    ] {
        let tree = core.build_page_render_tree(page).expect("중간 조각");
        let mut fragments = Vec::new();
        walk(&tree.root, &mut |node| {
            if matches!(&node.node_type, RenderNodeType::Table(table)
                if table.para_index == Some(0) && table.cell_context.is_none())
            {
                fragments.push(text(node));
            }
        });
        let [fragment] = fragments.as_slice() else {
            panic!("본문 표 조각은 한 번 그린다: {}", page + 1);
        };
        let leading = fragment.find(first).expect("중첩 행의 시작");
        let trailing = fragment.find(after).expect("중첩 행 뒤 원본 글줄");
        assert!(leading < trailing);
        let anomalies = scan_page(page, &tree.root, 48, &AnomalyOptions::default());
        assert!(
            anomalies.overflow.is_empty(),
            "중간 조각이 본문을 넘는다: {}",
            page + 1
        );
    }
}

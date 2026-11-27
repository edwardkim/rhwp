//! 검사 유형: 관계형·고정형(독립 한컴 Print의 전체 쪽수).
//! 주석 갱신: 2026-11-24T10:16:17+09:00.
//! 2026-11-24 Windows 한컴 2020 Print와 전 2쪽의 쪽수가 같다. Native·fresh WASM 모두 1·2쪽 및 전쪽 최저 100.00%.
//! 측정 방법: 96 DPI·PDFium·색상 경계 보정 전 2px 실루엣.
//! 공개 HWP·독립 Print의 전체 TSV 및 대조 PNG: mydocs/pr/assets/stored-float-before-visible-line/.

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SOURCE: &[u8] =
    include_bytes!("../fixtures/stored_float_before_host/float-with-single-column-marker-a4.hwp");

fn collect_runs<'a>(
    node: &'a RenderNode,
    table: Option<&'a RenderNode>,
    runs: &mut Vec<(&'a str, &'a RenderNode, Option<&'a RenderNode>)>,
) {
    let table = match node.node_type {
        RenderNodeType::Table(_) => Some(node),
        _ => table,
    };
    if let RenderNodeType::TextRun(run) = &node.node_type {
        let text = run.display_or_text().trim();
        if !text.is_empty() {
            runs.push((text, node, table));
        }
    }
    for child in &node.children {
        collect_runs(child, table, runs);
    }
}

#[test]
fn stored_float_before_visible_host_keeps_three_following_rows_on_page_one() {
    let document = DocumentCore::from_bytes(SOURCE).unwrap();
    assert_eq!(document.page_count(), 2);

    let first = document.build_page_render_tree(0).unwrap();
    let second = document.build_page_render_tree(1).unwrap();
    for (root, expected) in [
        (
            &first.root,
            &[
                "Before float",
                "Small table",
                "Host line",
                "Row A",
                "Row B",
                "Row C",
            ][..],
        ),
        (&second.root, &["Row D", "After nested rows"][..]),
    ] {
        let mut runs = Vec::new();
        collect_runs(root, None, &mut runs);
        runs.sort_by(|a, b| a.1.bbox.y.total_cmp(&b.1.bbox.y));
        let actual: Vec<_> = runs.iter().map(|(text, _, _)| *text).collect();
        assert_eq!(actual, expected);
        for (_, run, table) in &runs {
            let table = table.expect("Each label belongs to a table");
            assert!(table.bbox.y <= run.bbox.y);
            assert!(run.bbox.y + run.bbox.height <= table.bbox.y + table.bbox.height);
        }
        for pair in runs.windows(2) {
            let earlier = pair[0].1.bbox;
            let later = pair[1].1.bbox;
            assert!(earlier.y + earlier.height <= later.y);
        }
        if let [(_, before, _), (_, _, Some(float)), (_, host, _), (_, _, Some(rows)), ..] =
            runs.as_slice()
        {
            assert!(before.bbox.y + before.bbox.height <= float.bbox.y);
            assert!(float.bbox.y + float.bbox.height <= host.bbox.y);
            assert!(host.bbox.y + host.bbox.height <= rows.bbox.y);
        }
    }
}

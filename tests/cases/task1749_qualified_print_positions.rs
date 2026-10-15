//! 독립 Print: pdf/task1749/saved_bounds_cumulative_page_break-hwp-2020.pdf.
//! code2d500a에서 전5쪽 Native 최저99.22885%, fresh WASM99.68462%와 확대 판독 완료.
//! 첫 결재 표·제목·마지막 표 뒤 본문의 소속과 Print 글자 원점을 보호한다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::{
    document_core::DocumentCore,
    renderer::render_tree::{RenderNode, RenderNodeType},
};

const INPUT: &str = "samples/task1749/saved_bounds_cumulative_page_break.hwp";

fn compact(text: &str) -> String {
    text.chars().filter(|c| !c.is_whitespace()).collect()
}

fn text(node: &RenderNode) -> String {
    let mut result = match &node.node_type {
        RenderNodeType::TextRun(run) => run.text.clone(),
        _ => String::new(),
    };
    for child in &node.children {
        result.push_str(&text(child));
    }
    result
}

fn check(page: u32, pi: usize, expected: &str, origin: (f64, f64), in_cell: bool) {
    fn collect<'a>(
        node: &'a RenderNode,
        cell: Option<&'a RenderNode>,
        expected: &str,
        matches: &mut Vec<(&'a RenderNode, Option<&'a RenderNode>)>,
    ) {
        let cell = if matches!(node.node_type, RenderNodeType::TableCell(_)) {
            Some(node)
        } else {
            cell
        };
        if matches!(node.node_type, RenderNodeType::TextLine(_))
            && compact(&text(node)).starts_with(expected)
        {
            matches.push((node, cell));
        }
        for child in &node.children {
            collect(child, cell, expected, matches);
        }
    }
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(INPUT);
    let core = DocumentCore::from_bytes(&std::fs::read(path).unwrap()).unwrap();
    assert_eq!(
        core.page_count(),
        5,
        "독립 Print와 같은5쪽이며 대상 내용의 소속도 검사한다"
    );
    let tree = core.build_page_render_tree(page - 1).unwrap();
    let mut found = Vec::new();
    collect(&tree.root, None, &compact(expected), &mut found);
    assert_eq!(found.len(), 1, "Print의 대상 글줄은 같은 쪽에 한 번만 있다");
    let (line, cell) = found[0];
    let first = line
        .children
        .iter()
        .find(|node| {
            matches!(&node.node_type, RenderNodeType::TextRun(run)
            if !run.text.trim().is_empty())
        })
        .expect("독립 Print 글줄의 첫 가시 run");
    let RenderNodeType::TextRun(run) = &first.node_type else {
        unreachable!()
    };
    assert_eq!(run.para_index, Some(pi), "원본 문단 소속을 유지한다");
    assert!(
        (first.bbox.x - origin.0).abs() <= 2.0
            && (first.bbox.y + run.baseline - origin.1).abs() <= 2.0,
        "{page}쪽 '{expected}': Print {origin:?}, 출력 ({}, {})",
        first.bbox.x,
        first.bbox.y + run.baseline
    );
    if in_cell {
        let cell = cell.expect("원본 표의 소유 셀");
        assert!(first.bbox.x >= cell.bbox.x);
        assert!(first.bbox.y >= cell.bbox.y);
        assert!(first.bbox.y + first.bbox.height <= cell.bbox.y + cell.bbox.height);
    } else {
        fn preceding_table(node: &RenderNode) -> Option<&RenderNode> {
            if matches!(&node.node_type, RenderNodeType::Table(meta)
                if meta.para_index == Some(52))
            {
                return Some(node);
            }
            node.children.iter().find_map(preceding_table)
        }
        let table = preceding_table(&tree.root).expect("앞에서 이어받은 원본 표 조각");
        assert!(first.bbox.y >= table.bbox.y + table.bbox.height);
        assert!(cell.is_none(), "뒤 본문을 앞 표의 셀에 넣지 않는다");
    }
}

#[test]
fn approval_table_label_keeps_its_print_origin() {
    check(1, 0, "문서번호", (79.7967529296875, 178.52490234375), true);
}

#[test]
fn body_title_keeps_its_print_origin_in_the_owned_table() {
    check(
        2,
        1,
        "3차 운영위원회 추진 결과 보고",
        (161.03272501627603, 176.12752278645834),
        true,
    );
}

#[test]
fn body_after_the_last_table_fragment_keeps_its_print_origin() {
    check(
        5,
        54,
        "ㅇ (청정아트) 키워드 선정은 현재 진행 중이며, 웹진 발간과 리포트 발간",
        (77.23813883463542, 343.7843017578125),
        false,
    );
}

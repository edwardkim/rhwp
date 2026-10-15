//! 독립 Print: pdf/pr_6088_6144/hancom2020/
//! pr_6088_6144_issue6025_employment_support_criteria_3232693_employment_support_criteria-2020.pdf.
//! code2d500a의4쪽 Native98.64612%·fresh WASM99.97712%와 확대 판독을 확인했다.
//! 1~3쪽은 WASM98% 미만이므로 그쪽의 px 핀은 추가하지 않는다.
//! 기대 좌표는 출력 run에 대응하는 독립 Print 문자 원점이며 소유·내용 검사도 유지한다.
//! 26번 run은 숫자로, 27번 run은 선행 공백으로 시작하므로 PDF의 대응 문자도 구별한다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::{
    document_core::DocumentCore,
    renderer::render_tree::{RenderNode, RenderNodeType},
};

const INPUT: &str = "samples/issue6025/3232693_employment_support_criteria.hwpx";

fn compact(s: &str) -> String {
    s.chars().filter(|c| !c.is_whitespace()).collect()
}

fn text(node: &RenderNode) -> String {
    let mut value = match &node.node_type {
        RenderNodeType::TextRun(run) => run.text.clone(),
        _ => String::new(),
    };
    for child in &node.children {
        value.push_str(&text(child));
    }
    value
}

fn check(page: u32, pi: usize, prefix: &str, expected: (f64, f64), table_owned: bool) {
    fn collect<'a>(
        node: &'a RenderNode,
        table: Option<&'a RenderNode>,
        cell: Option<&'a RenderNode>,
        prefix: &str,
        out: &mut Vec<(
            &'a RenderNode,
            Option<&'a RenderNode>,
            Option<&'a RenderNode>,
        )>,
    ) {
        let table = if matches!(node.node_type, RenderNodeType::Table(_)) {
            Some(node)
        } else {
            table
        };
        let cell = if matches!(node.node_type, RenderNodeType::TableCell(_)) {
            Some(node)
        } else {
            cell
        };
        if matches!(node.node_type, RenderNodeType::TextLine(_))
            && compact(&text(node)).starts_with(prefix)
        {
            out.push((node, table, cell));
        }
        for child in &node.children {
            collect(child, table, cell, prefix, out);
        }
    }
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(INPUT);
    let core = DocumentCore::from_bytes(&std::fs::read(path).unwrap()).unwrap();
    assert_eq!(
        core.page_count(),
        4,
        "독립 Print의4쪽과 마지막 표 내용·소속을 함께 보호한다"
    );
    let tree = core.build_page_render_tree(page - 1).unwrap();
    let mut found = Vec::new();
    collect(&tree.root, None, None, &compact(prefix), &mut found);
    assert_eq!(found.len(), 1, "해당 내용의 글줄은 같은 쪽에 한 번만 있다");
    let (line, table, cell) = found[0];
    let first = line.children.iter().find(|node|
        matches!(&node.node_type, RenderNodeType::TextRun(run) if !run.text.trim().is_empty())
    ).expect("첫 가시 글자 run");
    let RenderNodeType::TextRun(run) = &first.node_type else {
        unreachable!()
    };
    assert_eq!(run.para_index, Some(pi), "원본 문단 소속을 보존한다");
    let visible_x = first.bbox.x;
    let (_, visible_baseline) = run
        .style
        .script_draw_metrics(run.style.font_size, first.bbox.y + run.baseline);
    assert!(
        (visible_x - expected.0).abs() <= 2.0,
        "{page}쪽 '{prefix}': Print x={}, 출력 x={}",
        expected.0,
        visible_x
    );
    assert!(
        (visible_baseline - expected.1).abs() <= 2.0,
        "{page}쪽 '{prefix}': Print 기준선={}, 출력 기준선={} (y={}, baseline={}, font={}, size={}, superscript={}, subscript={})",
        expected.1,
        visible_baseline,
        first.bbox.y,
        run.baseline,
        run.style.font_family,
        run.style.font_size,
        run.style.superscript,
        run.style.subscript
    );
    if table_owned {
        assert!(matches!(&table.expect("수급 요건 표").node_type,
            RenderNodeType::Table(meta) if meta.para_index == Some(1)));
        let cell = cell.expect("해당 항목을 소유한 셀");
        assert!(first.bbox.x >= cell.bbox.x);
        assert!(first.bbox.y >= cell.bbox.y);
        assert!(first.bbox.y + first.bbox.height <= cell.bbox.y + cell.bbox.height);
    } else {
        assert!(
            table.is_none() && cell.is_none(),
            "문서 제목을 표 안에 넣지 않는다"
        );
    }
}

#[test]
fn final_table_items_keep_their_independent_print_origins() {
    check(
        4,
        0,
        "26. 직접일자리사업 참여자",
        (100.26565551757812, 112.9965108235677),
        true,
    );
    check(
        4,
        0,
        "27. 가습기살균제 피해자",
        (93.54929606119792, 177.24629720052084),
        true,
    );
}

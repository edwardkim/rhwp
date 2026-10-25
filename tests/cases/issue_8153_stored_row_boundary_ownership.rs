//! #8153: 원본 완결 행이 소유한 실제 경계를 짧은 보조 폭으로 이동시키지 않는다.
//! 독립 Print368쪽과 색상 대조군에서 확인한 같은 실제 열 경계의 정렬 관계다.
#![cfg(not(target_arch = "wasm32"))]
use rhwp::{
    renderer::render_tree::{RenderNode, RenderNodeType},
    DocumentCore,
};

fn nodes<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    out.push(node);
    for child in &node.children {
        nodes(child, out);
    }
}

fn check(extension: &str) {
    let input = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("samples/2025 행정업무운영 편람(최종).{extension}"));
    let mut core = DocumentCore::from_bytes(&std::fs::read(input).unwrap()).unwrap();
    core.set_hangul2024_compat(true);
    let tree = core.build_page_render_tree(367).unwrap();
    let mut all = Vec::new();
    nodes(&tree.root, &mut all);
    let tables: Vec<_> = all
        .into_iter()
        .filter(|n| {
            matches!(&n.node_type,
        RenderNodeType::Table(t) if t.row_count==12 && t.col_count==25)
        })
        .collect();
    assert_eq!(tables.len(), 1, "관인·서명 생략 표시의 원본 표");
    let mut all = Vec::new();
    nodes(tables[0], &mut all);
    let cell = |row, col| {
        let found: Vec<_> = all
            .iter()
            .copied()
            .filter(|n| {
                matches!(&n.node_type,
            RenderNodeType::TableCell(c) if c.row==row && c.col==col)
            })
            .collect();
        assert_eq!(found.len(), 1, "실제 셀 소유 {row}/{col}");
        found[0]
    };
    for (upper_row, col) in [(2, 14), (3, 20)] {
        let upper = cell(upper_row, col);
        let lower = cell(10, col);
        assert!(
            (upper.bbox.x - lower.bbox.x).abs() <= 96.0 / 7200.0,
            "{extension}: 독립 Print의 같은 경계 {col}열을 보존한다: 위 {}, 아래 {}",
            upper.bbox.x,
            lower.bbox.x
        );
        assert!(lower.bbox.y >= upper.bbox.y + upper.bbox.height);
    }
    let arrow = cell(10, 12);
    let following = cell(10, 14);
    assert!(
        (arrow.bbox.x + arrow.bbox.width - following.bbox.x).abs() <= 96.0 / 7200.0,
        "복구한 셀 폭과 다음 셀의 시작은 같은 경계를 소비한다"
    );
}
#[test]
fn hwp_stored_complete_rows_own_the_lower_diagram_boundaries() {
    check("hwp");
}
#[test]
fn hwpx_stored_complete_rows_own_the_lower_diagram_boundaries() {
    check("hwpx");
}

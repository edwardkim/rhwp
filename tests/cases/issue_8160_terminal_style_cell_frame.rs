//! 동일 HWPX의 독립 2024 Print 26쪽에서 두 셀 문구의 기준선과 소유를 확인한다.
//! 가시 글자 8pt와 종단 빈 run 10pt가 공존하는 정상 저장 줄이다.
//! Native261/fresh WASM077 전쪽 검증과 26쪽 직접 판독 후 등록한다.
//! 증적: mydocs/pr/assets/issue_8160/semanticist21-rebase-20261026/README.md
#![cfg(not(target_arch = "wasm32"))]

use rhwp::{
    renderer::{
        font_environment::FontEnvironment,
        render_tree::{RenderNode, RenderNodeType},
    },
    DocumentCore,
};
fn nodes<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    out.push(node);
    for child in &node.children {
        nodes(child, out);
    }
}
#[test]
fn terminal_style_preserves_printed_cell_baselines_and_source_ownership() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut core = DocumentCore::from_bytes(
        &std::fs::read(root.join("samples/hwp3-sample16-hwp5.hwpx")).unwrap(),
    )
    .unwrap();
    core.set_hangul2024_compat(true);
    core.set_font_environment(Some(FontEnvironment::from_json(r#"{"id":"terminal-style-independent-print","substitutions":{"한양신명조":"한양신명조","휴먼명조":"휴먼명조","HCI Poppy":"HCI Poppy"},"trueTypeFaces":["휴먼명조"],"glyphFallbacks":{"휴먼명조":["Haansoft Batang"]}}"#).unwrap())).unwrap();
    assert_eq!(core.page_count(), 64);
    let tree = core.build_page_render_tree(25).unwrap();
    let mut all = Vec::new();
    nodes(&tree.root, &mut all);
    let tables: Vec<_> = all.iter().filter(|node| matches!(&node.node_type,
        RenderNodeType::Table(table) if table.para_index == Some(490) && table.control_index == Some(0))).collect();
    assert_eq!(tables.len(), 1, "원본 표의 누락·중복 없음");
    let cells: Vec<_> = tables[0]
        .children
        .iter()
        .filter(|node| {
            matches!(&node.node_type,
        RenderNodeType::TableCell(cell) if cell.model_cell_index == Some(26))
        })
        .collect();
    assert_eq!(cells.len(), 1, "원본 26번 셀의 누락·중복 없음");
    let cell = cells[0].bbox;
    let mut selected = Vec::new();
    // 독립 Print의 1pt=4/3 CSSpx, 96dpi 원점이다. 구현의 산식에서 산출하지 않는다.
    for (text, expected_y) in [
        ("주센타와", 536.47998046875),
        ("교체활용", 547.5199788411459),
    ] {
        let found: Vec<_> = all
            .iter()
            .filter_map(|node| match &node.node_type {
                RenderNodeType::TextRun(run) if run.display_or_text() == text => Some((*node, run)),
                _ => None,
            })
            .collect();
        assert_eq!(found.len(), 1, "{text}: 누락·중복 없음");
        let (node, run) = found[0];
        let owner = run.cell_context.as_ref().expect("원본 표 셀 소유");
        assert!(!owner.in_textbox);
        assert_eq!(owner.parent_para_index, 490);
        let y = node.bbox.y + run.baseline;
        println!("{text}: actual baseline={y}, independent Print={expected_y}");
        assert!(
            (y - expected_y).abs() <= 2.0,
            "{text}: 독립 Print 기준선 ±2px, actual={y}, expected={expected_y}"
        );
        assert!(
            (node.bbox.x - 633.1199951171875).abs() <= 2.0,
            "독립 Print 가로 원점"
        );
        assert!(
            node.bbox.x >= cell.x && node.bbox.x + node.bbox.width <= cell.x + cell.width,
            "가시 문구의 가로 범위가 원본 셀 안에 있음"
        );
        assert!(
            y >= cell.y && y <= cell.y + cell.height,
            "기준선이 원본 셀 안에 있음"
        );
        selected.push((node, run));
    }
    let first = selected[0].1.cell_context.as_ref().unwrap();
    let second = selected[1].1.cell_context.as_ref().unwrap();
    let cell_path = |owner: &rhwp::renderer::layout::CellContext| {
        owner
            .path
            .iter()
            .map(|entry| (entry.control_index, entry.cell_index))
            .collect::<Vec<_>>()
    };
    assert_eq!(cell_path(first), vec![(0, 26)], "원본 26번 셀");
    assert_eq!(cell_path(first), cell_path(second), "같은 원본 셀");
    assert_eq!(first.path[0].cell_para_index, 0);
    assert_eq!(second.path[0].cell_para_index, 1);
    assert!(
        selected[0].0.bbox.y + selected[0].1.baseline
            < selected[1].0.bbox.y + selected[1].1.baseline,
        "위아래 문단 순서"
    );
    assert_ne!(
        selected[0].1.para_index, selected[1].1.para_index,
        "서로 다른 원본 문단"
    );
}
